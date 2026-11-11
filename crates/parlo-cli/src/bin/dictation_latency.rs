use parlo_core::wav::{read_pcm16, write_pcm16};
use parlo_core::{AudioFormat, Wav};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
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
const FAST: Duration = Duration::from_millis(150);
const SLOW: Duration = Duration::from_millis(1_200);

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
        .trim();
    match text.strip_suffix('.') {
        Some(rest) => format!("{rest}!"),
        None => format!("{text}."),
    }
}

fn polish_server(delay: Duration) -> (u16, Arc<AtomicUsize>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let calls = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&calls);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let counted = Arc::clone(&counted);
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
                counted.fetch_add(1, Ordering::Relaxed);
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
    (port, calls)
}

fn parlo_binary() -> std::path::PathBuf {
    let mut path = std::env::current_exe().expect("the probe runs as a binary");
    path.set_file_name(if cfg!(windows) { "parlo.exe" } else { "parlo" });
    path
}

fn serve(port: u16, data: &std::path::Path, polish: u16, reject: bool) -> Serving {
    let child = Command::new(parlo_binary())
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
        .env("PARLO_INTELLIGENCE_LOCAL_MODEL", "test-polish")
        .env("PARLO_FIELD_REJECT", if reject { "1" } else { "0" })
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

fn release(port: u16, samples: &[f32]) -> String {
    let text = feed(port, samples);
    assert!(!text.is_empty(), "the stream heard nothing");
    text
}

fn dictate(port: u16, headers: &str, text: &str, seconds: f64) -> (u128, String) {
    let ended = post(port, "/listen/stream-end", "", b"");
    assert!(ended.starts_with("HTTP/1.1 200"), "{ended}");
    let payload = serde_json::json!({ "text": text, "seconds": seconds }).to_string();
    let began = Instant::now();
    let answer = post(port, "/dictate", headers, payload.as_bytes());
    let elapsed = began.elapsed().as_millis();
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    (elapsed, answer)
}

fn insert_polished(port: u16, text: &str) -> String {
    let payload =
        serde_json::json!({ "text": text, "replace": "", "latency_ms": 4, "polish": true })
            .to_string();
    let answer = post(
        port,
        "/insert",
        "Content-Type: application/json\r\n",
        payload.as_bytes(),
    );
    assert!(answer.starts_with("HTTP/1.1 409"), "{answer}");
    answer
}

fn wait_for_warm(calls: &Arc<AtomicUsize>) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while calls.load(Ordering::Relaxed) == 0 {
        assert!(Instant::now() < deadline, "the polish was never warmed");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn main() {
    let fixture = std::fs::read(FIXTURE).expect("the timing fixture");
    let fixture = read_pcm16(&fixture).expect("the fixture is 16 kHz mono pcm16");
    let audio = as_wav(&fixture.samples);
    let seconds = fixture.samples.len() as f64 / 16_000.0;
    let mut paused = fixture.samples.clone();
    paused.extend(std::iter::repeat_n(0.0, PAUSE));

    let (fast_port, fast_calls) = polish_server(FAST);
    let data = std::env::temp_dir().join(format!("parlo-latency-fast-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let port = free_port();
    let _fast = serve(port, &data, fast_port, false);
    wait_for_warm(&fast_calls);

    let cold = {
        let began = Instant::now();
        let text = release(port, &paused);
        let load = began.elapsed().as_millis();
        let (elapsed, answer) = dictate(port, JSON_OFF, &text, seconds);
        println!(
            "cold, cleanup off: engine {load} ms while speaking, {elapsed} ms from the key release, the server says {} ms, text {:?}",
            latency_of(&answer),
            text_of(&answer)
        );
        assert!(elapsed < 50, "the cold release took {elapsed} ms");
        assert_eq!(text_of(&answer), text);
        (load, elapsed)
    };

    let audio_off = {
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

    let streamed_off = {
        let text = release(port, &paused);
        let (elapsed, answer) = dictate(port, JSON_OFF, &text, seconds);
        println!(
            "streamed, cleanup off: {elapsed} ms from the key release, the server says {} ms, text {:?}",
            latency_of(&answer),
            text_of(&answer)
        );
        assert!(elapsed < 50, "the release took {elapsed} ms");
        elapsed
    };

    let fast_polish = {
        let text = release(port, &paused);
        let (elapsed, answer) = dictate(port, JSON_ON, &text, seconds);
        let polished = text_of(&answer);
        println!(
            "streamed, cleanup on with a {} ms model: the polished text arrives in {elapsed} ms, text {:?}",
            FAST.as_millis(),
            polished
        );
        assert_eq!(
            polished,
            format!("{}!", text.trim_end_matches('.')),
            "{answer}"
        );
        assert!(polished.ends_with('!'), "{answer}");
        assert!(
            elapsed >= 100,
            "the model answered in {elapsed} ms, which is not realistic"
        );
        elapsed
    };

    {
        let text = release(port, &paused);
        let answer = insert_polished(port, &text);
        assert!(body_json(&answer)["inserted"].is_null(), "{answer}");
        println!("Unverified insertion was refused.");
    }

    let audio_on = {
        let began = Instant::now();
        let answer = post(port, "/dictate", WAV_ON, &audio);
        let elapsed = began.elapsed().as_millis();
        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        println!(
            "audio, cleanup on with a {} ms model: {elapsed} ms, text {:?}",
            FAST.as_millis(),
            text_of(&answer)
        );
        assert!(text_of(&answer).ends_with('!'), "{answer}");
        elapsed
    };

    let (slow_port, slow_calls) = polish_server(SLOW);
    let slow_data = std::env::temp_dir().join(format!("parlo-latency-slow-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&slow_data);
    std::fs::create_dir_all(&slow_data).unwrap();
    let slow_serve = free_port();
    let _slow = serve(slow_serve, &slow_data, slow_port, false);
    wait_for_warm(&slow_calls);

    let (slow_cold, slow_polish) = {
        let text = release(slow_serve, &paused);
        let (raw, raw_answer) = dictate(slow_serve, JSON_OFF, &text, seconds);
        println!(
            "slow model, cleanup off: {raw} ms from the key release, text {:?}",
            text_of(&raw_answer)
        );
        assert!(raw < 50, "the release took {raw} ms");
        let text = release(slow_serve, &paused);
        let (polished, polish_answer) = dictate(slow_serve, JSON_ON, &text, seconds);
        println!(
            "slow model, cleanup on: the polish takes {polished} ms, the final text arrives after {polished} ms, text {:?}",
            text_of(&polish_answer)
        );
        assert!(
            polished >= 1_000,
            "the slow model answered in {polished} ms"
        );
        assert!(text_of(&polish_answer).ends_with('!'), "{polish_answer}");
        (raw, polished)
    };

    println!(
        "summary: cold engine {} ms while speaking, cold release {} ms, audio off {} ms, streamed off {} ms, {} ms for polish, audio on {} ms; slow model release {} ms, polish {} ms, warmed {} and {} times",
        cold.0,
        cold.1,
        audio_off,
        streamed_off,
        fast_polish,
        audio_on,
        slow_cold,
        slow_polish,
        fast_calls.load(Ordering::Relaxed),
        slow_calls.load(Ordering::Relaxed)
    );
    let _ = std::fs::remove_dir_all(&data);
    let _ = std::fs::remove_dir_all(&slow_data);
}
