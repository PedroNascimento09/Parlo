use crate::registry::{self, Source};

pub const PRESENTED: [&str; 21] = [
    "en", "pt", "es", "fr", "de", "it", "nl", "pl", "ru", "sv", "da", "fi", "nb", "cs", "ro", "tr",
    "uk", "ja", "ko", "zh", "ar",
];

pub const PARAKEET_LANGUAGES: [&str; 25] = [
    "bg", "hr", "cs", "da", "nl", "en", "et", "fi", "fr", "de", "el", "hu", "it", "lv", "lt", "mt",
    "pl", "pt", "ro", "sk", "sl", "es", "sv", "ru", "uk",
];

pub struct Pick {
    pub feature: &'static str,
    pub model: &'static str,
}

pub fn dictation(language: &str) -> Option<&'static str> {
    if !PRESENTED.contains(&language) {
        return None;
    }
    if PARAKEET_LANGUAGES.contains(&language) {
        return Some("parakeet");
    }
    Some("whisper")
}

pub fn picks(language: &str) -> Vec<Pick> {
    let mut picks = Vec::new();
    if let Some(spec) = registry::MODELS
        .iter()
        .find(|spec| spec.kind == "voice" && spec.languages.contains(&language))
    {
        picks.push(Pick {
            feature: "voice",
            model: spec.id,
        });
    }
    if let Some(model) = dictation(language) {
        picks.push(Pick {
            feature: "dictation",
            model,
        });
    }
    let translation = registry::MODELS
        .iter()
        .find(|spec| matches!(spec.source, Source::Translation(from, _) if from == language))
        .or_else(|| {
            registry::MODELS
                .iter()
                .find(|spec| spec.kind == "translation" && spec.languages.contains(&language))
        });
    if let Some(spec) = translation {
        picks.push(Pick {
            feature: "translation",
            model: spec.id,
        });
    }
    picks
}

#[cfg(test)]
pub fn covered() -> Vec<&'static str> {
    PRESENTED
        .iter()
        .copied()
        .filter(|language| picks(language).iter().any(|pick| pick.feature == "voice"))
        .collect()
}

pub fn as_json() -> serde_json::Value {
    let rows: Vec<serde_json::Value> = PRESENTED
        .iter()
        .map(|language| {
            let picks: Vec<serde_json::Value> = picks(language)
                .iter()
                .map(|pick| {
                    let name = registry::model(pick.model)
                        .map(|spec| spec.name)
                        .unwrap_or(pick.model);
                    serde_json::json!({
                        "feature": pick.feature,
                        "model": pick.model,
                        "name": name,
                    })
                })
                .collect();
            serde_json::json!({ "language": language, "picks": picks })
        })
        .collect();
    serde_json::Value::Array(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_presented_languages_are_the_ones_the_app_shows() {
        let mut ours = PRESENTED.to_vec();
        ours.sort_unstable();
        let mut theirs = crate::locales::locales();
        theirs.sort();
        assert_eq!(ours, theirs);
    }

    #[test]
    fn every_pick_names_a_model_the_app_offers() {
        for language in PRESENTED {
            for pick in picks(language) {
                let spec = registry::model(pick.model)
                    .unwrap_or_else(|| panic!("{language} recommends {}", pick.model));
                if pick.feature != "dictation" {
                    assert_eq!(spec.kind, pick.feature);
                    assert!(
                        spec.languages.contains(&language),
                        "{} does not cover {language}",
                        pick.model
                    );
                }
            }
        }
    }

    #[test]
    fn the_translation_pick_starts_from_the_language_the_user_chose() {
        for language in ["pt", "en", "es", "de", "fr"] {
            let pick = picks(language)
                .into_iter()
                .find(|pick| pick.feature == "translation")
                .unwrap_or_else(|| panic!("{language} has no translation"));
            let spec = registry::model(pick.model).expect("an offered model");
            if let Source::Translation(from, _) = spec.source {
                assert_eq!(from, language, "{} starts elsewhere", pick.model);
            }
        }
    }

    #[test]
    fn every_presented_language_can_be_dictated() {
        for language in PRESENTED {
            assert!(
                dictation(language).is_some(),
                "{language} cannot be dictated"
            );
        }
    }

    #[test]
    fn a_language_outside_the_app_gets_no_recommendation() {
        assert!(picks("xx").is_empty());
        assert!(dictation("xx").is_none());
    }

    #[test]
    fn every_translated_language_has_a_voice() {
        let covered = covered();
        for language in ["pt", "en", "es", "fr", "de", "it", "nl", "ru"] {
            assert!(covered.contains(&language), "{language} has no voice");
        }
    }

    #[test]
    fn the_report_names_every_presented_language_once() {
        let report = as_json();
        let rows = report.as_array().expect("a list of languages");
        assert_eq!(rows.len(), PRESENTED.len());
        assert!(rows.iter().all(|row| row["picks"]
            .as_array()
            .map(|p| !p.is_empty())
            .unwrap_or(false)));
    }
}
