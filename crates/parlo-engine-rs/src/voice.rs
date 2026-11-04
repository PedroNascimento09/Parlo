use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sherpa_onnx::{GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsVitsModelConfig};

pub struct Voice {
    inner: OfflineTts,
}

struct Settings {
    tokens: PathBuf,
    sample_rate: u32,
    num_speakers: u32,
    language: String,
    noise_scale: f32,
    noise_scale_w: f32,
    length_scale: f32,
}

impl Voice {
    pub fn open(model: &Path, threads: i32) -> Result<Self, String> {
        if !model.is_file() {
            return Err(format!(
                "read aloud needs {}; download the voice first",
                model.display()
            ));
        }
        let settings = settings_for(model)?;
        let patched = patched_model(model, &settings)?;
        let voice_path = patched.as_deref().unwrap_or(model);
        let data_dir = espeak_data(model).ok_or_else(|| {
            format!(
                "{} needs espeak-ng data; set PARLO_ESPEAK_DATA to the espeak-ng-data folder",
                model.display()
            )
        })?;
        let mut config = OfflineTtsConfig::default();
        config.model.vits = OfflineTtsVitsModelConfig {
            model: Some(voice_path.to_string_lossy().into_owned()),
            tokens: Some(settings.tokens.to_string_lossy().into_owned()),
            data_dir: Some(data_dir.to_string_lossy().into_owned()),
            noise_scale: settings.noise_scale,
            noise_scale_w: settings.noise_scale_w,
            length_scale: settings.length_scale,
            ..OfflineTtsVitsModelConfig::default()
        };
        config.model.num_threads = threads.max(1);
        config.model.provider = Some("cpu".into());
        let inner = OfflineTts::create(&config).ok_or_else(|| {
            format!(
                "the voice in {} could not be loaded; it may be incomplete",
                model.display()
            )
        });
        if let Some(path) = patched {
            fs::remove_file(path).ok();
        }
        Ok(Self { inner: inner? })
    }

    pub fn speak(&self, text: &str) -> Option<(Vec<f32>, i32)> {
        if text.trim().is_empty() {
            return None;
        }
        let audio = self.inner.generate_with_config(
            text,
            &GenerationConfig::default(),
            None::<fn(&[f32], f32) -> bool>,
        )?;
        Some((audio.samples().to_vec(), audio.sample_rate()))
    }
}

fn settings_for(model: &Path) -> Result<Settings, String> {
    let payload = payload_for(model)?;
    let tokens = tokens_for_payload(model, &payload)?;
    let sample_rate = payload
        .pointer("/audio/sample_rate")
        .and_then(Value::as_u64)
        .unwrap_or(22_050) as u32;
    let num_speakers = payload
        .get("num_speakers")
        .and_then(Value::as_u64)
        .unwrap_or(1) as u32;
    let language = payload
        .pointer("/espeak/voice")
        .and_then(Value::as_str)
        .or_else(|| payload.pointer("/language/family").and_then(Value::as_str))
        .unwrap_or("en")
        .to_string();
    let inference = payload.get("inference");
    let number = |key: &str, fallback: f32| {
        inference
            .and_then(|value| value.get(key))
            .and_then(Value::as_f64)
            .map(|value| value as f32)
            .unwrap_or(fallback)
    };
    Ok(Settings {
        tokens,
        sample_rate,
        num_speakers,
        language,
        noise_scale: number("noise_scale", 0.667),
        noise_scale_w: number("noise_w", 0.8),
        length_scale: number("length_scale", 1.0),
    })
}

fn payload_for(model: &Path) -> Result<Value, String> {
    let settings = model.with_extension("onnx.json");
    let text = fs::read_to_string(&settings).map_err(|error| {
        format!(
            "{} could not be read, so the voice cannot be prepared: {error}",
            settings.display()
        )
    })?;
    serde_json::from_str(&text)
        .map_err(|error| format!("{} is not valid json: {error}", settings.display()))
}

fn patched_model(model: &Path, settings: &Settings) -> Result<Option<PathBuf>, String> {
    let bytes = fs::read(model)
        .map_err(|error| format!("{} could not be read: {error}", model.display()))?;
    if bytes
        .windows(b"sample_rate".len())
        .any(|window| window == b"sample_rate")
    {
        return Ok(None);
    }
    let mut patched = bytes;
    append_metadata(
        &mut patched,
        "sample_rate",
        &settings.sample_rate.to_string(),
    );
    append_metadata(
        &mut patched,
        "n_speakers",
        &settings.num_speakers.to_string(),
    );
    append_metadata(&mut patched, "language", &settings.language);
    append_metadata(&mut patched, "voice", &settings.language);
    append_metadata(&mut patched, "comment", "piper");
    let path = std::env::temp_dir().join(format!("parlo-voice-{}.onnx", std::process::id()));
    fs::write(&path, patched)
        .map_err(|error| format!("{} could not be written: {error}", path.display()))?;
    Ok(Some(path))
}

fn append_metadata(out: &mut Vec<u8>, key: &str, value: &str) {
    let mut entry = Vec::new();
    append_field(&mut entry, 1, key.as_bytes());
    append_field(&mut entry, 2, value.as_bytes());
    out.push((14 << 3) | 2);
    append_varint(out, entry.len() as u64);
    out.extend(entry);
}

fn append_field(out: &mut Vec<u8>, field: u8, value: &[u8]) {
    out.push((field << 3) | 2);
    append_varint(out, value.len() as u64);
    out.extend_from_slice(value);
}

fn append_varint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn espeak_data(model: &Path) -> Option<PathBuf> {
    let configured = std::env::var_os("PARLO_ESPEAK_DATA").map(PathBuf::from);
    configured
        .into_iter()
        .chain(model.parent().map(|folder| folder.join("espeak-ng-data")))
        .find(|path| path.is_dir())
}

#[cfg(test)]
pub fn tokens_for(model: &Path) -> Result<PathBuf, String> {
    let payload = payload_for(model)?;
    tokens_for_payload(model, &payload)
}

fn tokens_for_payload(model: &Path, payload: &Value) -> Result<PathBuf, String> {
    let tokens = model.with_extension("tokens.txt");
    if tokens.is_file() {
        return Ok(tokens);
    }
    let phonemes = payload
        .get("phoneme_id_map")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!(
                "{} has no phoneme map",
                model.with_extension("onnx.json").display()
            )
        })?;
    let mut rows: Vec<(i64, String)> = Vec::with_capacity(phonemes.len());
    for (symbol, ids) in phonemes {
        let Some(id) = ids
            .as_array()
            .and_then(|list| list.first())
            .and_then(Value::as_i64)
        else {
            continue;
        };
        rows.push((id, symbol.clone()));
    }
    if rows.is_empty() {
        return Err(format!(
            "{} has an empty phoneme map",
            model.with_extension("onnx.json").display()
        ));
    }
    rows.sort();
    let mut body = String::new();
    for (id, symbol) in rows {
        body.push_str(&format!("{symbol} {id}\n"));
    }
    fs::write(&tokens, body)
        .map_err(|error| format!("{} could not be written: {error}", tokens.display()))?;
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_generated_from_a_phoneme_map() {
        let root = std::env::temp_dir().join(format!("parlo-voice-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let model = root.join("voice.onnx");
        fs::write(&model, b"fake").unwrap();
        fs::write(
            model.with_extension("onnx.json"),
            r#"{"phoneme_id_map":{"_":[0],"^":[1],"a":[7]}}"#,
        )
        .unwrap();
        let tokens = tokens_for(&model).unwrap();
        assert_eq!(fs::read_to_string(&tokens).unwrap(), "_ 0\n^ 1\na 7\n");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn metadata_is_appended_for_a_piper_voice() {
        let mut model = Vec::new();
        append_metadata(&mut model, "sample_rate", "22050");
        assert!(model
            .windows(b"sample_rate".len())
            .any(|w| w == b"sample_rate"));
        assert_eq!(model[0], 0x72);
    }

    #[test]
    fn a_missing_voice_says_so() {
        let error = Voice::open(Path::new("no-voice.onnx"), 1).err().unwrap();
        assert!(error.contains("download the voice first"));
    }
}
