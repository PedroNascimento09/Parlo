use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::{respond_file, Flow};
use crate::serve::http::Stream;
use crate::story;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    if path != "/stories" && !path.starts_with("/stories/") {
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
    if !authorised(headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }
    if method == "GET" && path == "/stories" {
        return match story::list() {
            Ok(stories) => {
                let body = serde_json::json!({ "stories": stories }).to_string();
                respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "500 Internal Server Error", &error),
        };
    }
    if method == "GET" && path == "/stories/voices" {
        let body = serde_json::json!({ "voices": story::voices() }).to_string();
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }
    if method == "POST" && path == "/stories" {
        let request = match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(error) => {
                return failure(
                    stream,
                    "400 Bad Request",
                    &format!("the story request is not valid JSON: {error}"),
                )
            }
        };
        return match story::create(request) {
            Ok(story) => {
                let body = serde_json::to_string(&story).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "201 Created", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "400 Bad Request", &error),
        };
    }
    let rest = &path["/stories/".len()..];
    if method == "POST" && rest.ends_with("/render") {
        let id = rest.trim_end_matches("/render");
        return match story::start(id) {
            Ok(story) => {
                let body = serde_json::to_string(&story).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "202 Accepted", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "409 Conflict", &error),
        };
    }
    if (method == "PUT" || method == "POST") && !rest.contains('/') {
        let request = match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(error) => {
                return failure(
                    stream,
                    "400 Bad Request",
                    &format!("the story request is not valid JSON: {error}"),
                )
            }
        };
        return match story::update(rest, request) {
            Ok(story) => {
                let body = serde_json::to_string(&story).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "400 Bad Request", &error),
        };
    }
    if method == "GET" {
        if let Some(id) = rest.strip_suffix("/audiobook") {
            return match story::audiobook(id) {
                Ok((path, name)) => respond_file(stream, &path, &name).map(|()| Flow::Serve),
                Err(error) => failure(stream, "409 Conflict", &error),
            };
        }
        if let Some((id, line)) = rest.split_once("/lines/") {
            return match story::line_audio(id, line) {
                Ok((path, name)) => respond_file(stream, &path, &name).map(|()| Flow::Serve),
                Err(error) => failure(stream, "404 Not Found", &error),
            };
        }
        if rest.is_empty() || rest.contains('/') {
            return missing(stream);
        }
        return match story::get(rest) {
            Ok(story) => {
                let body = serde_json::to_string(&story).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "404 Not Found", &error),
        };
    }
    missing(stream)
}

fn failure(stream: &mut dyn Stream, status: &str, error: &str) -> Result<Flow, String> {
    respond_json(stream, status, &error_json(error)).map(|()| Flow::Serve)
}

fn missing(stream: &mut dyn Stream) -> Result<Flow, String> {
    failure(stream, "404 Not Found", "story not found")
}
