use std::error::Error;
use std::fmt;

use crate::audio::AudioFormat;

#[derive(Debug, Clone, PartialEq)]
pub struct Wav {
    pub format: AudioFormat,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WavError {
    NotWave,
    Truncated,
    Unsupported { audio_format: u16, bits: u16 },
}

impl fmt::Display for WavError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotWave => f.write_str("not a RIFF/WAVE file"),
            Self::Truncated => f.write_str("truncated or malformed wave data"),
            Self::Unsupported { audio_format, bits } => {
                write!(
                    f,
                    "unsupported wave encoding: format {audio_format}, {bits} bits"
                )
            }
        }
    }
}

impl Error for WavError {}

pub fn read_pcm16(bytes: &[u8]) -> Result<Wav, WavError> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(WavError::NotWave);
    }

    let mut cursor = 12;
    let mut format = None;
    let mut data: Option<&[u8]> = None;

    while cursor + 8 <= bytes.len() {
        let id = &bytes[cursor..cursor + 4];
        let size = u32::from_le_bytes([
            bytes[cursor + 4],
            bytes[cursor + 5],
            bytes[cursor + 6],
            bytes[cursor + 7],
        ]) as usize;
        let start = cursor + 8;
        let end = start.checked_add(size).ok_or(WavError::Truncated)?;
        if end > bytes.len() {
            return Err(WavError::Truncated);
        }
        let body = &bytes[start..end];
        match id {
            b"fmt " => format = Some(parse_fmt(body)?),
            b"data" => data = Some(body),
            _ => {}
        }
        cursor = end + (size & 1);
    }

    let format = format.ok_or(WavError::Truncated)?;
    let data = data.ok_or(WavError::Truncated)?;
    if data.len() % 2 != 0 {
        return Err(WavError::Truncated);
    }

    let samples = data
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32_768.0)
        .collect();

    Ok(Wav { format, samples })
}

fn parse_fmt(body: &[u8]) -> Result<AudioFormat, WavError> {
    if body.len() < 16 {
        return Err(WavError::Truncated);
    }
    let audio_format = u16::from_le_bytes([body[0], body[1]]);
    let channels = u16::from_le_bytes([body[2], body[3]]);
    let sample_rate = u32::from_le_bytes([body[4], body[5], body[6], body[7]]);
    let bits = u16::from_le_bytes([body[14], body[15]]);
    if audio_format != 1 || bits != 16 {
        return Err(WavError::Unsupported { audio_format, bits });
    }
    AudioFormat::new(sample_rate, channels)
        .map_err(|_| WavError::Unsupported { audio_format, bits })
}

pub fn write_pcm16(wav: &Wav) -> Vec<u8> {
    let channels = wav.format.channels();
    let sample_rate = wav.format.sample_rate();
    let data_len = (wav.samples.len() * 2) as u32;

    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * channels as u32 * 2).to_le_bytes());
    out.extend_from_slice(&(channels * 2).to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for sample in &wav.samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_mono_audio() {
        let format = AudioFormat::mono(16_000).unwrap();
        let wav = Wav {
            format,
            samples: vec![0.0, 0.5, -0.5],
        };
        let decoded = read_pcm16(&write_pcm16(&wav)).unwrap();
        assert_eq!(decoded.format, format);
        assert_eq!(decoded.samples.len(), 3);
        assert!((decoded.samples[1] - 0.5).abs() < 1e-3);
    }

    #[test]
    fn rejects_non_wave_bytes() {
        assert_eq!(read_pcm16(b"nope"), Err(WavError::NotWave));
    }

    #[test]
    fn rejects_unsupported_bit_depth() {
        let mut bytes = write_pcm16(&Wav {
            format: AudioFormat::mono(16_000).unwrap(),
            samples: vec![0.0],
        });
        bytes[34] = 8;
        assert_eq!(
            read_pcm16(&bytes),
            Err(WavError::Unsupported {
                audio_format: 1,
                bits: 8
            })
        );
    }
}
