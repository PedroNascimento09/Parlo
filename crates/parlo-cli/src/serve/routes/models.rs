use std::net::TcpStream;

use super::super::http::respond_json;
use super::super::{Flow, Session};
use crate::models;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut TcpStream,
    session: &Session,
) -> Option<Result<Flow, String>> {
    if method == "GET" && path == "/v1/models" {
        let data: Vec<serde_json::Value> = models::status()
            .iter()
            .map(|model| {
                let spec = crate::registry::model(model.id);
                serde_json::json!({
                    "id": model.id,
                    "object": "model",
                    "owned_by": "parlo",
                    "kind": model.kind,
                    "feature": model.feature,
                    "installed": model.installed,
                    "licence": spec.map(|spec| spec.licence).unwrap_or_default(),
                    "languages": spec.map(|spec| spec.languages).unwrap_or(&[]),
                    "devices": spec.map(|spec| spec.devices).unwrap_or(&[]),
                    "bytes": spec.map(|spec| spec.bytes).unwrap_or(0),
                    "capabilities": spec.map(|spec| spec.capabilities).unwrap_or(&[]),
                    "default": spec.map(|spec| spec.default).unwrap_or(false),
                })
            })
            .collect();
        let body = serde_json::json!({ "object": "list", "data": data }).to_string();
        return Some(respond_json(stream, "200 OK", &body).map(|()| Flow::Serve));
    }

    if method == "GET" && path == "/models" {
        let lifecycle = session.lifecycle_snapshot();
        let pair = {
            let chosen = session.chosen();
            format!("{}-{}", chosen.from, chosen.to)
        };
        let body = models::as_json(&models::status(), &lifecycle, &pair, session.is_working());
        return Some(respond_json(stream, "200 OK", &body).map(|()| Flow::Serve));
    }

    None
}
