use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
mod schema;
#[cfg(test)]
mod tests;
mod types;

pub use types::*;

use schema::SCHEMA;

pub struct Store {
    connection: Connection,
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

pub(super) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default()
}
