use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static CURRENT: OnceLock<Mutex<String>> = OnceLock::new();

pub fn remember(client: &str) -> Result<(), String> {
    let id = clean(client)?;
    let slot = CURRENT.get_or_init(|| Mutex::new(String::new()));
    let mut held = slot
        .lock()
        .map_err(|_| "the agent name is locked".to_string())?;
    *held = id;
    Ok(())
}

pub fn current() -> Option<String> {
    let held = CURRENT.get()?.lock().ok()?;
    let id = held.trim().to_string();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

pub fn path() -> PathBuf {
    crate::config::data_root().join("agents.json")
}

pub fn bind(client: &str, voice: &str) -> Result<BTreeMap<String, String>, String> {
    bind_at(&path(), client, voice)
}

pub fn voice_for(client: &str) -> Option<String> {
    voice_at(&path(), client)
}

pub fn bindings_at(path: &Path) -> BTreeMap<String, String> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn voice_at(path: &Path, client: &str) -> Option<String> {
    bindings_at(path).remove(client)
}

pub fn bind_at(path: &Path, client: &str, voice: &str) -> Result<BTreeMap<String, String>, String> {
    let client = clean(client)?;
    let voice = voice_id(voice)?;
    let mut bindings = bindings_at(path);
    bindings.insert(client, voice);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(&bindings).map_err(|error| error.to_string())?;
    fs::write(path, text + "\n").map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(bindings)
}

fn clean(client: &str) -> Result<String, String> {
    let id = client.trim();
    let valid = !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        });
    if !valid {
        return Err(
            "an agent name is up to 64 letters, digits, dots, dashes or underscores".to_string(),
        );
    }
    Ok(id.to_string())
}

fn voice_id(voice: &str) -> Result<String, String> {
    let id = voice.trim();
    match crate::registry::model(id) {
        Some(spec) if spec.kind == "voice" => Ok(id.to_string()),
        _ => Err(format!("{id} is not a voice this app can read aloud with")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("parlo-agents-{name}-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        folder.join("agents.json")
    }

    #[test]
    fn a_client_resolves_to_the_voice_it_was_bound_to() {
        let path = scratch("bind");
        bind_at(&path, "claude-code", "voice-de").unwrap();
        assert_eq!(voice_at(&path, "claude-code").as_deref(), Some("voice-de"));
        assert_eq!(voice_at(&path, "other-agent"), None);
        bind_at(&path, "other-agent", "voice-en").unwrap();
        assert_eq!(voice_at(&path, "claude-code").as_deref(), Some("voice-de"));
        assert_eq!(voice_at(&path, "other-agent").as_deref(), Some("voice-en"));
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn a_binding_refuses_a_model_that_is_not_a_voice() {
        let path = scratch("refuse");
        assert!(bind_at(&path, "claude-code", "parakeet").is_err());
        assert!(bind_at(&path, "claude-code", "").is_err());
        assert!(bindings_at(&path).is_empty());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn an_agent_name_cannot_be_a_path_or_an_empty_string() {
        let path = scratch("name");
        for client in ["", "  ", "../escape", "a/b", &"x".repeat(65)] {
            assert!(bind_at(&path, client, "voice-en").is_err(), "{client}");
        }
        assert!(bind_at(&path, "claude-code", "voice-en").is_ok());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn the_session_keeps_the_client_that_said_its_name() {
        remember("cursor").unwrap();
        assert_eq!(current().as_deref(), Some("cursor"));
        assert!(remember("").is_err());
        assert_eq!(current().as_deref(), Some("cursor"));
        remember("claude-code").unwrap();
        assert_eq!(current().as_deref(), Some("claude-code"));
    }
}
