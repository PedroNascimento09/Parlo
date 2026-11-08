use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use parlo_core::{Consent, Store};

mod audio;
mod auth;
mod dub_exports;
mod dub_project;
pub(crate) mod http;
mod multipart;
mod routes;
use dub_project::{
    apply_dub, apply_project_edits, load_requested_project, project_json, write_cues, ProjectEdit,
};
mod listener;
mod session;
pub(crate) mod sse;
mod ticket;
mod tools;
mod ws;
use auth::authorised;
pub use auth::new_token;
use dub_exports::{host_is_share, origin_allowed_for, pin_offered, Share};
use http::{
    content_type, header, host_is_local, origin_allowed, percent_decode, respond_json, Stream,
};
pub use listener::{run, run_tls, TlsFiles};
use session::{rooms, Session};

#[derive(PartialEq, Eq)]
enum Flow {
    Serve,
    Stop,
}

const MAX_HEAD: usize = 16 * 1024;
const MAX_BODY: usize = 20 * 1024 * 1024;
const MAX_AUDIO_BODY: usize = 25 * 1024 * 1024;
const MAX_DUB_BODY: usize = 512 * 1024 * 1024;
const KEPT_DUBS: usize = 8;

const DUB_TIMEOUT: Duration = Duration::from_secs(3_600);
const VERSION: &str = env!("CARGO_PKG_VERSION");
const INDEX: &str = include_str!("../../web/index.html");
const STYLESHEET: &str = include_str!("../../web/app.css");
const SCRIPT: &str = include_str!("../../web/app.js");
const LANGUAGES: &str = include_str!("../../web/languages.js");
const ROUTES_SCRIPT: &str = include_str!("../../web/routes.js");
const LANGUAGES_TABLE: &str = include_str!("../../web/languages.json");
const ISLAND: &str = include_str!("../../web/island.html");
const CAPTIONS: &str = include_str!("../../web/captions.html");
const LIVE: &str = include_str!("../../web/live.html");

#[derive(serde::Deserialize)]
struct Settings {
    #[serde(default)]
    engine: Option<String>,
    #[serde(default)]
    stt: Option<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    write: Option<String>,
    #[serde(default)]
    shortcut: Option<String>,
    #[serde(default)]
    hands_free: Option<String>,
    #[serde(default)]
    captions: Option<String>,
    #[serde(default)]
    undo: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    history_days: Option<u32>,
    #[serde(default)]
    unload: Option<u32>,
    #[serde(default)]
    manage_memory: Option<bool>,
    #[serde(default)]
    compute: Option<String>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    locale: Option<String>,
    #[serde(default)]
    share: Option<bool>,
    #[serde(default)]
    share_pin: Option<String>,
    #[serde(default)]
    share_port: Option<u16>,
}

fn handle(
    stream: &mut dyn Stream,
    token: &str,
    port: u16,
    session: &Session,
    started: Instant,
    share: Option<&Share>,
    remote: bool,
) -> Result<Flow, String> {
    let (head, body) = read_http(stream)?;
    let mut lines = head.lines();
    let request = lines.next().unwrap_or_default();
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let mut headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
        .collect();

    if let Some(share) = share {
        if !host_is_share(&headers, share.port) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"host must name this share port\"}",
            )
            .map(|()| Flow::Serve);
        }
        if !pin_offered(&headers, query, &share.pin) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong PIN\"}")
                .map(|()| Flow::Serve);
        }
        headers.retain(|(name, _)| name != "authorization" && name != "x-parlo-token");
        headers.push(("authorization".to_string(), format!("Bearer {token}")));
    } else if remote {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
    } else if !host_is_local(&headers) {
        return respond_json(
            stream,
            "400 Bad Request",
            "{\"error\":\"host must be the local address\"}",
        )
        .map(|()| Flow::Serve);
    }

    if method == "POST" && !origin_allowed_for(&headers, port, share.is_some() || remote) {
        return respond_json(
            stream,
            "403 Forbidden",
            "{\"error\":\"origin is not the local app\"}",
        )
        .map(|()| Flow::Serve);
    }

    if let Some(answer) = routes::desktop::route(method, path, query, stream, &headers, token) {
        return answer;
    }

    if let Some(answer) =
        routes::retention::route(method, path, query, stream, &headers, token, &body)
    {
        return answer;
    }

    if let Some(answer) = routes::meetings::route(method, path, stream, &headers, token) {
        return answer;
    }

    if let Some(answer) =
        routes::meeting_live::route(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    if let Some(answer) = routes::vocabulary::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) =
        routes::settings_style::route(method, path, stream, &headers, token, &body)
    {
        return answer;
    }

    if let Some(answer) =
        routes::profiles_history::route(method, path, query, stream, &headers, token, &body)
    {
        return answer;
    }

    if let Some(answer) =
        routes::settings::route(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    if let Some(answer) = routes::dub::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) =
        routes::voice::route(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    if let Some(answer) = routes::dub_root::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) =
        routes::models::route_actions(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    if let Some(answer) = routes::update::route(method, path, stream, &headers, token) {
        return answer;
    }

    if let Some(answer) = routes::pages::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) =
        routes::listen::route(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    if let Some(answer) =
        routes::diarize::route(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    if let Some(answer) = routes::clipboard::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) =
        routes::session_state::route(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    if let Some((status, payload)) =
        tools::handle(method, path, query, &body, authorised(&headers, token))
    {
        return respond_json(stream, status, &payload).map(|()| Flow::Serve);
    }

    if let Some(answer) = routes::stream::route(
        method,
        path,
        query,
        stream,
        &headers,
        token,
        routes::stream::Listener { port, session },
    ) {
        return answer;
    }

    if let Some(answer) = routes::discovery::route(method, path, stream, session, started) {
        return answer;
    }

    if let Some(answer) = routes::models::route(method, path, stream, &headers, token, session) {
        return answer;
    }

    if let Some(answer) =
        routes::openai::route(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    if let Some(answer) = routes::mcp::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) = routes::batch::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) = routes::story::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) = routes::watch::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) = routes::jobs::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if let Some(answer) =
        routes::dictate::route(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    respond_json(stream, "404 Not Found", "{\"error\":\"not found\"}").map(|()| Flow::Serve)
}

#[derive(serde::Deserialize)]
struct KeyRequest {
    #[serde(default)]
    provider: String,
    #[serde(default)]
    key: String,
}

fn answer_with(
    stream: &mut dyn Stream,
    outcome: Result<serde_json::Value, String>,
    success: &str,
) -> Result<Flow, String> {
    match outcome {
        Ok(reply) => match reply.get("error").and_then(serde_json::Value::as_str) {
            Some(error) => respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(error)),
            )
            .map(|()| Flow::Serve),
            None => respond_json(stream, "200 OK", success).map(|()| Flow::Serve),
        },
        Err(error) => respond_json(
            stream,
            "500 Internal Server Error",
            &format!("{{\"error\":\"{}\"}}", json_string(&error)),
        )
        .map(|()| Flow::Serve),
    }
}

#[derive(serde::Deserialize)]
struct VoiceRequest {
    label: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    purpose: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    origin: Option<String>,
    #[serde(default)]
    declared_authorisation: bool,
    #[serde(default)]
    encrypt: bool,
}

#[derive(serde::Deserialize)]
struct VoiceTarget {
    id: String,
}

const MIN_SAMPLE_SECONDS: f64 = 10.0;
const MAX_SAMPLE_SECONDS: f64 = 30.0;

fn voice_store() -> Store {
    Store::new(crate::config::data_root().join("voices"))
}

const DEFAULT_VOICE_TEXT: &str = "This is the default voice.";

fn synthesized_wav(pcm16: &[u8], rate: u32) -> Result<Vec<u8>, String> {
    let samples: Vec<i16> = pcm16
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    crate::compare::to_wav(&samples, rate)
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

fn consent_for(
    origin: Option<&str>,
    declared_authorisation: bool,
    purpose: String,
    scope: String,
    accepted_at: u64,
) -> Result<Consent, String> {
    if origin == Some("third-party") {
        if !declared_authorisation {
            return Err(
                "a third party voice profile requires a declared authorisation".to_string(),
            );
        }
        return Ok(Consent::third_party(purpose, scope, accepted_at));
    }
    Ok(Consent::own_voice(purpose, scope, accepted_at))
}

fn meeting_json(meeting: crate::meeting::Meeting) -> serde_json::Value {
    let segments: Vec<serde_json::Value> = meeting
        .segments
        .iter()
        .map(|segment| {
            serde_json::json!({
                "at_ms": segment.at_ms,
                "speaker": segment.speaker,
                "spoken_by": segment.spoken_by(),
                "text": segment.text,
                "named": segment.name.is_some(),
            })
        })
        .collect();
    serde_json::json!({
        "id": meeting.id,
        "title": meeting.title,
        "started_at": meeting.started_at,
        "speakers": meeting.speakers(),
        "speaker_hint": meeting.speaker_hint,
        "segments": segments,
        "actions": crate::actions::for_meeting(&meeting),
        "decisions": crate::decisions::for_meeting(&meeting),
    })
}

pub(super) fn dubbed_name(name: &str, to: &str) -> String {
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "dubbed".to_string());
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_else(|| "wav".to_string());
    format!("{stem}.{to}.{extension}")
}

pub(crate) fn respond_file(stream: &mut dyn Stream, path: &Path, name: &str) -> Result<(), String> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let length = file.metadata().map_err(|error| error.to_string())?.len();
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {length}\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nConnection: close\r\n\r\n",
        content_type(name)
    );
    stream
        .write_all(head.as_bytes())
        .map_err(|error| error.to_string())?;
    std::io::copy(&mut BufReader::new(file), stream).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn wav_seconds(bytes: &[u8]) -> f64 {
    let rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]).max(1);
    let data = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as f64;
    data / 2.0 / f64::from(rate)
}

pub(crate) fn respond_named(
    stream: &mut dyn Stream,
    name: &str,
    body: &[u8],
) -> Result<(), String> {
    let head = format!(
        "HTTP/1.1 200 OK\nContent-Type: {}\nContent-Length: {}\nContent-Disposition: attachment; filename=\"{name}\"\nConnection: close\n\n",
        content_type(name),
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .map_err(|error| error.to_string())?;
    stream.write_all(body).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn transcript_header(transcript: &str) -> String {
    if transcript.is_empty() {
        String::new()
    } else {
        format!(
            "X-Parlo-Transcript: {}\r\nAccess-Control-Expose-Headers: X-Parlo-Transcript\r\n",
            BASE64.encode(transcript)
        )
    }
}

fn respond_audio(stream: &mut dyn Stream, body: &[u8], transcript: &str) -> Result<(), String> {
    let header = transcript_header(transcript);
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: audio/wav\r\nContent-Length: {}\r\n{header}Connection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .and_then(|()| stream.write_all(body))
        .and_then(|()| stream.flush())
        .map_err(|error| error.to_string())
}

fn dictation_target(write: &str, from: &str, to: &str, translating: bool) -> String {
    if !write.is_empty() {
        write.to_string()
    } else if translating && from != to {
        to.to_string()
    } else {
        from.to_string()
    }
}

pub(super) fn pieces_json(pieces: &[parlo_core::engine::StreamPiece]) -> String {
    let rows: Vec<serde_json::Value> = pieces
        .iter()
        .map(|piece| {
            serde_json::json!({
                "state": piece.state,
                "text": piece.text,
                "audio_ms": piece.audio_ms,
                "latency_ms": piece.latency_ms,
            })
        })
        .collect();
    serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string())
}

pub(super) fn parameter(query: &str, name: &str) -> String {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| percent_decode(value))
        .unwrap_or_default()
}

fn stale_retention(days: u64) -> Result<(Vec<serde_json::Value>, Vec<String>), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default();
    let cutoff = crate::retention::cutoff_ms(days, now)?;
    let meetings = crate::meeting::list();
    let stale = crate::retention::stale(&meetings, cutoff);
    let rows = meetings
        .iter()
        .filter(|meeting| stale.contains(&meeting.id))
        .map(|meeting| {
            serde_json::json!({
                "id": meeting.id,
                "title": meeting.title,
                "started_at": meeting.started_at,
            })
        })
        .collect();
    Ok((rows, stale))
}

fn restyle_fields(request: &serde_json::Value) -> Result<(String, String), String> {
    let id = text_field(request, "id");
    let style = text_field(request, "style").to_ascii_lowercase();
    if id.is_empty() {
        return Err("a dictation is named by its id".to_string());
    }
    if !crate::style::is_known(&style) {
        return Err("style needs formal, casual or excited".to_string());
    }
    Ok((id, style))
}

fn text_field(request: &serde_json::Value, name: &str) -> String {
    request
        .get(name)
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn absorb(
    output: parlo_core::PipelineOutput,
    samples: &mut Vec<f32>,
    transcript: &mut String,
) -> Option<String> {
    if let Some(text) = output.transcript {
        if !transcript.is_empty() {
            transcript.push(' ');
        }
        transcript.push_str(&text);
    }
    if let Some(frame) = output.frame {
        samples.extend_from_slice(frame.samples());
    }
    output.error
}

pub(super) fn json_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

fn read_http(stream: &mut dyn Stream) -> Result<(String, Vec<u8>), String> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        let read = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("connection closed before headers".to_string());
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(position) = find(&buffer, b"\r\n\r\n") {
            break position + 4;
        }
        if buffer.len() > MAX_HEAD {
            return Err("headers too large".to_string());
        }
    };

    let head = String::from_utf8_lossy(&buffer[..head_end]).into_owned();
    let length = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    if length > body_limit(&head) {
        return Err("body too large".to_string());
    }

    let mut body = buffer[head_end..].to_vec();
    while body.len() < length {
        let read = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(length);
    Ok((head, body))
}

fn body_limit(head: &str) -> usize {
    let path = head.split_whitespace().nth(1).unwrap_or_default();
    if path.starts_with("/dub/import") {
        MAX_BODY
    } else if path.starts_with("/dub") || path.starts_with("/batch") {
        MAX_DUB_BODY
    } else if path.starts_with("/v1/audio/") {
        MAX_AUDIO_BODY
    } else {
        MAX_BODY
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
#[cfg(test)]
use crate::dub;
#[cfg(test)]
use dub_project::{cue_document, SegmentEdit};
#[cfg(test)]
use parlo_core::wav::{read_pcm16, write_pcm16};
#[cfg(test)]
use parlo_core::{AudioFormat, Wav};
#[cfg(test)]
mod tests;
