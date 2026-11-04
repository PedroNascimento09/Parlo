use std::net::TcpStream;

use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::Flow;

#[derive(serde::Deserialize)]
struct CopyRequest {
    text: String,
}

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut TcpStream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    if path != "/clipboard" {
        return None;
    }
    if method != "POST" {
        return Some(
            respond_json(
                stream,
                "405 Method Not Allowed",
                &error_json("the clipboard accepts POST"),
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
    let request: CopyRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(error) => {
            return Some(
                respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json(&format!("this is not a clipboard request: {error}")),
                )
                .map(|()| Flow::Serve),
            )
        }
    };
    let text = request.text.trim();
    if text.is_empty() {
        return Some(
            respond_json(
                stream,
                "400 Bad Request",
                &error_json("the clipboard cannot take an empty sentence"),
            )
            .map(|()| Flow::Serve),
        );
    }
    if text.chars().count() > 20_000 {
        return Some(
            respond_json(
                stream,
                "400 Bad Request",
                &error_json("the clipboard text is too long"),
            )
            .map(|()| Flow::Serve),
        );
    }
    match crate::clipboard::copy(text) {
        Ok(generation) => {
            let answer = serde_json::json!({
                "copied": true,
                "generation": generation,
                "lease_seconds": crate::clipboard::LEASE_SECONDS,
            })
            .to_string();
            Some(respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve))
        }
        Err(error) => Some(
            respond_json(stream, "503 Service Unavailable", &error_json(&error))
                .map(|()| Flow::Serve),
        ),
    }
}
