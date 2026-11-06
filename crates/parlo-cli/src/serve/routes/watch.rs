use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::Flow;
use crate::serve::http::Stream;
use crate::watch;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    if path != "/watch" && !path.starts_with("/watch/") {
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
    if method == "GET" && path == "/watch" {
        return match watch::list() {
            Ok(folders) => {
                let body = serde_json::json!({ "folders": folders }).to_string();
                respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "500 Internal Server Error", &error),
        };
    }
    if method == "POST" && path == "/watch" {
        let request = match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(error) => {
                return failure(
                    stream,
                    "400 Bad Request",
                    &format!("the watched folder request is not valid JSON: {error}"),
                )
            }
        };
        return match watch::add(request) {
            Ok(folder) => {
                let body = serde_json::to_string(&folder).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "201 Created", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "400 Bad Request", &error),
        };
    }
    if method == "POST" && path == "/watch/scan" {
        return match watch::scan_all() {
            Ok(results) => {
                let body = serde_json::json!({ "scanned": results }).to_string();
                respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "500 Internal Server Error", &error),
        };
    }
    let rest = &path["/watch/".len()..];
    if method == "POST" {
        if let Some(id) = rest.strip_suffix("/scan") {
            return match watch::scan(id) {
                Ok(result) => {
                    let body = serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string());
                    respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
                }
                Err(error) => failure(stream, "404 Not Found", &error),
            };
        }
        if let Some(id) = rest.strip_suffix("/remove") {
            return match watch::remove(id) {
                Ok(()) => {
                    respond_json(stream, "200 OK", "{\"removed\":true}").map(|()| Flow::Serve)
                }
                Err(error) => failure(stream, "404 Not Found", &error),
            };
        }
    }
    respond_json(
        stream,
        "404 Not Found",
        &error_json("watched folder not found"),
    )
    .map(|()| Flow::Serve)
}

fn failure(stream: &mut dyn Stream, status: &str, error: &str) -> Result<Flow, String> {
    respond_json(stream, status, &error_json(error)).map(|()| Flow::Serve)
}
