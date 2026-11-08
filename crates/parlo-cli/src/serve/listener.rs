use std::fs;
use std::io::{BufReader, Read};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::dub_exports::{share_target, Share};
use super::http::respond;
use super::session::Session;
use super::{handle, respond_json, Flow};

pub struct Server {
    pub address: String,
    pub token: String,
    pub share: Option<String>,
}

pub struct TlsFiles {
    pub certificate: PathBuf,
    pub key: PathBuf,
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
    let session = Arc::new(Session::open(engine, stt, from, to)?);
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

    crate::watch::start();
    let address = format!("http://127.0.0.1:{port}");
    let accepting = Arc::new(AtomicBool::new(true));
    let share = match share_target(&crate::config::load()) {
        Some((bind, share_port, pin)) => {
            let shared_listener = TcpListener::bind((bind.as_str(), share_port))
                .map_err(|error| format!("cannot bind {bind}:{share_port}: {error}"))?;
            println!("  share      http://{bind}:{share_port} with a PIN");
            let shared_token = token.clone();
            let shared_session = Arc::clone(&session);
            let shared_accepting = Arc::clone(&accepting);
            let gate = Arc::new(Share {
                pin,
                port: share_port,
            });
            std::thread::spawn(move || {
                listen(
                    shared_listener,
                    shared_token,
                    share_port,
                    shared_session,
                    shared_accepting,
                    Some(gate),
                )
            });
            Some(format!("http://{bind}:{share_port}"))
        }
        None => {
            println!("  share      off");
            None
        }
    };
    let shared = token.clone();
    std::thread::spawn(move || listen(listener, shared, port, session, accepting, None));
    Ok(Server {
        address,
        token,
        share,
    })
}

pub fn start_tls(
    port: u16,
    token: String,
    files: TlsFiles,
    engine: String,
    stt: String,
    from: String,
    to: String,
) -> Result<Server, String> {
    if std::env::var("PARLO_APP_GUARD").is_ok() {
        exit_with_parent();
    }
    let session = Arc::new(Session::open(engine, stt, from, to)?);
    let listener = TcpListener::bind(("0.0.0.0", port))
        .map_err(|error| format!("cannot bind 0.0.0.0:{port}: {error}"))?;
    let config = Arc::new(tls_server_config(&files.certificate, &files.key)?);
    println!("listening on https://0.0.0.0:{port}, plain HTTP is refused");
    println!("  token      {token}");
    crate::watch::start();
    let accepting = Arc::new(AtomicBool::new(true));
    let shared = token.clone();
    std::thread::spawn(move || listen_tls(listener, shared, port, session, accepting, config));
    Ok(Server {
        address: format!("https://0.0.0.0:{port}"),
        token,
        share: None,
    })
}

pub fn run_tls(
    port: u16,
    token: String,
    files: TlsFiles,
    engine: String,
    stt: String,
    from: String,
    to: String,
) -> Result<(), String> {
    let server = start_tls(port, token, files, engine, stt, from, to)?;
    println!("  api        {}", server.address);
    loop {
        std::thread::park();
    }
}

pub(super) fn tls_server_config(
    certificate: &Path,
    key: &Path,
) -> Result<rustls::ServerConfig, String> {
    let certs: Vec<_> = rustls_pemfile::certs(&mut BufReader::new(
        fs::File::open(certificate)
            .map_err(|error| format!("{}: {error}", certificate.display()))?,
    ))
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| format!("{}: {error}", certificate.display()))?;
    let private = rustls_pemfile::private_key(&mut BufReader::new(
        fs::File::open(key).map_err(|error| format!("{}: {error}", key.display()))?,
    ))
    .map_err(|error| format!("{}: {error}", key.display()))?
    .ok_or_else(|| format!("{} has no private key", key.display()))?;
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private)
        .map_err(|error| error.to_string())
}

pub(super) fn listen_tls(
    listener: TcpListener,
    token: String,
    port: u16,
    session: Arc<Session>,
    accepting: Arc<AtomicBool>,
    config: Arc<rustls::ServerConfig>,
) {
    let started = Instant::now();
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
        let mut first = [0u8; 1];
        let plain = match stream.peek(&mut first) {
            Ok(1) => first[0] != 0x16,
            _ => true,
        };
        if plain {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
            let mut sink = [0u8; 4096];
            while matches!(stream.read(&mut sink), Ok(read) if read > 0) {}
            let _ = respond(
                &mut stream,
                "426 Upgrade Required",
                "application/json",
                b"{\"error\":\"this port speaks HTTPS only\"}",
            );
            continue;
        }
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
        let config = Arc::clone(&config);
        std::thread::spawn(move || {
            let _open = open;
            let connection = match rustls::ServerConnection::new(config) {
                Ok(connection) => connection,
                Err(error) => {
                    eprintln!("TLS failed: {error}");
                    return;
                }
            };
            let mut tls = rustls::StreamOwned::new(connection, stream);
            match handle(&mut tls, &token, port, &session, started, None, true) {
                Ok(Flow::Serve) => {}
                Ok(Flow::Stop) => accepting.store(false, Ordering::Relaxed),
                Err(error) => eprintln!("request failed: {error}"),
            }
        });
    }
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

pub(super) fn listen(
    listener: TcpListener,
    token: String,
    port: u16,
    session: Arc<Session>,
    accepting: Arc<AtomicBool>,
    share: Option<Arc<Share>>,
) {
    let started = Instant::now();
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
        let share = share.clone();
        std::thread::spawn(move || {
            let _open = open;
            match handle(
                &mut stream,
                &token,
                port,
                &session,
                started,
                share.as_deref(),
                false,
            ) {
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
    if let Some(share) = &server.share {
        println!("  share      {share} with a PIN");
    }
    loop {
        std::thread::park();
    }
}
