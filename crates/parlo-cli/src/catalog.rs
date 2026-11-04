const CATALOG: &str = include_str!("../../../catalog/models.json");

pub fn artifacts() -> Vec<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(CATALOG)
        .ok()
        .and_then(|document| {
            document
                .get("artifacts")
                .and_then(|value| value.as_array())
                .cloned()
        })
        .unwrap_or_default()
}

pub fn as_json() -> String {
    let entries: Vec<serde_json::Value> = artifacts()
        .into_iter()
        .map(|artifact| {
            let text = |name: &str| {
                artifact
                    .get(name)
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let optional = |name: &str| {
                artifact
                    .get(name)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            };
            serde_json::json!({
                "id": text("id"),
                "name": text("name"),
                "kind": text("kind"),
                "category": text("category"),
                "purpose": text("purpose"),
                "version": text("version"),
                "license": text("license"),
                "status": text("status"),
                "source": text("source"),
                "attribution": text("attribution"),
                "languages": optional("languages"),
                "size_mb": optional("size_mb"),
                "memory_mb": optional("memory_mb"),
                "speed": optional("speed"),
                "quality": optional("quality"),
                "rtf": optional("rtf"),
                "pair": optional("pair"),
                "bench": optional("bench"),
                "wer": optional("wer"),
                "latency_ms": optional("latency_ms"),
                "unit": optional("unit"),
                "note": optional("note"),
            })
        })
        .collect();
    serde_json::json!(entries).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_artifact_belongs_to_a_known_category() {
        let known = ["dictation", "voice", "translation", "text", "speakers"];
        for artifact in artifacts() {
            let category = artifact
                .get("category")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            assert!(known.contains(&category), "{artifact:?}");
        }
    }

    #[test]
    fn the_embedded_catalog_is_readable() {
        let artifacts = artifacts();
        assert!(!artifacts.is_empty());
    }

    #[test]
    fn every_artifact_declares_an_id_a_licence_and_a_state() {
        for artifact in artifacts() {
            for name in ["id", "license", "status"] {
                let value = artifact.get(name).and_then(|field| field.as_str());
                assert!(
                    value.map(|text| !text.trim().is_empty()).unwrap_or(false),
                    "{name} is missing on {artifact:?}"
                );
            }
        }
    }

    #[test]
    fn every_artifact_says_what_it_does_and_which_languages_it_covers() {
        for artifact in artifacts() {
            for name in ["name", "purpose"] {
                let value = artifact.get(name).and_then(|field| field.as_str());
                assert!(
                    value.map(|text| !text.trim().is_empty()).unwrap_or(false),
                    "{name} is missing on {artifact:?}"
                );
            }
            let languages = artifact.get("languages").and_then(|value| value.as_array());
            assert!(
                languages.map(|list| !list.is_empty()).unwrap_or(false),
                "languages are missing on {artifact:?}"
            );
        }
    }

    #[test]
    fn the_page_receives_id_kind_and_licence_for_every_entry() {
        let entries: Vec<serde_json::Value> =
            serde_json::from_str(&as_json()).expect("the catalogue is valid json");
        assert!(!entries.is_empty());
        for entry in entries {
            for name in ["id", "kind", "license", "status", "purpose", "languages"] {
                assert!(entry.get(name).is_some(), "{name} is missing on {entry:?}");
            }
        }
    }
}
