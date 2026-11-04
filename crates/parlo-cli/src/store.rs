use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

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
    target_text TEXT,
    state TEXT,
    qc_json TEXT
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
        self.connection.execute_batch(SCHEMA).map_err(sql)
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
}
