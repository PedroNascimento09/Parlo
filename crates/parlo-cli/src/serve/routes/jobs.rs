#[cfg(test)]
use std::time::Duration;

use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::Flow;
use crate::serve::http::Stream;
use crate::{jobs, models};

#[derive(serde::Deserialize)]
struct JobRequest {
    kind: String,
}

pub(crate) fn create(
    stream: &mut dyn Stream,
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
    #[cfg(test)]
    if request.kind == "fixture" {
        let id = jobs::manager().submit("fixture", "Fixture", "", |handle| {
            for step in 0..5 {
                if handle.cancelled() {
                    return Ok(());
                }
                handle.progress("fixture", step as f32 / 5.0);
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(())
        });
        let accepted = serde_json::json!({ "id": id, "state": "queued" }).to_string();
        return respond_json(stream, "202 Accepted", &accepted).map(|()| Flow::Serve);
    }

    #[cfg(test)]
    if request.kind == "fixture-hold" {
        let id = jobs::manager().submit("fixture", "Fixture hold", "", |handle| {
            while !handle.cancelled() {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(())
        });
        let accepted = serde_json::json!({ "id": id, "state": "queued" }).to_string();
        return respond_json(stream, "202 Accepted", &accepted).map(|()| Flow::Serve);
    }

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
    let id = jobs::manager().submit("models", "Download the remaining models", "", |handle| {
        models::download_remaining(
            |stage, progress| handle.progress(stage, progress),
            || handle.cancelled(),
        )
    });
    let accepted = serde_json::json!({ "id": id, "state": "queued" }).to_string();
    respond_json(stream, "202 Accepted", &accepted).map(|()| Flow::Serve)
}

pub(crate) fn list(
    stream: &mut dyn Stream,
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
    stream: &mut dyn Stream,
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
    stream: &mut dyn Stream,
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
