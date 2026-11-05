use std::path::PathBuf;

use crate::registry::{model, Source};

pub const MAX_TEXT: usize = 4096;

#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub id: String,
    pub label: String,
    pub language: String,
    pub file: PathBuf,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Reading {
    pub voice: String,
    pub label: String,
    pub language: String,
    pub start_ms: u64,
    pub duration_ms: u64,
}

pub fn plan<F>(text: &str, voices: &[String], revoked: F) -> Result<Vec<Spec>, String>
where
    F: Fn(&str) -> bool,
{
    if text.trim().is_empty() {
        return Err("the sentence to compare is empty".to_string());
    }
    if text.chars().count() > MAX_TEXT {
        return Err(format!("the sentence is longer than {MAX_TEXT} characters"));
    }
    if voices.len() != 2 {
        return Err("a comparison reads one sentence with exactly two voices".to_string());
    }
    let mut specs = Vec::new();
    for voice in voices {
        let id = voice.trim();
        if revoked(id) {
            return Err(format!("{id} was revoked, so it cannot be compared"));
        }
        let Some(spec) = model(id) else {
            return Err(format!("{id} is not a voice this app can read aloud with"));
        };
        if spec.kind != "voice" {
            return Err(format!("{id} is not a voice this app can read aloud with"));
        }
        let file = match spec.source {
            Source::Voice(file) => crate::models::models_root().join(file),
            _ => return Err(format!("{id} has no voice file to read with")),
        };
        specs.push(Spec {
            id: id.to_string(),
            label: spec.name.to_string(),
            language: spec.languages.first().copied().unwrap_or("pt").to_string(),
            file,
        });
    }
    if specs[0].id == specs[1].id {
        return Err("a comparison needs two different voices".to_string());
    }
    Ok(specs)
}

pub fn render<S>(
    specs: &[Spec],
    text: &str,
    mut speak: S,
) -> Result<(Vec<Reading>, Vec<i16>, u32), String>
where
    S: FnMut(&Spec, &str) -> Result<(Vec<u8>, u32), String>,
{
    let mut readings = Vec::new();
    let mut samples: Vec<i16> = Vec::new();
    let mut rate = 0;
    let mut start_ms = 0;
    for spec in specs {
        let (pcm, sample_rate) =
            speak(spec, text).map_err(|error| format!("{}: {error}", spec.id))?;
        if pcm.is_empty() {
            return Err(format!("{} produced no audio", spec.id));
        }
        if rate == 0 {
            rate = sample_rate;
        }
        if sample_rate != rate {
            return Err(format!(
                "{} was read at {sample_rate} Hz while the first voice used {rate} Hz",
                spec.id
            ));
        }
        let frames = pcm.chunks_exact(2).count();
        let duration_ms = frames as u64 * 1000 / u64::from(rate.max(1));
        readings.push(Reading {
            voice: spec.id.clone(),
            label: spec.label.clone(),
            language: spec.language.clone(),
            start_ms,
            duration_ms,
        });
        samples.extend(
            pcm.chunks_exact(2)
                .map(|pair| i16::from_le_bytes([pair[0], pair[1]])),
        );
        start_ms += duration_ms;
    }
    Ok((readings, samples, rate))
}

pub fn revoked_in_store(id: &str) -> bool {
    let store = parlo_core::voice::Store::new(crate::config::data_root().join("voices"));
    store
        .get(id)
        .map(|profile| !profile.usable())
        .unwrap_or(false)
}

pub fn to_wav(samples: &[i16], rate: u32) -> Result<Vec<u8>, String> {
    let format = parlo_core::AudioFormat::mono(rate).map_err(|error| error.to_string())?;
    Ok(parlo_core::wav::write_pcm16(&parlo_core::Wav {
        format,
        samples: samples
            .iter()
            .map(|sample| *sample as f32 / 32_768.0)
            .collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm(frames: i16) -> Vec<u8> {
        (0..frames).flat_map(|value| value.to_le_bytes()).collect()
    }

    #[test]
    fn the_same_sentence_comes_back_in_the_order_it_was_asked_for() {
        let voices = vec!["voice-pt".to_string(), "voice-en".to_string()];
        let specs = plan("Bom dia, tudo bem?", &voices, |_| false).unwrap();
        assert_eq!(specs[0].id, "voice-pt");
        assert_eq!(specs[1].id, "voice-en");

        let mut heard = Vec::new();
        let (readings, samples, rate) = render(&specs, "Bom dia, tudo bem?", |spec, text| {
            assert_eq!(text, "Bom dia, tudo bem?");
            heard.push(spec.id.clone());
            if spec.id == "voice-pt" {
                Ok((pcm(160), 16_000))
            } else {
                Ok((pcm(320), 16_000))
            }
        })
        .unwrap();

        assert_eq!(heard, ["voice-pt", "voice-en"]);
        assert_eq!(rate, 16_000);
        assert_eq!(readings.len(), 2);
        assert_eq!(readings[0].voice, "voice-pt");
        assert_eq!(readings[0].label, "Portuguese voice");
        assert_eq!(readings[0].language, "pt");
        assert_eq!(readings[0].start_ms, 0);
        assert_eq!(readings[0].duration_ms, 10);
        assert_eq!(readings[1].voice, "voice-en");
        assert_eq!(readings[1].label, "English voice");
        assert_eq!(readings[1].language, "en");
        assert_eq!(readings[1].start_ms, 10);
        assert_eq!(readings[1].duration_ms, 20);
        assert_eq!(samples.len(), 480);
    }

    #[test]
    fn the_comparison_is_one_wav_a_player_can_open() {
        let voices = vec!["voice-pt".to_string(), "voice-de".to_string()];
        let specs = plan("Bom dia", &voices, |_| false).unwrap();
        let (_, samples, rate) = render(&specs, "Bom dia", |_, _| Ok((pcm(80), 16_000))).unwrap();
        let wav = to_wav(&samples, rate).unwrap();
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(wav.len(), 44 + 160 * 2);
    }

    #[test]
    fn a_voice_that_does_not_exist_is_refused_before_any_audio() {
        let voices = vec!["voice-pt".to_string(), "parakeet".to_string()];
        let refused = plan("Bom dia", &voices, |_| false).unwrap_err();
        assert!(refused.contains("parakeet"), "{refused}");
        let unknown = plan(
            "Bom dia",
            &["voice-xx".to_string(), "voice-en".into()],
            |_| false,
        )
        .unwrap_err();
        assert!(unknown.contains("voice-xx"), "{unknown}");
    }

    #[test]
    fn a_revoked_voice_is_refused_by_name() {
        let voices = vec!["voice-pt".to_string(), "voice-en".to_string()];
        let refused = plan("Bom dia", &voices, |id| id == "voice-en").unwrap_err();
        assert!(refused.contains("voice-en"), "{refused}");
        assert!(refused.contains("revoked"), "{refused}");
    }

    #[test]
    fn two_reads_of_the_same_voice_are_refused() {
        let voices = vec!["voice-pt".to_string(), "voice-pt".to_string()];
        let refused = plan("Bom dia", &voices, |_| false).unwrap_err();
        assert!(refused.contains("two different voices"), "{refused}");
        let one = vec!["voice-pt".to_string()];
        assert!(plan("Bom dia", &one, |_| false).is_err());
        assert!(plan("   ", &voices, |_| false).is_err());
    }

    #[test]
    fn a_voice_that_fails_leaves_no_partial_comparison() {
        let voices = vec!["voice-pt".to_string(), "voice-en".to_string()];
        let specs = plan("Bom dia", &voices, |_| false).unwrap();
        let refused = render(&specs, "Bom dia", |spec, _| {
            if spec.id == "voice-en" {
                Err("the voice is not installed".to_string())
            } else {
                Ok((pcm(160), 16_000))
            }
        })
        .unwrap_err();
        assert!(refused.contains("voice-en"), "{refused}");
        assert!(refused.contains("not installed"), "{refused}");
    }
}
