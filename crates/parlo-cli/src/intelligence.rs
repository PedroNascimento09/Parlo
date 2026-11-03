use std::io::{Read, Write};
use std::process::{Command, Stdio};

use parlo_core::settings::Intelligence;
use serde_json::Value;

pub const MODES: [&str; 4] = ["off", "auto", "local", "cloud"];
pub const CLEAN: [&str; 3] = ["original", "light", "intelligent"];
pub const PROVIDERS: [&str; 4] = ["openai", "anthropic", "gemini", "compatible"];
const MAX_URL: usize = 200;
const MAX_MODEL: usize = 64;

pub fn check(settings: &Intelligence) -> Result<(), String> {
    if !MODES.contains(&settings.mode.as_str()) {
        return Err(format!(
            "unknown cleanup engine {}, expected one of {}",
            settings.mode,
            MODES.join(", ")
        ));
    }
    if !CLEAN.contains(&settings.clean.as_str()) {
        return Err(format!(
            "unknown cleanup amount {}, expected one of {}",
            settings.clean,
            CLEAN.join(", ")
        ));
    }
    if !settings.provider.is_empty() && !PROVIDERS.contains(&settings.provider.as_str()) {
        return Err(format!(
            "unknown provider {}, expected one of {}",
            settings.provider,
            PROVIDERS.join(", ")
        ));
    }
    check_model(&settings.model)?;
    check_model(&settings.local_model)?;
    check_url("the local model address", &settings.local_url)?;
    check_url("the compatible server address", &settings.base_url)?;
    if settings.provider == "compatible" && settings.base_url.is_empty() {
        return Err("the compatible provider needs a server address".to_string());
    }
    Ok(())
}

fn check_model(model: &str) -> Result<(), String> {
    if model.len() > MAX_MODEL {
        return Err("that model name is too long".to_string());
    }
    let allowed =
        |character: char| character.is_ascii_alphanumeric() || "._:/-".contains(character);
    if !model.chars().all(allowed) {
        return Err(format!(
            "{model} can only carry letters, digits, dots, dashes, colons and slashes"
        ));
    }
    Ok(())
}

fn check_url(label: &str, url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Ok(());
    }
    if url.len() > MAX_URL {
        return Err(format!("{label} is too long"));
    }
    if url.starts_with("https://") {
        return Ok(());
    }
    let Some(rest) = url.strip_prefix("http://") else {
        return Err(format!(
            "{label} must start with https://, or http:// for a server on this machine"
        ));
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_and_port = authority.rsplit('@').next().unwrap_or("");
    let host = match host_and_port.strip_prefix('[') {
        Some(rest) => rest.split(']').next().unwrap_or(""),
        None => host_and_port.split(':').next().unwrap_or(""),
    };
    if matches!(host, "localhost" | "127.0.0.1" | "::1") {
        Ok(())
    } else {
        Err(format!(
            "{label} over http:// is only allowed on this machine; use https:// for a server elsewhere"
        ))
    }
}

pub fn status() -> Value {
    run(&serde_json::json!({ "action": "status" }))
        .unwrap_or_else(|error| serde_json::json!({ "error": error }))
}

pub fn state_json(settings: &Intelligence, status: &Value) -> String {
    let mut state = serde_json::Map::new();
    state.insert(
        "settings".to_string(),
        serde_json::to_value(settings).unwrap_or(Value::Null),
    );
    for key in ["store", "providers", "error"] {
        if let Some(value) = status.get(key) {
            state.insert(key.to_string(), value.clone());
        }
    }
    Value::Object(state).to_string()
}

pub fn run(request: &Value) -> Result<Value, String> {
    let mut command = engine()?;
    let mut child = command
        .env("PARLO_MODELS_DIR", crate::models::models_root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("cannot start the engine: {error}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "the engine did not open its input".to_string())?;
    writeln!(stdin, "{request}").map_err(|error| format!("cannot reach the engine: {error}"))?;
    drop(stdin);
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "the engine did not open its output".to_string())?;
    let mut answer = String::new();
    stdout
        .read_to_string(&mut answer)
        .map_err(|error| format!("cannot read the engine: {error}"))?;
    child
        .wait()
        .map_err(|error| format!("the engine did not finish: {error}"))?;
    serde_json::from_str(answer.trim())
        .map_err(|error| format!("the engine answered with something else: {error}"))
}

fn engine() -> Result<Command, String> {
    let configured = crate::models::cascade_dir();
    let directory = std::path::absolute(&configured).unwrap_or(configured);
    let entry = directory.join("parlo_engine.py");
    if !entry.is_file() {
        return Err(format!(
            "cannot find {}; these settings need the engine beside Parlo",
            entry.display()
        ));
    }
    let configured = crate::sidecar::python_for(&directory);
    let python =
        std::path::absolute(&configured).unwrap_or_else(|_| std::path::PathBuf::from(configured));
    let mut command = Command::new(python);
    command.arg(&entry).current_dir(&directory);
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Intelligence {
        Intelligence::default()
    }

    #[test]
    fn the_defaults_are_accepted() {
        assert!(check(&settings()).is_ok());
    }

    #[test]
    fn an_unknown_mode_is_refused() {
        let wanted = Intelligence {
            mode: "smart".to_string(),
            ..settings()
        };
        assert!(check(&wanted).is_err());
    }

    #[test]
    fn an_unknown_cleanup_amount_is_refused() {
        let wanted = Intelligence {
            clean: "aggressive".to_string(),
            ..settings()
        };
        assert!(check(&wanted).is_err());
    }

    #[test]
    fn an_unknown_provider_is_refused() {
        let wanted = Intelligence {
            provider: "someone".to_string(),
            ..settings()
        };
        assert!(check(&wanted).is_err());
    }

    #[test]
    fn a_compatible_provider_needs_an_address() {
        let wanted = Intelligence {
            provider: "compatible".to_string(),
            ..settings()
        };
        assert!(check(&wanted).is_err());
    }

    #[test]
    fn loopback_over_plain_http_is_allowed() {
        for url in [
            "http://127.0.0.1:8080/v1/chat/completions",
            "http://localhost:1234/v1",
            "http://[::1]:8000/v1",
        ] {
            assert!(check(&Intelligence {
                local_url: url.to_string(),
                ..settings()
            })
            .is_ok());
        }
    }

    #[test]
    fn a_plain_connection_to_another_machine_is_refused() {
        for url in [
            "http://example.com/v1",
            "http://127.0.0.1.example.com/v1",
            "http://192.168.1.10:8080/v1",
        ] {
            assert!(check(&Intelligence {
                base_url: url.to_string(),
                ..settings()
            })
            .is_err());
        }
    }

    #[test]
    fn https_is_allowed_anywhere() {
        assert!(check(&Intelligence {
            provider: "compatible".to_string(),
            base_url: "https://api.example.com/v1".to_string(),
            ..settings()
        })
        .is_ok());
    }

    #[test]
    fn a_model_name_cannot_carry_spaces_or_quotes() {
        assert!(check(&Intelligence {
            model: "gpt 4o".to_string(),
            ..settings()
        })
        .is_err());
        assert!(check(&Intelligence {
            model: "gpt-4o-mini".to_string(),
            ..settings()
        })
        .is_ok());
    }
}
