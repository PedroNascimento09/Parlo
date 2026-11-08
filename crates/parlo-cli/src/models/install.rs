use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::*;

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

pub(super) fn feature_of(id: &str) -> Option<&'static str> {
    match crate::registry::model(id)?.kind {
        "voice" => Some("voice"),
        "translation" => Some("translation"),
        _ => None,
    }
}

pub(super) fn switch_off(feature: &str) -> Result<&'static str, String> {
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

pub(super) fn running() -> &'static Mutex<Option<Child>> {
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

pub(super) fn directory_bytes(path: &Path) -> u64 {
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

pub(super) fn copy_tree(from: &Path, to: &Path) -> Result<u64, String> {
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

pub(super) fn same_tree(left: &Path, right: &Path) -> bool {
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

pub(super) fn same_file(left: &Path, right: &Path) -> bool {
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
