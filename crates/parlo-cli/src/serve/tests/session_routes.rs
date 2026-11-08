use super::*;

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
