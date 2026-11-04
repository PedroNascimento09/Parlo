use std::fs;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use parlo_core::settings::Intelligence;
use parlo_core::wav::{read_pcm16, write_pcm16};
use parlo_core::{AudioFormat, Consent, Pipeline, PipelineConfig, Store, Wav};

use crate::commands;
use crate::dub;
use crate::insert;
use crate::models;
use crate::undo;

use crate::update;

#[derive(PartialEq, Eq)]
enum Flow {
    Serve,
    Stop,
}

const MAX_HEAD: usize = 16 * 1024;
const MAX_BODY: usize = 20 * 1024 * 1024;
const MAX_DUB_BODY: usize = 512 * 1024 * 1024;
const KEPT_DUBS: usize = 8;

static STATE: std::sync::Mutex<parlo_core::state::Machine> =
    std::sync::Mutex::new(parlo_core::state::Machine::new());
const DUB_TIMEOUT: Duration = Duration::from_secs(3_600);
const VERSION: &str = env!("CARGO_PKG_VERSION");
const INDEX: &str = include_str!("../web/index.html");
const STYLESHEET: &str = include_str!("../web/app.css");
const SCRIPT: &str = include_str!("../web/app.js");
const ISLAND: &str = include_str!("../web/island.html");
const CAPTIONS: &str = include_str!("../web/captions.html");
const LIVE: &str = include_str!("../web/live.html");

pub fn new_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    format!("{nanos:016x}{:08x}", std::process::id())
}

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
        let pipeline = build(&from, &to, &engine, &stt)?;
        let mut chosen = self.choose();
        let shortcut = chosen.shortcut.clone();
        let hands_free = chosen.hands_free.clone();
        let captions = chosen.captions.clone();
        let undo = chosen.undo.clone();
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
        drop(chosen);
        *self.pipeline() = pipeline;
        if let Ok(mut target) = self.target.lock() {
            *target = None;
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
        let chosen = session.chosen();
        println!(
            "  engine     {} on {}",
            session.pipeline().engine_id(),
            chosen.stt
        );
        println!("  languages  {} to {}", chosen.from, chosen.to);
    }
    println!("  input      16 kHz mono wav, POST /translate");

    let address = format!("http://127.0.0.1:{port}");
    let shared = token.clone();
    std::thread::spawn(move || listen(listener, shared, port, session));
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

fn listen(listener: TcpListener, token: String, port: u16, session: Session) {
    let started = Instant::now();
    let session = Arc::new(session);
    let accepting = Arc::new(AtomicBool::new(true));
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

struct DubFile {
    id: String,
    upload: PathBuf,
    dubbed: PathBuf,
    name: String,
}

fn dub_files() -> &'static Mutex<Vec<DubFile>> {
    static FILES: OnceLock<Mutex<Vec<DubFile>>> = OnceLock::new();
    FILES.get_or_init(|| Mutex::new(Vec::new()))
}

fn remember(id: String, upload: PathBuf, dubbed: PathBuf, name: String) {
    let mut files = match dub_files().lock() {
        Ok(files) => files,
        Err(_) => return,
    };
    files.push(DubFile {
        id,
        upload,
        dubbed,
        name,
    });
    while files.len() > KEPT_DUBS {
        let oldest = files.remove(0);
        fs::remove_file(&oldest.upload).ok();
        fs::remove_file(&oldest.dubbed).ok();
    }
}

fn recall(id: &str) -> Option<(PathBuf, String)> {
    let files = dub_files().lock().ok()?;
    files
        .iter()
        .find(|file| file.id == id)
        .map(|file| (file.dubbed.clone(), file.name.clone()))
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

        println!("switching to {engine} on {stt}, {from} to {to}, loading models");
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

        let mut stored = crate::config::load();
        stored.engine = session.chosen().engine.clone();
        stored.stt = session.chosen().stt.clone();
        stored.from = session.chosen().from.clone();
        stored.to = session.chosen().to.clone();
        if let Err(error) = crate::config::save(&stored) {
            eprintln!("could not save the settings: {error}");
        }
        {
            let chosen = session.chosen();
            println!(
                "  ready       {} on {}, {} to {}",
                session.pipeline().engine_id(),
                chosen.stt,
                chosen.from,
                chosen.to
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

    if method == "GET" && path.starts_with("/dub/") {
        let id = path.trim_start_matches("/dub/");
        let Some((file, name)) = recall(id) else {
            return respond_json(stream, "404 Not Found", "{\"error\":\"not found\"}")
                .map(|()| Flow::Serve);
        };
        return respond_file(stream, &file, &name).map(|()| Flow::Serve);
    }

    if method == "GET" && path == "/models" {
        let lifecycle = session.lifecycle_snapshot();
        let pair = {
            let chosen = session.chosen();
            format!("{}-{}", chosen.from, chosen.to)
        };
        let body = models::as_json(&models::status(), &lifecycle, &pair, session.is_working());
        return respond_json(stream, "200 OK", &body).map(|()| Flow::Serve);
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
        if body.is_empty() {
            return respond_json(
                stream,
                "400 Bad Request",
                "{\"error\":\"the file is empty\"}",
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
            return respond_json(
                stream,
                "400 Bad Request",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve);
        }

        let requested = header(&headers, "x-parlo-filename")
            .map(|value| percent_decode(&value))
            .unwrap_or_else(|| "upload.mp4".to_string());
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

        let upload = directory.join(format!("{id}-{name}"));
        if let Err(error) = fs::write(&upload, &body) {
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

        let out_name = dubbed_name(&name, &to);
        let dubbed = directory.join(format!("{id}-{out_name}"));
        let subtitles = dubbed.with_extension("srt");
        let _ = stream.set_write_timeout(Some(DUB_TIMEOUT));
        println!("dubbing {name} from {from} to {to}");
        let result = dub::run(&dub::Request {
            source: &upload,
            from: &from,
            to: &to,
            out: Some(dubbed.clone()),
            srt: Some(subtitles),
            keep_background: header(&headers, "x-parlo-background").is_some(),
        });
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

        return match result {
            Ok(result) => {
                let output = PathBuf::from(&result.out);
                remember(id.clone(), upload, output, out_name.clone());
                let lines: Vec<String> = result
                    .lines
                    .iter()
                    .map(|line| {
                        format!(
                            "{{\"start\":{:.2},\"end\":{:.2},\"source\":\"{}\",\"translated\":\"{}\"}}",
                            line.start,
                            line.end,
                            json_string(&line.source),
                            json_string(&line.translated)
                        )
                    })
                    .collect();
                let body = format!(
                    "{{\"id\":\"{id}\",\"name\":\"{}\",\"seconds\":{:.2},\"lines\":[{}]}}",
                    json_string(&out_name),
                    result.seconds,
                    lines.join(",")
                );
                respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
            }
            Err(error) => {
                fs::remove_file(&upload).ok();
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
        if let Err(error) = models::install(models::REMAINING_UNITS) {
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
            if let Some(text) = output.transcript {
                if !transcript.is_empty() {
                    transcript.push(' ');
                }
                transcript.push_str(&text);
            }
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

fn error_json(message: &str) -> String {
    serde_json::json!({ "error": message }).to_string()
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

fn header(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
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

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let pair = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or_default();
                match u8::from_str_radix(pair, 16) {
                    Ok(byte) => {
                        decoded.push(byte);
                        index += 3;
                    }
                    Err(_) => {
                        decoded.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
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

fn content_type(name: &str) -> &'static str {
    match Path::new(name)
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
        .as_deref()
    {
        Some("mp4" | "m4v") => "video/mp4",
        Some("mkv") => "video/x-matroska",
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        Some("md") => "text/markdown; charset=utf-8",
        Some("wav") => "audio/wav",
        Some("mp3") => "audio/mpeg",
        Some("m4a") => "audio/mp4",
        Some("srt") => "text/plain; charset=utf-8",
        Some("tsv") => "text/tab-separated-values; charset=utf-8",
        _ => "application/octet-stream",
    }
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

fn host_is_local(headers: &[(String, String)]) -> bool {
    let Some((_, value)) = headers.iter().find(|(name, _)| name == "host") else {
        return false;
    };
    let host = value
        .rsplit_once(':')
        .map(|(host, _)| host)
        .unwrap_or(value)
        .trim_matches(['[', ']']);
    host.eq_ignore_ascii_case("127.0.0.1")
        || host.eq_ignore_ascii_case("localhost")
        || host == "::1"
}

fn origin_allowed(headers: &[(String, String)], port: u16) -> bool {
    let origins = [
        format!("http://127.0.0.1:{port}"),
        format!("http://localhost:{port}"),
    ];
    match headers.iter().find(|(name, _)| name == "origin") {
        None => true,
        Some((_, value)) => origins.iter().any(|origin| origin == value),
    }
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
    if path.starts_with("/dub") {
        MAX_DUB_BODY
    } else {
        MAX_BODY
    }
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
    fn compares_without_short_circuiting_on_length() {
        assert!(!constant_time_eq("abc", "ab"));
        assert!(constant_time_eq("abc", "abc"));
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
        assert_eq!(body_limit("POST /translate HTTP/1.1"), MAX_BODY);
        assert_eq!(body_limit(""), MAX_BODY);
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
}
