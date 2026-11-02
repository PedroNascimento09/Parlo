mod protocol;
mod speech;
mod stream;
mod translate;
mod voice;

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::Instant;

use serde_json::Value;

use speech::Recognizer;
use stream::Session;
use translate::Translator;
use voice::Voice;

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

fn language(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_string())
}

fn voice_file(language: &str) -> &'static str {
    match language {
        "pt" => "pt_BR-faber-medium.onnx",
        _ => "en_US-lessac-medium.onnx",
    }
}

fn pack_dir(from: &str, to: &str) -> Option<PathBuf> {
    if let Ok(configured) = std::env::var("PARLO_CASCADE_MT_DIR") {
        return Some(PathBuf::from(configured));
    }
    let root = models_root().join("mt");
    let prefix = format!("translate-{from}_{to}");
    let mut matches: Vec<PathBuf> = std::fs::read_dir(&root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name == prefix || name.starts_with(&format!("{prefix}-")))
        })
        .collect();
    matches.sort();
    matches.pop()
}

struct Engine {
    recognizer: Option<Recognizer>,
    translator: Option<Translator>,
    voice: Option<Voice>,
    from: String,
    to: String,
}

impl Engine {
    fn new() -> Self {
        Self {
            recognizer: None,
            translator: None,
            voice: None,
            from: language("PARLO_FROM", "pt"),
            to: language("PARLO_TO", "en"),
        }
    }

    fn recognizer(&mut self) -> Result<&mut Recognizer, String> {
        if self.recognizer.is_none() {
            self.recognizer = Some(Recognizer::open(
                &models_root().join("parakeet"),
                threads(),
            )?);
        }
        self.recognizer
            .as_mut()
            .ok_or_else(|| "the recogniser is unavailable".to_string())
    }

    fn translator(&mut self) -> Result<&mut Translator, String> {
        if self.translator.is_none() {
            let folder = pack_dir(&self.from, &self.to).ok_or_else(|| {
                format!(
                    "no translation model for {} to {}; download it first",
                    self.from, self.to
                )
            })?;
            self.translator = Some(Translator::open(&folder, threads() as usize)?);
        }
        self.translator
            .as_mut()
            .ok_or_else(|| "the translator is unavailable".to_string())
    }

    fn voice(&mut self) -> Result<&mut Voice, String> {
        if self.voice.is_none() {
            let model = match std::env::var("PARLO_VOICE_PATH") {
                Ok(configured) => PathBuf::from(configured),
                Err(_) => models_root().join(voice_file(&self.to)),
            };
            self.voice = Some(Voice::open(&model, threads())?);
        }
        self.voice
            .as_mut()
            .ok_or_else(|| "the voice is unavailable".to_string())
    }
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut engine = Engine::new();
    let mut session = Session::default();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let answer = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle(&request, &mut engine, &mut session),
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

fn handle(request: &Value, engine: &mut Engine, session: &mut Session) -> String {
    let op = request.get("op").and_then(Value::as_str).unwrap_or("");
    let seq = request.get("seq").and_then(Value::as_u64).unwrap_or(0);
    match op {
        "hello" => protocol::hello(&engine.to),
        "dictate" => with_audio(request, seq, |samples| {
            Ok(protocol::dictated(
                seq,
                &engine.recognizer()?.transcribe(samples),
            ))
        }),
        "write" => with_audio(request, seq, |samples| {
            let text = engine.recognizer()?.transcribe(samples);
            let translation = engine.translator()?.translate(&text)?;
            Ok(protocol::written(seq, &text, &translation))
        }),
        "translate" => with_audio(request, seq, |samples| {
            let started = Instant::now();
            let text = engine.recognizer()?.transcribe(samples);
            let translation = engine.translator()?.translate(&text)?;
            let elapsed = |from: &Instant| from.elapsed().as_millis() as u64;
            match engine.voice() {
                Ok(voice) => match voice.speak(&translation) {
                    Some((spoken, rate)) => Ok(protocol::translated(
                        seq,
                        &spoken,
                        rate,
                        &translation,
                        elapsed(&started),
                    )),
                    None => Ok(protocol::translated_text(
                        seq,
                        &translation,
                        elapsed(&started),
                    )),
                },
                Err(_) => Ok(protocol::translated_text(
                    seq,
                    &translation,
                    elapsed(&started),
                )),
            }
        }),
        "stream" => with_audio(request, seq, |samples| {
            session.push(samples);
            let started = Instant::now();
            let text = engine.recognizer()?.transcribe(session.audio());
            let events = session.observe(&text, started.elapsed().as_millis() as u64);
            Ok(protocol::stream(seq, &events))
        }),
        "stream_close" => {
            let started = Instant::now();
            let text = match engine.recognizer() {
                Ok(recognizer) => recognizer.transcribe(session.audio()),
                Err(message) => return protocol::error(Some(seq), &message),
            };
            let events = session.finish(&text, started.elapsed().as_millis() as u64);
            protocol::stream(seq, &events)
        }
        other => protocol::error(Some(seq), &format!("unknown op: {other}")),
    }
}

fn with_audio<F>(request: &Value, seq: u64, run: F) -> String
where
    F: FnOnce(&[f32]) -> Result<String, String>,
{
    let samples = match protocol::pcm16(request) {
        Ok(samples) => samples,
        Err(message) => return protocol::error(Some(seq), &message),
    };
    match run(&samples) {
        Ok(answer) => answer,
        Err(message) => protocol::error(Some(seq), &message),
    }
}
