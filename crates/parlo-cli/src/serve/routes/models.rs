use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::{json_string, Flow, Session};
use crate::models;
use crate::serve::http::Stream;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    session: &Session,
) -> Option<Result<Flow, String>> {
    if method == "GET" && path == "/v1/models" {
        let data: Vec<serde_json::Value> = models::status()
            .iter()
            .map(|model| {
                let spec = crate::registry::model(model.id);
                serde_json::json!({
                    "id": model.id,
                    "object": "model",
                    "owned_by": "parlo",
                    "kind": model.kind,
                    "feature": model.feature,
                    "installed": model.installed,
                    "licence": spec.map(|spec| spec.licence).unwrap_or_default(),
                    "languages": spec.map(|spec| spec.languages).unwrap_or(&[]),
                    "devices": spec.map(|spec| spec.devices).unwrap_or(&[]),
                    "bytes": spec.map(|spec| spec.bytes).unwrap_or(0),
                    "capabilities": spec.map(|spec| spec.capabilities).unwrap_or(&[]),
                    "default": spec.map(|spec| spec.default).unwrap_or(false),
                })
            })
            .collect();
        let body = serde_json::json!({ "object": "list", "data": data }).to_string();
        return Some(respond_json(stream, "200 OK", &body).map(|()| Flow::Serve));
    }

    if method == "GET" && path == "/models" {
        let lifecycle = session.lifecycle_snapshot();
        let pair = {
            let chosen = session.chosen();
            format!("{}-{}", chosen.from, chosen.to)
        };
        let body = models::as_json(&models::status(), &lifecycle, &pair, session.is_working());
        let mut document: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
        document["recommendations"] = crate::recommend::as_json();
        let body = document.to_string();
        return Some(respond_json(stream, "200 OK", &body).map(|()| Flow::Serve));
    }

    if method == "POST" && path == "/models" {
        return Some(install(
            stream,
            headers,
            token,
            || models::ready(&models::status()),
            "{\"error\":\"models are already installed\"}",
            &[],
        ));
    }

    if method == "POST" && path == "/models/dictation" {
        return Some(install(
            stream,
            headers,
            token,
            || models::dictation_ready(&models::status()),
            "{\"error\":\"dictation is already installed\"}",
            models::DICTATION_UNITS,
        ));
    }

    None
}

#[derive(serde::Deserialize)]
struct Policy {
    id: String,
    #[serde(default)]
    minutes: Option<u32>,
}

pub(crate) fn route_actions(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Option<Result<Flow, String>> {
    if !matches!(
        (method, path),
        (
            "POST",
            "/models/rest"
                | "/models/download"
                | "/models/relocate"
                | "/models/cancel"
                | "/models/policy"
        )
    ) && !(method == "POST"
        && [
            "/models/download/",
            "/models/rollback/",
            "/models/remove/",
            "/models/load/",
            "/models/unload/",
        ]
        .iter()
        .any(|prefix| path.starts_with(prefix)))
    {
        return None;
    }
    Some(handle_action(
        method, path, stream, headers, token, body, session,
    ))
}

fn handle_action(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Result<Flow, String> {
    if method == "POST" && path == "/models/rest" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        if let Err(error) = models::install(&models::remaining_units()) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        return respond_json(stream, "202 Accepted", "{\"started\":true}").map(|()| Flow::Serve);
    }

    if method == "POST" && path.starts_with("/models/download/") {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = &path["/models/download/".len()..];
        if ![
            "parakeet", "whisper", "voice-pt", "voice-en", "mt-pt-en", "mt-en-pt",
        ]
        .contains(&id)
        {
            return respond_json(stream, "404 Not Found", "{\"error\":\"unknown model\"}")
                .map(|()| Flow::Serve);
        }
        if let Err(error) = models::install(&[id]) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        return respond_json(stream, "202 Accepted", "{\"started\":true}").map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/models/download" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let units: Vec<String> = request
            .get("units")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if units.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"choose at least one model\"}",
            )
            .map(|()| Flow::Serve);
        }
        let known = [
            "parakeet", "whisper", "voice-pt", "voice-en", "mt-pt-en", "mt-en-pt",
        ];
        if let Some(unknown) = units.iter().find(|unit| !known.contains(&unit.as_str())) {
            return respond_json(
                stream,
                "404 Not Found",
                &format!("{{\"error\":\"unknown model {}\"}}", json_string(unknown)),
            )
            .map(|()| Flow::Serve);
        }
        let chosen: Vec<&str> = units.iter().map(String::as_str).collect();
        if let Err(error) = models::install(&chosen) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        return respond_json(stream, "202 Accepted", "{\"started\":true}").map(|()| Flow::Serve);
    }

    handle_action_rest(method, path, stream, headers, token, body, session)
}

fn handle_action_rest(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Result<Flow, String> {
    if method == "POST" && path == "/models/relocate" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":{\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = serde_json::from_slice(body).unwrap_or_default();
        let target = request["path"].as_str().unwrap_or_default();
        if !request["confirm"].as_bool().unwrap_or(false) {
            return match models::relocate_plan(target) {
                Ok((needed, adopt)) => {
                    let size = models::describe_bytes(needed);
                    let message = if adopt {
                        format!(
                            "{size} are already in that folder. Press again to use them and remove the old copy."
                        )
                    } else {
                        format!(
                            "{size} will be copied to that folder. Nothing is deleted until the copy matches."
                        )
                    };
                    respond_json(
                        stream,
                        "200 OK",
                        &serde_json::json!({ "needed": needed, "adopt": adopt, "message": message })
                            .to_string(),
                    )
                    .map(|()| Flow::Serve)
                }
                Err(error) => respond_json(stream, "400 Bad Request", &error_json(&error))
                    .map(|()| Flow::Serve),
            };
        }
        return match models::relocate(target) {
            Ok(message) => respond_json(
                stream,
                "200 OK",
                &serde_json::json!({ "message": message }).to_string(),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path.starts_with("/models/rollback/") {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", &error_json("wrong token"))
                .map(|()| Flow::Serve);
        }
        let id = &path["/models/rollback/".len()..];
        return match models::rollback(id) {
            Ok(message) => respond_json(
                stream,
                "200 OK",
                &serde_json::json!({ "message": message }).to_string(),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    handle_action_tail(method, path, stream, headers, token, body, session)
}

fn handle_action_tail(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Result<Flow, String> {
    if method == "POST" && path == "/models/cancel" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match models::cancel() {
            Ok(()) => respond_json(
                stream,
                "200 OK",
                "{\"cancelled\":true,\"message\":\"Stopped. What already arrived stays on this machine.\"}",
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "409 Conflict", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path.starts_with("/models/remove/") {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = &path["/models/remove/".len()..];
        return match models::remove(id) {
            Ok(message) => respond_json(
                stream,
                "200 OK",
                &format!(
                    "{{\"removed\":true,\"message\":\"{}\"}}",
                    json_string(&message)
                ),
            )
            .map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST"
        && (path.starts_with("/models/load/") || path.starts_with("/models/unload/"))
    {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let loading = path.starts_with("/models/load/");
        let id = if loading {
            &path["/models/load/".len()..]
        } else {
            &path["/models/unload/".len()..]
        };
        let Some(unit) = models::unit_of(id) else {
            return respond_json(stream, "404 Not Found", "{\"error\":\"unknown model\"}")
                .map(|()| Flow::Serve);
        };
        if session.is_working() {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"a task is running; change this when it finishes\"}",
            )
            .map(|()| Flow::Serve);
        }
        let outcome = if loading {
            session.load_unit(unit)
        } else {
            session.unload_unit(unit)
        };
        return match outcome {
            Ok(()) => {
                let state = if loading { "loaded" } else { "unloaded" };
                let message = if loading {
                    "Loaded. It stays in memory until it sits idle too long or you unload it."
                } else {
                    "Freed from memory. The files stay on disk."
                };
                respond_json(
                    stream,
                    "200 OK",
                    &format!(
                        "{{\"model\":\"{}\",\"state\":\"{state}\",\"message\":\"{message}\"}}",
                        json_string(id)
                    ),
                )
                .map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/models/policy" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: Policy = match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        return match models::set_policy(&request.id, request.minutes) {
            Ok(message) => respond_json(
                stream,
                "200 OK",
                &format!("{{\"message\":\"{}\"}}", json_string(&message)),
            )
            .map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    respond_json(stream, "404 Not Found", &error_json("unknown model route")).map(|()| Flow::Serve)
}

fn install(
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    installed: impl FnOnce() -> bool,
    conflict: &str,
    units: &[&str],
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }
    if installed() {
        return respond_json(stream, "409 Conflict", conflict).map(|()| Flow::Serve);
    }
    if let Err(error) = models::install(units) {
        return respond_json(
            stream,
            "500 Internal Server Error",
            &format!("{{\"error\":\"{}\"}}", json_string(&error)),
        )
        .map(|()| Flow::Serve);
    }
    respond_json(stream, "202 Accepted", "{\"started\":true}").map(|()| Flow::Serve)
}
