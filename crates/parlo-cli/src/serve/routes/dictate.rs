use super::super::auth::authorised;
use super::super::http::{error_json, header, respond_json, Stream};
use super::super::session::Session;
use super::super::{absorb, dictation_target, json_string, respond_audio, Flow};
use crate::commands;
use crate::insert;
use crate::undo;
use parlo_core::wav::{read_pcm16, write_pcm16};
use parlo_core::{AudioFormat, Wav};
use std::time::{Duration, Instant};

#[derive(serde::Deserialize)]
struct StreamedRequest {
    text: String,
    #[serde(default)]
    seconds: f64,
}

#[derive(serde::Deserialize)]
struct InsertRequest {
    text: String,
    #[serde(default)]
    replace: String,
    #[serde(default)]
    live: bool,
    #[serde(default)]
    latency_ms: u64,
    #[serde(default)]
    polish: bool,
}

const POLISH_GRACE_MS: u64 = 400;

pub(crate) fn within_grace(polish_ms: u64) -> bool {
    polish_ms <= POLISH_GRACE_MS
}

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
        (method, path),
        ("POST", "/dictate/begin")
            | ("POST", "/undo")
            | ("POST", "/insert")
            | ("POST", "/warm")
            | ("POST", "/dictate")
            | ("POST", "/translate")
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
    if method == "POST" && path == "/dictate/begin" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match session.capture_target() {
            Ok(()) => respond_json(stream, "200 OK", "{\"target\":true}").map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/undo" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let last = match undo::take() {
            Ok(last) => last,
            Err(error) => {
                return respond_json(stream, "409 Conflict", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        return match insert::erase(last.window, last.characters) {
            Ok(()) => respond_json(
                stream,
                "200 OK",
                &format!("{{\"erased\":{}}}", last.characters),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                undo::restore(&last);
                respond_json(stream, "409 Conflict", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/warm" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        session.warm();
        return respond_json(stream, "200 OK", "{\"warm\":true}").map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/insert" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: InsertRequest = match serde_json::from_slice(body) {
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
        if request.text.trim().is_empty() && (!request.live || request.replace.is_empty()) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"nothing to write\"}",
            )
            .map(|()| Flow::Serve);
        }
        let latency = request.latency_ms.min(60_000);
        let outcome = if request.live {
            session.live_text(&request.text, &request.replace)
        } else {
            session.insert_text(&request.text, &request.replace, latency)
        };
        return match outcome {
            Ok(()) => {
                let mut polished = String::new();
                let mut polish_ms = 0u64;
                if request.polish && !request.live {
                    let began = Instant::now();
                    if let Ok(cleaned) = session.clean_text(request.text.trim(), None) {
                        polish_ms = began.elapsed().as_millis() as u64;
                        let cleaned = cleaned.trim();
                        if within_grace(polish_ms)
                            && !cleaned.is_empty()
                            && cleaned != request.text.trim()
                            && session.insert_text(cleaned, &request.text, 0).is_ok()
                        {
                            polished = cleaned.to_string();
                        }
                    }
                }
                respond_json(
                    stream,
                    "200 OK",
                    &format!(
                        "{{\"inserted\":true,\"polished\":\"{}\",\"polish_ms\":{polish_ms}}}",
                        json_string(&polished)
                    ),
                )
                .map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/dictate" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let streamed: Option<StreamedRequest> = serde_json::from_slice(body)
            .ok()
            .filter(|request: &StreamedRequest| !request.text.trim().is_empty());
        let wav = if streamed.is_some() {
            None
        } else {
            match read_pcm16(body) {
                Ok(wav) => Some(wav),
                Err(error) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &format!("{{\"error\":\"bad wav: {error}\"}}"),
                    )
                    .map(|()| Flow::Serve);
                }
            }
        };
        if let Some(wav) = &wav {
            if wav.format.channels() != 1 || wav.format.sample_rate() != 16_000 {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"input must be 16 kHz mono pcm16\"}",
                )
                .map(|()| Flow::Serve);
            }
            if wav.samples.is_empty() {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"the file is empty\"}",
                )
                .map(|()| Flow::Serve);
            }
        }

        let seconds = match (&streamed, &wav) {
            (Some(request), _) => request.seconds.clamp(0.0, 3_600.0) as f32,
            (_, Some(wav)) => wav.samples.len() as f32 / 16_000.0,
            (None, None) => 0.0,
        };
        let mut pcm16 = Vec::new();
        if let Some(wav) = &wav {
            pcm16.reserve(wav.samples.len() * 2);
            for sample in &wav.samples {
                let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                pcm16.extend_from_slice(&value.to_le_bytes());
            }
        }
        let write = header(headers, "x-parlo-write")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !write.is_empty() && !crate::config::check_language(&write) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"x-parlo-write needs a two letter language code\"}",
            )
            .map(|()| Flow::Serve);
        }
        let clean_header = header(headers, "x-parlo-clean")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !clean_header.is_empty() && !["on", "off"].contains(&clean_header.as_str()) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"x-parlo-clean needs on or off\"}",
            )
            .map(|()| Flow::Serve);
        }
        let stored = crate::config::load();
        let clean = if clean_header == "off" {
            Some("original".to_string())
        } else {
            Some(stored.intelligence.clean.clone())
        };
        let style_header = header(headers, "x-parlo-style")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !style_header.is_empty() && !crate::style::is_known(&style_header) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"x-parlo-style needs formal, casual or excited\"}",
            )
            .map(|()| Flow::Serve);
        }
        let app = session
            .target_app()
            .unwrap_or_else(|| "unknown".to_string());
        let profile = stored.profiles.get(&app.to_ascii_lowercase());
        let style = if !style_header.is_empty() {
            style_header
        } else if let Some(wanted) = profile.map(|profile| profile.style.clone()) {
            if crate::style::is_known(&wanted) {
                wanted
            } else {
                String::new()
            }
        } else if crate::style::is_known(&stored.style) {
            stored.style.clone()
        } else {
            String::new()
        };
        let write = if !write.is_empty() {
            write
        } else {
            profile
                .map(|profile| profile.write.clone())
                .unwrap_or_default()
        };
        let chosen = session.chosen();
        let from = chosen.from.clone();
        let to = chosen.to.clone();
        let target = dictation_target(&write, &from, &to, stored.features.translation);
        if streamed.is_some() && target != from {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"a text body can only be written in the language it was spoken\"}",
            )
            .map(|()| Flow::Serve);
        }
        let started = Instant::now();
        let _ = stream.set_write_timeout(Some(Duration::from_secs(120)));
        let (text, translation, note, written) =
            match streamed.as_ref().filter(|_| target == from) {
                Some(request) => match session.clean_text(request.text.trim(), clean.as_deref()) {
                    Ok(edited) => (edited, String::new(), String::new(), target.clone()),
                    Err(error) => {
                        return respond_json(
                            stream,
                            "500 Internal Server Error",
                            &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                        )
                        .map(|()| Flow::Serve);
                    }
                },
                None => match session.write(&pcm16, &target, clean.as_deref()) {
                    Ok((text, translation)) => (text, translation, String::new(), target.clone()),
                    Err(error) if target != from && error.contains("no translation model") => {
                        match session.write(&pcm16, &from, clean.as_deref()) {
                            Ok((text, translation)) => (
                                text,
                                translation,
                                "The translation model is missing, so the sentence stayed in the language spoken."
                                    .to_string(),
                                from.clone(),
                            ),
                            Err(error) => {
                                return respond_json(
                                    stream,
                                    "500 Internal Server Error",
                                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                                )
                                .map(|()| Flow::Serve);
                            }
                        }
                    }
                    Err(error) => {
                        return respond_json(
                            stream,
                            "500 Internal Server Error",
                            &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                        )
                        .map(|()| Flow::Serve);
                    }
                },
            };
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        let text = commands::apply(&text, &from);
        let text = crate::snippets::apply(&text, &stored.snippets);
        let text = crate::style::shape(&text, &style);
        let mut entry =
            crate::history::Entry::now(&text, &translation, &from, &written, &app, "dictate");
        entry.seconds = f64::from(seconds);
        if let Err(error) = crate::history::append(&entry) {
            println!("could not store the dictation: {error}");
        }
        println!(
            "dictated {seconds:.1} s in {} ms",
            started.elapsed().as_millis()
        );
        let body = format!(
            "{{\"text\":\"{}\",\"translation\":\"{}\",\"seconds\":{seconds:.3},\"latency_ms\":{},\"note\":\"{}\"}}",
            json_string(&text),
            json_string(&translation),
            started.elapsed().as_millis(),
            json_string(&note)
        );
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if !authorised(headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }

    let wav = match read_pcm16(body) {
        Ok(wav) => wav,
        Err(error) => {
            return respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"bad wav: {error}\"}}"),
            )
            .map(|()| Flow::Serve);
        }
    };
    if wav.format.channels() != 1 || wav.format.sample_rate() != 16_000 {
        return respond_json(
            stream,
            "400 Bad Request",
            "{\"error\":\"input must be 16 kHz mono pcm16\"}",
        )
        .map(|()| Flow::Serve);
    }

    let mut samples = Vec::new();
    let mut transcript = String::new();
    let _working = session.working();
    let mut pipeline = session.pipeline();
    for chunk in wav.samples.chunks(320) {
        for output in pipeline.push(chunk) {
            if let Some(error) = absorb(output, &mut samples, &mut transcript) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }
    }
    for output in pipeline.finish() {
        if let Some(error) = absorb(output, &mut samples, &mut transcript) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
    }

    let translated = Wav {
        format: AudioFormat::mono(16_000).map_err(|error| error.to_string())?,
        samples,
    };
    respond_audio(stream, &write_pcm16(&translated), &transcript).map(|()| Flow::Serve)
}

#[cfg(test)]
mod tests {
    use super::within_grace;

    #[test]
    fn the_polish_is_replaced_only_inside_its_grace() {
        assert!(within_grace(0));
        assert!(within_grace(150));
        assert!(within_grace(400));
        assert!(!within_grace(401));
        assert!(!within_grace(1_200));
    }
}
