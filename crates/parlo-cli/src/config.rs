use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub engine: String,
    pub from: String,
    pub to: String,
    pub port: u16,
    pub device: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            engine: "cascade".to_string(),
            from: "pt".to_string(),
            to: "en".to_string(),
            port: 8787,
            device: None,
        }
    }
}

pub fn path() -> PathBuf {
    if let Ok(configured) = env::var("PARLO_CONFIG") {
        return PathBuf::from(configured);
    }
    if cfg!(target_os = "windows") {
        if let Ok(appdata) = env::var("APPDATA") {
            return PathBuf::from(appdata).join("parlo").join("config.json");
        }
    }
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("parlo").join("config.json");
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("parlo")
        .join("config.json")
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
    fn defaults_match_the_documented_pair() {
        let config = Config::default();
        assert_eq!(config.engine, "cascade");
        assert_eq!(config.from, "pt");
        assert_eq!(config.to, "en");
        assert_eq!(config.port, 8787);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let config: Config = serde_json::from_str("{\"port\": 9000}").unwrap();
        assert_eq!(config.port, 9000);
        assert_eq!(config.engine, "cascade");
    }

    #[test]
    fn round_trips_through_json() {
        let config = Config {
            engine: "sidecar".to_string(),
            from: "en".to_string(),
            to: "pt".to_string(),
            port: 9100,
            device: Some(2),
        };
        let text = serde_json::to_string(&config).unwrap();
        assert_eq!(serde_json::from_str::<Config>(&text).unwrap(), config);
    }

    #[test]
    fn the_path_ends_in_config_json() {
        assert!(path().ends_with("config.json"));
    }
}
