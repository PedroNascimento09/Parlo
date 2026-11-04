use std::net::TcpStream;
use std::time::{Duration, Instant};

use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::Flow;
use crate::{jobs, models};

#[derive(serde::Deserialize)]
struct JobRequest {
    kind: String,
}

pub(crate) fn create(
    stream: &mut TcpStream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }
    let request: JobRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(_) => {
            return respond_json(stream, "400 Bad Request", &error_json("invalid job"))
                .map(|()| Flow::Serve)
        }
    };
    if request.kind != "models" {
        return respond_json(
            stream,
            "400 Bad Request",
            &error_json("the only job kind is models"),
        )
        .map(|()| Flow::Serve);
    }
    let targets = models::remaining_units();
    let current = models::status();
    let installed = targets
        .iter()
        .filter(|id| {
            current
                .iter()
                .any(|model| model.id == **id && model.installed)
        })
        .count();
    if targets.is_empty() || installed == targets.len() {
        return respond_json(
            stream,
            "409 Conflict",
            &error_json("every model is already installed"),
        )
        .map(|()| Flow::Serve);
    }
    let id = jobs::manager().submit("models", "Download the remaining models", "", {
        let targets = targets.clone();
        move |handle| {
            models::install(&targets)?;
            let deadline = Instant::now() + Duration::from_secs(3_600);
            loop {
                if handle.cancelled() {
                    let _ = models::cancel();
                    return Ok(());
                }
                let current = models::status();
                let installed = targets
                    .iter()
                    .filter(|id| {
                        current
                            .iter()
                            .any(|model| model.id == **id && model.installed)
                    })
                    .count();
                handle.progress("download", installed as f32 / targets.len() as f32);
                if installed == targets.len() {
                    return Ok(());
                }
                if Instant::now() > deadline {
                    return Err("the download did not finish within an hour".to_string());
                }
                if !models::downloading(&current) {
                    return Err("the download stopped before it finished".to_string());
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    });
    let accepted = serde_json::json!({ "id": id, "state": "queued" }).to_string();
    respond_json(stream, "202 Accepted", &accepted).map(|()| Flow::Serve)
}

pub(crate) fn list(
    stream: &mut TcpStream,
    headers: &[(String, String)],
    token: &str,
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }
    let rows: Vec<serde_json::Value> = jobs::manager()
        .list()
        .iter()
        .map(|job| job.value())
        .collect();
    let body = serde_json::json!({ "object": "list", "data": rows }).to_string();
    respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
}

pub(crate) fn read(
    stream: &mut TcpStream,
    headers: &[(String, String)],
    token: &str,
    path: &str,
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }
    let rest = &path["/v1/jobs/".len()..];
    if let Some(id) = rest.strip_suffix("/events") {
        if jobs::manager().get(id).is_none() {
            return respond_json(stream, "404 Not Found", &error_json("no such job"))
                .map(|()| Flow::Serve);
        }
        return jobs::manager().stream(stream, id).map(|()| Flow::Serve);
    }
    let id = rest.trim_end_matches('/');
    let Some(job) = jobs::manager().get(id) else {
        return respond_json(stream, "404 Not Found", &error_json("no such job"))
            .map(|()| Flow::Serve);
    };
    respond_json(stream, "200 OK", &job.value().to_string()).map(|()| Flow::Serve)
}

pub(crate) fn cancel(
    stream: &mut TcpStream,
    headers: &[(String, String)],
    token: &str,
    path: &str,
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }
    let id = path
        .trim_start_matches("/v1/jobs/")
        .trim_end_matches("/cancel");
    match jobs::manager().cancel(id) {
        Ok(()) => respond_json(stream, "200 OK", "{\"cancelled\":true}").map(|()| Flow::Serve),
        Err(error) => {
            respond_json(stream, "409 Conflict", &error_json(&error)).map(|()| Flow::Serve)
        }
    }
}
