use super::super::auth::authorised;
use super::super::http::{error_json, header, respond_json};
use super::super::{json_string, meeting_json, rooms, Flow, Session};
use crate::serve::http::Stream;
use parlo_core::wav::read_pcm16;

const PATHS: [&str; 9] = [
    "/meeting/window",
    "/meeting/summary",
    "/meetings/clear",
    "/meeting/import",
    "/meeting/name",
    "/meeting/delete",
    "/meeting/start",
    "/meeting/segment",
    "/meeting/ask",
];

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Option<Result<Flow, String>> {
    if method != "POST" || !(PATHS.contains(&path) || path == "/memory/ask") {
        return None;
    }
    Some(handle(method, path, stream, headers, token, body, session))
}

fn handle(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Result<Flow, String> {
        if method == "POST" && path == "/meeting/window" {
            if !authorised(headers, token) {
                return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                    .map(|()| Flow::Serve);
            }
            let id = header(headers, "x-parlo-meeting")
                .unwrap_or_default()
                .trim()
                .to_string();
            let at_ms = header(headers, "x-parlo-at")
                .unwrap_or_default()
                .trim()
                .parse::<u64>()
                .unwrap_or(0);
            let mut meeting = match crate::meeting::load(&id) {
                Ok(meeting) => meeting,
                Err(error) => {
                    return respond_json(stream, "404 Not Found", &error_json(&error))
                        .map(|()| Flow::Serve);
                }
            };
            let wav = match read_pcm16(body) {
                Ok(wav) => wav,
                Err(error) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &error_json(&format!("bad wav: {error}")),
                    )
                    .map(|()| Flow::Serve);
                }
            };
            if wav.format.channels() != 1
                || wav.format.sample_rate() != 16_000
                || wav.samples.is_empty()
            {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json("input must be 16 kHz mono pcm16"),
                )
                .map(|()| Flow::Serve);
            }
            let mut pcm16 = Vec::with_capacity(wav.samples.len() * 2);
            for sample in &wav.samples {
                pcm16.extend_from_slice(&sample.to_le_bytes());
            }
            let text = match session.dictate(&pcm16) {
                Ok(text) => text.trim().to_string(),
                Err(error) => {
                    return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                        .map(|()| Flow::Serve);
                }
            };
            if text.is_empty() {
                return respond_json(stream, "200 OK", "{\"text\":\"\",\"speaker\":\"\"}")
                    .map(|()| Flow::Serve);
            }
            let hint = meeting.speaker_hint;
            let label = match session.embed(&pcm16) {
                Ok(vector) => rooms()
                    .lock()
                    .ok()
                    .and_then(|mut open| {
                        open.entry(id.clone())
                            .or_insert_with(|| match hint {
                                Some(count) => crate::speakers::Room::with_limit(count),
                                None => crate::speakers::Room::new(),
                            })
                            .label(&vector)
                            .ok()
                    })
                    .unwrap_or_else(|| "Mic".to_string()),
                Err(_) => "Mic".to_string(),
            };
            if let Err(error) = crate::meeting::append(&mut meeting, &label, &text, at_ms) {
                return respond_json(stream, "400 Bad Request", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
            for named in crate::speakers::infer(&meeting.segments) {
                let _ = crate::meeting::name_speaker(
                    &mut meeting,
                    &named.speaker,
                    Some(&named.name),
                    &named.evidence,
                );
            }
            return match crate::meeting::save(&meeting) {
                Ok(()) => respond_json(
                    stream,
                    "200 OK",
                    &format!(
                        "{{\"text\":{},\"speaker\":{},\"meeting\":{}}}",
                        json_string(&text),
                        json_string(&label),
                        meeting_json(meeting)
                    ),
                )
                .map(|()| Flow::Serve),
                Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve),
            };
        }

        if method == "POST" && path == "/meeting/summary" {
            if !authorised(headers, token) {
                return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                    .map(|()| Flow::Serve);
            }
            let request: serde_json::Value =
                serde_json::from_slice(body).unwrap_or(serde_json::Value::Null);
            let id = request
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            let meeting = if id.is_empty() {
                crate::meeting::list().into_iter().next()
            } else {
                crate::meeting::load(&id).ok()
            };
            let Some(meeting) = meeting else {
                return respond_json(
                    stream,
                    "404 Not Found",
                    "{\"error\":\"there is no meeting to summarise\"}",
                )
                .map(|()| Flow::Serve);
            };
            let settings = crate::config::load().intelligence;
            let model = if settings.local_model.trim().is_empty() {
                "local-model".to_string()
            } else {
                settings.local_model.clone()
            };
            let body = crate::summary::answer(&meeting, &settings.local_url, &model);
            return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
        }

        if method == "POST" && path == "/meetings/clear" {
            if !authorised(headers, token) {
                return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                    .map(|()| Flow::Serve);
            }
            return match crate::meeting::clear() {
                Ok(()) => respond_json(stream, "200 OK", "{\"cleared\":true}").map(|()| Flow::Serve),
                Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve),
            };
        }

        if method == "POST"
            && (path == "/meeting/import" || path == "/meeting/name" || path == "/meeting/delete")
        {
            if !authorised(headers, token) {
                return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                    .map(|()| Flow::Serve);
            }
            let request: serde_json::Value = match serde_json::from_slice(body) {
                Ok(request) => request,
                Err(error) => {
                    return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                        .map(|()| Flow::Serve);
                }
            };
            let field = |name: &str| {
                request
                    .get(name)
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string()
            };

            if path == "/meeting/import" {
                let meeting = match crate::meeting::import(&field("title"), &field("text")) {
                    Ok(meeting) => meeting,
                    Err(error) => {
                        return respond_json(stream, "400 Bad Request", &error_json(&error))
                            .map(|()| Flow::Serve);
                    }
                };
                if let Err(error) = crate::meeting::save(&meeting) {
                    return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                        .map(|()| Flow::Serve);
                }
                return respond_json(stream, "200 OK", &meeting_json(meeting).to_string())
                    .map(|()| Flow::Serve);
            }

            if path == "/meeting/delete" {
                return match crate::meeting::delete(&field("id")) {
                    Ok(()) => {
                        respond_json(stream, "200 OK", "{\"deleted\":true}").map(|()| Flow::Serve)
                    }
                    Err(error) => respond_json(stream, "400 Bad Request", &error_json(&error))
                        .map(|()| Flow::Serve),
                };
            }

            let mut meeting = match crate::meeting::load(&field("id")) {
                Ok(meeting) => meeting,
                Err(error) => {
                    return respond_json(stream, "404 Not Found", &error_json(&error))
                        .map(|()| Flow::Serve);
                }
            };
            let name = field("name");
            let label = if name.is_empty() {
                None
            } else {
                Some(name.as_str())
            };
            let outcome = crate::meeting::name_speaker(
                &mut meeting,
                &field("speaker"),
                label,
                &field("evidence"),
            );
            return match outcome {
                Ok(_) => match crate::meeting::save(&meeting) {
                    Ok(()) => respond_json(stream, "200 OK", &meeting_json(meeting).to_string())
                        .map(|()| Flow::Serve),
                    Err(error) => {
                        respond_json(stream, "500 Internal Server Error", &error_json(&error))
                            .map(|()| Flow::Serve)
                    }
                },
                Err(error) => {
                    respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
                }
            };
        }

        if method == "POST" && (path == "/meeting/start" || path == "/meeting/segment") {
            if !authorised(headers, token) {
                return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                    .map(|()| Flow::Serve);
            }
            let request: serde_json::Value = match serde_json::from_slice(body) {
                Ok(request) => request,
                Err(error) => {
                    return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                        .map(|()| Flow::Serve);
                }
            };
            let field = |name: &str| {
                request
                    .get(name)
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string()
            };

            if path == "/meeting/start" {
                let speakers = request
                    .get("speakers")
                    .and_then(|value| value.as_u64())
                    .map(|count| count as usize);
                let meeting = match crate::meeting::start_with(&field("title"), speakers) {
                    Ok(meeting) => meeting,
                    Err(error) => {
                        return respond_json(stream, "400 Bad Request", &error_json(&error))
                            .map(|()| Flow::Serve)
                    }
                };
                if let Ok(mut open) = rooms().lock() {
                    open.remove(&meeting.id);
                }
                if let Err(error) = crate::meeting::save(&meeting) {
                    return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                        .map(|()| Flow::Serve);
                }
                return respond_json(stream, "200 OK", &meeting_json(meeting).to_string())
                    .map(|()| Flow::Serve);
            }

            let mut meeting = match crate::meeting::load(&field("id")) {
                Ok(meeting) => meeting,
                Err(error) => {
                    return respond_json(stream, "404 Not Found", &error_json(&error))
                        .map(|()| Flow::Serve);
                }
            };
            let at_ms = request
                .get("at_ms")
                .and_then(|value| value.as_u64())
                .unwrap_or_default();
            if let Err(error) =
                crate::meeting::append(&mut meeting, &field("speaker"), &field("text"), at_ms)
            {
                return respond_json(stream, "400 Bad Request", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
            return match crate::meeting::save(&meeting) {
                Ok(()) => respond_json(stream, "200 OK", &meeting_json(meeting).to_string())
                    .map(|()| Flow::Serve),
                Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve),
            };
        }

        if method == "POST" && (path == "/meeting/ask" || path == "/memory/ask") {
            if !authorised(headers, token) {
                return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                    .map(|()| Flow::Serve);
            }
            let request: serde_json::Value = match serde_json::from_slice(body) {
                Ok(request) => request,
                Err(error) => {
                    return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                        .map(|()| Flow::Serve);
                }
            };
            let question = request
                .get("question")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .trim();
            if question.is_empty() {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json("ask a question first"),
                )
                .map(|()| Flow::Serve);
            }
            let meetings = if path == "/memory/ask" {
                crate::meeting::list()
            } else {
                let id = request
                    .get("id")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                let chosen = if id.is_empty() {
                    crate::meeting::list().into_iter().next()
                } else {
                    crate::meeting::load(id).ok()
                };
                chosen.into_iter().collect()
            };
            let answer = crate::ask::ask(question, &meetings);
            let body =
                serde_json::to_string(&answer).unwrap_or_else(|error| error_json(&error.to_string()));
            return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
        }

    respond_json(stream, "404 Not Found", &error_json("unknown meeting route"))
        .map(|()| Flow::Serve)
}
