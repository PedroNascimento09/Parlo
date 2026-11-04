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

pub fn append(entry: &Entry) -> Result<(), String> {
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
        assert!(edit_with_suggestions("qualquer", "   ").is_err());
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

    #[test]
    fn a_search_finds_a_word_and_ignores_accents() {
        let path = scratch("search");
        kept(&path, "o relatório ficou pronto");
        kept(&path, "a reunião começa às dez");
        assert_eq!(search_at(&path, "relatorio", 10).len(), 1);
        assert_eq!(search_at(&path, "reuniao", 10).len(), 1);
        assert_eq!(search_at(&path, "nada", 10).len(), 0);
        tidy(&path);
    }

    #[test]
    fn a_search_reads_the_program_and_the_translation() {
        let path = scratch("search-app");
        append_to(
            &path,
            &Entry::now("bom dia", "good morning", "pt", "en", "Notepad", "dictate"),
        )
        .unwrap();
        assert_eq!(search_at(&path, "notepad", 10).len(), 1);
        assert_eq!(search_at(&path, "good morning", 10).len(), 1);
        tidy(&path);
    }

    #[test]
    fn a_search_keeps_the_newest_first_and_respects_the_limit() {
        let path = scratch("search-order");
        kept(&path, "alpha one");
        kept(&path, "alpha two");
        let hits = search_at(&path, "alpha", 1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text, "alpha two");
        tidy(&path);
    }

    #[test]
    fn a_restyle_rewrites_the_sentence_in_place() {
        let path = scratch("restyle");
        kept(&path, "bom dia como estas");
        let id = entries_at(&path)[0].id.clone();
        let changed = restyle_at(&path, &id, "formal").unwrap();
        assert_eq!(changed.text, "Bom dia como estas.");
        assert_eq!(entries_at(&path)[0].text, "Bom dia como estas.");
        tidy(&path);
    }

    #[test]
    fn a_correction_of_one_word_proposes_that_word() {
        let proposed = suggestions("meet me on teusday", "meet me on Tuesday");
        assert_eq!(
            proposed,
            vec![("teusday".to_string(), "Tuesday".to_string())]
        );
    }

    #[test]
    fn an_unchanged_word_proposes_nothing() {
        assert!(suggestions("bom dia", "bom dia").is_empty());
        assert!(suggestions("bom dia", "Bom dia.").is_empty());
    }

    #[test]
    fn a_whole_rewrite_proposes_nothing() {
        assert!(suggestions("one two three four five", "six seven eight nine ten").is_empty());
    }

    #[test]
    fn short_words_are_never_proposed() {
        assert!(suggestions("at it", "to it").is_empty());
    }

    #[test]
    fn an_edit_returns_the_suggestions_it_found() {
        let path = scratch("learn");
        kept(&path, "meet me on teusday");
        let id = entries_at(&path)[0].id.clone();
        let heard = entries_at(&path)[0].text.clone();
        let proposed = suggestions(&heard, "meet me on Tuesday");
        assert_eq!(proposed.len(), 1);
        with_entry(&path, &id, |entry| {
            entry.text = "meet me on Tuesday".to_string()
        })
        .unwrap();
        assert_eq!(entries_at(&path)[0].text, "meet me on Tuesday");
        tidy(&path);
    }

    #[test]
    fn an_export_names_the_day_the_program_and_the_sentence() {
        let path = scratch("export");
        let mut entry = Entry::now("olá", "hello", "pt", "en", "Slack", "dictate");
        entry.at = 1_700_000_000;
        append_to(&path, &entry).unwrap();
        let text = markdown(&entries_at(&path));
        assert!(text.starts_with("# Parlo dictations\n\n- **2023-11-14 22:13** · Slack"));
        assert!(text.contains("pt → en"));
        assert!(text.contains("\n  olá\n"));
        tidy(&path);
    }

    #[test]
    fn an_empty_export_says_so() {
        assert_eq!(markdown(&[]), "No dictations yet.\n");
    }

    #[test]
    fn a_line_older_than_the_window_is_removed() {
        let path = scratch("prune");
        let mut old = Entry::now("velha", "old", "pt", "en", "", "dictate");
        old.at = 10 * SECONDS_PER_DAY;
        append_to(&path, &old).unwrap();
        let mut fresh = Entry::now("nova", "new", "pt", "en", "", "dictate");
        fresh.at = 99 * SECONDS_PER_DAY;
        append_to(&path, &fresh).unwrap();
        assert_eq!(prune_at(&path, 30, 100 * SECONDS_PER_DAY), Ok(1));
        let left = entries_at(&path);
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].text, "nova");
        tidy(&path);
    }

    #[test]
    fn a_window_of_no_days_keeps_every_line() {
        let path = scratch("prune-off");
        let mut old = Entry::now("velha", "old", "pt", "en", "", "dictate");
        old.at = 1;
        append_to(&path, &old).unwrap();
        assert_eq!(prune_at(&path, 0, 100 * SECONDS_PER_DAY), Ok(0));
        assert_eq!(entries_at(&path).len(), 1);
        tidy(&path);
    }

    #[test]
    fn an_empty_search_returns_the_recent_lines() {
        let path = scratch("search-empty");
        kept(&path, "uma");
        assert_eq!(search_at(&path, "", 10).len(), 1);
        tidy(&path);
    }
}
