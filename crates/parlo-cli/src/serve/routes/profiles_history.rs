use super::super::auth::authorised;
use super::super::http::{error_json, respond_json};
use super::super::session::Session;
use super::super::{json_string, parameter, respond_named, restyle_fields, text_field, Flow};
use crate::serve::http::Stream;

pub(crate) struct Request<'a> {
    pub(crate) method: &'a str,
    pub(crate) path: &'a str,
    pub(crate) query: &'a str,
    pub(crate) headers: &'a [(String, String)],
    pub(crate) token: &'a str,
    pub(crate) body: &'a [u8],
    pub(crate) session: &'a Session,
}

pub(crate) fn route(
    request: &Request<'_>,
    stream: &mut dyn Stream,
) -> Option<Result<Flow, String>> {
    let (method, path) = (request.method, request.path);
    let handled = (path == "/profiles" && (method == "GET" || method == "POST"))
        || (method == "POST" && path == "/history/restyle")
        || (method == "POST" && path == "/history/reinsert")
        || (method == "GET"
            && matches!(
                path,
                "/hardware" | "/catalog" | "/history" | "/history/export" | "/locale"
            ))
        || (method == "POST" && path == "/history/clear")
        || (method == "POST"
            && matches!(
                path,
                "/history/favourite" | "/history/edit" | "/history/delete"
            ));
    if !handled {
        return None;
    }
    Some(handle(request, stream))
}

fn handle(request: &Request<'_>, stream: &mut dyn Stream) -> Result<Flow, String> {
    let (method, path, query, headers, token, body, session) = (
        request.method,
        request.path,
        request.query,
        request.headers,
        request.token,
        request.body,
        request.session,
    );
    if path == "/profiles" && (method == "GET" || method == "POST") {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let mut config = crate::config::load();
        if method == "POST" {
            let request: serde_json::Value = match serde_json::from_slice(body) {
                Ok(request) => request,
                Err(error) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &error_json(&error.to_string()),
                    )
                    .map(|()| Flow::Serve);
                }
            };
            let program = text_field(&request, "program").to_ascii_lowercase();
            let style = text_field(&request, "style").to_ascii_lowercase();
            let write = text_field(&request, "write").to_ascii_lowercase();
            if program.is_empty() {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"a profile names the program it applies to\"}",
                )
                .map(|()| Flow::Serve);
            }
            if !style.is_empty() && !crate::style::is_known(&style) {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"style needs formal, casual or excited\"}",
                )
                .map(|()| Flow::Serve);
            }
            if !write.is_empty() && !crate::config::check_language(&write) {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"write needs a two letter language code\"}",
                )
                .map(|()| Flow::Serve);
            }
            if style.is_empty() && write.is_empty() {
                config.profiles.remove(&program);
            } else {
                config
                    .profiles
                    .insert(program, parlo_core::settings::Profile { style, write });
            }
            if let Err(error) = crate::config::save(&config) {
                return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        }
        let body = serde_json::json!(config.profiles).to_string();
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/history/restyle" {
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
        let (id, style) = match restyle_fields(&request) {
            Ok(fields) => fields,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        return match crate::history::restyle(&id, &style) {
            Ok(entry) => respond_json(stream, "200 OK", &crate::history::entry_json(&entry))
                .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "404 Not Found", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/history/reinsert" {
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
        let id = text_field(&request, "id");
        if id.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"a dictation is named by its id\"}",
            )
            .map(|()| Flow::Serve);
        }
        let Some(entry) = crate::history::get(&id) else {
            return respond_json(
                stream,
                "404 Not Found",
                "{\"error\":\"no dictation carries that id\"}",
            )
            .map(|()| Flow::Serve);
        };
        return match session.reinsert(&entry.text) {
            Ok(()) => respond_json(stream, "200 OK", "{\"inserted\":true}").map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "409 Conflict", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "GET" && path == "/hardware" {
        return respond_json(stream, "200 OK", &crate::hardware::detect().as_json())
            .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/catalog" {
        return respond_json(stream, "200 OK", &crate::catalog::as_json()).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/history" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let asked = parameter(query, "q");
        let body = if asked.is_empty() {
            crate::history::as_json(20)
        } else {
            crate::history::search_json(&asked, 50)
        };
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/history/export" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let days = parameter(query, "days").parse::<u32>().unwrap_or(0);
        return match crate::history::export(days) {
            Ok(body) => {
                respond_named(stream, "parlo-dictations.md", body.as_bytes()).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/history/clear" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match crate::history::clear() {
            Ok(()) => respond_json(stream, "200 OK", "{\"cleared\":true}").map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST"
        && (path == "/history/favourite" || path == "/history/edit" || path == "/history/delete")
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
        let id = text_field(&request, "id");
        if id.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"a dictation is named by its id\"}",
            )
            .map(|()| Flow::Serve);
        }
        let outcome: Result<String, String> = if path == "/history/favourite" {
            let value = request
                .get("value")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            crate::history::favourite(&id, value).map(|entry| crate::history::entry_json(&entry))
        } else if path == "/history/edit" {
            let text = text_field(&request, "text");
            crate::history::edit_with_suggestions(&id, &text).map(|(entry, proposed)| {
                learned(&proposed);
                let mut value: serde_json::Value =
                    serde_json::from_str(&crate::history::entry_json(&entry)).unwrap_or_default();
                value["suggestions"] = serde_json::json!(proposed
                    .iter()
                    .map(|(heard, written)| serde_json::json!({
                        "heard": heard,
                        "written": written
                    }))
                    .collect::<Vec<_>>());
                value.to_string()
            })
        } else {
            crate::history::remove(&id).map(|()| "{\"deleted\":true}".to_string())
        };
        return match outcome {
            Ok(answer) => respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "404 Not Found", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "GET" && path == "/locale" {
        let stored = crate::config::load();
        let locale = crate::locales::check(&stored.locale)
            .unwrap_or_else(|_| crate::locales::DEFAULT.to_string());
        let available: Vec<serde_json::Value> = crate::locales::available()
            .into_iter()
            .map(|(code, name)| serde_json::json!({ "code": code, "name": name }))
            .collect();
        let body = serde_json::json!({
            "locale": locale,
            "available": available,
            "strings": crate::locales::strings(&locale),
        })
        .to_string();
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    respond_json(stream, "404 Not Found", "{\"error\":\"not found\"}").map(|()| Flow::Serve)
}

fn learned(pairs: &[(String, String)]) {
    if pairs.is_empty() {
        return;
    }
    let pairs = pairs.to_vec();
    std::thread::spawn(move || {
        for (heard, written) in pairs.iter().take(5) {
            let payload = serde_json::json!({
                "action": "vocabulary/add",
                "kind": "term",
                "wrong": heard,
                "right": written,
            });
            if let Err(error) = crate::intelligence::run(&payload) {
                println!("the correction {heard} to {written} was not kept: {error}");
            }
        }
    });
}
