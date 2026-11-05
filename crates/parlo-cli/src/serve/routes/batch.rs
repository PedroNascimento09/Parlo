use std::net::TcpStream;

use super::super::auth::authorised;
use super::super::http::{error_json, header, percent_decode, respond_json};
use super::super::{respond_file, respond_named, Flow};
use crate::batch;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut TcpStream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    if path != "/batch" && !path.starts_with("/batch/") {
        return None;
    }
    Some(handle(method, path, stream, headers, token, body))
}

fn handle(
    method: &str,
    path: &str,
    stream: &mut TcpStream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }
    if method == "GET" && path == "/batch" {
        return match batch::list() {
            Ok(batches) => {
                let body = serde_json::json!({ "batches": batches }).to_string();
                respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "500 Internal Server Error", &error),
        };
    }
    if method == "POST" && path == "/batch" {
        let request = match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(error) => {
                return failure(
                    stream,
                    "400 Bad Request",
                    &format!("the batch request is not valid JSON: {error}"),
                )
            }
        };
        return match batch::create(request) {
            Ok(batch) => {
                let body = serde_json::to_string(&batch).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "201 Created", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "400 Bad Request", &error),
        };
    }
    let rest = &path["/batch/".len()..];
    if method == "POST" {
        if let Some(id) = rest.strip_suffix("/files") {
            if id.is_empty() || id.contains('/') {
                return missing(stream);
            }
            let name = header(headers, "x-parlo-filename")
                .map(|value| percent_decode(&value))
                .unwrap_or_else(|| "upload.mp4".to_string());
            return match batch::add_file(id, &name, body) {
                Ok(batch) => {
                    let body = serde_json::to_string(&batch).unwrap_or_else(|_| "{}".to_string());
                    respond_json(stream, "201 Created", &body).map(|()| Flow::Serve)
                }
                Err(error) => failure(stream, "400 Bad Request", &error),
            };
        }
        if let Some(id) = rest.strip_suffix("/start") {
            return match batch::start(id) {
                Ok(batch) => {
                    let body = serde_json::to_string(&batch).unwrap_or_else(|_| "{}".to_string());
                    respond_json(stream, "202 Accepted", &body).map(|()| Flow::Serve)
                }
                Err(error) => failure(stream, "409 Conflict", &error),
            };
        }
        if let Some((id, item)) = rest.split_once("/retry/") {
            return match batch::retry(id, item) {
                Ok(batch) => {
                    let body = serde_json::to_string(&batch).unwrap_or_else(|_| "{}".to_string());
                    respond_json(stream, "202 Accepted", &body).map(|()| Flow::Serve)
                }
                Err(error) => failure(stream, "409 Conflict", &error),
            };
        }
    }
    if method == "GET" {
        if let Some(id) = rest.strip_suffix("/report") {
            return match batch::report(id) {
                Ok(report) => respond_named(
                    stream,
                    &format!("batch-{id}-report.json"),
                    report.as_bytes(),
                )
                .map(|()| Flow::Serve),
                Err(error) => failure(stream, "404 Not Found", &error),
            };
        }
        if let Some((id, item)) = rest.split_once("/items/") {
            return match batch::download(id, item) {
                Ok((path, name)) => respond_file(stream, &path, &name).map(|()| Flow::Serve),
                Err(error) => failure(stream, "404 Not Found", &error),
            };
        }
        if rest.is_empty() || rest.contains('/') {
            return missing(stream);
        }
        return match batch::get(rest) {
            Ok(batch) => {
                let body = serde_json::to_string(&batch).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
            }
            Err(error) => failure(stream, "404 Not Found", &error),
        };
    }
    missing(stream)
}

fn failure(stream: &mut TcpStream, status: &str, error: &str) -> Result<Flow, String> {
    respond_json(stream, status, &error_json(error)).map(|()| Flow::Serve)
}

fn missing(stream: &mut TcpStream) -> Result<Flow, String> {
    failure(stream, "404 Not Found", "batch not found")
}
