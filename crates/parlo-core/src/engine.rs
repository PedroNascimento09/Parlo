use std::error::Error;
use std::fmt;

use crate::audio::Frame;

#[derive(Debug, Clone, PartialEq)]
pub struct EngineOutput {
    pub audio: Option<Frame>,
    pub transcript: Option<String>,
}

impl EngineOutput {
    pub fn translated(frame: Frame) -> Self {
        Self {
            audio: Some(frame),
            transcript: None,
        }
    }

    pub fn with_transcript(frame: Frame, transcript: impl Into<String>) -> Self {
        Self {
            audio: Some(frame),
            transcript: Some(transcript.into()),
        }
    }

    pub fn empty() -> Self {
        Self {
            audio: None,
            transcript: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    Unavailable(String),
    Rejected(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(reason) => write!(f, "engine unavailable: {reason}"),
            Self::Rejected(reason) => write!(f, "engine rejected the frame: {reason}"),
        }
    }
}

impl Error for EngineError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitState {
    pub unit: String,
    pub loaded: bool,
    pub idle_seconds: Option<u64>,
    pub policy: u32,
    pub policy_origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamPiece {
    pub state: String,
    pub text: String,
    pub audio_ms: u64,
    pub latency_ms: u64,
}

pub trait Engine: Send {
    fn stream(&mut self, _samples: &[f32]) -> Result<Vec<StreamPiece>, EngineError> {
        Err(EngineError::Rejected(
            "this engine does not stream".to_string(),
        ))
    }

    fn stream_window(
        &mut self,
        samples: &[f32],
        _segment: f32,
    ) -> Result<Vec<StreamPiece>, EngineError> {
        self.stream(samples)
    }

    fn close_stream(&mut self) -> Result<Vec<StreamPiece>, EngineError> {
        Ok(Vec::new())
    }

    fn id(&self) -> &str;

    fn target_languages(&self) -> Vec<String>;

    fn latency_ms(&self) -> u32;

    fn process(&mut self, input: &Frame) -> Result<EngineOutput, EngineError>;

    fn dictate(&mut self, _pcm16: &[u8]) -> Result<String, EngineError> {
        Err(EngineError::Rejected(
            "this engine cannot dictate on its own".to_string(),
        ))
    }

    fn write(
        &mut self,
        _pcm16: &[u8],
        _target: Option<&str>,
        _clean: Option<&str>,
    ) -> Result<(String, String), EngineError> {
        Err(EngineError::Rejected(
            "this engine cannot write in another language".to_string(),
        ))
    }

    fn clean_text(&mut self, _text: &str, _clean: Option<&str>) -> Result<String, EngineError> {
        Err(EngineError::Rejected(
            "this engine cannot clean text".to_string(),
        ))
    }

    fn speak(&mut self, _text: &str) -> Result<(Vec<u8>, u32), EngineError> {
        Err(EngineError::Rejected(
            "this engine cannot speak".to_string(),
        ))
    }

    fn embed(&mut self, _pcm16: &[u8]) -> Result<Vec<f32>, EngineError> {
        Err(EngineError::Rejected(
            "this engine cannot tell voices apart".to_string(),
        ))
    }

    fn finish(&mut self) -> Result<EngineOutput, EngineError> {
        Ok(EngineOutput::empty())
    }

    fn translate_file(&mut self, frame: &Frame) -> Result<EngineOutput, EngineError> {
        translate_frames(self, frame)
    }

    fn loaded_units(&mut self) -> Option<Vec<String>> {
        None
    }

    fn lifecycle(&mut self) -> Option<Vec<UnitState>> {
        None
    }

    fn load_unit(&mut self, unit: &str) -> Result<(), EngineError> {
        Err(EngineError::Rejected(format!(
            "this engine cannot load {unit} on its own"
        )))
    }

    fn unload_unit(&mut self, unit: &str) -> Result<(), EngineError> {
        Err(EngineError::Rejected(format!(
            "this engine cannot unload {unit} on its own"
        )))
    }

    fn set_languages(&mut self, _from: &str, _to: &str) -> Result<(), EngineError> {
        Err(EngineError::Rejected(
            "this engine holds one language pair at a time".to_string(),
        ))
    }

    fn reset(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct EchoEngine {
    id: &'static str,
    latency_ms: u32,
    gain: f32,
}

impl EchoEngine {
    pub fn new(id: &'static str, latency_ms: u32, gain: f32) -> Self {
        Self {
            id,
            latency_ms,
            gain,
        }
    }
}

impl Default for EchoEngine {
    fn default() -> Self {
        Self::new("echo", 1_500, 1.0)
    }
}

pub fn translate_frames<E: Engine + ?Sized>(
    engine: &mut E,
    frame: &Frame,
) -> Result<EngineOutput, EngineError> {
    let step = (frame.format().sample_rate() as usize / 50).max(1);
    let mut audio = Vec::new();
    let mut texts = Vec::new();
    for chunk in frame.samples().chunks(step) {
        let output = engine.process(&Frame::new(frame.format(), chunk.to_vec()))?;
        if let Some(spoken) = output.audio {
            audio.extend_from_slice(spoken.samples());
        }
        if let Some(text) = output.transcript {
            if !text.trim().is_empty() {
                texts.push(text);
            }
        }
    }
    let audio = if audio.is_empty() {
        None
    } else {
        Some(Frame::new(frame.format(), audio))
    };
    let transcript = if texts.is_empty() {
        None
    } else {
        Some(texts.join(" "))
    };
    Ok(EngineOutput { audio, transcript })
}

impl Engine for EchoEngine {
    fn id(&self) -> &str {
        self.id
    }

    fn target_languages(&self) -> Vec<String> {
        vec!["en".to_string()]
    }

    fn latency_ms(&self) -> u32 {
        self.latency_ms
    }

    fn process(&mut self, input: &Frame) -> Result<EngineOutput, EngineError> {
        let samples = input.samples().iter().map(|s| s * self.gain).collect();
        Ok(EngineOutput::translated(Frame::new(
            input.format(),
            samples,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioFormat;

    #[test]
    fn echo_applies_gain_and_keeps_format() {
        let format = AudioFormat::mono(16_000).unwrap();
        let mut engine = EchoEngine::new("echo", 1_200, 0.5);
        let frame = Frame::new(format, vec![1.0, -1.0]);
        let output = engine.process(&frame).unwrap();
        let audio = output.audio.unwrap();
        assert_eq!(audio.format(), format);
        assert_eq!(audio.samples(), &[0.5, -0.5]);
        assert_eq!(engine.latency_ms(), 1_200);
        assert_eq!(engine.target_languages(), vec!["en".to_string()]);
    }

    #[test]
    fn echo_reports_that_it_cannot_dictate() {
        let mut engine = EchoEngine::default();
        let error = engine.dictate(&[0, 0]).unwrap_err();
        assert!(error.to_string().contains("cannot dictate"));
    }

    #[test]
    fn an_engine_that_cannot_switch_says_so() {
        let mut engine = EchoEngine::default();
        let error = engine.set_languages("pt", "en").unwrap_err();
        assert!(error.to_string().contains("one language pair"));
    }

    #[test]
    fn engine_error_reads_as_a_sentence() {
        let error = EngineError::Unavailable("model missing".to_string());
        assert_eq!(error.to_string(), "engine unavailable: model missing");
    }
}
