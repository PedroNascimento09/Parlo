use super::*;

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

    let recovery = ask(
        port,
        &format!("GET /recovery HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Parlo-Pin: 1234\r\n\r\n"),
    );
    assert!(recovery.starts_with("HTTP/1.1 403"), "{recovery}");

    let models = ask(
        port,
        &format!("GET /v1/models HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nX-Parlo-Pin: 1234\r\n\r\n"),
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
    let unauthenticated = crate::remote::post(&settings, "/translate", "audio/wav", b"").unwrap();
    assert_eq!(unauthenticated.status, 401);

    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let _ = stream
        .write_all(format!("GET /health HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n").as_bytes());
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
