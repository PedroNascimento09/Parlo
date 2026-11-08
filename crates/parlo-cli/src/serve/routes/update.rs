use super::super::auth::authorised;
use super::super::http::{error_json, respond_json, Stream};
use super::super::{json_string, Flow, VERSION};
use crate::update;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
) -> Option<Result<Flow, String>> {
    if !matches!(
        (method, path),
        ("GET", "/update/progress") | ("POST", "/update/restart") | ("GET" | "POST", "/update")
    ) {
        return None;
    }
    Some(handle(method, path, stream, headers, token))
}

fn handle(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
) -> Result<Flow, String> {
    if method == "GET" && path == "/update/progress" {
        return respond_json(stream, "200 OK", &update::progress_json()).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/update/restart" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        if let Err(error) = update::restart() {
            return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        respond_json(stream, "200 OK", "{\"restarting\":true}").map(|()| Flow::Serve)?;
        return Ok(Flow::Stop);
    }

    if method == "GET" && path == "/update" {
        let latest = match update::latest() {
            Ok(latest) => latest,
            Err(error) => {
                return respond_json(
                    stream,
                    "502 Bad Gateway",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
        let available = update::is_newer(VERSION, &latest.version);
        let build = latest.archive_url.is_some();
        let body = format!(
            "{{\"current\":\"{VERSION}\",\"latest\":\"{}\",\"available\":{available},\"build\":{build},\"notes\":\"{}\"}}",
            json_string(&latest.version),
            json_string(&latest.notes_url)
        );
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/update" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let latest = match update::latest() {
            Ok(latest) => latest,
            Err(error) => {
                return respond_json(
                    stream,
                    "502 Bad Gateway",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
        if !update::is_newer(VERSION, &latest.version) {
            return respond_json(stream, "409 Conflict", "{\"error\":\"already up to date\"}")
                .map(|()| Flow::Serve);
        }
        let binary = match update::install(&latest) {
            Ok(binary) => binary,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
        if let Err(error) = update::schedule_replacement(&binary) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        let body = format!(
            "{{\"installed\":true,\"version\":\"{}\"}}",
            json_string(&latest.version)
        );
        respond_json(stream, "200 OK", &body)?;
        return Ok(Flow::Stop);
    }

    respond_json(stream, "404 Not Found", &error_json("unknown update route")).map(|()| Flow::Serve)
}
