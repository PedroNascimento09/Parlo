use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
        return checkout;
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
        .map(|_| ())
        .map_err(|error| format!("cannot start the model download: {error}"))
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
        "{{\"models\":[{}],\"ready\":{}}}",
        rows.join(","),
        ready(models)
    )
}
