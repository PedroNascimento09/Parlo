use super::discovery::find_venv_python;
use super::*;

#[test]
fn finds_a_project_venv() {
    let root = std::env::temp_dir().join(format!("parlo-venv-{}", std::process::id()));
    let engine_dir = root.join("engines").join("cascade");
    std::fs::create_dir_all(&engine_dir).unwrap();
    let python = find_venv_python(&engine_dir);
    assert_eq!(python, None);

    let executable = if cfg!(windows) {
        "Scripts/python.exe"
    } else {
        "bin/python"
    };
    let expected = root.join(".venv").join(executable);
    std::fs::create_dir_all(expected.parent().unwrap()).unwrap();
    std::fs::write(&expected, b"").unwrap();
    assert_eq!(find_venv_python(&engine_dir), Some(expected.clone()));
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn pcm_roundtrip_preserves_levels() {
    let encoded = encode_pcm16(&[0.0, 0.5, -0.5]);
    let bytes = BASE64.decode(encoded).unwrap();
    let decoded = decode_pcm16(&bytes);
    assert_eq!(decoded.len(), 3);
    assert!((decoded[1] - 0.5).abs() < 1e-3);
    assert!((decoded[2] + 0.5).abs() < 1e-3);
}

#[test]
fn clamps_out_of_range_samples() {
    let bytes = BASE64.decode(encode_pcm16(&[4.0, -4.0])).unwrap();
    let decoded = decode_pcm16(&bytes);
    assert!(decoded[0] <= 1.0 && decoded[0] > 0.99);
    assert!(decoded[1] >= -1.0 && decoded[1] < -0.99);
}

#[test]
fn hello_request_only_carries_the_op() {
    let line = serde_json::to_string(&Request {
        op: "hello",
        seq: None,
        pcm16: None,
        target: None,
        clean: None,
    })
    .unwrap();
    assert_eq!(line, "{\"op\":\"hello\"}");
}

#[test]
fn a_pair_request_names_both_languages() {
    let line = serde_json::to_string(&PairRequest {
        op: "pair",
        seq: 3,
        from: "pt",
        to: "de",
    })
    .unwrap();
    assert_eq!(
        line,
        "{\"op\":\"pair\",\"seq\":3,\"from\":\"pt\",\"to\":\"de\"}"
    );
}

#[test]
fn a_write_request_carries_its_target() {
    let line = serde_json::to_string(&Request {
        op: "write",
        seq: Some(2),
        pcm16: Some("AAA=".to_string()),
        target: Some("en"),
        clean: None,
    })
    .unwrap();
    assert!(line.contains("\"target\":\"en\""));
}

#[test]
fn talks_to_the_python_sidecar_when_present() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../engines/hibiki");
    if !dir.join("serve.py").exists() || Command::new("python").arg("--version").output().is_err() {
        return;
    }

    let mut engine = SidecarEngine::spawn(&dir).expect("sidecar should start");
    assert_eq!(engine.id(), "hibiki");
    assert_eq!(engine.target_languages(), vec!["en".to_string()]);

    let frame = Frame::new(AudioFormat::mono(16_000).unwrap(), vec![0.1; 320]);
    let error = engine.process(&frame).unwrap_err().to_string();
    assert!(
        error.contains("not wired") || error.contains("torch"),
        "unexpected error: {error}"
    );
}

#[test]
fn a_response_without_an_op_is_refused() {
    let parsed: Result<Response, _> = serde_json::from_str(r#"{"text":"hello"}"#);
    assert!(parsed.is_err());
}

#[test]
fn a_bare_op_gets_every_default() {
    let response: Response = serde_json::from_str(r#"{"op":"hello"}"#).unwrap();
    assert_eq!(response.op, "hello");
    assert!(response.pcm16.is_none());
    assert!(response.events.is_empty());
    assert!(response.target_languages.is_empty());
}

#[test]
fn an_odd_trailing_byte_is_ignored() {
    let decoded = decode_pcm16(&[0, 64, 255]);
    assert_eq!(decoded.len(), 1);
}

#[test]
fn non_finite_samples_encode_as_silence() {
    for sample in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let decoded = decode_pcm16(&BASE64.decode(encode_pcm16(&[sample])).unwrap());
        assert!(decoded[0].abs() <= 1.0, "{sample} escaped the range");
    }
}

#[test]
fn parses_sidecar_errors() {
    let response: Response =
        serde_json::from_str(r#"{"op":"error","seq":1,"error":"torch is not installed"}"#).unwrap();
    assert_eq!(response.error.as_deref(), Some("torch is not installed"));
    assert!(response.pcm16.is_none());
}
