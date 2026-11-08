use super::super::auth::authorised;
use super::super::http::{error_json, respond, respond_json, Stream};
use super::super::{
    Flow, CAPTIONS, INDEX, ISLAND, LANGUAGES, LANGUAGES_TABLE, LIVE, ROUTES_SCRIPT, SCRIPT,
    STYLESHEET, VERSION,
};
use crate::captions;
use crate::config;

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
        (
            "GET",
            "/" | "/app.css"
                | "/island"
                | "/island.html"
                | "/captions"
                | "/captions.html"
                | "/live"
                | "/live.html"
                | "/app.js"
                | "/routes.js"
                | "/languages.js"
                | "/languages.json"
        ) | ("POST", "/captions/save")
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
    if method == "GET" && path == "/" {
        let page = INDEX.replace("__PARLO_VERSION__", VERSION);
        return respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            page.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/app.css" {
        return respond(
            stream,
            "200 OK",
            "text/css; charset=utf-8",
            STYLESHEET.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && (path == "/island" || path == "/island.html") {
        let page = ISLAND.replace("__PARLO_VERSION__", VERSION);
        return respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            page.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && (path == "/captions" || path == "/captions.html") {
        if !config::load().features.captions {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"captions are switched off in Settings\"}",
            )
            .map(|()| Flow::Serve);
        }
        let page = CAPTIONS.replace("__PARLO_VERSION__", VERSION);
        return respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            page.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/captions/save" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", &error_json("wrong token"))
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = serde_json::from_slice(body).unwrap_or_default();
        let lines: Vec<(u64, String)> = request["lines"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| {
                        let text = row["text"].as_str()?.trim();
                        if text.is_empty() {
                            return None;
                        }
                        Some((row["at_ms"].as_u64().unwrap_or(0), text.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        return match captions::save(&lines) {
            Ok(path) => respond_json(
                stream,
                "200 OK",
                &serde_json::json!({
                    "path": path.display().to_string(),
                    "message": format!("Saved to {}.", path.display()),
                })
                .to_string(),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "GET" && (path == "/live" || path == "/live.html") {
        if !config::load().features.translation {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"live mode is switched off in Settings\"}",
            )
            .map(|()| Flow::Serve);
        }
        let page = LIVE.replace("__PARLO_VERSION__", VERSION);
        return respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            page.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/app.js" {
        return respond(
            stream,
            "200 OK",
            "text/javascript; charset=utf-8",
            SCRIPT.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/routes.js" {
        return respond(
            stream,
            "200 OK",
            "text/javascript; charset=utf-8",
            ROUTES_SCRIPT.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/languages.js" {
        return respond(
            stream,
            "200 OK",
            "text/javascript; charset=utf-8",
            LANGUAGES.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/languages.json" {
        return respond(
            stream,
            "200 OK",
            "application/json; charset=utf-8",
            LANGUAGES_TABLE.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    respond_json(stream, "404 Not Found", &error_json("unknown page route")).map(|()| Flow::Serve)
}
