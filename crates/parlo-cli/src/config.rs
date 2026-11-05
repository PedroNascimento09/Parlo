pub use parlo_core::settings::{data_root, load, path, save};

#[cfg(test)]
pub use parlo_core::settings::Config;

pub const RECOGNISERS: [&str; 2] = ["parakeet", "whisper"];

pub fn check_stt(value: &str) -> Result<(), String> {
    if RECOGNISERS.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "unknown recogniser {value}, expected one of {}",
            RECOGNISERS.join(", ")
        ))
    }
}

pub fn check_language(value: &str) -> bool {
    value.len() == 2
        && value
            .chars()
            .all(|character| character.is_ascii_alphabetic())
}

pub fn check_pair(from: &str, to: &str) -> Result<(), String> {
    for (value, label) in [(from, "--from"), (to, "--to")] {
        if value.len() != 2
            || !value
                .chars()
                .all(|character| character.is_ascii_alphabetic())
        {
            return Err(format!(
                "{label} needs a two letter language code, got {value}"
            ));
        }
    }
    if from.eq_ignore_ascii_case(to) {
        return Err("the source and target language must differ".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_documented_pair() {
        let config = Config::default();
        assert_eq!(config.engine, "cascade");
        assert_eq!(config.from, parlo_core::settings::system_language());
        assert_eq!(
            config.to,
            parlo_core::settings::partner_language(&config.from)
        );
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
            stt: "whisper".to_string(),
            from: "en".to_string(),
            to: "pt".to_string(),
            write: "pt".to_string(),
            style: String::new(),
            profiles: Default::default(),
            shortcut: "Ctrl+Shift+D".to_string(),
            hands_free: "Ctrl+Alt+Shift+D".to_string(),
            captions: "Ctrl+Alt+C".to_string(),
            undo: "Ctrl+Alt+Z".to_string(),
            name: "Ana".to_string(),
            history_days: 30,
            port: 9100,
            device: Some(2),
            compute: "auto".to_string(),
            unload: 15,
            manage_memory: true,
            policies: Default::default(),
            models_dir: String::new(),
            intelligence: Default::default(),
            features: Default::default(),
        };
        let text = serde_json::to_string(&config).unwrap();
        assert_eq!(serde_json::from_str::<Config>(&text).unwrap(), config);
    }

    #[test]
    fn accepts_only_two_letter_pairs_that_differ() {
        assert!(check_pair("pt", "en").is_ok());
        assert!(check_pair("pt", "pt").is_err());
        assert!(check_pair("pt", "eng").is_err());
        assert!(check_pair("p1", "en").is_err());
    }

    #[test]
    fn accepts_only_two_letter_languages() {
        assert!(check_language("en"));
        assert!(!check_language(""));
        assert!(!check_language("eng"));
        assert!(!check_language("e1"));
    }

    #[test]
    fn accepts_only_known_recognisers() {
        assert!(check_stt("parakeet").is_ok());
        assert!(check_stt("whisper").is_ok());
        assert!(check_stt("hibiki").is_err());
    }

    #[test]
    fn the_path_ends_in_config_json() {
        assert!(path().ends_with("config.json"));
    }
}
