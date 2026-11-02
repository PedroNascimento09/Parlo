use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use sherpa_onnx::{GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsVitsModelConfig};

pub struct Voice {
    inner: OfflineTts,
}

impl Voice {
    pub fn open(model: &Path, threads: i32) -> Result<Self, String> {
        if !model.is_file() {
            return Err(format!(
                "read aloud needs {}; download the voice first",
                model.display()
            ));
        }
        if !embeds_sample_rate(model)? {
            return Err(format!(
                "{} has no sample rate in its metadata, so the offline speaker cannot load it",
                model.display()
            ));
        }
        let tokens = tokens_for(model)?;
        let mut config = OfflineTtsConfig::default();
        config.model.vits = OfflineTtsVitsModelConfig {
            model: Some(model.to_string_lossy().into_owned()),
            tokens: Some(tokens.to_string_lossy().into_owned()),
            ..OfflineTtsVitsModelConfig::default()
        };
        config.model.num_threads = threads.max(1);
        config.model.provider = Some("cpu".into());
        let inner = OfflineTts::create(&config).ok_or_else(|| {
            format!(
                "the voice in {} could not be loaded; it may be incomplete",
                model.display()
            )
        })?;
        Ok(Self { inner })
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

fn embeds_sample_rate(model: &Path) -> Result<bool, String> {
    let bytes = fs::read(model)
        .map_err(|error| format!("{} could not be read: {error}", model.display()))?;
    Ok(bytes
        .windows(b"sample_rate".len())
        .any(|window| window == b"sample_rate"))
}

pub fn tokens_for(model: &Path) -> Result<PathBuf, String> {
    let tokens = model.with_extension("tokens.txt");
    if tokens.is_file() {
        return Ok(tokens);
    }
    let settings = model.with_extension("onnx.json");
    let text = fs::read_to_string(&settings).map_err(|error| {
        format!(
            "{} could not be read, so the voice cannot be prepared: {error}",
            settings.display()
        )
    })?;
    let payload: Value = serde_json::from_str(&text)
        .map_err(|error| format!("{} is not valid json: {error}", settings.display()))?;
    let phonemes = payload
        .get("phoneme_id_map")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{} has no phoneme map", settings.display()))?;
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
        return Err(format!("{} has an empty phoneme map", settings.display()));
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
    fn a_missing_voice_says_so() {
        let error = Voice::open(Path::new("no-voice.onnx"), 1).err().unwrap();
        assert!(error.contains("download the voice first"));
    }
}
