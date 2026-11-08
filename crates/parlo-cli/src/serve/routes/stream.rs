use super::super::parameter;
use super::super::ws;
use super::super::{Flow, Session};
use crate::serve::http::Stream;

pub(crate) struct Listener<'a> {
    pub(crate) port: u16,
    pub(crate) session: &'a Session,
}

pub(crate) fn route(
    method: &str,
    path: &str,
    query: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    listener: Listener,
) -> Option<Result<Flow, String>> {
    if method == "GET" && path == "/v1/audio/transcriptions/stream" {
        return Some(ws::stream(
            stream,
            headers,
            token,
            &parameter(query, "t"),
            &parameter(query, "ticket"),
            listener.port,
            listener.session,
        ));
    }
    None
}
