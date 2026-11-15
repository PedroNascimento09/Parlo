use std::sync::OnceLock;

use serde_json::{Map, Value};

pub const DEFAULT: &str = "en";

const META: &str = include_str!("../web/locales/meta.json");

const STRINGS: &[(&str, &str)] = &[
    ("en", include_str!("../web/locales/en.json")),
    ("pt", include_str!("../web/locales/pt.json")),
    ("es", include_str!("../web/locales/es.json")),
    ("fr", include_str!("../web/locales/fr.json")),
    ("de", include_str!("../web/locales/de.json")),
    ("it", include_str!("../web/locales/it.json")),
    ("nl", include_str!("../web/locales/nl.json")),
    ("pl", include_str!("../web/locales/pl.json")),
    ("ru", include_str!("../web/locales/ru.json")),
    ("sv", include_str!("../web/locales/sv.json")),
    ("da", include_str!("../web/locales/da.json")),
    ("fi", include_str!("../web/locales/fi.json")),
    ("nb", include_str!("../web/locales/nb.json")),
    ("cs", include_str!("../web/locales/cs.json")),
    ("ro", include_str!("../web/locales/ro.json")),
    ("tr", include_str!("../web/locales/tr.json")),
    ("uk", include_str!("../web/locales/uk.json")),
    ("ja", include_str!("../web/locales/ja.json")),
    ("ko", include_str!("../web/locales/ko.json")),
    ("zh", include_str!("../web/locales/zh.json")),
    ("ar", include_str!("../web/locales/ar.json")),
];

fn meta() -> &'static Value {
    static META_VALUE: OnceLock<Value> = OnceLock::new();
    META_VALUE.get_or_init(|| serde_json::from_str(META).unwrap_or(Value::Null))
}

fn table(code: &str) -> Map<String, Value> {
    STRINGS
        .iter()
        .find(|(name, _)| *name == code)
        .and_then(|(_, text)| serde_json::from_str::<Value>(text).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

pub fn locales() -> Vec<String> {
    meta()
        .get("locales")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub fn available() -> Vec<(String, String)> {
    let meta = meta();
    locales()
        .into_iter()
        .map(|code| {
            let name = meta
                .pointer(&format!("/names/{code}"))
                .and_then(Value::as_str)
                .unwrap_or(&code)
                .to_string();
            (code, name)
        })
        .collect()
}

pub fn check(value: &str) -> Result<String, String> {
    let code = value.trim().to_ascii_lowercase();
    if locales().contains(&code) {
        Ok(code)
    } else {
        Err(format!(
            "unknown interface language {value}, expected one of {}",
            locales().join(", ")
        ))
    }
}

pub fn strings(locale: &str) -> Map<String, Value> {
    let wanted = check(locale).unwrap_or_else(|_| DEFAULT.to_string());
    let mut merged = table(DEFAULT);
    if wanted != DEFAULT {
        for (key, value) in table(&wanted) {
            merged.insert(key, value);
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_locale_has_every_key_and_nothing_is_empty() {
        let keys = strings("en").keys().cloned().collect::<Vec<String>>();
        assert!(!keys.is_empty());
        assert_eq!(locales().len(), 21);
        assert_eq!(STRINGS.len(), locales().len());
        for (code, _) in STRINGS {
            assert!(
                locales().contains(&code.to_string()),
                "{code} is not listed"
            );
        }
        for locale in locales() {
            let table = strings(&locale);
            for key in &keys {
                let value = table
                    .get(key)
                    .and_then(Value::as_str)
                    .unwrap_or_else(|| panic!("{locale} misses {key}"));
                assert!(!value.trim().is_empty(), "{locale} leaves {key} empty");
            }
            assert_eq!(table.len(), keys.len(), "{locale} carries an extra key");
        }
    }

    #[test]
    fn a_locale_really_changes_the_words() {
        assert_eq!(strings("en")["nav.home"], "Home");
        assert_eq!(strings("pt")["nav.home"], "Início");
        assert_eq!(strings("ja")["nav.settings"], "設定");
        assert_ne!(
            strings("pt")["settings.title"],
            strings("en")["settings.title"]
        );
    }

    #[test]
    fn an_unknown_locale_falls_back_to_english() {
        assert_eq!(strings("xx")["nav.home"], "Home");
        assert_eq!(check(" PT ").unwrap(), "pt");
        assert!(check("xx").is_err());
        assert_eq!(available().len(), 21);
        assert_eq!(available()[1].1, "Português");
    }
}
