use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::respond_named;
use super::super::{text_field, Flow};
use crate::serve::http::Stream;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    let known = matches!(
        (method, path),
        ("GET", "/vocabulary" | "/vocabulary/export")
            | ("POST", "/vocabulary" | "/vocabulary/delete" | "/vocabulary/accept" | "/vocabulary/import")
    );
    if !known {
        return None;
    }
    if !authorised(headers, token) {
        return Some(
            respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve),
        );
    }
    Some(match (method, path) {
        ("GET", "/vocabulary") => list(stream),
        ("GET", "/vocabulary/export") => export(stream),
        ("POST", "/vocabulary") => edit(stream, body, "vocabulary/add"),
        ("POST", "/vocabulary/delete") => edit(stream, body, "vocabulary/delete"),
        ("POST", "/vocabulary/accept") => accept(stream, body),
        _ => import(stream, body),
    })
}

fn list(stream: &mut dyn Stream) -> Result<Flow, String> {
    match crate::intelligence::run(&serde_json::json!({ "action": "vocabulary" })) {
        Ok(reply) => respond_json(stream, "200 OK", &reply.to_string()).map(|()| Flow::Serve),
        Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
            .map(|()| Flow::Serve),
    }
}

fn export(stream: &mut dyn Stream) -> Result<Flow, String> {
    match crate::intelligence::run(&serde_json::json!({ "action": "vocabulary/export" })) {
        Ok(reply) => {
            let text = reply
                .get("text")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            respond_named(stream, "parlo-words.tsv", text.as_bytes()).map(|()| Flow::Serve)
        }
        Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
            .map(|()| Flow::Serve),
    }
}

fn edit(stream: &mut dyn Stream, body: &[u8], action: &str) -> Result<Flow, String> {
    let request: serde_json::Value = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(error) => {
            return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                .map(|()| Flow::Serve);
        }
    };
    let field = |name: &str| {
        request
            .get(name)
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let payload = serde_json::json!({
        "action": action,
        "kind": field("kind"),
        "wrong": field("wrong"),
        "right": field("right"),
    });
    run_and_reply(stream, &payload)
}

fn accept(stream: &mut dyn Stream, body: &[u8]) -> Result<Flow, String> {
    let request: serde_json::Value = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(error) => {
            return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                .map(|()| Flow::Serve);
        }
    };
    let heard = text_field(&request, "heard");
    let written = text_field(&request, "written");
    if heard.is_empty() || written.is_empty() {
        return respond_json(
            stream,
            "400 Bad Request",
            "{\"error\":\"a suggestion names what was heard and what to write\"}",
        )
        .map(|()| Flow::Serve);
    }
    let payload = serde_json::json!({
        "action": "vocabulary/add",
        "kind": "term",
        "wrong": heard,
        "right": written,
    });
    run_and_reply(stream, &payload)
}

fn import(stream: &mut dyn Stream, body: &[u8]) -> Result<Flow, String> {
    let request: serde_json::Value = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(error) => {
            return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                .map(|()| Flow::Serve);
        }
    };
    let text = text_field(&request, "text");
    if text.is_empty() {
        return respond_json(
            stream,
            "400 Bad Request",
            "{\"error\":\"an import needs at least one line\"}",
        )
        .map(|()| Flow::Serve);
    }
    let payload = serde_json::json!({ "action": "vocabulary/import", "text": text });
    run_and_reply(stream, &payload)
}

fn run_and_reply(stream: &mut dyn Stream, payload: &serde_json::Value) -> Result<Flow, String> {
    match crate::intelligence::run(payload) {
        Ok(reply) => match reply.get("error").and_then(serde_json::Value::as_str) {
            Some(error) => {
                respond_json(stream, "400 Bad Request", &error_json(error)).map(|()| Flow::Serve)
            }
            None => respond_json(stream, "200 OK", &reply.to_string()).map(|()| Flow::Serve),
        },
        Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
            .map(|()| Flow::Serve),
    }
}
