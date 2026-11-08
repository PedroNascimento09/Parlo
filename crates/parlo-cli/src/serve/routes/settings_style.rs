use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::{json_string, Flow};
use crate::serve::http::Stream;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    if !matches!(
        (method, path),
        ("GET" | "POST", "/features") | ("POST", "/style")
    ) {
        return None;
    }
    Some(handle(method, path, stream, headers, token, body))
}

fn handle(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Result<Flow, String> {
    if method == "GET" && path == "/features" {
        let features = crate::config::load().features;
        let body = serde_json::to_string(&features).unwrap_or_else(|_| "{}".to_string());
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/features" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let requested: parlo_core::settings::Features = match serde_json::from_slice(body) {
            Ok(features) => features,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        let mut config = crate::config::load();
        config.features = requested;
        if let Err(error) = crate::config::save(&config) {
            return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        let body = serde_json::to_string(&config.features).unwrap_or_else(|_| "{}".to_string());
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/style" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        let style = request
            .get("style")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if !style.is_empty() && !crate::style::is_known(&style) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"style needs formal, casual or excited\"}",
            )
            .map(|()| Flow::Serve);
        }
        let mut config = crate::config::load();
        config.style = style;
        if let Err(error) = crate::config::save(&config) {
            return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        let body = format!("{{\"style\":\"{}\"}}", json_string(&config.style));
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    respond_json(
        stream,
        "404 Not Found",
        &error_json("unknown settings route"),
    )
    .map(|()| Flow::Serve)
}
