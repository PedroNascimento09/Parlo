use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parlo_core::wav::{read_pcm16, write_pcm16};
use parlo_core::{AudioFormat, Engine, Pipeline, PipelineConfig, Wav};

use crate::update;

#[derive(PartialEq, Eq)]
enum Flow {
    Serve,
    Stop,
}

const MAX_HEAD: usize = 16 * 1024;
const MAX_BODY: usize = 20 * 1024 * 1024;
const VERSION: &str = env!("CARGO_PKG_VERSION");
const INDEX: &str = include_str!("../web/index.html");
const STYLESHEET: &str = include_str!("../web/app.css");
const SCRIPT: &str = include_str!("../web/app.js");

pub fn new_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    format!("{nanos:016x}{:08x}", std::process::id())
}

pub fn run(
    port: u16,
    token: String,
    engine: Box<dyn Engine>,
    open_app: bool,
) -> Result<(), String> {
    let format = AudioFormat::mono(16_000).map_err(|error| error.to_string())?;
    let mut pipeline = Pipeline::new(PipelineConfig::new(format, format), engine)
        .map_err(|error| error.to_string())?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|error| format!("cannot bind 127.0.0.1:{port}: {error}"))?;

    println!("listening on http://127.0.0.1:{port}");
    println!("  token      {token}");
    println!("  engine     {}", pipeline.engine_id());
    println!("  input      16 kHz mono wav, POST /translate");
    if open_app {
        let address = format!("http://127.0.0.1:{port}/?t={token}");
        println!("  app        {address}");
        if let Err(error) = open_browser(&address) {
            eprintln!("could not open the browser: {error}, open the app link above");
        }
    }

    let started = Instant::now();
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("accept failed: {error}");
                continue;
            }
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        match handle(&mut stream, &token, &mut pipeline, started) {
            Ok(Flow::Serve) => {}
            Ok(Flow::Stop) => break,
            Err(error) => eprintln!("request failed: {error}"),
        }
    }
    Ok(())
}

fn handle(
    stream: &mut TcpStream,
    token: &str,
    pipeline: &mut Pipeline,
    started: Instant,
) -> Result<Flow, String> {
    let (head, body) = read_http(stream)?;
    let mut lines = head.lines();
    let request = lines.next().unwrap_or_default();
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts
        .next()
        .unwrap_or_default()
        .split('?')
        .next()
        .unwrap_or_default();
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
        .collect();

    if method == "GET" && path == "/update" {
        let latest = match update::latest() {
            Ok(latest) => latest,
            Err(error) => {
                return respond_json(
                    stream,
                    "502 Bad Gateway",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
        let available = update::is_newer(VERSION, &latest.version);
        let body = format!(
            "{{\"current\":\"{VERSION}\",\"latest\":\"{}\",\"available\":{available},\"notes\":\"{}\"}}",
            json_string(&latest.version),
            json_string(&latest.notes_url)
        );
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/update" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let latest = match update::latest() {
            Ok(latest) => latest,
            Err(error) => {
                return respond_json(
                    stream,
                    "502 Bad Gateway",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
        if !update::is_newer(VERSION, &latest.version) {
            return respond_json(stream, "409 Conflict", "{\"error\":\"already up to date\"}")
                .map(|()| Flow::Serve);
        }
        let binary = match update::install(&latest) {
            Ok(binary) => binary,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
        if let Err(error) = update::replace_running(&binary) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        let body = format!(
            "{{\"installed\":true,\"version\":\"{}\"}}",
            json_string(&latest.version)
        );
        respond_json(stream, "200 OK", &body)?;
        return Ok(Flow::Stop);
    }

    if method == "GET" && path == "/" {
        let page = INDEX.replace("__PARLO_VERSION__", VERSION);
        return respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            page.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/app.css" {
        return respond(
            stream,
            "200 OK",
            "text/css; charset=utf-8",
            STYLESHEET.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/app.js" {
        return respond(
            stream,
            "200 OK",
            "text/javascript; charset=utf-8",
            SCRIPT.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if path == "/health" {
        let uptime = started.elapsed().as_secs();
        let engine = pipeline.engine_id();
        let body = format!("{{\"status\":\"ok\",\"uptime_s\":{uptime},\"engine\":\"{engine}\"}}");
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method != "POST" || path != "/translate" {
        return respond_json(stream, "404 Not Found", "{\"error\":\"not found\"}")
            .map(|()| Flow::Serve);
    }
    if !authorised(&headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }

    let wav = match read_pcm16(&body) {
        Ok(wav) => wav,
        Err(error) => {
            return respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"bad wav: {error}\"}}"),
            )
            .map(|()| Flow::Serve);
        }
    };
    if wav.format.channels() != 1 || wav.format.sample_rate() != 16_000 {
        return respond_json(
            stream,
            "400 Bad Request",
            "{\"error\":\"input must be 16 kHz mono pcm16\"}",
        )
        .map(|()| Flow::Serve);
    }

    let mut samples = Vec::new();
    for chunk in wav.samples.chunks(320) {
        for output in pipeline.push(chunk) {
            if let Some(error) = output.error {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
            if let Some(frame) = output.frame {
                samples.extend_from_slice(frame.samples());
            }
        }
    }

    let translated = Wav {
        format: AudioFormat::mono(16_000).map_err(|error| error.to_string())?,
        samples,
    };
    respond(stream, "200 OK", "audio/wav", &write_pcm16(&translated)).map(|()| Flow::Serve)
}

fn json_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

fn open_browser(address: &str) -> Result<(), String> {
    let mut command = if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.args(["/c", "start", ""]);
        command
    } else if cfg!(target_os = "macos") {
        Command::new("open")
    } else {
        Command::new("xdg-open")
    };
    command
        .arg(address)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn read_http(stream: &mut TcpStream) -> Result<(String, Vec<u8>), String> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        let read = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("connection closed before headers".to_string());
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(position) = find(&buffer, b"\r\n\r\n") {
            break position + 4;
        }
        if buffer.len() > MAX_HEAD {
            return Err("headers too large".to_string());
        }
    };

    let head = String::from_utf8_lossy(&buffer[..head_end]).into_owned();
    let length = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    if length > MAX_BODY {
        return Err("body too large".to_string());
    }

    let mut body = buffer[head_end..].to_vec();
    while body.len() < length {
        let read = stream.read(&mut chunk).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(length);
    Ok((head, body))
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn authorised(headers: &[(String, String)], token: &str) -> bool {
    headers.iter().any(|(name, value)| {
        if name == "authorization" {
            value
                .strip_prefix("Bearer ")
                .is_some_and(|candidate| constant_time_eq(candidate.trim(), token))
        } else if name == "x-parlo-token" {
            constant_time_eq(value, token)
        } else {
            false
        }
    })
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

fn respond_json(stream: &mut TcpStream, status: &str, body: &str) -> Result<(), String> {
    respond(stream, status, "application/json", body.as_bytes())
}

fn respond(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .and_then(|()| stream.write_all(body))
        .and_then(|()| stream.flush())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_bearer_token() {
        let headers = vec![("authorization".to_string(), "Bearer abc".to_string())];
        assert!(authorised(&headers, "abc"));
        assert!(!authorised(&headers, "abd"));
    }

    #[test]
    fn accepts_the_short_header() {
        let headers = vec![("x-parlo-token".to_string(), "abc".to_string())];
        assert!(authorised(&headers, "abc"));
    }

    #[test]
    fn rejects_a_missing_token() {
        assert!(!authorised(&[], "abc"));
    }

    #[test]
    fn compares_without_short_circuiting_on_length() {
        assert!(!constant_time_eq("abc", "ab"));
        assert!(constant_time_eq("abc", "abc"));
    }

    #[test]
    fn json_strings_survive_windows_paths_and_quotes() {
        assert_eq!(
            json_string("C:\\temp\\parlo.exe said \"no\""),
            "C:\\\\temp\\\\parlo.exe said \\\"no\\\""
        );
    }

    #[test]
    fn the_app_page_carries_the_running_version() {
        let page = INDEX.replace("__PARLO_VERSION__", VERSION);
        assert!(page.contains(&format!("content=\"{VERSION}\"")));
        assert!(!page.contains("__PARLO_VERSION__"));
        assert!(page.contains("/app.css") && page.contains("/app.js"));
    }

    #[test]
    fn tokens_are_hex_and_unique_enough() {
        let token = new_token();
        assert_eq!(token.len(), 24);
        assert!(token.chars().all(|character| character.is_ascii_hexdigit()));
        assert_ne!(token, new_token());
    }
}
