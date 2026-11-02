use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use serde_json::{json, Value};

use crate::stream::Event;

pub fn pcm16(request: &Value) -> Result<Vec<f32>, String> {
    let payload = request
        .get("pcm16")
        .and_then(Value::as_str)
        .ok_or_else(|| "pcm16 must be base64 text".to_string())?;
    let bytes = BASE64
        .decode(payload)
        .map_err(|_| "pcm16 is not valid base64".to_string())?;
    if bytes.len() % 2 != 0 {
        return Err("pcm16 must hold 16 bit samples".to_string());
    }
    let (pairs, _) = bytes.as_chunks::<2>();
    Ok(pairs
        .iter()
        .map(|pair| i16::from_le_bytes(*pair) as f32 / 32_768.0)
        .collect())
}

pub fn encode(samples: &[f32]) -> String {
    let bytes: Vec<u8> = samples
        .iter()
        .flat_map(|sample| {
            let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            value.to_le_bytes()
        })
        .collect();
    BASE64.encode(bytes)
}

pub fn hello(target: &str) -> String {
    json!({
        "op": "hello",
        "engine": "cascade",
        "available": true,
        "sample_rate": 16_000,
        "target_languages": [target],
    })
    .to_string()
}

pub fn loaded(units: &[&str]) -> String {
    json!({"op": "loaded", "units": units}).to_string()
}

pub fn dictated(seq: u64, text: &str) -> String {
    json!({"op": "dictated", "seq": seq, "text": text}).to_string()
}

pub fn written(seq: u64, text: &str, translation: &str) -> String {
    json!({"op": "written", "seq": seq, "text": text, "translation": translation}).to_string()
}

pub fn translated(seq: u64, samples: &[f32], rate: i32, text: &str, latency_ms: u64) -> String {
    json!({
        "op": "translated",
        "seq": seq,
        "pcm16": encode(samples),
        "sample_rate": rate,
        "text": text,
        "latency_ms": latency_ms,
    })
    .to_string()
}

pub fn translated_text(seq: u64, text: &str, latency_ms: u64) -> String {
    json!({
        "op": "translated",
        "seq": seq,
        "pcm16": "",
        "text": text,
        "latency_ms": latency_ms,
    })
    .to_string()
}

pub fn stream(seq: u64, events: &[Event]) -> String {
    let events: Vec<Value> = events
        .iter()
        .map(|event| {
            json!({
                "state": event.state,
                "text": event.text,
                "audio_ms": event.audio_ms,
                "latency_ms": event.latency_ms,
            })
        })
        .collect();
    json!({"op": "stream", "seq": seq, "mode": "reprocess", "events": events}).to_string()
}

pub fn error(seq: Option<u64>, message: &str) -> String {
    match seq {
        Some(seq) => json!({"op": "error", "seq": seq, "error": message}).to_string(),
        None => json!({"op": "error", "error": message}).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(bytes: &[u8]) -> String {
        BASE64.encode(bytes)
    }

    #[test]
    fn odd_byte_counts_are_refused() {
        let request = json!({"pcm16": encoded(&[1, 2, 3])});
        assert!(pcm16(&request).unwrap_err().contains("16 bit"));
    }

    #[test]
    fn missing_audio_is_refused() {
        let request = json!({"op": "dictate"});
        assert!(pcm16(&request).unwrap_err().contains("base64"));
    }

    #[test]
    fn samples_keep_their_level() {
        let mut bytes = Vec::new();
        for value in [0i16, 16_384, -16_384] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        let request = json!({"pcm16": encoded(&bytes)});
        let samples = pcm16(&request).unwrap();
        assert_eq!(samples.len(), 3);
        assert!((samples[1] - 0.5).abs() < 1e-3);
        assert!((samples[2] + 0.5).abs() < 1e-3);
    }

    #[test]
    fn audio_round_trips_through_base64() {
        let samples = vec![0.25f32, -0.25, 0.0];
        let request = json!({"pcm16": encode(&samples)});
        let decoded = pcm16(&request).unwrap();
        assert!((decoded[0] - 0.25).abs() < 1e-3);
        assert!((decoded[1] + 0.25).abs() < 1e-3);
    }

    #[test]
    fn errors_carry_the_sequence() {
        assert!(error(Some(4), "boom").contains("\"seq\":4"));
        assert!(!error(None, "boom").contains("seq"));
    }

    #[test]
    fn hello_names_the_target_language() {
        let line = hello("en");
        assert!(line.contains("\"engine\":\"cascade\""));
        assert!(line.contains("\"en\""));
    }

    #[test]
    fn loaded_lists_the_units_in_memory() {
        let line = loaded(&["recognition", "voice"]);
        assert!(line.contains("\"op\":\"loaded\""));
        assert!(line.contains("\"recognition\""));
    }
}
