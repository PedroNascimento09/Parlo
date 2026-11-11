#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    pub type Window = *mut c_void;

    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const KEYEVENTF_UNICODE: u32 = 0x0004;
    const VK_RETURN: u16 = 0x0D;
    pub const SW_RESTORE: i32 = 9;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct MouseInput {
        pub dx: i32,
        pub dy: i32,
        pub mouse_data: u32,
        pub flags: u32,
        pub time: u32,
        pub extra: usize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct KeyboardInput {
        pub vk: u16,
        pub scan: u16,
        pub flags: u32,
        pub time: u32,
        pub extra: usize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct HardwareInput {
        pub message: u32,
        pub low: u16,
        pub high: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub union Union {
        pub mouse: MouseInput,
        pub keyboard: KeyboardInput,
        pub hardware: HardwareInput,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Input {
        pub kind: u32,
        pub union: Union,
    }

    impl Input {
        pub fn unicode(unit: u16, released: bool) -> Self {
            Self {
                kind: INPUT_KEYBOARD,
                union: Union {
                    keyboard: KeyboardInput {
                        vk: 0,
                        scan: unit,
                        flags: if released {
                            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                        } else {
                            KEYEVENTF_UNICODE
                        },
                        time: 0,
                        extra: 0,
                    },
                },
            }
        }

        pub fn enter(released: bool) -> Self {
            Self {
                kind: INPUT_KEYBOARD,
                union: Union {
                    keyboard: KeyboardInput {
                        vk: VK_RETURN,
                        scan: 0,
                        flags: if released { KEYEVENTF_KEYUP } else { 0 },
                        time: 0,
                        extra: 0,
                    },
                },
            }
        }
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn GetForegroundWindow() -> Window;
        pub fn GetWindowThreadProcessId(window: Window, process: *mut u32) -> u32;
        pub fn SendInput(count: u32, inputs: *const Input, size: i32) -> u32;
        pub fn SetForegroundWindow(window: Window) -> i32;
        pub fn ShowWindow(window: Window, command: i32) -> i32;
        pub fn IsIconic(window: Window) -> i32;
        pub fn BringWindowToTop(window: Window) -> i32;
        pub fn AttachThreadInput(mine: u32, theirs: u32, attach: i32) -> i32;
        pub fn IsWindow(window: Window) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetCurrentThreadId() -> u32;
        pub fn GetCurrentProcessId() -> u32;
        pub fn OpenProcess(access: u32, inherit: i32, process: u32) -> *mut c_void;
        pub fn QueryFullProcessImageNameW(
            handle: *mut c_void,
            flags: u32,
            buffer: *mut u16,
            size: *mut u32,
        ) -> i32;
        pub fn CloseHandle(handle: *mut c_void) -> i32;
    }

    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    pub fn process_name(process: u32) -> Option<String> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process);
            if handle.is_null() {
                return None;
            }
            let mut buffer = [0u16; 1024];
            let mut size = buffer.len() as u32;
            let answered = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size);
            CloseHandle(handle);
            if answered == 0 {
                return None;
            }
            let path = String::from_utf16_lossy(&buffer[..size as usize]);
            std::path::Path::new(&path)
                .file_stem()
                .and_then(|name| name.to_str())
                .map(str::to_string)
                .filter(|name| !name.is_empty())
        }
    }
}

#[cfg(windows)]
static LAST_TARGET: std::sync::Mutex<Option<usize>> = std::sync::Mutex::new(None);

#[cfg(windows)]
fn live_foreground() -> Result<usize, String> {
    use win::*;
    let window = unsafe { GetForegroundWindow() };
    if window.is_null() {
        return Err("no window has the focus right now".to_string());
    }
    let mut process = 0u32;
    unsafe {
        GetWindowThreadProcessId(window, &mut process);
    }
    if process == unsafe { GetCurrentProcessId() } {
        return Err("Parlo itself has the focus, so the text would land in Parlo".to_string());
    }
    Ok(window as usize)
}

#[cfg(windows)]
pub fn capture_target() -> Result<usize, String> {
    use win::*;
    match live_foreground() {
        Ok(window) => {
            if let Ok(mut slot) = LAST_TARGET.lock() {
                *slot = Some(window);
            }
            Ok(window)
        }
        Err(error) => {
            let remembered = LAST_TARGET.lock().ok().and_then(|slot| *slot);
            match remembered {
                Some(window) if unsafe { IsWindow(window as Window) } != 0 => Ok(window),
                _ => Err(error),
            }
        }
    }
}

#[cfg(windows)]
pub fn foreground() -> Result<(usize, String), String> {
    use win::*;
    let window = capture_target()?;
    let mut process = 0u32;
    unsafe {
        GetWindowThreadProcessId(window as Window, &mut process);
    }
    let app = process_name(process).unwrap_or_else(|| "another program".to_string());
    Ok((window, app))
}

#[cfg(target_os = "linux")]
pub fn foreground() -> Result<(usize, String), String> {
    let window = x11::active_window()?;
    Ok((window, x11::window_name(window)))
}

#[cfg(target_os = "macos")]
pub fn foreground() -> Result<(usize, String), String> {
    let app = mac::frontmost()?;
    if let Ok(mut slot) = CAPTURED_APP.lock() {
        *slot = Some(app.clone());
    }
    Ok((0, app))
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn foreground() -> Result<(usize, String), String> {
    Err(NO_FIELD.to_string())
}

#[cfg(windows)]
fn focus(window: win::Window) -> Result<(), ()> {
    use win::*;
    if unsafe { IsIconic(window) } != 0 {
        unsafe {
            ShowWindow(window, SW_RESTORE);
        }
    }
    for _ in 0..3 {
        if unsafe { GetForegroundWindow() } == window {
            return Ok(());
        }
        unsafe {
            let in_front = GetForegroundWindow();
            let mine = GetCurrentThreadId();
            let theirs = GetWindowThreadProcessId(in_front, std::ptr::null_mut());
            let attached = theirs != 0 && mine != theirs && AttachThreadInput(mine, theirs, 1) != 0;
            BringWindowToTop(window);
            SetForegroundWindow(window);
            if attached {
                AttachThreadInput(mine, theirs, 0);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        if unsafe { GetForegroundWindow() } == window {
            return Ok(());
        }
    }
    Err(())
}

#[cfg(windows)]
pub fn type_text(target: usize, text: &str) -> Result<(), String> {
    use win::*;
    let window = target as Window;
    if focus(window).is_err() {
        return Err("the window you were writing in is not in front any more".to_string());
    }
    let mut batch: Vec<Input> = Vec::new();
    for character in text.chars() {
        if character == '\n' {
            batch.push(Input::enter(false));
            batch.push(Input::enter(true));
        } else {
            let mut units = [0u16; 2];
            for unit in character.encode_utf16(&mut units) {
                batch.push(Input::unicode(*unit, false));
                batch.push(Input::unicode(*unit, true));
            }
        }
        if batch.len() >= 256 {
            send(&batch)?;
            batch.clear();
        }
    }
    if !batch.is_empty() {
        send(&batch)?;
    }
    Ok(())
}

#[cfg(windows)]
fn send(batch: &[win::Input]) -> Result<(), String> {
    let sent = unsafe {
        win::SendInput(
            batch.len() as u32,
            batch.as_ptr(),
            std::mem::size_of::<win::Input>() as i32,
        )
    };
    if sent as usize == batch.len() {
        Ok(())
    } else {
        Err("the system refused part of the text".to_string())
    }
}

#[cfg(target_os = "linux")]
mod x11 {
    use std::ffi::OsStr;
    use std::process::Command;

    pub fn run(program: &str, arguments: &[&OsStr]) -> Result<String, String> {
        let output = Command::new(program)
            .args(arguments)
            .output()
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::NotFound => {
                    format!("{program} is not installed, so the text goes to the clipboard instead")
                }
                _ => format!("{program} could not run: {error}"),
            })?;
        if !output.status.success() {
            let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if reason.is_empty() {
                format!("{program} refused the command")
            } else {
                format!("{program} says {reason}")
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn active_window() -> Result<usize, String> {
        if std::env::var("WAYLAND_DISPLAY")
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return Err(
                "Wayland does not let one program type into another window, so the text goes to the clipboard"
                    .to_string(),
            );
        }
        let named = run("xdotool", &[OsStr::new("getactivewindow")])?;
        named
            .parse::<usize>()
            .map_err(|_| "xdotool did not name the active window".to_string())
    }

    pub fn window_name(window: usize) -> String {
        let id = window.to_string();
        run("xdotool", &[OsStr::new("getwindowname"), OsStr::new(&id)])
            .unwrap_or_else(|_| "another program".to_string())
    }

    pub fn write(text: &str) -> Result<(), String> {
        run(
            "xdotool",
            &[
                OsStr::new("type"),
                OsStr::new("--clearmodifiers"),
                OsStr::new("--delay"),
                OsStr::new("0"),
                OsStr::new("--"),
                OsStr::new(text),
            ],
        )
        .map(|_| ())
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::OsStr;
    use std::process::Command;

    pub fn run(script: &str) -> Result<String, String> {
        let output = Command::new("osascript")
            .args([OsStr::new("-e"), OsStr::new(script)])
            .output()
            .map_err(|error| format!("osascript could not run: {error}"))?;
        if !output.status.success() {
            let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if reason.is_empty() {
                "macOS refused the keystroke; allow Parlo under Accessibility".to_string()
            } else {
                format!("macOS refused the keystroke: {reason}")
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn frontmost() -> Result<String, String> {
        run("tell application \"System Events\" to get name of first application process whose frontmost is true")
    }
}

#[cfg(any(target_os = "macos", test))]
fn applescript_for(text: &str) -> String {
    let mut body = String::new();
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            body.push_str("key code 36\n");
        }
        body.push_str("keystroke \"");
        for character in line.chars() {
            if character == '\\' || character == '"' {
                body.push('\\');
            }
            body.push(character);
        }
        body.push_str("\"\n");
    }
    format!(
        "tell application \"System Events\"\n{}\nend tell",
        body.trim_end()
    )
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
const NO_FIELD: &str =
    "this platform has no way to type into another window yet; the text goes to the clipboard";

#[cfg(target_os = "macos")]
static CAPTURED_APP: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

#[cfg(target_os = "linux")]
pub fn type_text(target: usize, text: &str) -> Result<(), String> {
    if x11::active_window()? != target {
        return Err("the window you were writing in is not in front any more".to_string());
    }
    x11::write(text)
}

#[cfg(target_os = "macos")]
pub fn type_text(_target: usize, text: &str) -> Result<(), String> {
    let app = mac::frontmost()?;
    let held = CAPTURED_APP
        .lock()
        .map_err(|_| "the program in front could not be read".to_string())?
        .clone();
    if held.as_deref() != Some(app.as_str()) {
        return Err("the program you were writing in is not in front any more".to_string());
    }
    mac::run(&applescript_for(text)).map(|_| ())
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn type_text(_target: usize, _text: &str) -> Result<(), String> {
    Err(NO_FIELD.to_string())
}

#[cfg(test)]
mod tests;

pub trait Field: Send + Sync {
    fn write(&self, window: usize, text: &str) -> Result<(), String>;
}

struct Native;

impl Field for Native {
    fn write(&self, window: usize, text: &str) -> Result<(), String> {
        type_text(window, text)
            .map_err(|error| format!("{error}. Copy the text and paste it yourself."))
    }
}

#[cfg(debug_assertions)]
struct Logged {
    path: std::path::PathBuf,
}

#[cfg(debug_assertions)]
impl Field for Logged {
    fn write(&self, window: usize, text: &str) -> Result<(), String> {
        event(&self.path, "write", window, text)
    }
}

#[cfg(debug_assertions)]
fn event(path: &std::path::Path, op: &str, window: usize, text: &str) -> Result<(), String> {
    use std::io::Write as _;
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let started = START.get_or_init(std::time::Instant::now);
    let line = serde_json::json!({
        "at_ms": started.elapsed().as_millis() as u64,
        "op": op,
        "window": window,
        "text": text,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())
}

#[cfg(debug_assertions)]
fn logged() -> Option<Box<dyn Field>> {
    let path = std::env::var("PARLO_FIELD_LOG").ok()?;
    if path.trim().is_empty() {
        return None;
    }
    Some(Box::new(Logged {
        path: std::path::PathBuf::from(path),
    }))
}

#[cfg(not(debug_assertions))]
fn logged() -> Option<Box<dyn Field>> {
    None
}

pub fn field() -> Box<dyn Field> {
    match logged() {
        Some(field) => field,
        None => Box::new(Native),
    }
}

pub fn replace_span(
    field: &dyn Field,
    window: usize,
    raw: &str,
    cleaned: &str,
) -> Result<(), String> {
    if !raw.is_empty() {
        return Err("Parlo cannot verify the previous text or cursor position, so it did not change the field.".to_string());
    }
    if !cleaned.is_empty() {
        field.write(window, cleaned)?;
    }
    Ok(())
}

#[cfg(test)]
mod sink_tests {
    use super::*;

    #[test]
    fn native_field_refuses_windows_that_are_not_in_front() {
        let field = Native;
        for window in [0, 1, usize::MAX] {
            assert!(field.write(window, "text").is_err());
            assert!(field.write(window, "").is_err());
        }
    }

    #[test]
    fn a_replacement_never_erases_or_writes_without_verification() {
        struct Recording(std::sync::Mutex<Vec<String>>);
        impl Field for Recording {
            fn write(&self, _window: usize, text: &str) -> Result<(), String> {
                self.0.lock().unwrap().push(text.to_string());
                Ok(())
            }
        }
        let field = Recording(std::sync::Mutex::new(Vec::new()));
        replace_span(&field, 1, "", "Bom dia.").unwrap();
        assert!(replace_span(&field, 1, "Bom dia.", "Bom dia!").is_err());
        assert!(replace_span(&field, 1, "Bom dia.", "Bom dia.").is_err());
        assert!(replace_span(&field, 1, "Bom dia.", "").is_err());
        assert_eq!(*field.0.lock().unwrap(), vec!["Bom dia."]);
    }
}
