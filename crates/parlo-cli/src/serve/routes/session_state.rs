use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use parlo_core::state::Machine;

use super::super::http::{error_json, respond_json, Stream};
use super::super::{authorised, json_string, ticket, ws, Flow, Session};

static STATE: Mutex<Machine> = Mutex::new(Machine::new());

fn recovery_request(
    method: &str,
    body: &[u8],
    path: &std::path::Path,
) -> Result<serde_json::Value, String> {
    if !matches!(method, "GET" | "POST") {
        return Err("the recovery request is not supported".to_string());
    }
    let request = if method == "POST" {
        let value: serde_json::Value =
            serde_json::from_slice(body).map_err(|error| error.to_string())?;
        let text = value
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or("recovery needs text")?;
        let partial = value
            .get("partial")
            .and_then(|v| v.as_bool())
            .ok_or("recovery needs a partial flag")?;
        Some((text.to_string(), partial))
    } else {
        None
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(path)
            .map_err(|error| error.to_string())?;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;
    }
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|error| error.to_string())?;
    connection.execute_batch("PRAGMA secure_delete = ON; CREATE TABLE IF NOT EXISTS pending (id INTEGER PRIMARY KEY CHECK (id = 1), text TEXT NOT NULL, partial INTEGER NOT NULL);").map_err(|error| error.to_string())?;
    if let Some((text, partial)) = request {
        if text.is_empty() {
            connection
                .execute("DELETE FROM pending WHERE id = 1", [])
                .map_err(|error| error.to_string())?;
        } else {
            connection.execute("INSERT INTO pending (id, text, partial) VALUES (1, ?1, ?2) ON CONFLICT(id) DO UPDATE SET text=excluded.text, partial=excluded.partial", params![text, partial]).map_err(|error| error.to_string())?;
        }
    }
    let pending: Option<(String, bool)> = connection
        .query_row(
            "SELECT text, partial FROM pending WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    Ok(match pending {
        Some((text, partial)) => serde_json::json!({"text": text, "partial": partial}),
        None => serde_json::json!({"text": "", "partial": false}),
    })
}

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Option<Result<Flow, String>> {
    if path == "/recovery" {
        if !authorised(headers, token) {
            return Some(
                respond_json(stream, "401 Unauthorized", &error_json("wrong token"))
                    .map(|()| Flow::Serve),
            );
        }
        let result = recovery_request(
            method,
            body,
            &crate::config::data_root().join("recovery.db"),
        );
        return Some(
            match result {
                Ok(answer) => respond_json(stream, "200 OK", &answer.to_string()),
                Err(error) => {
                    respond_json(stream, "500 Internal Server Error", &error_json(&error))
                }
            }
            .map(|()| Flow::Serve),
        );
    }

    if method == "POST" && path == "/ticket" {
        if !authorised(headers, token) {
            return Some(
                respond_json(stream, "401 Unauthorized", &error_json("wrong token"))
                    .map(|()| Flow::Serve),
            );
        }
        let (granted, lifetime) = match ticket::issue() {
            Ok(ticket) => ticket,
            Err(error) => {
                return Some(
                    respond_json(stream, "500 Internal Server Error", &error_json(&error))
                        .map(|()| Flow::Serve),
                );
            }
        };
        let answer = serde_json::json!({
            "ticket": granted,
            "expires_in": lifetime,
            "protocol": ws::PROTOCOL,
        })
        .to_string();
        return Some(respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve));
    }

    if method == "GET" && path == "/target" {
        if !authorised(headers, token) {
            return Some(
                respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                    .map(|()| Flow::Serve),
            );
        }
        let answer = match session.target_app() {
            Some(app) => format!("{{\"insertable\":false,\"app\":\"{}\",\"reason\":\"Parlo cannot verify the focused field. Copy the text and paste it yourself.\"}}", json_string(&app)),
            None => format!(
                "{{\"insertable\":false,\"reason\":\"{}\"}}",
                json_string("Parlo does not know which field to write into")
            ),
        };
        return Some(respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve));
    }

    if path == "/state" {
        let mut machine = match STATE.lock() {
            Ok(machine) => machine,
            Err(_) => {
                return Some(
                    respond_json(
                        stream,
                        "500 Internal Server Error",
                        "{\"error\":\"the state is unavailable\"}",
                    )
                    .map(|()| Flow::Serve),
                );
            }
        };
        if method == "POST" {
            let next = serde_json::from_slice::<serde_json::Value>(body)
                .ok()
                .and_then(|request| {
                    request
                        .get("state")
                        .and_then(serde_json::Value::as_str)
                        .and_then(parlo_core::state::State::parse)
                });
            let Some(next) = next else {
                return Some(
                    respond_json(stream, "400 Bad Request", "{\"error\":\"unknown state\"}")
                        .map(|()| Flow::Serve),
                );
            };
            if let Err(error) = machine.enter(next) {
                return Some(
                    respond_json(stream, "409 Conflict", &error_json(&error)).map(|()| Flow::Serve),
                );
            }
        }
        let answer = format!("{{\"state\":\"{}\"}}", machine.state().as_str());
        return Some(respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve));
    }

    if method == "GET" && path == "/engine" {
        let stored_compute = crate::config::load().compute;
        let gpu = crate::engines::detect_gpu_memory_mb();
        let choice = crate::engines::choose_preferred(
            crate::engines::detect_cores(),
            gpu,
            crate::engines::Fidelity::Cloned,
            &stored_compute,
            &parlo_core::device::profile().describe(),
        );
        let logged = crate::engines::log_choice(&choice);
        let steps: Vec<&str> = crate::engines::chain(choice.requested)
            .iter()
            .map(|fidelity| fidelity.label())
            .collect();
        let body = serde_json::json!({
            "profile": choice.profile.label(),
            "requested": choice.requested.label(),
            "chosen": choice.chosen.label(),
            "chain": steps,
            "reason": choice.reason,
            "compute": stored_compute,
            "device": crate::engines::device_label(&stored_compute, gpu),
            "log": logged.map(|path| path.display().to_string()),
            "switches": crate::engines::recent_switches(5),
        })
        .to_string();
        return Some(respond_json(stream, "200 OK", &body).map(|()| Flow::Serve));
    }

    None
}

#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[test]
    fn saved_text_survives_reopen_and_can_be_discarded() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!("recovery-test-{}.db", std::process::id()));
        let initial = recovery_request("GET", b"", &path).unwrap();
        assert_eq!(initial["text"], "");
        recovery_request(
            "POST",
            br#"{"text":"First sentence. Second.","partial":true}"#,
            &path,
        )
        .unwrap();
        let restored = recovery_request("GET", b"", &path).unwrap();
        assert_eq!(restored["text"], "First sentence. Second.");
        assert_eq!(restored["partial"], true);
        assert!(recovery_request("POST", br#"{"text":"other"}"#, &path).is_err());
        assert_eq!(
            recovery_request("GET", b"", &path).unwrap()["text"],
            "First sentence. Second."
        );
        recovery_request("POST", br#"{"text":"","partial":false}"#, &path).unwrap();
        assert_eq!(recovery_request("GET", b"", &path).unwrap()["text"], "");
        std::fs::remove_file(path).unwrap();
    }
}
