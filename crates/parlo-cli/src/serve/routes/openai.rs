use std::net::TcpStream;

use parlo_core::resample::Resampler;
use parlo_core::wav::read_pcm16;

use super::super::auth::authorised;
use super::super::http::{header, respond, respond_json};
use super::super::multipart::{self, Part};
use super::super::{Flow, Session};

pub(crate) fn transcriptions(
    stream: &mut TcpStream,
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
    stream: &mut TcpStream,
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

fn prepare(
    stream: &mut TcpStream,
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

    let wav = match read_pcm16(&file.data) {
        Ok(wav) => wav,
        Err(error) => {
            let _ = fail(
                stream,
                "400 Bad Request",
                &format!("the file is not a 16-bit PCM wav: {error}"),
                "invalid_request_error",
                Some("file"),
            );
            return Err(());
        }
    };
    if wav.samples.is_empty() {
        let _ = fail(
            stream,
            "400 Bad Request",
            "the file has no audio",
            "invalid_request_error",
            Some("file"),
        );
        return Err(());
    }

    let channels = wav.format.channels() as usize;
    let seconds = wav.samples.len() as f32 / channels as f32 / wav.format.sample_rate() as f32;
    let mut samples: Vec<f32> = wav
        .samples
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect();
    if wav.format.sample_rate() != 16_000 {
        let mut resampler = Resampler::new(wav.format.sample_rate(), 16_000);
        let mut resampled = Vec::new();
        resampler.process(&samples, &mut resampled);
        samples = resampled;
    }

    let mut pcm16 = Vec::with_capacity(samples.len() * 2);
    for sample in &samples {
        pcm16
            .extend_from_slice(&((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes());
    }
    Ok((pcm16, seconds, format))
}

fn answer(
    stream: &mut TcpStream,
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
    stream: &mut TcpStream,
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
