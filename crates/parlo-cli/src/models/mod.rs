use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use parlo_core::{device, UnitState};
mod install;
mod rollback;

pub use install::*;
pub use rollback::*;

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

#[cfg(test)]
mod adopt_tests;

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
mod pair_tests;

#[cfg(test)]
mod tests;
