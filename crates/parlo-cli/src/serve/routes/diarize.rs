use std::net::TcpStream;

use super::super::auth::authorised;
use super::super::http::{error_json, header, respond_json};
use super::super::{audio, Flow, Session};

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut TcpStream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Option<Result<Flow, String>> {
    if path != "/meeting/diarize" {
        return None;
    }
    if method != "POST" {
        return Some(
            respond_json(
                stream,
                "405 Method Not Allowed",
                &error_json("speaker separation accepts POST"),
            )
            .map(|()| Flow::Serve),
        );
    }
    if !authorised(headers, token) {
        return Some(
            respond_json(stream, "401 Unauthorized", &error_json("wrong token"))
                .map(|()| Flow::Serve),
        );
    }
    let hint = match header(headers, "x-parlo-speakers") {
        Some(value) if !value.trim().is_empty() => match value.trim().parse::<usize>() {
            Ok(count) if (1..=20).contains(&count) => Some(count),
            _ => {
                return Some(
                    respond_json(
                        stream,
                        "400 Bad Request",
                        &error_json("x-parlo-speakers must be between 1 and 20"),
                    )
                    .map(|()| Flow::Serve),
                )
            }
        },
        _ => None,
    };
    let (pcm16, seconds) = match audio::decode(body) {
        Ok(decoded) => decoded,
        Err(error) => {
            return Some(
                respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve),
            )
        }
    };
    if seconds < 1.0 {
        return Some(
            respond_json(
                stream,
                "400 Bad Request",
                &error_json("speaker separation needs at least a second of audio"),
            )
            .map(|()| Flow::Serve),
        );
    }
    match session.diarize(&pcm16, hint) {
        Ok(result) => match serde_json::to_string(&result) {
            Ok(answer) => Some(respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve)),
            Err(error) => Some(
                respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&error.to_string()),
                )
                .map(|()| Flow::Serve),
            ),
        },
        Err(error) => Some(
            respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        ),
    }
}
