use std::path::{Path, PathBuf};

pub fn python_for(script_dir: &Path) -> String {
    if let Ok(explicit) = std::env::var("PARLO_PYTHON") {
        return explicit;
    }
    find_venv_python(script_dir)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "python".to_string())
}

pub(super) fn find_venv_python(script_dir: &Path) -> Option<PathBuf> {
    let executable = if cfg!(windows) {
        "Scripts/python.exe"
    } else {
        "bin/python"
    };
    script_dir
        .ancestors()
        .take(3)
        .map(|base| base.join(".venv").join(executable))
        .find(|candidate| candidate.is_file())
}

pub fn bundled_engine() -> Option<PathBuf> {
    engine_candidates().into_iter().next()
}

fn engine_name() -> &'static str {
    if cfg!(windows) {
        "parlo-engine.exe"
    } else {
        "parlo-engine"
    }
}

fn rust_candidates() -> Vec<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|root| root.join("parlo-engine-rs").join(engine_name()))
        .filter(|path| path.is_file())
        .into_iter()
        .collect()
}

fn python_candidates() -> Vec<PathBuf> {
    let mut candidates = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|root| {
            vec![
                root.join("parlo-engine").join(engine_name()),
                root.join(engine_name()),
            ]
        })
        .unwrap_or_default();
    candidates.retain(|path| path.is_file());
    candidates
}

pub(super) fn engine_candidates() -> Vec<PathBuf> {
    if let Ok(configured) = std::env::var("PARLO_ENGINE") {
        match configured.as_str() {
            "rust" => {
                let mut candidates = rust_candidates();
                candidates.extend(python_candidates());
                return candidates;
            }
            "python" => return python_candidates(),
            _ => {
                let path = PathBuf::from(configured);
                if path.is_file() {
                    return vec![path];
                }
            }
        }
    }
    let mut candidates = python_candidates();
    candidates.extend(rust_candidates());
    candidates
}

pub(super) fn explicit_engine() -> Option<PathBuf> {
    let configured = std::env::var("PARLO_ENGINE").ok()?;
    if configured == "rust" || configured == "python" {
        return None;
    }
    let path = PathBuf::from(configured);
    path.is_file().then_some(path)
}

pub(super) fn espeak_data_for(engine: &Path) -> Option<PathBuf> {
    let folder = engine.parent()?;
    let mut candidates = vec![folder.join("espeak-ng-data")];
    if let Some(root) = folder.parent() {
        candidates.push(root.join("parlo-engine/_internal/piper/espeak-ng-data"));
    }
    candidates.into_iter().find(|path| path.is_dir())
}
