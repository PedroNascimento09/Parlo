use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const LIMIT: usize = 200;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub at: u64,
    pub text: String,
    pub translation: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub app: String,
    #[serde(default)]
    pub mode: String,
}

impl Entry {
    pub fn now(text: &str, translation: &str, from: &str, to: &str, app: &str, mode: &str) -> Self {
        Self {
            at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|elapsed| elapsed.as_secs())
                .unwrap_or_default(),
            text: text.to_string(),
            translation: translation.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            app: app.to_string(),
            mode: mode.to_string(),
        }
    }
}

pub fn path() -> PathBuf {
    crate::config::data_root().join("history.jsonl")
}

pub fn append(entry: &Entry) -> Result<(), String> {
    append_to(&path(), entry)
}

pub fn recent(limit: usize) -> Vec<Entry> {
    recent_at(&path(), limit)
}

pub fn clear() -> Result<(), String> {
    clear_at(&path())
}

pub fn as_json(limit: usize) -> String {
    let entries: Vec<serde_json::Value> = recent(limit)
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "at": entry.at,
                "text": entry.text,
                "translation": entry.translation,
                "from": entry.from,
                "to": entry.to,
                "app": entry.app,
                "mode": entry.mode,
            })
        })
        .collect();
    serde_json::json!(entries).to_string()
}

fn append_to(path: &Path, entry: &Entry) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let line = serde_json::to_string(entry).map_err(|error| error.to_string())?;
    let mut text = fs::read_to_string(path).unwrap_or_default();
    text.push_str(&line);
    text.push('\n');
    let kept = bounded(&text);
    fs::write(path, kept).map_err(|error| format!("{}: {error}", path.display()))
}

fn bounded(text: &str) -> String {
    let mut entries: Vec<String> = text
        .lines()
        .filter(|line| serde_json::from_str::<Entry>(line).is_ok())
        .map(|line| line.to_string())
        .collect();
    if entries.len() > LIMIT {
        entries.drain(..entries.len() - LIMIT);
    }
    let mut kept = entries.join("\n");
    kept.push('\n');
    kept
}

fn entries_at(path: &Path) -> Vec<Entry> {
    fs::read_to_string(path)
        .map(|text| {
            text.lines()
                .filter_map(|line| serde_json::from_str::<Entry>(line).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn recent_at(path: &Path, limit: usize) -> Vec<Entry> {
    let mut entries = entries_at(path);
    entries.reverse();
    entries.truncate(limit);
    entries
}

fn clear_at(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        let folder = std::env::temp_dir().join(format!("parlo-history-{name}-{nanos}"));
        fs::create_dir_all(&folder).unwrap();
        folder.join("history.jsonl")
    }

    fn tidy(path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn keeps_the_newest_first() {
        let path = scratch("order");
        append_to(
            &path,
            &Entry::now("bom dia", "good morning", "pt", "en", "Notepad", "dictate"),
        )
        .unwrap();
        append_to(
            &path,
            &Entry::now("até logo", "see you soon", "pt", "en", "Slack", "dictate"),
        )
        .unwrap();
        let recent = recent_at(&path, 10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].translation, "see you soon");
        assert_eq!(recent[1].text, "bom dia");
        tidy(&path);
    }

    #[test]
    fn a_limit_returns_only_what_was_asked() {
        let path = scratch("limit");
        for index in 0..5 {
            let text = format!("frase {index}");
            append_to(
                &path,
                &Entry::now(&text, &text, "pt", "pt", "Notepad", "dictate"),
            )
            .unwrap();
        }
        let recent = recent_at(&path, 2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].text, "frase 4");
        tidy(&path);
    }

    #[test]
    fn the_store_keeps_a_bound() {
        let path = scratch("bound");
        for index in 0..(LIMIT + 5) {
            let text = format!("linha {index}");
            append_to(
                &path,
                &Entry::now(&text, &text, "pt", "pt", "Notepad", "dictate"),
            )
            .unwrap();
        }
        let entries = entries_at(&path);
        assert_eq!(entries.len(), LIMIT);
        assert_eq!(entries[LIMIT - 1].text, format!("linha {}", LIMIT + 4));
        tidy(&path);
    }

    #[test]
    fn a_damaged_line_never_hides_the_rest() {
        let path = scratch("damaged");
        append_to(
            &path,
            &Entry::now("primeira", "first", "pt", "en", "Notepad", "dictate"),
        )
        .unwrap();
        let mut file = fs::read_to_string(&path).unwrap();
        file.push_str("isto não é json\n");
        fs::write(&path, file).unwrap();
        append_to(
            &path,
            &Entry::now("segunda", "second", "pt", "en", "Notepad", "dictate"),
        )
        .unwrap();
        let recent = recent_at(&path, 10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].text, "segunda");
        tidy(&path);
    }

    #[test]
    fn clearing_removes_the_file_and_tolerates_a_missing_one() {
        let path = scratch("clear");
        append_to(
            &path,
            &Entry::now("apagar", "delete", "pt", "en", "Notepad", "dictate"),
        )
        .unwrap();
        assert!(path.exists());
        clear_at(&path).unwrap();
        assert!(!path.exists());
        assert!(clear_at(&path).is_ok());
        tidy(&path);
    }
}
