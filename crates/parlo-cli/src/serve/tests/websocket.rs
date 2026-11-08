use super::*;

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
