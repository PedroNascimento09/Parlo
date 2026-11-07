use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use parlo_core::gain::peak;
use parlo_core::pronounce;
use parlo_core::wav::write_pcm16;
use parlo_core::Wav;

struct Job {
    state: String,
    result: String,
    error: String,
}

fn jobs() -> &'static Mutex<HashMap<String, Job>> {
    static JOBS: OnceLock<Mutex<HashMap<String, Job>>> = OnceLock::new();
    JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default()
}

fn field(body: &[u8], name: &str) -> String {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get(name)
                .and_then(|inner| inner.as_str())
                .map(str::to_string)
        })
        .unwrap_or_default()
}

fn seconds_field(body: &[u8], fallback: u64) -> u64 {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|value| value.get("seconds").and_then(|inner| inner.as_u64()))
        .unwrap_or(fallback)
        .clamp(1, 60)
}

pub fn snapshot() -> String {
    let hardware = crate::hardware::detect();
    let cores = crate::engines::detect_cores();
    let gpu = crate::engines::detect_gpu_memory_mb();
    let profile = crate::engines::profile_for(cores, gpu);
    let fidelity = crate::engines::fidelity_for(profile);
    let (min_ms, max_ms) = profile.target_ms();
    let engines: Vec<serde_json::Value> = crate::engines::ENGINES
        .iter()
        .map(|engine| {
            serde_json::json!({
                "id": engine.id,
                "name": engine.name,
                "state": if engine.available { "available" } else { "planned" },
                "license": engine.license.label(),
                "note": engine.license.commercial_note(),
                "hardware": engine.hardware,
                "needs": if engine.requires_gpu { "gpu" } else { "cpu" },
                "default": engine.default,
                "fidelity": engine.fidelity.iter().map(|level| level.to_string()).collect::<Vec<String>>(),
            })
        })
        .collect();
    let inputs = crate::record::devices().unwrap_or_default();
    let outputs = crate::playback::devices().unwrap_or_default();
    let guidance = microphone(&inputs);
    let executable = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    serde_json::json!({
        "hardware": {
            "memory_mb": hardware.memory_mb,
            "free_disk_mb": hardware.free_disk_mb,
            "threads": hardware.threads,
        },
        "profile": {
            "cores": cores,
            "gpu_mb": gpu,
            "label": profile.label(),
            "fidelity": fidelity.to_string(),
            "target_ms": [min_ms, max_ms],
        },
        "engines": engines,
        "inputs": inputs,
        "outputs": outputs,
        "microphone": guidance,
        "pronounce": { "words": pronounce::entries().len() },
        "mcp": { "command": executable, "args": ["mcp"], "transport": "stdio" },
        "models": crate::models::models_root().display().to_string(),
    })
    .to_string()
}

fn microphone(inputs: &[String]) -> String {
    #[cfg(target_os = "windows")]
    {
        crate::mic::windows_guidance(inputs)
    }
    #[cfg(not(target_os = "windows"))]
    {
        match crate::mic::list_command().run() {
            Ok(listing) => match crate::mic::find_module(&listing) {
                Some(id) => format!("virtual microphone  loaded as module {id}\noutput              parlo listen --play feeds it"),
                None => "virtual microphone  not loaded\nnext                install pactl, then load the null sink".to_string(),
            },
            Err(error) => format!("virtual microphone  could not be checked: {error}"),
        }
    }
}

fn pronounce_body(body: &[u8]) -> Result<String, String> {
    let text = field(body, "text").trim().to_string();
    if text.is_empty() {
        return Err("a word or a sentence is needed".to_string());
    }
    if text.chars().count() > 240 {
        return Err("keep it under 240 characters".to_string());
    }
    let entry = pronounce::lookup(&text);
    Ok(serde_json::json!({
        "text": text,
        "spoken": pronounce::respell(&text),
        "ipa": entry.map(|row| row.ipa.clone()).unwrap_or_default(),
        "cmu": entry.map(|row| row.cmu.clone()).unwrap_or_default(),
        "known": entry.is_some(),
    })
    .to_string())
}

fn record_body(body: &[u8]) -> Result<String, String> {
    let seconds = seconds_field(body, 3).min(15);
    let recording = crate::record::capture(seconds as f32, None)?;
    let level = peak(&recording.samples);
    let decibels = if level > 0.0 {
        20.0 * f64::from(level).log10()
    } else {
        -120.0
    };
    let rate = recording.format.sample_rate();
    let duration = recording.samples.len() as f32 / rate as f32;
    let path = std::env::temp_dir().join(format!("parlo-tool-record-{}.wav", millis()));
    std::fs::write(
        &path,
        write_pcm16(&Wav {
            format: recording.format,
            samples: recording.samples,
        }),
    )
    .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(serde_json::json!({
        "path": path.display().to_string(),
        "seconds": (duration * 10.0).round() / 10.0,
        "rate": rate,
        "peak_db": (decibels * 10.0).round() / 10.0,
        "quiet": level < 0.001,
    })
    .to_string())
}

fn start_record_job(seconds: u64) -> Result<String, String> {
    let id = format!("record-{}", millis());
    jobs()
        .lock()
        .map_err(|_| "jobs are unavailable".to_string())?
        .insert(
            id.clone(),
            Job {
                state: "running".to_string(),
                result: String::new(),
                error: String::new(),
            },
        );
    let key = id.clone();
    std::thread::spawn(move || {
        let body = serde_json::json!({ "seconds": seconds }).to_string();
        let outcome = record_body(body.as_bytes());
        if let Ok(mut store) = jobs().lock() {
            if let Some(job) = store.get_mut(&key) {
                job.state = "done".to_string();
                match outcome {
                    Ok(result) => job.result = result,
                    Err(error) => job.error = error,
                }
            }
        }
    });
    Ok(serde_json::json!({ "id": id }).to_string())
}

fn start_job(kind: &str, seconds: u64) -> Result<String, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let report = std::env::temp_dir().join(format!("parlo-soak-report-{}.json", millis()));
    let mut command = std::process::Command::new(executable);
    match kind {
        "bench" => {
            command.args(["bench", "--json"]);
        }
        "soak" => {
            command.args([
                "soak",
                "--seconds",
                &seconds.to_string(),
                "--interval",
                "2",
                "--report",
                &report.display().to_string(),
            ]);
        }
        other => return Err(format!("{other} is not a job this window runs")),
    }
    let id = format!("{kind}-{}", millis());
    jobs()
        .lock()
        .map_err(|_| "jobs are unavailable".to_string())?
        .insert(
            id.clone(),
            Job {
                state: "running".to_string(),
                result: String::new(),
                error: String::new(),
            },
        );
    let key = id.clone();
    std::thread::spawn(move || {
        let outcome = command
            .output()
            .map_err(|error| error.to_string())
            .and_then(|output| {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    Err(if stderr.is_empty() {
                        format!("the run stopped with {}", output.status)
                    } else {
                        stderr
                    })
                }
            });
        let mut result = outcome.unwrap_or_default();
        if result.trim().is_empty() && report.is_file() {
            result = std::fs::read_to_string(&report).unwrap_or_default();
        }
        if let Ok(mut store) = jobs().lock() {
            if let Some(job) = store.get_mut(&key) {
                job.state = "done".to_string();
                job.result = result;
            }
        }
    });
    Ok(serde_json::json!({ "id": id }).to_string())
}

fn job_body(query: &str) -> String {
    let id = query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == "id")
        .map(|(_, value)| value.to_string())
        .unwrap_or_default();
    let store = match jobs().lock() {
        Ok(store) => store,
        Err(_) => return "{\"error\":\"jobs are unavailable\"}".to_string(),
    };
    match store.get(&id) {
        Some(job) => serde_json::json!({
            "id": id,
            "state": job.state,
            "result": job.result,
            "error": job.error,
        })
        .to_string(),
        None => "{\"state\":\"unknown\"}".to_string(),
    }
}

pub fn handle(
    method: &str,
    path: &str,
    query: &str,
    body: &[u8],
    allowed: bool,
) -> Option<(&'static str, String)> {
    if !path.starts_with("/tools") {
        return None;
    }
    if !allowed {
        return Some((
            "401 Unauthorized",
            "{\"error\":\"wrong token\"}".to_string(),
        ));
    }
    let outcome = match (method, path) {
        ("GET", "/tools/snapshot") => Ok(snapshot()),
        ("POST", "/tools/pronounce") => pronounce_body(body),
        ("POST", "/tools/record") => start_record_job(seconds_field(body, 3).min(15)),
        ("POST", "/tools/job") => {
            let kind = field(body, "kind");
            start_job(&kind, seconds_field(body, 20))
        }
        ("GET", "/tools/job") => Ok(job_body(query)),
        _ => return None,
    };
    Some(match outcome {
        Ok(payload) => ("200 OK", payload),
        Err(error) => (
            "400 Bad Request",
            serde_json::json!({ "error": error }).to_string(),
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_snapshot_carries_what_every_tool_panel_reads() {
        let value: serde_json::Value = serde_json::from_str(&snapshot()).unwrap();
        for key in [
            "hardware",
            "profile",
            "engines",
            "inputs",
            "outputs",
            "microphone",
            "pronounce",
            "mcp",
        ] {
            assert!(value.get(key).is_some(), "{key} is missing");
        }
        assert!(value["hardware"]["threads"].as_u64().unwrap() >= 1);
        assert!(!value["engines"].as_array().unwrap().is_empty());
        assert!(value["mcp"]["command"].as_str().unwrap().len() > 3);
    }

    #[test]
    fn a_pronunciation_comes_back_with_a_respelling() {
        let body = pronounce_body(br#"{"text":"Parlo"}"#).unwrap();
        let value: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(value["text"], "Parlo");
        assert!(value["spoken"].as_str().unwrap().len() >= 5);
    }

    #[test]
    fn an_empty_word_is_refused() {
        assert!(pronounce_body(br#"{"text":"   "}"#).is_err());
    }

    #[test]
    fn a_job_that_is_not_a_tool_is_refused() {
        assert!(start_job("dub", 1).is_err());
    }

    #[test]
    fn a_missing_token_is_refused() {
        let outcome = handle("GET", "/tools/snapshot", "", b"", false).unwrap();
        assert_eq!(outcome.0, "401 Unauthorized");
    }
}
