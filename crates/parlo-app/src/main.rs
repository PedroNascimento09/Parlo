#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::hash_map::RandomState;
use std::fs::OpenOptions;
use std::hash::{BuildHasher, Hasher};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::webview::{PermissionKind, PermissionResponse};
use tauri::{Manager, PhysicalPosition, PhysicalSize, RunEvent, Url, WebviewWindow};

mod island;
mod presence;

const READY_TIMEOUT: Duration = Duration::from_secs(240);
const ISLAND_WIDTH: f64 = 640.0;
const ISLAND_HEIGHT: f64 = 132.0;
const ISLAND_MARGIN: f64 = 64.0;

struct Engine(Mutex<Option<Child>>);

struct Address(Mutex<Option<(u16, String)>>);

fn main() {
    let app = tauri::Builder::default()
        .on_permission_request(|_webview, kind| match kind {
            PermissionKind::Microphone => PermissionResponse::Allow,
            _ => PermissionResponse::Default,
        })
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .ok_or("the main window is missing")?;
            app.manage(Engine(Mutex::new(None)));
            app.manage(Address(Mutex::new(None)));
            if let Err(error) = presence::install(app.handle()) {
                note(&format!("the tray presence is not available: {error}"));
            }
            island::listen(app.handle().clone());
            let handle = app.handle().clone();
            std::thread::spawn(move || serve(handle, window));
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("cannot build the window");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            stop(handle);
        }
    });
}

fn serve(handle: tauri::AppHandle, window: WebviewWindow) {
    let (child, port, token) = match launch() {
        Ok(started) => started,
        Err(error) => {
            note(&error);
            return fail(&window);
        }
    };

    if let Some(engine) = handle.try_state::<Engine>() {
        if let Ok(mut slot) = engine.0.lock() {
            *slot = Some(child);
        }
    }

    if !ready(port) {
        note(&format!("the engine did not answer on port {port}"));
        stop(&handle);
        return fail(&window);
    }

    if let Some(address) = handle.try_state::<Address>() {
        if let Ok(mut slot) = address.0.lock() {
            *slot = Some((port, token.clone()));
        }
    }

    match Url::parse(&format!("http://127.0.0.1:{port}/?t={token}")) {
        Ok(url) => {
            if let Err(error) = window.navigate(url) {
                note(&format!("cannot open the app: {error}"));
                fail(&window);
            }
        }
        Err(error) => {
            note(&format!("cannot build the app address: {error}"));
            fail(&window);
        }
    }

    show_island(&handle, port, &token);

    let watcher = handle.clone();
    let watch_token = token.clone();
    std::thread::spawn(move || watch_island(watcher, port, watch_token));
}

fn show_island(handle: &tauri::AppHandle, port: u16, token: &str) {
    let Some(window) = handle.get_webview_window("island") else {
        note("the dictation island is missing from the window list");
        return;
    };
    place(&window);
    match Url::parse(&format!("http://127.0.0.1:{port}/island?t={token}")) {
        Ok(url) => {
            if let Err(error) = window.navigate(url) {
                note(&format!("cannot open the dictation island: {error}"));
            }
        }
        Err(error) => note(&format!("cannot build the island address: {error}")),
    }
}

pub fn captions_toggle(handle: &tauri::AppHandle) {
    surface_toggle(handle, "captions", "/captions");
}

pub fn undo_last(handle: &tauri::AppHandle) {
    let address = handle
        .try_state::<Address>()
        .and_then(|address| address.0.lock().ok().and_then(|slot| slot.clone()));
    let Some((port, token)) = address else {
        note("undo needs the engine address, which is not ready yet");
        return;
    };
    match post(port, &token, "/undo") {
        Ok(answer) => note(&format!("undo: {answer}")),
        Err(error) => note(&format!("undo: {error}")),
    }
}

fn post(port: u16, token: &str, path: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|error| error.to_string())?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let request = format!(
        "POST {path} HTTP/1.1\nHost: 127.0.0.1\nAuthorization: Bearer {token}\nContent-Length: 0\nConnection: close\n\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| error.to_string())?;
    let mut answer = String::new();
    stream
        .read_to_string(&mut answer)
        .map_err(|error| error.to_string())?;
    Ok(answer.lines().last().unwrap_or("").trim().to_string())
}

pub fn live_toggle(handle: &tauri::AppHandle) {
    surface_toggle(handle, "live", "/live");
}

fn surface_toggle(handle: &tauri::AppHandle, label: &str, page: &str) {
    let Some(window) = handle.get_webview_window(label) else {
        note(&format!(
            "the {label} window is missing from the window list"
        ));
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }
    let address = handle
        .try_state::<Address>()
        .and_then(|address| address.0.lock().ok().and_then(|slot| slot.clone()));
    let Some((port, token)) = address else {
        note(&format!(
            "{label} needs the engine address, which is not ready yet"
        ));
        return;
    };
    match Url::parse(&format!("http://127.0.0.1:{port}{page}?t={token}")) {
        Ok(url) => {
            if let Err(error) = window.navigate(url) {
                note(&format!("cannot open {label}: {error}"));
                return;
            }
            let _ = window.show();
        }
        Err(error) => note(&format!("cannot build the {label} address: {error}")),
    }
}

fn watch_island(handle: tauri::AppHandle, port: u16, token: String) {
    let mut quiet = 0;
    loop {
        std::thread::sleep(Duration::from_millis(500));
        let state = island_state(port, &token);
        if let Some(name) = state.as_deref() {
            presence::set_status(&handle, &format!("Parlo · {name}"));
        }
        let calm = matches!(state.as_deref(), Some("idle" | "success" | "cancelled"));
        let Some(window) = handle.get_webview_window(island::ISLAND) else {
            quiet = 0;
            continue;
        };
        if !window.is_visible().unwrap_or(false) || !calm {
            quiet = 0;
            continue;
        }
        quiet += 1;
        if quiet >= 3 {
            let _ = window.hide();
            quiet = 0;
        }
    }
}

fn island_state(port: u16, token: &str) -> Option<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let request = format!(
        "GET /state HTTP/1.1
Host: 127.0.0.1
Authorization: Bearer {token}
Connection: close

"
    );
    stream.write_all(request.as_bytes()).ok()?;
    let mut answer = String::new();
    stream.read_to_string(&mut answer).ok()?;
    for state in [
        "idle",
        "armed",
        "listening",
        "paused",
        "processing",
        "inserting",
        "success",
        "error",
        "cancelled",
    ] {
        if answer.contains(&format!("\"state\":\"{state}\"")) {
            return Some(state.to_string());
        }
    }
    None
}

fn place(window: &WebviewWindow) {
    let monitor = window
        .cursor_position()
        .ok()
        .and_then(|cursor| window.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        note("no monitor was reported, the dictation island keeps its default place");
        return;
    };
    let scale = monitor.scale_factor();
    let origin = monitor.position();
    let screen = monitor.size();
    let size = window.outer_size().unwrap_or(PhysicalSize::new(
        (ISLAND_WIDTH * scale) as u32,
        (ISLAND_HEIGHT * scale) as u32,
    ));
    let margin = (ISLAND_MARGIN * scale) as i32;
    let x = origin.x + (screen.width as i32 - size.width as i32) / 2;
    let y = origin.y + screen.height as i32 - size.height as i32 - margin;
    let _ = window.set_position(PhysicalPosition::new(x.max(origin.x), y.max(origin.y)));
}

fn launch() -> Result<(Child, u16, String), String> {
    let port = free_port()?;
    let token = fresh_token();
    let log = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(std::env::temp_dir().join("parlo-engine.log"))
        .map_err(|error| format!("cannot write the engine log: {error}"))?;
    let mut command = Command::new(parlo_binary());
    command
        .arg("serve")
        .args(["--port", &port.to_string()])
        .args(["--token", &token])
        .env("PARLO_APP_GUARD", "1")
        .stdin(Stdio::piped())
        .stdout(log.try_clone().map_err(|error| error.to_string())?)
        .stderr(log);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }

    command
        .spawn()
        .map(|child| (child, port, token))
        .map_err(|error| format!("cannot start the engine: {error}"))
}

fn parlo_binary() -> PathBuf {
    let folder = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from));
    let name = if cfg!(windows) { "parlo.exe" } else { "parlo" };
    match folder {
        Some(folder) => {
            let beside = folder.join(name);
            if beside.is_file() {
                beside
            } else {
                PathBuf::from(name)
            }
        }
        None => PathBuf::from(name),
    }
}

fn free_port() -> Result<u16, String> {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).map_err(|error| format!("no free port: {error}"))?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|error| error.to_string())
}

fn fresh_token() -> String {
    (0..4)
        .map(|_| {
            let mut hasher = RandomState::new().build_hasher();
            hasher.write_u64(now());
            format!("{:016x}", hasher.finish())
        })
        .collect()
}

fn ready(port: u16) -> bool {
    let deadline = Instant::now() + READY_TIMEOUT;
    while Instant::now() < deadline {
        if replies(port) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    false
}

fn replies(port: u16) -> bool {
    let mut stream = match TcpStream::connect(("127.0.0.1", port)) {
        Ok(stream) => stream,
        Err(_) => return false,
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let request = "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut head = [0u8; 16];
    match stream.read(&mut head) {
        Ok(read) => String::from_utf8_lossy(&head[..read]).starts_with("HTTP/1.1 200"),
        Err(_) => false,
    }
}

fn fail(window: &WebviewWindow) {
    let Ok(mut url) = window.url() else {
        return;
    };
    url.set_path("/failed.html");
    let _ = window.navigate(url);
}

fn stop(handle: &tauri::AppHandle) {
    if let Some(engine) = handle.try_state::<Engine>() {
        if let Ok(mut slot) = engine.0.lock() {
            if let Some(mut child) = slot.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

fn note(line: &str) {
    let path = std::env::temp_dir().join("parlo-app.log");
    let stamp = now();
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{stamp} {line}");
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}
