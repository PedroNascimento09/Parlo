use super::*;

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
    let doc = include_str!("../../../../../docs/API.md");
    let mut paths: Vec<String> = Vec::new();
    for source in [
        include_str!("../mod.rs"),
        include_str!("../routes/openai.rs"),
        include_str!("../routes/models.rs"),
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
