use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS migrations (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS voices (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    engine TEXT,
    licence TEXT,
    created_at INTEGER,
    consent_at INTEGER,
    origin TEXT,
    sealed INTEGER NOT NULL DEFAULT 0,
    revoked_at INTEGER,
    recipe_json TEXT
);
CREATE TABLE IF NOT EXISTS voice_samples (
    id TEXT PRIMARY KEY,
    voice_id TEXT NOT NULL REFERENCES voices(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    seconds REAL,
    peak_dbfs REAL,
    speech_ratio REAL,
    created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS designs (
    id TEXT PRIMARY KEY,
    voice_id TEXT REFERENCES voices(id) ON DELETE SET NULL,
    description TEXT,
    attributes_json TEXT,
    seed INTEGER
);
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    kind TEXT,
    title TEXT,
    source_path TEXT,
    source_hash TEXT,
    created_at INTEGER,
    updated_at INTEGER,
    settings_json TEXT
);
CREATE TABLE IF NOT EXISTS segments (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    start_ms INTEGER,
    end_ms INTEGER,
    speaker TEXT,
    voice_id TEXT,
    source_text TEXT,
    original_text TEXT,
    revisions_json TEXT,
    pace REAL,
    target_text TEXT,
    state TEXT,
    qc_json TEXT
);
CREATE TABLE IF NOT EXISTS segment_translations (
    segment_id TEXT REFERENCES segments(id) ON DELETE CASCADE,
    language TEXT NOT NULL,
    text TEXT NOT NULL,
    state TEXT NOT NULL,
    PRIMARY KEY (segment_id, language)
);
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    state TEXT NOT NULL,
    stage TEXT,
    progress REAL NOT NULL DEFAULT 0,
    input_hash TEXT,
    title TEXT,
    error TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS job_stages (
    job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    state TEXT NOT NULL,
    started_at INTEGER,
    ended_at INTEGER,
    detail TEXT,
    PRIMARY KEY (job_id, name)
);
CREATE TABLE IF NOT EXISTS takes (
    id TEXT PRIMARY KEY,
    voice_id TEXT REFERENCES voices(id) ON DELETE SET NULL,
    text TEXT,
    path TEXT,
    seconds REAL,
    created_at INTEGER
);
CREATE TABLE IF NOT EXISTS exports (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    path TEXT,
    format TEXT,
    bytes INTEGER,
    created_at INTEGER
);
CREATE TABLE IF NOT EXISTS meetings (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    duration_ms INTEGER,
    source TEXT,
    summary TEXT,
    summary_kind TEXT
);
CREATE TABLE IF NOT EXISTS meeting_segments (
    id TEXT PRIMARY KEY,
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    start_ms INTEGER NOT NULL,
    speaker TEXT,
    text TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS speakers (
    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    name TEXT,
    PRIMARY KEY (meeting_id, label)
);
CREATE TABLE IF NOT EXISTS history (
    id TEXT PRIMARY KEY,
    at INTEGER NOT NULL,
    said TEXT NOT NULL,
    wrote TEXT,
    app TEXT,
    mode TEXT,
    language_from TEXT,
    language_to TEXT
);
CREATE TABLE IF NOT EXISTS glossary (
    id TEXT PRIMARY KEY,
    term TEXT NOT NULL,
    replacement TEXT NOT NULL,
    language TEXT,
    kind TEXT
);
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

pub struct Store {
    connection: Connection,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Imported {
    pub history: usize,
    pub meetings: usize,
    pub settings: usize,
    pub already: bool,
}

pub struct StoredJob {
    pub id: String,
    pub kind: String,
    pub stage: String,
    pub progress: f32,
    pub input_hash: String,
    pub title: String,
    pub created_at: i64,
}

pub struct JobRecord<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub state: &'a str,
    pub stage: &'a str,
    pub progress: f32,
    pub input_hash: &'a str,
    pub title: &'a str,
    pub error: Option<&'a str>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DubProject {
    pub id: String,
    pub title: String,
    pub source_path: Option<String>,
    pub from: String,
    pub to: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub targets: Vec<String>,
    pub cast: BTreeMap<String, String>,
    pub segments: Vec<DubSegment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DubSummary {
    pub id: String,
    pub title: String,
    pub source_path: Option<String>,
    pub from: String,
    pub to: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub segments: usize,
    pub invalid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DubSegment {
    pub id: String,
    pub position: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub speaker: Option<String>,
    pub voice_id: Option<String>,
    pub translations: BTreeMap<String, String>,
    pub source_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pace: Option<f64>,
    pub target_text: Option<String>,
    pub state: String,
}

impl DubProject {
    pub fn new(
        id: String,
        title: String,
        from: &str,
        to: &str,
        source_path: Option<String>,
    ) -> Self {
        let stamp = now();
        Self {
            id,
            title,
            source_path,
            from: from.to_string(),
            to: to.to_string(),
            created_at: stamp,
            updated_at: stamp,
            targets: vec![to.to_string()],
            cast: BTreeMap::new(),
            segments: Vec::new(),
        }
    }
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let connection = Connection::open(path).map_err(sql)?;
        let store = Self { connection };
        store.prepare()?;
        Ok(store)
    }

    #[cfg(test)]
    pub fn open_memory() -> Result<Self, String> {
        let connection = Connection::open_in_memory().map_err(sql)?;
        let store = Self { connection };
        store.prepare()?;
        Ok(store)
    }

    pub fn default_path() -> PathBuf {
        crate::config::data_root().join("parlo.db")
    }

    pub fn open_default() -> Result<Self, String> {
        let mut store = Self::open(&Self::default_path())?;
        store.import_legacy(&crate::config::data_root())?;
        Ok(store)
    }

    fn prepare(&self) -> Result<(), String> {
        self.connection.execute_batch(SCHEMA).map_err(sql)?;
        let columns = {
            let mut statement = self
                .connection
                .prepare("PRAGMA table_info(segments)")
                .map_err(sql)?;
            let columns = statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(sql)?;
            let mut names = std::collections::HashSet::new();
            for column in columns {
                names.insert(column.map_err(sql)?);
            }
            names
        };
        if !columns.contains("original_text") {
            self.connection
                .execute("ALTER TABLE segments ADD COLUMN original_text TEXT", [])
                .map_err(sql)?;
        }
        if !columns.contains("pace") {
            self.connection
                .execute("ALTER TABLE segments ADD COLUMN pace REAL", [])
                .map_err(sql)?;
        }
        if !columns.contains("revisions_json") {
            self.connection
                .execute("ALTER TABLE segments ADD COLUMN revisions_json TEXT", [])
                .map_err(sql)?;
        }
        Ok(())
    }

    pub fn import_legacy(&mut self, root: &Path) -> Result<Imported, String> {
        let applied: i64 = self
            .connection
            .query_row(
                "SELECT COUNT(*) FROM migrations WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .map_err(sql)?;
        if applied > 0 {
            return Ok(Imported {
                already: true,
                ..Imported::default()
            });
        }

        let transaction = self.connection.transaction().map_err(sql)?;
        let mut imported = Imported::default();

        if let Ok(text) = fs::read_to_string(root.join("history.jsonl")) {
            for (index, line) in text.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                let Ok(entry) = serde_json::from_str::<crate::history::Entry>(line) else {
                    continue;
                };
                let id = if entry.id.is_empty() {
                    format!("legacy-{index}")
                } else {
                    entry.id
                };
                let changed = transaction
                    .execute(
                        "INSERT OR IGNORE INTO history (id, at, said, wrote, app, mode, language_from, language_to) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![id, entry.at, entry.text, entry.translation, entry.app, entry.mode, entry.from, entry.to],
                    )
                    .map_err(sql)?;
                imported.history += changed;
            }
        }

        if let Ok(entries) = fs::read_dir(root.join("meetings")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("json") {
                    continue;
                }
                let Ok(text) = fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(meeting) = serde_json::from_str::<crate::meeting::Meeting>(&text) else {
                    continue;
                };
                let duration = meeting
                    .segments
                    .iter()
                    .map(|segment| segment.at_ms)
                    .max()
                    .unwrap_or_default();
                let changed = transaction
                    .execute(
                        "INSERT OR IGNORE INTO meetings (id, title, started_at, duration_ms, source) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![meeting.id, meeting.title, meeting.started_at as i64, duration as i64, "legacy"],
                    )
                    .map_err(sql)?;
                imported.meetings += changed;
                for (index, segment) in meeting.segments.iter().enumerate() {
                    transaction
                        .execute(
                            "INSERT OR IGNORE INTO meeting_segments (id, meeting_id, start_ms, speaker, text) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![
                                format!("{}-{index}", meeting.id),
                                meeting.id,
                                segment.at_ms as i64,
                                segment.spoken_by(),
                                segment.text
                            ],
                        )
                        .map_err(sql)?;
                    transaction
                        .execute(
                            "INSERT OR IGNORE INTO speakers (meeting_id, label, name) VALUES (?1, ?2, ?3)",
                            params![meeting.id, segment.speaker, segment.spoken_by()],
                        )
                        .map_err(sql)?;
                }
            }
        }

        if let Ok(text) = fs::read_to_string(root.join("config.json")) {
            if let Ok(serde_json::Value::Object(fields)) = serde_json::from_str(&text) {
                for (key, value) in fields {
                    let changed = transaction
                        .execute(
                            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                            params![key, value.to_string()],
                        )
                        .map_err(sql)?;
                    imported.settings += changed;
                }
            }
        }

        transaction
            .execute(
                "INSERT INTO migrations (version, applied_at) VALUES (1, ?1)",
                params![now()],
            )
            .map_err(sql)?;
        transaction.commit().map_err(sql)?;
        Ok(imported)
    }

    pub fn unfinished_jobs(&self) -> Result<Vec<StoredJob>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT id, kind, stage, progress, input_hash, title, created_at FROM jobs WHERE state IN ('queued', 'running')")
            .map_err(sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok(StoredJob {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    stage: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    progress: row.get(3)?,
                    input_hash: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    title: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    created_at: row.get(6)?,
                })
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }

    #[cfg(test)]
    pub fn count(&self, table: &str) -> Result<usize, String> {
        let known = [
            "history",
            "meetings",
            "meeting_segments",
            "speakers",
            "settings",
            "jobs",
            "voices",
        ];
        if !known.contains(&table) {
            return Err(format!("unknown table {table}"));
        }
        self.connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|count| count as usize)
            .map_err(sql)
    }

    pub fn save_job(&self, job: &JobRecord<'_>) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO jobs (id, kind, state, stage, progress, input_hash, title, error, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) ON CONFLICT(id) DO UPDATE SET state = excluded.state, stage = excluded.stage, progress = excluded.progress, error = excluded.error, updated_at = excluded.updated_at",
                params![
                    job.id,
                    job.kind,
                    job.state,
                    job.stage,
                    job.progress,
                    job.input_hash,
                    job.title,
                    job.error,
                    job.created_at,
                    job.updated_at
                ],
            )
            .map(|_| ())
            .map_err(sql)
    }

    pub fn save_stage(&self, job_id: &str, name: &str, state: &str) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO job_stages (job_id, name, state, started_at, ended_at) VALUES (?1, ?2, ?3, ?4, CASE WHEN ?3 = 'done' THEN ?4 END) ON CONFLICT(job_id, name) DO UPDATE SET state = excluded.state, ended_at = excluded.ended_at",
                params![job_id, name, state, now()],
            )
            .map(|_| ())
            .map_err(sql)
    }

    pub fn completed_stages(&self, job_id: &str) -> Result<Vec<String>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT name FROM job_stages WHERE job_id = ?1 AND state = 'done' ORDER BY ended_at, rowid")
            .map_err(sql)?;
        let rows = statement
            .query_map([job_id], |row| row.get(0))
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }

    pub fn save_dub_project(&mut self, project: &mut DubProject) -> Result<(), String> {
        let stamp = now();
        if project.created_at == 0 {
            project.created_at = stamp;
        }
        project.updated_at = stamp;
        let settings = serde_json::json!({
            "from": project.from,
            "to": project.to,
            "targets": project.targets,
            "cast": project.cast,
        })
        .to_string();
        let transaction = self.connection.transaction().map_err(sql)?;
        transaction
            .execute(
                "INSERT INTO projects (id, kind, title, source_path, source_hash, created_at, updated_at, settings_json) VALUES (?1, 'dub', ?2, ?3, NULL, ?4, ?5, ?6) ON CONFLICT(id) DO UPDATE SET title = excluded.title, source_path = excluded.source_path, updated_at = excluded.updated_at, settings_json = excluded.settings_json",
                params![
                    project.id,
                    project.title,
                    project.source_path,
                    project.created_at,
                    project.updated_at,
                    settings
                ],
            )
            .map_err(sql)?;
        transaction
            .execute(
                "DELETE FROM segments WHERE project_id = ?1",
                params![project.id],
            )
            .map_err(sql)?;
        for (position, segment) in project.segments.iter_mut().enumerate() {
            segment.position = position;
            if segment.id.is_empty() {
                segment.id = format!("{}-{position}", project.id);
            }
            if segment.revisions.is_empty() {
                if let Some(original) = segment.original_text.clone() {
                    segment.revisions.push(original);
                }
            }
            segment.original_text = segment.revisions.first().cloned();
            transaction
                .execute(
                    "INSERT INTO segments (id, project_id, position, start_ms, end_ms, speaker, voice_id, source_text, original_text, revisions_json, pace, target_text, state, qc_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, NULL)",
                    params![
                        segment.id,
                        project.id,
                        segment.position as i64,
                        segment.start_ms as i64,
                        segment.end_ms as i64,
                        segment.speaker,
                        segment.voice_id,
                        segment.source_text,
                        segment.original_text,
                        serde_json::to_string(&segment.revisions)
                            .map_err(|error| error.to_string())?,
                        segment.pace,
                        segment.target_text,
                        segment.state
                    ],
                )
                .map_err(sql)?;
            for (language, text) in &segment.translations {
                transaction
                    .execute(
                        "INSERT INTO segment_translations (segment_id, language, text, state) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(segment_id, language) DO UPDATE SET text = excluded.text, state = excluded.state",
                        params![segment.id, language, text, segment.state],
                    )
                    .map_err(sql)?;
            }
        }
        transaction.commit().map_err(sql)
    }

    pub fn list_dub_projects(&self) -> Result<Vec<DubSummary>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT p.id, p.title, p.source_path, p.created_at, p.updated_at, p.settings_json, (SELECT COUNT(*) FROM segments s WHERE s.project_id = p.id) FROM projects p WHERE p.kind = 'dub' ORDER BY p.updated_at DESC, p.created_at DESC",
            )
            .map_err(sql)?;
        let rows = statement
            .query_map([], |row| {
                let settings_text: Option<String> = row.get(5)?;
                let settings = settings_text
                    .as_deref()
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok());
                let invalid = settings_text.is_some() && settings.is_none();
                Ok(DubSummary {
                    id: row.get(0)?,
                    title: row
                        .get::<_, Option<String>>(1)?
                        .unwrap_or_else(|| "Untitled project".to_string()),
                    source_path: row.get(2)?,
                    from: settings
                        .as_ref()
                        .and_then(|value| value.get("from"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("pt")
                        .to_string(),
                    to: settings
                        .as_ref()
                        .and_then(|value| value.get("to"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("en")
                        .to_string(),
                    created_at: row.get::<_, Option<i64>>(3)?.unwrap_or_default(),
                    updated_at: row.get::<_, Option<i64>>(4)?.unwrap_or_default(),
                    segments: row.get::<_, i64>(6)?.max(0) as usize,
                    invalid,
                    error: invalid.then(|| "the project settings are not valid JSON".to_string()),
                })
            })
            .map_err(sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql)
    }

    pub fn delete_dub_project(&self, id: &str) -> Result<bool, String> {
        self.connection
            .execute(
                "DELETE FROM projects WHERE id = ?1 AND kind = 'dub'",
                params![id],
            )
            .map(|changed| changed > 0)
            .map_err(sql)
    }

    pub fn load_dub_project(&self, id: &str) -> Result<Option<DubProject>, String> {
        let project = self
            .connection
            .query_row(
                "SELECT id, title, source_path, created_at, updated_at, settings_json FROM projects WHERE id = ?1 AND kind = 'dub'",
                params![id],
                |row| {
                    let settings: serde_json::Value = row
                        .get::<_, Option<String>>(5)?
                        .and_then(|text| serde_json::from_str(&text).ok())
                        .unwrap_or_default();
                    Ok(DubProject {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        source_path: row.get(2)?,
                        from: settings
                            .get("from")
                            .and_then(|value| value.as_str())
                            .unwrap_or("pt")
                            .to_string(),
                        to: settings
                            .get("to")
                            .and_then(|value| value.as_str())
                            .unwrap_or("en")
                            .to_string(),
                        created_at: row.get::<_, Option<i64>>(3)?.unwrap_or_default(),
                        updated_at: row.get::<_, Option<i64>>(4)?.unwrap_or_default(),
                        targets: settings
                            .get("targets")
                            .and_then(|value| value.as_array())
                            .map(|items| {
                                items
                                    .iter()
                                    .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        cast: settings
                            .get("cast")
                            .and_then(|value| serde_json::from_value(value.clone()).ok())
                            .unwrap_or_default(),
                        segments: Vec::new(),
                    })
                },
            )
            .optional()
            .map_err(sql)?;
        let Some(mut project) = project else {
            return Ok(None);
        };
        let mut statement = self
            .connection
            .prepare("SELECT id, position, start_ms, end_ms, speaker, voice_id, source_text, original_text, revisions_json, pace, target_text, state FROM segments WHERE project_id = ?1 ORDER BY position")
            .map_err(sql)?;
        let rows = statement
            .query_map(params![id], |row| {
                Ok(DubSegment {
                    id: row.get(0)?,
                    position: row.get::<_, i64>(1)?.max(0) as usize,
                    start_ms: row.get::<_, i64>(2)?.max(0) as u64,
                    end_ms: row.get::<_, i64>(3)?.max(0) as u64,
                    speaker: row.get(4)?,
                    voice_id: row.get(5)?,
                    translations: BTreeMap::new(),
                    source_text: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                    original_text: row.get(7)?,
                    revisions: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|text| serde_json::from_str(&text).ok())
                        .unwrap_or_default(),
                    pace: row.get(9)?,
                    target_text: row.get(10)?,
                    state: row
                        .get::<_, Option<String>>(11)?
                        .unwrap_or_else(|| "imported".to_string()),
                })
            })
            .map_err(sql)?;
        project.segments = rows.collect::<Result<Vec<_>, _>>().map_err(sql)?;
        for segment in &mut project.segments {
            if segment.revisions.is_empty() {
                if let Some(original) = segment.original_text.clone() {
                    segment.revisions.push(original);
                }
            }
            segment.original_text = segment.revisions.first().cloned();
        }
        if project.targets.is_empty() {
            project.targets.push(project.to.clone());
        }
        let mut translations = self
            .connection
            .prepare("SELECT t.segment_id, t.language, t.text FROM segment_translations t JOIN segments s ON s.id = t.segment_id WHERE s.project_id = ?1")
            .map_err(sql)?;
        let rows = translations
            .query_map(params![id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(sql)?;
        for row in rows {
            let (segment_id, language, text) = row.map_err(sql)?;
            if let Some(segment) = project
                .segments
                .iter_mut()
                .find(|segment| segment.id == segment_id)
            {
                segment.translations.insert(language, text);
            }
        }
        Ok(Some(project))
    }
}

fn sql(error: rusqlite::Error) -> String {
    error.to_string()
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::Entry;
    use crate::meeting::{Meeting, Name, Segment};

    fn scratch() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        std::env::temp_dir().join(format!("parlo-store-{}-{nanos}", std::process::id()))
    }

    fn legacy_folder() -> PathBuf {
        let root = scratch();
        fs::create_dir_all(root.join("meetings")).unwrap();
        let first = Entry {
            at: 1_700_000_000,
            text: "bom dia".to_string(),
            translation: "good morning".to_string(),
            from: "pt".to_string(),
            to: "en".to_string(),
            app: "editor".to_string(),
            mode: "dictation".to_string(),
            id: "h1".to_string(),
            favourite: false,
            seconds: 1.5,
        };
        let second = Entry {
            id: String::new(),
            ..first.clone()
        };
        fs::write(
            root.join("history.jsonl"),
            format!(
                "{}\n{}\n",
                serde_json::to_string(&first).unwrap(),
                serde_json::to_string(&second).unwrap()
            ),
        )
        .unwrap();
        let meeting = Meeting {
            id: "m1".to_string(),
            title: "Launch".to_string(),
            started_at: 1_700_000_100,
            speaker_hint: None,
            segments: vec![Segment {
                at_ms: 1_500,
                speaker: "speaker-1".to_string(),
                text: "we ship on Friday".to_string(),
                name: Some(Name {
                    name: "Ana".to_string(),
                    evidence: "said so".to_string(),
                    inferred: false,
                    applied_at: 1_700_000_200,
                }),
            }],
        };
        fs::write(
            root.join("meetings/m1.json"),
            serde_json::to_string(&meeting).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join("config.json"),
            r#"{"port":9000,"name":"Ana","history_days":30}"#,
        )
        .unwrap();
        root
    }

    #[test]
    fn imports_a_legacy_folder_once() {
        let root = legacy_folder();
        let mut store = Store::open_memory().unwrap();

        let imported = store.import_legacy(&root).unwrap();
        assert_eq!(
            imported,
            Imported {
                history: 2,
                meetings: 1,
                settings: 3,
                already: false,
            }
        );
        assert_eq!(store.count("history").unwrap(), 2);
        assert_eq!(store.count("meetings").unwrap(), 1);
        assert_eq!(store.count("meeting_segments").unwrap(), 1);
        assert_eq!(store.count("speakers").unwrap(), 1);
        assert_eq!(store.count("settings").unwrap(), 3);

        let again = store.import_legacy(&root).unwrap();
        assert!(again.already);
        assert_eq!(store.count("history").unwrap(), 2);
        assert_eq!(store.count("meetings").unwrap(), 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_meeting_keeps_its_speaker_name_and_text() {
        let root = legacy_folder();
        let mut store = Store::open_memory().unwrap();
        store.import_legacy(&root).unwrap();

        let text: String = store
            .connection
            .query_row(
                "SELECT text FROM meeting_segments WHERE meeting_id = 'm1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(text, "we ship on Friday");
        let speaker: String = store
            .connection
            .query_row(
                "SELECT name FROM speakers WHERE meeting_id = 'm1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(speaker, "Ana");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn only_completed_stages_are_returned() {
        let store = Store::open_memory().unwrap();
        store
            .save_job(&JobRecord {
                id: "j1",
                kind: "dub",
                state: "running",
                stage: "transcribe",
                progress: 0.1,
                input_hash: "abc",
                title: "A file",
                error: None,
                created_at: 10,
                updated_at: 11,
            })
            .unwrap();
        store.save_stage("j1", "transcribe", "done").unwrap();
        store.save_stage("j1", "translate", "running").unwrap();
        assert_eq!(
            store.completed_stages("j1").unwrap(),
            vec!["transcribe".to_string()]
        );
        store.save_stage("j1", "translate", "done").unwrap();
        assert_eq!(store.completed_stages("j1").unwrap().len(), 2);
    }

    #[test]
    fn a_job_is_persisted_and_updated() {
        let store = Store::open_memory().unwrap();
        let mut job = JobRecord {
            id: "j1",
            kind: "dub",
            state: "running",
            stage: "transcribe",
            progress: 0.25,
            input_hash: "abc",
            title: "A file",
            error: None,
            created_at: 10,
            updated_at: 11,
        };
        store.save_job(&job).unwrap();
        assert_eq!(store.count("jobs").unwrap(), 1);
        job.state = "done";
        job.progress = 1.0;
        job.updated_at = 12;
        store.save_job(&job).unwrap();
        assert_eq!(store.count("jobs").unwrap(), 1);
    }

    #[test]
    fn a_dub_project_round_trips_with_its_segments() {
        let mut store = Store::open_memory().unwrap();
        let mut project = DubProject::new(
            "p1".to_string(),
            "Launch".to_string(),
            "pt",
            "en",
            Some("launch.srt".to_string()),
        );
        project
            .cast
            .insert("Ana".to_string(), "voice-en".to_string());
        project.segments.push(DubSegment {
            id: "p1-0".to_string(),
            position: 0,
            start_ms: 1_250,
            end_ms: 3_500,
            speaker: Some("Ana".to_string()),
            voice_id: Some("voice-de".to_string()),
            translations: BTreeMap::from([
                ("en".to_string(), "Good morning".to_string()),
                ("de".to_string(), "Guten Morgen".to_string()),
            ]),
            source_text: "Bom dia".to_string(),
            original_text: Some("Bom dia antes".to_string()),
            revisions: Vec::new(),
            pace: Some(0.9),
            target_text: Some("Good morning".to_string()),
            state: "imported".to_string(),
        });
        project.segments.push(DubSegment {
            id: "p1-1".to_string(),
            position: 1,
            start_ms: 4_000,
            end_ms: 5_000,
            speaker: None,
            voice_id: None,
            translations: BTreeMap::new(),
            source_text: "Até logo".to_string(),
            original_text: None,
            revisions: Vec::new(),
            pace: None,
            target_text: None,
            state: "imported".to_string(),
        });
        project.targets.push("de".to_string());
        store.save_dub_project(&mut project).unwrap();

        let loaded = store.load_dub_project("p1").unwrap().unwrap();
        assert_eq!(loaded.id, "p1");
        assert_eq!(loaded.title, "Launch");
        assert_eq!(loaded.from, "pt");
        assert_eq!(loaded.to, "en");
        assert_eq!(loaded.targets, ["en", "de"]);
        assert_eq!(loaded.segments, project.segments);
        assert_eq!(loaded.segments[0].speaker.as_deref(), Some("Ana"));
        assert_eq!(loaded.segments[0].voice_id.as_deref(), Some("voice-de"));
        assert_eq!(
            loaded.segments[0].original_text.as_deref(),
            Some("Bom dia antes")
        );
        assert_eq!(loaded.segments[0].pace, Some(0.9));
        assert_eq!(
            loaded.segments[0]
                .translations
                .get("de")
                .map(String::as_str),
            Some("Guten Morgen")
        );
        assert_eq!(loaded.cast.get("Ana").map(String::as_str), Some("voice-en"));
        assert_eq!(loaded.segments[0].start_ms, 1_250);
        assert_eq!(loaded.segments[1].target_text, None);
        assert!(store.load_dub_project("missing").unwrap().is_none());
    }

    #[test]
    fn a_segment_saved_before_revisions_existed_still_reads_the_same() {
        let store = Store::open_memory().unwrap();
        store
            .connection
            .execute(
                "INSERT INTO projects (id, kind, title, created_at, updated_at, settings_json) VALUES ('p1', 'dub', 'Launch', 1, 1, '{\"from\":\"pt\",\"to\":\"en\"}')",
                [],
            )
            .unwrap();
        store
            .connection
            .execute(
                "INSERT INTO segments (id, project_id, position, start_ms, end_ms, source_text, original_text, state) VALUES ('p1-0', 'p1', 0, 0, 1000, 'Bom dia a todos', 'Bom dia', 'edited')",
                [],
            )
            .unwrap();
        let loaded = store.load_dub_project("p1").unwrap().unwrap();
        let segment = &loaded.segments[0];
        assert_eq!(segment.source_text, "Bom dia a todos");
        assert_eq!(segment.original_text.as_deref(), Some("Bom dia"));
        assert_eq!(segment.revisions, vec!["Bom dia".to_string()]);
    }

    #[test]
    fn the_revisions_column_is_added_to_an_older_segments_table() {
        let store = Store::open_memory().unwrap();
        let count: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('segments') WHERE name = 'revisions_json'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn projects_are_listed_and_corrupt_settings_are_marked() {
        let mut store = Store::open_memory().unwrap();
        let mut project = DubProject::new(
            "p1".to_string(),
            "Launch".to_string(),
            "pt",
            "en",
            Some("launch.srt".to_string()),
        );
        store.save_dub_project(&mut project).unwrap();
        let listed = store.list_dub_projects().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].title, "Launch");
        assert!(!listed[0].invalid);

        store
            .connection
            .execute("UPDATE projects SET settings_json = '{broken'", [])
            .unwrap();
        let listed = store.list_dub_projects().unwrap();
        assert!(listed[0].invalid);
        assert!(listed[0].error.is_some());
        assert!(store.delete_dub_project("p1").unwrap());
        assert!(store.list_dub_projects().unwrap().is_empty());
    }

    #[test]
    fn an_old_segment_table_gains_the_new_columns() {
        let root = scratch();
        fs::create_dir_all(&root).unwrap();
        let path = root.join("parlo.db");
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE segments (id TEXT PRIMARY KEY, project_id TEXT, position INTEGER NOT NULL, start_ms INTEGER, end_ms INTEGER, speaker TEXT, voice_id TEXT, source_text TEXT, target_text TEXT, state TEXT, qc_json TEXT)",
                )
                .unwrap();
        }
        let store = Store::open(&path).unwrap();
        let present: i64 = store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('segments') WHERE name IN ('original_text', 'pace')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(present, 2);
        let _ = fs::remove_dir_all(&root);
    }
}
