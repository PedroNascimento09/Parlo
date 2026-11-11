mod docs;
mod dubbing;
mod http;
mod jobs;
mod library;
mod openai;
mod routes;
mod session_routes;
mod share;
mod websocket;

use super::*;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::dub_exports::share_target;
use super::listener::{listen, listen_tls, tls_server_config};
use super::session::Chosen;
use parlo_core::audio::Frame;
use parlo_core::engine::{Engine, EngineError, EngineOutput, StreamPiece};
use parlo_core::{Pipeline, PipelineConfig};
use std::io::Read;
use std::sync::RwLock;

struct RecordingEngine {
    heard: Arc<Mutex<Vec<u8>>>,
}

impl Engine for RecordingEngine {
    fn id(&self) -> &str {
        "recording"
    }

    fn target_languages(&self) -> Vec<String> {
        Vec::new()
    }

    fn latency_ms(&self) -> u32 {
        1
    }

    fn process(&mut self, _input: &Frame) -> Result<EngineOutput, EngineError> {
        Ok(EngineOutput::empty())
    }

    fn dictate(&mut self, pcm16: &[u8]) -> Result<String, EngineError> {
        self.heard
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend_from_slice(pcm16);
        Ok("ola mundo".to_string())
    }

    fn write(
        &mut self,
        pcm16: &[u8],
        _target: Option<&str>,
        _clean: Option<&str>,
    ) -> Result<(String, String), EngineError> {
        self.heard
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend_from_slice(pcm16);
        Ok(("ola".to_string(), "hello".to_string()))
    }

    fn speak(&mut self, _text: &str) -> Result<(Vec<u8>, u32), EngineError> {
        let pcm: Vec<u8> = (0..1600i16)
            .flat_map(|sample| (sample / 2).to_le_bytes())
            .collect();
        Ok((pcm, 16_000))
    }
}

#[derive(Default)]
struct SpeakingEngine {
    spoken: Arc<Mutex<Vec<String>>>,
}

impl Engine for SpeakingEngine {
    fn id(&self) -> &str {
        "speaking"
    }

    fn target_languages(&self) -> Vec<String> {
        Vec::new()
    }

    fn latency_ms(&self) -> u32 {
        1
    }

    fn process(&mut self, _input: &Frame) -> Result<EngineOutput, EngineError> {
        Ok(EngineOutput::empty())
    }

    fn speak(&mut self, text: &str) -> Result<(Vec<u8>, u32), EngineError> {
        self.spoken
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(text.to_string());
        let pcm: Vec<u8> = (0..1600i16)
            .flat_map(|sample| (sample / 2).to_le_bytes())
            .collect();
        Ok((pcm, 16_000))
    }
}

#[derive(Default)]
struct StreamingEngine {
    calls: usize,
}

impl Engine for StreamingEngine {
    fn id(&self) -> &str {
        "streaming"
    }

    fn target_languages(&self) -> Vec<String> {
        Vec::new()
    }

    fn latency_ms(&self) -> u32 {
        1
    }

    fn process(&mut self, _input: &Frame) -> Result<EngineOutput, EngineError> {
        Ok(EngineOutput::empty())
    }

    fn stream(&mut self, samples: &[f32]) -> Result<Vec<StreamPiece>, EngineError> {
        self.calls += 1;
        let (state, text) = if self.calls == 1 {
            ("partial", "bom")
        } else {
            ("stable", "bom dia")
        };
        Ok(vec![StreamPiece {
            state: state.to_string(),
            text: text.to_string(),
            audio_ms: samples.len() as u64 * 1000 / 16_000,
            latency_ms: 1,
        }])
    }

    fn close_stream(&mut self) -> Result<Vec<StreamPiece>, EngineError> {
        Ok(vec![StreamPiece {
            state: "final".to_string(),
            text: "bom dia.".to_string(),
            audio_ms: 200,
            latency_ms: 1,
        }])
    }
}

impl Session {
    fn with_engine(engine: Box<dyn Engine>, from: &str, to: &str) -> Self {
        let format = AudioFormat::mono(16_000).unwrap();
        let pipeline = Pipeline::new(PipelineConfig::new(format, format), engine).unwrap();
        Self {
            pipeline: Mutex::new(pipeline),
            settings: RwLock::new(Chosen {
                engine: "recording".to_string(),
                stt: "parakeet".to_string(),
                from: from.to_string(),
                to: to.to_string(),
                write: String::new(),
                shortcut: String::new(),
                hands_free: String::new(),
                captions: String::new(),
                command: String::new(),
                copy_shortcut: String::new(),
                undo: String::new(),
            }),
            target: Mutex::new(None),
            busy: AtomicBool::new(false),
            known: Mutex::new(None),
        }
    }
}

fn ask_raw(port: u16, request: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(request).unwrap();
    let mut answer = Vec::new();
    stream.read_to_end(&mut answer).unwrap();
    answer
}

fn ask_bytes(port: u16, request: &[u8]) -> String {
    String::from_utf8(ask_raw(port, request)).unwrap()
}

fn ask(port: u16, request: &str) -> String {
    ask_bytes(port, request.as_bytes())
}

fn start_server_with(session: Session) -> (u16, Arc<AtomicBool>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepting = Arc::new(AtomicBool::new(true));
    let stop = Arc::clone(&accepting);
    let session = Arc::new(session);
    std::thread::spawn(move || listen(listener, "t0ken".into(), port, session, accepting, None));
    (port, stop)
}

fn start_server() -> (u16, Arc<AtomicBool>) {
    start_server_with(
        Session::open("echo".into(), "parakeet".into(), "pt".into(), "en".into()).unwrap(),
    )
}

fn send(port: u16, method: &str, path: &str, body: &str) -> String {
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nx-parlo-token: t0ken\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    ask(port, &request)
}

fn body_json(answer: &str) -> serde_json::Value {
    let body = answer.split("\r\n\r\n").nth(1).unwrap_or("");
    serde_json::from_str(body).unwrap()
}

fn websocket_attempt(port: u16, query: &str) -> Result<TcpStream, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let key = BASE64.encode([0u8; 16]);
    let request = format!(
        "GET /v1/audio/transcriptions/stream{query} HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\nOrigin: http://127.0.0.1:{port}\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = Vec::new();
    let mut byte = [0u8; 1];
    while !response.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        response.push(byte[0]);
    }
    let head = String::from_utf8_lossy(&response).to_string();
    if head.starts_with("HTTP/1.1 101") {
        Ok(stream)
    } else {
        Err(head)
    }
}

fn websocket_client(port: u16) -> TcpStream {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let key = BASE64.encode([0u8; 16]);
    let request = format!(
        "GET /v1/audio/transcriptions/stream?t=t0ken HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\nOrigin: http://127.0.0.1:{port}\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = Vec::new();
    let mut byte = [0u8; 1];
    while !response.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        response.push(byte[0]);
    }
    assert!(
        response.starts_with(b"HTTP/1.1 101"),
        "{}",
        String::from_utf8_lossy(&response)
    );
    stream
}

fn send_websocket_frame(stream: &mut dyn Stream, opcode: u8, payload: &[u8]) {
    let mut frame = vec![0x80 | opcode];
    let length = payload.len();
    if length < 126 {
        frame.push(0x80 | length as u8);
    } else if length <= u16::MAX as usize {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(length as u16).to_be_bytes());
    } else {
        frame.push(0x80 | 127);
        frame.extend_from_slice(&(length as u64).to_be_bytes());
    }
    let mask = [0x12, 0x34, 0x56, 0x78];
    frame.extend_from_slice(&mask);
    frame.extend(
        payload
            .iter()
            .enumerate()
            .map(|(index, byte)| byte ^ mask[index % 4]),
    );
    stream.write_all(&frame).unwrap();
}

fn read_exact_bytes(stream: &mut dyn Stream, length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    stream.read_exact(&mut bytes).unwrap();
    bytes
}

fn read_websocket_frame(stream: &mut dyn Stream) -> (u8, Vec<u8>) {
    let head = read_exact_bytes(stream, 2);
    let opcode = head[0] & 0x0f;
    let masked = head[1] & 0x80 != 0;
    let mut length = (head[1] & 0x7f) as usize;
    if length == 126 {
        let bytes = read_exact_bytes(stream, 2);
        length = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
    } else if length == 127 {
        let bytes = read_exact_bytes(stream, 8);
        length = u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]) as usize;
    }
    let mask = masked.then(|| read_exact_bytes(stream, 4));
    let mut payload = read_exact_bytes(stream, length);
    if let Some(mask) = mask {
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[index % 4];
        }
    }
    (opcode, payload)
}

fn read_websocket_json(stream: &mut dyn Stream) -> serde_json::Value {
    let (opcode, payload) = read_websocket_frame(stream);
    assert_eq!(opcode, 1);
    serde_json::from_slice(&payload).unwrap()
}

const ROUTES: &[(&str, &str, bool)] = &[
    ("GET", "/batch", true),
    ("POST", "/batch", true),
    ("POST", "/batch/{id}/files", true),
    ("POST", "/batch/{id}/start", true),
    ("GET", "/batch/{id}/report", true),
    ("GET", "/stories", true),
    ("POST", "/stories", true),
    ("GET", "/stories/voices", true),
    ("GET", "/v1/models", false),
    ("GET", "/watch", true),
    ("POST", "/watch", true),
    ("POST", "/watch/scan", true),
    ("ANY", "/batch/", true),
    ("ANY", "/dub", true),
    ("ANY", "/dub/", true),
    ("ANY", "/dub/import", true),
    ("ANY", "/dub/project/", true),
    ("ANY", "/models/download/", true),
    ("ANY", "/models/remove/", true),
    ("ANY", "/models/rollback/", true),
    ("ANY", "/stories/", true),
    ("ANY", "/v1/audio/", true),
    ("ANY", "/v1/jobs/", true),
    ("ANY", "/voice/", true),
    ("ANY", "/voice/packs/", true),
    ("ANY", "/watch/", true),
    ("GET", "/call", true),
    ("GET", "/pronounce", false),
    ("GET", "/retention", true),
    ("POST", "/retention", true),
    ("GET", "/meetings", true),
    ("POST", "/meeting/window", true),
    ("POST", "/meeting/summary", true),
    ("POST", "/meetings/clear", true),
    ("ANY", "/meeting/import", false),
    ("ANY", "/meeting/delete", false),
    ("POST", "/meeting/start", true),
    ("POST", "/meeting/segment", true),
    ("POST", "/meeting/ask", true),
    ("POST", "/memory/ask", true),
    ("GET", "/vocabulary", true),
    ("POST", "/vocabulary", true),
    ("POST", "/vocabulary/delete", true),
    ("POST", "/vocabulary/accept", true),
    ("GET", "/vocabulary/export", true),
    ("POST", "/vocabulary/import", true),
    ("POST", "/command", true),
    ("POST", "/command/copy", true),
    ("GET", "/features", false),
    ("POST", "/features", true),
    ("POST", "/style", true),
    ("GET", "/profiles", true),
    ("POST", "/profiles", true),
    ("POST", "/history/restyle", true),
    ("GET", "/hardware", false),
    ("GET", "/catalog", false),
    ("GET", "/history", true),
    ("GET", "/history/export", true),
    ("POST", "/history/clear", true),
    ("ANY", "/history/favourite", false),
    ("ANY", "/history/edit", false),
    ("GET", "/locale", false),
    ("GET", "/settings", true),
    ("POST", "/settings", true),
    ("GET", "/intelligence", false),
    ("POST", "/intelligence", true),
    ("POST", "/intelligence/key", true),
    ("POST", "/intelligence/key/delete", true),
    ("POST", "/intelligence/test", true),
    ("POST", "/dub/import", true),
    ("GET", "/dub/projects", true),
    ("GET", "/voice", false),
    ("POST", "/voice", true),
    ("POST", "/voice/revoke", true),
    ("POST", "/voice/delete", true),
    ("POST", "/voice/sample", true),
    ("POST", "/voice/sample/play", true),
    ("POST", "/voice/export", true),
    ("POST", "/voice/import", true),
    ("POST", "/voice/compare", true),
    ("GET", "/voice/packs", false),
    ("POST", "/voice/packs", true),
    ("POST", "/dub", true),
    ("GET", "/models", false),
    ("POST", "/models", true),
    ("POST", "/models/dictation", true),
    ("POST", "/models/rest", true),
    ("POST", "/models/download", true),
    ("POST", "/models/relocate", true),
    ("POST", "/models/cancel", true),
    ("POST", "/models/policy", true),
    ("GET", "/update/progress", false),
    ("POST", "/update/restart", true),
    ("GET", "/update", false),
    ("POST", "/update", true),
    ("GET", "/", false),
    ("GET", "/app.css", false),
    ("GET", "/island", false),
    ("GET", "/island.html", false),
    ("GET", "/captions", false),
    ("GET", "/captions.html", false),
    ("POST", "/captions/save", true),
    ("GET", "/live", false),
    ("GET", "/live.html", false),
    ("GET", "/app.js", false),
    ("GET", "/routes.js", false),
    ("GET", "/languages.js", false),
    ("GET", "/languages.json", false),
    ("POST", "/ticket", true),
    ("GET", "/v1/audio/transcriptions/stream", false),
    ("GET", "/target", true),
    ("ANY", "/recovery", true),
    ("POST", "/listen/stream", true),
    ("POST", "/listen/stream-end", true),
    ("ANY", "/state", false),
    ("GET", "/engine", false),
    ("ANY", "/health", false),
    ("POST", "/v1/audio/transcriptions", false),
    ("POST", "/v1/audio/translations", false),
    ("POST", "/v1/audio/speech", false),
    ("ANY", "/mcp", true),
    ("POST", "/v1/jobs", true),
    ("GET", "/v1/jobs", true),
    ("POST", "/dictate/begin", true),
    ("POST", "/undo", true),
    ("POST", "/insert", true),
    ("POST", "/warm", true),
    ("POST", "/dictate", true),
];

fn router_source() -> String {
    let routes = concat!(env!("CARGO_MANIFEST_DIR"), "/src/serve/routes");
    let mut source = include_str!("../mod.rs").to_string();
    let mut files: Vec<_> = std::fs::read_dir(routes)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    files.sort();
    for file in files {
        source.push_str(&std::fs::read_to_string(file).unwrap());
    }
    source
}

fn routed_paths() -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    for line in router_source().lines() {
        if !line.contains("path ") || !line.contains("if ") {
            continue;
        }
        let method = line
            .split_once("method == \"")
            .map(|(_, rest)| rest.split('"').next().unwrap_or("").to_string())
            .unwrap_or_else(|| "ANY".to_string());
        for marker in ["path == \"", "path.starts_with(\""] {
            let mut rest = line;
            while let Some(at) = rest.find(marker) {
                rest = &rest[at + marker.len()..];
                let Some(end) = rest.find('"') else {
                    break;
                };
                let path = rest[..end].to_string();
                let known = found.iter().any(|(kept_method, kept_path)| {
                    kept_path == &path
                        && (kept_method == &method || kept_method == "ANY" || method == "ANY")
                });
                if !known {
                    found.push((method.clone(), path));
                }
            }
        }
    }
    found
}

fn probe_path(path: &str) -> String {
    if path.ends_with('/') {
        format!("{path}x")
    } else {
        path.to_string()
    }
}

fn ask_with(port: u16, method: &str, path: &str, headers: &str, body: &str) -> String {
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nx-parlo-token: t0ken\r\n{headers}Content-Length: {}\r\n\r\n{body}",
        body.len()
    );
    ask(port, &request)
}

fn multipart(boundary: &str, model: &str, audio: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\n{model}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"sample.wav\"\r\nContent-Type: audio/wav\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(audio);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

fn post_multipart(port: u16, path: &str, body: &[u8], boundary: &str) -> String {
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer t0ken\r\nContent-Type: multipart/form-data; boundary={boundary}\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    let mut request = head.into_bytes();
    request.extend_from_slice(body);
    ask_bytes(port, &request)
}

fn post_transcription(port: u16, body: &[u8], boundary: &str) -> String {
    post_multipart(port, "/v1/audio/transcriptions", body, boundary)
}
