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
        "pt" => "pt_PT-tugão-medium.onnx",
        "de" => "de_DE-thorsten-medium.onnx",
        "fr" => "fr_FR-siwis-medium.onnx",
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

const KEPT_TRANSLATORS: usize = 2;
const KEPT_VOICES: usize = 2;

struct Engine {
    recognizer: Option<Recognizer>,
    translators: Vec<(String, Translator)>,
    voices: Vec<(String, Voice)>,
    from: String,
    to: String,
}

impl Engine {
    fn new() -> Self {
        Self {
            recognizer: None,
            translators: Vec::new(),
            voices: Vec::new(),
            from: language("PARLO_FROM", "pt"),
            to: language("PARLO_TO", "en"),
        }
    }

    fn set_pair(&mut self, from: &str, to: &str) -> Result<(), String> {
        let from = from.trim().to_ascii_lowercase();
        let to = to.trim().to_ascii_lowercase();
        let code = |value: &str| {
            value.len() == 2
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphabetic())
        };
        if !code(&from) || !code(&to) {
            return Err("a language pair needs two letter codes".to_string());
        }
        if from == to {
            return Err("the two languages must differ".to_string());
        }
        self.from = from;
        self.to = to;
        Ok(())
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
        let key = format!("{}>{}", self.from, self.to);
        if let Some(index) = self.translators.iter().position(|(name, _)| name == &key) {
            let held = self.translators.remove(index);
            self.translators.push(held);
        } else {
            let folder = pack_dir(&self.from, &self.to).ok_or_else(|| {
                format!(
                    "no translation model for {} to {}; download it first",
                    self.from, self.to
                )
            })?;
            let translator = Translator::open(&folder, threads() as usize)?;
            self.translators.push((key, translator));
            while self.translators.len() > KEPT_TRANSLATORS {
                self.translators.remove(0);
            }
        }
        self.translators
            .last_mut()
            .map(|(_, translator)| translator)
            .ok_or_else(|| "the translator is unavailable".to_string())
    }

    fn voice(&mut self) -> Result<&mut Voice, String> {
        let model = match std::env::var("PARLO_VOICE_PATH") {
            Ok(configured) => PathBuf::from(configured),
            Err(_) => models_root().join(voice_file(&self.to)),
        };
        let key = model.to_string_lossy().into_owned();
        if let Some(index) = self.voices.iter().position(|(name, _)| name == &key) {
            let held = self.voices.remove(index);
            self.voices.push(held);
        } else {
            let voice = Voice::open(&model, threads())?;
            self.voices.push((key, voice));
            while self.voices.len() > KEPT_VOICES {
                self.voices.remove(0);
            }
        }
        self.voices
            .last_mut()
            .map(|(_, voice)| voice)
            .ok_or_else(|| "the voice is unavailable".to_string())
    }

    fn loaded(&self) -> Vec<&'static str> {
        let mut units = Vec::new();
        if self.recognizer.is_some() {
            units.push("recognition");
        }
        if !self.translators.is_empty() {
            units.push("translation");
        }
        if !self.voices.is_empty() {
            units.push("voice");
        }
        units
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
        "pair" => {
            let from = request.get("from").and_then(Value::as_str).unwrap_or("");
            let to = request.get("to").and_then(Value::as_str).unwrap_or("");
            match engine.set_pair(from, to) {
                Ok(()) => protocol::switched(seq, &engine.to),
                Err(message) => protocol::error(Some(seq), &message),
            }
        }
        "loaded" => protocol::loaded(&engine.loaded()),
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
        "speak" => {
            let text = match request.get("text").and_then(Value::as_str) {
                Some(text) if !text.trim().is_empty() => text,
                _ => return protocol::error(Some(seq), "text must not be empty"),
            };
            match engine.voice() {
                Ok(voice) => match voice.speak(text) {
                    Some((spoken, rate)) => protocol::spoken(seq, &spoken, rate),
                    None => protocol::error(Some(seq), "the voice produced no audio"),
                },
                Err(message) => protocol::error(Some(seq), &message),
            }
        }
        "translate_text" => {
            let text = match request.get("text").and_then(Value::as_str) {
                Some(text) if !text.trim().is_empty() => text,
                _ => return protocol::error(Some(seq), "text must not be empty"),
            };
            match engine
                .translator()
                .and_then(|translator| translator.translate(text))
            {
                Ok(translation) => protocol::translation(seq, &translation),
                Err(message) => protocol::error(Some(seq), &message),
            }
        }
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
            if !session.wants_decode() {
                return Ok(protocol::stream(seq, &[]));
            }
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
            let events = session.close(&text, started.elapsed().as_millis() as u64);
            protocol::stream(seq, &events)
        }
        other => protocol::error(Some(seq), &format!("unknown op: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pair_needs_two_language_codes() {
        let mut engine = Engine::new();
        assert!(engine.set_pair("pt", "eng").is_err());
        assert!(engine.set_pair("p1", "en").is_err());
        assert!(engine.set_pair("pt", "pt").is_err());
    }

    #[test]
    fn switching_languages_keeps_the_recogniser_slot_untouched() {
        let mut engine = Engine::new();
        engine.set_pair("pt", "de").unwrap();
        assert_eq!(engine.from, "pt");
        assert_eq!(engine.to, "de");
        assert!(engine.recognizer.is_none());
    }

    #[test]
    fn each_language_speaks_with_its_own_voice() {
        assert_eq!(voice_file("pt"), "pt_PT-tugão-medium.onnx");
        assert_eq!(voice_file("en"), "en_US-lessac-medium.onnx");
        assert_eq!(voice_file("de"), "de_DE-thorsten-medium.onnx");
        assert_eq!(voice_file("fr"), "fr_FR-siwis-medium.onnx");
    }

    #[test]
    fn a_language_without_a_voice_falls_back_to_english() {
        assert_eq!(voice_file("ja"), "en_US-lessac-medium.onnx");
    }

    #[test]
    fn a_switch_drops_the_voices_for_the_old_target() {
        let mut engine = Engine::new();
        engine.set_pair("pt", "en").unwrap();
        assert!(engine.voices.is_empty());
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
