use super::super::auth::authorised;
use super::super::http::respond_json;
use super::super::{meeting_json, respond_named, Flow};
use crate::serve::http::Stream;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
) -> Option<Result<Flow, String>> {
    if method != "GET" || (path != "/meetings" && !is_markdown(path)) {
        return None;
    }
    if !authorised(headers, token) {
        return Some(
            respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve),
        );
    }
    if path == "/meetings" {
        let meetings: Vec<serde_json::Value> = crate::meeting::list()
            .into_iter()
            .map(meeting_json)
            .collect();
        return Some(
            respond_json(stream, "200 OK", &serde_json::json!(meetings).to_string())
                .map(|()| Flow::Serve),
        );
    }
    Some(markdown(stream, path))
}

fn is_markdown(path: &str) -> bool {
    path.starts_with("/meetings/") && path.ends_with("/markdown")
}

fn markdown(stream: &mut dyn Stream, path: &str) -> Result<Flow, String> {
    let id = path
        .trim_start_matches("/meetings/")
        .trim_end_matches("/markdown")
        .trim_end_matches('/');
    let meeting = match crate::meeting::load(id) {
        Ok(meeting) => meeting,
        Err(error) => {
            let body = serde_json::json!({ "error": error }).to_string();
            return respond_json(stream, "404 Not Found", &body).map(|()| Flow::Serve);
        }
    };
    let name = crate::notes::file_name(&meeting);
    let body = crate::notes::markdown(&meeting);
    respond_named(stream, &name, body.as_bytes()).map(|()| Flow::Serve)
}
