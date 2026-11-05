use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::dub;

pub const DRAFT: &str = "draft";
pub const QUEUED: &str = "queued";
pub const RUNNING: &str = "running";
pub const DONE: &str = "done";
pub const FAILED: &str = "failed";

const MAX_TARGETS: usize = 8;
const MAX_ITEMS: usize = 100;

static NEXT: AtomicU64 = AtomicU64::new(0);
static RENDER: Mutex<()> = Mutex::new(());
static ACTIVE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static RECOVERED: OnceLock<()> = OnceLock::new();

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub source: String,
    pub title: String,
    pub to: String,
    pub state: String,
    pub stage: String,
    pub progress: f64,
    pub error: Option<String>,
    pub out: Option<String>,
    pub srt: Option<String>,
    pub seconds: f64,
    pub lines: usize,
    pub started_at: Option<u64>,
    pub ended_at: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Batch {
    pub id: String,
    pub from: String,
    pub targets: Vec<String>,
    pub timing: String,
    pub fastest: f64,
    pub slowest: f64,
    pub keep_background: bool,
    pub state: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub items: Vec<Item>,
}

#[derive(Deserialize)]
pub struct NewBatch {
    pub from: String,
    pub targets: Vec<String>,
    #[serde(default = "default_timing")]
    pub timing: String,
    #[serde(default = "default_fastest")]
    pub fastest: f64,
    #[serde(default = "default_slowest")]
    pub slowest: f64,
    #[serde(default)]
    pub keep_background: bool,
}

struct Rendered {
    out: String,
    srt: String,
    seconds: f64,
    lines: usize,
}

struct ActiveGuard(String);

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = active().lock() {
            active.remove(&self.0);
        }
    }
}

pub fn create(request: NewBatch) -> Result<Batch, String> {
    let (from, targets) = languages(&request.from, &request.targets)?;
    dub::check_timing(&request.timing, request.fastest, request.slowest)?;
    let stamp = now();
    let mut batch = Batch {
        id: new_id(),
        from,
        targets,
        timing: request.timing,
        fastest: request.fastest,
        slowest: request.slowest,
        keep_background: request.keep_background,
        state: DRAFT.to_string(),
        created_at: stamp,
        updated_at: stamp,
        items: Vec::new(),
    };
    write(&mut batch)?;
    Ok(batch)
}

pub fn add_file(id: &str, original: &str, bytes: &[u8]) -> Result<Batch, String> {
    if bytes.is_empty() {
        return Err("the file is empty".to_string());
    }
    let mut batch = read(id)?;
    if batch.state != DRAFT {
        return Err("this batch has already started".to_string());
    }
    if batch.items.len() + batch.targets.len() > MAX_ITEMS {
        return Err(format!("a batch takes at most {MAX_ITEMS} renders"));
    }
    let directory = directory(id)?;
    let number = batch.items.len() / batch.targets.len() + 1;
    let title = dub::safe_name(original);
    let source = format!("f{number}-{title}");
    let path = relative(&directory, &source)?;
    fs::write(&path, bytes).map_err(|error| format!("cannot store {}: {error}", path.display()))?;
    let mut added = Vec::new();
    for target in batch.targets.clone() {
        let item_id = format!("f{number}-{target}");
        let out = format!("{item_id}-{}", output_name(&title, &target));
        let srt = Path::new(&out)
            .with_extension("srt")
            .to_string_lossy()
            .to_string();
        added.push(Item {
            id: item_id,
            source: source.clone(),
            title: title.clone(),
            to: target,
            state: QUEUED.to_string(),
            stage: "waiting".to_string(),
            progress: 0.0,
            error: None,
            out: Some(out),
            srt: Some(srt),
            seconds: 0.0,
            lines: 0,
            started_at: None,
            ended_at: None,
        });
    }
    batch.items.extend(added);
    if let Err(error) = write(&mut batch) {
        fs::remove_file(&path).ok();
        return Err(error);
    }
    Ok(batch)
}

pub fn start(id: &str) -> Result<Batch, String> {
    let mut batch = read(id)?;
    if batch.items.is_empty() {
        return Err("the batch has no files yet".to_string());
    }
    if batch.state == RUNNING || batch.state == QUEUED {
        return Err("the batch is already running".to_string());
    }
    for item in &mut batch.items {
        if item.state == RUNNING {
            item.state = QUEUED.to_string();
            item.stage = "waiting".to_string();
            item.progress = 0.0;
        }
    }
    batch.state = QUEUED.to_string();
    write(&mut batch)?;
    spawn(id.to_string());
    Ok(batch)
}

pub fn retry(id: &str, item_id: &str) -> Result<Batch, String> {
    let mut batch = read(id)?;
    if batch.state == RUNNING || batch.state == QUEUED {
        return Err("wait for the batch to finish before retrying".to_string());
    }
    let directory = directory(id)?;
    let item = batch
        .items
        .iter_mut()
        .find(|item| item.id == item_id)
        .ok_or_else(|| format!("no item {item_id} in this batch"))?;
    if item.state != FAILED {
        return Err("only a failed render can be retried".to_string());
    }
    for name in [item.out.as_deref(), item.srt.as_deref()]
        .into_iter()
        .flatten()
    {
        if let Ok(path) = relative(&directory, name) {
            fs::remove_file(path).ok();
        }
    }
    item.state = QUEUED.to_string();
    item.stage = "waiting".to_string();
    item.progress = 0.0;
    item.error = None;
    item.seconds = 0.0;
    item.lines = 0;
    item.started_at = None;
    item.ended_at = None;
    batch.state = QUEUED.to_string();
    write(&mut batch)?;
    spawn(id.to_string());
    Ok(batch)
}

pub fn get(id: &str) -> Result<Batch, String> {
    recover();
    read(id)
}

pub fn list() -> Result<Vec<Batch>, String> {
    recover();
    let root = root()?;
    let mut batches = Vec::new();
    for entry in
        fs::read_dir(&root).map_err(|error| format!("cannot read {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if let Ok(batch) = read(&id) {
            batches.push(batch);
        }
    }
    batches.sort_by_key(|batch| std::cmp::Reverse(batch.created_at));
    Ok(batches)
}

pub fn report(id: &str) -> Result<String, String> {
    let batch = read(id)?;
    let done = batch.items.iter().filter(|item| item.state == DONE).count();
    let failed = batch
        .items
        .iter()
        .filter(|item| item.state == FAILED)
        .count();
    let items: Vec<serde_json::Value> = batch
        .items
        .iter()
        .map(|item| {
            serde_json::json!({
                "id": item.id,
                "title": item.title,
                "to": item.to,
                "state": item.state,
                "error": item.error,
                "out": item.out,
                "srt": item.srt,
                "seconds": item.seconds,
                "lines": item.lines,
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "batch_id": batch.id,
        "state": batch.state,
        "from": batch.from,
        "targets": batch.targets,
        "created_at": batch.created_at,
        "finished": batch.state == DONE || batch.state == FAILED,
        "total": batch.items.len(),
        "done": done,
        "failed": failed,
        "items": items,
    }))
    .map_err(|error| format!("cannot encode the report: {error}"))
}

pub fn download(id: &str, item_id: &str) -> Result<(PathBuf, String), String> {
    let batch = read(id)?;
    let item = batch
        .items
        .iter()
        .find(|item| item.id == item_id)
        .ok_or_else(|| "this render is not in the batch".to_string())?;
    if item.state != DONE {
        return Err("this render is not finished".to_string());
    }
    let name = item
        .out
        .as_deref()
        .ok_or_else(|| "this render has no output".to_string())?;
    let path = relative(&directory(id)?, name)?;
    if !path.is_file() {
        return Err("the finished file is missing".to_string());
    }
    Ok((path, name.to_string()))
}

fn run(id: &str) {
    let mut batch = match read(id) {
        Ok(batch) => batch,
        Err(error) => {
            eprintln!("cannot read batch {id}: {error}");
            return;
        }
    };
    batch.state = RUNNING.to_string();
    batch.updated_at = now();
    if let Err(error) = write(&mut batch) {
        eprintln!("cannot save batch {id}: {error}");
    }
    for index in 0..batch.items.len() {
        if batch.items[index].state != QUEUED {
            continue;
        }
        let stamp = now();
        {
            let item = &mut batch.items[index];
            item.state = RUNNING.to_string();
            item.stage = "synthesizing".to_string();
            item.progress = 0.05;
            item.error = None;
            item.started_at = Some(stamp);
        }
        batch.updated_at = stamp;
        if let Err(error) = write(&mut batch) {
            eprintln!("cannot save batch {id}: {error}");
        }
        let result = {
            let _rendering = RENDER.lock().unwrap_or_else(|error| error.into_inner());
            render(&batch, index)
        };
        let stamp = now();
        {
            let item = &mut batch.items[index];
            item.ended_at = Some(stamp);
            match result {
                Ok(rendered) => {
                    item.state = DONE.to_string();
                    item.stage = "done".to_string();
                    item.progress = 1.0;
                    item.error = None;
                    item.out = Some(rendered.out);
                    item.srt = Some(rendered.srt);
                    item.seconds = rendered.seconds;
                    item.lines = rendered.lines;
                }
                Err(error) => {
                    item.state = FAILED.to_string();
                    item.stage = "stopped".to_string();
                    item.error = Some(error);
                }
            }
        }
        batch.updated_at = stamp;
        if let Err(error) = write(&mut batch) {
            eprintln!("cannot save batch {id}: {error}");
        }
    }
    let done = batch.items.iter().filter(|item| item.state == DONE).count();
    batch.state = if done > 0 || batch.items.is_empty() {
        DONE.to_string()
    } else {
        FAILED.to_string()
    };
    batch.updated_at = now();
    if let Err(error) = write(&mut batch) {
        eprintln!("cannot save batch {id}: {error}");
    }
    write_report(&batch);
}

fn render(batch: &Batch, index: usize) -> Result<Rendered, String> {
    let item = &batch.items[index];
    let directory = directory(&batch.id)?;
    let source = relative(&directory, &item.source)?;
    if !source.is_file() {
        return Err(format!("the saved source {} is missing", item.source));
    }
    let out_name = item
        .out
        .as_deref()
        .ok_or_else(|| "the render has no output name".to_string())?;
    let srt_name = item
        .srt
        .as_deref()
        .ok_or_else(|| "the render has no subtitle name".to_string())?;
    let out = relative(&directory, out_name)?;
    let srt = relative(&directory, srt_name)?;
    let work = directory.join(format!("{}-work", item.id));
    let report = directory.join(format!("{}-report.json", item.id));
    let result = dub::run(&dub::Request {
        source: &source,
        from: &batch.from,
        to: &item.to,
        out: Some(out),
        srt: Some(srt),
        cues: None,
        track: None,
        lines_dir: None,
        work: Some(work.clone()),
        report: Some(report.clone()),
        previous: None,
        retry: None,
        skip: None,
        keep_background: batch.keep_background,
        timing: &batch.timing,
        fastest: batch.fastest,
        slowest: batch.slowest,
    });
    fs::remove_dir_all(&work).ok();
    fs::remove_file(&report).ok();
    let result = result?;
    Ok(Rendered {
        out: out_name.to_string(),
        srt: srt_name.to_string(),
        seconds: result.seconds,
        lines: result.lines.len(),
    })
}

fn write_report(batch: &Batch) {
    let Ok(text) = report(&batch.id) else {
        return;
    };
    let Ok(directory) = directory(&batch.id) else {
        return;
    };
    let path = directory.join("report.json");
    if let Err(error) = fs::write(&path, text) {
        eprintln!("cannot write {}: {error}", path.display());
    }
}

fn recover() {
    if RECOVERED.set(()).is_err() {
        return;
    }
    let Ok(root) = root() else {
        return;
    };
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        let Ok(mut batch) = read(&id) else {
            continue;
        };
        if batch.state != RUNNING && batch.state != QUEUED {
            continue;
        }
        for item in &mut batch.items {
            if item.state == RUNNING {
                item.state = QUEUED.to_string();
                item.stage = "waiting".to_string();
                item.progress = 0.0;
                item.error = None;
                item.started_at = None;
            }
        }
        batch.state = QUEUED.to_string();
        if write(&mut batch).is_ok() {
            spawn(id);
        }
    }
}

fn spawn(id: String) {
    let mut active = active().lock().unwrap_or_else(|error| error.into_inner());
    if !active.insert(id.clone()) {
        return;
    }
    drop(active);
    std::thread::spawn(move || {
        let _guard = ActiveGuard(id.clone());
        run(&id);
    });
}

fn active() -> &'static Mutex<HashSet<String>> {
    ACTIVE.get_or_init(|| Mutex::new(HashSet::new()))
}

fn root() -> Result<PathBuf, String> {
    let root = crate::config::data_root().join("batches");
    fs::create_dir_all(&root)
        .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
    Ok(root)
}

fn directory(id: &str) -> Result<PathBuf, String> {
    if id.is_empty()
        || !id.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        return Err("the batch id is not valid".to_string());
    }
    let directory = root()?.join(id);
    fs::create_dir_all(&directory)
        .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    Ok(directory)
}

fn read(id: &str) -> Result<Batch, String> {
    let path = directory(id)?.join("manifest.json");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn write(batch: &mut Batch) -> Result<(), String> {
    let directory = directory(&batch.id)?;
    batch.updated_at = now();
    let text = serde_json::to_string_pretty(batch)
        .map_err(|error| format!("cannot encode the batch: {error}"))?;
    let path = directory.join("manifest.json");
    fs::write(&path, text).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn relative(directory: &Path, value: &str) -> Result<PathBuf, String> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
    {
        return Err("the batch contains an unsafe file name".to_string());
    }
    Ok(directory.join(value))
}

fn languages(from: &str, targets: &[String]) -> Result<(String, Vec<String>), String> {
    let from = from.trim().to_ascii_lowercase();
    if targets.is_empty() {
        return Err("choose at least one language".to_string());
    }
    if targets.len() > MAX_TARGETS {
        return Err(format!("a batch takes at most {MAX_TARGETS} languages"));
    }
    let mut seen = HashSet::new();
    let mut cleaned = Vec::new();
    for target in targets {
        let target = target.trim().to_ascii_lowercase();
        if !seen.insert(target.clone()) {
            return Err(format!("the language {target} appears twice"));
        }
        crate::config::check_pair(&from, &target)?;
        cleaned.push(target);
    }
    Ok((from, cleaned))
}

fn output_name(name: &str, target: &str) -> String {
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("audio");
    match path.extension().and_then(|value| value.to_str()) {
        Some(extension) if !extension.is_empty() => format!("{stem}.{target}.{extension}"),
        _ => format!("{stem}.{target}"),
    }
}

fn new_id() -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();
    format!("b{stamp}-{}", NEXT.fetch_add(1, Ordering::Relaxed))
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

fn default_fastest() -> f64 {
    1.25
}

fn default_slowest() -> f64 {
    0.9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_names_keep_the_extension() {
        assert_eq!(output_name("clip.mp4", "en"), "clip.en.mp4");
        assert_eq!(output_name("clip", "en"), "clip.en");
    }

    #[test]
    fn batch_languages_are_unique_and_differ_from_the_source() {
        assert!(languages("pt", &["en".to_string(), "en".to_string()]).is_err());
        assert!(languages("pt", &["pt".to_string()]).is_err());
        assert_eq!(
            languages("pt", &["en".to_string(), "es".to_string()]).unwrap(),
            ("pt".to_string(), vec!["en".to_string(), "es".to_string()])
        );
    }

    #[test]
    fn batch_file_names_cannot_leave_the_batch_folder() {
        let directory = Path::new("C:/tmp/batch");
        assert!(relative(directory, "../secret").is_err());
        assert!(relative(directory, "a/b").is_err());
        assert!(relative(directory, "clip.mp4").is_ok());
    }
}
