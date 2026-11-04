use std::net::TcpStream;

use super::super::http::respond_json;
use super::super::{Flow, VERSION};

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut TcpStream,
) -> Option<Result<Flow, String>> {
    if method != "GET" || path != "/.well-known/parlo" {
        return None;
    }
    let body = serde_json::json!({
        "name": "Parlo",
        "version": VERSION,
        "api": "v1",
        "local": true,
        "authentication": "x-parlo-token or Authorization: Bearer",
        "capabilities": ["dictation", "translation", "meetings", "voices", "dubbing"],
    })
    .to_string();
    Some(respond_json(stream, "200 OK", &body).map(|()| Flow::Serve))
}
