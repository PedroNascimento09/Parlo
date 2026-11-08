use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::batch::NewBatch;

const STABLE_POLLS: u32 = 2;
const MIN_AGE: Duration = Duration::from_millis(500);
const MAX_FILE_BYTES: u64 = 512 * 1024 * 1024;
const DEFAULT_INTERVAL: u64 = 2;
const EXTENSIONS: [&str; 11] = [
    "mp4", "mkv", "mov", "webm", "avi", "m4v", "wav", "mp3", "m4a", "flac", "ogg",
];

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileKey {
    pub volume: u64,
    pub index: u64,
    pub size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claim {
    pub key: FileKey,
    pub path: String,
    pub batch: String,
    pub at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchFolder {
    pub id: String,
    pub path: String,
    pub from: String,
    pub targets: Vec<String>,
    pub timing: String,
    pub interval_seconds: u64,
    pub keep_background: bool,
}

#[derive(Deserialize)]
pub struct NewWatch {
    pub path: String,
    pub from: String,
    pub targets: Vec<String>,
    #[serde(default = "default_timing")]
    pub timing: String,
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
    #[serde(default)]
    pub keep_background: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct WatchConfig {
    #[serde(default)]
    folders: Vec<WatchFolder>,
}

#[derive(Clone, Debug, Serialize, Default)]
struct FolderStatus {
    last_scan: Option<u64>,
    last_error: Option<String>,
    pending: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScanResult {
    pub folder: String,
    pub ingested: usize,
    pub skipped: usize,
    pub deferred: usize,
}

#[derive(Clone, Copy)]
struct Pending {
    modified: Option<SystemTime>,
    seen: u32,
    first: Instant,
}

static STATUS: OnceLock<Mutex<HashMap<String, FolderStatus>>> = OnceLock::new();
static PENDING: OnceLock<Mutex<HashMap<(String, FileKey), Pending>>> = OnceLock::new();
static STARTED: OnceLock<()> = OnceLock::new();

#[cfg(windows)]
#[allow(dead_code)]
#[repr(C)]
struct FileTime {
    low: u32,
    high: u32,
}

#[cfg(windows)]
#[allow(dead_code)]
#[repr(C)]
struct ByHandleFileInformation {
    attributes: u32,
    creation: FileTime,
    access: FileTime,
    write: FileTime,
    volume: u32,
    size_high: u32,
    size_low: u32,
    links: u32,
    index_high: u32,
    index_low: u32,
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetFileInformationByHandle(
        handle: *mut core::ffi::c_void,
        information: *mut ByHandleFileInformation,
    ) -> i32;
}

pub fn add(request: NewWatch) -> Result<WatchFolder, String> {
    let path = fs::canonicalize(Path::new(request.path.trim()))
        .map_err(|error| format!("the folder cannot be read: {error}"))?;
    if !path.is_dir() {
        return Err("the watched path is not a folder".to_string());
    }
    let path = path
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string();
    let from = request.from.trim().to_ascii_lowercase();
    let targets = clean_targets(&from, request.targets)?;
    crate::dub::check_timing(&request.timing, 1.25, 0.9)?;
    let interval_seconds = request.interval_seconds.clamp(1, 3_600);
    let id = folder_id(&path);
    let folder = WatchFolder {
        id: id.clone(),
        path,
        from,
        targets,
        timing: request.timing,
        interval_seconds,
        keep_background: request.keep_background,
    };
    let mut config = load()?;
    if let Some(existing) = config.folders.iter_mut().find(|item| item.id == id) {
        *existing = folder.clone();
    } else {
        config.folders.push(folder.clone());
    }
    save(&config)?;
    journal_dir(&id)?;
    Ok(folder)
}

pub fn remove(id: &str) -> Result<(), String> {
    let mut config = load()?;
    let before = config.folders.len();
    config.folders.retain(|folder| folder.id != id);
    if config.folders.len() == before {
        return Err("this folder is not watched".to_string());
    }
    save(&config)
}

pub fn list() -> Result<Vec<serde_json::Value>, String> {
    let config = load()?;
    let mut values = Vec::new();
    for folder in config.folders {
        let claims = read_claims(&folder.id).unwrap_or_default();
        let status = status(&folder.id);
        let state = if status.last_error.is_some() {
            "error"
        } else {
            "watching"
        };
        values.push(serde_json::json!({
            "id": folder.id,
            "path": folder.path,
            "from": folder.from,
            "targets": folder.targets,
            "timing": folder.timing,
            "interval_seconds": folder.interval_seconds,
            "keep_background": folder.keep_background,
            "state": state,
            "ingested": claims.len(),
            "pending": status.pending,
            "last_scan": status.last_scan,
            "last_error": status.last_error,
        }));
    }
    Ok(values)
}

pub fn scan_all() -> Result<Vec<ScanResult>, String> {
    let config = load()?;
    let mut results = Vec::new();
    let mut errors = Vec::new();
    for folder in config.folders {
        match scan_folder(&folder) {
            Ok(result) => results.push(result),
            Err(error) => errors.push(format!("{}: {error}", folder.path)),
        }
    }
    if errors.is_empty() {
        Ok(results)
    } else {
        Err(errors.join("; "))
    }
}

pub fn scan(id: &str) -> Result<ScanResult, String> {
    let config = load()?;
    let folder = config
        .folders
        .into_iter()
        .find(|folder| folder.id == id)
        .ok_or_else(|| "this folder is not watched".to_string())?;
    scan_folder(&folder)
}

pub fn start() {
    if STARTED.set(()).is_err() {
        return;
    }
    std::thread::spawn(|| {
        let mut last: HashMap<String, Instant> = HashMap::new();
        loop {
            if let Ok(config) = load() {
                for folder in &config.folders {
                    let due = last
                        .get(&folder.id)
                        .map(|at| at.elapsed() >= Duration::from_secs(folder.interval_seconds))
                        .unwrap_or(true);
                    if due {
                        let _ = scan_folder(folder);
                        last.insert(folder.id.clone(), Instant::now());
                    }
                }
                last.retain(|id, _| config.folders.iter().any(|folder| folder.id == *id));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}

fn scan_folder(folder: &WatchFolder) -> Result<ScanResult, String> {
    clear_error(&folder.id);
    let path = Path::new(&folder.path);
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => {
            set_error(
                &folder.id,
                format!("cannot read {}: {error}", path.display()),
            );
            return Err(format!("cannot read {}: {error}", path.display()));
        }
    };
    let mut claims = match read_claims(&folder.id) {
        Ok(claims) => claims,
        Err(error) => {
            set_error(&folder.id, error.clone());
            return Err(error);
        }
    };
    let mut pending = PENDING
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut result = ScanResult {
        folder: folder.id.clone(),
        ingested: 0,
        skipped: 0,
        deferred: 0,
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                set_error(&folder.id, format!("cannot read an entry: {error}"));
                continue;
            }
        };
        let candidate = entry.path();
        if !candidate.is_file() || !supported(&candidate) {
            continue;
        }
        let (mut file, key) = match open_identity(&candidate) {
            Ok(value) => value,
            Err(error) if is_busy(&error) => {
                result.deferred += 1;
                continue;
            }
            Err(error) => {
                set_error(
                    &folder.id,
                    format!("cannot inspect {}: {error}", candidate.display()),
                );
                continue;
            }
        };
        if key.size > MAX_FILE_BYTES {
            set_error(
                &folder.id,
                format!("{} is larger than the watch limit", candidate.display()),
            );
            continue;
        }
        if let Some(claim) = claims.get(&key) {
            if crate::batch::get(&claim.batch).is_err() {
                if let Ok(bytes) = read_bytes(&mut file, key.size) {
                    if let Err(error) = ensure_batch(folder, claim, &bytes) {
                        set_error(&folder.id, error);
                    }
                }
            }
            result.skipped += 1;
            continue;
        }
        let id = batch_id(&folder.id, &key);
        if crate::batch::get(&id).is_ok() {
            let claim = Claim {
                key: key.clone(),
                path: candidate
                    .to_string_lossy()
                    .trim_start_matches(r"\\?\")
                    .to_string(),
                batch: id,
                at: now(),
            };
            if let Err(error) = append_claim(&folder.id, &claim) {
                set_error(&folder.id, error);
            } else {
                claims.insert(key, claim);
            }
            result.skipped += 1;
            continue;
        }
        let modified = fs::metadata(&candidate)
            .ok()
            .and_then(|meta| meta.modified().ok());
        let key_for_pending = (folder.id.clone(), key.clone());
        let pending_entry = pending.entry(key_for_pending.clone()).or_insert(Pending {
            modified,
            seen: 0,
            first: Instant::now(),
        });
        if pending_entry.modified == modified {
            pending_entry.seen += 1;
        } else {
            *pending_entry = Pending {
                modified,
                seen: 1,
                first: Instant::now(),
            };
        }
        if pending_entry.seen < STABLE_POLLS || pending_entry.first.elapsed() < MIN_AGE {
            result.deferred += 1;
            continue;
        }
        let bytes = match read_bytes(&mut file, key.size) {
            Ok(bytes) => bytes,
            Err(error) => {
                set_error(
                    &folder.id,
                    format!("cannot read {}: {error}", candidate.display()),
                );
                result.deferred += 1;
                continue;
            }
        };
        let claim = Claim {
            key: key.clone(),
            path: candidate
                .to_string_lossy()
                .trim_start_matches(r"\\?\")
                .to_string(),
            batch: id,
            at: now(),
        };
        if let Err(error) = append_claim(&folder.id, &claim) {
            set_error(&folder.id, error);
            result.deferred += 1;
            continue;
        }
        claims.insert(key, claim.clone());
        pending.remove(&key_for_pending);
        match ensure_batch(folder, &claim, &bytes) {
            Ok(()) => result.ingested += 1,
            Err(error) => set_error(&folder.id, error),
        }
    }
    let pending_left = pending.keys().filter(|(id, _)| id == &folder.id).count();
    drop(pending);
    mark_scanned(&folder.id, pending_left);
    Ok(result)
}

fn ensure_batch(folder: &WatchFolder, claim: &Claim, bytes: &[u8]) -> Result<(), String> {
    let existing = crate::batch::get(&claim.batch).ok();
    let mut batch = match existing {
        Some(batch) => batch,
        None => {
            crate::batch::create_with_id(
                NewBatch {
                    from: folder.from.clone(),
                    targets: folder.targets.clone(),
                    timing: folder.timing.clone(),
                    fastest: 1.25,
                    slowest: 0.9,
                    keep_background: folder.keep_background,
                },
                claim.batch.clone(),
            )?;
            crate::batch::get(&claim.batch)?
        }
    };
    if batch.items.is_empty() {
        let name = Path::new(&claim.path)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("dropped-file");
        crate::batch::add_file(&claim.batch, name, bytes)?;
        batch = crate::batch::get(&claim.batch)?;
    }
    if batch.state == crate::batch::DRAFT {
        crate::batch::start(&claim.batch)?;
    }
    Ok(())
}

fn read_bytes(file: &mut File, expected: u64) -> Result<Vec<u8>, String> {
    file.seek(SeekFrom::Start(0))
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 != expected {
        return Err("the file changed while it was being read".to_string());
    }
    Ok(bytes)
}

fn supported(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| EXTENSIONS.contains(&value.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn clean_targets(from: &str, targets: Vec<String>) -> Result<Vec<String>, String> {
    if targets.is_empty() {
        return Err("choose at least one target language".to_string());
    }
    if targets.len() > 8 {
        return Err("a folder takes at most 8 target languages".to_string());
    }
    let mut seen = std::collections::HashSet::new();
    let mut cleaned = Vec::new();
    for target in targets {
        let target = target.trim().to_ascii_lowercase();
        if !seen.insert(target.clone()) {
            return Err(format!("the language {target} appears twice"));
        }
        crate::config::check_pair(from, &target)?;
        cleaned.push(target);
    }
    Ok(cleaned)
}

fn folder_id(path: &str) -> String {
    format!("w{:016x}", hash(path))
}

fn batch_id(folder: &str, key: &FileKey) -> String {
    format!(
        "w{}",
        hash(&format!(
            "{}:{}:{}:{}",
            folder, key.volume, key.index, key.size
        ))
    )
}

fn hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn config_path() -> PathBuf {
    crate::config::data_root().join("watch.json")
}

fn load() -> Result<WatchConfig, String> {
    let path = config_path();
    if !path.is_file() {
        return Ok(WatchConfig::default());
    }
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn save(config: &WatchConfig) -> Result<(), String> {
    let root = crate::config::data_root();
    fs::create_dir_all(&root)
        .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
    let path = config_path();
    let text = serde_json::to_string_pretty(config)
        .map_err(|error| format!("cannot encode the watch folders: {error}"))?;
    fs::write(&path, text).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn journal_dir(id: &str) -> Result<PathBuf, String> {
    let root = crate::config::data_root().join("watch").join(id);
    fs::create_dir_all(&root)
        .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
    Ok(root)
}

fn journal_path(id: &str) -> Result<PathBuf, String> {
    Ok(journal_dir(id)?.join("journal.jsonl"))
}

fn read_claims(id: &str) -> Result<HashMap<FileKey, Claim>, String> {
    read_claims_at(&journal_path(id)?)
}

fn read_claims_at(path: &Path) -> Result<HashMap<FileKey, Claim>, String> {
    if !path.is_file() {
        return Ok(HashMap::new());
    }
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut claims = HashMap::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Claim>(line) {
            Ok(claim) => {
                claims.insert(claim.key.clone(), claim);
            }
            Err(_) => continue,
        }
    }
    Ok(claims)
}

fn append_claim(id: &str, claim: &Claim) -> Result<(), String> {
    append_claim_at(&journal_path(id)?, claim)
}

fn append_claim_at(path: &Path, claim: &Claim) -> Result<(), String> {
    let needs_newline = if path.is_file() {
        let length = fs::metadata(path).map_err(|error| error.to_string())?.len();
        if length == 0 {
            false
        } else {
            let mut file = File::open(path).map_err(|error| error.to_string())?;
            file.seek(SeekFrom::End(-1))
                .map_err(|error| error.to_string())?;
            let mut last = [0u8; 1];
            file.read_exact(&mut last)
                .map_err(|error| error.to_string())?;
            last[0] != b'\n'
        }
    } else {
        false
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    if needs_newline {
        file.write_all(b"\n").map_err(|error| error.to_string())?;
    }
    let line = serde_json::to_string(claim).map_err(|error| error.to_string())?;
    file.write_all(line.as_bytes())
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn status(id: &str) -> FolderStatus {
    STATUS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .ok()
        .and_then(|status| status.get(id).cloned())
        .unwrap_or_default()
}

fn set_error(id: &str, error: String) {
    if let Ok(mut status) = STATUS.get_or_init(|| Mutex::new(HashMap::new())).lock() {
        let entry = status.entry(id.to_string()).or_default();
        entry.last_error = Some(error);
    }
}

fn mark_scanned(id: &str, pending: usize) {
    if let Ok(mut status) = STATUS.get_or_init(|| Mutex::new(HashMap::new())).lock() {
        let entry = status.entry(id.to_string()).or_default();
        entry.last_scan = Some(now());
        entry.pending = pending;
    }
}

fn clear_error(id: &str) {
    if let Ok(mut status) = STATUS.get_or_init(|| Mutex::new(HashMap::new())).lock() {
        status.entry(id.to_string()).or_default().last_error = None;
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

fn default_timing() -> String {
    "natural".to_string()
}

fn default_interval() -> u64 {
    DEFAULT_INTERVAL
}

#[cfg(windows)]
fn open_identity(path: &Path) -> io::Result<(File, FileKey)> {
    use std::mem::MaybeUninit;
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;

    let file = OpenOptions::new().read(true).share_mode(0).open(path)?;
    let mut information = MaybeUninit::<ByHandleFileInformation>::uninit();
    let result =
        unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    let information = unsafe { information.assume_init() };
    Ok((
        file,
        FileKey {
            volume: u64::from(information.volume),
            index: (u64::from(information.index_high) << 32) | u64::from(information.index_low),
            size: (u64::from(information.size_high) << 32) | u64::from(information.size_low),
        },
    ))
}

#[cfg(not(windows))]
fn open_identity(path: &Path) -> io::Result<(File, FileKey)> {
    use std::os::unix::fs::MetadataExt;

    let file = File::open(path)?;
    let metadata = file.metadata()?;
    Ok((
        file,
        FileKey {
            volume: metadata.dev(),
            index: metadata.ino(),
            size: metadata.size(),
        },
    ))
}

fn is_busy(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::PermissionDenied
        || matches!(error.raw_os_error(), Some(32) | Some(33))
}

#[cfg(test)]
mod tests;
