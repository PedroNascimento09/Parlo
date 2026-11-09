#[cfg(not(windows))]
fn main() {
    println!("the push to talk probe needs windows");
}

#[cfg(windows)]
fn main() {
    if std::env::args().nth(1).as_deref() == Some("field") {
        win::field_host();
    }
    if let Err(error) = probe() {
        eprintln!("push to talk probe failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn probe() -> Result<(), String> {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut child = Command::new(exe)
        .arg("field")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| error.to_string())?;
    let stdout = child.stdout.take().ok_or("the field host has no output")?;
    let lines = win::watch_lines(stdout);
    let mut field = 0usize;
    loop {
        let line = win::next_line(&lines, Duration::from_secs(20))?;
        if let Some(rest) = line.strip_prefix("HWND ") {
            field = usize::from_str_radix(rest.trim(), 16)
                .map_err(|error| format!("the field window is unreadable: {error}"))?;
        }
        if line == "FOCUSED" {
            break;
        }
        if line == "NOFOCUS" {
            return Err("the field window never took the focus".to_string());
        }
    }

    let captured = parlo_cli::insert::capture_target()
        .map_err(|error| format!("the field could not be captured: {error}"))?;
    if captured != field {
        return Err(format!(
            "the captured window is 0x{captured:x} instead of the field 0x{field:x}"
        ));
    }
    println!("field captured: 0x{field:x}");

    let overlay = win::create_overlay()?;
    if !parlo_core::focus::reveal(overlay) {
        return Err("the island was not revealed".to_string());
    }
    if !win::visible(overlay) {
        return Err("the island did not become visible".to_string());
    }
    if !win::topmost(overlay) {
        return Err("the island is not topmost".to_string());
    }
    let foreground = win::foreground();
    if foreground != field {
        return Err(format!(
            "the island stole the focus: 0x{foreground:x} instead of 0x{field:x}"
        ));
    }
    println!(
        "island visible without focus theft: overlay=0x{overlay:x} field=0x{field:x} foreground=0x{foreground:x}"
    );

    win::watch_shortcut(
        0x44,
        parlo_core::hotkey::MOD_CONTROL | parlo_core::hotkey::MOD_ALT,
    );
    std::thread::sleep(Duration::from_millis(400));
    win::press(0x11);
    win::press(0x12);
    win::press(0x44);
    std::thread::sleep(Duration::from_millis(150));
    win::lift(0x11);
    let ended = win::wait_for_stop(Duration::from_secs(5));
    win::lift(0x44);
    win::lift(0x12);
    if !ended {
        return Err("the dictation kept running after a modifier was released".to_string());
    }
    println!("hold ended on release: the modifier lifted, the recording stopped");

    win::ask(&mut child, &lines, "clear")?;
    parlo_cli::insert::replace_span(&*parlo_cli::insert::field(), field, "", "Bom dia.")
        .map_err(|error| format!("the text could not be written: {error}"))?;
    let typed = win::wait_for_text(&mut child, &lines, "Bom dia.", Duration::from_secs(5))?;
    println!("text inserted into the field: {typed}");

    win::force_foreground(overlay);
    match parlo_cli::insert::capture_target() {
        Ok(remembered) if remembered == field => println!("target remembered: 0x{remembered:x}"),
        other => {
            return Err(format!(
                "the last target was not remembered when the island had the focus: {other:?}"
            ))
        }
    }

    let _ = win::ask(&mut child, &lines, "quit");
    let _ = child.kill();
    Ok(())
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Child, ChildStdout};
    use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
    use std::sync::mpsc::{channel, Receiver};
    use std::time::{Duration, Instant};

    pub type Hwnd = *mut c_void;

    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
    const WS_VISIBLE: u32 = 0x10000000;
    const WS_CHILD: u32 = 0x40000000;
    const WS_EX_TOOLWINDOW: u32 = 0x00000080;
    const WS_EX_TOPMOST: i32 = 0x00000008;
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const WS_BORDER: u32 = 0x00800000;
    const SW_SHOW: i32 = 5;
    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const WH_KEYBOARD_LL: i32 = 13;
    const WM_KEYDOWN: usize = 0x0100;
    const WM_KEYUP: usize = 0x0101;
    const WM_SYSKEYDOWN: usize = 0x0104;
    const WM_SYSKEYUP: usize = 0x0105;
    const WM_DESTROY: u32 = 0x0002;
    const GWL_EXSTYLE: i32 = -20;

    #[repr(C)]
    struct WndClassW {
        style: u32,
        proc_: Option<unsafe extern "system" fn(Hwnd, u32, usize, isize) -> isize>,
        cls_extra: i32,
        wnd_extra: i32,
        instance: Hwnd,
        icon: Hwnd,
        cursor: Hwnd,
        background: Hwnd,
        menu_name: *const u16,
        class_name: *const u16,
    }

    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct Message {
        window: Hwnd,
        id: u32,
        w_param: usize,
        l_param: isize,
        time: u32,
        point: Point,
        private: u32,
    }

    #[repr(C)]
    struct Key {
        vk: u32,
        scan: u32,
        flags: u32,
        time: u32,
        extra: usize,
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassW(class: *const WndClassW) -> u16;
        fn CreateWindowExW(
            style: u32,
            class: *const u16,
            title: *const u16,
            flags: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: Hwnd,
            menu: Hwnd,
            instance: Hwnd,
            param: *mut c_void,
        ) -> Hwnd;
        fn DefWindowProcW(window: Hwnd, message: u32, w_param: usize, l_param: isize) -> isize;
        fn GetMessageW(message: *mut Message, window: Hwnd, first: u32, last: u32) -> i32;
        fn TranslateMessage(message: *const Message) -> i32;
        fn DispatchMessageW(message: *const Message) -> isize;
        fn PostQuitMessage(code: i32);
        fn ShowWindow(window: Hwnd, command: i32) -> i32;
        fn IsWindowVisible(window: Hwnd) -> i32;
        fn GetForegroundWindow() -> Hwnd;
        fn SetForegroundWindow(window: Hwnd) -> i32;
        fn BringWindowToTop(window: Hwnd) -> i32;
        fn AttachThreadInput(mine: u32, theirs: u32, attach: i32) -> i32;
        fn GetWindowThreadProcessId(window: Hwnd, process: *mut u32) -> u32;
        fn GetClientRect(window: Hwnd, rect: *mut Rect) -> i32;
        fn MoveWindow(window: Hwnd, x: i32, y: i32, width: i32, height: i32, repaint: i32) -> i32;
        fn SetFocus(window: Hwnd) -> Hwnd;
        fn GetWindowTextW(window: Hwnd, text: *mut u16, length: i32) -> i32;
        fn SetWindowTextW(window: Hwnd, text: *const u16) -> i32;
        fn GetWindowLongW(window: Hwnd, index: i32) -> i32;
        fn keybd_event(vk: u8, scan: u8, flags: u32, extra: usize);
        fn SetWindowsHookExW(
            id: i32,
            procedure: unsafe extern "system" fn(i32, usize, isize) -> isize,
            instance: Hwnd,
            thread: u32,
        ) -> Hwnd;
        fn CallNextHookEx(hook: Hwnd, code: i32, w_param: usize, l_param: isize) -> isize;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> Hwnd;
        fn GetCurrentThreadId() -> u32;
    }

    static EDIT: AtomicIsize = AtomicIsize::new(0);
    static HELD: AtomicBool = AtomicBool::new(false);
    static STOPPED: AtomicBool = AtomicBool::new(false);
    static TRIGGER: AtomicU32 = AtomicU32::new(0);
    static MODIFIERS: AtomicU32 = AtomicU32::new(0);

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe extern "system" fn window_proc(
        window: Hwnd,
        message: u32,
        w_param: usize,
        l_param: isize,
    ) -> isize {
        if message == WM_DESTROY {
            PostQuitMessage(0);
            return 0;
        }
        DefWindowProcW(window, message, w_param, l_param)
    }

    unsafe extern "system" fn watch(code: i32, w_param: usize, l_param: isize) -> isize {
        if code >= 0 && l_param != 0 {
            let key = &*(l_param as *const Key);
            let pressed = w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN;
            let released = w_param == WM_KEYUP || w_param == WM_SYSKEYUP;
            let trigger = TRIGGER.load(Ordering::Relaxed);
            let modifiers = MODIFIERS.load(Ordering::Relaxed);
            if pressed && key.vk == trigger {
                HELD.store(true, Ordering::Relaxed);
            } else if released
                && HELD.load(Ordering::Relaxed)
                && parlo_core::hotkey::ends_hold(key.vk, trigger, modifiers)
            {
                HELD.store(false, Ordering::Relaxed);
                STOPPED.store(true, Ordering::Relaxed);
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param)
    }

    fn register(name: &str) -> Vec<u16> {
        let class = wide(name);
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
        let registered = WndClassW {
            style: 0,
            proc_: Some(window_proc),
            cls_extra: 0,
            wnd_extra: 0,
            instance,
            icon: std::ptr::null_mut(),
            cursor: std::ptr::null_mut(),
            background: std::ptr::null_mut(),
            menu_name: std::ptr::null(),
            class_name: class.as_ptr(),
        };
        unsafe {
            RegisterClassW(&registered);
        }
        class
    }

    pub fn field_host() -> ! {
        let class = register("ParloProbeField");
        let edit_class = wide("EDIT");
        let name = wide("Parlo probe field");
        let empty = wide("");
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
        let top = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                name.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                120,
                120,
                640,
                220,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null_mut(),
            )
        };
        let edit = unsafe {
            CreateWindowExW(
                0,
                edit_class.as_ptr(),
                empty.as_ptr(),
                WS_CHILD | WS_VISIBLE | ES_MULTILINE | ES_AUTOVSCROLL | WS_BORDER,
                0,
                0,
                600,
                160,
                top,
                std::ptr::null_mut(),
                instance,
                std::ptr::null_mut(),
            )
        };
        let mut rect = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        unsafe {
            GetClientRect(top, &mut rect);
            MoveWindow(edit, 0, 0, rect.right, rect.bottom, 1);
        }
        EDIT.store(edit as isize, Ordering::SeqCst);
        println!("HWND {:x}", top as usize);
        let _ = std::io::stdout().flush();
        let mut focused = false;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            force_foreground(top as usize);
            unsafe {
                SetFocus(edit);
            }
            if foreground_is(top) {
                focused = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        println!("{}", if focused { "FOCUSED" } else { "NOFOCUS" });
        let _ = std::io::stdout().flush();
        std::thread::spawn(commands);
        unsafe {
            let mut message: Message = std::mem::zeroed();
            while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        std::process::exit(0);
    }

    fn foreground_is(window: Hwnd) -> bool {
        unsafe { GetForegroundWindow() == window }
    }

    fn commands() {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            match line.trim() {
                "text" => {
                    let edit = EDIT.load(Ordering::SeqCst) as Hwnd;
                    let mut buffer = vec![0u16; 512];
                    let length = unsafe { GetWindowTextW(edit, buffer.as_mut_ptr(), 512) };
                    let text = String::from_utf16_lossy(&buffer[..length.max(0) as usize]);
                    println!("TEXT {text}");
                }
                "clear" => {
                    let edit = EDIT.load(Ordering::SeqCst) as Hwnd;
                    let empty = wide("");
                    unsafe {
                        SetWindowTextW(edit, empty.as_ptr());
                    }
                    println!("CLEARED");
                }
                "focus" => {
                    let top = unsafe { GetForegroundWindow() };
                    force_foreground(top as usize);
                    println!("FOCUSED");
                }
                "quit" => std::process::exit(0),
                _ => println!("OK"),
            }
            let _ = std::io::stdout().flush();
        }
        std::process::exit(0);
    }

    pub fn create_overlay() -> Result<usize, String> {
        let class = register("ParloProbeIsland");
        let name = wide("Parlo probe island");
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
        let window = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW,
                class.as_ptr(),
                name.as_ptr(),
                0,
                40,
                40,
                320,
                72,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null_mut(),
            )
        };
        if window.is_null() {
            return Err("the probe island was not created".to_string());
        }
        Ok(window as usize)
    }

    pub fn visible(window: usize) -> bool {
        unsafe { IsWindowVisible(window as Hwnd) != 0 }
    }

    pub fn topmost(window: usize) -> bool {
        unsafe { GetWindowLongW(window as Hwnd, GWL_EXSTYLE) & WS_EX_TOPMOST != 0 }
    }

    pub fn foreground() -> usize {
        unsafe { GetForegroundWindow() as usize }
    }

    pub fn force_foreground(window: usize) {
        unsafe {
            let mine = GetCurrentThreadId();
            let current = GetForegroundWindow();
            let theirs = GetWindowThreadProcessId(current, std::ptr::null_mut());
            AttachThreadInput(mine, theirs, 1);
            BringWindowToTop(window as Hwnd);
            ShowWindow(window as Hwnd, SW_SHOW);
            SetForegroundWindow(window as Hwnd);
            AttachThreadInput(mine, theirs, 0);
        }
    }

    pub fn press(vk: u8) {
        unsafe {
            keybd_event(vk, 0, 0, 0);
        }
    }

    pub fn lift(vk: u8) {
        unsafe {
            keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
        }
    }

    pub fn watch_shortcut(trigger: u32, modifiers: u32) {
        TRIGGER.store(trigger, Ordering::SeqCst);
        MODIFIERS.store(modifiers, Ordering::SeqCst);
        STOPPED.store(false, Ordering::SeqCst);
        HELD.store(false, Ordering::SeqCst);
        std::thread::spawn(|| unsafe {
            if SetWindowsHookExW(WH_KEYBOARD_LL, watch, std::ptr::null_mut(), 0).is_null() {
                return;
            }
            let mut message: Message = std::mem::zeroed();
            while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        });
    }

    pub fn wait_for_stop(limit: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < limit {
            if STOPPED.load(Ordering::SeqCst) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    pub fn watch_lines(stdout: ChildStdout) -> Receiver<String> {
        let (sender, receiver) = channel();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        receiver
    }

    pub fn next_line(lines: &Receiver<String>, limit: Duration) -> Result<String, String> {
        lines
            .recv_timeout(limit)
            .map_err(|error| format!("the field host went quiet: {error}"))
    }

    pub fn ask(
        child: &mut Child,
        lines: &Receiver<String>,
        command: &str,
    ) -> Result<String, String> {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "the field host takes no input".to_string())?;
        stdin
            .write_all(format!("{command}\n").as_bytes())
            .map_err(|error| error.to_string())?;
        stdin.flush().map_err(|error| error.to_string())?;
        next_line(lines, Duration::from_secs(5))
    }

    pub fn wait_for_text(
        child: &mut Child,
        lines: &Receiver<String>,
        wanted: &str,
        limit: Duration,
    ) -> Result<String, String> {
        let start = Instant::now();
        while start.elapsed() < limit {
            let reply = ask(child, lines, "text")?;
            if let Some(text) = reply.strip_prefix("TEXT ") {
                if text == wanted {
                    return Ok(text.to_string());
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(format!("the field never held {wanted:?}"))
    }
}
