#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    pub type Window = *mut c_void;

    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const KEYEVENTF_UNICODE: u32 = 0x0004;
    const VK_RETURN: u16 = 0x0D;

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
    }

    #[link(name = "kernel32")]
    extern "system" {
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
pub fn capture_target() -> Result<usize, String> {
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
    Ok((0, app))
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn foreground() -> Result<(usize, String), String> {
    Err(NO_FIELD.to_string())
}

#[cfg(windows)]
pub fn type_text(target: usize, text: &str) -> Result<(), String> {
    use win::*;
    let window = target as Window;
    if unsafe { GetForegroundWindow() } != window {
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

#[cfg(target_os = "linux")]
pub fn capture_target() -> Result<usize, String> {
    x11::active_window()
}

#[cfg(target_os = "macos")]
static CAPTURED_APP: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

#[cfg(target_os = "macos")]
pub fn capture_target() -> Result<usize, String> {
    let app = mac::frontmost()?;
    *CAPTURED_APP
        .lock()
        .map_err(|_| "the program in front could not be remembered".to_string())? = Some(app);
    Ok(0)
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn capture_target() -> Result<usize, String> {
    Err(NO_FIELD.to_string())
}

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
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn the_keyboard_input_matches_the_system_size() {
        assert_eq!(std::mem::size_of::<win::Input>(), 40);
    }

    #[cfg(windows)]
    #[test]
    fn a_letter_becomes_one_sixteen_bit_unit() {
        let encoded: Vec<u16> = "ã".encode_utf16().collect();
        assert_eq!(encoded, vec![0x00E3]);
    }

    #[cfg(windows)]
    #[test]
    fn one_press_and_one_release_are_queued_per_unit() {
        let pressed = win::Input::unicode(0x00E3, false);
        let released = win::Input::unicode(0x00E3, true);
        assert_eq!(pressed.kind, 1);
        assert_ne!(unsafe { pressed.union.keyboard.flags }, unsafe {
            released.union.keyboard.flags
        });
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    #[test]
    fn other_platforms_say_what_is_missing() {
        let error = type_text(0, "olá").unwrap_err();
        assert!(error.contains("no way to type"));
    }

    #[test]
    fn an_applescript_keystroke_escapes_quotes_and_backslashes() {
        let script = applescript_for("Diz \"olá\" \\ adeus");
        assert!(script.starts_with("tell application \"System Events\""));
        assert!(script.contains("keystroke \"Diz \\\"olá\\\" \\\\ adeus\""));
    }

    #[test]
    fn a_newline_becomes_a_return_key() {
        let script = applescript_for("primeira\nsegunda");
        assert_eq!(script.matches("keystroke").count(), 2);
        assert!(script.contains("key code 36"));
    }
}
