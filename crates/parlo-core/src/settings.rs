use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::shortcut::HANDS_FREE as DEFAULT_HANDS_FREE;
use crate::shortcut::{
    CAPTIONS as DEFAULT_CAPTIONS, DEFAULT as DEFAULT_SHORTCUT, UNDO as DEFAULT_UNDO,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub engine: String,
    pub stt: String,
    pub from: String,
    pub to: String,
    pub write: String,
    pub style: String,
    pub profiles: BTreeMap<String, Profile>,
    pub shortcut: String,
    pub hands_free: String,
    pub captions: String,
    pub undo: String,
    pub name: String,
    pub history_days: u32,
    pub port: u16,
    pub update_channel: String,
    pub share: bool,
    pub share_pin: String,
    pub share_port: u16,
    pub remote_url: String,
    pub remote_token: String,
    pub remote_ca: String,
    pub locale: String,
    pub device: Option<usize>,
    pub compute: String,
    pub unload: u32,
    pub manage_memory: bool,
    pub policies: BTreeMap<String, u32>,
    pub models_dir: String,
    pub intelligence: Intelligence,
    pub features: Features,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Profile {
    pub style: String,
    pub write: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ProfileForm {
    Style(String),
    Fields {
        #[serde(default)]
        style: String,
        #[serde(default)]
        write: String,
    },
}

impl<'de> Deserialize<'de> for Profile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match ProfileForm::deserialize(deserializer)? {
            ProfileForm::Style(style) => Profile {
                style,
                write: String::new(),
            },
            ProfileForm::Fields { style, write } => Profile { style, write },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Features {
    pub translation: bool,
    pub voice: bool,
    pub captions: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            translation: true,
            voice: true,
            captions: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Intelligence {
    pub mode: String,
    pub clean: String,
    pub allow_cloud: bool,
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub local_url: String,
    pub local_model: String,
}

impl Default for Intelligence {
    fn default() -> Self {
        Self {
            mode: "auto".to_string(),
            clean: "light".to_string(),
            allow_cloud: false,
            provider: String::new(),
            model: String::new(),
            base_url: String::new(),
            local_url: String::new(),
            local_model: String::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            engine: "cascade".to_string(),
            stt: "parakeet".to_string(),
            from: "en".to_string(),
            to: "pt".to_string(),
            write: String::new(),
            style: String::new(),
            profiles: BTreeMap::new(),
            shortcut: DEFAULT_SHORTCUT.to_string(),
            hands_free: DEFAULT_HANDS_FREE.to_string(),
            captions: DEFAULT_CAPTIONS.to_string(),
            undo: DEFAULT_UNDO.to_string(),
            name: String::new(),
            history_days: 0,
            port: 8787,
            update_channel: "stable".to_string(),
            share: false,
            share_pin: String::new(),
            share_port: 8788,
            remote_url: String::new(),
            remote_token: String::new(),
            remote_ca: String::new(),
            locale: "en".to_string(),
            device: None,
            compute: "auto".to_string(),
            unload: 30,
            manage_memory: true,
            policies: BTreeMap::new(),
            models_dir: String::new(),
            intelligence: Intelligence::default(),
            features: Features::default(),
        }
    }
}

pub fn check_pin(value: &str) -> Result<String, String> {
    let pin = value.trim();
    if (4..=8).contains(&pin.len()) && pin.chars().all(|digit| digit.is_ascii_digit()) {
        Ok(pin.to_string())
    } else {
        Err("the PIN must be four to eight digits".to_string())
    }
}

pub fn check_compute(value: &str) -> Result<String, String> {
    let wanted = value.trim().to_ascii_lowercase();
    match wanted.as_str() {
        "auto" | "cpu" => Ok(wanted),
        _ => Err("the processor setting is either auto or cpu".to_string()),
    }
}

pub fn partner_language(language: &str) -> String {
    if language.eq_ignore_ascii_case("en") {
        "pt".to_string()
    } else {
        "en".to_string()
    }
}

pub fn system_language() -> String {
    #[cfg(windows)]
    {
        extern "system" {
            fn GetUserDefaultUILanguage() -> u16;
        }
        let language = unsafe { GetUserDefaultUILanguage() } as u32;
        if let Some(code) = language_code(language & 0x3ff) {
            return code.to_string();
        }
    }
    for name in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = env::var(name) {
            let code = value
                .split(['_', '.', '-', '@'])
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if code.len() == 2 && code.chars().all(|letter| letter.is_ascii_alphabetic()) {
                return code;
            }
        }
    }
    "en".to_string()
}

#[cfg(windows)]
fn language_code(primary: u32) -> Option<&'static str> {
    Some(match primary {
        0x09 => "en",
        0x07 => "de",
        0x0a => "es",
        0x0c => "fr",
        0x10 => "it",
        0x13 => "nl",
        0x15 => "pl",
        0x16 => "pt",
        0x19 => "ru",
        0x1d => "sv",
        _ => return None,
    })
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
    fn features_start_on_and_round_trip() {
        let features = Features::default();
        assert!(features.translation && features.voice && features.captions);
        let text = serde_json::to_string(&features).unwrap();
        assert_eq!(serde_json::from_str::<Features>(&text).unwrap(), features);
    }

    #[test]
    fn a_config_without_features_keeps_them_on() {
        let config: Config = serde_json::from_str("{\"port\": 9000}").unwrap();
        assert!(config.features.translation);
    }

    #[test]
    fn defaults_carry_the_documented_shortcut() {
        assert_eq!(Config::default().shortcut, DEFAULT_SHORTCUT);
    }

    #[test]
    fn the_defaults_carry_the_three_documented_shortcuts() {
        let config = Config::default();
        assert_eq!(config.hands_free, DEFAULT_HANDS_FREE);
        assert_eq!(config.captions, DEFAULT_CAPTIONS);
    }

    #[test]
    fn an_older_config_keeps_the_hands_free_and_captions_defaults() {
        let config: Config = serde_json::from_str("{\"shortcut\": \"Ctrl+Alt+K\"}").unwrap();
        assert_eq!(config.shortcut, "Ctrl+Alt+K");
        assert_eq!(config.hands_free, DEFAULT_HANDS_FREE);
        assert_eq!(config.captions, DEFAULT_CAPTIONS);
    }

    #[test]
    fn a_missing_shortcut_falls_back_to_the_default() {
        let config: Config = serde_json::from_str("{\"port\": 9000}").unwrap();
        assert_eq!(config.port, 9000);
        assert_eq!(config.shortcut, DEFAULT_SHORTCUT);
    }

    #[test]
    fn a_fresh_install_starts_on_the_two_main_languages() {
        let config = Config::default();
        assert_eq!(config.from, "en");
        assert_eq!(config.to, "pt");
        assert_ne!(config.from, config.to);
    }

    #[test]
    fn the_path_ends_in_config_json() {
        assert!(path().ends_with("config.json"));
    }

    #[test]
    fn an_older_config_keeps_the_cleanup_defaults() {
        let config: Config = serde_json::from_str("{\"from\": \"en\"}").unwrap();
        assert_eq!(config.intelligence.mode, "auto");
        assert_eq!(config.intelligence.clean, "light");
        assert!(!config.intelligence.allow_cloud);
    }

    #[test]
    fn the_interface_language_round_trips() {
        let mut config = Config::default();
        assert_eq!(config.locale, "en");
        config.locale = "pt".to_string();
        let text = serde_json::to_string(&config).unwrap();
        assert_eq!(serde_json::from_str::<Config>(&text).unwrap().locale, "pt");
    }

    #[test]
    fn an_older_config_speaks_english() {
        let config: Config = serde_json::from_str("{\"port\": 9000}").unwrap();
        assert_eq!(config.locale, "en");
    }

    #[test]
    fn a_remote_backend_round_trips() {
        let mut config = Config::default();
        assert!(config.remote_url.is_empty());
        config.remote_url = "https://desk.example:8788".to_string();
        config.remote_token = "secret".to_string();
        config.remote_ca = "desk.pem".to_string();
        let text = serde_json::to_string(&config).unwrap();
        let loaded: Config = serde_json::from_str(&text).unwrap();
        assert_eq!(loaded.remote_url, "https://desk.example:8788");
        assert_eq!(loaded.remote_token, "secret");
        assert_eq!(loaded.remote_ca, "desk.pem");
    }

    #[test]
    fn an_older_config_has_no_remote_backend() {
        let config: Config = serde_json::from_str("{\"port\": 9000}").unwrap();
        assert!(config.remote_url.is_empty());
        assert!(config.remote_token.is_empty());
        assert!(config.remote_ca.is_empty());
    }

    #[test]
    fn sharing_starts_off_and_round_trips() {
        let mut config = Config::default();
        assert!(!config.share);
        assert_eq!(config.share_port, 8788);
        config.share = true;
        config.share_pin = "4821".to_string();
        config.share_port = 9100;
        let text = serde_json::to_string(&config).unwrap();
        let loaded: Config = serde_json::from_str(&text).unwrap();
        assert!(loaded.share);
        assert_eq!(loaded.share_pin, "4821");
        assert_eq!(loaded.share_port, 9100);
    }

    #[test]
    fn an_older_config_shares_nothing() {
        let config: Config = serde_json::from_str("{\"port\": 9000}").unwrap();
        assert!(!config.share);
        assert!(config.share_pin.is_empty());
        assert_eq!(config.share_port, 8788);
    }

    #[test]
    fn a_pin_is_four_to_eight_digits() {
        assert_eq!(check_pin(" 1234 ").unwrap(), "1234");
        assert_eq!(check_pin("12345678").unwrap(), "12345678");
        assert!(check_pin("123").is_err());
        assert!(check_pin("123456789").is_err());
        assert!(check_pin("12a4").is_err());
        assert!(check_pin("").is_err());
    }

    #[test]
    fn the_update_channel_starts_stable_and_round_trips() {
        let mut config = Config::default();
        assert_eq!(config.update_channel, "stable");
        config.update_channel = "preview".to_string();
        let text = serde_json::to_string(&config).unwrap();
        let loaded: Config = serde_json::from_str(&text).unwrap();
        assert_eq!(loaded.update_channel, "preview");
    }

    #[test]
    fn an_older_config_updates_on_the_stable_channel() {
        let config: Config = serde_json::from_str("{\"port\": 9000}").unwrap();
        assert_eq!(config.update_channel, "stable");
    }

    #[test]
    fn the_cleanup_settings_survive_a_round_trip() {
        let mut config = Config::default();
        config.intelligence.provider = "openai".to_string();
        config.intelligence.allow_cloud = true;
        let text = serde_json::to_string(&config).unwrap();
        assert_eq!(serde_json::from_str::<Config>(&text).unwrap(), config);
    }
}
