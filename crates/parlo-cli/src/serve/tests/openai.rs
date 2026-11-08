use super::*;

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
        .flat_map(|sample| (((*sample).clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes())
        .collect();
    assert_eq!(*heard.lock().unwrap(), expected);

    let rejected = post_transcription(port, &multipart(boundary, "not-a-model", &wav), boundary);
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

    let speech_body = br#"{"model":"tts-1","voice":"alloy","input":"Ola","response_format":"wav"}"#;
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
        .flat_map(|sample| (((*sample).clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes())
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
