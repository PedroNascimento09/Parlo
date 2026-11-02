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

pub trait Engine: Send {
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
    ) -> Result<(String, String), EngineError> {
        Err(EngineError::Rejected(
            "this engine cannot write in another language".to_string(),
        ))
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
    fn engine_error_reads_as_a_sentence() {
        let error = EngineError::Unavailable("model missing".to_string());
        assert_eq!(error.to_string(), "engine unavailable: model missing");
    }
}
