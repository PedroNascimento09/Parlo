use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::shortcut::DEFAULT as DEFAULT_SHORTCUT;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub engine: String,
    pub stt: String,
    pub from: String,
    pub to: String,
    pub write: String,
    pub shortcut: String,
    pub port: u16,
    pub device: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            engine: "cascade".to_string(),
            stt: "parakeet".to_string(),
            from: "pt".to_string(),
            to: "en".to_string(),
            write: String::new(),
            shortcut: DEFAULT_SHORTCUT.to_string(),
            port: 8787,
            device: None,
        }
    }
}

pub fn data_root() -> PathBuf {
    if let Ok(configured) = env::var("PARLO_DATA_DIR") {
        return PathBuf::from(configured);
    }
    if cfg!(target_os = "windows") {
        if let Ok(appdata) = env::var("APPDATA") {
            return PathBuf::from(appdata).join("parlo");
        }
    }
    if cfg!(target_os = "macos") {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("parlo");
    }
    if let Ok(xdg) = env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("parlo");
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("parlo")
}

pub fn path() -> PathBuf {
    if let Ok(configured) = env::var("PARLO_CONFIG") {
        return PathBuf::from(configured);
    }
    data_root().join("config.json")
}

pub fn load() -> Config {
    fs::read_to_string(path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(config: &Config) -> Result<PathBuf, String> {
    let path = path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
    fs::write(&path, text + "\n").map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_carry_the_documented_shortcut() {
        assert_eq!(Config::default().shortcut, DEFAULT_SHORTCUT);
    }

    #[test]
    fn a_missing_shortcut_falls_back_to_the_default() {
        let config: Config = serde_json::from_str("{\"port\": 9000}").unwrap();
        assert_eq!(config.port, 9000);
        assert_eq!(config.shortcut, DEFAULT_SHORTCUT);
    }

    #[test]
    fn the_path_ends_in_config_json() {
        assert!(path().ends_with("config.json"));
    }
}
