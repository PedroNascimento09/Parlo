use super::*;

#[test]
fn unsafe_insertions_and_undo_are_refused_over_http() {
    let (port, stop) = start_server();
    for body in [
        r#"{"text":"hello"}"#,
        r#"{"text":"hello","replace":"old"}"#,
        r#"{"text":"hello","live":true}"#,
        r#"{"text":"","replace":"old","live":true}"#,
    ] {
        let answer = send(port, "POST", "/insert", body);
        assert!(answer.starts_with("HTTP/1.1 409"), "{answer}");
        assert!(!answer.contains("\"inserted\":true"), "{answer}");
    }
    let answer = send(port, "POST", "/undo", "");
    assert!(answer.starts_with("HTTP/1.1 409"), "{answer}");
    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));
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
    std::thread::spawn(move || listen(listener, "t0ken".into(), port, session, accepting, None));

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
fn translated_dictation_uses_the_spoken_language_for_commands() {
    struct CommandEngine(Option<&'static str>);

    impl Engine for CommandEngine {
        fn id(&self) -> &str {
            "command-test"
        }

        fn target_languages(&self) -> Vec<String> {
            vec!["en".to_string()]
        }

        fn latency_ms(&self) -> u32 {
            1
        }

        fn process(&mut self, _input: &Frame) -> Result<EngineOutput, EngineError> {
            Ok(EngineOutput::empty())
        }

        fn write(
            &mut self,
            _pcm16: &[u8],
            _target: Option<&str>,
            _clean: Option<&str>,
        ) -> Result<(String, String), EngineError> {
            Ok((
                "hello comma there vírgula mundo ponto final".to_string(),
                self.0.unwrap_or("").to_string(),
            ))
        }
    }

    let (port, stop) = start_server_with(Session::with_engine(
        Box::new(CommandEngine(Some("translated sentence"))),
        "pt",
        "en",
    ));
    let wav = parlo_core::wav::write_pcm16(&parlo_core::Wav {
        format: parlo_core::AudioFormat::mono(16_000).unwrap(),
        samples: vec![0.1; 3_200],
    });
    let mut request = format!(
        "POST /dictate HTTP/1.1\r\nHost: 127.0.0.1\r\nx-parlo-token: t0ken\r\nx-parlo-write: en\r\nx-parlo-clean: off\r\nx-parlo-style: formal\r\nContent-Length: {}\r\n\r\n",
        wav.len()
    )
    .into_bytes();
    request.extend_from_slice(&wav);
    let answer = String::from_utf8(ask_raw(port, &request)).unwrap();
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    let body = body_json(&answer);
    assert_eq!(body["text"], "Hello comma there, mundo.");
    assert_eq!(body["translation"], "translated sentence");
    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));

    let (port, stop) = start_server_with(Session::with_engine(
        Box::new(CommandEngine(None)),
        "pt",
        "en",
    ));
    let answer = String::from_utf8(ask_raw(port, &request)).unwrap();
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    let body = body_json(&answer);
    assert_eq!(body["text"], "Hello comma there, mundo.");
    assert_eq!(body["translation"], "");
    assert!(body["note"].as_str().unwrap().contains("unavailable"));
    stop.store(false, Ordering::Relaxed);
    let _ = TcpStream::connect(("127.0.0.1", port));

    let (port, stop) = start_server_with(Session::with_engine(
        Box::new(CommandEngine(Some(
            "hello comma there vírgula mundo ponto final",
        ))),
        "pt",
        "en",
    ));
    let mut request = format!(
        "POST /dictate HTTP/1.1\r\nHost: 127.0.0.1\r\nx-parlo-token: t0ken\r\nx-parlo-write: pt\r\nx-parlo-clean: off\r\nx-parlo-style: formal\r\nContent-Length: {}\r\n\r\n",
        wav.len()
    )
    .into_bytes();
    request.extend_from_slice(&wav);
    let answer = String::from_utf8(ask_raw(port, &request)).unwrap();
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    let body = body_json(&answer);
    assert_eq!(body["text"], "Hello comma there, mundo.");
    assert_eq!(body["translation"], "");
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
