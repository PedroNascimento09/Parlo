use super::*;
use std::io::Write;
use std::net::TcpStream;

use parlo_core::audio::Frame;
use parlo_core::engine::{Engine, EngineOutput, StreamPiece};

#[test]
fn the_library_payloads_carry_the_fields_the_panels_read() {
    let meeting = crate::meeting::Meeting {
        id: "m1".to_string(),
        title: "Review".to_string(),
        started_at: 1,
        segments: vec![crate::meeting::Segment {
            at_ms: 0,
            speaker: "Speaker 1".to_string(),
            text: "We ship on Friday.".to_string(),
            name: None,
        }],
        speaker_hint: None,
    };
    let payload = meeting_json(meeting);
    for key in [
        "id",
        "title",
        "started_at",
        "speakers",
        "segments",
        "actions",
        "decisions",
    ] {
        assert!(payload.get(key).is_some(), "meetings lost {key}");
    }
    let segment = &payload["segments"][0];
    for key in ["at_ms", "speaker", "spoken_by", "text", "named"] {
        assert!(segment.get(key).is_some(), "a meeting line lost {key}");
    }

    let batch = crate::batch::Batch {
        id: "b1".to_string(),
        from: "en".to_string(),
        targets: vec!["pt".to_string()],
        timing: "natural".to_string(),
        fastest: 1.0,
        slowest: 2.0,
        keep_background: false,
        state: crate::batch::RUNNING.to_string(),
        created_at: 1,
        updated_at: 1,
        story: None,
        items: vec![crate::batch::Item {
            id: "i1".to_string(),
            source: "a.wav".to_string(),
            title: "a.wav".to_string(),
            to: "pt".to_string(),
            state: crate::batch::DONE.to_string(),
            stage: "ready".to_string(),
            progress: 1.0,
            error: None,
            out: Some("a.pt.wav".to_string()),
            srt: None,
            seconds: 1.0,
            lines: 1,
            started_at: None,
            ended_at: None,
        }],
    };
    let payload = serde_json::to_value(&batch).expect("a batch payload");
    for key in ["id", "state", "from", "targets", "story", "items"] {
        assert!(payload.get(key).is_some(), "batches lost {key}");
    }
    for key in ["id", "title", "to", "state", "error"] {
        assert!(
            payload["items"][0].get(key).is_some(),
            "a batch item lost {key}"
        );
    }

    let story = crate::story::Story {
        id: "s1".to_string(),
        title: "Coast".to_string(),
        author: None,
        year: None,
        series: None,
        comment: None,
        language: "pt".to_string(),
        state: crate::story::DRAFT.to_string(),
        batch: None,
        error: None,
        rendered: 0,
        cached: 0,
        failed: 0,
        created_at: 1,
        updated_at: 1,
        characters: vec![crate::story::Character {
            id: "c1".to_string(),
            name: "Keeper".to_string(),
            voice: "pt_PT-tugão-medium".to_string(),
        }],
        chapters: vec![crate::story::Chapter {
            id: "ch1".to_string(),
            title: "Chapter 1".to_string(),
            lines: Vec::new(),
        }],
    };
    let payload = serde_json::to_value(&story).expect("a story payload");
    for key in ["id", "title", "state", "characters", "chapters"] {
        assert!(payload.get(key).is_some(), "stories lost {key}");
    }

    let app = include_str!("../../web/app.js");
    for token in [
        "meeting.segments",
        "meeting.speakers",
        "meeting.decisions",
        "meeting.actions",
        "batch.targets",
        "story.characters",
        "project.exports",
        "entry.translation",
    ] {
        assert!(app.contains(token), "the panels no longer read {token}");
    }
}

#[test]
fn an_empty_write_choice_follows_the_pair() {
    let table: serde_json::Value =
        serde_json::from_str(include_str!("../../web/languages.json"))
            .expect("a language table");
    for pair in table["pairs"].as_array().expect("language pairs") {
        let from = pair[0].as_str().expect("a source language");
        let to = pair[1].as_str().expect("a target language");
        assert!(
            crate::config::check_language(to),
            "{to} would be refused with a 400"
        );
        assert_eq!(dictation_target("", from, to, true), to);
        assert_eq!(dictation_target("", from, to, false), from);
        assert_eq!(dictation_target(to, from, to, true), to);
    }
}

struct RecordingEngine {
    heard: Arc<Mutex<Vec<u8>>>,
}

impl Engine for RecordingEngine {
    fn id(&self) -> &str {
        "recording"
    }

    fn target_languages(&self) -> Vec<String> {
        Vec::new()
    }

    fn latency_ms(&self) -> u32 {
        1
    }

    fn process(&mut self, _input: &Frame) -> Result<EngineOutput, EngineError> {
        Ok(EngineOutput::empty())
    }

    fn dictate(&mut self, pcm16: &[u8]) -> Result<String, EngineError> {
        self.heard
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend_from_slice(pcm16);
        Ok("ola mundo".to_string())
    }

    fn write(
        &mut self,
        pcm16: &[u8],
        _target: Option<&str>,
        _clean: Option<&str>,
    ) -> Result<(String, String), EngineError> {
        self.heard
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .extend_from_slice(pcm16);
        Ok(("ola".to_string(), "hello".to_string()))
    }

    fn speak(&mut self, _text: &str) -> Result<(Vec<u8>, u32), EngineError> {
        let pcm: Vec<u8> = (0..1600i16)
            .flat_map(|sample| (sample / 2).to_le_bytes())
            .collect();
        Ok((pcm, 16_000))
    }
}

#[derive(Default)]
struct SpeakingEngine {
    spoken: Arc<Mutex<Vec<String>>>,
}

impl Engine for SpeakingEngine {
    fn id(&self) -> &str {
        "speaking"
    }

    fn target_languages(&self) -> Vec<String> {
        Vec::new()
    }

    fn latency_ms(&self) -> u32 {
        1
    }

    fn process(&mut self, _input: &Frame) -> Result<EngineOutput, EngineError> {
        Ok(EngineOutput::empty())
    }

    fn speak(&mut self, text: &str) -> Result<(Vec<u8>, u32), EngineError> {
        self.spoken
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(text.to_string());
        let pcm: Vec<u8> = (0..1600i16)
            .flat_map(|sample| (sample / 2).to_le_bytes())
            .collect();
        Ok((pcm, 16_000))
    }
}

#[derive(Default)]
struct StreamingEngine {
    calls: usize,
}

impl Engine for StreamingEngine {
    fn id(&self) -> &str {
        "streaming"
    }

    fn target_languages(&self) -> Vec<String> {
        Vec::new()
    }

    fn latency_ms(&self) -> u32 {
        1
    }

    fn process(&mut self, _input: &Frame) -> Result<EngineOutput, EngineError> {
        Ok(EngineOutput::empty())
    }

    fn stream(&mut self, samples: &[f32]) -> Result<Vec<StreamPiece>, EngineError> {
        self.calls += 1;
        let (state, text) = if self.calls == 1 {
            ("partial", "bom")
        } else {
            ("stable", "bom dia")
        };
        Ok(vec![StreamPiece {
            state: state.to_string(),
            text: text.to_string(),
            audio_ms: samples.len() as u64 * 1000 / 16_000,
            latency_ms: 1,
        }])
    }

    fn close_stream(&mut self) -> Result<Vec<StreamPiece>, EngineError> {
        Ok(vec![StreamPiece {
            state: "final".to_string(),
            text: "bom dia.".to_string(),
            audio_ms: 200,
            latency_ms: 1,
        }])
    }
}

impl Session {
    fn with_engine(engine: Box<dyn Engine>, from: &str, to: &str) -> Self {
        let format = AudioFormat::mono(16_000).unwrap();
        let pipeline = Pipeline::new(PipelineConfig::new(format, format), engine).unwrap();
        Self {
            pipeline: Mutex::new(pipeline),
            settings: RwLock::new(Chosen {
                engine: "recording".to_string(),
                stt: "parakeet".to_string(),
                from: from.to_string(),
                to: to.to_string(),
                write: String::new(),
                shortcut: String::new(),
                hands_free: String::new(),
                captions: String::new(),
                undo: String::new(),
            }),
            target: Mutex::new(None),
            busy: AtomicBool::new(false),
            known: Mutex::new(None),
        }
    }
}

fn ask_raw(port: u16, request: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream.write_all(request).unwrap();
    let mut answer = Vec::new();
    stream.read_to_end(&mut answer).unwrap();
    answer
}

fn ask_bytes(port: u16, request: &[u8]) -> String {
    String::from_utf8(ask_raw(port, request)).unwrap()
}

fn ask(port: u16, request: &str) -> String {
    ask_bytes(port, request.as_bytes())
}

#[test]
fn the_main_routes_answer_on_a_real_socket() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let session = Arc::new(
        Session::open("echo".into(), "parakeet".into(), "pt".into(), "en".into()).unwrap(),
    );
    let accepting = Arc::new(AtomicBool::new(true));
    let stop = Arc::clone(&accepting);
    std::thread::spawn(move || {
        listen(listener, "t0ken".into(), port, session, accepting, None)
    });

    let health = ask(port, "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    assert!(health.starts_with("HTTP/1.1 200 OK"));
    assert!(health.contains("\"status\":\"ok\""));

    let state = ask(port, "GET /state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    assert!(state.starts_with("HTTP/1.1 200 OK"));
    assert!(state.contains("\"state\":"));

    let models = ask(port, "GET /models HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    assert!(models.starts_with("HTTP/1.1 200 OK"));

    let discovery = ask(
        port,
        "GET /.well-known/parlo HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert!(discovery.starts_with("HTTP/1.1 200 OK"));
    assert!(discovery.contains("\"name\":\"Parlo\""));
    assert!(discovery.contains("\"local\":true"));

    let listed = ask(port, "GET /v1/models HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    assert!(listed.starts_with("HTTP/1.1 200 OK"));
    assert!(listed.contains("\"object\":\"list\""));
    assert!(listed.contains("\"owned_by\":\"parlo\""));
    assert!(listed.contains("\"licence\":"));
    assert!(listed.contains("\"languages\":"));

    let foreign = ask(port, "GET /state HTTP/1.1\r\nHost: evil.example\r\n\r\n");
    assert!(foreign.starts_with("HTTP/1.1 400"));

    for request in [
        "GET /v1/jobs HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        "GET /v1/jobs/missing HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        "GET /v1/jobs/missing/events HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        "POST /v1/jobs/missing/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        "POST /v1/jobs HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 2\r\n\r\n{}",
    ] {
        let answer = ask(port, request);
        assert!(answer.starts_with("HTTP/1.1 401"), "{answer}");
    }

    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}

#[test]
fn a_session_can_be_shared_between_threads() {
    fn shareable<T: Send + Sync>() {}
    shareable::<Session>();
}

#[test]
fn a_wav_length_is_read_from_its_header() {
    let mut bytes = vec![0u8; 44];
    bytes[24..28].copy_from_slice(&16000u32.to_le_bytes());
    bytes[40..44].copy_from_slice(&(16000u32 * 2 * 5).to_le_bytes());
    assert!((wav_seconds(&bytes) - 5.0).abs() < 0.01);
}

#[test]
fn accepts_a_bearer_token() {
    let headers = vec![("authorization".to_string(), "Bearer abc".to_string())];
    assert!(authorised(&headers, "abc"));
    assert!(!authorised(&headers, "abd"));
}

#[test]
fn accepts_the_short_header() {
    let headers = vec![("x-parlo-token".to_string(), "abc".to_string())];
    assert!(authorised(&headers, "abc"));
}

#[test]
fn rejects_a_missing_token() {
    assert!(!authorised(&[], "abc"));
}

#[test]
fn only_the_local_host_is_accepted() {
    let local = vec![("host".to_string(), "127.0.0.1:8787".to_string())];
    assert!(host_is_local(&local));
    let named = vec![("host".to_string(), "localhost:8787".to_string())];
    assert!(host_is_local(&named));
    let rebinding = vec![("host".to_string(), "attacker.example:8787".to_string())];
    assert!(!host_is_local(&rebinding));
    assert!(!host_is_local(&[]));
}

#[test]
fn cross_origin_posts_are_refused() {
    assert!(origin_allowed(&[], 8787));
    let same = vec![("origin".to_string(), "http://127.0.0.1:8787".to_string())];
    assert!(origin_allowed(&same, 8787));
    let other = vec![("origin".to_string(), "https://evil.example".to_string())];
    assert!(!origin_allowed(&other, 8787));
}

#[test]
fn the_transcript_header_is_base64_and_absent_when_empty() {
    assert_eq!(transcript_header(""), "");
    let header = transcript_header("olá mundo");
    assert!(header.starts_with("X-Parlo-Transcript: b2zDoSBtdW5kbw=="));
    assert!(header.contains("Access-Control-Expose-Headers"));
}

#[test]
fn dictation_follows_the_pair_until_a_language_is_chosen() {
    assert_eq!(dictation_target("", "en", "pt", true), "pt");
    assert_eq!(dictation_target("", "en", "pt", false), "en");
    assert_eq!(dictation_target("en", "en", "pt", true), "en");
    assert_eq!(dictation_target("de", "en", "pt", true), "de");
}

#[test]
fn a_restyle_needs_an_id_and_a_known_style() {
    let good = serde_json::json!({ "id": "h1", "style": "Casual" });
    assert_eq!(
        restyle_fields(&good).unwrap(),
        ("h1".to_string(), "casual".to_string())
    );
    let nameless = serde_json::json!({ "style": "casual" });
    assert!(restyle_fields(&nameless)
        .unwrap_err()
        .contains("named by its id"));
    let odd = serde_json::json!({ "id": "h1", "style": "shouty" });
    assert!(restyle_fields(&odd)
        .unwrap_err()
        .contains("formal, casual or excited"));
}

#[test]
fn a_search_query_is_read_from_the_address() {
    assert_eq!(parameter("q=bom+dia", "q"), "bom dia");
    assert_eq!(parameter("q=bom%20dia", "q"), "bom dia");
    assert_eq!(parameter("limit=5", "q"), "");
}

#[test]
fn json_strings_survive_windows_paths_and_quotes() {
    assert_eq!(
        json_string("C:\\temp\\parlo.exe said \"no\""),
        "C:\\\\temp\\\\parlo.exe said \\\"no\\\""
    );
}

#[test]
fn the_app_page_carries_the_running_version() {
    let page = INDEX.replace("__PARLO_VERSION__", VERSION);
    assert!(page.contains(&format!("content=\"{VERSION}\"")));
    assert!(!page.contains("__PARLO_VERSION__"));
    assert!(page.contains("/app.css") && page.contains("/app.js"));
}

#[test]
fn the_island_page_dictates_and_carries_the_version() {
    let page = ISLAND.replace("__PARLO_VERSION__", VERSION);
    assert!(page.contains("/dictate"));
    assert!(page.contains("/update"));
    assert!(page.contains(&format!("content=\"{VERSION}\"")));
    assert!(!page.contains("__PARLO_VERSION__"));
}

#[test]
fn dub_uploads_get_their_own_size_limit() {
    assert_eq!(body_limit("POST /dub HTTP/1.1"), MAX_DUB_BODY);
    assert_eq!(body_limit("POST /dub/import HTTP/1.1"), MAX_BODY);
    assert_eq!(body_limit("POST /translate HTTP/1.1"), MAX_BODY);
    assert_eq!(body_limit(""), MAX_BODY);
}

#[test]
fn edited_segments_survive_the_store_and_invalidate_the_translation() {
    let mut store = crate::store::Store::open_memory().unwrap();
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: Some("Good morning".to_string()),
        state: "dubbed".to_string(),
    });
    store.save_dub_project(&mut project).unwrap();

    apply_project_edits(
        &mut project,
        &[SegmentEdit {
            id: "p1-0".to_string(),
            start_ms: 500,
            end_ms: 2_750,
            speaker: Some("Beatriz".to_string()),
            voice_id: None,
            pace: None,
            source_text: "Bom dia a todos".to_string(),
        }],
        &std::collections::BTreeMap::new(),
    )
    .unwrap();
    store.save_dub_project(&mut project).unwrap();
    let loaded = store.load_dub_project("p1").unwrap().unwrap();

    assert_eq!(loaded.segments[0].start_ms, 500);
    assert_eq!(loaded.segments[0].end_ms, 2_750);
    assert_eq!(loaded.segments[0].speaker.as_deref(), Some("Beatriz"));
    assert_eq!(loaded.segments[0].source_text, "Bom dia a todos");
    assert_eq!(loaded.segments[0].original_text.as_deref(), Some("Bom dia"));
    assert!(loaded.segments[0].target_text.is_none());
    assert_eq!(loaded.segments[0].state, "edited");

    apply_project_edits(
        &mut project,
        &[SegmentEdit {
            id: "p1-0".to_string(),
            start_ms: 500,
            end_ms: 2_750,
            speaker: Some("Beatriz".to_string()),
            voice_id: None,
            pace: None,
            source_text: "Bom dia a todos, bom ano".to_string(),
        }],
        &std::collections::BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(
        project.segments[0].revisions,
        ["Bom dia", "Bom dia a todos"]
    );
    assert_eq!(project.segments[0].source_text, "Bom dia a todos, bom ano");
    assert_eq!(
        project.segments[0].original_text.as_deref(),
        Some("Bom dia")
    );

    apply_project_edits(
        &mut project,
        &[SegmentEdit {
            id: "p1-0".to_string(),
            start_ms: 500,
            end_ms: 2_750,
            speaker: Some("Beatriz".to_string()),
            voice_id: None,
            pace: None,
            source_text: "Bom dia".to_string(),
        }],
        &std::collections::BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(
        project.segments[0].revisions,
        ["Bom dia", "Bom dia a todos", "Bom dia a todos, bom ano"]
    );
    assert_eq!(project.segments[0].source_text, "Bom dia");
    assert_eq!(
        project.segments[0].original_text.as_deref(),
        Some("Bom dia")
    );

    let published = serde_json::to_value(&project).unwrap();
    assert_eq!(published["segments"][0]["source_text"], "Bom dia");
    assert_eq!(published["segments"][0]["original_text"], "Bom dia");
    assert_eq!(
        published["segments"][0]["revisions"]
            .as_array()
            .map(|versions| versions.len()),
        Some(3)
    );

    store.save_dub_project(&mut project).unwrap();
    let reloaded = store.load_dub_project("p1").unwrap().unwrap();
    assert_eq!(
        reloaded.segments[0].revisions,
        project.segments[0].revisions
    );
    assert_eq!(reloaded.segments[0].source_text, "Bom dia");
}

#[test]
fn cue_documents_keep_segment_identity_and_timing() {
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: Some(1.1),
        target_text: None,
        state: "imported".to_string(),
    });

    let json = cue_document(&project).unwrap();
    assert!(json.contains("\"pace\":1.1"), "{json}");
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(document["cues"][0]["id"], "p1-0");
    assert_eq!(document["cues"][0]["pace"], 1.1);
    assert_eq!(document["cues"][0]["start_ms"], 1_250);
    assert_eq!(document["cues"][0]["end_ms"], 3_500);
    assert_eq!(document["cues"][0]["source"], "Bom dia");
}

#[test]
fn casting_and_overrides_decide_the_voice_each_line_uses() {
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project
        .cast
        .insert("Ana".to_string(), "voice-en".to_string());
    project
        .cast
        .insert("Bob".to_string(), "voice-de".to_string());
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });
    project.segments.push(crate::store::DubSegment {
        id: "p1-1".to_string(),
        position: 1,
        start_ms: 4_000,
        end_ms: 6_000,
        speaker: Some("Bob".to_string()),
        voice_id: Some("voice-fr".to_string()),
        translations: std::collections::BTreeMap::new(),
        source_text: "Até logo".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });
    project.segments.push(crate::store::DubSegment {
        id: "p1-2".to_string(),
        position: 2,
        start_ms: 7_000,
        end_ms: 8_000,
        speaker: None,
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Sem voz".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });

    let document: serde_json::Value =
        serde_json::from_str(&cue_document(&project).unwrap()).unwrap();

    assert_eq!(document["cues"][0]["voice"], "voice-en");
    assert_eq!(document["cues"][1]["voice"], "voice-fr");
    assert!(document["cues"][2]["voice"].is_null());
}

#[test]
fn project_edits_keep_the_cast_and_refuse_an_unknown_voice() {
    let mut store = crate::store::Store::open_memory().unwrap();
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: Some("Good morning".to_string()),
        state: "dubbed".to_string(),
    });
    store.save_dub_project(&mut project).unwrap();
    let edit = [SegmentEdit {
        id: "p1-0".to_string(),
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: Some("voice-fr".to_string()),
        pace: Some(0.9),
        source_text: "Bom dia".to_string(),
    }];
    let mut cast = std::collections::BTreeMap::new();
    cast.insert("Ana".to_string(), "voice-en".to_string());

    apply_project_edits(&mut project, &edit, &cast).unwrap();
    store.save_dub_project(&mut project).unwrap();
    let loaded = store.load_dub_project("p1").unwrap().unwrap();

    assert_eq!(loaded.cast.get("Ana").map(String::as_str), Some("voice-en"));
    assert_eq!(loaded.segments[0].voice_id.as_deref(), Some("voice-fr"));
    assert_eq!(loaded.segments[0].pace, Some(0.9));
    assert_eq!(
        loaded.segments[0].target_text.as_deref(),
        Some("Good morning")
    );
    assert_eq!(loaded.segments[0].state, "edited");

    let mut wrong = std::collections::BTreeMap::new();
    wrong.insert("Ana".to_string(), "voice-klingon".to_string());
    let refused = apply_project_edits(&mut project, &edit, &wrong).unwrap_err();
    assert!(refused.contains("is not a voice"), "{refused}");

    let too_fast = [SegmentEdit {
        id: "p1-0".to_string(),
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        pace: Some(1.5),
        source_text: "Bom dia".to_string(),
    }];
    let refused = apply_project_edits(&mut project, &too_fast, &cast).unwrap_err();
    assert!(refused.contains("pace"), "{refused}");
}

#[test]
fn a_finished_dub_updates_matching_project_segments() {
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });
    let lines = vec![
        dub::Line {
            start: 1.25,
            end: 2.0,
            source: "Bom dia".to_string(),
            translated: "Good morning".to_string(),
            id: Some("p1-0".to_string()),
            voice: None,
            ..Default::default()
        },
        dub::Line {
            start: 4.0,
            end: 5.0,
            source: "Até logo".to_string(),
            translated: "See you".to_string(),
            id: Some("missing".to_string()),
            voice: None,
            ..Default::default()
        },
    ];

    apply_dub(&mut project, &lines, "en");

    assert_eq!(
        project.segments[0].target_text.as_deref(),
        Some("Good morning")
    );
    assert_eq!(project.segments[0].state, "dubbed");
}

#[test]
fn a_project_keeps_one_translation_per_target_language() {
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: Some("voice-en".to_string()),
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });
    let line = |text: &str, voice: &str| dub::Line {
        start: 1.25,
        end: 2.0,
        source: "Bom dia".to_string(),
        translated: text.to_string(),
        id: Some("p1-0".to_string()),
        voice: Some(voice.to_string()),
        ..Default::default()
    };

    apply_dub(&mut project, &[line("Good morning", "voice-en")], "en");
    assert_eq!(project.targets, ["en"]);
    apply_dub(&mut project, &[line("Guten Morgen", "voice-de")], "de");

    assert_eq!(project.targets, ["en", "de"]);
    assert_eq!(
        project.segments[0]
            .translations
            .get("en")
            .map(String::as_str),
        Some("Good morning")
    );
    assert_eq!(
        project.segments[0]
            .translations
            .get("de")
            .map(String::as_str),
        Some("Guten Morgen")
    );
    assert_eq!(
        project.segments[0].target_text.as_deref(),
        Some("Good morning")
    );
}

#[test]
fn reads_the_filename_header_encoded() {
    assert_eq!(percent_decode("my%20clip.mp4"), "my clip.mp4");
    assert_eq!(percent_decode("a+b.mov"), "a b.mov");
    assert_eq!(percent_decode("plain.mp4"), "plain.mp4");
    assert_eq!(percent_decode("broken%zz.mp4"), "broken%zz.mp4");
}

#[test]
fn dubbed_names_keep_stem_and_suffix() {
    assert_eq!(dubbed_name("video.mp4", "en"), "video.en.mp4");
    assert_eq!(dubbed_name("talk.mkv", "pt"), "talk.pt.mkv");
    assert_eq!(dubbed_name("recording", "en"), "recording.en.wav");
}

#[test]
fn content_types_follow_the_suffix() {
    assert_eq!(content_type("a.en.mp4"), "video/mp4");
    assert_eq!(content_type("story.m4b"), "audio/mp4");
    assert_eq!(content_type("a.en.srt"), "text/plain; charset=utf-8");
    assert_eq!(content_type("a.en.weird"), "application/octet-stream");
}

#[test]
fn a_third_party_profile_cannot_skip_the_authorisation() {
    let refused = consent_for(
        Some("third-party"),
        false,
        "translation".into(),
        "local".into(),
        1,
    );
    assert!(refused.is_err());
    let allowed = consent_for(
        Some("third-party"),
        true,
        "translation".into(),
        "local".into(),
        1,
    )
    .unwrap();
    assert_eq!(allowed.origin.as_str(), "third-party");
    let own = consent_for(None, false, "translation".into(), "local".into(), 1).unwrap();
    assert_eq!(own.origin.as_str(), "own-voice");
}

#[test]
fn a_stream_piece_carries_its_state_and_text() {
    let pieces = [parlo_core::engine::StreamPiece {
        state: "partial".to_string(),
        text: "bom dia".to_string(),
        audio_ms: 640,
        latency_ms: 210,
    }];
    let answer = pieces_json(&pieces);
    assert!(answer.contains("\"partial\""));
    assert!(answer.contains("bom dia"));
}

#[test]
fn tokens_are_hex_and_unique_enough() {
    let token = new_token();
    assert_eq!(token.len(), 24);
    assert!(token.chars().all(|character| character.is_ascii_hexdigit()));
    assert_ne!(token, new_token());
}

fn start_server_with(session: Session) -> (u16, Arc<AtomicBool>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepting = Arc::new(AtomicBool::new(true));
    let stop = Arc::clone(&accepting);
    let session = Arc::new(session);
    std::thread::spawn(move || {
        listen(listener, "t0ken".into(), port, session, accepting, None)
    });
    (port, stop)
}

#[test]
fn the_share_listener_refuses_anything_without_the_pin() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let session = Arc::new(
        Session::open("echo".into(), "parakeet".into(), "pt".into(), "en".into()).unwrap(),
    );
    let accepting = Arc::new(AtomicBool::new(true));
    let gate = Arc::new(Share {
        pin: "1234".to_string(),
        port,
    });
    std::thread::spawn(move || {
        listen(
            listener,
            "t0ken".into(),
            port,
            session,
            accepting,
            Some(gate),
        )
    });

    let missing = ask(
        port,
        &format!("GET /state HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"),
    );
    assert!(missing.starts_with("HTTP/1.1 401"), "{missing}");

    let audio = ask(
        port,
        &format!("GET /v1/models HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"),
    );
    assert!(audio.starts_with("HTTP/1.1 401"), "{audio}");

    let wrong = ask(
        port,
        &format!("GET /state HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Parlo-Pin: 4321\r\n\r\n"),
    );
    assert!(wrong.starts_with("HTTP/1.1 401"), "{wrong}");

    let right = ask(
        port,
        &format!("GET /state HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Parlo-Pin: 1234\r\n\r\n"),
    );
    assert!(right.starts_with("HTTP/1.1 200"), "{right}");
    assert!(right.contains("\"state\":"));

    let typed = ask(
        port,
        &format!("GET /state?pin=1234 HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"),
    );
    assert!(typed.starts_with("HTTP/1.1 200"), "{typed}");

    let models = ask(
        port,
        &format!(
            "GET /v1/models HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Parlo-Pin: 1234\r\n\r\n"
        ),
    );
    assert!(models.starts_with("HTTP/1.1 200"), "{models}");

    let rebinding = ask(
        port,
        &format!(
            "GET /state HTTP/1.1\r\nHost: attacker.example:{port}\r\nX-Parlo-Pin: 1234\r\n\r\n"
        ),
    );
    assert!(rebinding.starts_with("HTTP/1.1 400"), "{rebinding}");
}

#[test]
fn a_share_port_needs_a_share_flag_and_a_pin() {
    let mut config = parlo_core::settings::Config::default();
    assert!(share_target(&config).is_none());
    config.share = true;
    assert!(share_target(&config).is_none());
    config.share_pin = "12".to_string();
    assert!(share_target(&config).is_none());
    config.share_pin = "4821".to_string();
    config.share_port = 9100;
    assert_eq!(
        share_target(&config),
        Some(("0.0.0.0".to_string(), 9100, "4821".to_string()))
    );
    config.share = false;
    assert!(share_target(&config).is_none());
}

#[test]
fn a_closed_share_leaves_the_port_untouched() {
    let probe = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    let config = parlo_core::settings::Config {
        share: false,
        share_pin: "4821".to_string(),
        share_port: port,
        ..Default::default()
    };
    assert!(share_target(&config).is_none());
    assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
}

fn start_server() -> (u16, Arc<AtomicBool>) {
    start_server_with(
        Session::open("echo".into(), "parakeet".into(), "pt".into(), "en".into()).unwrap(),
    )
}

#[test]
fn a_remote_client_verifies_the_certificate_and_refuses_plain_http() {
    let authority_key = rcgen::KeyPair::generate().unwrap();
    let mut authority_params = rcgen::CertificateParams::new(Vec::new()).unwrap();
    authority_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let authority = authority_params.self_signed(&authority_key).unwrap();
    let server_key = rcgen::KeyPair::generate().unwrap();
    let server_params = rcgen::CertificateParams::new(vec!["localhost".to_string()]).unwrap();
    let server = server_params
        .signed_by(&server_key, &authority, &authority_key)
        .unwrap();

    let directory = std::env::temp_dir().join(format!("parlo-tls-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let authority_path = directory.join("authority.pem");
    let certificate_path = directory.join("server.pem");
    let key_path = directory.join("server.key");
    fs::write(&authority_path, authority.pem()).unwrap();
    fs::write(&certificate_path, server.pem()).unwrap();
    fs::write(&key_path, server_key.serialize_pem()).unwrap();

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let session = Arc::new(
        Session::open("echo".into(), "parakeet".into(), "pt".into(), "en".into()).unwrap(),
    );
    let accepting = Arc::new(AtomicBool::new(true));
    let config = Arc::new(tls_server_config(&certificate_path, &key_path).unwrap());
    std::thread::spawn(move || {
        listen_tls(listener, "t0ken".into(), port, session, accepting, config)
    });

    let mut settings = parlo_core::settings::Config {
        remote_url: format!("https://localhost:{port}"),
        remote_token: "t0ken".to_string(),
        ..Default::default()
    };
    let refused = crate::remote::post(&settings, "/translate", "audio/wav", b"")
        .expect_err("a certificate the client does not trust must be refused");
    assert!(refused.contains("certificate"), "{refused}");

    settings.remote_ca = authority_path.display().to_string();
    let trusted = crate::remote::post(&settings, "/translate", "audio/wav", b"").unwrap();
    assert_eq!(trusted.status, 400);
    assert!(String::from_utf8_lossy(&trusted.body).contains("bad wav"));

    settings.remote_token = "stolen".to_string();
    let unauthenticated =
        crate::remote::post(&settings, "/translate", "audio/wav", b"").unwrap();
    assert_eq!(unauthenticated.status, 401);

    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let _ = stream.write_all(
        format!("GET /health HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n").as_bytes(),
    );
    let mut raw = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => raw.extend_from_slice(&chunk[..read]),
            Err(_) => break,
        }
    }
    let text = String::from_utf8_lossy(&raw);
    assert!(text.starts_with("HTTP/1.1 426"), "{text}");
}

#[test]
fn the_locale_route_answers_with_the_table() {
    let (port, _stop) = start_server();
    let answer = ask(port, "GET /locale HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    let body = body_json(&answer);
    let strings = body
        .get("strings")
        .and_then(|value| value.as_object())
        .unwrap();
    assert!(strings.contains_key("nav.home"));
    assert_eq!(
        body.get("available")
            .and_then(|value| value.as_array())
            .unwrap()
            .len(),
        21
    );
}

fn send(port: u16, method: &str, path: &str, body: &str) -> String {
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nx-parlo-token: t0ken\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    ask(port, &request)
}

fn body_json(answer: &str) -> serde_json::Value {
    let body = answer.split("\r\n\r\n").nth(1).unwrap_or("");
    serde_json::from_str(body).unwrap()
}

#[test]
fn a_speaker_count_is_kept_with_the_meeting() {
    let (port, stop) = start_server();
    let started = send(
        port,
        "POST",
        "/meeting/start",
        "{\"title\":\"Retro\",\"speakers\":3}",
    );
    assert!(started.starts_with("HTTP/1.1 200"), "{started}");
    assert_eq!(body_json(&started)["speaker_hint"], 3);
    let listed = send(port, "GET", "/meetings", "");
    assert!(listed.contains("\"speaker_hint\":3"), "{listed}");
    let refused = send(
        port,
        "POST",
        "/meeting/start",
        "{\"title\":\"Retro\",\"speakers\":21}",
    );
    assert!(refused.starts_with("HTTP/1.1 400"), "{refused}");
    stop.store(false, Ordering::Relaxed);
}

#[test]
fn every_model_card_carries_its_licence() {
    let (port, stop) = start_server();
    let answer = send(port, "GET", "/models", "");
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    let state = body_json(&answer);
    let models = state["models"].as_array().unwrap();
    assert!(!models.is_empty());
    for model in models {
        let licence = model["licence"].as_str().unwrap_or("");
        assert!(!licence.is_empty(), "{} has no licence", model["id"]);
    }
    stop.store(false, Ordering::Relaxed);
}

fn websocket_attempt(port: u16, query: &str) -> Result<TcpStream, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let key = BASE64.encode([0u8; 16]);
    let request = format!(
        "GET /v1/audio/transcriptions/stream{query} HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\nOrigin: http://127.0.0.1:{port}\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = Vec::new();
    let mut byte = [0u8; 1];
    while !response.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        response.push(byte[0]);
    }
    let head = String::from_utf8_lossy(&response).to_string();
    if head.starts_with("HTTP/1.1 101") {
        Ok(stream)
    } else {
        Err(head)
    }
}

#[test]
fn a_ticket_opens_one_socket_and_cannot_be_replayed() {
    let (port, stop) = start_server();
    let granted = send(port, "POST", "/ticket", "");
    assert!(granted.starts_with("HTTP/1.1 200"), "{granted}");
    let value = body_json(&granted);
    let ticket = value["ticket"].as_str().unwrap().to_string();
    assert!(!ticket.is_empty());
    assert_eq!(value["expires_in"], 30);
    assert_eq!(value["protocol"], ws::PROTOCOL);
    let query = format!("?ticket={ticket}");
    let mut socket = websocket_attempt(port, &query).unwrap();
    let started = read_websocket_json(&mut socket);
    assert_eq!(started["type"], "session.started");
    drop(socket);
    let refused = websocket_attempt(port, &query).unwrap_err();
    assert!(refused.starts_with("HTTP/1.1 401"), "{refused}");
    stop.store(false, Ordering::Relaxed);
}

#[test]
fn a_ticket_cannot_be_made_up() {
    let (port, stop) = start_server();
    let refused = websocket_attempt(port, "?ticket=not-a-ticket").unwrap_err();
    assert!(refused.starts_with("HTTP/1.1 401"), "{refused}");
    stop.store(false, Ordering::Relaxed);
}

fn websocket_client(port: u16) -> TcpStream {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let key = BASE64.encode([0u8; 16]);
    let request = format!(
        "GET /v1/audio/transcriptions/stream?t=t0ken HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\nOrigin: http://127.0.0.1:{port}\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = Vec::new();
    let mut byte = [0u8; 1];
    while !response.ends_with(b"\r\n\r\n") {
        stream.read_exact(&mut byte).unwrap();
        response.push(byte[0]);
    }
    assert!(
        response.starts_with(b"HTTP/1.1 101"),
        "{}",
        String::from_utf8_lossy(&response)
    );
    stream
}

fn send_websocket_frame(stream: &mut dyn Stream, opcode: u8, payload: &[u8]) {
    let mut frame = vec![0x80 | opcode];
    let length = payload.len();
    if length < 126 {
        frame.push(0x80 | length as u8);
    } else if length <= u16::MAX as usize {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(length as u16).to_be_bytes());
    } else {
        frame.push(0x80 | 127);
        frame.extend_from_slice(&(length as u64).to_be_bytes());
    }
    let mask = [0x12, 0x34, 0x56, 0x78];
    frame.extend_from_slice(&mask);
    frame.extend(
        payload
            .iter()
            .enumerate()
            .map(|(index, byte)| byte ^ mask[index % 4]),
    );
    stream.write_all(&frame).unwrap();
}

fn read_exact_bytes(stream: &mut dyn Stream, length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    stream.read_exact(&mut bytes).unwrap();
    bytes
}

fn read_websocket_frame(stream: &mut dyn Stream) -> (u8, Vec<u8>) {
    let head = read_exact_bytes(stream, 2);
    let opcode = head[0] & 0x0f;
    let masked = head[1] & 0x80 != 0;
    let mut length = (head[1] & 0x7f) as usize;
    if length == 126 {
        let bytes = read_exact_bytes(stream, 2);
        length = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
    } else if length == 127 {
        let bytes = read_exact_bytes(stream, 8);
        length = u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]) as usize;
    }
    let mask = masked.then(|| read_exact_bytes(stream, 4));
    let mut payload = read_exact_bytes(stream, length);
    if let Some(mask) = mask {
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[index % 4];
        }
    }
    (opcode, payload)
}

fn read_websocket_json(stream: &mut dyn Stream) -> serde_json::Value {
    let (opcode, payload) = read_websocket_frame(stream);
    assert_eq!(opcode, 1);
    serde_json::from_slice(&payload).unwrap()
}

#[test]
fn a_websocket_stream_emits_partial_stable_and_final() {
    let session = Session::with_engine(Box::<StreamingEngine>::default(), "pt", "en");
    let (port, stop) = start_server_with(session);
    let mut socket = websocket_client(port);
    let pcm: Vec<u8> = (0..1600i16)
        .flat_map(|sample| sample.to_le_bytes())
        .collect();

    send_websocket_frame(&mut socket, 0x2, &pcm);
    send_websocket_frame(&mut socket, 0x2, &pcm);
    send_websocket_frame(&mut socket, 0x1, br#"{"type":"input_audio.end"}"#);

    let started = read_websocket_json(&mut socket);
    assert_eq!(started["type"], "session.started");
    assert_eq!(started["protocol"], "parlo.speech.v1");
    let session_id = started["session_id"].clone();

    let partial = read_websocket_json(&mut socket);
    assert_eq!(partial["type"], "partial");
    assert_eq!(partial["text"], "bom");
    assert_eq!(partial["session_id"], session_id);

    let stable = read_websocket_json(&mut socket);
    assert_eq!(stable["type"], "stable");
    assert_eq!(stable["text"], "bom dia");
    assert_eq!(stable["session_id"], session_id);

    let final_piece = read_websocket_json(&mut socket);
    assert_eq!(final_piece["type"], "final");
    assert_eq!(final_piece["text"], "bom dia.");
    assert_eq!(final_piece["session_id"], session_id);

    let ended = read_websocket_json(&mut socket);
    assert_eq!(ended["type"], "input_audio.end");
    assert_eq!(ended["session_id"], session_id);

    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}

#[test]
fn the_capabilities_document_lists_engines_and_limits() {
    let (port, stop) = start_server();
    let answer = ask(
        port,
        "GET /.well-known/parlo HTTP/1.1\r\nHost: localhost\r\n\r\n",
    );
    assert!(answer.starts_with("HTTP/1.1 200 OK"), "{answer}");
    let document = body_json(&answer);
    assert_eq!(document["protocol"], "parlo.speech.v1");
    let engines = document["engines"].as_array().unwrap();
    assert!(engines.iter().any(|engine| {
        engine["id"] == "cascade" && engine["available"] == true && engine["default"] == true
    }));
    assert!(engines
        .iter()
        .any(|engine| engine["id"] == "hibiki" && engine["available"] == false));
    assert_eq!(document["limits"]["max_audio_bytes"], MAX_AUDIO_BODY as u64);
    assert_eq!(
        document["limits"]["max_websocket_message_bytes"],
        super::ws::MAX_MESSAGE as u64
    );

    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}

#[test]
fn every_public_endpoint_has_an_example_in_the_api_doc() {
    let doc = include_str!("../../../../docs/API.md");
    let mut paths: Vec<String> = Vec::new();
    for source in [
        include_str!("mod.rs"),
        include_str!("routes/openai.rs"),
        include_str!("routes/models.rs"),
    ] {
        let mut rest = source;
        while let Some(at) = rest.find("path") {
            rest = &rest[at + 4..];
            let window = rest
                .split(
                    "
",
                )
                .next()
                .unwrap_or("");
            let Some(start) = window.find("\"/v1/") else {
                continue;
            };
            let tail = &window[start + 1..];
            let Some(end) = tail.find('"') else {
                continue;
            };
            let path = &tail[..end];
            if !paths.iter().any(|kept| kept == path) {
                paths.push(path.to_string());
            }
        }
    }
    assert!(paths.len() >= 4, "{paths:?}");
    for path in paths {
        assert!(doc.contains(&path), "{path} has no example in docs/API.md");
    }
}

#[test]
fn the_example_client_runs_against_the_server() {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let session = Session::with_engine(
        Box::new(RecordingEngine {
            heard: Arc::clone(&heard),
        }),
        "pt",
        "en",
    );
    let (port, stop) = start_server_with(session);
    let script = format!(
        "{}/../../examples/openai_client.py",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = std::process::Command::new("python")
        .arg(script)
        .args(["--port", &port.to_string(), "--token", "t0ken"])
        .output()
        .expect("python must be available for the example");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "the example client failed\nstdout: {stdout}\nstderr: {stderr}"
    );
    for line in [
        "models:",
        "transcribe:",
        "translate:",
        "speech:",
        "refused: 400",
    ] {
        assert!(stdout.contains(line), "{line} is missing from\n{stdout}");
    }
    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}

const ROUTES: &[(&str, &str, bool)] = &[
    ("GET", "/call", true),
    ("GET", "/pronounce", false),
    ("GET", "/retention", true),
    ("POST", "/retention", true),
    ("GET", "/meetings", true),
    ("POST", "/meeting/window", true),
    ("POST", "/meeting/summary", true),
    ("POST", "/meetings/clear", true),
    ("ANY", "/meeting/import", false),
    ("ANY", "/meeting/delete", false),
    ("POST", "/meeting/start", true),
    ("POST", "/meeting/segment", true),
    ("POST", "/meeting/ask", true),
    ("POST", "/memory/ask", true),
    ("GET", "/vocabulary", true),
    ("POST", "/vocabulary", true),
    ("POST", "/vocabulary/delete", true),
    ("POST", "/vocabulary/accept", true),
    ("GET", "/vocabulary/export", true),
    ("POST", "/vocabulary/import", true),
    ("GET", "/features", false),
    ("POST", "/features", true),
    ("POST", "/style", true),
    ("GET", "/profiles", true),
    ("POST", "/profiles", true),
    ("POST", "/history/restyle", true),
    ("GET", "/hardware", false),
    ("GET", "/catalog", false),
    ("GET", "/history", true),
    ("GET", "/history/export", true),
    ("POST", "/history/clear", true),
    ("ANY", "/history/favourite", false),
    ("ANY", "/history/edit", false),
    ("GET", "/locale", false),
    ("GET", "/settings", true),
    ("POST", "/settings", true),
    ("GET", "/intelligence", false),
    ("POST", "/intelligence", true),
    ("POST", "/intelligence/key", true),
    ("POST", "/intelligence/key/delete", true),
    ("POST", "/intelligence/test", true),
    ("POST", "/dub/import", true),
    ("GET", "/dub/projects", true),
    ("GET", "/voice", false),
    ("POST", "/voice", true),
    ("POST", "/voice/revoke", true),
    ("POST", "/voice/delete", true),
    ("POST", "/voice/sample", true),
    ("POST", "/voice/sample/play", true),
    ("POST", "/voice/export", true),
    ("POST", "/voice/import", true),
    ("POST", "/voice/compare", true),
    ("GET", "/voice/packs", false),
    ("POST", "/voice/packs", true),
    ("POST", "/dub", true),
    ("GET", "/models", false),
    ("POST", "/models", true),
    ("POST", "/models/dictation", true),
    ("POST", "/models/rest", true),
    ("POST", "/models/download", true),
    ("POST", "/models/relocate", true),
    ("POST", "/models/cancel", true),
    ("POST", "/models/policy", true),
    ("GET", "/update/progress", false),
    ("POST", "/update/restart", true),
    ("GET", "/update", false),
    ("POST", "/update", true),
    ("GET", "/", false),
    ("GET", "/app.css", false),
    ("GET", "/island", false),
    ("GET", "/island.html", false),
    ("GET", "/captions", false),
    ("GET", "/captions.html", false),
    ("POST", "/captions/save", true),
    ("GET", "/live", false),
    ("GET", "/live.html", false),
    ("GET", "/app.js", false),
    ("GET", "/routes.js", false),
    ("GET", "/languages.js", false),
    ("GET", "/languages.json", false),
    ("POST", "/ticket", true),
    ("GET", "/v1/audio/transcriptions/stream", false),
    ("GET", "/target", true),
    ("POST", "/listen/stream", true),
    ("POST", "/listen/stream-end", true),
    ("ANY", "/state", false),
    ("GET", "/engine", false),
    ("ANY", "/health", false),
    ("POST", "/v1/audio/transcriptions", false),
    ("POST", "/v1/audio/translations", false),
    ("POST", "/v1/audio/speech", false),
    ("ANY", "/mcp", true),
    ("POST", "/v1/jobs", true),
    ("GET", "/v1/jobs", true),
    ("POST", "/dictate/begin", true),
    ("POST", "/undo", true),
    ("POST", "/insert", true),
    ("POST", "/dictate", true),
];

fn router_source() -> &'static str {
    let source = include_str!("mod.rs");
    &source[..source.rfind("mod tests {").unwrap_or(source.len())]
}

fn routed_paths() -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    for line in router_source().lines() {
        if !line.contains("path ") || !line.contains("if ") {
            continue;
        }
        let method = line
            .split_once("method == \"")
            .map(|(_, rest)| rest.split('"').next().unwrap_or("").to_string())
            .unwrap_or_else(|| "ANY".to_string());
        for marker in ["path == \"", "path.starts_with(\""] {
            let mut rest = line;
            while let Some(at) = rest.find(marker) {
                rest = &rest[at + marker.len()..];
                let Some(end) = rest.find('"') else {
                    break;
                };
                let path = rest[..end].to_string();
                let known = found.iter().any(|(kept_method, kept_path)| {
                    kept_path == &path
                        && (kept_method == &method || kept_method == "ANY" || method == "ANY")
                });
                if !known {
                    found.push((method.clone(), path));
                }
            }
        }
    }
    found
}

fn probe_path(path: &str) -> String {
    if path.ends_with('/') {
        format!("{path}x")
    } else {
        path.to_string()
    }
}

fn ask_with(port: u16, method: &str, path: &str, headers: &str, body: &str) -> String {
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nx-parlo-token: t0ken\r\n{headers}Content-Length: {}\r\n\r\n{body}",
        body.len()
    );
    ask(port, &request)
}

#[test]
fn the_route_table_covers_every_route_the_router_serves() {
    for (method, path) in routed_paths() {
        let kept = ROUTES.iter().any(|(kept_method, kept_path, _)| {
            kept_path == &path
                && (kept_method == &method || *kept_method == "ANY" || method == "ANY")
        });
        assert!(kept, "{method} {path} is routed but has no row in ROUTES");
    }
}

#[test]
fn every_route_that_needs_a_token_refuses_a_wrong_one() {
    let (port, stop) = start_server();
    for (method, path, auth) in ROUTES {
        if !*auth || path.ends_with("transcriptions/stream") {
            continue;
        }
        let probe = if *method == "ANY" { "POST" } else { method };
        let request = format!(
            "{probe} {} HTTP/1.1\r\nHost: 127.0.0.1\r\nx-parlo-token: wrong\r\nContent-Length: 0\r\n\r\n",
            probe_path(path)
        );
        let answer = ask(port, &request);
        assert!(
            answer.starts_with("HTTP/1.1 401"),
            "{method} {path} answered without the token: {answer}"
        );
    }
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn the_extra_pages_answer_with_html() {
    let (port, stop) = start_server();
    for path in [
        "/island",
        "/island.html",
        "/captions",
        "/captions.html",
        "/live",
        "/live.html",
    ] {
        let answer = send(port, "GET", path, "");
        assert!(
            answer.starts_with("HTTP/1.1 200") || answer.starts_with("HTTP/1.1 409"),
            "{path}: {answer}"
        );
        if answer.starts_with("HTTP/1.1 200") {
            assert!(
                answer.contains("text/html; charset=utf-8"),
                "{path}: {answer}"
            );
        }
    }
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn the_read_only_routes_report_their_state() {
    let (port, stop) = start_server();
    for path in [
        "/call",
        "/hardware",
        "/catalog",
        "/features",
        "/intelligence",
        "/target",
        "/update/progress",
        "/dub/projects",
        "/history",
        "/history/export",
        "/profiles",
    ] {
        let answer = send(port, "GET", path, "");
        assert!(answer.starts_with("HTTP/1.1 200"), "{path}: {answer}");
    }
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn a_meeting_is_imported_asked_summarised_and_deleted() {
    let (port, stop) = start_server();
    let imported = send(
        port,
        "POST",
        "/meeting/import",
        "{\"title\":\"Retro\",\"text\":\"Ana: bom dia\\nRui: ola\"}",
    );
    assert!(imported.starts_with("HTTP/1.1 200"), "{imported}");
    let id = body_json(&imported)["id"].as_str().unwrap().to_string();
    let segmented = send(
        port,
        "POST",
        "/meeting/segment",
        &format!(
            "{{\"id\":\"{id}\",\"speaker\":\"Ana\",\"text\":\"revimos a frente\",\"at_ms\":1000}}"
        ),
    );
    assert!(segmented.starts_with("HTTP/1.1 200"), "{segmented}");
    let asked = send(
        port,
        "POST",
        "/meeting/ask",
        &format!("{{\"id\":\"{id}\",\"question\":\"what did Ana say\"}}"),
    );
    assert!(asked.starts_with("HTTP/1.1 200"), "{asked}");
    let memory = send(
        port,
        "POST",
        "/memory/ask",
        "{\"question\":\"what did Ana say\"}",
    );
    assert!(memory.starts_with("HTTP/1.1 200"), "{memory}");
    let summary = send(
        port,
        "POST",
        "/meeting/summary",
        &format!("{{\"id\":\"{id}\"}}"),
    );
    assert!(summary.starts_with("HTTP/1.1 200"), "{summary}");
    let window = send(port, "POST", "/meeting/window", "");
    assert!(window.starts_with("HTTP/1.1 404"), "{window}");
    let empty = send(port, "POST", "/meeting/ask", "{\"question\":\"\"}");
    assert!(empty.starts_with("HTTP/1.1 400"), "{empty}");
    let deleted = send(
        port,
        "POST",
        "/meeting/delete",
        &format!("{{\"id\":\"{id}\"}}"),
    );
    assert!(deleted.starts_with("HTTP/1.1 200"), "{deleted}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn the_vocabulary_routes_report_a_missing_engine() {
    let (port, stop) = start_server();
    for (method, path, body) in [
        ("GET", "/vocabulary", ""),
        (
            "POST",
            "/vocabulary",
            "{\"kind\":\"term\",\"wrong\":\"a\",\"right\":\"b\"}",
        ),
        ("GET", "/vocabulary/export", ""),
        (
            "POST",
            "/vocabulary/delete",
            "{\"kind\":\"term\",\"wrong\":\"a\"}",
        ),
    ] {
        let answer = send(port, method, path, body);
        assert!(
            answer.starts_with("HTTP/1.1 500"),
            "{method} {path}: {answer}"
        );
        assert!(
            answer.contains("engine beside Parlo"),
            "{method} {path}: {answer}"
        );
    }
    let empty_accept = send(port, "POST", "/vocabulary/accept", "{}");
    assert!(empty_accept.starts_with("HTTP/1.1 400"), "{empty_accept}");
    let accepted = send(
        port,
        "POST",
        "/vocabulary/accept",
        "{\"heard\":\"a\",\"written\":\"b\"}",
    );
    assert!(accepted.starts_with("HTTP/1.1 500"), "{accepted}");
    let empty_import = send(port, "POST", "/vocabulary/import", "{\"text\":\"\"}");
    assert!(empty_import.starts_with("HTTP/1.1 400"), "{empty_import}");
    let imported = send(port, "POST", "/vocabulary/import", "{\"text\":\"a\\tb\"}");
    assert!(imported.starts_with("HTTP/1.1 500"), "{imported}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn the_voice_routes_refuse_an_unknown_profile() {
    let (port, stop) = start_server();
    for (path, body) in [
        ("/voice/revoke", "{\"id\":\"missing\"}"),
        ("/voice/delete", "{\"id\":\"missing\"}"),
        ("/voice/sample/play", "{\"id\":\"missing\"}"),
        ("/voice/export", "{\"id\":\"missing\"}"),
    ] {
        let answer = send(port, "POST", path, body);
        assert!(answer.starts_with("HTTP/1.1 404"), "{path}: {answer}");
    }
    let no_id = send(port, "POST", "/voice/sample", "");
    assert!(no_id.starts_with("HTTP/1.1 400"), "{no_id}");
    let foreign = send(port, "POST", "/voice/import", "not a voice file");
    assert!(foreign.starts_with("HTTP/1.1 400"), "{foreign}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn a_revoked_voice_plays_the_default_voice_instead_of_failing() {
    let spoken = Arc::new(Mutex::new(Vec::new()));
    let session = Session::with_engine(
        Box::new(SpeakingEngine {
            spoken: Arc::clone(&spoken),
        }),
        "pt",
        "en",
    );
    let (port, stop) = start_server_with(session);
    let store = voice_store();
    let label = format!(
        "Route test {} {}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default()
    );
    let profile = store
        .create(
            &label,
            "pt-PT",
            parlo_core::voice::Consent::own_voice("test", "this machine", 1),
        )
        .expect("the test profile is created");
    store
        .save_sample(&profile.id, b"RIFFtest-sample")
        .expect("the sample is saved");
    store
        .revoke(&profile.id, 2)
        .expect("the profile is revoked");
    assert!(
        store.sample(&profile.id).is_err(),
        "the sample survived the revocation"
    );
    let body = format!("{{\"id\":\"{}\"}}", profile.id);
    let request = format!(
        "POST /voice/sample/play HTTP/1.1\r\nHost: 127.0.0.1\r\nx-parlo-token: t0ken\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let raw = ask_raw(port, request.as_bytes());
    let answer = String::from_utf8_lossy(&raw).into_owned();
    let kept = store
        .create(
            &format!("{label} kept"),
            "pt-PT",
            parlo_core::voice::Consent::own_voice("test", "this machine", 1),
        )
        .expect("the second test profile is created");
    let missing = send(
        port,
        "POST",
        "/voice/sample/play",
        &format!("{{\"id\":\"{}\"}}", kept.id),
    );
    store
        .delete(&profile.id)
        .expect("the test profile is deleted");
    store
        .delete(&kept.id)
        .expect("the second profile is deleted");
    assert!(
        missing.starts_with("HTTP/1.1 404"),
        "a profile without a sample answered {missing}"
    );
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(answer.contains("audio/wav"), "{answer}");
    assert_eq!(
        spoken
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_slice(),
        [DEFAULT_VOICE_TEXT.to_string()]
    );
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn the_model_routes_refuse_an_unknown_unit() {
    let (port, stop) = start_server();
    let unknown = send(port, "POST", "/models/download/unknown", "");
    assert!(unknown.starts_with("HTTP/1.1 404"), "{unknown}");
    let empty = send(port, "POST", "/models/download", "{}");
    assert!(empty.starts_with("HTTP/1.1 400"), "{empty}");
    let loading = send(port, "POST", "/models/load/unknown", "");
    assert!(loading.starts_with("HTTP/1.1 404"), "{loading}");
    let policy = send(
        port,
        "POST",
        "/models/policy",
        "{\"id\":\"unknown\",\"minutes\":5}",
    );
    assert!(policy.starts_with("HTTP/1.1 409"), "{policy}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn the_history_routes_report_a_missing_dictation() {
    let (port, stop) = start_server();
    for (path, body) in [
        (
            "/history/restyle",
            "{\"id\":\"missing\",\"style\":\"formal\"}",
        ),
        ("/history/favourite", "{\"id\":\"missing\"}"),
        ("/history/edit", "{\"id\":\"missing\",\"text\":\"x\"}"),
    ] {
        let answer = send(port, "POST", path, body);
        assert!(answer.starts_with("HTTP/1.1 404"), "{path}: {answer}");
    }
    let exported = send(port, "GET", "/history/export", "");
    assert!(exported.contains("parlo-dictations.md"), "{exported}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn a_dub_project_is_imported_listed_and_deleted() {
    let (port, stop) = start_server();
    let refused = ask_with(
        port,
        "POST",
        "/dub/import",
        "x-parlo-filename: retro.txt\r\n",
        "1\n00:00:00,000 --> 00:00:01,000\nBom dia\n",
    );
    assert!(refused.starts_with("HTTP/1.1 400"), "{refused}");
    let imported = ask_with(
        port,
        "POST",
        "/dub/import",
        "x-parlo-filename: retro.srt\r\n",
        "1\n00:00:00,000 --> 00:00:01,000\nBom dia\n",
    );
    assert!(imported.starts_with("HTTP/1.1 200"), "{imported}");
    let id = body_json(&imported)["id"].as_str().unwrap().to_string();
    let listed = send(port, "GET", "/dub/projects", "");
    assert!(listed.starts_with("HTTP/1.1 200"), "{listed}");
    assert!(listed.contains(&id), "{listed}");
    let one = send(port, "GET", &format!("/dub/project/{id}"), "");
    assert!(one.starts_with("HTTP/1.1 200"), "{one}");
    let deleted = send(port, "DELETE", &format!("/dub/project/{id}"), "");
    assert!(deleted.starts_with("HTTP/1.1 200"), "{deleted}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn the_insert_and_listen_routes_refuse_a_bad_request() {
    let (port, stop) = start_server();
    let listen = send(port, "POST", "/listen/stream", "not a wav");
    assert!(listen.starts_with("HTTP/1.1 400"), "{listen}");
    let ended = send(port, "POST", "/listen/stream-end", "");
    assert!(!ended.starts_with("HTTP/1.1 404"), "{ended}");
    let inserted = send(port, "POST", "/insert", "{}");
    assert!(inserted.starts_with("HTTP/1.1 400"), "{inserted}");
    let undone = send(port, "POST", "/undo", "");
    assert!(!undone.starts_with("HTTP/1.1 404"), "{undone}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn the_settings_routes_refuse_a_value_they_do_not_know() {
    let (port, stop) = start_server();
    let key = send(port, "POST", "/intelligence/key", "{\"provider\":\"nope\"}");
    assert!(key.starts_with("HTTP/1.1 400"), "{key}");
    let removed = send(
        port,
        "POST",
        "/intelligence/key/delete",
        "{\"provider\":\"nope\"}",
    );
    assert!(removed.starts_with("HTTP/1.1 400"), "{removed}");
    let settings = send(port, "POST", "/intelligence", "nonsense");
    assert!(settings.starts_with("HTTP/1.1 400"), "{settings}");
    let style = send(port, "POST", "/style", "{\"style\":\"angry\"}");
    assert!(style.starts_with("HTTP/1.1 400"), "{style}");
    let features = send(port, "POST", "/features", "nonsense");
    assert!(features.starts_with("HTTP/1.1 400"), "{features}");
    let profile = send(port, "POST", "/profiles", "{}");
    assert!(profile.starts_with("HTTP/1.1 400"), "{profile}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn a_comparison_that_names_a_missing_pack_is_refused() {
    let (port, stop) = start_server();
    let refused = send(
        port,
        "POST",
        "/voice/compare",
        "{\"text\":\"Bom dia\",\"pack\":\"no such pack\"}",
    );
    assert!(refused.starts_with("HTTP/1.1 400"), "{refused}");
    assert!(refused.contains("no voice pack"), "{refused}");
    let broken = send(port, "POST", "/voice/packs", "{\"format\":\"other\"}");
    assert!(broken.starts_with("HTTP/1.1 400"), "{broken}");
    assert!(broken.contains("not a Parlo voice pack"), "{broken}");
    let listed = send(port, "GET", "/voice/packs", "");
    assert!(listed.starts_with("HTTP/1.1 200"), "{listed}");
    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}

#[test]
fn the_processor_setting_is_shown_and_checked() {
    let (port, stop) = start_server();
    let refused = send(port, "POST", "/settings", "{\"compute\":\"tpu\"}");
    assert!(refused.starts_with("HTTP/1.1 400 Bad Request"), "{refused}");
    assert!(refused.contains("auto or cpu"), "{refused}");
    let settings = send(port, "GET", "/settings", "");
    let values = body_json(&settings);
    assert!(
        matches!(values["compute"].as_str(), Some("auto" | "cpu")),
        "{values}"
    );
    let engine = send(port, "GET", "/engine", "");
    assert!(engine.starts_with("HTTP/1.1 200 OK"), "{engine}");
    let detail = body_json(&engine);
    assert!(
        matches!(detail["device"].as_str(), Some("cpu" | "gpu")),
        "{detail}"
    );
    assert!(detail["compute"].is_string(), "{detail}");
    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}

#[test]
fn an_agent_uses_mcp_over_http() {
    let (port, stop) = start_server();
    let initialized = send(
        port,
        "POST",
        "/mcp",
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })
        .to_string(),
    );
    assert!(initialized.starts_with("HTTP/1.1 200 OK"), "{initialized}");
    assert_eq!(
        body_json(&initialized)["result"]["serverInfo"]["name"],
        "parlo"
    );

    let listed = send(
        port,
        "POST",
        "/mcp",
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {},
        })
        .to_string(),
    );
    assert!(listed.starts_with("HTTP/1.1 200 OK"), "{listed}");
    assert_eq!(
        body_json(&listed)["result"]["tools"]
            .as_array()
            .unwrap()
            .len(),
        15
    );

    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}

fn multipart(boundary: &str, model: &str, audio: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"model\"\r\n\r\n{model}\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"sample.wav\"\r\nContent-Type: audio/wav\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(audio);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

fn post_multipart(port: u16, path: &str, body: &[u8], boundary: &str) -> String {
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer t0ken\r\nContent-Type: multipart/form-data; boundary={boundary}\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    let mut request = head.into_bytes();
    request.extend_from_slice(body);
    ask_bytes(port, &request)
}

fn post_transcription(port: u16, body: &[u8], boundary: &str) -> String {
    post_multipart(port, "/v1/audio/transcriptions", body, boundary)
}

#[test]
fn an_openai_transcription_runs_through_the_pipeline() {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let session = Session::with_engine(
        Box::new(RecordingEngine {
            heard: Arc::clone(&heard),
        }),
        "pt",
        "en",
    );
    let (port, stop) = start_server_with(session);

    let format = AudioFormat::mono(16_000).unwrap();
    let samples: Vec<f32> = (0..16_000)
        .map(|index| (index as f32 * 0.07).sin() * 0.2)
        .collect();
    let wav = write_pcm16(&Wav { format, samples });
    let boundary = "----parlo-openai-test";
    let body = multipart(boundary, "whisper-1", &wav);
    let request = String::from_utf8_lossy(&body);
    assert!(request.contains("name=\"model\""));
    assert!(request.contains("name=\"file\"; filename=\"sample.wav\""));
    assert!(request.contains("whisper-1"));
    assert!(request.contains("audio/wav"));

    let answer = post_transcription(port, &body, boundary);
    assert!(answer.starts_with("HTTP/1.1 200 OK"), "{answer}");
    assert!(
        answer.contains("Content-Type: application/json"),
        "{answer}"
    );
    assert_eq!(body_json(&answer)["text"], "ola mundo");

    let decoded = read_pcm16(&wav).unwrap();
    let expected: Vec<u8> = decoded
        .samples
        .iter()
        .flat_map(|sample| {
            (((*sample).clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes()
        })
        .collect();
    assert_eq!(*heard.lock().unwrap(), expected);

    let rejected =
        post_transcription(port, &multipart(boundary, "not-a-model", &wav), boundary);
    assert!(rejected.starts_with("HTTP/1.1 400"), "{rejected}");
    assert!(rejected.contains("\"param\":\"model\""), "{rejected}");

    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}

#[test]
fn openai_speech_translation_and_models_answer_with_their_shapes() {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let session = Session::with_engine(
        Box::new(RecordingEngine {
            heard: Arc::clone(&heard),
        }),
        "pt",
        "en",
    );
    let (port, stop) = start_server_with(session);

    let speech_body =
        br#"{"model":"tts-1","voice":"alloy","input":"Ola","response_format":"wav"}"#;
    let speech_head = format!(
        "POST /v1/audio/speech HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer t0ken\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        speech_body.len()
    );
    let mut speech_request = speech_head.into_bytes();
    speech_request.extend_from_slice(speech_body);
    let speech = ask_raw(port, &speech_request);
    let head_end = find(&speech, b"\r\n\r\n").unwrap();
    let speech_head = String::from_utf8_lossy(&speech[..head_end]);
    assert!(speech_head.starts_with("HTTP/1.1 200 OK"), "{speech_head}");
    assert!(
        speech_head.contains("Content-Type: audio/wav"),
        "{speech_head}"
    );
    let spoken = read_pcm16(&speech[head_end + 4..]).unwrap();
    assert_eq!(spoken.samples.len(), 1600);
    assert_eq!(spoken.format.sample_rate(), 16_000);

    let format = AudioFormat::mono(16_000).unwrap();
    let samples: Vec<f32> = (0..1600)
        .map(|index| (index as f32 * 0.05).sin() * 0.2)
        .collect();
    let wav = write_pcm16(&Wav { format, samples });
    let boundary = "----parlo-openai-translation";
    let translation = post_multipart(
        port,
        "/v1/audio/translations",
        &multipart(boundary, "whisper-1", &wav),
        boundary,
    );
    assert!(translation.starts_with("HTTP/1.1 200 OK"), "{translation}");
    assert!(
        translation.contains("Content-Type: application/json"),
        "{translation}"
    );
    assert_eq!(body_json(&translation)["text"], "hello");

    let decoded = read_pcm16(&wav).unwrap();
    let expected: Vec<u8> = decoded
        .samples
        .iter()
        .flat_map(|sample| {
            (((*sample).clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes()
        })
        .collect();
    assert_eq!(*heard.lock().unwrap(), expected);

    let models = ask(port, "GET /v1/models HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    let document = body_json(&models);
    assert_eq!(document["object"], "list");
    let data = document["data"].as_array().unwrap();
    let parakeet = data.iter().find(|model| model["id"] == "parakeet").unwrap();
    assert_eq!(parakeet["object"], "model");
    assert_eq!(parakeet["owned_by"], "parlo");
    assert_eq!(parakeet["default"], true);
    assert_eq!(parakeet["licence"], "CC-BY-4.0");
    assert_eq!(parakeet["languages"], serde_json::json!(["pt", "en"]));

    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}

#[test]
fn the_authenticated_job_flow_works_over_http() {
    let (port, stop) = start_server();

    let created = send(port, "POST", "/v1/jobs", "{\"kind\":\"fixture\"}");
    assert!(created.starts_with("HTTP/1.1 202"), "{created}");
    let id = body_json(&created)["id"].as_str().unwrap().to_string();
    let state = send(port, "GET", &format!("/v1/jobs/{id}"), "");
    assert!(state.starts_with("HTTP/1.1 200"), "{state}");
    assert!(state.contains(&id));

    let events = send(port, "GET", &format!("/v1/jobs/{id}/events"), "");
    assert!(events.starts_with("HTTP/1.1 200"), "{events}");
    assert!(events.contains("data: "), "{events}");
    assert!(events.contains("\"state\":\"done\""), "{events}");
    assert!(events.contains("\"progress\":"), "{events}");

    let held = send(port, "POST", "/v1/jobs", "{\"kind\":\"fixture-hold\"}");
    assert!(held.starts_with("HTTP/1.1 202"), "{held}");
    let held_id = body_json(&held)["id"].as_str().unwrap().to_string();
    std::thread::sleep(Duration::from_millis(30));
    let cancelled = send(port, "POST", &format!("/v1/jobs/{held_id}/cancel"), "");
    assert!(cancelled.starts_with("HTTP/1.1 200"), "{cancelled}");
    let state = send(port, "GET", &format!("/v1/jobs/{held_id}"), "");
    assert!(state.contains("\"state\":\"cancelled\""), "{state}");
    let events = send(port, "GET", &format!("/v1/jobs/{held_id}/events"), "");
    assert!(events.contains("\"state\":\"cancelled\""), "{events}");

    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
}
