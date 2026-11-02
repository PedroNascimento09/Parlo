mod protocol;
mod speech;
mod stream;

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::Instant;

use serde_json::Value;

use speech::Recognizer;
use stream::Session;

fn models_root() -> PathBuf {
    std::env::var("PARLO_MODELS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("engines/cascade/models"))
}

fn threads() -> i32 {
    std::env::var("PARLO_CASCADE_THREADS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(4)
}

fn target_language() -> String {
    std::env::var("PARLO_TO").unwrap_or_else(|_| "en".to_string())
}

fn recognizer(slot: &mut Option<Recognizer>) -> Result<&mut Recognizer, String> {
    if slot.is_none() {
        *slot = Some(Recognizer::open(
            &models_root().join("parakeet"),
            threads(),
        )?);
    }
    slot.as_mut()
        .ok_or_else(|| "the recogniser is unavailable".to_string())
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut slot: Option<Recognizer> = None;
    let mut session = Session::default();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let answer = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle(&request, &mut slot, &mut session),
            Err(_) => protocol::error(None, "invalid json"),
        };
        if writeln!(out, "{answer}")
            .and_then(|()| out.flush())
            .is_err()
        {
            break;
        }
    }
}

fn handle(request: &Value, slot: &mut Option<Recognizer>, session: &mut Session) -> String {
    let op = request.get("op").and_then(Value::as_str).unwrap_or("");
    let seq = request.get("seq").and_then(Value::as_u64).unwrap_or(0);
    match op {
        "hello" => protocol::hello(&target_language()),
        "dictate" => {
            let samples = match protocol::pcm16(request) {
                Ok(samples) => samples,
                Err(message) => return protocol::error(Some(seq), &message),
            };
            match recognizer(slot) {
                Ok(engine) => protocol::dictated(seq, &engine.transcribe(&samples)),
                Err(message) => protocol::error(Some(seq), &message),
            }
        }
        "stream" => {
            let samples = match protocol::pcm16(request) {
                Ok(samples) => samples,
                Err(message) => return protocol::error(Some(seq), &message),
            };
            session.push(&samples);
            let started = Instant::now();
            let text = match recognizer(slot) {
                Ok(engine) => engine.transcribe(session.audio()),
                Err(message) => return protocol::error(Some(seq), &message),
            };
            let events = session.observe(&text, started.elapsed().as_millis() as u64);
            protocol::stream(seq, &events)
        }
        "stream_close" => {
            let started = Instant::now();
            let text = match recognizer(slot) {
                Ok(engine) => engine.transcribe(session.audio()),
                Err(message) => return protocol::error(Some(seq), &message),
            };
            let events = session.finish(&text, started.elapsed().as_millis() as u64);
            protocol::stream(seq, &events)
        }
        "translate" | "write" => protocol::error(
            Some(seq),
            "the Rust engine cannot translate yet; use the Python engine for translation",
        ),
        other => protocol::error(Some(seq), &format!("unknown op: {other}")),
    }
}
