use crate::registry::PIPELINES;

use super::super::http::respond_json;
use super::super::{Flow, MAX_AUDIO_BODY, MAX_BODY, MAX_DUB_BODY, VERSION};
use crate::serve::http::Stream;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
) -> Option<Result<Flow, String>> {
    if method != "GET" || path != "/.well-known/parlo" {
        return None;
    }
    let engines: Vec<serde_json::Value> = PIPELINES
        .iter()
        .map(|engine| {
            serde_json::json!({
                "id": engine.id,
                "name": engine.name,
                "licence": engine.licence,
                "hardware": engine.hardware,
                "requires_gpu": engine.requires_gpu,
                "available": engine.available,
                "default": engine.default,
                "fidelity": engine.fidelity,
            })
        })
        .collect();
    let body = serde_json::json!({
        "name": "Parlo",
        "version": VERSION,
        "api": "v1",
        "protocol": super::super::ws::PROTOCOL,
        "local": true,
        "authentication": "x-parlo-token or Authorization: Bearer",
        "capabilities": ["dictation", "translation", "meetings", "voices", "dubbing", "streaming"],
        "engines": engines,
        "limits": {
            "max_body_bytes": MAX_BODY,
            "max_audio_bytes": MAX_AUDIO_BODY,
            "max_dub_bytes": MAX_DUB_BODY,
            "max_websocket_message_bytes": super::super::ws::MAX_MESSAGE,
        },
    })
    .to_string();
    Some(respond_json(stream, "200 OK", &body).map(|()| Flow::Serve))
}
