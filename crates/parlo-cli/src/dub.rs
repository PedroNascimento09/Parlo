use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

const DEFAULT_ENGINE_DIR: &str = "engines/cascade";

#[derive(Deserialize)]
pub struct Line {
    pub start: f64,
    pub end: f64,
    pub source: String,
    pub translated: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub voice: Option<String>,
}

#[derive(Deserialize)]
pub struct Dub {
    pub out: String,
    pub srt: String,
    pub seconds: f64,
    pub lines: Vec<Line>,
}

pub struct Request<'a> {
    pub source: &'a Path,
    pub from: &'a str,
    pub to: &'a str,
    pub out: Option<PathBuf>,
    pub srt: Option<PathBuf>,
    pub cues: Option<PathBuf>,
    pub keep_background: bool,
    pub timing: &'a str,
    pub fastest: f64,
    pub slowest: f64,
}

pub const TIMING_MODES: [&str; 5] = ["natural", "concise", "smart-fit", "stretch", "lip-sync"];

pub fn check_timing(mode: &str, fastest: f64, slowest: f64) -> Result<(), String> {
    if !TIMING_MODES.contains(&mode) {
        return Err(format!("timing must be one of {}", TIMING_MODES.join(", ")));
    }
    if !fastest.is_finite() || !(1.0..=4.0).contains(&fastest) {
        return Err("fastest must be between 1 and 4".to_string());
    }
    if !slowest.is_finite() || !(0.25..=1.0).contains(&slowest) {
        return Err("slowest must be between 0.25 and 1".to_string());
    }
    Ok(())
}

pub fn run(request: &Request) -> Result<Dub, String> {
    let source = request
        .source
        .canonicalize()
        .map_err(|error| format!("{}: {error}", request.source.display()))?;
    if !source.is_file() {
        return Err(format!("{} is not a file", request.source.display()));
    }
    let here = env::current_dir().map_err(|error| error.to_string())?;
    let absolute = |path: &Option<PathBuf>| -> Option<PathBuf> {
        path.as_ref().map(|path| {
            if path.is_absolute() {
                path.clone()
            } else {
                here.join(path)
            }
        })
    };
    let out = absolute(&request.out);
    let srt = absolute(&request.srt);
    let cues = absolute(&request.cues);
    let engine_dir = env::var("PARLO_CASCADE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_ENGINE_DIR))
        .canonicalize()
        .map_err(|error| format!("{DEFAULT_ENGINE_DIR}: {error}"))?;
    let script = engine_dir.join("scripts").join("dub.py");
    if !script.is_file() {
        return Err(format!(
            "cannot find {}; run dub from the source checkout",
            script.display()
        ));
    }

    let report = env::temp_dir().join(format!("parlo-dub-{}-{}.json", std::process::id(), stamp()));
    let python = crate::sidecar::python_for(&engine_dir);
    let fastest = request.fastest.to_string();
    let slowest = request.slowest.to_string();
    let mut command = Command::new(&python);
    command
        .arg(&script)
        .arg(&source)
        .args(["--from", request.from, "--to", request.to])
        .args(["--timing", request.timing])
        .args(["--fastest", &fastest, "--slowest", &slowest])
        .arg("--json")
        .arg(&report)
        .current_dir(&engine_dir)
        .env("PYTHONPATH", &engine_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(out) = &out {
        command.arg("--out").arg(out);
    }
    if let Some(srt) = &srt {
        command.arg("--srt").arg(srt);
    }
    if let Some(cues) = &cues {
        command.arg("--cues").arg(cues);
    }
    if request.keep_background {
        command.arg("--keep-background");
    }

    let status = command.status().map_err(|error| {
        format!("cannot start {python}: {error}; install the cascade requirements first")
    })?;
    if !status.success() {
        fs::remove_file(&report).ok();
        return Err(format!(
            "dubbing stopped with {}; the reason is printed above",
            status
                .code()
                .map(|code| format!("status {code}"))
                .unwrap_or_else(|| "a signal".to_string())
        ));
    }

    let text = fs::read_to_string(&report)
        .map_err(|error| format!("cannot read the dub report {}: {error}", report.display()))?;
    fs::remove_file(&report).ok();
    serde_json::from_str(&text).map_err(|error| format!("invalid dub report: {error}"))
}

fn stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default()
}

pub fn safe_name(value: &str) -> String {
    let stem = value
        .trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default();
    let cleaned: String = stem
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || "-_. ".contains(character) {
                character
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        "file".to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_plain_names_and_drops_paths() {
        assert_eq!(safe_name("video.mp4"), "video.mp4");
        assert_eq!(safe_name("../../etc/passwd"), "passwd");
        assert_eq!(safe_name("C:\\Users\\pedro\\clip.mov"), "clip.mov");
    }

    #[test]
    fn replaces_unsafe_characters() {
        assert_eq!(safe_name("a<b>c?.mkv"), "a_b_c_.mkv");
        assert_eq!(safe_name("olá.mp4"), "ol_.mp4");
    }

    #[test]
    fn never_returns_an_empty_name() {
        assert_eq!(safe_name(".."), "file");
        assert_eq!(safe_name(""), "file");
        assert_eq!(safe_name("/"), "file");
    }

    #[test]
    fn accepts_every_timing_mode_with_sane_limits() {
        for mode in TIMING_MODES {
            assert!(check_timing(mode, 1.25, 0.9).is_ok());
        }
    }

    #[test]
    fn refuses_an_unknown_mode_and_bad_limits() {
        assert!(check_timing("fast", 1.25, 0.9)
            .unwrap_err()
            .contains("timing"));
        assert!(check_timing("smart-fit", 0.5, 0.9)
            .unwrap_err()
            .contains("fastest"));
        assert!(check_timing("smart-fit", 1.25, 1.5)
            .unwrap_err()
            .contains("slowest"));
        assert!(check_timing("smart-fit", f64::NAN, 0.9).is_err());
    }
}
