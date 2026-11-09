use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../engines/cascade/tests/fixtures/timing/source.wav"
);
const WAV_OFF: &str = "Content-Type: audio/wav\r\nx-parlo-clean: off\r\nx-parlo-write: pt\r\n";
const WAV_ON: &str = "Content-Type: audio/wav\r\nx-parlo-clean: on\r\nx-parlo-write: pt\r\n";
const JSON_ON: &str = "Content-Type: application/json\r\nx-parlo-clean: on\r\n";

struct Serving(Child);

impl Drop for Serving {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn free_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.local_addr().unwrap().port()
}

fn serve(port: u16, data: &std::path::Path) -> Serving {
    let child = Command::new(env!("CARGO_BIN_EXE_parlo"))
        .args([
            "serve",
            "--port",
            &port.to_string(),
            "--token",
            "t0ken",
            "--engine",
            "cascade",
            "--stt",
            "parakeet",
            "--from",
            "pt",
            "--to",
            "en",
        ])
        .env("PARLO_DATA_DIR", data)
        .env(
            "PARLO_CASCADE_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../engines/cascade"),
        )
        .env(
            "PARLO_MODELS_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../engines/cascade/models"),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("parlo serve starts");
    let serving = Serving(child);
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let deadline = Instant::now() + Duration::from_secs(60);
    while TcpStream::connect_timeout(&address, Duration::from_millis(500)).is_err() {
        assert!(
            Instant::now() < deadline,
            "parlo serve never listened on 127.0.0.1:{port}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    serving
}

fn ask(port: u16, request: &[u8]) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .unwrap();
    stream.write_all(request).unwrap();
    let mut answer = Vec::new();
    let _ = stream.read_to_end(&mut answer);
    String::from_utf8_lossy(&answer).into_owned()
}

fn post(port: u16, path: &str, headers: &str, body: &[u8]) -> String {
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer t0ken\r\n{headers}Content-Length: {}\r\n\r\n",
        body.len()
    );
    let mut request = head.into_bytes();
    request.extend_from_slice(body);
    ask(port, &request)
}

fn heard(port: u16, wav: &[u8]) -> String {
    post(
        port,
        "/listen/stream",
        "Content-Type: audio/wav\r\nx-parlo-window: 3\r\n",
        wav,
    )
}

fn body_json(answer: &str) -> serde_json::Value {
    let body = answer.split("\r\n\r\n").nth(1).unwrap_or("");
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
}

fn text_of(answer: &str) -> String {
    body_json(answer)["text"].as_str().unwrap_or("").to_string()
}

fn latency_of(answer: &str) -> u64 {
    body_json(answer)["latency_ms"].as_u64().unwrap_or(0)
}

fn report(label: &str, elapsed: u128, answer: &str) {
    println!(
        "{label}: {elapsed} ms from the key release, the server says {} ms, text {:?}",
        latency_of(answer),
        text_of(answer)
    );
}

#[test]
#[ignore = "needs the cascade models and the timing fixture on this machine"]
fn the_dictation_latency_is_measured_with_cleanup_off_and_on() {
    let data = std::env::temp_dir().join(format!("parlo-latency-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let wav = std::fs::read(FIXTURE).expect("the timing fixture");
    let port = free_port();
    let _serving = serve(port, &data);

    let warm = post(port, "/dictate", WAV_OFF, &wav);
    assert!(warm.starts_with("HTTP/1.1 200"), "{warm}");
    println!("warm: the first dictation took {} ms", latency_of(&warm));

    let off = {
        let partial = heard(port, &wav);
        assert!(partial.starts_with("HTTP/1.1 200"), "{partial}");
        let began = Instant::now();
        let answer = post(port, "/dictate", WAV_OFF, &wav);
        let elapsed = began.elapsed().as_millis();
        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        assert!(!text_of(&answer).is_empty(), "{answer}");
        report("cleanup off", elapsed, &answer);
        elapsed
    };

    let on = {
        let partial = heard(port, &wav);
        let text = text_of(&partial);
        if text.is_empty() {
            let began = Instant::now();
            let answer = post(port, "/dictate", WAV_ON, &wav);
            let elapsed = began.elapsed().as_millis();
            assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
            report("cleanup on, audio", elapsed, &answer);
            elapsed
        } else {
            let began = Instant::now();
            let body = serde_json::json!({ "text": text, "seconds": 2.8 }).to_string();
            let answer = post(port, "/dictate", JSON_ON, body.as_bytes());
            let elapsed = began.elapsed().as_millis();
            assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
            assert!(!text_of(&answer).is_empty(), "{answer}");
            report("cleanup on", elapsed, &answer);
            elapsed
        }
    };

    let polished = {
        let partial = heard(port, &wav);
        assert!(partial.starts_with("HTTP/1.1 200"), "{partial}");
        let began = Instant::now();
        let answer = post(port, "/dictate", WAV_ON, &wav);
        let elapsed = began.elapsed().as_millis();
        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        report("cleanup on, audio", elapsed, &answer);
        elapsed
    };

    println!(
        "summary: cleanup off {off} ms, cleanup on {on} ms, cleanup on from audio {polished} ms"
    );
    let _ = std::fs::remove_dir_all(&data);
}
