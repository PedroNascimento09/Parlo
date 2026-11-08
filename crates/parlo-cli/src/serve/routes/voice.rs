use super::super::auth::authorised;
use super::super::http::{error_json, header, respond_json, Stream};
use super::super::{
    consent_for, json_string, now_seconds, respond_audio, respond_named, synthesized_wav,
    voice_store, wav_seconds, Flow, Session, VoiceRequest, VoiceTarget, DEFAULT_VOICE_TEXT,
    MAX_SAMPLE_SECONDS, MIN_SAMPLE_SECONDS,
};

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Option<Result<Flow, String>> {
    if !(path == "/voice" || path.starts_with("/voice/")) {
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
    if method == "GET" && path == "/voice" {
        return match voice_store().list() {
            Ok(profiles) => {
                let listed = serde_json::to_string(&profiles).unwrap_or_else(|_| "[]".to_string());
                respond_json(stream, "200 OK", &format!("{{\"profiles\":{listed}}}"))
                    .map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/voice" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: VoiceRequest = match serde_json::from_slice(body) {
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
        let purpose = request
            .purpose
            .unwrap_or_else(|| "local voice cloning and playback".to_string());
        let scope = request
            .scope
            .unwrap_or_else(|| "on this computer".to_string());
        let consent = match consent_for(
            request.origin.as_deref(),
            request.declared_authorisation,
            purpose,
            scope,
            now_seconds(),
        ) {
            Ok(consent) => consent,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
        let language = request.language.unwrap_or_else(|| "pt-PT".to_string());
        let store = voice_store();
        let created = store
            .create(&request.label, &language, consent)
            .and_then(|profile| {
                if request.encrypt {
                    store.set_encryption(&profile.id, true)
                } else {
                    Ok(profile)
                }
            });
        return match created {
            Ok(profile) => {
                let body = serde_json::to_string(&profile).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "201 Created", &body).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && (path == "/voice/revoke" || path == "/voice/delete") {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let target: VoiceTarget = match serde_json::from_slice(body) {
            Ok(target) => target,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        let store = voice_store();
        let outcome = if path == "/voice/revoke" {
            store
                .revoke(&target.id, now_seconds())
                .map(|profile| serde_json::to_string(&profile).unwrap_or_else(|_| "{}".to_string()))
        } else {
            let removed = store
                .get(&target.id)
                .map(|profile| profile.artefacts)
                .unwrap_or_default();
            store.delete(&target.id).map(|()| {
            serde_json::json!({
                "deleted": true,
                "removed": removed,
                "message": "Deleted. The sample and everything made from it are gone; copies in system backups are outside Parlo's reach.",
            })
            .to_string()
        })
        };
        return match outcome {
            Ok(body) => respond_json(stream, "200 OK", &body).map(|()| Flow::Serve),
            Err(error) => {
                let status = match error {
                    parlo_core::VoiceError::NotFound(_) => "404 Not Found",
                    _ => "400 Bad Request",
                };
                respond_json(
                    stream,
                    status,
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/voice/sample" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let Some(id) = header(headers, "x-parlo-voice") else {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"the sample needs a voice profile id\"}",
            )
            .map(|()| Flow::Serve);
        };
        let audio = match parlo_core::wav::read_pcm16(body) {
            Ok(audio) => audio,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json(&format!("the sample is not a readable wav: {error}")),
                )
                .map(|()| Flow::Serve)
            }
        };
        let seconds = wav_seconds(body);
        if !(MIN_SAMPLE_SECONDS..=MAX_SAMPLE_SECONDS).contains(&seconds) {
            return respond_json(
            stream,
            "400 Bad Request",
            &error_json(&format!(
                "the sample is {seconds:.1} s; a voice profile takes between {MIN_SAMPLE_SECONDS:.0} and {MAX_SAMPLE_SECONDS:.0} s of speech"
            )),
        )
        .map(|()| Flow::Serve);
        }
        if let Some(problem) = crate::sample::problem(&audio.samples, seconds) {
            return respond_json(stream, "400 Bad Request", &error_json(&problem))
                .map(|()| Flow::Serve);
        }
        return match voice_store().save_sample(&id, body) {
            Ok(profile) => {
                let saved = serde_json::to_string(&profile).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "200 OK", &saved).map(|()| Flow::Serve)
            }
            Err(error) => {
                let status = match error {
                    parlo_core::VoiceError::NotFound(_) => "404 Not Found",
                    _ => "400 Bad Request",
                };
                respond_json(stream, status, &error_json(&error.to_string())).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/voice/sample/play" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let target: VoiceTarget = match serde_json::from_slice(body) {
            Ok(target) => target,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        let store = voice_store();
        match store.sample(&target.id) {
            Ok(bytes) => return respond_audio(stream, &bytes, "").map(|()| Flow::Serve),
            Err(error) => {
                let revoked = store
                    .get(&target.id)
                    .map(|profile| !profile.usable())
                    .unwrap_or(false);
                if !revoked {
                    return respond_json(stream, "404 Not Found", &error_json(&error.to_string()))
                        .map(|()| Flow::Serve);
                }
            }
        }
        return match session
            .speak(DEFAULT_VOICE_TEXT)
            .and_then(|(pcm, rate)| synthesized_wav(&pcm, rate))
        {
            Ok(audio) => respond_audio(stream, &audio, "").map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("the default voice did not answer: {error}")),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/voice/export" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let target: VoiceTarget = match serde_json::from_slice(body) {
            Ok(target) => target,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        let store = voice_store();
        let profile = match store.get(&target.id) {
            Ok(profile) => profile,
            Err(error) => {
                return respond_json(stream, "404 Not Found", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        let sample = store.sample(&target.id).unwrap_or_default();
        let exported = crate::voicefile::export(&profile, &sample);
        return respond_named(stream, &format!("parlo-voice-{}.txt", target.id), &exported)
            .map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/voice/import" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match crate::voicefile::import_into(&voice_store(), body) {
            Ok(profile) => {
                let text = serde_json::to_string(&profile).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "201 Created", &text).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/voice/compare" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        #[derive(serde::Deserialize)]
        struct CompareRequest {
            text: String,
            #[serde(default)]
            voices: Vec<String>,
            #[serde(default)]
            pack: Option<String>,
        }
        let request: CompareRequest = match serde_json::from_slice(body) {
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
        let voices = match request.pack.as_deref() {
            Some(name) => {
                if !request.voices.is_empty() {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &error_json("send either two voices or a pack, not both"),
                    )
                    .map(|()| Flow::Serve);
                }
                match crate::packs::read(name) {
                    Ok(pack) => pack.voices,
                    Err(error) => {
                        return respond_json(stream, "400 Bad Request", &error_json(&error))
                            .map(|()| Flow::Serve)
                    }
                }
            }
            None => request.voices.clone(),
        };
        let specs =
            match crate::compare::plan(&request.text, &voices, crate::compare::revoked_in_store) {
                Ok(specs) => specs,
                Err(error) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                    )
                    .map(|()| Flow::Serve)
                }
            };
        let (readings, samples, rate) =
            match crate::compare::render(&specs, &request.text, |spec, text| {
                session.speak_with_voice(&spec.file, text)
            }) {
                Ok(result) => result,
                Err(error) => {
                    return respond_json(
                        stream,
                        "500 Internal Server Error",
                        &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                    )
                    .map(|()| Flow::Serve)
                }
            };
        let wav = match crate::compare::to_wav(&samples, rate) {
            Ok(wav) => wav,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
        use base64::Engine as _;
        let body = serde_json::json!({
            "text": request.text.trim(),
            "readings": readings,
            "sample_rate": rate,
            "audio": base64::engine::general_purpose::STANDARD.encode(&wav),
        })
        .to_string();
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/voice/packs" {
        return match crate::packs::list() {
            Ok(packs) => respond_json(
                stream,
                "200 OK",
                &serde_json::json!({ "packs": packs }).to_string(),
            )
            .map(|()| Flow::Serve),
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/voice/packs" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match crate::packs::import(body, crate::compare::revoked_in_store) {
            Ok(pack) => respond_json(
                stream,
                "201 Created",
                &serde_json::to_string(&pack).unwrap_or_else(|_| "{}".to_string()),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "GET" && path.starts_with("/voice/packs/") {
        let name = path["/voice/packs/".len()..].trim();
        return match crate::packs::export(name) {
            Ok(bytes) => {
                respond_named(stream, &format!("{name}.json"), &bytes).map(|()| Flow::Serve)
            }
            Err(error) => {
                respond_json(stream, "404 Not Found", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }
    respond_json(stream, "404 Not Found", "{\"error\":\"not found\"}").map(|()| Flow::Serve)
}
