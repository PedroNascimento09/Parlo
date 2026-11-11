use super::super::auth::authorised;
use super::super::http::respond_json;
use super::super::{answer_with, json_string, Flow, KeyRequest, Session, Settings, DUB_TIMEOUT};
use crate::serve::http::Stream;
use parlo_core::settings::Intelligence;
use std::time::Duration;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Option<Result<Flow, String>> {
    if !matches!(
        path,
        "/settings"
            | "/intelligence"
            | "/intelligence/key"
            | "/intelligence/key/delete"
            | "/intelligence/test"
    ) {
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
    if method == "GET" && path == "/settings" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return respond_json(stream, "200 OK", &session.as_json()).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/settings" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let requested: Settings = match serde_json::from_slice(body) {
            Ok(settings) => settings,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        let wanted_keys = [
            ("dictation", requested.shortcut.as_deref()),
            ("hands free", requested.hands_free.as_deref()),
            ("captions", requested.captions.as_deref()),
            ("undo", requested.undo.as_deref()),
            ("copy", requested.copy_shortcut.as_deref()),
            ("command", requested.command.as_deref()),
        ];
        if wanted_keys.iter().any(|(_, text)| text.is_some()) {
            let mut config = crate::config::load();
            let before = [
                config.shortcut.clone(),
                config.hands_free.clone(),
                config.captions.clone(),
                config.undo.clone(),
                config.copy_shortcut.clone(),
                config.command.clone(),
            ];
            let mut after = before.clone();
            for (index, (label, text)) in wanted_keys.iter().enumerate() {
                let Some(text) = *text else {
                    continue;
                };
                if index == 4 && text.trim().is_empty() {
                    after[index].clear();
                    continue;
                }
                let parsed = match parlo_core::Shortcut::parse(text) {
                    Ok(shortcut) => shortcut,
                    Err(error) => {
                        return respond_json(
                            stream,
                            "400 Bad Request",
                            &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                        )
                        .map(|()| Flow::Serve)
                    }
                };
                let canonical = parsed.text();
                if let Some(other) = (0..after.len()).find(|other| {
                    *other != index
                        && parlo_core::Shortcut::parse(&after[*other])
                            .is_ok_and(|shortcut| shortcut == parsed)
                }) {
                    let message = format!(
                        "{} is already the {} shortcut, so {label} needs another one.",
                        canonical,
                        [
                            "dictation",
                            "hands free",
                            "captions",
                            "undo",
                            "copy",
                            "command"
                        ][other]
                    );
                    return respond_json(
                        stream,
                        "409 Conflict",
                        &format!("{{\"error\":\"{}\"}}", json_string(&message)),
                    )
                    .map(|()| Flow::Serve);
                }
                if canonical != before[index]
                    && parlo_core::shortcut::taken_by_another_program(&parsed)
                {
                    let message = format!("{canonical} is already used by another program.");
                    return respond_json(
                        stream,
                        "409 Conflict",
                        &format!("{{\"error\":\"{}\"}}", json_string(&message)),
                    )
                    .map(|()| Flow::Serve);
                }
                after[index] = canonical;
            }
            config.shortcut = after[0].clone();
            config.hands_free = after[1].clone();
            config.captions = after[2].clone();
            config.undo = after[3].clone();
            config.copy_shortcut = after[4].clone();
            config.command = after[5].clone();
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
            {
                let mut chosen = session.choose();
                chosen.shortcut = after[0].clone();
                chosen.hands_free = after[1].clone();
                chosen.captions = after[2].clone();
                chosen.undo = after[3].clone();
                chosen.copy_shortcut = after[4].clone();
                chosen.command = after[5].clone();
            }
            let only_shortcut = requested.engine.is_none()
                && requested.stt.is_none()
                && requested.from.is_none()
                && requested.to.is_none()
                && requested.write.is_none()
                && requested.unload.is_none()
                && requested.channel.is_none()
                && requested.share.is_none()
                && requested.share_pin.is_none()
                && requested.share_port.is_none()
                && requested.locale.is_none();
            if only_shortcut {
                return respond_json(stream, "200 OK", &session.as_json()).map(|()| Flow::Serve);
            }
        }

        if let Some(minutes) = requested.unload {
            if minutes > 1_440 {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"the models cannot be kept for longer than a day\"}",
                )
                .map(|()| Flow::Serve);
            }
            let mut config = crate::config::load();
            config.unload = minutes;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        if let Some(days) = requested.history_days {
            if days > 3_650 {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"the history cannot be kept for longer than ten years\"}",
                )
                .map(|()| Flow::Serve);
            }
            let mut config = crate::config::load();
            config.history_days = days;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
            let _ = crate::history::prune(days);
        }

        if let Some(name) = requested.name {
            let cleaned = name.split_whitespace().collect::<Vec<_>>().join(" ");
            if cleaned.chars().count() > 40 {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"a name cannot be longer than 40 characters\"}",
                )
                .map(|()| Flow::Serve);
            }
            let mut config = crate::config::load();
            config.name = cleaned;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        if let Some(compute) = requested.compute.as_deref() {
            let wanted = match parlo_core::settings::check_compute(compute) {
                Ok(value) => value,
                Err(error) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                    )
                    .map(|()| Flow::Serve)
                }
            };
            let mut config = crate::config::load();
            config.compute = wanted;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        if let Some(channel) = requested.channel.as_deref() {
            let wanted = match crate::update::check_channel(channel) {
                Ok(value) => value,
                Err(error) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                    )
                    .map(|()| Flow::Serve)
                }
            };
            let mut config = crate::config::load();
            config.update_channel = wanted;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        if let Some(locale) = requested.locale.as_deref() {
            let wanted = match crate::locales::check(locale) {
                Ok(value) => value,
                Err(error) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                    )
                    .map(|()| Flow::Serve)
                }
            };
            let mut config = crate::config::load();
            config.locale = wanted;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        if requested.share.is_some()
            || requested.share_pin.is_some()
            || requested.share_port.is_some()
        {
            let mut config = crate::config::load();
            if let Some(share) = requested.share {
                config.share = share;
            }
            if let Some(pin) = requested.share_pin.as_deref() {
                if pin.trim().is_empty() {
                    config.share_pin.clear();
                } else {
                    match parlo_core::settings::check_pin(pin) {
                        Ok(value) => config.share_pin = value,
                        Err(error) => {
                            return respond_json(
                                stream,
                                "400 Bad Request",
                                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                            )
                            .map(|()| Flow::Serve)
                        }
                    }
                }
            }
            if let Some(share_port) = requested.share_port {
                if share_port < 1024 {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        "{\"error\":\"the share port must be 1024 or higher\"}",
                    )
                    .map(|()| Flow::Serve);
                }
                config.share_port = share_port;
            }
            if config.share && config.share_pin.is_empty() {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"set a PIN before sharing this computer\"}",
                )
                .map(|()| Flow::Serve);
            }
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        if let Some(manage) = requested.manage_memory {
            let mut config = crate::config::load();
            config.manage_memory = manage;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        let only_settings = requested.engine.is_none()
            && requested.stt.is_none()
            && requested.from.is_none()
            && requested.to.is_none()
            && requested.write.is_none()
            && requested.unload.is_none()
            && requested.manage_memory.is_none()
            && requested.compute.is_none()
            && requested.channel.is_none()
            && requested.share.is_none()
            && requested.share_pin.is_none()
            && requested.share_port.is_none()
            && requested.locale.is_none();
        if only_settings {
            return respond_json(stream, "200 OK", &session.as_json()).map(|()| Flow::Serve);
        }

        let engine = requested
            .engine
            .unwrap_or_else(|| session.chosen().engine.clone());
        let stt = requested
            .stt
            .unwrap_or_else(|| session.chosen().stt.clone());
        let from = requested
            .from
            .unwrap_or_else(|| session.chosen().from.clone())
            .to_ascii_lowercase();
        let to = requested
            .to
            .unwrap_or_else(|| session.chosen().to.clone())
            .to_ascii_lowercase();

        let write = requested
            .write
            .unwrap_or_else(|| session.chosen().write.clone())
            .to_ascii_lowercase();
        if !write.is_empty() && !crate::config::check_language(&write) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"the language for dictation needs a two letter code\"}",
            )
            .map(|()| Flow::Serve);
        }

        println!("switching to {engine} on {stt}, {from} to {to}");
        let _ = stream.set_write_timeout(Some(DUB_TIMEOUT));
        let stored = (
            engine.clone(),
            stt.clone(),
            from.clone(),
            to.clone(),
            write.clone(),
        );
        let outcome = session.switch(engine, stt, from, to, write);
        if outcome.is_ok() {
            let mut config = crate::config::load();
            config.engine = stored.0;
            config.stt = stored.1;
            config.from = stored.2;
            config.to = stored.3;
            config.write = stored.4;
            if let Err(error) = crate::config::save(&config) {
                println!("could not store the settings: {error}");
            }
        }
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        if let Err(error) = outcome {
            return respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }

        {
            let engine_id = session.pipeline().engine_id().to_string();
            let chosen = session.chosen();
            println!(
                "  ready       {engine_id} on {}, {} to {}",
                chosen.stt, chosen.from, chosen.to
            );
        }
        return respond_json(stream, "200 OK", &session.as_json()).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/intelligence" {
        let settings = crate::config::load().intelligence;
        let state = crate::intelligence::state_json(&settings, &crate::intelligence::status());
        return respond_json(stream, "200 OK", &state).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/intelligence" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let mut requested: Intelligence = match serde_json::from_slice(body) {
            Ok(settings) => settings,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        requested.mode = requested.mode.trim().to_ascii_lowercase();
        requested.clean = requested.clean.trim().to_ascii_lowercase();
        requested.provider = requested.provider.trim().to_ascii_lowercase();
        requested.model = requested.model.trim().to_string();
        requested.base_url = requested.base_url.trim().trim_end_matches('/').to_string();
        requested.local_url = requested.local_url.trim().to_string();
        requested.local_model = requested.local_model.trim().to_string();
        if let Err(error) = crate::intelligence::check(&requested) {
            return respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        let mut config = crate::config::load();
        config.intelligence = requested;
        if let Err(error) = crate::config::save(&config) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        let state =
            crate::intelligence::state_json(&config.intelligence, &crate::intelligence::status());
        return respond_json(stream, "200 OK", &state).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/intelligence/key" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: KeyRequest = match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        if !crate::intelligence::PROVIDERS.contains(&request.provider.as_str()) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"choose a provider before saving a key\"}",
            )
            .map(|()| Flow::Serve);
        }
        if request.key.trim().is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"a key is needed to store one\"}",
            )
            .map(|()| Flow::Serve);
        }
        let payload = serde_json::json!({
            "action": "store",
            "provider": request.provider,
            "key": request.key,
        });
        return answer_with(
            stream,
            crate::intelligence::run(&payload),
            "{\"stored\":true}",
        );
    }

    if method == "POST" && path == "/intelligence/key/delete" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: KeyRequest = match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        if !crate::intelligence::PROVIDERS.contains(&request.provider.as_str()) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"choose a provider before removing a key\"}",
            )
            .map(|()| Flow::Serve);
        }
        let payload = serde_json::json!({
            "action": "delete",
            "provider": request.provider,
        });
        return answer_with(
            stream,
            crate::intelligence::run(&payload),
            "{\"deleted\":true}",
        );
    }

    if method == "POST" && path == "/intelligence/test" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let payload = serde_json::json!({ "action": "test" });
        return match crate::intelligence::run(&payload) {
            Ok(reply) => match reply.get("error").and_then(serde_json::Value::as_str) {
                Some(error) => respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(error)),
                )
                .map(|()| Flow::Serve),
                None => respond_json(stream, "200 OK", &reply.to_string()).map(|()| Flow::Serve),
            },
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }
    respond_json(stream, "404 Not Found", "{\"error\":\"not found\"}").map(|()| Flow::Serve)
}
