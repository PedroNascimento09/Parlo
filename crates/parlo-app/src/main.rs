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

use tauri::{Manager, RunEvent, Url, WebviewWindow};

const READY_TIMEOUT: Duration = Duration::from_secs(240);

struct Engine(Mutex<Option<Child>>);

fn main() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .ok_or("the main window is missing")?;
            app.manage(Engine(Mutex::new(None)));
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
