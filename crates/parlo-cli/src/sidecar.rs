use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<&'a str>,
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
    translation: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    events: Vec<StreamEvent>,
    #[serde(default)]
    mode: Option<String>,
}

#[derive(Deserialize)]
pub struct StreamEvent {
    pub state: String,
    pub text: String,
    pub audio_ms: u64,
    pub latency_ms: u64,
}

pub struct SidecarEngine {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    id: String,
    languages: Vec<String>,
    sample_rate: u32,
    seq: u64,
    stream_mode: String,
}

pub fn python_for(script_dir: &Path) -> String {
    if let Ok(explicit) = std::env::var("PARLO_PYTHON") {
        return explicit;
    }
    find_venv_python(script_dir)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "python".to_string())
}

fn find_venv_python(script_dir: &Path) -> Option<PathBuf> {
    let executable = if cfg!(windows) {
        "Scripts/python.exe"
    } else {
        "bin/python"
    };
    script_dir
        .ancestors()
        .take(3)
        .map(|base| base.join(".venv").join(executable))
        .find(|candidate| candidate.is_file())
}

pub fn bundled_engine() -> Option<PathBuf> {
    if let Ok(configured) = std::env::var("PARLO_ENGINE") {
        let path = PathBuf::from(configured);
        if path.is_file() {
            return Some(path);
        }
    }
    let name = if cfg!(windows) {
        "parlo-engine.exe"
    } else {
        "parlo-engine"
    };
    let directory = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let beside = directory.join(name);
    if beside.is_file() {
        return Some(beside);
    }
    let bundled = directory.join("parlo-engine").join(name);
    bundled.is_file().then_some(bundled)
}

impl SidecarEngine {
    pub fn spawn(script_dir: &Path) -> Result<Self, EngineError> {
        let models = crate::models::models_root();
        let mut child = match bundled_engine() {
            Some(engine) => Command::new(&engine)
                .env("PARLO_MODELS_DIR", &models)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|error| {
                    EngineError::Unavailable(format!("cannot start {}: {error}", engine.display()))
                })?,
            None => {
                let python = python_for(script_dir);
                Command::new(&python)
                    .arg("serve.py")
                    .env("PARLO_MODELS_DIR", &models)
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
                    })?
            }
        };

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
            stream_mode: "unknown".to_string(),
        };

        let hello = engine.request(&Request {
            op: "hello",
            seq: None,
            pcm16: None,
            target: None,
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

    pub fn stream(&mut self, samples: &[f32]) -> Result<Vec<StreamEvent>, EngineError> {
        self.seq += 1;
        self.exchange("stream", Some(encode_pcm16(samples)))
    }

    pub fn close_stream(&mut self) -> Result<Vec<StreamEvent>, EngineError> {
        self.seq += 1;
        self.exchange("stream_close", None)
    }

    pub fn dictate(&mut self, pcm16: &[u8]) -> Result<String, EngineError> {
        self.seq += 1;
        let response = self.request(&Request {
            op: "dictate",
            seq: Some(self.seq),
            pcm16: Some(BASE64.encode(pcm16)),
            target: None,
        })?;
        if response.op == "error" {
            return Err(EngineError::Unavailable(
                response
                    .error
                    .unwrap_or_else(|| "sidecar reported an error".to_string()),
            ));
        }
        Ok(response.text.unwrap_or_default())
    }

    pub fn write(
        &mut self,
        pcm16: &[u8],
        target: Option<&str>,
    ) -> Result<(String, String), EngineError> {
        self.seq += 1;
        let response = self.request(&Request {
            op: "write",
            seq: Some(self.seq),
            pcm16: Some(BASE64.encode(pcm16)),
            target,
        })?;
        if response.op == "error" {
            return Err(EngineError::Unavailable(
                response
                    .error
                    .unwrap_or_else(|| "sidecar reported an error".to_string()),
            ));
        }
        let text = response.text.unwrap_or_default();
        let translation = response.translation.unwrap_or_else(|| text.clone());
        Ok((text, translation))
    }

    pub fn stream_mode(&self) -> &str {
        &self.stream_mode
    }

    fn exchange(
        &mut self,
        op: &str,
        pcm16: Option<String>,
    ) -> Result<Vec<StreamEvent>, EngineError> {
        let response = self.request(&Request {
            op,
            seq: Some(self.seq),
            pcm16,
            target: None,
        })?;
        if response.op == "error" {
            return Err(EngineError::Unavailable(
                response
                    .error
                    .unwrap_or_else(|| "sidecar reported an error".to_string()),
            ));
        }
        if let Some(mode) = response.mode {
            self.stream_mode = mode;
        }
        Ok(response.events)
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

    fn dictate(&mut self, pcm16: &[u8]) -> Result<String, EngineError> {
        SidecarEngine::dictate(self, pcm16)
    }

    fn write(
        &mut self,
        pcm16: &[u8],
        target: Option<&str>,
    ) -> Result<(String, String), EngineError> {
        SidecarEngine::write(self, pcm16, target)
    }

    fn process(&mut self, input: &Frame) -> Result<EngineOutput, EngineError> {
        self.seq += 1;
        let response = self.request(&Request {
            op: "translate",
            seq: Some(self.seq),
            pcm16: Some(encode_pcm16(input.samples())),
            target: None,
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
        let samples = decode_pcm16(&bytes);
        let audio = if samples.is_empty() {
            None
        } else {
            Some(Frame::new(format, samples))
        };

        Ok(EngineOutput {
            audio,
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
    fn finds_a_project_venv() {
        let root = std::env::temp_dir().join(format!("parlo-venv-{}", std::process::id()));
        let engine_dir = root.join("engines").join("cascade");
        std::fs::create_dir_all(&engine_dir).unwrap();
        let python = find_venv_python(&engine_dir);
        assert_eq!(python, None);

        let executable = if cfg!(windows) {
            "Scripts/python.exe"
        } else {
            "bin/python"
        };
        let expected = root.join(".venv").join(executable);
        std::fs::create_dir_all(expected.parent().unwrap()).unwrap();
        std::fs::write(&expected, b"").unwrap();
        assert_eq!(find_venv_python(&engine_dir), Some(expected.clone()));
        std::fs::remove_dir_all(&root).ok();
    }

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
            target: None,
        })
        .unwrap();
        assert_eq!(line, "{\"op\":\"hello\"}");
    }

    #[test]
    fn a_write_request_carries_its_target() {
        let line = serde_json::to_string(&Request {
            op: "write",
            seq: Some(2),
            pcm16: Some("AAA=".to_string()),
            target: Some("en"),
        })
        .unwrap();
        assert!(line.contains("\"target\":\"en\""));
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
