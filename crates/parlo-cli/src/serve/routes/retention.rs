use super::super::auth::authorised;
use super::super::http::respond_json;
use super::super::{json_string, parameter, stale_retention, Flow};
use crate::serve::http::Stream;

pub(crate) fn route(
    method: &str,
    path: &str,
    query: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    if path != "/retention" {
        return None;
    }
    if !authorised(headers, token) {
        return Some(
            respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve),
        );
    }
    match method {
        "GET" => {
            let days = parameter(query, "days").parse::<u64>().unwrap_or(30);
            Some(match stale_retention(days) {
                Ok((rows, stale)) => {
                    let body = serde_json::json!({
                        "days": days,
                        "count": stale.len(),
                        "meetings": rows,
                    })
                    .to_string();
                    respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
                }
                Err(error) => respond_json(stream, "400 Bad Request", &format!("{{\"error\":\"{}\"}}", json_string(&error)))
                    .map(|()| Flow::Serve),
            })
        }
        "POST" => Some(apply_retention(stream, body)),
        _ => None,
    }
}

fn apply_retention(stream: &mut dyn Stream, body: &[u8]) -> Result<Flow, String> {
    let request: serde_json::Value = serde_json::from_slice(body).unwrap_or_default();
    let days = request["days"].as_u64().unwrap_or(30);
    let apply = request["apply"].as_bool().unwrap_or(false);
    let (rows, stale) = match stale_retention(days) {
        Ok(result) => result,
        Err(error) => {
            return respond_json(stream, "400 Bad Request", &format!("{{\"error\":\"{}\"}}", json_string(&error)))
                .map(|()| Flow::Serve)
        }
    };
    let mut removed = 0;
    if apply {
        for id in &stale {
            if let Err(error) = crate::meeting::delete(id) {
                return respond_json(stream, "500 Internal Server Error", &format!("{{\"error\":\"{}\"}}", json_string(&error)))
                    .map(|()| Flow::Serve);
            }
            removed += 1;
        }
    }
    let body = serde_json::json!({
        "days": days,
        "count": stale.len(),
        "removed": removed,
        "applied": apply,
        "meetings": rows,
    })
    .to_string();
    respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
}
