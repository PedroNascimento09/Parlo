use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::{text_field, Flow, Session};
use crate::serve::http::Stream;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Option<Result<Flow, String>> {
    if method != "POST" || !(path == "/command" || path == "/command/copy") {
        return None;
    }
    if !authorised(headers, token) {
        return Some(
            respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve),
        );
    }
    Some(match path {
        "/command/copy" => copy(stream, session),
        _ => transform(stream, body),
    })
}

fn transform(stream: &mut dyn Stream, body: &[u8]) -> Result<Flow, String> {
    let request: serde_json::Value = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(error) => {
            return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                .map(|()| Flow::Serve);
        }
    };
    let text = text_field(&request, "text");
    let command = text_field(&request, "command").to_ascii_lowercase();
    if text.is_empty() {
        return respond_json(
            stream,
            "400 Bad Request",
            &error_json("there is no selected text to change"),
        )
        .map(|()| Flow::Serve);
    }
    match crate::style::command(&text, &command) {
        Ok(changed) => respond_json(
            stream,
            "200 OK",
            &serde_json::json!({ "text": changed }).to_string(),
        )
        .map(|()| Flow::Serve),
        Err(error) => {
            respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
        }
    }
}

fn copy(stream: &mut dyn Stream, session: &Session) -> Result<Flow, String> {
    let window = match session.captured_window() {
        Some(window) => window,
        None => {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("Parlo has no window to read from"),
            )
            .map(|()| Flow::Serve);
        }
    };
    match crate::insert::copy_selection(window) {
        Ok(selected) => respond_json(
            stream,
            "200 OK",
            &serde_json::json!({ "text": selected }).to_string(),
        )
        .map(|()| Flow::Serve),
        Err(error) => {
            respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
        }
    }
}
