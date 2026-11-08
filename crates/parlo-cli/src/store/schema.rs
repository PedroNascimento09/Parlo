pub(super) const SCHEMA: &str = r#"
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
