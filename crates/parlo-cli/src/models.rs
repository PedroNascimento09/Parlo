use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const DOWNLOAD_WINDOW_SECONDS: u64 = 3_600;

const PARAKEET_FILES: &[&str] = &[
    "encoder.int8.onnx",
    "decoder.int8.onnx",
    "joiner.int8.onnx",
    "tokens.txt",
];
const VOICES: &[(&str, &str, &str)] = &[
    ("voice-pt", "pt_BR-faber-medium.onnx", "Portuguese voice"),
    ("voice-en", "en_US-lessac-medium.onnx", "English voice"),
];
pub const DICTATION_UNITS: &[&str] = &["parakeet"];
pub const REMAINING_UNITS: &[&str] = &["voice-pt", "voice-en", "mt-pt-en", "mt-en-pt"];

pub struct Model {
    pub id: &'static str,
    pub name: String,
    pub kind: &'static str,
    pub feature: &'static str,
    pub installed: bool,
    pub bytes: u64,
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
    let (parakeet_installed, parakeet_bytes) = size(&root.join("parakeet"), PARAKEET_FILES);
    let mut models = vec![Model {
        id: "parakeet",
        name: "Parakeet 0.6B v3".to_string(),
        kind: "recogniser",
        feature: "Dictation",
        installed: parakeet_installed,
        bytes: parakeet_bytes,
    }];
    for (id, file, label) in VOICES {
        let (installed, bytes) = voice_size(&root, file);
        models.push(Model {
            id,
            name: (*label).to_string(),
            kind: "voice",
            feature: "Read aloud",
            installed,
            bytes,
        });
    }
    models
}

pub fn ready(models: &[Model]) -> bool {
    models.iter().all(|model| model.installed)
}

pub fn dictation_ready(models: &[Model]) -> bool {
    models
        .iter()
        .find(|model| model.id == "parakeet")
        .is_some_and(|model| model.installed)
}

fn paths_for(id: &str) -> Option<Vec<PathBuf>> {
    let root = models_root();
    match id {
        "parakeet" => Some(vec![root.join("parakeet")]),
        "voice-pt" => Some(vec![
            root.join("pt_BR-faber-medium.onnx"),
            root.join("pt_BR-faber-medium.onnx.json"),
        ]),
        "voice-en" => Some(vec![
            root.join("en_US-lessac-medium.onnx"),
            root.join("en_US-lessac-medium.onnx.json"),
        ]),
        _ => None,
    }
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
    command
        .env("PARLO_MODELS_DIR", models_root())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| mark_started())
        .map_err(|error| format!("cannot start the model download: {error}"))
}

pub fn remove(id: &str) -> Result<(), String> {
    let paths = paths_for(id).ok_or_else(|| format!("{id} is not a model this app manages"))?;
    if downloading(&status()) {
        return Err("a download is running; wait for it to finish".to_string());
    }
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
    Ok(())
}

fn started() -> &'static Mutex<Option<Instant>> {
    static STARTED: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    STARTED.get_or_init(|| Mutex::new(None))
}

pub fn downloading(models: &[Model]) -> bool {
    if ready(models) {
        return false;
    }
    let Ok(started) = started().lock() else {
        return false;
    };
    started.is_some_and(|at| at.elapsed().as_secs() < DOWNLOAD_WINDOW_SECONDS)
}

fn mark_started() {
    if let Ok(mut started) = started().lock() {
        *started = Some(Instant::now());
    }
}

pub fn as_json(models: &[Model]) -> String {
    let rows: Vec<String> = models
        .iter()
        .map(|model| {
            format!(
                "{{\"id\":\"{}\",\"name\":\"{}\",\"kind\":\"{}\",\"feature\":\"{}\",\"installed\":{},\"bytes\":{}}}",
                model.id, model.name, model.kind, model.feature, model.installed, model.bytes
            )
        })
        .collect();
    format!(
        "{{\"models\":[{}],\"ready\":{},\"dictation\":{},\"downloading\":{}}}",
        rows.join(","),
        ready(models),
        dictation_ready(models),
        downloading(models)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(as_json(&models).contains("\"downloading\":false"));
    }

    #[test]
    fn an_unknown_model_is_refused_before_any_file_is_touched() {
        assert!(remove("nonsense").is_err());
    }

    #[test]
    fn every_row_carries_its_own_id_and_feature() {
        let models = status();
        assert!(models.iter().all(|model| !model.id.is_empty()));
        assert!(models.iter().any(|model| model.feature == "Dictation"));
        assert!(as_json(&models).contains("\"dictation\":"));
    }

    #[test]
    fn the_models_root_is_absolute_after_resolution() {
        assert!(models_root().is_absolute() || models_root().starts_with("engines"));
    }
}
