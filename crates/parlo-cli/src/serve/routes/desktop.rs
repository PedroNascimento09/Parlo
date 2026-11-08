use super::super::auth::authorised;
use super::super::http::{respond_json, Stream};
use super::super::{parameter, Flow};

pub(crate) fn route(
    method: &str,
    path: &str,
    query: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
) -> Option<Result<Flow, String>> {
    if !matches!((method, path), ("GET", "/call" | "/pronounce")) {
        return None;
    }
    Some(handle(method, path, query, stream, headers, token))
}

fn handle(
    method: &str,
    path: &str,
    query: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
) -> Result<Flow, String> {
    if method == "GET" && path == "/call" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let answer = match crate::calls::detected() {
            Some((app, title)) => {
                serde_json::json!({ "app": app, "title": title.chars().take(80).collect::<String>() })
            }
            None => serde_json::json!({ "app": serde_json::Value::Null }),
        };
        return respond_json(stream, "200 OK", &answer.to_string()).map(|()| Flow::Serve);
    }

    let text = parameter(query, "text");
    let text = if text.trim().is_empty() {
        parameter(query, "word")
    } else {
        text
    };
    let text = text.trim();
    if text.is_empty() {
        return respond_json(
            stream,
            "400 Bad Request",
            "{\"error\":\"send a word or a sentence to pronounce\"}",
        )
        .map(|()| Flow::Serve);
    }
    let entry = parlo_core::pronounce::lookup(text);
    let body = serde_json::json!({
        "text": text,
        "respell": parlo_core::pronounce::respell_in(text, None),
        "word": entry.map(|entry| entry.word.clone()),
        "ipa": entry.map(|entry| entry.ipa.clone()),
        "cmu": entry.map(|entry| entry.cmu.clone()),
    })
    .to_string();
    respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
}
