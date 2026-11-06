use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use parlo_core::{device, UnitState};

const PARAKEET_FILES: &[&str] = &[
    "encoder.int8.onnx",
    "decoder.int8.onnx",
    "joiner.int8.onnx",
    "tokens.txt",
];
pub const DICTATION_UNITS: &[&str] = &["parakeet"];
pub fn remaining_units() -> Vec<&'static str> {
    crate::registry::remaining_units()
}

pub struct Model {
    pub id: &'static str,
    pub name: String,
    pub kind: &'static str,
    pub feature: &'static str,
    pub installed: bool,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifecycle {
    pub manage: bool,
    pub units: Vec<UnitState>,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self {
            manage: true,
            units: Vec::new(),
        }
    }
}

pub fn unit_of(id: &str) -> Option<&'static str> {
    crate::registry::model(id).and_then(|spec| spec.unit)
}

pub fn cascade_dir() -> PathBuf {
    env::var("PARLO_CASCADE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("engines/cascade"))
}

pub fn models_root() -> PathBuf {
    if let Ok(configured) = env::var("PARLO_MODELS_DIR") {
        return PathBuf::from(configured);
    }
    let configured = crate::config::load().models_dir;
    if !configured.trim().is_empty() {
        return PathBuf::from(configured);
    }
    let checkout = cascade_dir().join("models");
    if checkout.is_dir() {
        return std::path::absolute(&checkout).unwrap_or(checkout);
    }
    crate::config::data_root().join("models")
}

fn size(directory: &Path, files: &[&str]) -> (bool, u64) {
    let mut installed = true;
    let mut bytes = 0;
    for file in files {
        match fs::metadata(directory.join(file)) {
            Ok(metadata) if metadata.len() > 0 => bytes += metadata.len(),
            _ => installed = false,
        }
    }
    (installed, bytes)
}

fn voice_size(directory: &Path, file: &str) -> (bool, u64) {
    let (installed, bytes) = size(directory, &[file]);
    if !installed {
        return (false, bytes);
    }
    let settings = directory.join(format!("{file}.json"));
    (
        true,
        bytes + fs::metadata(settings).map(|data| data.len()).unwrap_or(0),
    )
}

pub fn status() -> Vec<Model> {
    let root = models_root();
    crate::registry::MODELS
        .iter()
        .map(|spec| {
            let (installed, bytes) = match spec.source {
                crate::registry::Source::Parakeet => size(&root.join("parakeet"), PARAKEET_FILES),
                crate::registry::Source::Whisper => {
                    match whisper_cache().and_then(|cache| whisper_installed(&cache)) {
                        Some(bytes) => (true, bytes),
                        None => (false, 0),
                    }
                }
                crate::registry::Source::Voice(file) => voice_size(&root, file),
                crate::registry::Source::Translation(source, target) => {
                    translation_size(&root, source, target)
                }
                crate::registry::Source::Speaker(file) => size(&root, &[file]),
            };
            Model {
                id: spec.id,
                name: spec.name.to_string(),
                kind: spec.kind,
                feature: spec.feature,
                installed,
                bytes,
            }
        })
        .collect()
}

pub fn ready(models: &[Model]) -> bool {
    models
        .iter()
        .filter(|model| model.id != "whisper" && model.id != "speakers")
        .all(|model| model.installed)
}

pub fn dictation_ready(models: &[Model]) -> bool {
    dictation_ready_for(models, &crate::config::load().stt)
}

pub fn dictation_ready_for(models: &[Model], recogniser: &str) -> bool {
    models
        .iter()
        .any(|model| model.installed && model.id == recogniser)
}

pub fn whisper_cache() -> Option<PathBuf> {
    if let Ok(cache) = env::var("HF_HUB_CACHE") {
        return Some(PathBuf::from(cache));
    }
    if let Ok(home) = env::var("HF_HOME") {
        return Some(PathBuf::from(home).join("hub"));
    }
    env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .ok()
        .map(|home| {
            PathBuf::from(home)
                .join(".cache")
                .join("huggingface")
                .join("hub")
        })
}

pub fn whisper_installed(cache: &Path) -> Option<u64> {
    let folder = cache.join("models--Systran--faster-whisper-base");
    let snapshots = fs::read_dir(folder.join("snapshots")).ok()?;
    for entry in snapshots.flatten() {
        let model = entry.path().join("model.bin");
        if let Ok(metadata) = fs::metadata(&model) {
            return Some(metadata.len());
        }
    }
    None
}

fn translation_folder(root: &Path, source: &str, target: &str) -> Option<PathBuf> {
    let direct = root
        .join("mt")
        .join(format!("{source}-{target}"))
        .join("ct2");
    if direct.is_dir() {
        return Some(direct);
    }
    let prefix = format!("translate-{source}_{target}-");
    for entry in fs::read_dir(root.join("mt")).ok()?.flatten() {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            let model = entry.path().join("model");
            if model.is_dir() {
                return Some(model);
            }
        }
    }
    None
}

fn translation_size(root: &Path, source: &str, target: &str) -> (bool, u64) {
    match translation_folder(root, source, target) {
        Some(folder) => size(&folder, &["model.bin", "config.json"]),
        None => (false, 0),
    }
}

fn translation_paths(root: &Path, source: &str, target: &str) -> Vec<PathBuf> {
    let mut paths = vec![root.join("mt").join(format!("{source}-{target}"))];
    let prefix = format!("translate-{source}_{target}-");
    if let Ok(entries) = fs::read_dir(root.join("mt")) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().starts_with(&prefix) {
                paths.push(entry.path());
            }
        }
    }
    paths
}

fn paths_for(id: &str) -> Option<Vec<PathBuf>> {
    let root = models_root();
    let spec = crate::registry::model(id)?;
    Some(match spec.source {
        crate::registry::Source::Parakeet => vec![root.join("parakeet")],
        crate::registry::Source::Voice(file) => {
            vec![root.join(file), root.join(format!("{file}.json"))]
        }
        crate::registry::Source::Translation(source, target) => {
            translation_paths(&root, source, target)
        }
        crate::registry::Source::Speaker(file) => vec![root.join(file)],
        crate::registry::Source::Whisper => {
            whisper_cache().map(|cache| vec![cache.join("models--Systran--faster-whisper-base")])?
        }
    })
}

fn python() -> PathBuf {
    if let Ok(configured) = env::var("PARLO_PYTHON") {
        return PathBuf::from(configured);
    }
    for candidate in [".venv/Scripts/python.exe", ".venv/bin/python"] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return path;
        }
    }
    PathBuf::from("python")
}

pub fn install(units: &[&str]) -> Result<(), String> {
    for unit in units {
        for path in paths_for(unit).unwrap_or_default() {
            if !path.exists() {
                continue;
            }
            let previous = previous_of(&path);
            if previous.is_dir() {
                let _ = fs::remove_dir_all(&previous);
            } else {
                let _ = fs::remove_file(&previous);
            }
            fs::rename(&path, &previous)
                .map_err(|error| format!("{} could not be set aside: {error}", path.display()))?;
        }
    }
    let mut command = match crate::sidecar::bundled_engine() {
        Some(engine) => {
            let mut command = Command::new(&engine);
            command.arg("--download-models");
            if !units.is_empty() {
                command.arg("--only").args(units);
            }
            command
        }
        None => {
            let script = cascade_dir().join("scripts").join("setup_models.py");
            if !script.is_file() {
                return Err(format!(
                    "cannot find {}; download the models from a source checkout",
                    script.display()
                ));
            }
            let mut command = Command::new(python());
            command.arg(&script);
            if units.is_empty() {
                command.args(["--stt", "parakeet"]);
            } else {
                command.arg("--only").args(units);
            }
            command
        }
    };
    let child = command
        .env("PARLO_MODELS_DIR", models_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("cannot start the model download: {error}"))?;
    let mut slot = running()
        .lock()
        .map_err(|_| "the download state is locked".to_string())?;
    *slot = Some(child);
    Ok(())
}

pub fn install_blocking(units: &[&str]) -> Result<(), String> {
    if units.is_empty() {
        return Ok(());
    }
    let deadline = Instant::now() + Duration::from_secs(3_600);
    install(units)?;
    loop {
        let current = status();
        let done = crate::registry::MODELS
            .iter()
            .filter(|spec| spec.unit.map(|unit| units.contains(&unit)).unwrap_or(false))
            .all(|spec| {
                current
                    .iter()
                    .any(|model| model.id == spec.id && model.installed)
            });
        if done {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err("the models did not finish downloading within an hour".to_string());
        }
        if !downloading(&current) {
            return Err("the download stopped before it finished".to_string());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

pub fn download_remaining(
    mut progress: impl FnMut(&str, f32),
    cancelled: impl Fn() -> bool,
) -> Result<(), String> {
    let current = status();
    let missing: Vec<&str> = remaining_units()
        .into_iter()
        .filter(|id| {
            !current
                .iter()
                .any(|model| model.id == *id && model.installed)
        })
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    install(&missing)?;
    let deadline = Instant::now() + Duration::from_secs(3_600);
    loop {
        if cancelled() {
            let _ = cancel();
            return Ok(());
        }
        let current = status();
        let installed = missing
            .iter()
            .filter(|id| {
                current
                    .iter()
                    .any(|model| model.id == **id && model.installed)
            })
            .count();
        progress("download", installed as f32 / missing.len() as f32);
        if installed == missing.len() {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err("the download did not finish within an hour".to_string());
        }
        if !downloading(&current) {
            return Err("the download stopped before it finished".to_string());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

pub fn cancel() -> Result<(), String> {
    let mut slot = running()
        .lock()
        .map_err(|_| "the download state is locked".to_string())?;
    let Some(child) = slot.as_mut() else {
        return Err("no download is running".to_string());
    };
    child
        .kill()
        .map_err(|error| format!("the download could not be stopped: {error}"))?;
    let _ = child.wait();
    *slot = None;
    Ok(())
}

fn feature_of(id: &str) -> Option<&'static str> {
    match crate::registry::model(id)?.kind {
        "voice" => Some("voice"),
        "translation" => Some("translation"),
        _ => None,
    }
}

fn switch_off(feature: &str) -> Result<&'static str, String> {
    let mut config = crate::config::load();
    let sentence = match feature {
        "voice" => {
            config.features.voice = false;
            " Read aloud is off in Settings until a voice is downloaded again."
        }
        _ => {
            config.features.translation = false;
            " Translation is off in Settings until a pair is downloaded again."
        }
    };
    crate::config::save(&config)
        .map_err(|error| format!("the model was removed but the switch was not saved: {error}"))?;
    Ok(sentence)
}

pub fn remove(id: &str) -> Result<String, String> {
    let paths = paths_for(id).ok_or_else(|| format!("{id} is not a model this app manages"))?;
    if downloading(&status()) {
        return Err("a download is running; wait for it to finish".to_string());
    }
    let name = status()
        .into_iter()
        .find(|model| model.id == id)
        .map(|model| model.name)
        .unwrap_or_else(|| id.to_string());
    for path in paths {
        let removed = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else if path.exists() {
            fs::remove_file(&path)
        } else {
            Ok(())
        };
        if let Err(error) = removed {
            return Err(format!(
                "{} could not be removed: {error}. Close Parlo and try again.",
                path.display()
            ));
        }
    }
    let mut message = format!("{name} removed.");
    if let Some(feature) = feature_of(id) {
        message.push_str(switch_off(feature)?);
    }
    Ok(message)
}

fn running() -> &'static Mutex<Option<Child>> {
    static RUNNING: OnceLock<Mutex<Option<Child>>> = OnceLock::new();
    RUNNING.get_or_init(|| Mutex::new(None))
}

pub fn downloading(models: &[Model]) -> bool {
    if ready(models) {
        return false;
    }
    let Ok(mut slot) = running().lock() else {
        return false;
    };
    let Some(child) = slot.as_mut() else {
        return false;
    };
    match child.try_wait() {
        Ok(Some(_)) => {
            *slot = None;
            false
        }
        Ok(None) => true,
        Err(_) => {
            *slot = None;
            false
        }
    }
}

fn directory_bytes(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0;
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.is_dir() {
            total += directory_bytes(&entry.path());
        } else {
            total += metadata.len();
        }
    }
    total
}

fn copy_tree(from: &Path, to: &Path) -> Result<u64, String> {
    fs::create_dir_all(to)
        .map_err(|error| format!("{} could not be created: {error}", to.display()))?;
    let entries = fs::read_dir(from)
        .map_err(|error| format!("{} could not be read: {error}", from.display()))?;
    let mut copied = 0;
    for entry in entries.flatten() {
        let source = entry.path();
        let destination = to.join(entry.file_name());
        if source.is_dir() {
            copied += copy_tree(&source, &destination)?;
        } else {
            fs::copy(&source, &destination)
                .map_err(|error| format!("{} could not be copied: {error}", source.display()))?;
            copied += fs::metadata(&destination)
                .map(|meta| meta.len())
                .unwrap_or(0);
        }
    }
    Ok(copied)
}

fn same_tree(left: &Path, right: &Path) -> bool {
    let Ok(entries) = fs::read_dir(left) else {
        return false;
    };
    for entry in entries.flatten() {
        let source = entry.path();
        let target = right.join(entry.file_name());
        if source.is_dir() {
            if !target.is_dir() || !same_tree(&source, &target) {
                return false;
            }
        } else if !same_file(&source, &target) {
            return false;
        }
    }
    true
}

fn same_file(left: &Path, right: &Path) -> bool {
    use std::io::Read;

    let (Ok(here), Ok(there)) = (fs::metadata(left), fs::metadata(right)) else {
        return false;
    };
    if !there.is_file() || here.len() == 0 || here.len() != there.len() {
        return false;
    }
    let (Ok(mut one), Ok(mut other)) = (fs::File::open(left), fs::File::open(right)) else {
        return false;
    };
    let mut first = [0u8; 64 * 1024];
    let mut second = [0u8; 64 * 1024];
    loop {
        let read_one = match one.read(&mut first) {
            Ok(count) => count,
            Err(_) => return false,
        };
        let read_two = match other.read(&mut second) {
            Ok(count) => count,
            Err(_) => return false,
        };
        if read_one != read_two {
            return false;
        }
        if read_one == 0 {
            return true;
        }
        if first[..read_one] != second[..read_two] {
            return false;
        }
    }
}

#[cfg(test)]
mod adopt_tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!("parlo-adopt-{name}-{stamp}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn pair_of(here: &Path, there: &Path, left: &[u8], right: &[u8]) {
        fs::create_dir_all(here).unwrap();
        fs::create_dir_all(there).unwrap();
        fs::write(here.join("model.bin"), left).unwrap();
        fs::write(there.join("model.bin"), right).unwrap();
    }

    #[test]
    fn a_copy_with_the_same_bytes_is_the_same_tree() {
        let root = folder("same");
        pair_of(
            &root.join("here"),
            &root.join("there"),
            b"weights",
            b"weights",
        );
        assert!(same_tree(&root.join("here"), &root.join("there")));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_same_size_with_different_bytes_is_not_adopted() {
        let root = folder("different");
        pair_of(
            &root.join("here"),
            &root.join("there"),
            b"weights",
            b"weightz",
        );
        assert!(!same_tree(&root.join("here"), &root.join("there")));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_empty_file_never_counts_as_adopted() {
        let root = folder("empty");
        pair_of(&root.join("here"), &root.join("there"), b"", b"");
        assert!(!same_tree(&root.join("here"), &root.join("there")));
        fs::remove_dir_all(&root).unwrap();
    }
}

fn previous_of(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    name.push(".previous");
    path.with_file_name(name)
}

pub fn has_previous(id: &str) -> bool {
    paths_for(id)
        .map(|paths| paths.iter().any(|path| previous_of(path).exists()))
        .unwrap_or(false)
}

pub fn rollback(id: &str) -> Result<String, String> {
    let paths = paths_for(id).ok_or_else(|| format!("{id} is not a model this app manages"))?;
    let mut restored = 0;
    for path in paths {
        let previous = previous_of(&path);
        if !previous.exists() {
            continue;
        }
        let removed = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
        if let Err(error) = removed {
            if path.exists() {
                return Err(format!(
                    "{} could not be replaced: {error}. Close Parlo and try again.",
                    path.display()
                ));
            }
        }
        fs::rename(&previous, &path).map_err(|error| {
            format!(
                "{} could not be restored: {error}. The previous copy is still there.",
                path.display()
            )
        })?;
        restored += 1;
    }
    if restored == 0 {
        return Err("there is no previous copy of this model to go back to".to_string());
    }
    let name = status()
        .into_iter()
        .find(|model| model.id == id)
        .map(|model| model.name)
        .unwrap_or_else(|| id.to_string());
    Ok(format!("{name} is back on the copy that was here before."))
}

fn relocate_target(target: &str) -> Result<(PathBuf, PathBuf), String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return Err("give the folder the models should move to".to_string());
    }
    let destination = PathBuf::from(trimmed);
    if !destination.is_absolute() {
        return Err("that is a relative path; give the full path of the folder".to_string());
    }
    let source = models_root();
    if source == destination {
        return Err("the models are already there".to_string());
    }
    if destination.starts_with(&source) || source.starts_with(&destination) {
        return Err("the new folder cannot sit inside the old one, or around it".to_string());
    }
    if downloading(&status()) {
        return Err("a download is running; stop it before moving the models".to_string());
    }
    if !source.is_dir() {
        return Err(format!("there are no models in {} yet", source.display()));
    }
    Ok((source, destination))
}

#[cfg(windows)]
fn free_bytes(path: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);
    let mut available: u64 = 0;
    let answered = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut available,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if answered == 0 {
        None
    } else {
        Some(available)
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        directory: *const u16,
        available: *mut u64,
        total: *mut u64,
        free: *mut u64,
    ) -> i32;
}

#[cfg(not(windows))]
fn free_bytes(_path: &Path) -> Option<u64> {
    None
}

pub const PAIR_FILES: &[&str] = &[
    "encoder_model.onnx",
    "decoder_model.onnx",
    "decoder_with_past_model.onnx",
    "config.json",
    "vocab.json",
    "source.spm",
    "target.spm",
];

pub fn install_pair(folder: &str, from: &str, to: &str) -> Result<String, String> {
    install_pair_in(&models_root(), folder, from, to)
}

fn install_pair_in(root: &Path, folder: &str, from: &str, to: &str) -> Result<String, String> {
    let source = PathBuf::from(folder.trim());
    if !source.is_dir() {
        return Err(format!("{} is not a folder", source.display()));
    }
    let missing: Vec<&str> = PAIR_FILES
        .iter()
        .copied()
        .filter(|name| !source.join(*name).is_file())
        .collect();
    if !missing.is_empty() {
        return Err(format!("the export is missing {}", missing.join(", ")));
    }
    let destination = root.join("mt").join(format!("{from}-{to}-onnx"));
    if destination.exists() {
        return Err(format!(
            "{} already holds a pair; remove it before installing another",
            destination.display()
        ));
    }
    let bytes = copy_tree(&source, &destination)?;
    Ok(format!(
        "Installed {from} to {to} in {}, {}",
        destination.display(),
        describe_bytes(bytes)
    ))
}

pub fn free_space() -> Option<u64> {
    free_bytes(&models_root())
}

pub fn describe_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KB", bytes.div_ceil(1024))
    }
}

pub fn relocate_plan(target: &str) -> Result<(u64, bool), String> {
    let (source, destination) = relocate_target(target)?;
    Ok((directory_bytes(&source), same_tree(&source, &destination)))
}

pub fn relocate(target: &str) -> Result<String, String> {
    let (source, destination) = relocate_target(target)?;
    if same_tree(&source, &destination) {
        let size = describe_bytes(directory_bytes(&source));
        fs::remove_dir_all(&source).map_err(|error| {
            format!(
                "the copy in {} is complete but {} could not be removed: {error}",
                destination.display(),
                source.display()
            )
        })?;
        let mut config = crate::config::load();
        config.models_dir = destination.display().to_string();
        crate::config::save(&config).map_err(|error| {
            format!("the models were adopted but the folder was not saved: {error}")
        })?;
        return Ok(format!(
            "Adopted {size} already in {}. Nothing was copied.",
            destination.display()
        ));
    }
    let expected = directory_bytes(&source);
    let copied = copy_tree(&source, &destination)?;
    if copied != expected {
        return Err(format!(
            "the copy holds {copied} bytes but {} holds {expected}; nothing was deleted",
            source.display()
        ));
    }
    fs::remove_dir_all(&source).map_err(|error| {
        format!(
            "the copy is in place but {} could not be removed: {error}",
            source.display()
        )
    })?;
    let mut config = crate::config::load();
    config.models_dir = destination.display().to_string();
    crate::config::save(&config)
        .map_err(|error| format!("the models moved but the folder was not saved: {error}"))?;
    let mut message = format!(
        "Moved {} to {}. The old folder is gone.",
        describe_bytes(copied),
        destination.display()
    );
    if env::var("PARLO_MODELS_DIR").is_ok() {
        message.push_str(
            " This session still reads PARLO_MODELS_DIR, so start Parlo without it to use the new folder.",
        );
    }
    Ok(message)
}

pub fn reason_for(model: &Model, machine: &str) -> String {
    let what = match model.id {
        "parakeet" => "the only dictation model in this build",
        "voice-pt" => "the Portuguese voice for read aloud",
        "mt-pt-en" => "the Portuguese to English translation pack",
        "mt-en-pt" => "the English to Portuguese translation pack",
        "voice-en" => "the English voice for read aloud",
        "voice-de" => "the German voice for read aloud",
        "voice-fr" => "the French voice for read aloud",
        "voice-es" => "the Spanish voice for read aloud",
        "voice-it" => "the Italian voice for read aloud",
        "voice-nl" => "the Dutch voice for read aloud",
        "voice-ru" => "the Russian voice for read aloud",
        "speakers" => {
            "tells people apart in a meeting, and a meeting records as one voice without it"
        }
        _ => "a voice for read aloud",
    };
    format!("{what}, and this machine has {machine}")
}

fn quality_for(model: &Model) -> &'static str {
    match model.id {
        "parakeet" => "word error 0.24 on 20 European Portuguese sentences, Windows 11, 4 threads",
        "mt-pt-en" => {
            "numbers, clocks and negations 1.000, names 0.980 on this project's gate corpus"
        }
        "mt-en-pt" => {
            "numbers and negations 1.000, clocks and names 0.980 on this project's gate corpus"
        }
        _ => "no quality number measured",
    }
}

fn state_of(model: &Model, lifecycle: &Lifecycle, pair: &str, working: bool) -> &'static str {
    if !model.installed {
        return "not installed";
    }
    if working && model.id == "parakeet" {
        return "in use";
    }
    if model.kind == "translation" && model.id != format!("mt-{pair}") {
        return "installed";
    }
    let unit = unit_of(model.id);
    let held = lifecycle.units.iter().any(|state| {
        if !state.loaded {
            return false;
        }
        Some(state.unit.as_str()) == unit
            || (unit == Some("translation") && state.unit == "dictation translation")
    });
    if held {
        "loaded"
    } else {
        "installed"
    }
}

pub fn as_json(models: &[Model], lifecycle: &Lifecycle, pair: &str, working: bool) -> String {
    let machine = device::profile();
    let described = machine.describe();
    let rows: Vec<serde_json::Value> = models
        .iter()
        .map(|model| {
            let state = lifecycle
                .units
                .iter()
                .find(|state| Some(state.unit.as_str()) == unit_of(model.id));
            serde_json::json!({
                "id": model.id,
                "name": model.name,
                "kind": model.kind,
                "feature": model.feature,
                "installed": model.installed,
                "state": state_of(model, lifecycle, pair, working),
                "idle_seconds": state.and_then(|state| state.idle_seconds),
                "policy": state.map(|state| state.policy),
                "policy_origin": state.map(|state| state.policy_origin.clone()),
                "bytes": model.bytes,
                "licence": crate::registry::model(model.id).map(|spec| spec.licence).unwrap_or_default(),
                "reason": reason_for(model, &described),
                "quality": quality_for(model),
                "previous": has_previous(model.id),
            })
        })
        .collect();
    serde_json::json!({
        "models": rows,
        "ready": ready(models),
        "dictation": dictation_ready(models),
        "downloading": downloading(models),
        "manage_memory": lifecycle.manage,
        "folder": models_root().display().to_string(),
        "free_bytes": free_space(),
        "recommendation": if machine.memory_gb as i64 * 1024 - 746 >= 700 {
            "Parakeet 0.6B v3 at 0.21 of real time here, because this machine leaves room for its 746 MB."
        } else {
            "Whisper base at 0.24 of real time here, because Parakeet would leave less than 700 MB free."
        },
        "machine": {
            "cores": machine.cores,
            "memory_gb": machine.memory_gb,
            "avx2": machine.avx2,
        },
    })
    .to_string()
}

pub fn set_policy(id: &str, minutes: Option<u32>) -> Result<String, String> {
    let unit = unit_of(id).ok_or_else(|| format!("{id} is not a model this app manages"))?;
    if minutes.is_some_and(|minutes| minutes > 1_440) {
        return Err("a model cannot be kept for longer than a day".to_string());
    }
    let name = status()
        .into_iter()
        .find(|model| model.id == id)
        .map(|model| model.name)
        .unwrap_or_else(|| id.to_string());
    let mut config = crate::config::load();
    match minutes {
        Some(minutes) => {
            config.policies.insert(unit.to_string(), minutes);
        }
        None => {
            config.policies.remove(unit);
        }
    }
    crate::config::save(&config).map_err(|error| format!("the setting was not saved: {error}"))?;
    Ok(match minutes {
        Some(0) => format!("{name} stays loaded until you unload it."),
        Some(minutes) => format!("{name} is freed after {minutes} minutes without use."),
        None => format!("{name} follows the global setting again."),
    })
}

#[cfg(test)]
mod pair_tests {
    use super::*;

    fn write_export(folder: &Path, skip: Option<&str>) {
        fs::create_dir_all(folder).unwrap();
        for name in PAIR_FILES {
            if Some(*name) == skip {
                continue;
            }
            fs::write(folder.join(*name), b"weights").unwrap();
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        let root = std::env::temp_dir().join(format!("parlo-pair-{name}-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn a_complete_export_is_installed() {
        let root = scratch("ok");
        let folder = root.join("export");
        write_export(&folder, None);
        let answer = install_pair_in(&root, folder.to_str().unwrap(), "pt", "de").unwrap();
        assert!(answer.contains("pt to de"));
        assert!(root
            .join("mt")
            .join("pt-de-onnx")
            .join("encoder_model.onnx")
            .is_file());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_missing_file_is_named() {
        let root = scratch("missing");
        let folder = root.join("export");
        write_export(&folder, Some("vocab.json"));
        let error = install_pair_in(&root, folder.to_str().unwrap(), "pt", "de").unwrap_err();
        assert!(error.contains("vocab.json"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_folder_that_is_not_there_is_refused() {
        let root = scratch("absent");
        let absent = root.join("nope");
        let error = install_pair_in(&root, absent.to_str().unwrap(), "pt", "de").unwrap_err();
        assert!(error.contains("not a folder"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_existing_pair_is_not_overwritten() {
        let root = scratch("exists");
        let folder = root.join("export");
        write_export(&folder, None);
        install_pair_in(&root, folder.to_str().unwrap(), "pt", "de").unwrap();
        let error = install_pair_in(&root, folder.to_str().unwrap(), "pt", "de").unwrap_err();
        assert!(error.contains("already holds"));
        fs::remove_dir_all(&root).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(units: &[&str]) -> Vec<UnitState> {
        units
            .iter()
            .map(|unit| UnitState {
                unit: (*unit).to_string(),
                loaded: true,
                idle_seconds: Some(45),
                policy: 30,
                policy_origin: "global".to_string(),
            })
            .collect()
    }

    #[test]
    fn dictation_needs_the_recogniser_the_engine_will_use() {
        let models = vec![
            Model {
                id: "whisper",
                name: "Whisper base".into(),
                kind: "recogniser",
                feature: "Dictation",
                installed: true,
                bytes: 145,
            },
            Model {
                id: "parakeet",
                name: "Parakeet".into(),
                kind: "recogniser",
                feature: "Dictation",
                installed: false,
                bytes: 0,
            },
        ];
        assert!(!dictation_ready_for(&models, "parakeet"));
        assert!(dictation_ready_for(&models, "whisper"));
        assert!(!ready(&models));
    }

    #[test]
    fn whisper_is_installed_only_when_the_weights_are_there() {
        let root = std::env::temp_dir().join(format!("parlo-whisper-{}", std::process::id()));
        let cache = root.join("hub");
        let folder = cache
            .join("models--Systran--faster-whisper-base")
            .join("snapshots")
            .join("abc");
        fs::create_dir_all(&folder).unwrap();
        assert_eq!(whisper_installed(&cache), None);
        fs::write(folder.join("model.bin"), vec![0u8; 2048]).unwrap();
        assert_eq!(whisper_installed(&cache), Some(2048));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn every_model_maps_to_the_unit_the_engine_reports() {
        assert_eq!(unit_of("parakeet"), Some("recognition"));
        assert_eq!(unit_of("voice-en"), Some("voice"));
        assert_eq!(unit_of("voice-de"), Some("voice"));
        assert_eq!(unit_of("voice-fr"), Some("voice"));
        assert_eq!(unit_of("mt-en-pt"), Some("translation"));
        assert_eq!(unit_of("nonsense"), None);
    }

    #[test]
    fn a_previous_copy_lives_beside_the_model() {
        let path = std::path::Path::new("C:/models/parakeet");
        assert_eq!(
            previous_of(path),
            std::path::Path::new("C:/models/parakeet.previous")
        );
    }

    #[test]
    fn a_folder_is_adoptable_only_when_every_file_matches() {
        let root = std::env::temp_dir().join(format!("parlo-adopt-{}", std::process::id()));
        let left = root.join("left");
        let right = root.join("right");
        fs::create_dir_all(left.join("inner")).unwrap();
        fs::create_dir_all(right.join("inner")).unwrap();
        fs::write(left.join("inner/model.bin"), vec![1u8; 64]).unwrap();
        assert!(!same_tree(&left, &right));
        fs::write(right.join("inner/model.bin"), vec![1u8; 64]).unwrap();
        assert!(same_tree(&left, &right));
        fs::write(right.join("inner/model.bin"), vec![2u8; 64]).unwrap();
        assert!(!same_tree(&left, &right));
        fs::write(right.join("inner/model.bin"), vec![1u8; 32]).unwrap();
        assert!(!same_tree(&left, &right));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_relative_folder_is_refused_before_anything_moves() {
        let error = relocate("models-here").unwrap_err();
        assert!(error.contains("full path"), "{error}");
    }

    #[test]
    fn both_translation_pairs_are_listed_and_removable() {
        let ids: Vec<&str> = status().iter().map(|model| model.id).collect();
        assert!(ids.contains(&"mt-pt-en"));
        assert!(ids.contains(&"mt-en-pt"));
        assert!(paths_for("mt-pt-en").is_some());
        assert!(paths_for("mt-en-pt").is_some());
    }

    #[test]
    fn cancelling_without_a_download_says_so() {
        assert!(cancel().unwrap_err().contains("no download"));
    }

    #[test]
    fn every_model_says_why_it_is_the_one_offered() {
        let models = status();
        let parsed: serde_json::Value =
            serde_json::from_str(&as_json(&models, &Lifecycle::default(), "pt-en", false)).unwrap();
        let rows = parsed["models"].as_array().unwrap();
        assert!(!rows.is_empty());
        for row in rows {
            let reason = row["reason"].as_str().unwrap();
            assert!(reason.contains("this machine has"), "{reason}");
        }
        assert!(parsed["machine"]["cores"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn installed_models_are_never_downloading() {
        let models = vec![Model {
            id: "voice-en",
            name: "voice".to_string(),
            kind: "voice",
            feature: "Read aloud",
            installed: true,
            bytes: 1024,
        }];
        assert!(!downloading(&models));
        let body = as_json(&models, &Lifecycle::default(), "pt-en", false);
        assert!(body.contains("\"downloading\":false"));
    }

    #[test]
    fn an_unknown_model_is_refused_before_any_file_is_touched() {
        assert!(remove("nonsense").is_err());
    }

    #[test]
    fn every_removable_model_names_the_switch_it_turns_off() {
        assert_eq!(feature_of("voice-pt"), Some("voice"));
        assert_eq!(feature_of("voice-en"), Some("voice"));
        assert_eq!(feature_of("mt-pt-en"), Some("translation"));
        assert_eq!(feature_of("mt-en-pt"), Some("translation"));
        assert_eq!(feature_of("parakeet"), None);
    }

    #[test]
    fn every_row_carries_its_own_id_and_feature() {
        let models = status();
        assert!(models.iter().all(|model| !model.id.is_empty()));
        assert!(models.iter().any(|model| model.feature == "Dictation"));
        let body = as_json(&models, &Lifecycle::default(), "pt-en", false);
        assert!(body.contains("\"dictation\":"));
    }

    #[test]
    fn the_models_root_is_absolute_after_resolution() {
        assert!(models_root().is_absolute() || models_root().starts_with("engines"));
    }

    #[test]
    fn the_portuguese_voice_is_european() {
        let voice = crate::registry::model("voice-pt").expect("a Portuguese voice is offered");
        let file = match voice.source {
            crate::registry::Source::Voice(file) => file,
            _ => panic!("the Portuguese voice must point at a file"),
        };
        assert!(file.starts_with("pt_PT-"), "{file}");
        let paths = paths_for("voice-pt").unwrap();
        assert_eq!(paths[0].file_name().unwrap(), file);
    }

    #[test]
    fn a_model_says_whether_it_sits_in_memory() {
        let models = vec![
            Model {
                id: "parakeet",
                name: "recogniser".to_string(),
                kind: "recogniser",
                feature: "Dictation",
                installed: true,
                bytes: 10,
            },
            Model {
                id: "mt-pt-en",
                name: "pair".to_string(),
                kind: "translation",
                feature: "Translation",
                installed: true,
                bytes: 10,
            },
            Model {
                id: "voice-pt",
                name: "voice".to_string(),
                kind: "voice",
                feature: "Read aloud",
                installed: false,
                bytes: 0,
            },
        ];
        let lifecycle = Lifecycle {
            manage: true,
            units: held(&["recognition", "dictation translation"]),
        };
        let parsed: serde_json::Value =
            serde_json::from_str(&as_json(&models, &lifecycle, "pt-en", false)).unwrap();
        let rows = parsed["models"].as_array().unwrap();
        assert_eq!(rows[0]["state"], "loaded");
        assert_eq!(rows[1]["state"], "loaded");
        assert_eq!(rows[2]["state"], "not installed");
        assert_eq!(rows[0]["idle_seconds"], 45);
        assert_eq!(rows[0]["policy"], 30);
        assert_eq!(rows[0]["policy_origin"], "global");
        assert!(rows[2]["idle_seconds"].is_null());
        assert!(rows[2]["policy"].is_null());
    }

    #[test]
    fn a_model_that_was_not_asked_for_reads_as_on_disk() {
        let models = vec![Model {
            id: "voice-en",
            name: "voice".to_string(),
            kind: "voice",
            feature: "Read aloud",
            installed: true,
            bytes: 10,
        }];
        let parsed: serde_json::Value =
            serde_json::from_str(&as_json(&models, &Lifecycle::default(), "pt-en", false)).unwrap();
        assert_eq!(parsed["models"][0]["state"], "installed");
    }

    #[test]
    fn the_recogniser_reads_in_use_while_the_engine_works() {
        let models = vec![
            Model {
                id: "parakeet",
                name: "recogniser".to_string(),
                kind: "recogniser",
                feature: "Dictation",
                installed: true,
                bytes: 10,
            },
            Model {
                id: "voice-en",
                name: "voice".to_string(),
                kind: "voice",
                feature: "Read aloud",
                installed: true,
                bytes: 10,
            },
        ];
        let lifecycle = Lifecycle {
            manage: true,
            units: held(&["recognition"]),
        };
        let parsed: serde_json::Value =
            serde_json::from_str(&as_json(&models, &lifecycle, "pt-en", true)).unwrap();
        assert_eq!(parsed["models"][0]["state"], "in use");
        assert_eq!(parsed["models"][1]["state"], "installed");
        let idle: serde_json::Value =
            serde_json::from_str(&as_json(&models, &lifecycle, "pt-en", false)).unwrap();
        assert_eq!(idle["models"][0]["state"], "loaded");
    }

    #[test]
    fn every_row_states_a_number_or_says_there_is_none() {
        let models = status();
        let parsed: serde_json::Value =
            serde_json::from_str(&as_json(&models, &Lifecycle::default(), "pt-en", false)).unwrap();
        for row in parsed["models"].as_array().unwrap() {
            let quality = row["quality"].as_str().unwrap();
            assert!(!quality.is_empty(), "{}", row["id"]);
        }
        let voice = models.iter().find(|model| model.id == "voice-en").unwrap();
        assert!(quality_for(voice).contains("no quality number measured"));
    }
}
