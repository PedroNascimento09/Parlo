use std::fs;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use parlo_core::engine::EngineError;
use parlo_core::settings::Intelligence;
use parlo_core::wav::{read_pcm16, write_pcm16};
use parlo_core::{AudioFormat, Consent, Pipeline, PipelineConfig, Store, Wav};

use crate::commands;
use crate::dub;
use crate::insert;
use crate::models;
use crate::store::Store as ProjectStore;
use crate::subtitles;
use crate::undo;

use crate::update;

mod audio;
mod auth;
pub(crate) mod http;
mod multipart;
mod routes;
pub(crate) mod sse;
mod ws;
use auth::authorised;
pub use auth::new_token;
use http::{
    content_type, error_json, header, host_is_local, origin_allowed, percent_decode, respond,
    respond_json,
};

#[derive(PartialEq, Eq)]
enum Flow {
    Serve,
    Stop,
}

const MAX_HEAD: usize = 16 * 1024;
const MAX_BODY: usize = 20 * 1024 * 1024;
const MAX_AUDIO_BODY: usize = 25 * 1024 * 1024;
const MAX_DUB_BODY: usize = 512 * 1024 * 1024;
const KEPT_DUBS: usize = 8;

static STATE: std::sync::Mutex<parlo_core::state::Machine> =
    std::sync::Mutex::new(parlo_core::state::Machine::new());
const DUB_TIMEOUT: Duration = Duration::from_secs(3_600);
const VERSION: &str = env!("CARGO_PKG_VERSION");
const INDEX: &str = include_str!("../../web/index.html");
const STYLESHEET: &str = include_str!("../../web/app.css");
const SCRIPT: &str = include_str!("../../web/app.js");
const ISLAND: &str = include_str!("../../web/island.html");
const CAPTIONS: &str = include_str!("../../web/captions.html");
const LIVE: &str = include_str!("../../web/live.html");

#[derive(serde::Deserialize)]
struct Settings {
    #[serde(default)]
    engine: Option<String>,
    #[serde(default)]
    stt: Option<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    write: Option<String>,
    #[serde(default)]
    shortcut: Option<String>,
    #[serde(default)]
    hands_free: Option<String>,
    #[serde(default)]
    captions: Option<String>,
    #[serde(default)]
    undo: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    history_days: Option<u32>,
    #[serde(default)]
    unload: Option<u32>,
    #[serde(default)]
    manage_memory: Option<bool>,
}

#[derive(serde::Deserialize)]
struct Policy {
    id: String,
    #[serde(default)]
    minutes: Option<u32>,
}

struct Chosen {
    engine: String,
    stt: String,
    from: String,
    to: String,
    write: String,
    shortcut: String,
    hands_free: String,
    captions: String,
    undo: String,
}

pub struct Session {
    pipeline: Mutex<Pipeline>,
    settings: RwLock<Chosen>,
    target: Mutex<Option<usize>>,
    busy: AtomicBool,
    known: Mutex<Option<Vec<parlo_core::UnitState>>>,
}

struct Working<'a>(&'a AtomicBool);

impl Drop for Working<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

fn rooms() -> &'static Mutex<std::collections::HashMap<String, crate::speakers::Room>> {
    static ROOMS: std::sync::OnceLock<
        Mutex<std::collections::HashMap<String, crate::speakers::Room>>,
    > = std::sync::OnceLock::new();
    ROOMS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

fn build(from: &str, to: &str, engine: &str, stt: &str) -> Result<Pipeline, String> {
    crate::config::check_pair(from, to)?;
    crate::config::check_stt(stt)?;
    std::env::set_var("PARLO_CASCADE_SOURCE", from);
    std::env::set_var("PARLO_CASCADE_TARGET", to);
    std::env::set_var("PARLO_CASCADE_STT", stt);
    let format = AudioFormat::mono(16_000).map_err(|error| error.to_string())?;
    Pipeline::new(
        PipelineConfig::new(format, format),
        crate::build_engine(engine)?,
    )
    .map_err(|error| error.to_string())
}

impl Session {
    fn pipeline(&self) -> MutexGuard<'_, Pipeline> {
        self.pipeline
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn working(&self) -> Working<'_> {
        self.busy.store(true, Ordering::Relaxed);
        Working(&self.busy)
    }

    fn is_working(&self) -> bool {
        self.busy.load(Ordering::Relaxed)
    }

    fn lifecycle_snapshot(&self) -> models::Lifecycle {
        let manage = crate::config::load().manage_memory;
        if self.is_working() {
            let cached = self.known.lock().map(|known| known.clone()).unwrap_or(None);
            return models::Lifecycle {
                manage,
                units: cached.unwrap_or_default(),
            };
        }
        let units = self.pipeline().lifecycle().unwrap_or_default();
        if let Ok(mut known) = self.known.lock() {
            *known = Some(units.clone());
        }
        models::Lifecycle { manage, units }
    }

    fn load_unit(&self, unit: &str) -> Result<(), String> {
        self.pipeline()
            .load_unit(unit)
            .map_err(|error| error.to_string())
    }

    fn unload_unit(&self, unit: &str) -> Result<(), String> {
        self.pipeline()
            .unload_unit(unit)
            .map_err(|error| error.to_string())
    }

    fn chosen(&self) -> RwLockReadGuard<'_, Chosen> {
        self.settings
            .read()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn choose(&self) -> RwLockWriteGuard<'_, Chosen> {
        self.settings
            .write()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn open(engine: String, stt: String, from: String, to: String) -> Result<Self, String> {
        let pipeline = build(&from, &to, &engine, &stt)?;
        let stored = crate::config::load();
        Ok(Self {
            pipeline: Mutex::new(pipeline),
            settings: RwLock::new(Chosen {
                engine,
                stt,
                from,
                to,
                write: stored.write,
                shortcut: stored.shortcut,
                hands_free: stored.hands_free,
                captions: stored.captions,
                undo: stored.undo,
            }),
            target: Mutex::new(None),
            busy: AtomicBool::new(false),
            known: Mutex::new(None),
        })
    }

    fn switch(
        &self,
        engine: String,
        stt: String,
        from: String,
        to: String,
        write: String,
    ) -> Result<(), String> {
        crate::config::check_pair(&from, &to)?;
        crate::config::check_stt(&stt)?;
        let mut pipeline = self.pipeline();
        let chosen = self.chosen();
        let same_backend = chosen.engine == engine && chosen.stt == stt;
        let same_pair = chosen.from == from && chosen.to == to;
        let (shortcut, hands_free, captions, undo) = (
            chosen.shortcut.clone(),
            chosen.hands_free.clone(),
            chosen.captions.clone(),
            chosen.undo.clone(),
        );
        let mut kept = same_backend;
        if same_backend && !same_pair {
            match pipeline.set_languages(&from, &to) {
                Ok(()) => {
                    std::env::set_var("PARLO_CASCADE_SOURCE", &from);
                    std::env::set_var("PARLO_CASCADE_TARGET", &to);
                }
                Err(error) => {
                    eprintln!(
                        "changing the language pair in place failed, restarting the engine: {error}"
                    );
                    kept = false;
                }
            }
        }
        drop(chosen);
        if !kept {
            *pipeline = build(&from, &to, &engine, &stt)?;
        }
        {
            let mut chosen = self.choose();
            *chosen = Chosen {
                engine,
                stt,
                from,
                to,
                write,
                shortcut,
                hands_free,
                captions,
                undo,
            };
        }
        drop(pipeline);
        if !kept {
            if let Ok(mut target) = self.target.lock() {
                *target = None;
            }
        }
        Ok(())
    }

    fn capture_target(&self) -> Result<(), String> {
        let target = insert::capture_target()?;
        if let Ok(mut stored) = self.target.lock() {
            *stored = Some(target);
        }
        Ok(())
    }

    fn insert_text(&self, text: &str) -> Result<(), String> {
        let target = self
            .target
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
            .ok_or_else(|| "Parlo does not know which field to write into".to_string())?;
        insert::type_text(target, text)?;
        undo::remember(target, text);
        Ok(())
    }

    fn dictate(&self, pcm16: &[u8]) -> Result<String, String> {
        let _working = self.working();
        self.pipeline()
            .dictate(pcm16)
            .map_err(|error| error.to_string())
    }

    fn embed(&self, pcm16: &[u8]) -> Result<Vec<f32>, String> {
        let _working = self.working();
        self.pipeline()
            .embed(pcm16)
            .map_err(|error| error.to_string())
    }

    fn translate_audio(&self, pcm16: &[u8]) -> Result<String, String> {
        let _working = self.working();
        let (_, translation) = self
            .pipeline()
            .write(pcm16, None, None)
            .map_err(|error| error.to_string())?;
        let translation = translation.trim().to_string();
        if translation.is_empty() {
            return Err("no translation model is installed for this language pair".to_string());
        }
        Ok(translation)
    }

    fn speak(&self, text: &str) -> Result<(Vec<u8>, u32), String> {
        let _working = self.working();
        let mut pipeline = self.pipeline();
        pipeline.speak(text).map_err(|error| error.to_string())
    }

    fn diarize(
        &self,
        pcm16: &[u8],
        hint: Option<usize>,
    ) -> Result<crate::diarize::Diarization, String> {
        let _working = self.working();
        let samples: Vec<f32> = pcm16
            .chunks_exact(2)
            .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32_768.0)
            .collect();
        const RATE: usize = 16_000;
        const WINDOW: usize = 24_000;
        const HOP: usize = 12_000;
        const FLOOR: f32 = 0.003;
        if samples.len() < RATE {
            return Err("speaker separation needs at least a second of audio".to_string());
        }
        let mut pipeline = self.pipeline();
        let mut vectors = Vec::new();
        let mut starts = Vec::new();
        let mut ends = Vec::new();
        let mut offset = 0;
        while offset + WINDOW <= samples.len() {
            let window = &samples[offset..offset + WINDOW];
            let energy = window.iter().map(|sample| sample * sample).sum::<f32>() / WINDOW as f32;
            if energy.sqrt() >= FLOOR {
                let mut bytes = Vec::with_capacity(WINDOW * 2);
                for sample in window {
                    bytes.extend_from_slice(
                        &((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes(),
                    );
                }
                match pipeline.embed(&bytes) {
                    Ok(vector) if !vector.is_empty() => {
                        vectors.push(vector);
                        starts.push((offset as u64 * 1_000) / RATE as u64);
                        ends.push(((offset + WINDOW) as u64 * 1_000) / RATE as u64);
                    }
                    Ok(_) => {}
                    Err(EngineError::Rejected(_)) => {}
                    Err(error) => return Err(error.to_string()),
                }
            }
            offset += HOP;
        }
        drop(pipeline);
        let result = crate::diarize::group(&vectors, &starts, &ends, hint);
        if result.turns.is_empty() {
            return Err("no voice was found in that recording".to_string());
        }
        Ok(result)
    }

    fn write(
        &self,
        pcm16: &[u8],
        target: &str,
        clean: Option<&str>,
    ) -> Result<(String, String), String> {
        let _working = self.working();
        let target = if target.is_empty() {
            None
        } else {
            Some(target)
        };
        self.pipeline()
            .write(pcm16, target, clean)
            .map_err(|error| error.to_string())
    }

    fn stream_window(
        &self,
        samples: &[f32],
    ) -> Result<Vec<parlo_core::engine::StreamPiece>, String> {
        let _working = self.working();
        self.pipeline()
            .stream(samples)
            .map_err(|error| error.to_string())
    }

    fn finish_stream(&self) -> Result<Vec<parlo_core::engine::StreamPiece>, String> {
        let _working = self.working();
        self.pipeline()
            .close_stream()
            .map_err(|error| error.to_string())
    }

    fn as_json(&self) -> String {
        let chosen = self.chosen();
        let stored = crate::config::load();
        format!(
            "{{\"engine\":\"{}\",\"stt\":\"{}\",\"from\":\"{}\",\"to\":\"{}\",\"write\":\"{}\",\"shortcut\":\"{}\",\"hands_free\":\"{}\",\"captions\":\"{}\",\"undo\":\"{}\",\"recognisers\":[\"parakeet\",\"whisper\"],\"manage_memory\":{},\"unload\":{},\"name\":\"{}\",\"history_days\":{}}}",
            json_string(&chosen.engine),
            json_string(&chosen.stt),
            json_string(&chosen.from),
            json_string(&chosen.to),
            json_string(&chosen.write),
            json_string(&chosen.shortcut),
            json_string(&chosen.hands_free),
            json_string(&chosen.captions),
            json_string(&chosen.undo),
            stored.manage_memory,
            stored.unload,
            json_string(&stored.name),
            stored.history_days
        )
    }
}

pub struct Server {
    pub address: String,
    pub token: String,
}

impl Server {
    pub fn app_url(&self) -> String {
        format!("{}/?t={}", self.address, self.token)
    }
}

pub fn start(
    port: u16,
    token: String,
    engine: String,
    stt: String,
    from: String,
    to: String,
) -> Result<Server, String> {
    if std::env::var("PARLO_APP_GUARD").is_ok() {
        exit_with_parent();
    }
    let session = Session::open(engine, stt, from, to)?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|error| format!("cannot bind 127.0.0.1:{port}: {error}"))?;

    println!("listening on http://127.0.0.1:{port}");
    println!("  token      {token}");
    {
        let engine_id = session.pipeline().engine_id().to_string();
        let chosen = session.chosen();
        println!("  engine     {engine_id} on {}", chosen.stt);
        println!("  languages  {} to {}", chosen.from, chosen.to);
    }
    println!("  input      16 kHz mono wav, POST /translate");

    let address = format!("http://127.0.0.1:{port}");
    let shared = token.clone();
    let accepting = Arc::new(AtomicBool::new(true));
    std::thread::spawn(move || listen(listener, shared, port, session, accepting));
    Ok(Server { address, token })
}

fn exit_with_parent() {
    std::thread::spawn(|| {
        let mut sink = [0u8; 64];
        let mut stdin = std::io::stdin();
        while matches!(stdin.read(&mut sink), Ok(read) if read > 0) {}
        std::process::exit(0);
    });
}

static OPEN: AtomicUsize = AtomicUsize::new(0);
const MAX_OPEN: usize = 64;

struct Open;

impl Drop for Open {
    fn drop(&mut self) {
        OPEN.fetch_sub(1, Ordering::Relaxed);
    }
}

fn listen(
    listener: TcpListener,
    token: String,
    port: u16,
    session: Session,
    accepting: Arc<AtomicBool>,
) {
    let started = Instant::now();
    let session = Arc::new(session);
    for stream in listener.incoming() {
        if !accepting.load(Ordering::Relaxed) {
            break;
        }
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("accept failed: {error}");
                continue;
            }
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        if OPEN.fetch_add(1, Ordering::Relaxed) >= MAX_OPEN {
            OPEN.fetch_sub(1, Ordering::Relaxed);
            let _ = respond_json(
                &mut stream,
                "503 Service Unavailable",
                "{\"error\":\"too many open connections\"}",
            );
            continue;
        }
        let open = Open;
        let token = token.clone();
        let session = Arc::clone(&session);
        let accepting = Arc::clone(&accepting);
        std::thread::spawn(move || {
            let _open = open;
            match handle(&mut stream, &token, port, &session, started) {
                Ok(Flow::Serve) => {}
                Ok(Flow::Stop) => accepting.store(false, Ordering::Relaxed),
                Err(error) => eprintln!("request failed: {error}"),
            }
        });
    }
}

pub fn run(
    port: u16,
    token: String,
    engine: String,
    stt: String,
    from: String,
    to: String,
) -> Result<(), String> {
    let server = start(port, token, engine, stt, from, to)?;
    println!("  api        {}", server.app_url());
    loop {
        std::thread::park();
    }
}

#[derive(Clone)]
struct DubExport {
    format: String,
    label: String,
    name: String,
    path: PathBuf,
}

#[derive(Clone)]
struct DubFile {
    id: String,
    upload: PathBuf,
    dubbed: PathBuf,
    srt: PathBuf,
    track: PathBuf,
    lines_dir: PathBuf,
    report: PathBuf,
    cues: Option<PathBuf>,
    name: String,
    from: String,
    to: String,
    timing: String,
    fastest: f64,
    slowest: f64,
    project_id: Option<String>,
    keep_background: bool,
    seconds: f64,
    lines: Vec<dub::Line>,
    exports: Vec<DubExport>,
}

fn dub_files() -> &'static Mutex<Vec<DubFile>> {
    static FILES: OnceLock<Mutex<Vec<DubFile>>> = OnceLock::new();
    FILES.get_or_init(|| Mutex::new(Vec::new()))
}

fn remember(file: DubFile) {
    let mut files = match dub_files().lock() {
        Ok(files) => files,
        Err(_) => return,
    };
    files.push(file);
    while files.len() > KEPT_DUBS {
        let oldest = files.remove(0);
        forget_dub(&oldest);
    }
}

fn recall(id: &str) -> Option<(PathBuf, String)> {
    let files = dub_files().lock().ok()?;
    files
        .iter()
        .find(|file| file.id == id)
        .map(|file| (file.dubbed.clone(), file.name.clone()))
}

fn dub_exports(items: &[dub::Export]) -> Vec<DubExport> {
    items
        .iter()
        .filter_map(|item| {
            let path = PathBuf::from(&item.path);
            let name = path.file_name()?.to_string_lossy().to_string();
            Some(DubExport {
                format: item.format.clone(),
                label: item.label.clone(),
                name,
                path,
            })
        })
        .collect()
}

fn dub_body(file: &DubFile) -> String {
    let lines: Vec<String> = file
        .lines
        .iter()
        .map(|line| serde_json::to_string(line).unwrap_or_else(|_| "null".to_string()))
        .collect();
    let project = file
        .project_id
        .as_ref()
        .map(|id| format!(",\"project_id\":\"{}\"", json_string(id)))
        .unwrap_or_default();
    let written: Vec<String> = file
        .exports
        .iter()
        .map(|item| {
            format!(
                "{{\"format\":\"{}\",\"label\":\"{}\",\"name\":\"{}\",\"url\":\"/dub/{}/export/{}\"}}",
                json_string(&item.format),
                json_string(&item.label),
                json_string(&item.name),
                json_string(&file.id),
                json_string(&item.format)
            )
        })
        .collect();
    format!(
        "{{\"id\":\"{}\",\"name\":\"{}\",\"to\":\"{}\",\"timing\":\"{}\",\"fastest\":{},\"slowest\":{},\"seconds\":{:.2},\"lines\":[{}],\"exports\":[{}]{}}}",
        json_string(&file.id),
        json_string(&file.name),
        json_string(&file.to),
        json_string(&file.timing),
        file.fastest,
        file.slowest,
        file.seconds,
        lines.join(","),
        written.join(","),
        project
    )
}

fn forget_dub(file: &DubFile) {
    fs::remove_file(&file.upload).ok();
    fs::remove_file(&file.dubbed).ok();
    fs::remove_file(&file.srt).ok();
    fs::remove_file(&file.track).ok();
    fs::remove_file(&file.report).ok();
    if let Some(path) = &file.cues {
        fs::remove_file(path).ok();
    }
    fs::remove_dir_all(&file.lines_dir).ok();
    for item in &file.exports {
        fs::remove_file(&item.path).ok();
    }
}

fn handle(
    stream: &mut TcpStream,
    token: &str,
    port: u16,
    session: &Session,
    started: Instant,
) -> Result<Flow, String> {
    let (head, body) = read_http(stream)?;
    let mut lines = head.lines();
    let request = lines.next().unwrap_or_default();
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
        .collect();

    if !host_is_local(&headers) {
        return respond_json(
            stream,
            "400 Bad Request",
            "{\"error\":\"host must be the local address\"}",
        )
        .map(|()| Flow::Serve);
    }

    if method == "POST" && !origin_allowed(&headers, port) {
        return respond_json(
            stream,
            "403 Forbidden",
            "{\"error\":\"origin is not the local app\"}",
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/call" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let answer = match crate::calls::detected() {
            Some((app, title)) => {
                serde_json::json!({ "app": app, "title": title.chars().take(80).collect::<String>() })
            }
            None => serde_json::json!({ "app": serde_json::Value::Null }),
        };
        return respond_json(stream, "200 OK", &answer.to_string()).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/meetings" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let meetings: Vec<serde_json::Value> = crate::meeting::list()
            .into_iter()
            .map(meeting_json)
            .collect();
        return respond_json(stream, "200 OK", &serde_json::json!(meetings).to_string())
            .map(|()| Flow::Serve);
    }

    if method == "GET" && path.starts_with("/meetings/") && path.ends_with("/markdown") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = path
            .trim_start_matches("/meetings/")
            .trim_end_matches("/markdown")
            .trim_end_matches('/');
        let meeting = match crate::meeting::load(id) {
            Ok(meeting) => meeting,
            Err(error) => {
                let body = serde_json::json!({ "error": error }).to_string();
                return respond_json(stream, "404 Not Found", &body).map(|()| Flow::Serve);
            }
        };
        let name = crate::notes::file_name(&meeting);
        let body = crate::notes::markdown(&meeting);
        return respond_named(stream, &name, body.as_bytes()).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/meeting/window" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = header(&headers, "x-parlo-meeting")
            .unwrap_or_default()
            .trim()
            .to_string();
        let at_ms = header(&headers, "x-parlo-at")
            .unwrap_or_default()
            .trim()
            .parse::<u64>()
            .unwrap_or(0);
        let mut meeting = match crate::meeting::load(&id) {
            Ok(meeting) => meeting,
            Err(error) => {
                return respond_json(stream, "404 Not Found", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        let wav = match read_pcm16(&body) {
            Ok(wav) => wav,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json(&format!("bad wav: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        if wav.format.channels() != 1
            || wav.format.sample_rate() != 16_000
            || wav.samples.is_empty()
        {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("input must be 16 kHz mono pcm16"),
            )
            .map(|()| Flow::Serve);
        }
        let mut pcm16 = Vec::with_capacity(wav.samples.len() * 2);
        for sample in &wav.samples {
            pcm16.extend_from_slice(&sample.to_le_bytes());
        }
        let text = match session.dictate(&pcm16) {
            Ok(text) => text.trim().to_string(),
            Err(error) => {
                return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        if text.is_empty() {
            return respond_json(stream, "200 OK", "{\"text\":\"\",\"speaker\":\"\"}")
                .map(|()| Flow::Serve);
        }
        let label = match session.embed(&pcm16) {
            Ok(vector) => rooms()
                .lock()
                .ok()
                .and_then(|mut open| {
                    open.entry(id.clone())
                        .or_insert_with(crate::speakers::Room::new)
                        .label(&vector)
                        .ok()
                })
                .unwrap_or_else(|| "Mic".to_string()),
            Err(_) => "Mic".to_string(),
        };
        if let Err(error) = crate::meeting::append(&mut meeting, &label, &text, at_ms) {
            return respond_json(stream, "400 Bad Request", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        for named in crate::speakers::infer(&meeting.segments) {
            let _ = crate::meeting::name_speaker(
                &mut meeting,
                &named.speaker,
                Some(&named.name),
                &named.evidence,
            );
        }
        return match crate::meeting::save(&meeting) {
            Ok(()) => respond_json(
                stream,
                "200 OK",
                &format!(
                    "{{\"text\":{},\"speaker\":{},\"meeting\":{}}}",
                    json_string(&text),
                    json_string(&label),
                    meeting_json(meeting)
                ),
            )
            .map(|()| Flow::Serve),
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/meeting/summary" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
        let id = request
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string();
        let meeting = if id.is_empty() {
            crate::meeting::list().into_iter().next()
        } else {
            crate::meeting::load(&id).ok()
        };
        let Some(meeting) = meeting else {
            return respond_json(
                stream,
                "404 Not Found",
                "{\"error\":\"there is no meeting to summarise\"}",
            )
            .map(|()| Flow::Serve);
        };
        let settings = crate::config::load().intelligence;
        if settings.local_url.trim().is_empty() {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"no model is configured on this machine. Set the local model address in Settings, Cleanup, and the summary stays here\"}",
            )
            .map(|()| Flow::Serve);
        }
        let model = if settings.local_model.trim().is_empty() {
            "local-model".to_string()
        } else {
            settings.local_model.clone()
        };
        let text = crate::summary::prompt(&meeting.title, &meeting.segments);
        return match crate::summary::ask(&settings.local_url, &model, &text) {
            Ok(summary) => respond_json(
                stream,
                "200 OK",
                &format!(
                    "{{\"meeting\":\"{}\",\"summary\":\"{}\"}}",
                    json_string(&meeting.id),
                    json_string(&summary)
                ),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "502 Bad Gateway", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/meetings/clear" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match crate::meeting::clear() {
            Ok(()) => respond_json(stream, "200 OK", "{\"cleared\":true}").map(|()| Flow::Serve),
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "POST"
        && (path == "/meeting/import" || path == "/meeting/name" || path == "/meeting/delete")
    {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let field = |name: &str| {
            request
                .get(name)
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .trim()
                .to_string()
        };

        if path == "/meeting/import" {
            let meeting = match crate::meeting::import(&field("title"), &field("text")) {
                Ok(meeting) => meeting,
                Err(error) => {
                    return respond_json(stream, "400 Bad Request", &error_json(&error))
                        .map(|()| Flow::Serve);
                }
            };
            if let Err(error) = crate::meeting::save(&meeting) {
                return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
            return respond_json(stream, "200 OK", &meeting_json(meeting).to_string())
                .map(|()| Flow::Serve);
        }

        if path == "/meeting/delete" {
            return match crate::meeting::delete(&field("id")) {
                Ok(()) => {
                    respond_json(stream, "200 OK", "{\"deleted\":true}").map(|()| Flow::Serve)
                }
                Err(error) => respond_json(stream, "400 Bad Request", &error_json(&error))
                    .map(|()| Flow::Serve),
            };
        }

        let mut meeting = match crate::meeting::load(&field("id")) {
            Ok(meeting) => meeting,
            Err(error) => {
                return respond_json(stream, "404 Not Found", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        let name = field("name");
        let label = if name.is_empty() {
            None
        } else {
            Some(name.as_str())
        };
        let outcome = crate::meeting::name_speaker(
            &mut meeting,
            &field("speaker"),
            label,
            &field("evidence"),
        );
        return match outcome {
            Ok(_) => match crate::meeting::save(&meeting) {
                Ok(()) => respond_json(stream, "200 OK", &meeting_json(meeting).to_string())
                    .map(|()| Flow::Serve),
                Err(error) => {
                    respond_json(stream, "500 Internal Server Error", &error_json(&error))
                        .map(|()| Flow::Serve)
                }
            },
            Err(error) => {
                respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && (path == "/meeting/start" || path == "/meeting/segment") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let field = |name: &str| {
            request
                .get(name)
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .trim()
                .to_string()
        };

        if path == "/meeting/start" {
            let meeting = crate::meeting::start(&field("title"));
            if let Ok(mut open) = rooms().lock() {
                open.remove(&meeting.id);
            }
            if let Err(error) = crate::meeting::save(&meeting) {
                return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
            return respond_json(stream, "200 OK", &meeting_json(meeting).to_string())
                .map(|()| Flow::Serve);
        }

        let mut meeting = match crate::meeting::load(&field("id")) {
            Ok(meeting) => meeting,
            Err(error) => {
                return respond_json(stream, "404 Not Found", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        let at_ms = request
            .get("at_ms")
            .and_then(|value| value.as_u64())
            .unwrap_or_default();
        if let Err(error) =
            crate::meeting::append(&mut meeting, &field("speaker"), &field("text"), at_ms)
        {
            return respond_json(stream, "400 Bad Request", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        return match crate::meeting::save(&meeting) {
            Ok(()) => respond_json(stream, "200 OK", &meeting_json(meeting).to_string())
                .map(|()| Flow::Serve),
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && (path == "/meeting/ask" || path == "/memory/ask") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let question = request
            .get("question")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim();
        if question.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("ask a question first"),
            )
            .map(|()| Flow::Serve);
        }
        let meetings = if path == "/memory/ask" {
            crate::meeting::list()
        } else {
            let id = request
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let chosen = if id.is_empty() {
                crate::meeting::list().into_iter().next()
            } else {
                crate::meeting::load(id).ok()
            };
            chosen.into_iter().collect()
        };
        let answer = crate::ask::ask(question, &meetings);
        let body =
            serde_json::to_string(&answer).unwrap_or_else(|error| error_json(&error.to_string()));
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/vocabulary" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match crate::intelligence::run(&serde_json::json!({ "action": "vocabulary" })) {
            Ok(reply) => respond_json(stream, "200 OK", &reply.to_string()).map(|()| Flow::Serve),
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && (path == "/vocabulary" || path == "/vocabulary/delete") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let field = |name: &str| {
            request
                .get(name)
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let action = if path == "/vocabulary" {
            "vocabulary/add"
        } else {
            "vocabulary/delete"
        };
        let payload = serde_json::json!({
            "action": action,
            "kind": field("kind"),
            "wrong": field("wrong"),
            "right": field("right"),
        });
        return match crate::intelligence::run(&payload) {
            Ok(reply) => match reply.get("error").and_then(serde_json::Value::as_str) {
                Some(error) => respond_json(stream, "400 Bad Request", &error_json(error))
                    .map(|()| Flow::Serve),
                None => respond_json(stream, "200 OK", &reply.to_string()).map(|()| Flow::Serve),
            },
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/vocabulary/accept" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let heard = text_field(&request, "heard");
        let written = text_field(&request, "written");
        if heard.is_empty() || written.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"a suggestion names what was heard and what to write\"}",
            )
            .map(|()| Flow::Serve);
        }
        let payload = serde_json::json!({
            "action": "vocabulary/add",
            "kind": "term",
            "wrong": heard,
            "right": written,
        });
        return match crate::intelligence::run(&payload) {
            Ok(reply) => match reply.get("error").and_then(serde_json::Value::as_str) {
                Some(error) => respond_json(stream, "400 Bad Request", &error_json(error))
                    .map(|()| Flow::Serve),
                None => respond_json(stream, "200 OK", &reply.to_string()).map(|()| Flow::Serve),
            },
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "GET" && path == "/vocabulary/export" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match crate::intelligence::run(&serde_json::json!({ "action": "vocabulary/export" }))
        {
            Ok(reply) => {
                let text = reply
                    .get("text")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                respond_named(stream, "parlo-words.tsv", text.as_bytes()).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/vocabulary/import" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let text = text_field(&request, "text");
        if text.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"an import needs at least one line\"}",
            )
            .map(|()| Flow::Serve);
        }
        let payload = serde_json::json!({ "action": "vocabulary/import", "text": text });
        return match crate::intelligence::run(&payload) {
            Ok(reply) => match reply.get("error").and_then(serde_json::Value::as_str) {
                Some(error) => respond_json(stream, "400 Bad Request", &error_json(error))
                    .map(|()| Flow::Serve),
                None => respond_json(stream, "200 OK", &reply.to_string()).map(|()| Flow::Serve),
            },
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "GET" && path == "/features" {
        let features = crate::config::load().features;
        let body = serde_json::to_string(&features).unwrap_or_else(|_| "{}".to_string());
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/features" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let requested: parlo_core::settings::Features = match serde_json::from_slice(&body) {
            Ok(features) => features,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        let mut config = crate::config::load();
        config.features = requested;
        if let Err(error) = crate::config::save(&config) {
            return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        let body = serde_json::to_string(&config.features).unwrap_or_else(|_| "{}".to_string());
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/style" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        let style = request
            .get("style")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if !style.is_empty() && !crate::style::is_known(&style) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"style needs formal, casual or excited\"}",
            )
            .map(|()| Flow::Serve);
        }
        let mut config = crate::config::load();
        config.style = style;
        if let Err(error) = crate::config::save(&config) {
            return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        let body = format!("{{\"style\":\"{}\"}}", json_string(&config.style));
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if path == "/profiles" && (method == "GET" || method == "POST") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let mut config = crate::config::load();
        if method == "POST" {
            let request: serde_json::Value = match serde_json::from_slice(&body) {
                Ok(request) => request,
                Err(error) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &error_json(&error.to_string()),
                    )
                    .map(|()| Flow::Serve);
                }
            };
            let program = text_field(&request, "program").to_ascii_lowercase();
            let style = text_field(&request, "style").to_ascii_lowercase();
            let write = text_field(&request, "write").to_ascii_lowercase();
            if program.is_empty() {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"a profile names the program it applies to\"}",
                )
                .map(|()| Flow::Serve);
            }
            if !style.is_empty() && !crate::style::is_known(&style) {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"style needs formal, casual or excited\"}",
                )
                .map(|()| Flow::Serve);
            }
            if !write.is_empty() && !crate::config::check_language(&write) {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"write needs a two letter language code\"}",
                )
                .map(|()| Flow::Serve);
            }
            if style.is_empty() && write.is_empty() {
                config.profiles.remove(&program);
            } else {
                config
                    .profiles
                    .insert(program, parlo_core::settings::Profile { style, write });
            }
            if let Err(error) = crate::config::save(&config) {
                return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        }
        let body = serde_json::json!(config.profiles).to_string();
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/history/restyle" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let (id, style) = match restyle_fields(&request) {
            Ok(fields) => fields,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        return match crate::history::restyle(&id, &style) {
            Ok(entry) => respond_json(stream, "200 OK", &crate::history::entry_json(&entry))
                .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "404 Not Found", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "GET" && path == "/hardware" {
        return respond_json(stream, "200 OK", &crate::hardware::detect().as_json())
            .map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/catalog" {
        return respond_json(stream, "200 OK", &crate::catalog::as_json()).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/history" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let asked = parameter(query, "q");
        let body = if asked.is_empty() {
            crate::history::as_json(20)
        } else {
            crate::history::search_json(&asked, 50)
        };
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/history/export" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let days = parameter(query, "days").parse::<u32>().unwrap_or(0);
        return match crate::history::export(days) {
            Ok(body) => {
                respond_named(stream, "parlo-dictations.md", body.as_bytes()).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/history/clear" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match crate::history::clear() {
            Ok(()) => respond_json(stream, "200 OK", "{\"cleared\":true}").map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST"
        && (path == "/history/favourite" || path == "/history/edit" || path == "/history/delete")
    {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let id = text_field(&request, "id");
        if id.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"a dictation is named by its id\"}",
            )
            .map(|()| Flow::Serve);
        }
        let outcome: Result<String, String> = if path == "/history/favourite" {
            let value = request
                .get("value")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            crate::history::favourite(&id, value).map(|entry| crate::history::entry_json(&entry))
        } else if path == "/history/edit" {
            let text = text_field(&request, "text");
            crate::history::edit_with_suggestions(&id, &text).map(|(entry, proposed)| {
                let mut value: serde_json::Value =
                    serde_json::from_str(&crate::history::entry_json(&entry)).unwrap_or_default();
                value["suggestions"] = serde_json::json!(proposed
                    .iter()
                    .map(|(heard, written)| serde_json::json!({
                        "heard": heard,
                        "written": written
                    }))
                    .collect::<Vec<_>>());
                value.to_string()
            })
        } else {
            crate::history::remove(&id).map(|()| "{\"deleted\":true}".to_string())
        };
        return match outcome {
            Ok(answer) => respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "404 Not Found", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "GET" && path == "/settings" {
        return respond_json(stream, "200 OK", &session.as_json()).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/settings" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let requested: Settings = match serde_json::from_slice(&body) {
            Ok(settings) => settings,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        let wanted_keys = [
            ("dictation", requested.shortcut.as_deref()),
            ("hands free", requested.hands_free.as_deref()),
            ("captions", requested.captions.as_deref()),
            ("undo", requested.undo.as_deref()),
        ];
        if wanted_keys.iter().any(|(_, text)| text.is_some()) {
            let mut config = crate::config::load();
            let before = [
                config.shortcut.clone(),
                config.hands_free.clone(),
                config.captions.clone(),
                config.undo.clone(),
            ];
            let mut after = before.clone();
            for (index, (label, text)) in wanted_keys.iter().enumerate() {
                let Some(text) = *text else {
                    continue;
                };
                let parsed = match parlo_core::Shortcut::parse(text) {
                    Ok(shortcut) => shortcut,
                    Err(error) => {
                        return respond_json(
                            stream,
                            "400 Bad Request",
                            &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                        )
                        .map(|()| Flow::Serve)
                    }
                };
                let canonical = parsed.text();
                if let Some(other) =
                    (0..after.len()).find(|other| *other != index && after[*other] == canonical)
                {
                    let message = format!(
                        "{} is already the {} shortcut, so {label} needs another one.",
                        canonical,
                        ["dictation", "hands free", "captions", "undo"][other]
                    );
                    return respond_json(
                        stream,
                        "409 Conflict",
                        &format!("{{\"error\":\"{}\"}}", json_string(&message)),
                    )
                    .map(|()| Flow::Serve);
                }
                if canonical != before[index]
                    && parlo_core::shortcut::taken_by_another_program(&parsed)
                {
                    let message = format!("{canonical} is already used by another program.");
                    return respond_json(
                        stream,
                        "409 Conflict",
                        &format!("{{\"error\":\"{}\"}}", json_string(&message)),
                    )
                    .map(|()| Flow::Serve);
                }
                after[index] = canonical;
            }
            config.shortcut = after[0].clone();
            config.hands_free = after[1].clone();
            config.captions = after[2].clone();
            config.undo = after[3].clone();
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
            {
                let mut chosen = session.choose();
                chosen.shortcut = after[0].clone();
                chosen.hands_free = after[1].clone();
                chosen.captions = after[2].clone();
                chosen.undo = after[3].clone();
            }
            let only_shortcut = requested.engine.is_none()
                && requested.stt.is_none()
                && requested.from.is_none()
                && requested.to.is_none()
                && requested.write.is_none()
                && requested.unload.is_none();
            if only_shortcut {
                return respond_json(stream, "200 OK", &session.as_json()).map(|()| Flow::Serve);
            }
        }

        if let Some(minutes) = requested.unload {
            if minutes > 1_440 {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"the models cannot be kept for longer than a day\"}",
                )
                .map(|()| Flow::Serve);
            }
            let mut config = crate::config::load();
            config.unload = minutes;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        if let Some(days) = requested.history_days {
            if days > 3_650 {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"the history cannot be kept for longer than ten years\"}",
                )
                .map(|()| Flow::Serve);
            }
            let mut config = crate::config::load();
            config.history_days = days;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
            let _ = crate::history::prune(days);
        }

        if let Some(name) = requested.name {
            let cleaned = name.split_whitespace().collect::<Vec<_>>().join(" ");
            if cleaned.chars().count() > 40 {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"a name cannot be longer than 40 characters\"}",
                )
                .map(|()| Flow::Serve);
            }
            let mut config = crate::config::load();
            config.name = cleaned;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        if let Some(manage) = requested.manage_memory {
            let mut config = crate::config::load();
            config.manage_memory = manage;
            if let Err(error) = crate::config::save(&config) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }

        let only_settings = requested.engine.is_none()
            && requested.stt.is_none()
            && requested.from.is_none()
            && requested.to.is_none()
            && requested.write.is_none()
            && requested.unload.is_none()
            && requested.manage_memory.is_none();
        if only_settings {
            return respond_json(stream, "200 OK", &session.as_json()).map(|()| Flow::Serve);
        }

        let engine = requested
            .engine
            .unwrap_or_else(|| session.chosen().engine.clone());
        let stt = requested
            .stt
            .unwrap_or_else(|| session.chosen().stt.clone());
        let from = requested
            .from
            .unwrap_or_else(|| session.chosen().from.clone())
            .to_ascii_lowercase();
        let to = requested
            .to
            .unwrap_or_else(|| session.chosen().to.clone())
            .to_ascii_lowercase();

        let write = requested
            .write
            .unwrap_or_else(|| session.chosen().write.clone())
            .to_ascii_lowercase();
        if !write.is_empty() && !crate::config::check_language(&write) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"the language for dictation needs a two letter code\"}",
            )
            .map(|()| Flow::Serve);
        }

        println!("switching to {engine} on {stt}, {from} to {to}");
        let _ = stream.set_write_timeout(Some(DUB_TIMEOUT));
        let stored = (
            engine.clone(),
            stt.clone(),
            from.clone(),
            to.clone(),
            write.clone(),
        );
        let outcome = session.switch(engine, stt, from, to, write);
        if outcome.is_ok() {
            let mut config = crate::config::load();
            config.engine = stored.0;
            config.stt = stored.1;
            config.from = stored.2;
            config.to = stored.3;
            config.write = stored.4;
            if let Err(error) = crate::config::save(&config) {
                println!("could not store the settings: {error}");
            }
        }
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        if let Err(error) = outcome {
            return respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }

        {
            let engine_id = session.pipeline().engine_id().to_string();
            let chosen = session.chosen();
            println!(
                "  ready       {engine_id} on {}, {} to {}",
                chosen.stt, chosen.from, chosen.to
            );
        }
        return respond_json(stream, "200 OK", &session.as_json()).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/intelligence" {
        let settings = crate::config::load().intelligence;
        let state = crate::intelligence::state_json(&settings, &crate::intelligence::status());
        return respond_json(stream, "200 OK", &state).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/intelligence" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let mut requested: Intelligence = match serde_json::from_slice(&body) {
            Ok(settings) => settings,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        requested.mode = requested.mode.trim().to_ascii_lowercase();
        requested.clean = requested.clean.trim().to_ascii_lowercase();
        requested.provider = requested.provider.trim().to_ascii_lowercase();
        requested.model = requested.model.trim().to_string();
        requested.base_url = requested.base_url.trim().trim_end_matches('/').to_string();
        requested.local_url = requested.local_url.trim().to_string();
        requested.local_model = requested.local_model.trim().to_string();
        if let Err(error) = crate::intelligence::check(&requested) {
            return respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        let mut config = crate::config::load();
        config.intelligence = requested;
        if let Err(error) = crate::config::save(&config) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        let state =
            crate::intelligence::state_json(&config.intelligence, &crate::intelligence::status());
        return respond_json(stream, "200 OK", &state).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/intelligence/key" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: KeyRequest = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        if !crate::intelligence::PROVIDERS.contains(&request.provider.as_str()) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"choose a provider before saving a key\"}",
            )
            .map(|()| Flow::Serve);
        }
        if request.key.trim().is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"a key is needed to store one\"}",
            )
            .map(|()| Flow::Serve);
        }
        let payload = serde_json::json!({
            "action": "store",
            "provider": request.provider,
            "key": request.key,
        });
        return answer_with(
            stream,
            crate::intelligence::run(&payload),
            "{\"stored\":true}",
        );
    }

    if method == "POST" && path == "/intelligence/key/delete" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: KeyRequest = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        if !crate::intelligence::PROVIDERS.contains(&request.provider.as_str()) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"choose a provider before removing a key\"}",
            )
            .map(|()| Flow::Serve);
        }
        let payload = serde_json::json!({
            "action": "delete",
            "provider": request.provider,
        });
        return answer_with(
            stream,
            crate::intelligence::run(&payload),
            "{\"deleted\":true}",
        );
    }

    if method == "POST" && path == "/intelligence/test" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let payload = serde_json::json!({ "action": "test" });
        return match crate::intelligence::run(&payload) {
            Ok(reply) => match reply.get("error").and_then(serde_json::Value::as_str) {
                Some(error) => respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(error)),
                )
                .map(|()| Flow::Serve),
                None => respond_json(stream, "200 OK", &reply.to_string()).map(|()| Flow::Serve),
            },
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/dub/import" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let requested = header(&headers, "x-parlo-filename")
            .map(|value| percent_decode(&value))
            .unwrap_or_else(|| "subtitles.srt".to_string());
        let name = dub::safe_name(&requested);
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".srt") && !lower.ends_with(".vtt") {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("choose an .srt or .vtt subtitle file"),
            )
            .map(|()| Flow::Serve);
        }
        let stored = crate::config::load();
        let from = header(&headers, "x-parlo-from")
            .unwrap_or_else(|| stored.from.clone())
            .to_ascii_lowercase();
        let to = header(&headers, "x-parlo-to")
            .unwrap_or_else(|| stored.to.clone())
            .to_ascii_lowercase();
        if let Err(error) = crate::config::check_pair(&from, &to) {
            return respond_json(stream, "400 Bad Request", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        let mut project = match subtitles::project(new_token(), &name, &body, &from, &to) {
            Ok(project) => project,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        let mut store = match ProjectStore::open_default() {
            Ok(store) => store,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot open the project store: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        if let Err(error) = store.save_dub_project(&mut project) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot save the imported project: {error}")),
            )
            .map(|()| Flow::Serve);
        }
        return match project_json(&project) {
            Ok(answer) => respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot encode the imported project: {error}")),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "GET" && path.starts_with("/dub/project/") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = path
            .trim_start_matches("/dub/project/")
            .trim_end_matches('/');
        if id.is_empty() {
            return respond_json(stream, "404 Not Found", &error_json("project not found"))
                .map(|()| Flow::Serve);
        }
        let store = match ProjectStore::open_default() {
            Ok(store) => store,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot open the project store: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        return match store.load_dub_project(id) {
            Ok(Some(project)) => match project_json(&project) {
                Ok(answer) => respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve),
                Err(error) => respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot encode the project: {error}")),
                )
                .map(|()| Flow::Serve),
            },
            Ok(None) => respond_json(stream, "404 Not Found", &error_json("project not found"))
                .map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot load the project: {error}")),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path.starts_with("/dub/project/") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = path
            .trim_start_matches("/dub/project/")
            .trim_end_matches('/');
        if id.is_empty() {
            return respond_json(stream, "404 Not Found", &error_json("project not found"))
                .map(|()| Flow::Serve);
        }
        let edit: ProjectEdit = match serde_json::from_slice(&body) {
            Ok(edit) => edit,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json(&format!("the project edit is not valid JSON: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        let mut store = match ProjectStore::open_default() {
            Ok(store) => store,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot open the project store: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        let mut project = match store.load_dub_project(id) {
            Ok(Some(project)) => project,
            Ok(None) => {
                return respond_json(stream, "404 Not Found", &error_json("project not found"))
                    .map(|()| Flow::Serve);
            }
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot load the project: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        if let Err(error) = apply_project_edits(&mut project, &edit.segments, &edit.cast) {
            return respond_json(stream, "400 Bad Request", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        if let Err(error) = store.save_dub_project(&mut project) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot save the project: {error}")),
            )
            .map(|()| Flow::Serve);
        }
        return match project_json(&project) {
            Ok(answer) => respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot encode the project: {error}")),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path.starts_with("/dub/") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let rest = path.trim_start_matches("/dub/");
        let Some((id, action)) = rest.rsplit_once('/') else {
            return respond_json(stream, "404 Not Found", &error_json("not found"))
                .map(|()| Flow::Serve);
        };
        if action != "retry" && action != "skip" {
            return respond_json(stream, "404 Not Found", &error_json("not found"))
                .map(|()| Flow::Serve);
        }
        let Some(line_id) = header(&headers, "x-parlo-line")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        else {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("a line id is required"),
            )
            .map(|()| Flow::Serve);
        };
        let state = match dub_files().lock() {
            Ok(files) => files.iter().find(|file| file.id == id).cloned(),
            Err(_) => None,
        };
        let Some(state) = state else {
            return respond_json(stream, "404 Not Found", &error_json("not found"))
                .map(|()| Flow::Serve);
        };
        if !state
            .lines
            .iter()
            .any(|line| line.id.as_deref() == Some(line_id.as_str()))
        {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json(&format!("line {line_id} is not part of this render")),
            )
            .map(|()| Flow::Serve);
        }
        if state.cues.is_none() {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("retrying a line needs imported subtitles"),
            )
            .map(|()| Flow::Serve);
        }
        let retry = (action == "retry").then(|| line_id.clone());
        let skip = (action == "skip").then(|| line_id.clone());
        let _ = stream.set_write_timeout(Some(DUB_TIMEOUT));
        let result = dub::run(&dub::Request {
            source: &state.upload,
            from: &state.from,
            to: &state.to,
            out: Some(state.dubbed.clone()),
            srt: Some(state.srt.clone()),
            cues: state.cues.clone(),
            track: Some(state.track.clone()),
            lines_dir: Some(state.lines_dir.clone()),
            report: Some(state.report.clone()),
            previous: Some(state.report.clone()),
            retry,
            skip,
            keep_background: state.keep_background,
            timing: &state.timing,
            fastest: state.fastest,
            slowest: state.slowest,
        });
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        return match result {
            Ok(result) => {
                let mut updated = state;
                updated.lines = result.lines;
                updated.seconds = result.seconds;
                updated.exports = dub_exports(&result.exports);
                if let Ok(mut files) = dub_files().lock() {
                    if let Some(slot) = files.iter_mut().find(|file| file.id == updated.id) {
                        *slot = updated.clone();
                    }
                }
                if let Some(project_id) = updated.project_id.as_deref() {
                    if let Ok(mut store) = ProjectStore::open_default() {
                        if let Ok(Some(mut project)) = store.load_dub_project(project_id) {
                            apply_dub(&mut project, &updated.lines, &updated.to);
                            if let Err(error) = store.save_dub_project(&mut project) {
                                return respond_json(
                                    stream,
                                    "500 Internal Server Error",
                                    &error_json(&format!(
                                        "the line changed, but the project could not be saved: {error}"
                                    )),
                                )
                                .map(|()| Flow::Serve);
                            }
                        }
                    }
                }
                respond_json(stream, "200 OK", &dub_body(&updated)).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "GET" && path.starts_with("/dub/") {
        let rest = path.trim_start_matches("/dub/");
        if let Some((id, format)) = rest.split_once("/export/") {
            let found = match dub_files().lock() {
                Ok(files) => files.iter().find(|file| file.id == id).and_then(|file| {
                    file.exports
                        .iter()
                        .find(|item| item.format == format)
                        .map(|item| (item.path.clone(), item.name.clone()))
                }),
                Err(_) => None,
            };
            let Some((file, name)) = found else {
                return respond_json(stream, "404 Not Found", &error_json("not found"))
                    .map(|()| Flow::Serve);
            };
            return respond_file(stream, &file, &name).map(|()| Flow::Serve);
        }
        let id = rest;
        let Some((file, name)) = recall(id) else {
            return respond_json(stream, "404 Not Found", "{\"error\":\"not found\"}")
                .map(|()| Flow::Serve);
        };
        return respond_file(stream, &file, &name).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/voice" {
        return match voice_store().list() {
            Ok(profiles) => {
                let listed = serde_json::to_string(&profiles).unwrap_or_else(|_| "[]".to_string());
                respond_json(stream, "200 OK", &format!("{{\"profiles\":{listed}}}"))
                    .map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/voice" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: VoiceRequest = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        let purpose = request
            .purpose
            .unwrap_or_else(|| "local voice cloning and playback".to_string());
        let scope = request
            .scope
            .unwrap_or_else(|| "on this computer".to_string());
        let consent = match consent_for(
            request.origin.as_deref(),
            request.declared_authorisation,
            purpose,
            scope,
            now_seconds(),
        ) {
            Ok(consent) => consent,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
        let language = request.language.unwrap_or_else(|| "pt-PT".to_string());
        let store = voice_store();
        let created = store
            .create(&request.label, &language, consent)
            .and_then(|profile| {
                if request.encrypt {
                    store.set_encryption(&profile.id, true)
                } else {
                    Ok(profile)
                }
            });
        return match created {
            Ok(profile) => {
                let body = serde_json::to_string(&profile).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "201 Created", &body).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && (path == "/voice/revoke" || path == "/voice/delete") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let target: VoiceTarget = match serde_json::from_slice(&body) {
            Ok(target) => target,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        let store = voice_store();
        let outcome = if path == "/voice/revoke" {
            store
                .revoke(&target.id, now_seconds())
                .map(|profile| serde_json::to_string(&profile).unwrap_or_else(|_| "{}".to_string()))
        } else {
            let removed = store
                .get(&target.id)
                .map(|profile| profile.artefacts)
                .unwrap_or_default();
            store.delete(&target.id).map(|()| {
                serde_json::json!({
                    "deleted": true,
                    "removed": removed,
                    "message": "Deleted. The sample and everything made from it are gone; copies in system backups are outside Parlo's reach.",
                })
                .to_string()
            })
        };
        return match outcome {
            Ok(body) => respond_json(stream, "200 OK", &body).map(|()| Flow::Serve),
            Err(error) => {
                let status = match error {
                    parlo_core::VoiceError::NotFound(_) => "404 Not Found",
                    _ => "400 Bad Request",
                };
                respond_json(
                    stream,
                    status,
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/voice/sample" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let Some(id) = header(&headers, "x-parlo-voice") else {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"the sample needs a voice profile id\"}",
            )
            .map(|()| Flow::Serve);
        };
        let audio = match parlo_core::wav::read_pcm16(&body) {
            Ok(audio) => audio,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json(&format!("the sample is not a readable wav: {error}")),
                )
                .map(|()| Flow::Serve)
            }
        };
        let seconds = wav_seconds(&body);
        if !(MIN_SAMPLE_SECONDS..=MAX_SAMPLE_SECONDS).contains(&seconds) {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json(&format!(
                    "the sample is {seconds:.1} s; a voice profile takes between {MIN_SAMPLE_SECONDS:.0} and {MAX_SAMPLE_SECONDS:.0} s of speech"
                )),
            )
            .map(|()| Flow::Serve);
        }
        if let Some(problem) = crate::sample::problem(&audio.samples, seconds) {
            return respond_json(stream, "400 Bad Request", &error_json(&problem))
                .map(|()| Flow::Serve);
        }
        return match voice_store().save_sample(&id, &body) {
            Ok(profile) => {
                let saved = serde_json::to_string(&profile).unwrap_or_else(|_| "{}".to_string());
                respond_json(stream, "200 OK", &saved).map(|()| Flow::Serve)
            }
            Err(error) => {
                let status = match error {
                    parlo_core::VoiceError::NotFound(_) => "404 Not Found",
                    _ => "400 Bad Request",
                };
                respond_json(stream, status, &error_json(&error.to_string())).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/voice/sample/play" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let target: VoiceTarget = match serde_json::from_slice(&body) {
            Ok(target) => target,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        return match voice_store().sample(&target.id) {
            Ok(bytes) => respond_audio(stream, &bytes, "").map(|()| Flow::Serve),
            Err(error) => respond_json(stream, "404 Not Found", &error_json(&error.to_string()))
                .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/voice/export" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let target: VoiceTarget = match serde_json::from_slice(&body) {
            Ok(target) => target,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        let store = voice_store();
        let profile = match store.get(&target.id) {
            Ok(profile) => profile,
            Err(error) => {
                return respond_json(stream, "404 Not Found", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve)
            }
        };
        let sample = store.sample(&target.id).unwrap_or_default();
        let header = serde_json::json!({
            "format": "parlo-voice-profile",
            "version": 1,
            "synthetic": true,
            "notice": "Audio from this profile is synthetic voice. The person who authorised it is named in the consent record below.",
            "profile": profile,
        });
        let mut exported = header.to_string().into_bytes();
        exported.extend_from_slice(b"\n\n");
        exported.extend_from_slice(&sample);
        return respond_named(stream, &format!("parlo-voice-{}.txt", target.id), &exported)
            .map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/dub" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        if !crate::config::load().features.translation {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"dubbing needs translation, and translation is switched off in Settings\"}",
            )
            .map(|()| Flow::Serve);
        }
        let requested_url = header(&headers, "x-parlo-url")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if requested_url.is_some() && !body.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("send either a file or a link, not both"),
            )
            .map(|()| Flow::Serve);
        }
        if body.is_empty() && requested_url.is_none() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"the file is empty\"}",
            )
            .map(|()| Flow::Serve);
        }

        let stored = crate::config::load();
        let (mut project, mut project_store) = match load_requested_project(&headers) {
            Ok(Some(loaded)) => (Some(loaded.project), Some(loaded.store)),
            Ok(None) => (None, None),
            Err((reason, message)) => {
                return respond_json(stream, reason, &error_json(&message)).map(|()| Flow::Serve);
            }
        };
        let (from, to) = match project.as_ref() {
            Some(project) => (
                project.from.clone(),
                header(&headers, "x-parlo-to")
                    .map(|value| value.trim().to_ascii_lowercase())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| project.to.clone()),
            ),
            None => (
                header(&headers, "x-parlo-from")
                    .unwrap_or_else(|| stored.from.clone())
                    .to_ascii_lowercase(),
                header(&headers, "x-parlo-to")
                    .unwrap_or_else(|| stored.to.clone())
                    .to_ascii_lowercase(),
            ),
        };
        if let Err(error) = crate::config::check_pair(&from, &to) {
            return respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        let timing = header(&headers, "x-parlo-timing")
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "natural".to_string());
        let fastest = match header(&headers, "x-parlo-fastest") {
            Some(value) => match value.trim().parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &error_json("fastest must be a number"),
                    )
                    .map(|()| Flow::Serve);
                }
            },
            None => 1.25,
        };
        let slowest = match header(&headers, "x-parlo-slowest") {
            Some(value) => match value.trim().parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    return respond_json(
                        stream,
                        "400 Bad Request",
                        &error_json("slowest must be a number"),
                    )
                    .map(|()| Flow::Serve);
                }
            },
            None => 0.9,
        };
        if let Err(error) = dub::check_timing(&timing, fastest, slowest) {
            return respond_json(stream, "400 Bad Request", &error_json(&error))
                .map(|()| Flow::Serve);
        }

        let requested = header(&headers, "x-parlo-filename")
            .map(|value| percent_decode(&value))
            .unwrap_or_else(|| match &requested_url {
                Some(url) => crate::ingest::name_from_url(url),
                None => "upload.mp4".to_string(),
            });
        let name = dub::safe_name(&requested);
        let id = new_token();
        let directory = std::env::temp_dir().join("parlo-dub");
        if let Err(error) = fs::create_dir_all(&directory) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!(
                    "{{\"error\":\"cannot create {}: {}\"}}",
                    json_string(&directory.display().to_string()),
                    json_string(&error.to_string())
                ),
            )
            .map(|()| Flow::Serve);
        }

        let cues = match write_cues(&directory, &id, project.as_ref()) {
            Ok(cues) => cues,
            Err(error) => {
                return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };

        let upload = directory.join(format!("{id}-{name}"));
        if let Some(url) = &requested_url {
            if let Err(failure) = crate::ingest::download(url, &upload) {
                if let Some(path) = &cues {
                    fs::remove_file(path).ok();
                }
                return respond_json(stream, failure.status, &error_json(&failure.message))
                    .map(|()| Flow::Serve);
            }
            if let Err(reason) = crate::ingest::probe(&upload) {
                fs::remove_file(&upload).ok();
                if let Some(path) = &cues {
                    fs::remove_file(path).ok();
                }
                return respond_json(stream, "400 Bad Request", &error_json(&reason))
                    .map(|()| Flow::Serve);
            }
        } else if let Err(error) = fs::write(&upload, &body) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!(
                    "{{\"error\":\"cannot store the upload: {}\"}}",
                    json_string(&error.to_string())
                ),
            )
            .map(|()| Flow::Serve);
        }

        if timing == "lip-sync" {
            if let Ok(false) = crate::ingest::has_video(&upload) {
                fs::remove_file(&upload).ok();
                if let Some(path) = &cues {
                    fs::remove_file(path).ok();
                }
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json("lip sync needs a video file with a picture"),
                )
                .map(|()| Flow::Serve);
            }
        }

        let out_name = dubbed_name(&name, &to);
        let dubbed = directory.join(format!("{id}-{out_name}"));
        let subtitles = dubbed.with_extension("srt");
        let track = directory.join(format!("{id}-track.wav"));
        let lines_dir = directory.join(format!("{id}-lines"));
        let report = directory.join(format!("{id}-report.json"));
        let keep_background = header(&headers, "x-parlo-background").is_some();
        let _ = stream.set_write_timeout(Some(DUB_TIMEOUT));
        println!("dubbing {name} from {from} to {to}");
        let result = dub::run(&dub::Request {
            source: &upload,
            from: &from,
            to: &to,
            out: Some(dubbed.clone()),
            srt: Some(subtitles.clone()),
            cues: cues.clone(),
            track: Some(track.clone()),
            lines_dir: Some(lines_dir.clone()),
            report: Some(report.clone()),
            previous: None,
            retry: None,
            skip: None,
            keep_background,
            timing: &timing,
            fastest,
            slowest,
        });
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

        return match result {
            Ok(result) => {
                let project_id = project.as_ref().map(|project| project.id.clone());
                if let (Some(project), Some(store)) = (project.as_mut(), project_store.as_mut()) {
                    apply_dub(project, &result.lines, &to);
                    if let Err(error) = store.save_dub_project(project) {
                        return respond_json(
                            stream,
                            "500 Internal Server Error",
                            &error_json(&format!(
                                "the dub finished, but the project could not be saved: {error}"
                            )),
                        )
                        .map(|()| Flow::Serve);
                    }
                }
                let written = dub_exports(&result.exports);
                let file = DubFile {
                    id: id.clone(),
                    upload,
                    dubbed,
                    srt: subtitles,
                    track,
                    lines_dir,
                    report,
                    cues,
                    name: out_name,
                    from,
                    to,
                    timing,
                    fastest,
                    slowest,
                    project_id,
                    keep_background,
                    seconds: result.seconds,
                    lines: result.lines,
                    exports: written,
                };
                let body = dub_body(&file);
                remember(file);
                respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
            }
            Err(error) => {
                fs::remove_file(&upload).ok();
                fs::remove_file(&track).ok();
                fs::remove_file(&report).ok();
                fs::remove_dir_all(&lines_dir).ok();
                if let Some(path) = &cues {
                    fs::remove_file(path).ok();
                }
                respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/models" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        if models::ready(&models::status()) {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"models are already installed\"}",
            )
            .map(|()| Flow::Serve);
        }
        if let Err(error) = models::install(&[]) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        return respond_json(stream, "202 Accepted", "{\"started\":true}").map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/models/dictation" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        if models::dictation_ready(&models::status()) {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"dictation is already installed\"}",
            )
            .map(|()| Flow::Serve);
        }
        if let Err(error) = models::install(models::DICTATION_UNITS) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        return respond_json(stream, "202 Accepted", "{\"started\":true}").map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/models/rest" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        if let Err(error) = models::install(&models::remaining_units()) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        return respond_json(stream, "202 Accepted", "{\"started\":true}").map(|()| Flow::Serve);
    }

    if method == "POST" && path.starts_with("/models/download/") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = &path["/models/download/".len()..];
        if ![
            "parakeet", "whisper", "voice-pt", "voice-en", "mt-pt-en", "mt-en-pt",
        ]
        .contains(&id)
        {
            return respond_json(stream, "404 Not Found", "{\"error\":\"unknown model\"}")
                .map(|()| Flow::Serve);
        }
        if let Err(error) = models::install(&[id]) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        return respond_json(stream, "202 Accepted", "{\"started\":true}").map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/models/download" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error.to_string()))
                    .map(|()| Flow::Serve);
            }
        };
        let units: Vec<String> = request
            .get("units")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if units.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"choose at least one model\"}",
            )
            .map(|()| Flow::Serve);
        }
        let known = [
            "parakeet", "whisper", "voice-pt", "voice-en", "mt-pt-en", "mt-en-pt",
        ];
        if let Some(unknown) = units.iter().find(|unit| !known.contains(&unit.as_str())) {
            return respond_json(
                stream,
                "404 Not Found",
                &format!("{{\"error\":\"unknown model {}\"}}", json_string(unknown)),
            )
            .map(|()| Flow::Serve);
        }
        let chosen: Vec<&str> = units.iter().map(String::as_str).collect();
        if let Err(error) = models::install(&chosen) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
        return respond_json(stream, "202 Accepted", "{\"started\":true}").map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/models/relocate" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":{\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
        let target = request["path"].as_str().unwrap_or_default();
        if !request["confirm"].as_bool().unwrap_or(false) {
            return match models::relocate_plan(target) {
                Ok((needed, adopt)) => {
                    let size = models::describe_bytes(needed);
                    let message = if adopt {
                        format!(
                            "{size} are already in that folder. Press again to use them and remove the old copy."
                        )
                    } else {
                        format!(
                            "{size} will be copied to that folder. Nothing is deleted until the copy matches."
                        )
                    };
                    respond_json(
                        stream,
                        "200 OK",
                        &serde_json::json!({ "needed": needed, "adopt": adopt, "message": message })
                            .to_string(),
                    )
                    .map(|()| Flow::Serve)
                }
                Err(error) => respond_json(stream, "400 Bad Request", &error_json(&error))
                    .map(|()| Flow::Serve),
            };
        }
        return match models::relocate(target) {
            Ok(message) => respond_json(
                stream,
                "200 OK",
                &serde_json::json!({ "message": message }).to_string(),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path.starts_with("/models/rollback/") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", &error_json("wrong token"))
                .map(|()| Flow::Serve);
        }
        let id = &path["/models/rollback/".len()..];
        return match models::rollback(id) {
            Ok(message) => respond_json(
                stream,
                "200 OK",
                &serde_json::json!({ "message": message }).to_string(),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/models/cancel" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match models::cancel() {
            Ok(()) => respond_json(
                stream,
                "200 OK",
                "{\"cancelled\":true,\"message\":\"Stopped. What already arrived stays on this machine.\"}",
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "409 Conflict", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path.starts_with("/models/remove/") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = &path["/models/remove/".len()..];
        return match models::remove(id) {
            Ok(message) => respond_json(
                stream,
                "200 OK",
                &format!(
                    "{{\"removed\":true,\"message\":\"{}\"}}",
                    json_string(&message)
                ),
            )
            .map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST"
        && (path.starts_with("/models/load/") || path.starts_with("/models/unload/"))
    {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let loading = path.starts_with("/models/load/");
        let id = if loading {
            &path["/models/load/".len()..]
        } else {
            &path["/models/unload/".len()..]
        };
        let Some(unit) = models::unit_of(id) else {
            return respond_json(stream, "404 Not Found", "{\"error\":\"unknown model\"}")
                .map(|()| Flow::Serve);
        };
        if session.is_working() {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"a task is running; change this when it finishes\"}",
            )
            .map(|()| Flow::Serve);
        }
        let outcome = if loading {
            session.load_unit(unit)
        } else {
            session.unload_unit(unit)
        };
        return match outcome {
            Ok(()) => {
                let state = if loading { "loaded" } else { "unloaded" };
                let message = if loading {
                    "Loaded. It stays in memory until it sits idle too long or you unload it."
                } else {
                    "Freed from memory. The files stay on disk."
                };
                respond_json(
                    stream,
                    "200 OK",
                    &format!(
                        "{{\"model\":\"{}\",\"state\":\"{state}\",\"message\":\"{message}\"}}",
                        json_string(id)
                    ),
                )
                .map(|()| Flow::Serve)
            }
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/models/policy" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: Policy = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        return match models::set_policy(&request.id, request.minutes) {
            Ok(message) => respond_json(
                stream,
                "200 OK",
                &format!("{{\"message\":\"{}\"}}", json_string(&message)),
            )
            .map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "GET" && path == "/update/progress" {
        return respond_json(stream, "200 OK", &update::progress_json()).map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/update/restart" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        if let Err(error) = update::restart() {
            return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        respond_json(stream, "200 OK", "{\"restarting\":true}").map(|()| Flow::Serve)?;
        return Ok(Flow::Stop);
    }

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
        let build = latest.archive_url.is_some();
        let body = format!(
            "{{\"current\":\"{VERSION}\",\"latest\":\"{}\",\"available\":{available},\"build\":{build},\"notes\":\"{}\"}}",
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
        if let Err(error) = update::schedule_replacement(&binary) {
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

    if method == "GET" && (path == "/island" || path == "/island.html") {
        let page = ISLAND.replace("__PARLO_VERSION__", VERSION);
        return respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            page.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "GET" && (path == "/captions" || path == "/captions.html") {
        if !crate::config::load().features.captions {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"captions are switched off in Settings\"}",
            )
            .map(|()| Flow::Serve);
        }
        let page = CAPTIONS.replace("__PARLO_VERSION__", VERSION);
        return respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            page.as_bytes(),
        )
        .map(|()| Flow::Serve);
    }

    if method == "POST" && path == "/captions/save" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", &error_json("wrong token"))
                .map(|()| Flow::Serve);
        }
        let request: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
        let lines: Vec<(u64, String)> = request["lines"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| {
                        let text = row["text"].as_str()?.trim();
                        if text.is_empty() {
                            return None;
                        }
                        Some((row["at_ms"].as_u64().unwrap_or(0), text.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        return match crate::captions::save(&lines) {
            Ok(path) => respond_json(
                stream,
                "200 OK",
                &serde_json::json!({
                    "path": path.display().to_string(),
                    "message": format!("Saved to {}.", path.display()),
                })
                .to_string(),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "GET" && (path == "/live" || path == "/live.html") {
        if !crate::config::load().features.translation {
            return respond_json(
                stream,
                "409 Conflict",
                "{\"error\":\"live mode is switched off in Settings\"}",
            )
            .map(|()| Flow::Serve);
        }
        let page = LIVE.replace("__PARLO_VERSION__", VERSION);
        return respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            page.as_bytes(),
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

    if method == "GET" && path == "/v1/audio/transcriptions/stream" {
        return ws::stream(
            stream,
            &headers,
            token,
            &parameter(query, "t"),
            port,
            session,
        );
    }

    if let Some(answer) =
        routes::diarize::route(method, path, stream, &headers, token, &body, session)
    {
        return answer;
    }

    if let Some(answer) = routes::clipboard::route(method, path, stream, &headers, token, &body) {
        return answer;
    }

    if method == "GET" && path == "/target" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let answer = match crate::insert::foreground() {
            Ok((_, app)) => format!("{{\"insertable\":true,\"app\":\"{}\"}}", json_string(&app)),
            Err(error) => format!(
                "{{\"insertable\":false,\"reason\":\"{}\"}}",
                json_string(&error)
            ),
        };
        return respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve);
    }

    if method == "POST" && (path == "/listen/stream" || path == "/listen/stream-end") {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        if path == "/listen/stream-end" {
            return match session.finish_stream() {
                Ok(pieces) => {
                    respond_json(stream, "200 OK", &pieces_json(&pieces)).map(|()| Flow::Serve)
                }
                Err(error) => {
                    respond_json(stream, "500 Internal Server Error", &error_json(&error))
                        .map(|()| Flow::Serve)
                }
            };
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
        if wav.samples.is_empty() {
            return respond_json(stream, "200 OK", "[]").map(|()| Flow::Serve);
        }
        return match session.stream_window(&wav.samples) {
            Ok(pieces) => {
                respond_json(stream, "200 OK", &pieces_json(&pieces)).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if path == "/state" {
        let mut machine = match STATE.lock() {
            Ok(machine) => machine,
            Err(_) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    "{\"error\":\"the state is unavailable\"}",
                )
                .map(|()| Flow::Serve);
            }
        };
        if method == "POST" {
            let next = serde_json::from_slice::<serde_json::Value>(&body)
                .ok()
                .and_then(|request| {
                    request
                        .get("state")
                        .and_then(serde_json::Value::as_str)
                        .and_then(parlo_core::state::State::parse)
                });
            let Some(next) = next else {
                return respond_json(stream, "400 Bad Request", "{\"error\":\"unknown state\"}")
                    .map(|()| Flow::Serve);
            };
            if let Err(error) = machine.enter(next) {
                return respond_json(stream, "409 Conflict", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        }
        let answer = format!("{{\"state\":\"{}\"}}", machine.state().as_str());
        return respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/engine" {
        let choice = crate::engines::choose(
            crate::engines::detect_cores(),
            crate::engines::detect_gpu_memory_mb(),
            crate::engines::Fidelity::Cloned,
        );
        let logged = crate::engines::log_choice(&choice);
        let steps: Vec<&str> = crate::engines::chain(choice.requested)
            .iter()
            .map(|fidelity| fidelity.label())
            .collect();
        let body = serde_json::json!({
            "profile": choice.profile.label(),
            "requested": choice.requested.label(),
            "chosen": choice.chosen.label(),
            "chain": steps,
            "reason": choice.reason,
            "log": logged.map(|path| path.display().to_string()),
            "switches": crate::engines::recent_switches(5),
        })
        .to_string();
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if path == "/health" {
        let uptime = started.elapsed().as_secs();
        let chosen = session.chosen();
        let engine = chosen.engine.clone();
        let body = format!(
            "{{\"status\":\"ok\",\"uptime_s\":{uptime},\"engine\":\"{engine}\",\"stt\":\"{}\",\"from\":\"{}\",\"to\":\"{}\"}}",
            json_string(&chosen.stt),
            json_string(&chosen.from),
            json_string(&chosen.to)
        );
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
    }

    if let Some(answer) = routes::discovery::route(method, path, stream) {
        return answer;
    }

    if let Some(answer) = routes::models::route(method, path, stream, session) {
        return answer;
    }

    if method == "POST" && path == "/v1/audio/transcriptions" {
        return routes::openai::transcriptions(stream, &headers, token, &body, session);
    }

    if method == "POST" && path == "/v1/audio/translations" {
        return routes::openai::translations(stream, &headers, token, &body, session);
    }

    if method == "POST" && path == "/v1/audio/speech" {
        return routes::openai::speech(stream, &headers, token, &body, session);
    }

    if path == "/mcp" {
        return routes::mcp::route(method, stream, &headers, token, &body)
            .unwrap_or(Ok(Flow::Serve));
    }

    if method == "POST" && path == "/v1/jobs" {
        return routes::jobs::create(stream, &headers, token, &body);
    }

    if method == "GET" && path == "/v1/jobs" {
        return routes::jobs::list(stream, &headers, token);
    }

    if method == "GET" && path.starts_with("/v1/jobs/") {
        return routes::jobs::read(stream, &headers, token, path);
    }

    if method == "POST" && path.starts_with("/v1/jobs/") && path.ends_with("/cancel") {
        return routes::jobs::cancel(stream, &headers, token, path);
    }

    if method == "POST" && path == "/dictate/begin" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match session.capture_target() {
            Ok(()) => respond_json(stream, "200 OK", "{\"target\":true}").map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/undo" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let last = match undo::take() {
            Ok(last) => last,
            Err(error) => {
                return respond_json(stream, "409 Conflict", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        return match insert::erase(last.window, last.characters) {
            Ok(()) => respond_json(
                stream,
                "200 OK",
                &format!("{{\"erased\":{}}}", last.characters),
            )
            .map(|()| Flow::Serve),
            Err(error) => {
                undo::restore(&last);
                respond_json(stream, "409 Conflict", &error_json(&error)).map(|()| Flow::Serve)
            }
        };
    }

    if method == "POST" && path == "/insert" {
        if !authorised(&headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let request: InsertRequest = match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error.to_string())),
                )
                .map(|()| Flow::Serve)
            }
        };
        if request.text.trim().is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"nothing to write\"}",
            )
            .map(|()| Flow::Serve);
        }
        return match session.insert_text(&request.text) {
            Ok(()) => respond_json(stream, "200 OK", "{\"inserted\":true}").map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "409 Conflict",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path == "/dictate" {
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
        if wav.samples.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"the file is empty\"}",
            )
            .map(|()| Flow::Serve);
        }

        let seconds = wav.samples.len() as f32 / 16_000.0;
        let mut pcm16 = Vec::with_capacity(wav.samples.len() * 2);
        for sample in &wav.samples {
            pcm16.extend_from_slice(&sample.to_le_bytes());
        }
        let write = header(&headers, "x-parlo-write")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !write.is_empty() && !crate::config::check_language(&write) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"x-parlo-write needs a two letter language code\"}",
            )
            .map(|()| Flow::Serve);
        }
        let clean_header = header(&headers, "x-parlo-clean")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !clean_header.is_empty() && !["on", "off"].contains(&clean_header.as_str()) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"x-parlo-clean needs on or off\"}",
            )
            .map(|()| Flow::Serve);
        }
        let stored = crate::config::load();
        let clean = if clean_header == "off" {
            Some("original".to_string())
        } else {
            Some(stored.intelligence.clean.clone())
        };
        let style_header = header(&headers, "x-parlo-style")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !style_header.is_empty() && !crate::style::is_known(&style_header) {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"x-parlo-style needs formal, casual or excited\"}",
            )
            .map(|()| Flow::Serve);
        }
        let app = crate::insert::foreground()
            .map(|(_, name)| name)
            .unwrap_or_else(|_| "unknown".to_string());
        let profile = stored.profiles.get(&app.to_ascii_lowercase());
        let style = if !style_header.is_empty() {
            style_header
        } else if let Some(wanted) = profile.map(|profile| profile.style.clone()) {
            if crate::style::is_known(&wanted) {
                wanted
            } else {
                String::new()
            }
        } else if crate::style::is_known(&stored.style) {
            stored.style.clone()
        } else {
            String::new()
        };
        let write = if !write.is_empty() {
            write
        } else {
            profile
                .map(|profile| profile.write.clone())
                .unwrap_or_default()
        };
        let target = dictation_target(&write, &session.chosen().from);
        let started = Instant::now();
        let _ = stream.set_write_timeout(Some(Duration::from_secs(120)));
        let (text, translation) = match session.write(&pcm16, &target, clean.as_deref()) {
            Ok(pair) => pair,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        };
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        let text = commands::apply(&text);
        let text = crate::style::shape(&text, &style);
        let mut entry = crate::history::Entry::now(
            &text,
            &translation,
            &session.chosen().from,
            &target,
            &app,
            "dictate",
        );
        entry.seconds = f64::from(seconds);
        if let Err(error) = crate::history::append(&entry) {
            println!("could not store the dictation: {error}");
        }
        println!(
            "dictated {seconds:.1} s in {} ms",
            started.elapsed().as_millis()
        );
        let body = format!(
            "{{\"text\":\"{}\",\"translation\":\"{}\",\"seconds\":{seconds:.3},\"latency_ms\":{}}}",
            json_string(&text),
            json_string(&translation),
            started.elapsed().as_millis()
        );
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
    let mut transcript = String::new();
    let _working = session.working();
    let mut pipeline = session.pipeline();
    for chunk in wav.samples.chunks(320) {
        for output in pipeline.push(chunk) {
            if let Some(error) = absorb(output, &mut samples, &mut transcript) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &format!("{{\"error\":\"{}\"}}", json_string(&error)),
                )
                .map(|()| Flow::Serve);
            }
        }
    }
    for output in pipeline.finish() {
        if let Some(error) = absorb(output, &mut samples, &mut transcript) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }
    }

    let translated = Wav {
        format: AudioFormat::mono(16_000).map_err(|error| error.to_string())?,
        samples,
    };
    respond_audio(stream, &write_pcm16(&translated), &transcript).map(|()| Flow::Serve)
}

#[derive(serde::Deserialize)]
struct KeyRequest {
    #[serde(default)]
    provider: String,
    #[serde(default)]
    key: String,
}

fn answer_with(
    stream: &mut TcpStream,
    outcome: Result<serde_json::Value, String>,
    success: &str,
) -> Result<Flow, String> {
    match outcome {
        Ok(reply) => match reply.get("error").and_then(serde_json::Value::as_str) {
            Some(error) => respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(error)),
            )
            .map(|()| Flow::Serve),
            None => respond_json(stream, "200 OK", success).map(|()| Flow::Serve),
        },
        Err(error) => respond_json(
            stream,
            "500 Internal Server Error",
            &format!("{{\"error\":\"{}\"}}", json_string(&error)),
        )
        .map(|()| Flow::Serve),
    }
}

#[derive(serde::Deserialize)]
struct InsertRequest {
    text: String,
}

#[derive(serde::Deserialize)]
struct VoiceRequest {
    label: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    purpose: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    origin: Option<String>,
    #[serde(default)]
    declared_authorisation: bool,
    #[serde(default)]
    encrypt: bool,
}

#[derive(serde::Deserialize)]
struct VoiceTarget {
    id: String,
}

const MIN_SAMPLE_SECONDS: f64 = 10.0;
const MAX_SAMPLE_SECONDS: f64 = 30.0;

fn voice_store() -> Store {
    Store::new(crate::config::data_root().join("voices"))
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

fn consent_for(
    origin: Option<&str>,
    declared_authorisation: bool,
    purpose: String,
    scope: String,
    accepted_at: u64,
) -> Result<Consent, String> {
    if origin == Some("third-party") {
        if !declared_authorisation {
            return Err(
                "a third party voice profile requires a declared authorisation".to_string(),
            );
        }
        return Ok(Consent::third_party(purpose, scope, accepted_at));
    }
    Ok(Consent::own_voice(purpose, scope, accepted_at))
}

fn meeting_json(meeting: crate::meeting::Meeting) -> serde_json::Value {
    let segments: Vec<serde_json::Value> = meeting
        .segments
        .iter()
        .map(|segment| {
            serde_json::json!({
                "at_ms": segment.at_ms,
                "speaker": segment.speaker,
                "spoken_by": segment.spoken_by(),
                "text": segment.text,
                "named": segment.name.is_some(),
            })
        })
        .collect();
    serde_json::json!({
        "id": meeting.id,
        "title": meeting.title,
        "started_at": meeting.started_at,
        "speakers": meeting.speakers(),
        "segments": segments,
        "actions": crate::actions::for_meeting(&meeting),
        "decisions": crate::decisions::for_meeting(&meeting),
    })
}

fn dubbed_name(name: &str, to: &str) -> String {
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "dubbed".to_string());
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_else(|| "wav".to_string());
    format!("{stem}.{to}.{extension}")
}

struct LoadedProject {
    project: crate::store::DubProject,
    store: ProjectStore,
}

fn load_requested_project(
    headers: &[(String, String)],
) -> Result<Option<LoadedProject>, (&'static str, String)> {
    let id = header(headers, "x-parlo-project").unwrap_or_default();
    if id.is_empty() {
        return Ok(None);
    }
    let store = ProjectStore::open_default().map_err(|error| {
        (
            "Internal Server Error",
            format!("cannot open the project store: {error}"),
        )
    })?;
    match store.load_dub_project(&id) {
        Ok(Some(project)) => {
            if project.segments.is_empty() {
                return Err((
                    "Bad Request",
                    "the imported project has no cues".to_string(),
                ));
            }
            Ok(Some(LoadedProject { project, store }))
        }
        Ok(None) => Err(("Not Found", "project not found".to_string())),
        Err(error) => Err((
            "Internal Server Error",
            format!("cannot load the project: {error}"),
        )),
    }
}

fn write_cues(
    directory: &Path,
    id: &str,
    project: Option<&crate::store::DubProject>,
) -> Result<Option<PathBuf>, String> {
    let Some(project) = project else {
        return Ok(None);
    };
    let path = directory.join(format!("{id}-cues.json"));
    let document =
        cue_document(project).map_err(|error| format!("cannot encode the cues: {error}"))?;
    fs::write(&path, document).map_err(|error| format!("cannot store the cues: {error}"))?;
    Ok(Some(path))
}

const MAX_SEGMENT_TEXT: usize = 10_000;

#[derive(serde::Deserialize)]
struct ProjectEdit {
    #[serde(default)]
    cast: std::collections::BTreeMap<String, String>,
    segments: Vec<SegmentEdit>,
}

#[derive(serde::Deserialize)]
struct SegmentEdit {
    id: String,
    start_ms: u64,
    end_ms: u64,
    #[serde(default)]
    speaker: Option<String>,
    #[serde(default)]
    voice_id: Option<String>,
    source_text: String,
}

fn apply_project_edits(
    project: &mut crate::store::DubProject,
    edits: &[SegmentEdit],
    cast: &std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    if project.segments.is_empty() {
        return Err("the project has no segments to edit".to_string());
    }
    if edits.len() != project.segments.len() {
        return Err("send every segment exactly once".to_string());
    }
    for (speaker, voice) in cast {
        if speaker.trim().is_empty() || speaker.chars().count() > 120 {
            return Err("a speaker needs a name of up to 120 characters".to_string());
        }
        if !known_voice(voice) {
            return Err(format!("{voice} is not a voice Parlo can cast"));
        }
    }
    let mut seen = std::collections::HashSet::new();
    for edit in edits {
        if edit.id.is_empty() {
            return Err("a segment has no id".to_string());
        }
        if !seen.insert(edit.id.as_str()) {
            return Err(format!("segment {} appears twice", edit.id));
        }
        if edit.start_ms >= edit.end_ms {
            return Err(format!("segment {} ends before it starts", edit.id));
        }
        if edit.source_text.trim().is_empty() {
            return Err(format!("segment {} has no text", edit.id));
        }
        if edit.source_text.chars().count() > MAX_SEGMENT_TEXT {
            return Err(format!("segment {} has too much text", edit.id));
        }
        if edit
            .speaker
            .as_deref()
            .map(str::trim)
            .map(|speaker| speaker.chars().count() > 120)
            .unwrap_or(false)
        {
            return Err(format!(
                "segment {} has a speaker name that is too long",
                edit.id
            ));
        }
        if let Some(voice) = edit.voice_id.as_deref().filter(|voice| !voice.is_empty()) {
            if !known_voice(voice) {
                return Err(format!("{voice} is not a voice Parlo can cast"));
            }
        }
        if !project.segments.iter().any(|segment| segment.id == edit.id) {
            return Err(format!("segment {} is not in this project", edit.id));
        }
    }
    let before: std::collections::HashMap<String, Option<String>> = project
        .segments
        .iter()
        .map(|segment| {
            (
                segment.id.clone(),
                effective_voice(project, segment).map(ToOwned::to_owned),
            )
        })
        .collect();
    for segment in &mut project.segments {
        let edit = edits
            .iter()
            .find(|edit| edit.id == segment.id)
            .ok_or_else(|| format!("segment {} is missing", segment.id))?;
        let speaker = edit
            .speaker
            .as_deref()
            .map(str::trim)
            .filter(|speaker| !speaker.is_empty())
            .map(ToOwned::to_owned);
        let text_changed = segment.start_ms != edit.start_ms
            || segment.end_ms != edit.end_ms
            || segment.speaker != speaker
            || segment.source_text != edit.source_text;
        segment.start_ms = edit.start_ms;
        segment.end_ms = edit.end_ms;
        segment.speaker = speaker;
        segment.source_text = edit.source_text.clone();
        segment.voice_id = edit
            .voice_id
            .clone()
            .filter(|voice| !voice.trim().is_empty());
        let after = segment.voice_id.clone().or_else(|| {
            segment
                .speaker
                .as_deref()
                .and_then(|speaker| cast.get(speaker))
                .cloned()
        });
        if text_changed || before.get(&segment.id).cloned().flatten() != after {
            if text_changed {
                segment.target_text = None;
                segment.translations.clear();
            }
            segment.state = "edited".to_string();
        }
    }
    project.cast = cast.clone();
    Ok(())
}

#[derive(serde::Serialize)]
struct DubCue<'a> {
    id: &'a str,
    start_ms: u64,
    end_ms: u64,
    source: &'a str,
    speaker: Option<&'a str>,
    voice: Option<&'a str>,
}

#[derive(serde::Serialize)]
struct DubCueDocument<'a> {
    cues: Vec<DubCue<'a>>,
}

fn cue_document(project: &crate::store::DubProject) -> Result<String, serde_json::Error> {
    let cues: Vec<DubCue<'_>> = project
        .segments
        .iter()
        .map(|segment| DubCue {
            id: segment.id.as_str(),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            source: segment.source_text.as_str(),
            speaker: segment.speaker.as_deref(),
            voice: effective_voice(project, segment),
        })
        .collect();
    serde_json::to_string(&DubCueDocument { cues })
}

fn known_voice(id: &str) -> bool {
    crate::registry::model(id)
        .map(|spec| spec.kind == "voice")
        .unwrap_or(false)
}

fn effective_voice<'a>(
    project: &'a crate::store::DubProject,
    segment: &'a crate::store::DubSegment,
) -> Option<&'a str> {
    segment.voice_id.as_deref().or_else(|| {
        segment
            .speaker
            .as_deref()
            .and_then(|speaker| project.cast.get(speaker))
            .map(String::as_str)
    })
}

fn project_json(project: &crate::store::DubProject) -> Result<String, serde_json::Error> {
    let installed: std::collections::HashSet<&str> = crate::models::status()
        .into_iter()
        .filter(|model| model.installed)
        .map(|model| model.id)
        .collect();
    let voices: Vec<serde_json::Value> = crate::registry::MODELS
        .iter()
        .filter(|spec| spec.kind == "voice")
        .map(|spec| {
            serde_json::json!({
                "id": spec.id,
                "name": spec.name,
                "installed": installed.contains(spec.id),
            })
        })
        .collect();
    let mut languages: Vec<&str> = Vec::new();
    for spec in crate::registry::MODELS
        .iter()
        .filter(|spec| spec.kind == "translation")
    {
        if spec.languages.first() == Some(&project.from.as_str()) {
            if let Some(target) = spec.languages.get(1) {
                if !languages.contains(target) {
                    languages.push(target);
                }
            }
        }
    }
    if !languages.contains(&project.to.as_str()) {
        languages.push(&project.to);
    }
    let mut document = serde_json::to_value(project)?;
    document["voices"] = serde_json::Value::Array(voices);
    document["languages"] = serde_json::json!(languages);
    serde_json::to_string(&document)
}

fn apply_dub(project: &mut crate::store::DubProject, lines: &[dub::Line], language: &str) {
    let primary = language == project.to;
    for line in lines {
        let Some(id) = line.id.as_deref() else {
            continue;
        };
        if let Some(segment) = project.segments.iter_mut().find(|segment| segment.id == id) {
            if !line.translated.is_empty() {
                segment
                    .translations
                    .insert(language.to_string(), line.translated.clone());
                if primary {
                    segment.target_text = Some(line.translated.clone());
                }
            }
            segment.state = if line.failed {
                "failed".to_string()
            } else if line.skipped {
                "skipped".to_string()
            } else {
                "dubbed".to_string()
            };
        }
    }
    if !project.targets.iter().any(|target| target == language) {
        project.targets.push(language.to_string());
    }
}

fn respond_file(stream: &mut TcpStream, path: &Path, name: &str) -> Result<(), String> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let length = file.metadata().map_err(|error| error.to_string())?.len();
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {length}\r\nContent-Disposition: attachment; filename=\"{name}\"\r\nConnection: close\r\n\r\n",
        content_type(name)
    );
    stream
        .write_all(head.as_bytes())
        .map_err(|error| error.to_string())?;
    std::io::copy(&mut BufReader::new(file), stream).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn wav_seconds(bytes: &[u8]) -> f64 {
    let rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]).max(1);
    let data = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as f64;
    data / 2.0 / f64::from(rate)
}

fn respond_named(stream: &mut TcpStream, name: &str, body: &[u8]) -> Result<(), String> {
    let head = format!(
        "HTTP/1.1 200 OK\nContent-Type: {}\nContent-Length: {}\nContent-Disposition: attachment; filename=\"{name}\"\nConnection: close\n\n",
        content_type(name),
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .map_err(|error| error.to_string())?;
    stream.write_all(body).map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())
}

fn transcript_header(transcript: &str) -> String {
    if transcript.is_empty() {
        String::new()
    } else {
        format!(
            "X-Parlo-Transcript: {}\r\nAccess-Control-Expose-Headers: X-Parlo-Transcript\r\n",
            BASE64.encode(transcript)
        )
    }
}

fn respond_audio(stream: &mut TcpStream, body: &[u8], transcript: &str) -> Result<(), String> {
    let header = transcript_header(transcript);
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: audio/wav\r\nContent-Length: {}\r\n{header}Connection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .and_then(|()| stream.write_all(body))
        .and_then(|()| stream.flush())
        .map_err(|error| error.to_string())
}

fn dictation_target(write: &str, from: &str) -> String {
    if write.is_empty() {
        from.to_string()
    } else {
        write.to_string()
    }
}

fn pieces_json(pieces: &[parlo_core::engine::StreamPiece]) -> String {
    let rows: Vec<serde_json::Value> = pieces
        .iter()
        .map(|piece| {
            serde_json::json!({
                "state": piece.state,
                "text": piece.text,
                "audio_ms": piece.audio_ms,
                "latency_ms": piece.latency_ms,
            })
        })
        .collect();
    serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string())
}

fn parameter(query: &str, name: &str) -> String {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| percent_decode(value))
        .unwrap_or_default()
}

fn restyle_fields(request: &serde_json::Value) -> Result<(String, String), String> {
    let id = text_field(request, "id");
    let style = text_field(request, "style").to_ascii_lowercase();
    if id.is_empty() {
        return Err("a dictation is named by its id".to_string());
    }
    if !crate::style::is_known(&style) {
        return Err("style needs formal, casual or excited".to_string());
    }
    Ok((id, style))
}

fn text_field(request: &serde_json::Value, name: &str) -> String {
    request
        .get(name)
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn absorb(
    output: parlo_core::PipelineOutput,
    samples: &mut Vec<f32>,
    transcript: &mut String,
) -> Option<String> {
    if let Some(text) = output.transcript {
        if !transcript.is_empty() {
            transcript.push(' ');
        }
        transcript.push_str(&text);
    }
    if let Some(frame) = output.frame {
        samples.extend_from_slice(frame.samples());
    }
    output.error
}

fn json_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
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
    if length > body_limit(&head) {
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

fn body_limit(head: &str) -> usize {
    let path = head.split_whitespace().nth(1).unwrap_or_default();
    if path.starts_with("/dub/import") {
        MAX_BODY
    } else if path.starts_with("/dub") {
        MAX_DUB_BODY
    } else if path.starts_with("/v1/audio/") {
        MAX_AUDIO_BODY
    } else {
        MAX_BODY
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(port: u16, request: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
    }

    #[test]
    fn the_main_routes_answer_on_a_real_socket() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let session =
            Session::open("echo".into(), "parakeet".into(), "pt".into(), "en".into()).unwrap();
        let accepting = Arc::new(AtomicBool::new(true));
        let stop = Arc::clone(&accepting);
        std::thread::spawn(move || listen(listener, "t0ken".into(), port, session, accepting));

        let health = ask(port, "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        assert!(health.starts_with("HTTP/1.1 200 OK"));
        assert!(health.contains("\"status\":\"ok\""));

        let state = ask(port, "GET /state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        assert!(state.starts_with("HTTP/1.1 200 OK"));
        assert!(state.contains("\"state\":"));

        let models = ask(port, "GET /models HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        assert!(models.starts_with("HTTP/1.1 200 OK"));

        let discovery = ask(
            port,
            "GET /.well-known/parlo HTTP/1.1\r\nHost: localhost\r\n\r\n",
        );
        assert!(discovery.starts_with("HTTP/1.1 200 OK"));
        assert!(discovery.contains("\"name\":\"Parlo\""));
        assert!(discovery.contains("\"local\":true"));

        let listed = ask(port, "GET /v1/models HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        assert!(listed.starts_with("HTTP/1.1 200 OK"));
        assert!(listed.contains("\"object\":\"list\""));
        assert!(listed.contains("\"owned_by\":\"parlo\""));
        assert!(listed.contains("\"licence\":"));
        assert!(listed.contains("\"languages\":"));

        let foreign = ask(port, "GET /state HTTP/1.1\r\nHost: evil.example\r\n\r\n");
        assert!(foreign.starts_with("HTTP/1.1 400"));

        for request in [
            "GET /v1/jobs HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
            "GET /v1/jobs/missing HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
            "GET /v1/jobs/missing/events HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
            "POST /v1/jobs/missing/cancel HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
            "POST /v1/jobs HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 2\r\n\r\n{}",
        ] {
            let answer = ask(port, request);
            assert!(answer.starts_with("HTTP/1.1 401"), "{answer}");
        }

        stop.store(false, Ordering::Relaxed);
        let _ = TcpStream::connect(("127.0.0.1", port));
    }

    #[test]
    fn a_session_can_be_shared_between_threads() {
        fn shareable<T: Send + Sync>() {}
        shareable::<Session>();
    }

    #[test]
    fn a_wav_length_is_read_from_its_header() {
        let mut bytes = vec![0u8; 44];
        bytes[24..28].copy_from_slice(&16000u32.to_le_bytes());
        bytes[40..44].copy_from_slice(&(16000u32 * 2 * 5).to_le_bytes());
        assert!((wav_seconds(&bytes) - 5.0).abs() < 0.01);
    }

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
    fn only_the_local_host_is_accepted() {
        let local = vec![("host".to_string(), "127.0.0.1:8787".to_string())];
        assert!(host_is_local(&local));
        let named = vec![("host".to_string(), "localhost:8787".to_string())];
        assert!(host_is_local(&named));
        let rebinding = vec![("host".to_string(), "attacker.example:8787".to_string())];
        assert!(!host_is_local(&rebinding));
        assert!(!host_is_local(&[]));
    }

    #[test]
    fn cross_origin_posts_are_refused() {
        assert!(origin_allowed(&[], 8787));
        let same = vec![("origin".to_string(), "http://127.0.0.1:8787".to_string())];
        assert!(origin_allowed(&same, 8787));
        let other = vec![("origin".to_string(), "https://evil.example".to_string())];
        assert!(!origin_allowed(&other, 8787));
    }

    #[test]
    fn the_transcript_header_is_base64_and_absent_when_empty() {
        assert_eq!(transcript_header(""), "");
        let header = transcript_header("olá mundo");
        assert!(header.starts_with("X-Parlo-Transcript: b2zDoSBtdW5kbw=="));
        assert!(header.contains("Access-Control-Expose-Headers"));
    }

    #[test]
    fn dictation_writes_the_language_spoken_unless_another_is_asked_for() {
        assert_eq!(dictation_target("", "pt"), "pt");
        assert_eq!(dictation_target("", "en"), "en");
        assert_eq!(dictation_target("en", "pt"), "en");
    }

    #[test]
    fn a_restyle_needs_an_id_and_a_known_style() {
        let good = serde_json::json!({ "id": "h1", "style": "Casual" });
        assert_eq!(
            restyle_fields(&good).unwrap(),
            ("h1".to_string(), "casual".to_string())
        );
        let nameless = serde_json::json!({ "style": "casual" });
        assert!(restyle_fields(&nameless)
            .unwrap_err()
            .contains("named by its id"));
        let odd = serde_json::json!({ "id": "h1", "style": "shouty" });
        assert!(restyle_fields(&odd)
            .unwrap_err()
            .contains("formal, casual or excited"));
    }

    #[test]
    fn a_search_query_is_read_from_the_address() {
        assert_eq!(parameter("q=bom+dia", "q"), "bom dia");
        assert_eq!(parameter("q=bom%20dia", "q"), "bom dia");
        assert_eq!(parameter("limit=5", "q"), "");
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
    fn the_island_page_dictates_and_carries_the_version() {
        let page = ISLAND.replace("__PARLO_VERSION__", VERSION);
        assert!(page.contains("/dictate"));
        assert!(page.contains("/update"));
        assert!(page.contains(&format!("content=\"{VERSION}\"")));
        assert!(!page.contains("__PARLO_VERSION__"));
    }

    #[test]
    fn dub_uploads_get_their_own_size_limit() {
        assert_eq!(body_limit("POST /dub HTTP/1.1"), MAX_DUB_BODY);
        assert_eq!(body_limit("POST /dub/import HTTP/1.1"), MAX_BODY);
        assert_eq!(body_limit("POST /translate HTTP/1.1"), MAX_BODY);
        assert_eq!(body_limit(""), MAX_BODY);
    }

    #[test]
    fn edited_segments_survive_the_store_and_invalidate_the_translation() {
        let mut store = crate::store::Store::open_memory().unwrap();
        let mut project =
            crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
        project.segments.push(crate::store::DubSegment {
            id: "p1-0".to_string(),
            position: 0,
            start_ms: 1_250,
            end_ms: 3_500,
            speaker: Some("Ana".to_string()),
            voice_id: None,
            translations: std::collections::BTreeMap::new(),
            source_text: "Bom dia".to_string(),
            target_text: Some("Good morning".to_string()),
            state: "dubbed".to_string(),
        });
        store.save_dub_project(&mut project).unwrap();

        apply_project_edits(
            &mut project,
            &[SegmentEdit {
                id: "p1-0".to_string(),
                start_ms: 500,
                end_ms: 2_750,
                speaker: Some("Beatriz".to_string()),
                voice_id: None,
                source_text: "Bom dia a todos".to_string(),
            }],
            &std::collections::BTreeMap::new(),
        )
        .unwrap();
        store.save_dub_project(&mut project).unwrap();
        let loaded = store.load_dub_project("p1").unwrap().unwrap();

        assert_eq!(loaded.segments[0].start_ms, 500);
        assert_eq!(loaded.segments[0].end_ms, 2_750);
        assert_eq!(loaded.segments[0].speaker.as_deref(), Some("Beatriz"));
        assert_eq!(loaded.segments[0].source_text, "Bom dia a todos");
        assert!(loaded.segments[0].target_text.is_none());
        assert_eq!(loaded.segments[0].state, "edited");
    }

    #[test]
    fn cue_documents_keep_segment_identity_and_timing() {
        let mut project =
            crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
        project.segments.push(crate::store::DubSegment {
            id: "p1-0".to_string(),
            position: 0,
            start_ms: 1_250,
            end_ms: 3_500,
            speaker: Some("Ana".to_string()),
            voice_id: None,
            translations: std::collections::BTreeMap::new(),
            source_text: "Bom dia".to_string(),
            target_text: None,
            state: "imported".to_string(),
        });

        let document: serde_json::Value =
            serde_json::from_str(&cue_document(&project).unwrap()).unwrap();

        assert_eq!(document["cues"][0]["id"], "p1-0");
        assert_eq!(document["cues"][0]["start_ms"], 1_250);
        assert_eq!(document["cues"][0]["end_ms"], 3_500);
        assert_eq!(document["cues"][0]["source"], "Bom dia");
    }

    #[test]
    fn casting_and_overrides_decide_the_voice_each_line_uses() {
        let mut project =
            crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
        project
            .cast
            .insert("Ana".to_string(), "voice-en".to_string());
        project
            .cast
            .insert("Bob".to_string(), "voice-de".to_string());
        project.segments.push(crate::store::DubSegment {
            id: "p1-0".to_string(),
            position: 0,
            start_ms: 1_250,
            end_ms: 3_500,
            speaker: Some("Ana".to_string()),
            voice_id: None,
            translations: std::collections::BTreeMap::new(),
            source_text: "Bom dia".to_string(),
            target_text: None,
            state: "imported".to_string(),
        });
        project.segments.push(crate::store::DubSegment {
            id: "p1-1".to_string(),
            position: 1,
            start_ms: 4_000,
            end_ms: 6_000,
            speaker: Some("Bob".to_string()),
            voice_id: Some("voice-fr".to_string()),
            translations: std::collections::BTreeMap::new(),
            source_text: "Até logo".to_string(),
            target_text: None,
            state: "imported".to_string(),
        });
        project.segments.push(crate::store::DubSegment {
            id: "p1-2".to_string(),
            position: 2,
            start_ms: 7_000,
            end_ms: 8_000,
            speaker: None,
            voice_id: None,
            translations: std::collections::BTreeMap::new(),
            source_text: "Sem voz".to_string(),
            target_text: None,
            state: "imported".to_string(),
        });

        let document: serde_json::Value =
            serde_json::from_str(&cue_document(&project).unwrap()).unwrap();

        assert_eq!(document["cues"][0]["voice"], "voice-en");
        assert_eq!(document["cues"][1]["voice"], "voice-fr");
        assert!(document["cues"][2]["voice"].is_null());
    }

    #[test]
    fn project_edits_keep_the_cast_and_refuse_an_unknown_voice() {
        let mut store = crate::store::Store::open_memory().unwrap();
        let mut project =
            crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
        project.segments.push(crate::store::DubSegment {
            id: "p1-0".to_string(),
            position: 0,
            start_ms: 1_250,
            end_ms: 3_500,
            speaker: Some("Ana".to_string()),
            voice_id: None,
            translations: std::collections::BTreeMap::new(),
            source_text: "Bom dia".to_string(),
            target_text: Some("Good morning".to_string()),
            state: "dubbed".to_string(),
        });
        store.save_dub_project(&mut project).unwrap();
        let edit = [SegmentEdit {
            id: "p1-0".to_string(),
            start_ms: 1_250,
            end_ms: 3_500,
            speaker: Some("Ana".to_string()),
            voice_id: Some("voice-fr".to_string()),
            source_text: "Bom dia".to_string(),
        }];
        let mut cast = std::collections::BTreeMap::new();
        cast.insert("Ana".to_string(), "voice-en".to_string());

        apply_project_edits(&mut project, &edit, &cast).unwrap();
        store.save_dub_project(&mut project).unwrap();
        let loaded = store.load_dub_project("p1").unwrap().unwrap();

        assert_eq!(loaded.cast.get("Ana").map(String::as_str), Some("voice-en"));
        assert_eq!(loaded.segments[0].voice_id.as_deref(), Some("voice-fr"));
        assert_eq!(
            loaded.segments[0].target_text.as_deref(),
            Some("Good morning")
        );
        assert_eq!(loaded.segments[0].state, "edited");

        let mut wrong = std::collections::BTreeMap::new();
        wrong.insert("Ana".to_string(), "voice-klingon".to_string());
        let refused = apply_project_edits(&mut project, &edit, &wrong).unwrap_err();
        assert!(refused.contains("is not a voice"), "{refused}");
    }

    #[test]
    fn a_finished_dub_updates_matching_project_segments() {
        let mut project =
            crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
        project.segments.push(crate::store::DubSegment {
            id: "p1-0".to_string(),
            position: 0,
            start_ms: 1_250,
            end_ms: 3_500,
            speaker: Some("Ana".to_string()),
            voice_id: None,
            translations: std::collections::BTreeMap::new(),
            source_text: "Bom dia".to_string(),
            target_text: None,
            state: "imported".to_string(),
        });
        let lines = vec![
            dub::Line {
                start: 1.25,
                end: 2.0,
                source: "Bom dia".to_string(),
                translated: "Good morning".to_string(),
                id: Some("p1-0".to_string()),
                voice: None,
                ..Default::default()
            },
            dub::Line {
                start: 4.0,
                end: 5.0,
                source: "Até logo".to_string(),
                translated: "See you".to_string(),
                id: Some("missing".to_string()),
                voice: None,
                ..Default::default()
            },
        ];

        apply_dub(&mut project, &lines, "en");

        assert_eq!(
            project.segments[0].target_text.as_deref(),
            Some("Good morning")
        );
        assert_eq!(project.segments[0].state, "dubbed");
    }

    #[test]
    fn a_project_keeps_one_translation_per_target_language() {
        let mut project =
            crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
        project.segments.push(crate::store::DubSegment {
            id: "p1-0".to_string(),
            position: 0,
            start_ms: 1_250,
            end_ms: 3_500,
            speaker: Some("Ana".to_string()),
            voice_id: Some("voice-en".to_string()),
            translations: std::collections::BTreeMap::new(),
            source_text: "Bom dia".to_string(),
            target_text: None,
            state: "imported".to_string(),
        });
        let line = |text: &str, voice: &str| dub::Line {
            start: 1.25,
            end: 2.0,
            source: "Bom dia".to_string(),
            translated: text.to_string(),
            id: Some("p1-0".to_string()),
            voice: Some(voice.to_string()),
            ..Default::default()
        };

        apply_dub(&mut project, &[line("Good morning", "voice-en")], "en");
        assert_eq!(project.targets, ["en"]);
        apply_dub(&mut project, &[line("Guten Morgen", "voice-de")], "de");

        assert_eq!(project.targets, ["en", "de"]);
        assert_eq!(
            project.segments[0]
                .translations
                .get("en")
                .map(String::as_str),
            Some("Good morning")
        );
        assert_eq!(
            project.segments[0]
                .translations
                .get("de")
                .map(String::as_str),
            Some("Guten Morgen")
        );
        assert_eq!(
            project.segments[0].target_text.as_deref(),
            Some("Good morning")
        );
    }

    #[test]
    fn reads_the_filename_header_encoded() {
        assert_eq!(percent_decode("my%20clip.mp4"), "my clip.mp4");
        assert_eq!(percent_decode("a+b.mov"), "a b.mov");
        assert_eq!(percent_decode("plain.mp4"), "plain.mp4");
        assert_eq!(percent_decode("broken%zz.mp4"), "broken%zz.mp4");
    }

    #[test]
    fn dubbed_names_keep_stem_and_suffix() {
        assert_eq!(dubbed_name("video.mp4", "en"), "video.en.mp4");
        assert_eq!(dubbed_name("talk.mkv", "pt"), "talk.pt.mkv");
        assert_eq!(dubbed_name("recording", "en"), "recording.en.wav");
    }

    #[test]
    fn content_types_follow_the_suffix() {
        assert_eq!(content_type("a.en.mp4"), "video/mp4");
        assert_eq!(content_type("a.en.srt"), "text/plain; charset=utf-8");
        assert_eq!(content_type("a.en.weird"), "application/octet-stream");
    }

    #[test]
    fn a_third_party_profile_cannot_skip_the_authorisation() {
        let refused = consent_for(
            Some("third-party"),
            false,
            "translation".into(),
            "local".into(),
            1,
        );
        assert!(refused.is_err());
        let allowed = consent_for(
            Some("third-party"),
            true,
            "translation".into(),
            "local".into(),
            1,
        )
        .unwrap();
        assert_eq!(allowed.origin.as_str(), "third-party");
        let own = consent_for(None, false, "translation".into(), "local".into(), 1).unwrap();
        assert_eq!(own.origin.as_str(), "own-voice");
    }

    #[test]
    fn a_stream_piece_carries_its_state_and_text() {
        let pieces = [parlo_core::engine::StreamPiece {
            state: "partial".to_string(),
            text: "bom dia".to_string(),
            audio_ms: 640,
            latency_ms: 210,
        }];
        let answer = pieces_json(&pieces);
        assert!(answer.contains("\"partial\""));
        assert!(answer.contains("bom dia"));
    }

    #[test]
    fn tokens_are_hex_and_unique_enough() {
        let token = new_token();
        assert_eq!(token.len(), 24);
        assert!(token.chars().all(|character| character.is_ascii_hexdigit()));
        assert_ne!(token, new_token());
    }

    fn start_server() -> (u16, Arc<AtomicBool>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let session =
            Session::open("echo".into(), "parakeet".into(), "pt".into(), "en".into()).unwrap();
        let accepting = Arc::new(AtomicBool::new(true));
        let stop = Arc::clone(&accepting);
        std::thread::spawn(move || listen(listener, "t0ken".into(), port, session, accepting));
        (port, stop)
    }

    fn send(port: u16, method: &str, path: &str, body: &str) -> String {
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nx-parlo-token: t0ken\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        ask(port, &request)
    }

    fn body_json(answer: &str) -> serde_json::Value {
        let body = answer.split("\r\n\r\n").nth(1).unwrap_or("");
        serde_json::from_str(body).unwrap()
    }

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
}
