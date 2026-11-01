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
            let field = |name: &str| {
                artifact
                    .get(name)
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            serde_json::json!({
                "id": field("id"),
                "kind": field("kind"),
                "version": field("version"),
                "license": field("license"),
                "status": field("status"),
                "source": field("source"),
                "attribution": field("attribution"),
            })
        })
        .collect();
    serde_json::json!(entries).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn the_page_receives_id_kind_and_licence_for_every_entry() {
        let entries: Vec<serde_json::Value> =
            serde_json::from_str(&as_json()).expect("the catalogue is valid json");
        assert!(!entries.is_empty());
        for entry in entries {
            for name in ["id", "kind", "license", "status"] {
                assert!(entry.get(name).is_some(), "{name} is missing on {entry:?}");
            }
        }
    }
}
