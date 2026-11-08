use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const LIMIT: usize = 200;
const SECONDS_PER_DAY: u64 = 86_400;

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

fn worth_keeping(entry: &Entry) -> bool {
    !entry.text.trim().is_empty()
}

pub fn append(entry: &Entry) -> Result<(), String> {
    if !worth_keeping(entry) {
        return Ok(());
    }
    let _ = prune(crate::config::load().history_days);
    append_to(&path(), entry)
}

pub fn export(days: u32) -> Result<String, String> {
    let mut entries = entries_at(&path());
    if days > 0 {
        let cutoff = now_seconds().saturating_sub(u64::from(days) * SECONDS_PER_DAY);
        entries.retain(|entry| entry.at >= cutoff);
    }
    Ok(markdown(&entries))
}

pub fn prune(days: u32) -> Result<usize, String> {
    prune_at(&path(), days, now_seconds())
}

pub fn markdown(entries: &[Entry]) -> String {
    if entries.is_empty() {
        return "No dictations yet.\n".to_string();
    }
    let mut out = String::from("# Parlo dictations\n\n");
    for entry in entries {
        let program = if entry.app.is_empty() {
            "another program"
        } else {
            entry.app.as_str()
        };
        out.push_str(&format!("- **{}** · {program}", moment(entry.at)));
        if !entry.from.is_empty() && !entry.to.is_empty() {
            out.push_str(&format!(" · {} → {}", entry.from, entry.to));
        }
        out.push('\n');
        out.push_str(&format!("  {}\n", entry.text.trim()));
    }
    out
}

fn prune_at(path: &Path, days: u32, now: u64) -> Result<usize, String> {
    if days == 0 {
        return Ok(0);
    }
    let mut entries = entries_at(path);
    let cutoff = now.saturating_sub(u64::from(days) * SECONDS_PER_DAY);
    let before = entries.len();
    entries.retain(|entry| entry.at >= cutoff);
    let removed = before - entries.len();
    if removed > 0 {
        write_entries(path, &entries)?;
    }
    Ok(removed)
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

fn moment(seconds: u64) -> String {
    let clock = seconds % SECONDS_PER_DAY;
    let (year, month, day) = civil((seconds / SECONDS_PER_DAY) as i64);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        clock / 3_600,
        (clock / 60) % 60
    )
}

fn civil(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = (shifted - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_index + 2) / 5 + 1) as u32;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

pub fn recent(limit: usize) -> Vec<Entry> {
    recent_at(&path(), limit)
}

pub fn favourites(limit: usize) -> Vec<Entry> {
    let mut entries: Vec<Entry> = entries_at(&path())
        .into_iter()
        .rev()
        .filter(|entry| entry.favourite)
        .collect();
    entries.truncate(limit);
    entries
}

pub fn clear() -> Result<(), String> {
    clear_at(&path())
}

pub fn favourite(id: &str, value: bool) -> Result<Entry, String> {
    with_entry(&path(), id, |entry| entry.favourite = value)
}

pub fn edit_with_suggestions(
    id: &str,
    text: &str,
) -> Result<(Entry, Vec<(String, String)>), String> {
    let spoken = text.trim();
    if spoken.is_empty() {
        return Err("a corrected sentence cannot be empty".to_string());
    }
    let path = path();
    let heard = entries_at(&path)
        .into_iter()
        .find(|entry| entry.id == id)
        .map(|entry| entry.text)
        .ok_or_else(|| "no dictation carries that id".to_string())?;
    let suggestions = suggestions(&heard, spoken);
    let entry = with_entry(&path, id, |entry| entry.text = spoken.to_string())?;
    Ok((entry, suggestions))
}

pub fn suggestions(heard: &str, written: &str) -> Vec<(String, String)> {
    const MOST: usize = 3;
    let heard_words = words(heard);
    let written_words = words(written);
    let lost: Vec<&String> = heard_words
        .iter()
        .filter(|word| !written_words.iter().any(|kept| fold(kept) == fold(word)))
        .collect();
    let gained: Vec<&String> = written_words
        .iter()
        .filter(|word| !heard_words.iter().any(|had| fold(had) == fold(word)))
        .collect();
    if lost.is_empty() || gained.is_empty() || lost.len() > MOST || gained.len() > MOST {
        return Vec::new();
    }
    lost.iter()
        .zip(gained.iter())
        .map(|(wrong, right)| (wrong.to_string(), right.to_string()))
        .collect()
}

fn words(text: &str) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        let plain: String = word
            .chars()
            .filter(|character| character.is_alphanumeric())
            .collect();
        if plain.chars().count() < 3 {
            continue;
        }
        if kept.iter().any(|seen| fold(seen) == fold(&plain)) {
            continue;
        }
        kept.push(plain);
    }
    kept
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

pub fn search(query: &str, limit: usize) -> Vec<Entry> {
    let wanted = fold(query);
    if wanted.is_empty() {
        return recent(limit);
    }
    search_at(&path(), &wanted, limit)
}

pub fn search_json(query: &str, limit: usize) -> String {
    let entries: Vec<serde_json::Value> = search(query, limit)
        .iter()
        .filter_map(|entry| serde_json::from_str(&entry_json(entry)).ok())
        .collect();
    serde_json::json!(entries).to_string()
}

pub fn restyle(id: &str, style: &str) -> Result<Entry, String> {
    restyle_at(&path(), id, style)
}

fn restyle_at(path: &Path, id: &str, style: &str) -> Result<Entry, String> {
    with_entry(path, id, |entry| {
        entry.text = crate::style::shape(&entry.text, style)
    })
}

fn search_at(path: &Path, wanted: &str, limit: usize) -> Vec<Entry> {
    let mut hits: Vec<Entry> = entries_at(path)
        .into_iter()
        .rev()
        .filter(|entry| {
            fold(&entry.text).contains(wanted)
                || fold(&entry.translation).contains(wanted)
                || fold(&entry.app).contains(wanted)
        })
        .collect();
    hits.truncate(limit);
    hits
}

fn fold(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        let plain = match character {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            other => other.to_ascii_lowercase(),
        };
        out.push(plain);
    }
    out
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
mod tests;
