use parlo_core::wav::{read_pcm16, write_pcm16};
use parlo_core::{AudioFormat, Wav};
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
const JSON_OFF: &str =
    "Content-Type: application/json\r\nx-parlo-clean: off\r\nx-parlo-write: pt\r\n";
const JSON_ON: &str =
    "Content-Type: application/json\r\nx-parlo-clean: on\r\nx-parlo-write: pt\r\n";
const WINDOW: &str = "Content-Type: audio/wav\r\nx-parlo-window: 3\r\n";
const CHUNK: usize = 4_800;
const PAUSE: usize = 9_600;

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

fn split_request(buffer: &[u8]) -> (usize, usize) {
    let mut end = None;
    for index in 0..buffer.len().saturating_sub(3) {
        if &buffer[index..index + 4] == b"\r\n\r\n" {
            end = Some(index + 4);
            break;
        }
    }
    let Some(end) = end else {
        return (0, usize::MAX);
    };
    let head = String::from_utf8_lossy(&buffer[..end]).to_lowercase();
    let length = head
        .lines()
        .find_map(|line| line.strip_prefix("content-length:"))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    (end, end + length)
}

fn edited_copy(body: &[u8]) -> String {
    let (end, _) = split_request(body);
    let payload: serde_json::Value =
        serde_json::from_slice(&body[end.min(body.len())..]).unwrap_or(serde_json::Value::Null);
    let text = payload["messages"]
        .as_array()
        .and_then(|messages| messages.last())
        .and_then(|message| message["content"].as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if text.ends_with('.') || text.ends_with('!') || text.ends_with('?') {
        text
    } else {
        format!("{text}.")
    }
}

fn slow_polish(delay: Duration) -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            std::thread::spawn(move || {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
                let mut buffer = Vec::new();
                let mut chunk = [0u8; 4096];
                let mut wanted = usize::MAX;
                loop {
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(read) => buffer.extend_from_slice(&chunk[..read]),
                    }
                    if wanted == usize::MAX {
                        let (end, expected) = split_request(&buffer);
                        if end > 0 {
                            wanted = expected;
                        }
                    }
                    if buffer.len() >= wanted {
                        break;
                    }
                }
                std::thread::sleep(delay);
                let content = serde_json::json!({ "text": edited_copy(&buffer) }).to_string();
                let body =
                    serde_json::json!({ "choices": [{ "message": { "content": content } }] })
                        .to_string();
                let answer = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(answer.as_bytes());
                let _ = stream.flush();
            });
        }
    });
    port
}

fn serve(port: u16, data: &std::path::Path, polish: u16) -> Serving {
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
        .env("PARLO_INTELLIGENCE_MODE", "local")
        .env(
            "PARLO_INTELLIGENCE_LOCAL_URL",
            format!("http://127.0.0.1:{polish}/v1/chat/completions"),
        )
        .env("PARLO_INTELLIGENCE_LOCAL_MODEL", "slow-polish")
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

fn spoken_so_far(answer: &str) -> String {
    let rows = body_json(answer);
    let rows = rows.as_array().cloned().unwrap_or_default();
    let mut settled = String::new();
    for row in &rows {
        let state = row["state"].as_str().unwrap_or("");
        if state == "final" || state == "corrected" {
            settled = row["text"].as_str().unwrap_or("").trim().to_string();
        }
    }
    if settled.is_empty() {
        if let Some(last) = rows.last() {
            settled = last["text"].as_str().unwrap_or("").trim().to_string();
        }
    }
    settled
}

fn as_wav(samples: &[f32]) -> Vec<u8> {
    write_pcm16(&Wav {
        format: AudioFormat::mono(16_000).unwrap(),
        samples: samples.to_vec(),
    })
}

fn feed(port: u16, samples: &[f32]) -> String {
    let mut text = String::new();
    for chunk in samples.chunks(CHUNK) {
        let answer = post(port, "/listen/stream", WINDOW, &as_wav(chunk));
        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        let spoken = spoken_so_far(&answer);
        if !spoken.is_empty() {
            text = spoken;
        }
    }
    text
}

fn release_and_dictate(port: u16, headers: &str, text: &str, seconds: f64) -> (u128, String) {
    let began = Instant::now();
    let ended = post(port, "/listen/stream-end", "", b"");
    assert!(ended.starts_with("HTTP/1.1 200"), "{ended}");
    let closed = spoken_so_far(&ended);
    let body = if closed.is_empty() {
        text
    } else {
        closed.as_str()
    };
    let payload = serde_json::json!({ "text": body, "seconds": seconds }).to_string();
    let answer = post(port, "/dictate", headers, payload.as_bytes());
    let elapsed = began.elapsed().as_millis();
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(!text_of(&answer).is_empty(), "{answer}");
    (elapsed, answer)
}

#[test]
#[ignore = "needs the cascade models and the timing fixture on this machine"]
fn the_dictation_latency_is_measured_with_cleanup_off_and_on() {
    let data = std::env::temp_dir().join(format!("parlo-latency-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let bytes = std::fs::read(FIXTURE).expect("the timing fixture");
    let fixture = read_pcm16(&bytes).expect("the fixture is 16 kHz mono pcm16");
    let audio = as_wav(&fixture.samples);
    let seconds = fixture.samples.len() as f64 / 16_000.0;
    let mut paused = fixture.samples.clone();
    paused.extend(std::iter::repeat_n(0.0, PAUSE));
    let polish = slow_polish(Duration::from_millis(1_200));
    let port = free_port();
    let _serving = serve(port, &data, polish);

    let (cold_load, cold_release) = {
        let began = Instant::now();
        let text = feed(port, &paused);
        let load = began.elapsed().as_millis();
        println!("cold: the engine took {load} ms to load while the speech was arriving");
        assert!(!text.is_empty(), "the stream heard nothing");
        let (elapsed, answer) = release_and_dictate(port, JSON_OFF, &text, seconds);
        println!(
            "cold: {elapsed} ms from the key release, the server says {} ms, text {:?}",
            latency_of(&answer),
            text_of(&answer)
        );
        (load, elapsed)
    };

    let wav_off = {
        let began = Instant::now();
        let answer = post(port, "/dictate", WAV_OFF, &audio);
        let elapsed = began.elapsed().as_millis();
        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        println!(
            "audio, cleanup off: {elapsed} ms, the server says {} ms, text {:?}",
            latency_of(&answer),
            text_of(&answer)
        );
        elapsed
    };

    let wav_on = {
        let began = Instant::now();
        let answer = post(port, "/dictate", WAV_ON, &audio);
        let elapsed = began.elapsed().as_millis();
        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        assert!(text_of(&answer).ends_with('.'), "{answer}");
        println!(
            "audio, cleanup on with a slow model: {elapsed} ms, text {:?}",
            text_of(&answer)
        );
        elapsed
    };

    let streamed = {
        let text = feed(port, &paused);
        assert!(!text.is_empty(), "the stream heard nothing");
        let (elapsed, answer) = release_and_dictate(port, JSON_OFF, &text, seconds);
        println!(
            "streamed, cleanup off: {elapsed} ms from the key release, the server says {} ms, text {:?}",
            latency_of(&answer),
            text_of(&answer)
        );
        elapsed
    };

    let polish_behind_the_insert = {
        let payload = serde_json::json!({ "text": "Bom dia. Isto é uma prova dos modos de tempo do Dói", "seconds": seconds })
            .to_string();
        let began = Instant::now();
        let answer = post(port, "/dictate", JSON_ON, payload.as_bytes());
        let elapsed = began.elapsed().as_millis();
        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        assert!(text_of(&answer).ends_with('.'), "{answer}");
        println!(
            "streamed, cleanup on with a slow model: {elapsed} ms, which is why it never sits in front of the insert"
        );
        elapsed
    };

    println!(
        "summary: cold load {cold_load} ms during the speech, cold release {cold_release} ms; audio off {wav_off} ms, audio on {wav_on} ms; streamed off {streamed} ms; polish {polish_behind_the_insert} ms"
    );
    let _ = std::fs::remove_dir_all(&data);
}
