use std::error::Error;
use std::fmt;

use crate::audio::{AudioFormat, Frame};
use crate::engine::{Engine, EngineError};
use crate::gain::AutoGain;
use crate::latency::LatencyBudget;
use crate::resample::Resampler;
use crate::vad::VoiceGate;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PipelineConfig {
    pub input: AudioFormat,
    pub output: AudioFormat,
    pub frame_ms: u32,
    pub gate_threshold_db: f32,
    pub hangover_frames: u32,
}

impl PipelineConfig {
    pub fn new(input: AudioFormat, output: AudioFormat) -> Self {
        Self {
            input,
            output,
            frame_ms: 20,
            gate_threshold_db: -55.0,
            hangover_frames: 15,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    UnsupportedChannels { input: u16, output: u16 },
    InvalidFrameMs(u32),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedChannels { input, output } => write!(
                f,
                "mono audio required, got {input} input and {output} output channels"
            ),
            Self::InvalidFrameMs(frame_ms) => {
                write!(f, "frame size must be between 1 and 200 ms, got {frame_ms}")
            }
        }
    }
}

impl Error for PipelineError {}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineOutput {
    pub voice: bool,
    pub frame: Option<Frame>,
    pub transcript: Option<String>,
    pub latency_ms: u32,
    pub error: Option<String>,
}

pub struct Pipeline {
    config: PipelineConfig,
    resampler: Resampler,
    gate: VoiceGate,
    gain: AutoGain,
    engine: Box<dyn Engine>,
    budget: LatencyBudget,
    pending: Vec<f32>,
    frame_samples: usize,
}

impl Pipeline {
    pub fn new(config: PipelineConfig, engine: Box<dyn Engine>) -> Result<Self, PipelineError> {
        if config.input.channels() != 1 || config.output.channels() != 1 {
            return Err(PipelineError::UnsupportedChannels {
                input: config.input.channels(),
                output: config.output.channels(),
            });
        }
        if config.frame_ms == 0 || config.frame_ms > 200 {
            return Err(PipelineError::InvalidFrameMs(config.frame_ms));
        }

        let budget = LatencyBudget::new(config.frame_ms, engine.latency_ms(), 5);
        Ok(Self {
            resampler: Resampler::new(config.input.sample_rate(), config.output.sample_rate()),
            gate: VoiceGate::new(config.gate_threshold_db, config.hangover_frames),
            gain: AutoGain::new(),
            frame_samples: config.input.frame_samples(config.frame_ms),
            engine,
            budget,
            config,
            pending: Vec::new(),
        })
    }

    pub fn config(&self) -> PipelineConfig {
        self.config
    }

    pub fn budget(&self) -> LatencyBudget {
        self.budget
    }

    pub fn engine_id(&self) -> &str {
        self.engine.id()
    }

    pub fn loaded_units(&mut self) -> Option<Vec<String>> {
        self.engine.loaded_units()
    }

    pub fn lifecycle(&mut self) -> Option<Vec<crate::engine::UnitState>> {
        self.engine.lifecycle()
    }

    pub fn load_unit(&mut self, unit: &str) -> Result<(), EngineError> {
        self.engine.load_unit(unit)
    }

    pub fn unload_unit(&mut self, unit: &str) -> Result<(), EngineError> {
        self.engine.unload_unit(unit)
    }

    pub fn input_gain(&self) -> f32 {
        self.gain.gain()
    }

    pub fn dictate(&mut self, pcm16: &[u8]) -> Result<String, EngineError> {
        self.engine.dictate(pcm16)
    }

    pub fn write(
        &mut self,
        pcm16: &[u8],
        target: Option<&str>,
    ) -> Result<(String, String), EngineError> {
        self.engine.write(pcm16, target)
    }

    pub fn push(&mut self, samples: &[f32]) -> Vec<PipelineOutput> {
        self.pending.extend_from_slice(samples);
        let mut outputs = Vec::new();
        while self.pending.len() >= self.frame_samples {
            let frame: Vec<f32> = self.pending.drain(..self.frame_samples).collect();
            outputs.push(self.process_frame(frame));
        }
        outputs
    }

    fn process_frame(&mut self, mut samples: Vec<f32>) -> PipelineOutput {
        let voice = self.gate.process(&samples);
        if !voice {
            return PipelineOutput {
                voice,
                frame: None,
                transcript: None,
                latency_ms: self.budget.total_ms(),
                error: None,
            };
        }

        self.gain.process(&mut samples);
        let mut resampled = Vec::with_capacity(self.frame_samples + 2);
        self.resampler.process(&samples, &mut resampled);
        let frame = Frame::new(self.config.output, resampled);

        match self.engine.process(&frame) {
            Ok(output) => PipelineOutput {
                voice,
                frame: output.audio,
                transcript: output.transcript,
                latency_ms: self.budget.total_ms(),
                error: None,
            },
            Err(error) => PipelineOutput {
                voice,
                frame: None,
                transcript: None,
                latency_ms: self.budget.total_ms(),
                error: Some(error.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{EchoEngine, EngineOutput};

    struct DictatingEngine;

    impl Engine for DictatingEngine {
        fn id(&self) -> &str {
            "dictating"
        }

        fn target_languages(&self) -> Vec<String> {
            Vec::new()
        }

        fn latency_ms(&self) -> u32 {
            10
        }

        fn process(&mut self, input: &Frame) -> Result<EngineOutput, EngineError> {
            Ok(EngineOutput::translated(Frame::new(
                input.format(),
                Vec::new(),
            )))
        }

        fn dictate(&mut self, pcm16: &[u8]) -> Result<String, EngineError> {
            Ok(format!("{} bytes of speech", pcm16.len()))
        }
    }

    #[test]
    fn dictation_reaches_the_engine() {
        let input = AudioFormat::mono(48_000).unwrap();
        let output = AudioFormat::mono(16_000).unwrap();
        let mut pipeline = Pipeline::new(
            PipelineConfig::new(input, output),
            Box::new(DictatingEngine),
        )
        .unwrap();
        assert_eq!(
            pipeline.dictate(&[0, 1, 2, 3]).unwrap(),
            "4 bytes of speech"
        );
    }

    #[test]
    fn an_engine_without_dictation_says_so() {
        let mut pipeline = pipeline();
        let error = pipeline.dictate(&[0, 1]).unwrap_err();
        assert!(error.to_string().contains("cannot dictate"));
    }

    fn pipeline() -> Pipeline {
        let input = AudioFormat::mono(48_000).unwrap();
        let output = AudioFormat::mono(16_000).unwrap();
        Pipeline::new(
            PipelineConfig::new(input, output),
            Box::new(EchoEngine::new("echo", 1_500, 1.0)),
        )
        .unwrap()
    }

    #[test]
    fn emits_one_frame_per_twenty_milliseconds() {
        let mut pipeline = pipeline();
        let outputs = pipeline.push(&vec![0.5; 4_800]);
        assert_eq!(outputs.len(), 5);
        assert!(outputs.iter().all(|o| o.voice));
        assert_eq!(outputs[0].frame.as_ref().unwrap().samples().len(), 320);
        assert_eq!(outputs[0].latency_ms, 1_525);
    }

    #[test]
    fn opens_the_gate_for_a_quiet_microphone() {
        let mut pipeline = pipeline();
        let outputs = pipeline.push(&vec![0.004; 4_800]);
        assert!(outputs.iter().all(|output| output.voice));
    }

    #[test]
    fn silences_silent_audio() {
        let mut pipeline = pipeline();
        let outputs = pipeline.push(&vec![0.0; 960]);
        assert_eq!(outputs.len(), 1);
        assert!(!outputs[0].voice);
        assert!(outputs[0].frame.is_none());
    }

    #[test]
    fn rejects_stereo_input() {
        let input = AudioFormat::new(48_000, 2).unwrap();
        let output = AudioFormat::mono(16_000).unwrap();
        let result = Pipeline::new(
            PipelineConfig::new(input, output),
            Box::new(EchoEngine::default()),
        );
        assert!(matches!(
            result,
            Err(PipelineError::UnsupportedChannels { .. })
        ));
    }

    #[test]
    fn rejects_invalid_frame_size() {
        let input = AudioFormat::mono(48_000).unwrap();
        let output = AudioFormat::mono(16_000).unwrap();
        let mut config = PipelineConfig::new(input, output);
        config.frame_ms = 0;
        let result = Pipeline::new(config, Box::new(EchoEngine::default()));
        assert!(matches!(result, Err(PipelineError::InvalidFrameMs(0))));
    }
}
