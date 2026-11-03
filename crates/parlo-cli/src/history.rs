use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const LIMIT: usize = 200;

static NEXT: AtomicU64 = AtomicU64::new(0);

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
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub favourite: bool,
    #[serde(default)]
    pub seconds: f64,
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
            id: String::new(),
            favourite: false,
            seconds: 0.0,
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

pub fn favourite(id: &str, value: bool) -> Result<Entry, String> {
    with_entry(&path(), id, |entry| entry.favourite = value)
}

pub fn edit(id: &str, text: &str) -> Result<Entry, String> {
    let spoken = text.trim();
    if spoken.is_empty() {
        return Err("a corrected sentence cannot be empty".to_string());
    }
    with_entry(&path(), id, |entry| entry.text = spoken.to_string())
}

pub fn remove(id: &str) -> Result<(), String> {
    let path = path();
    let mut entries = entries_at(&path);
    let before = entries.len();
    entries.retain(|entry| entry.id != id);
    if entries.len() == before {
        return Err("no dictation carries that id".to_string());
    }
    write_entries(&path, &entries)
}

pub fn entry_json(entry: &Entry) -> String {
    serde_json::json!({
        "id": entry.id,
        "at": entry.at,
        "text": entry.text,
        "translation": entry.translation,
        "from": entry.from,
        "to": entry.to,
        "app": entry.app,
        "mode": entry.mode,
        "favourite": entry.favourite,
        "seconds": entry.seconds,
    })
    .to_string()
}

pub fn as_json(limit: usize) -> String {
    let entries: Vec<serde_json::Value> = recent(limit)
        .iter()
        .filter_map(|entry| serde_json::from_str(&entry_json(entry)).ok())
        .collect();
    serde_json::json!(entries).to_string()
}

fn new_id() -> String {
    let counter = NEXT.fetch_add(1, Ordering::Relaxed);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();
    format!("h{stamp}-{counter}")
}

fn with_entry<F>(path: &Path, id: &str, apply: F) -> Result<Entry, String>
where
    F: FnOnce(&mut Entry),
{
    let mut entries = entries_at(path);
    let Some(index) = entries.iter().position(|entry| entry.id == id) else {
        return Err("no dictation carries that id".to_string());
    };
    apply(&mut entries[index]);
    let changed = entries[index].clone();
    write_entries(path, &entries)?;
    Ok(changed)
}

fn append_to(path: &Path, entry: &Entry) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let mut entries = entries_at(path);
    let mut fresh = entry.clone();
    if fresh.id.is_empty() {
        fresh.id = new_id();
    }
    entries.push(fresh);
    if entries.len() > LIMIT {
        entries.drain(..entries.len() - LIMIT);
    }
    write_entries(path, &entries)
}

fn write_entries(path: &Path, entries: &[Entry]) -> Result<(), String> {
    let mut text = String::new();
    for entry in entries {
        text.push_str(&serde_json::to_string(entry).map_err(|error| error.to_string())?);
        text.push('\n');
    }
    fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

fn entries_at(path: &Path) -> Vec<Entry> {
    let mut entries: Vec<Entry> = fs::read_to_string(path)
        .map(|text| {
            text.lines()
                .filter_map(|line| serde_json::from_str::<Entry>(line).ok())
                .collect()
        })
        .unwrap_or_default();
    for (position, entry) in entries.iter_mut().enumerate() {
        if entry.id.is_empty() {
            entry.id = format!("e{position}");
        }
    }
    entries
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

    fn kept(path: &Path, text: &str) {
        append_to(
            path,
            &Entry::now(text, text, "pt", "en", "Notepad", "dictate"),
        )
        .unwrap();
    }

    #[test]
    fn keeps_the_newest_first() {
        let path = scratch("order");
        kept(&path, "bom dia");
        kept(&path, "até logo");
        let recent = recent_at(&path, 10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].translation, "até logo");
        assert_eq!(recent[1].text, "bom dia");
        tidy(&path);
    }

    #[test]
    fn a_limit_returns_only_what_was_asked() {
        let path = scratch("limit");
        for index in 0..5 {
            kept(&path, &format!("frase {index}"));
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
            kept(&path, &format!("linha {index}"));
        }
        let entries = entries_at(&path);
        assert_eq!(entries.len(), LIMIT);
        assert_eq!(entries[LIMIT - 1].text, format!("linha {}", LIMIT + 4));
        tidy(&path);
    }

    #[test]
    fn a_damaged_line_never_hides_the_rest() {
        let path = scratch("damaged");
        kept(&path, "primeira");
        let mut file = fs::read_to_string(&path).unwrap();
        file.push_str("isto não é json\n");
        fs::write(&path, file).unwrap();
        kept(&path, "segunda");
        let recent = recent_at(&path, 10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].text, "segunda");
        tidy(&path);
    }

    #[test]
    fn clearing_removes_the_file_and_tolerates_a_missing_one() {
        let path = scratch("clear");
        kept(&path, "apagar");
        assert!(path.exists());
        clear_at(&path).unwrap();
        assert!(!path.exists());
        assert!(clear_at(&path).is_ok());
        tidy(&path);
    }

    #[test]
    fn every_entry_carries_its_own_id() {
        let path = scratch("ids");
        kept(&path, "primeira");
        kept(&path, "segunda");
        let entries = entries_at(&path);
        assert_ne!(entries[0].id, entries[1].id);
        assert!(!entries[0].favourite);
        tidy(&path);
    }

    #[test]
    fn a_favourite_survives_a_reload() {
        let path = scratch("favourite");
        kept(&path, "guardar");
        let id = entries_at(&path)[0].id.clone();
        with_entry(&path, &id, |entry| entry.favourite = true).unwrap();
        assert!(entries_at(&path)[0].favourite);
        tidy(&path);
    }

    #[test]
    fn a_correction_replaces_the_heard_sentence() {
        let path = scratch("edit");
        kept(&path, "errado");
        let id = entries_at(&path)[0].id.clone();
        with_entry(&path, &id, |entry| entry.text = "certo".to_string()).unwrap();
        assert_eq!(entries_at(&path)[0].text, "certo");
        tidy(&path);
    }

    #[test]
    fn an_empty_correction_is_refused() {
        assert!(edit("qualquer", "   ").is_err());
    }

    #[test]
    fn removing_one_keeps_the_rest() {
        let path = scratch("remove");
        kept(&path, "primeira");
        kept(&path, "segunda");
        let entries = entries_at(&path);
        let target = entries[1].id.clone();
        let mut remaining = entries_at(&path);
        remaining.retain(|entry| entry.id != target);
        write_entries(&path, &remaining).unwrap();
        let left = entries_at(&path);
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].text, "primeira");
        tidy(&path);
    }

    #[test]
    fn a_line_written_before_ids_existed_still_answers() {
        let path = scratch("legacy");
        fs::write(
            &path,
            "{\"at\":1,\"text\":\"antiga\",\"translation\":\"old\",\"from\":\"pt\",\"to\":\"en\",\"app\":\"\",\"mode\":\"\"}\n",
        )
        .unwrap();
        assert_eq!(entries_at(&path)[0].id, "e0");
        let entries = entries_at(&path);
        let id = entries[0].id.clone();
        with_entry(&path, &id, |entry| entry.favourite = true).unwrap();
        assert!(entries_at(&path)[0].favourite);
        tidy(&path);
    }

    #[test]
    fn an_unknown_id_is_refused() {
        let path = scratch("unknown");
        assert!(with_entry(&path, "nada", |_| {}).is_err());
        tidy(&path);
    }
}
