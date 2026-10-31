use std::error::Error;
use std::fmt;
use std::num::{NonZeroU16, NonZeroU32};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    ZeroSampleRate,
    ZeroChannels,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroSampleRate => f.write_str("sample rate must be greater than zero"),
            Self::ZeroChannels => f.write_str("channel count must be greater than zero"),
        }
    }
}

impl Error for FormatError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioFormat {
    sample_rate: NonZeroU32,
    channels: NonZeroU16,
}

impl AudioFormat {
    pub fn new(sample_rate: u32, channels: u16) -> Result<Self, FormatError> {
        Ok(Self {
            sample_rate: NonZeroU32::new(sample_rate).ok_or(FormatError::ZeroSampleRate)?,
            channels: NonZeroU16::new(channels).ok_or(FormatError::ZeroChannels)?,
        })
    }

    pub fn mono(sample_rate: u32) -> Result<Self, FormatError> {
        Self::new(sample_rate, 1)
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate.get()
    }

    pub fn channels(&self) -> u16 {
        self.channels.get()
    }

    pub fn frame_samples(&self, frame_ms: u32) -> usize {
        let per_channel = self.sample_rate() as u64 * frame_ms as u64 / 1000;
        per_channel as usize * self.channels() as usize
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    format: AudioFormat,
    samples: Vec<f32>,
}

impl Frame {
    pub fn new(format: AudioFormat, samples: Vec<f32>) -> Self {
        Self { format, samples }
    }

    pub fn silence(format: AudioFormat, samples: usize) -> Self {
        Self::new(format, vec![0.0; samples])
    }

    pub fn format(&self) -> AudioFormat {
        self.format
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    pub fn samples_mut(&mut self) -> &mut [f32] {
        &mut self.samples
    }

    pub fn into_samples(self) -> Vec<f32> {
        self.samples
    }

    pub fn duration_ms(&self) -> f32 {
        let per_channel = self.samples.len() as f32 / self.format.channels() as f32;
        per_channel / self.format.sample_rate() as f32 * 1000.0
    }

    pub fn rms(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.samples.iter().map(|s| s * s).sum();
        (sum / self.samples.len() as f32).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_values() {
        assert_eq!(AudioFormat::new(0, 1), Err(FormatError::ZeroSampleRate));
        assert_eq!(AudioFormat::new(16_000, 0), Err(FormatError::ZeroChannels));
    }

    #[test]
    fn computes_frame_samples_for_stereo() {
        let format = AudioFormat::new(48_000, 2).unwrap();
        assert_eq!(format.frame_samples(10), 960);
    }

    #[test]
    fn reports_duration_and_level() {
        let format = AudioFormat::mono(16_000).unwrap();
        let frame = Frame::new(format, vec![0.5; 160]);
        assert!((frame.duration_ms() - 10.0).abs() < 1e-3);
        assert!((frame.rms() - 0.5).abs() < 1e-6);
    }
}
