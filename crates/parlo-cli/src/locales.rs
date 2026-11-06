use serde_json::{Map, Value};

pub const DEFAULT: &str = "en";

const TABLE: &str = include_str!("../web/locales.json");

fn root() -> Value {
    serde_json::from_str(TABLE).unwrap_or(Value::Null)
}

pub fn locales() -> Vec<String> {
    root()
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
    let root = root();
    locales()
        .into_iter()
        .map(|code| {
            let name = root
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
    let root = root();
    let mut merged = root
        .pointer("/strings/en")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let wanted = check(locale).unwrap_or_else(|_| DEFAULT.to_string());
    if let Some(chosen) = root
        .pointer(&format!("/strings/{wanted}"))
        .and_then(Value::as_object)
    {
        for (key, value) in chosen {
            merged.insert(key.clone(), value.clone());
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
