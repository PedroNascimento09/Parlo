use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
        latency_ms: 0,
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
