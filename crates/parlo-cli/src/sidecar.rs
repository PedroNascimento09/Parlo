use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use parlo_core::{AudioFormat, Engine, EngineError, EngineOutput, Frame};
use serde::{Deserialize, Serialize};

const DECLARED_LATENCY_MS: u32 = 1_500;

#[derive(Serialize)]
struct Request<'a> {
    op: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pcm16: Option<String>,
}

#[derive(Deserialize)]
struct Response {
    op: String,
    #[serde(default)]
    engine: Option<String>,
    #[serde(default)]
    sample_rate: Option<u32>,
    #[serde(default)]
    target_languages: Vec<String>,
    #[serde(default)]
    pcm16: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

pub struct SidecarEngine {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    id: String,
    languages: Vec<String>,
    sample_rate: u32,
    seq: u64,
}

impl SidecarEngine {
    pub fn spawn(script_dir: &Path) -> Result<Self, EngineError> {
        let python = std::env::var("PARLO_PYTHON").unwrap_or_else(|_| "python".to_string());
        let mut child = Command::new(&python)
            .arg("serve.py")
            .current_dir(script_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| {
                EngineError::Unavailable(format!(
                    "cannot start {python} in {}: {error}",
                    script_dir.display()
                ))
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| EngineError::Unavailable("sidecar stdin is closed".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| EngineError::Unavailable("sidecar stdout is closed".to_string()))?;

        let mut engine = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            id: "sidecar".to_string(),
            languages: Vec::new(),
            sample_rate: 16_000,
            seq: 0,
        };

        let hello = engine.request(&Request {
            op: "hello",
            seq: None,
            pcm16: None,
        })?;
        if hello.op == "error" {
            return Err(EngineError::Unavailable(
                hello
                    .error
                    .unwrap_or_else(|| "sidecar rejected hello".to_string()),
            ));
        }
        engine.id = hello.engine.unwrap_or_else(|| "sidecar".to_string());
        engine.languages = hello.target_languages;
        if let Some(rate) = hello.sample_rate {
            engine.sample_rate = rate;
        }
        Ok(engine)
    }

    fn request(&mut self, request: &Request) -> Result<Response, EngineError> {
        let mut line = serde_json::to_string(request)
            .map_err(|error| EngineError::Rejected(error.to_string()))?;
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .and_then(|()| self.stdin.flush())
            .map_err(|error| EngineError::Unavailable(format!("sidecar write failed: {error}")))?;

        let mut response = String::new();
        self.stdout
            .read_line(&mut response)
            .map_err(|error| EngineError::Unavailable(format!("sidecar read failed: {error}")))?;
        if response.trim().is_empty() {
            return Err(EngineError::Unavailable(
                "sidecar exited without answering".to_string(),
            ));
        }
        serde_json::from_str(&response)
            .map_err(|error| EngineError::Rejected(format!("invalid sidecar response: {error}")))
    }
}

impl Engine for SidecarEngine {
    fn id(&self) -> &str {
        &self.id
    }

    fn target_languages(&self) -> Vec<String> {
        self.languages.clone()
    }

    fn latency_ms(&self) -> u32 {
        DECLARED_LATENCY_MS
    }

    fn process(&mut self, input: &Frame) -> Result<EngineOutput, EngineError> {
        self.seq += 1;
        let response = self.request(&Request {
            op: "translate",
            seq: Some(self.seq),
            pcm16: Some(encode_pcm16(input.samples())),
        })?;

        if response.op == "error" {
            return Err(EngineError::Unavailable(
                response
                    .error
                    .unwrap_or_else(|| "sidecar reported an error".to_string()),
            ));
        }

        let encoded = response
            .pcm16
            .ok_or_else(|| EngineError::Rejected("sidecar returned no audio".to_string()))?;
        let bytes = BASE64
            .decode(encoded)
            .map_err(|error| EngineError::Rejected(format!("invalid base64 audio: {error}")))?;
        let format = AudioFormat::mono(response.sample_rate.unwrap_or(self.sample_rate))
            .map_err(|error| EngineError::Rejected(error.to_string()))?;

        Ok(EngineOutput {
            audio: Some(Frame::new(format, decode_pcm16(&bytes))),
            transcript: response.text,
        })
    }
}

impl Drop for SidecarEngine {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn encode_pcm16(samples: &[f32]) -> String {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    BASE64.encode(bytes)
}

pub fn decode_pcm16(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32_768.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcm_roundtrip_preserves_levels() {
        let encoded = encode_pcm16(&[0.0, 0.5, -0.5]);
        let bytes = BASE64.decode(encoded).unwrap();
        let decoded = decode_pcm16(&bytes);
        assert_eq!(decoded.len(), 3);
        assert!((decoded[1] - 0.5).abs() < 1e-3);
        assert!((decoded[2] + 0.5).abs() < 1e-3);
    }

    #[test]
    fn clamps_out_of_range_samples() {
        let bytes = BASE64.decode(encode_pcm16(&[4.0, -4.0])).unwrap();
        let decoded = decode_pcm16(&bytes);
        assert!(decoded[0] <= 1.0 && decoded[0] > 0.99);
        assert!(decoded[1] >= -1.0 && decoded[1] < -0.99);
    }

    #[test]
    fn hello_request_only_carries_the_op() {
        let line = serde_json::to_string(&Request {
            op: "hello",
            seq: None,
            pcm16: None,
        })
        .unwrap();
        assert_eq!(line, "{\"op\":\"hello\"}");
    }

    #[test]
    fn talks_to_the_python_sidecar_when_present() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../engines/hibiki");
        if !dir.join("serve.py").exists()
            || Command::new("python").arg("--version").output().is_err()
        {
            return;
        }

        let mut engine = SidecarEngine::spawn(&dir).expect("sidecar should start");
        assert_eq!(engine.id(), "hibiki");
        assert_eq!(engine.target_languages(), vec!["en".to_string()]);

        let frame = Frame::new(AudioFormat::mono(16_000).unwrap(), vec![0.1; 320]);
        let error = engine.process(&frame).unwrap_err().to_string();
        assert!(
            error.contains("not wired") || error.contains("torch"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn parses_sidecar_errors() {
        let response: Response =
            serde_json::from_str(r#"{"op":"error","seq":1,"error":"torch is not installed"}"#)
                .unwrap();
        assert_eq!(response.error.as_deref(), Some("torch is not installed"));
        assert!(response.pcm16.is_none());
    }
}
