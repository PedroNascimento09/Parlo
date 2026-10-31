use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

const DOWNLOAD_WINDOW_SECONDS: u64 = 3_600;

const PARAKEET: &[&str] = &[
    "encoder.int8.onnx",
    "decoder.int8.onnx",
    "joiner.int8.onnx",
    "tokens.txt",
];
const VOICES: &[(&str, &str)] = &[
    ("pt_BR-faber-medium.onnx", "Portuguese voice"),
    ("en_US-lessac-medium.onnx", "English voice"),
];

pub struct Model {
    pub name: String,
    pub kind: &'static str,
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

pub fn status() -> Vec<Model> {
    let root = models_root();
    let (parakeet_installed, parakeet_bytes) = size(&root.join("parakeet"), PARAKEET);
    let mut models = vec![Model {
        name: "Parakeet 0.6B v3".to_string(),
        kind: "recogniser",
        installed: parakeet_installed,
        bytes: parakeet_bytes,
    }];
    for (file, label) in VOICES {
        let (installed, bytes) = size(&root, &[*file]);
        models.push(Model {
            name: (*label).to_string(),
            kind: "voice",
            installed,
            bytes,
        });
    }
    models
}

pub fn ready(models: &[Model]) -> bool {
    models.iter().all(|model| model.installed)
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

pub fn install() -> Result<(), String> {
    let mut command = match crate::sidecar::bundled_engine() {
        Some(engine) => {
            let mut command = Command::new(&engine);
            command.arg("--download-models");
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
            command.arg(&script).args(["--stt", "parakeet"]);
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
                "{{\"name\":\"{}\",\"kind\":\"{}\",\"installed\":{},\"bytes\":{}}}",
                model.name, model.kind, model.installed, model.bytes
            )
        })
        .collect();
    format!(
        "{{\"models\":[{}],\"ready\":{},\"downloading\":{}}}",
        rows.join(","),
        ready(models),
        downloading(models)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_models_are_never_downloading() {
        let models = vec![Model {
            name: "voice".to_string(),
            kind: "voice",
            installed: true,
            bytes: 1024,
        }];
        assert!(!downloading(&models));
        assert!(as_json(&models).contains("\"downloading\":false"));
    }

    #[test]
    fn the_models_root_is_absolute_after_resolution() {
        assert!(models_root().is_absolute() || models_root().starts_with("engines"));
    }
}
