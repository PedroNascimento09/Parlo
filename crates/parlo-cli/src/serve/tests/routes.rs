use super::*;

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
        if !*auth
            || path.ends_with("transcriptions/stream")
            || *method == "ANY" && path.ends_with('/')
        {
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
fn a_saved_dictation_is_reinserted_into_the_captured_field() {
    struct Tidy;
    impl Drop for Tidy {
        fn drop(&mut self) {
            let _ = crate::history::remove("keep-me");
        }
    }
    let _tidy = Tidy;
    let mut entry = crate::history::Entry::now(
        "Bom dia a todos.",
        "Good morning everyone.",
        "pt",
        "en",
        "Notepad",
        "dictate",
    );
    entry.id = "keep-me".to_string();
    crate::history::append(&entry).unwrap();

    let (port, stop) = start_server();
    let unknown = send(port, "POST", "/history/reinsert", "{\"id\":\"missing\"}");
    assert!(unknown.starts_with("HTTP/1.1 404"), "{unknown}");
    let unnamed = send(port, "POST", "/history/reinsert", "{}");
    assert!(unnamed.starts_with("HTTP/1.1 400"), "{unnamed}");
    let without = send(port, "POST", "/history/reinsert", "{\"id\":\"keep-me\"}");
    assert!(without.starts_with("HTTP/1.1 409"), "{without}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);

    let log_dir = std::env::temp_dir().join(format!("parlo-reinsert-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&log_dir);
    std::fs::create_dir_all(&log_dir).unwrap();
    let log = log_dir.join("field.jsonl");
    let previous_log = std::env::var("PARLO_FIELD_LOG").ok();
    std::env::set_var("PARLO_FIELD_LOG", &log);

    let session = Session::with_engine(
        Box::new(RecordingEngine {
            heard: Arc::new(Mutex::new(Vec::new())),
        }),
        "pt",
        "en",
    );
    *session.target.lock().unwrap() = Some(crate::serve::session::Target {
        window: 7,
        app: "Notepad".to_string(),
    });
    let (port, stop) = start_server_with(session);
    let inserted = send(port, "POST", "/history/reinsert", "{\"id\":\"keep-me\"}");
    assert!(inserted.starts_with("HTTP/1.1 200"), "{inserted}");
    assert_eq!(body_json(&inserted)["inserted"], true);
    let seen = std::fs::read_to_string(&log).unwrap();
    assert!(seen.contains("Bom dia a todos."), "{seen}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);

    match previous_log {
        Some(value) => std::env::set_var("PARLO_FIELD_LOG", value),
        None => std::env::remove_var("PARLO_FIELD_LOG"),
    }
    let _ = std::fs::remove_dir_all(&log_dir);
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

#[test]
fn a_command_transforms_the_selection_and_refuses_unknown_ones() {
    let (port, stop) = start_server();
    let changed = send(
        port,
        "POST",
        "/command",
        "{\"text\":\"we ship. we test.\",\"command\":\"bullets\"}",
    );
    assert!(changed.starts_with("HTTP/1.1 200"), "{changed}");
    assert_eq!(
        body_json(&changed)["text"].as_str().unwrap(),
        "- we ship\n- we test"
    );
    let unknown = send(
        port,
        "POST",
        "/command",
        "{\"text\":\"hello\",\"command\":\"whisper\"}",
    );
    assert!(unknown.starts_with("HTTP/1.1 400"), "{unknown}");
    let empty = send(
        port,
        "POST",
        "/command",
        "{\"text\":\"\",\"command\":\"uppercase\"}",
    );
    assert!(empty.starts_with("HTTP/1.1 400"), "{empty}");
    let no_window = send(port, "POST", "/command/copy", "");
    assert!(no_window.starts_with("HTTP/1.1 400"), "{no_window}");
    stop.store(false, std::sync::atomic::Ordering::SeqCst);
}
