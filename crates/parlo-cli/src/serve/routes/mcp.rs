use super::super::auth::authorised;
use super::super::http::{error_json, respond, respond_json};
use super::super::Flow;
use crate::serve::http::Stream;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    if path != "/mcp" {
        return None;
    }
    if method != "POST" {
        return Some(
            respond_json(
                stream,
                "405 Method Not Allowed",
                &error_json("MCP over HTTP accepts POST"),
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
    let request: serde_json::Value = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(error) => {
            return Some(
                respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json(&format!("this is not JSON: {error}")),
                )
                .map(|()| Flow::Serve),
            )
        }
    };
    match crate::mcp::handle(&request) {
        Some(reply) => {
            Some(respond_json(stream, "200 OK", &reply.to_string()).map(|()| Flow::Serve))
        }
        None => Some(
            respond(stream, "202 Accepted", "text/plain; charset=utf-8", b"").map(|()| Flow::Serve),
        ),
    }
}
