use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use parlo_core::{AudioFormat, Engine, EngineError, EngineOutput, Frame, UnitState};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    clean: Option<&'a str>,
}

#[derive(Serialize)]
struct UnitRequest<'a> {
    op: &'a str,
    unit: &'a str,
}

#[derive(Serialize)]
struct PairRequest<'a> {
    op: &'a str,
    seq: u64,
    from: &'a str,
    to: &'a str,
}

#[derive(Serialize)]
struct SpeechRequest<'a> {
    op: &'a str,
    seq: u64,
    text: &'a str,
}

#[derive(Deserialize)]
struct WireUnit {
    unit: String,
    state: String,
    #[serde(default)]
    idle_seconds: Option<u64>,
    #[serde(default)]
    policy: u32,
    #[serde(default)]
    policy_origin: String,
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
    #[serde(default)]
    units: Vec<serde_json::Value>,
    #[serde(default)]
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
pub struct StreamEvent {
    pub state: String,
    pub text: String,
    pub audio_ms: u64,
    pub latency_ms: u64,
}

fn pieces(events: Vec<StreamEvent>) -> Vec<parlo_core::engine::StreamPiece> {
    events
        .into_iter()
        .map(|event| parlo_core::engine::StreamPiece {
            state: event.state,
            text: event.text,
            audio_ms: event.audio_ms,
            latency_ms: event.latency_ms,
        })
        .collect()
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
    engine_candidates().into_iter().next()
}

fn engine_name() -> &'static str {
    if cfg!(windows) {
        "parlo-engine.exe"
    } else {
        "parlo-engine"
    }
}

fn rust_candidates() -> Vec<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|root| root.join("parlo-engine-rs").join(engine_name()))
        .filter(|path| path.is_file())
        .into_iter()
        .collect()
}

fn python_candidates() -> Vec<PathBuf> {
    let mut candidates = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|root| {
            vec![
                root.join("parlo-engine").join(engine_name()),
                root.join(engine_name()),
            ]
        })
        .unwrap_or_default();
    candidates.retain(|path| path.is_file());
    candidates
}

fn engine_candidates() -> Vec<PathBuf> {
    if let Ok(configured) = std::env::var("PARLO_ENGINE") {
        match configured.as_str() {
            "rust" => {
                let mut candidates = rust_candidates();
                candidates.extend(python_candidates());
                return candidates;
            }
            "python" => return python_candidates(),
            _ => {
                let path = PathBuf::from(configured);
                if path.is_file() {
                    return vec![path];
                }
            }
        }
    }
    let mut candidates = python_candidates();
    candidates.extend(rust_candidates());
    candidates
}

fn explicit_engine() -> Option<PathBuf> {
    let configured = std::env::var("PARLO_ENGINE").ok()?;
    if configured == "rust" || configured == "python" {
        return None;
    }
    let path = PathBuf::from(configured);
    path.is_file().then_some(path)
}

fn espeak_data_for(engine: &Path) -> Option<PathBuf> {
    let folder = engine.parent()?;
    let mut candidates = vec![folder.join("espeak-ng-data")];
    if let Some(root) = folder.parent() {
        candidates.push(root.join("parlo-engine/_internal/piper/espeak-ng-data"));
    }
    candidates.into_iter().find(|path| path.is_dir())
}

impl SidecarEngine {
    pub fn spawn(script_dir: &Path) -> Result<Self, EngineError> {
        Self::start(script_dir, true)
    }

    pub fn spawn_offline(script_dir: &Path) -> Result<Self, EngineError> {
        Self::start(script_dir, false)
    }

    fn start(script_dir: &Path, warmup: bool) -> Result<Self, EngineError> {
        let models = crate::models::models_root();
        let explicit = explicit_engine();
        let candidates = explicit
            .clone()
            .map(|path| vec![path])
            .unwrap_or_else(engine_candidates);
        let mut failures = Vec::new();
        for engine in candidates {
            match Self::spawn_binary(&engine, &models, warmup) {
                Ok(sidecar) => return Ok(sidecar),
                Err(error) => failures.push(format!("{}: {error}", engine.display())),
            }
        }
        if explicit.is_none() {
            let python = python_for(script_dir);
            match Self::spawn_python(script_dir, &python, &models, warmup) {
                Ok(sidecar) => return Ok(sidecar),
                Err(error) => failures.push(format!("{python}: {error}")),
            }
        }
        Err(EngineError::Unavailable(if failures.is_empty() {
            "no engine was found".to_string()
        } else {
            format!("no engine started: {}", failures.join("; "))
        }))
    }

    fn spawn_binary(engine: &Path, models: &Path, warmup: bool) -> Result<Self, EngineError> {
        let mut command = Command::new(engine);
        command
            .env("PARLO_MODELS_DIR", models)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if !warmup {
            command.env("PARLO_CASCADE_WARMUP", "0");
        }
        if std::env::var_os("PARLO_ESPEAK_DATA").is_none() {
            if let Some(data) = espeak_data_for(engine) {
                command.env("PARLO_ESPEAK_DATA", data);
            }
        }
        let child = command.spawn().map_err(|error| {
            EngineError::Unavailable(format!("cannot start {}: {error}", engine.display()))
        })?;
        Self::handshake(child)
    }

    fn spawn_python(
        script_dir: &Path,
        python: &str,
        models: &Path,
        warmup: bool,
    ) -> Result<Self, EngineError> {
        let mut command = Command::new(python);
        command
            .arg("serve.py")
            .env("PARLO_MODELS_DIR", models)
            .current_dir(script_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if !warmup {
            command.env("PARLO_CASCADE_WARMUP", "0");
        }
        let child = command.spawn().map_err(|error| {
            EngineError::Unavailable(format!(
                "cannot start {python} in {}: {error}",
                script_dir.display()
            ))
        })?;
        Self::handshake(child)
    }

    fn audio_from(&self, response: &Response) -> Result<Option<Frame>, EngineError> {
        let encoded = match response.pcm16.as_deref() {
            Some(encoded) if !encoded.is_empty() => encoded,
            _ => return Ok(None),
        };
        let bytes = BASE64
            .decode(encoded)
            .map_err(|error| EngineError::Rejected(format!("invalid base64 audio: {error}")))?;
        let format = AudioFormat::mono(response.sample_rate.unwrap_or(self.sample_rate))
            .map_err(|error| EngineError::Rejected(error.to_string()))?;
        let samples = decode_pcm16(&bytes);
        if samples.is_empty() {
            return Ok(None);
        }
        Ok(Some(Frame::new(format, samples)))
    }

    fn handshake(mut child: Child) -> Result<Self, EngineError> {
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
            clean: None,
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
            clean: None,
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
        clean: Option<&str>,
    ) -> Result<(String, String), EngineError> {
        self.seq += 1;
        let response = self.request(&Request {
            op: "write",
            seq: Some(self.seq),
            pcm16: Some(BASE64.encode(pcm16)),
            target,
            clean,
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

    pub fn set_pair(&mut self, from: &str, to: &str) -> Result<(), EngineError> {
        self.seq += 1;
        let response = self.request(&PairRequest {
            op: "pair",
            seq: self.seq,
            from,
            to,
        })?;
        if response.op == "error" {
            return Err(EngineError::Unavailable(response.error.unwrap_or_else(
                || "the engine did not change languages".to_string(),
            )));
        }
        if !response.target_languages.is_empty() {
            self.languages = response.target_languages;
        }
        Ok(())
    }

    pub fn speak(&mut self, text: &str) -> Result<(Vec<u8>, u32), EngineError> {
        if text.trim().is_empty() {
            return Err(EngineError::Rejected("text must not be empty".to_string()));
        }
        self.seq += 1;
        let response = self.request(&SpeechRequest {
            op: "speak",
            seq: self.seq,
            text,
        })?;
        if response.op == "error" {
            return Err(EngineError::Unavailable(
                response
                    .error
                    .unwrap_or_else(|| "the engine could not speak".to_string()),
            ));
        }
        let encoded = response
            .pcm16
            .ok_or_else(|| EngineError::Rejected("the engine returned no audio".to_string()))?;
        let bytes = BASE64
            .decode(encoded)
            .map_err(|error| EngineError::Rejected(format!("invalid base64 audio: {error}")))?;
        if bytes.len() % 2 != 0 {
            return Err(EngineError::Rejected(
                "the engine returned an odd number of bytes".to_string(),
            ));
        }
        let rate = response.sample_rate.unwrap_or(self.sample_rate);
        if rate == 0 {
            return Err(EngineError::Rejected(
                "the engine returned no sample rate".to_string(),
            ));
        }
        Ok((bytes, rate))
    }

    pub fn embed(&mut self, pcm16: &[u8]) -> Result<Vec<f32>, EngineError> {
        self.seq += 1;
        let response = self.request(&Request {
            op: "embed",
            seq: Some(self.seq),
            pcm16: Some(BASE64.encode(pcm16)),
            target: None,
            clean: None,
        })?;
        if response.op == "error" {
            return Err(EngineError::Unavailable(
                response
                    .error
                    .unwrap_or_else(|| "sidecar reported an error".to_string()),
            ));
        }
        if response.embedding.is_empty() {
            return Err(EngineError::Rejected(
                "the engine measured no voice in that window".to_string(),
            ));
        }
        Ok(response.embedding)
    }

    pub fn stream_mode(&self) -> &str {
        &self.stream_mode
    }

    pub fn loaded(&mut self) -> Result<Vec<String>, EngineError> {
        let response = self.request(&Request {
            op: "loaded",
            seq: None,
            pcm16: None,
            target: None,
            clean: None,
        })?;
        if response.op == "error" {
            return Err(EngineError::Unavailable(response.error.unwrap_or_else(
                || "the engine did not say what is loaded".to_string(),
            )));
        }
        Ok(response
            .units
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect())
    }

    pub fn lifecycle_report(&mut self) -> Result<Vec<UnitState>, EngineError> {
        let response = self.request(&Request {
            op: "lifecycle",
            seq: None,
            pcm16: None,
            target: None,
            clean: None,
        })?;
        if response.op == "error" {
            return Err(EngineError::Unavailable(response.error.unwrap_or_else(
                || "the engine did not say what it holds".to_string(),
            )));
        }
        response
            .units
            .iter()
            .map(|value| {
                let wire: WireUnit = serde_json::from_value(value.clone())
                    .map_err(|error| EngineError::Rejected(error.to_string()))?;
                Ok(UnitState {
                    unit: wire.unit,
                    loaded: wire.state == "loaded",
                    idle_seconds: wire.idle_seconds,
                    policy: wire.policy,
                    policy_origin: wire.policy_origin,
                })
            })
            .collect()
    }

    fn switch(&mut self, op: &str, unit: &str) -> Result<(), EngineError> {
        let response = self.request(&UnitRequest { op, unit })?;
        if response.op == "error" {
            return Err(EngineError::Unavailable(
                response
                    .error
                    .unwrap_or_else(|| format!("the engine did not {op} {unit}")),
            ));
        }
        Ok(())
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
            clean: None,
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

    fn request<T: Serialize>(&mut self, request: &T) -> Result<Response, EngineError> {
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
        clean: Option<&str>,
    ) -> Result<(String, String), EngineError> {
        SidecarEngine::write(self, pcm16, target, clean)
    }

    fn stream(
        &mut self,
        samples: &[f32],
    ) -> Result<Vec<parlo_core::engine::StreamPiece>, EngineError> {
        SidecarEngine::stream(self, samples).map(pieces)
    }

    fn close_stream(&mut self) -> Result<Vec<parlo_core::engine::StreamPiece>, EngineError> {
        SidecarEngine::close_stream(self).map(pieces)
    }

    fn speak(&mut self, text: &str) -> Result<(Vec<u8>, u32), EngineError> {
        SidecarEngine::speak(self, text)
    }

    fn set_languages(&mut self, from: &str, to: &str) -> Result<(), EngineError> {
        SidecarEngine::set_pair(self, from, to)
    }

    fn embed(&mut self, pcm16: &[u8]) -> Result<Vec<f32>, EngineError> {
        SidecarEngine::embed(self, pcm16)
    }

    fn loaded_units(&mut self) -> Option<Vec<String>> {
        SidecarEngine::loaded(self).ok()
    }

    fn lifecycle(&mut self) -> Option<Vec<UnitState>> {
        SidecarEngine::lifecycle_report(self).ok()
    }

    fn load_unit(&mut self, unit: &str) -> Result<(), EngineError> {
        SidecarEngine::switch(self, "load", unit)
    }

    fn unload_unit(&mut self, unit: &str) -> Result<(), EngineError> {
        SidecarEngine::switch(self, "unload", unit)
    }

    fn finish(&mut self) -> Result<EngineOutput, EngineError> {
        self.seq += 1;
        let response = self.request(&Request {
            op: "flush",
            seq: Some(self.seq),
            pcm16: None,
            target: None,
            clean: None,
        })?;

        if response.op == "error" {
            return Err(EngineError::Unavailable(
                response
                    .error
                    .unwrap_or_else(|| "sidecar reported an error".to_string()),
            ));
        }

        Ok(EngineOutput {
            audio: self.audio_from(&response)?,
            transcript: response.text,
        })
    }

    fn translate_file(&mut self, frame: &Frame) -> Result<EngineOutput, EngineError> {
        self.seq += 1;
        let response = self.request(&Request {
            op: "translate_file",
            seq: Some(self.seq),
            pcm16: Some(encode_pcm16(frame.samples())),
            target: None,
            clean: None,
        })?;

        if response.op == "unsupported" {
            return parlo_core::engine::translate_frames(self, frame);
        }

        if response.op == "error" {
            return Err(EngineError::Unavailable(
                response
                    .error
                    .unwrap_or_else(|| "sidecar reported an error".to_string()),
            ));
        }

        Ok(EngineOutput {
            audio: self.audio_from(&response)?,
            transcript: response.text,
        })
    }

    fn process(&mut self, input: &Frame) -> Result<EngineOutput, EngineError> {
        self.seq += 1;
        let response = self.request(&Request {
            op: "translate",
            seq: Some(self.seq),
            pcm16: Some(encode_pcm16(input.samples())),
            target: None,
            clean: None,
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
            clean: None,
        })
        .unwrap();
        assert_eq!(line, "{\"op\":\"hello\"}");
    }

    #[test]
    fn a_pair_request_names_both_languages() {
        let line = serde_json::to_string(&PairRequest {
            op: "pair",
            seq: 3,
            from: "pt",
            to: "de",
        })
        .unwrap();
        assert_eq!(
            line,
            "{\"op\":\"pair\",\"seq\":3,\"from\":\"pt\",\"to\":\"de\"}"
        );
    }

    #[test]
    fn a_write_request_carries_its_target() {
        let line = serde_json::to_string(&Request {
            op: "write",
            seq: Some(2),
            pcm16: Some("AAA=".to_string()),
            target: Some("en"),
            clean: None,
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
