use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use parlo_core::wav::write_pcm16;
use parlo_core::{AudioFormat, Wav};

use super::super::auth::authorised;
use super::super::http::{header, respond, respond_json};
use super::super::multipart::{self, Part};
use super::super::{Flow, Session};
use crate::serve::http::Stream;

const OPENAI_AUDIO_MODELS: [&str; 3] = ["whisper-1", "whisper", "parakeet"];

#[derive(serde::Deserialize)]
struct SpeechRequest {
    input: String,
    #[serde(default)]
    response_format: Option<String>,
    #[serde(default)]
    speed: Option<f32>,
}

pub(crate) fn transcriptions(
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return fail(
            stream,
            "401 Unauthorized",
            "wrong token",
            "invalid_request_error",
            None,
        )
        .map(|()| Flow::Serve);
    }
    let (pcm16, seconds, format) = match prepare(stream, headers, body) {
        Ok(prepared) => prepared,
        Err(()) => return Ok(Flow::Serve),
    };
    let text = match session.dictate(&pcm16) {
        Ok(text) => text.trim().to_string(),
        Err(error) => {
            return fail(
                stream,
                "500 Internal Server Error",
                &error,
                "server_error",
                None,
            )
            .map(|()| Flow::Serve)
        }
    };
    let language = session.chosen().from.clone();
    answer(stream, &text, seconds, &language, &format, "transcribe")
}

pub(crate) fn translations(
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return fail(
            stream,
            "401 Unauthorized",
            "wrong token",
            "invalid_request_error",
            None,
        )
        .map(|()| Flow::Serve);
    }
    let (pcm16, seconds, format) = match prepare(stream, headers, body) {
        Ok(prepared) => prepared,
        Err(()) => return Ok(Flow::Serve),
    };
    let text = match session.translate_audio(&pcm16) {
        Ok(text) => text,
        Err(error) => {
            return fail(
                stream,
                "500 Internal Server Error",
                &error,
                "server_error",
                None,
            )
            .map(|()| Flow::Serve)
        }
    };
    let language = session.chosen().from.clone();
    answer(stream, &text, seconds, &language, &format, "translate")
}

pub(crate) fn speech(
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return fail(
            stream,
            "401 Unauthorized",
            "wrong token",
            "invalid_request_error",
            None,
        )
        .map(|()| Flow::Serve);
    }
    let request: SpeechRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(error) => {
            return fail(
                stream,
                "400 Bad Request",
                &format!("the request is not valid json: {error}"),
                "invalid_request_error",
                None,
            )
            .map(|()| Flow::Serve)
        }
    };
    let input = request.input.trim();
    if input.is_empty() {
        return fail(
            stream,
            "400 Bad Request",
            "input must not be empty",
            "invalid_request_error",
            Some("input"),
        )
        .map(|()| Flow::Serve);
    }
    if input.chars().count() > 4096 {
        return fail(
            stream,
            "400 Bad Request",
            "input must be at most 4096 characters",
            "invalid_request_error",
            Some("input"),
        )
        .map(|()| Flow::Serve);
    }
    if let Some(speed) = request.speed {
        if !(0.25..=4.0).contains(&speed) {
            return fail(
                stream,
                "400 Bad Request",
                "speed must be between 0.25 and 4",
                "invalid_request_error",
                Some("speed"),
            )
            .map(|()| Flow::Serve);
        }
        if (speed - 1.0).abs() > 0.01 {
            return fail(
                stream,
                "400 Bad Request",
                "this build speaks at normal speed; speed must be 1",
                "invalid_request_error",
                Some("speed"),
            )
            .map(|()| Flow::Serve);
        }
    }
    let format = request
        .response_format
        .as_deref()
        .unwrap_or("mp3")
        .to_ascii_lowercase();
    if !matches!(format.as_str(), "wav" | "pcm" | "mp3") {
        return fail(
            stream,
            "400 Bad Request",
            "response_format must be wav, pcm or mp3 in this build",
            "invalid_request_error",
            Some("response_format"),
        )
        .map(|()| Flow::Serve);
    }
    let (pcm16, rate) = match session.speak(input) {
        Ok(speech) => speech,
        Err(error) => {
            return fail(
                stream,
                "500 Internal Server Error",
                &error,
                "server_error",
                None,
            )
            .map(|()| Flow::Serve)
        }
    };
    if pcm16.is_empty() {
        return fail(
            stream,
            "500 Internal Server Error",
            "the voice produced no audio",
            "server_error",
            None,
        )
        .map(|()| Flow::Serve);
    }
    if format == "pcm" {
        return respond(stream, "200 OK", "audio/pcm", &pcm16).map(|()| Flow::Serve);
    }
    let audio_format = match AudioFormat::mono(rate) {
        Ok(format) => format,
        Err(error) => {
            return fail(
                stream,
                "500 Internal Server Error",
                &error.to_string(),
                "server_error",
                None,
            )
            .map(|()| Flow::Serve)
        }
    };
    let samples = pcm16
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32_768.0)
        .collect();
    let wav = write_pcm16(&Wav {
        format: audio_format,
        samples,
    });
    if format == "mp3" {
        return match encode_mp3(&wav) {
            Ok(audio) => respond(stream, "200 OK", "audio/mpeg", &audio).map(|()| Flow::Serve),
            Err(error) => fail(
                stream,
                "500 Internal Server Error",
                &error,
                "server_error",
                None,
            )
            .map(|()| Flow::Serve),
        };
    }
    respond(stream, "200 OK", "audio/wav", &wav).map(|()| Flow::Serve)
}

fn encode_mp3(wav: &[u8]) -> Result<Vec<u8>, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let base = std::env::temp_dir().join(format!("parlo-speech-{}-{stamp}", std::process::id()));
    let source = base.with_extension("wav");
    let target = base.with_extension("mp3");
    if let Err(error) = std::fs::write(&source, wav) {
        let _ = std::fs::remove_file(&source);
        return Err(format!("the mp3 encoder could not read the audio: {error}"));
    }
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(&source)
        .args([
            "-ac",
            "1",
            "-ar",
            "24000",
            "-codec:a",
            "libmp3lame",
            "-b:a",
            "128k",
            "-f",
            "mp3",
        ])
        .arg(&target)
        .output();
    let _ = std::fs::remove_file(&source);
    let result = match output {
        Ok(output) if output.status.success() => std::fs::read(&target)
            .map_err(|error| format!("the mp3 encoder produced no readable audio: {error}"))
            .and_then(|audio| {
                if audio.is_empty() {
                    Err("the mp3 encoder produced no audio".to_string())
                } else {
                    Ok(audio)
                }
            }),
        Ok(output) => Err(format!(
            "the mp3 encoder failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(error) => Err(format!("the mp3 encoder could not start: {error}")),
    };
    let _ = std::fs::remove_file(&target);
    result
}

fn prepare(
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    body: &[u8],
) -> Result<(Vec<u8>, f32, String), ()> {
    let content_type = header(headers, "content-type").unwrap_or_default();
    let parts = match multipart::parse(&content_type, body) {
        Ok(parts) => parts,
        Err(error) => {
            let _ = fail(
                stream,
                "400 Bad Request",
                &error,
                "invalid_request_error",
                None,
            );
            return Err(());
        }
    };
    let Some(file) = parts.iter().find(|part| part.name == "file") else {
        let _ = fail(
            stream,
            "400 Bad Request",
            "the form needs a file",
            "invalid_request_error",
            Some("file"),
        );
        return Err(());
    };
    if file.data.is_empty() {
        let _ = fail(
            stream,
            "400 Bad Request",
            "the file is empty",
            "invalid_request_error",
            Some("file"),
        );
        return Err(());
    }
    let Some(model) = field(&parts, "model") else {
        let _ = fail(
            stream,
            "400 Bad Request",
            "the form needs a model",
            "invalid_request_error",
            Some("model"),
        );
        return Err(());
    };
    if !OPENAI_AUDIO_MODELS.contains(&model) {
        let _ = fail(
            stream,
            "400 Bad Request",
            "model must be whisper-1, whisper or parakeet in this build",
            "invalid_request_error",
            Some("model"),
        );
        return Err(());
    }

    let format = field(&parts, "response_format")
        .unwrap_or("json")
        .to_string();
    if !matches!(
        format.as_str(),
        "json" | "text" | "verbose_json" | "srt" | "vtt"
    ) {
        let _ = fail(
            stream,
            "400 Bad Request",
            "response_format must be json, text, verbose_json, srt or vtt",
            "invalid_request_error",
            Some("response_format"),
        );
        return Err(());
    }

    let (pcm16, seconds) = match super::super::audio::decode(&file.data) {
        Ok(decoded) => decoded,
        Err(error) => {
            let _ = fail(
                stream,
                "400 Bad Request",
                &error,
                "invalid_request_error",
                Some("file"),
            );
            return Err(());
        }
    };
    Ok((pcm16, seconds, format))
}

fn answer(
    stream: &mut dyn Stream,
    text: &str,
    seconds: f32,
    language: &str,
    format: &str,
    task: &str,
) -> Result<Flow, String> {
    match format {
        "text" => respond(
            stream,
            "200 OK",
            "text/plain; charset=utf-8",
            text.as_bytes(),
        )
        .map(|()| Flow::Serve),
        "srt" => {
            let body = format!(
                "1\n{} --> {}\n{}\n",
                timestamp(0.0, ','),
                timestamp(seconds, ','),
                text
            );
            respond(
                stream,
                "200 OK",
                "application/x-subrip; charset=utf-8",
                body.as_bytes(),
            )
            .map(|()| Flow::Serve)
        }
        "vtt" => {
            let body = format!(
                "WEBVTT\n\n{} --> {}\n{}\n",
                timestamp(0.0, '.'),
                timestamp(seconds, '.'),
                text
            );
            respond(stream, "200 OK", "text/vtt; charset=utf-8", body.as_bytes())
                .map(|()| Flow::Serve)
        }
        "verbose_json" => {
            let body = serde_json::json!({
                "task": task,
                "language": language,
                "duration": seconds,
                "text": text,
                "segments": [{
                    "id": 0,
                    "seek": 0,
                    "start": 0.0,
                    "end": seconds,
                    "text": text,
                }],
                "words": [],
            })
            .to_string();
            respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
        }
        _ => respond_json(
            stream,
            "200 OK",
            &serde_json::json!({ "text": text }).to_string(),
        )
        .map(|()| Flow::Serve),
    }
}

fn field<'a>(parts: &'a [Part], name: &str) -> Option<&'a str> {
    parts
        .iter()
        .find(|part| part.name == name)
        .and_then(|part| std::str::from_utf8(&part.data).ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn timestamp(seconds: f32, separator: char) -> String {
    let total_ms = (seconds.max(0.0) * 1000.0).round() as u64;
    let milliseconds = total_ms % 1000;
    let total_seconds = total_ms / 1000;
    let secs = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3600;
    format!("{hours:02}:{minutes:02}:{secs:02}{separator}{milliseconds:03}")
}

fn fail(
    stream: &mut dyn Stream,
    status: &str,
    message: &str,
    kind: &str,
    param: Option<&str>,
) -> Result<(), String> {
    let body = serde_json::json!({
        "error": {
            "message": message,
            "type": kind,
            "param": param,
            "code": serde_json::Value::Null,
        }
    })
    .to_string();
    respond_json(stream, status, &body)
}
