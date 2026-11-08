use super::*;

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
