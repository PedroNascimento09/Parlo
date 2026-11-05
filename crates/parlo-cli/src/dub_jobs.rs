use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const RENDERING: &str = "rendering";
pub const DONE: &str = "done";
pub const FAILED: &str = "failed";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Export {
    pub format: String,
    pub label: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub source: String,
    pub cues: Option<String>,
    pub out: String,
    pub srt: String,
    pub state: String,
    pub stage: String,
    pub progress: f64,
    pub error: Option<String>,
    pub from: String,
    pub to: String,
    pub timing: String,
    pub fastest: f64,
    pub slowest: f64,
    pub keep_background: bool,
    pub seconds: f64,
    pub lines: usize,
    pub created_at: u64,
    pub updated_at: u64,
    pub exports: Vec<Export>,
}

pub fn directory(id: &str) -> Result<PathBuf, String> {
    if id.is_empty()
        || !id.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        return Err("the project id is not valid".to_string());
    }
    let root = crate::config::data_root().join("dubs");
    fs::create_dir_all(&root)
        .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
    Ok(root.join(id))
}

pub fn source_name(name: &str) -> String {
    let extension = Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 8
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric())
        })
        .unwrap_or_else(|| "bin".to_string());
    format!("source.{extension}")
}

pub fn write(dir: &Path, job: &mut Job) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    job.updated_at = now();
    let text = serde_json::to_string_pretty(job)
        .map_err(|error| format!("cannot encode the render: {error}"))?;
    let path = dir.join("manifest.json");
    fs::write(&path, text).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

pub fn read(dir: &Path) -> Result<Job, String> {
    let path = dir.join("manifest.json");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

pub fn has_manifest(dir: &Path) -> bool {
    dir.join("manifest.json").is_file()
}

pub fn path(dir: &Path, value: &str) -> Option<PathBuf> {
    if value.is_empty()
        || value.contains("..")
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_. ".contains(character))
    {
        return None;
    }
    Some(dir.join(value))
}

pub fn name(dir: &Path, value: &str) -> Option<PathBuf> {
    let path = path(dir, value)?;
    path.is_file().then_some(path)
}

pub fn export_path(dir: &Path, format: &str) -> Option<(PathBuf, String)> {
    let job = read(dir).ok()?;
    if job.state != DONE {
        return None;
    }
    let item = job.exports.iter().find(|item| item.format == format)?;
    name(dir, &item.name).map(|path| (path, item.name.clone()))
}

pub fn clean_outputs(dir: &Path) {
    let previous = read(dir).ok();
    if let Some(job) = &previous {
        for item in &job.exports {
            if let Some(path) = name(dir, &item.name) {
                fs::remove_file(path).ok();
            }
        }
        if let Some(path) = name(dir, &job.out) {
            fs::remove_file(path).ok();
        }
        if let Some(path) = name(dir, &job.srt) {
            fs::remove_file(path).ok();
        }
        fs::remove_file(dir.join(format!("{}-track.wav", job.id))).ok();
        fs::remove_file(dir.join(format!("{}-report.json", job.id))).ok();
    }
    fs::remove_dir_all(dir.join("work")).ok();
    fs::remove_dir_all(dir.join("lines")).ok();
    fs::remove_file(dir.join("track.wav")).ok();
    fs::remove_file(dir.join("report.json")).ok();
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Job {
        Job {
            id: "abc".to_string(),
            project_id: Some("abc".to_string()),
            title: "Demo".to_string(),
            source: "source.mp4".to_string(),
            cues: Some("abc-cues.json".to_string()),
            out: "abc-demo.mp4".to_string(),
            srt: "abc-demo.srt".to_string(),
            state: RENDERING.to_string(),
            stage: "synthesizing".to_string(),
            progress: 0.05,
            error: None,
            from: "pt".to_string(),
            to: "en".to_string(),
            timing: "natural".to_string(),
            fastest: 1.25,
            slowest: 0.9,
            keep_background: false,
            seconds: 0.0,
            lines: 0,
            created_at: 1,
            updated_at: 1,
            exports: Vec::new(),
        }
    }

    #[test]
    fn manifest_round_trips_and_cleanup_keeps_the_source() {
        let dir =
            std::env::temp_dir().join(format!("parlo-dub-job-{}-{}", std::process::id(), now()));
        fs::create_dir_all(dir.join("work")).unwrap();
        fs::write(dir.join("source.mp4"), b"source").unwrap();
        fs::write(dir.join("abc-cues.json"), b"cues").unwrap();
        fs::write(dir.join("abc-demo.mp4"), b"video").unwrap();
        fs::write(dir.join("abc-demo.srt"), b"subs").unwrap();
        fs::write(dir.join("work").join("part.wav"), b"part").unwrap();
        let mut job = sample();
        write(&dir, &mut job).unwrap();
        assert_eq!(read(&dir).unwrap().title, "Demo");
        clean_outputs(&dir);
        assert!(dir.join("source.mp4").exists());
        assert!(dir.join("abc-cues.json").exists());
        assert!(!dir.join("abc-demo.mp4").exists());
        assert!(!dir.join("work").exists());
        assert!(dir.join("manifest.json").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn names_cannot_leave_the_job_directory() {
        let dir = Path::new("C:/tmp/job");
        assert!(path(dir, "../secret").is_none());
        assert!(path(dir, "a/b").is_none());
        assert!(path(dir, "video.mp4").is_some());
    }
}
