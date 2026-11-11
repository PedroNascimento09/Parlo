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
fn step(text: &str) {
    use std::io::Write;
    println!("step: {text}");
    let _ = std::io::stdout().flush();
}

#[cfg(windows)]
struct Cleanup {
    app: std::process::Child,
    field: Option<std::process::Child>,
}

#[cfg(windows)]
impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Some(field) = self.field.as_mut() {
            let _ = field.kill();
            let _ = field.wait();
        }
        let _ = self.app.kill();
        let _ = self.app.wait();
    }
}

#[cfg(windows)]
fn probe() -> Result<(), String> {
    use std::fs;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    let root = std::env::temp_dir().join(format!("parlo-probe-{}", std::process::id()));
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let config = root.join("config.json");
    fs::write(&config, "{\"shortcut\":\"Ctrl+Alt+F9\"}").map_err(|error| error.to_string())?;
    let address = root.join("address.json");
    let _ = fs::remove_file(&address);

    let app = std::env::var("PARLO_APP_EXE")
        .map_err(|_| "PARLO_APP_EXE is unset, so there is no release app to drive".to_string())?;
    if !std::path::Path::new(&app).is_file() {
        return Err(format!("the release app is missing at {app}"));
    }

    let running = Command::new(&app)
        .env("PARLO_CONFIG", &config)
        .env("PARLO_DATA_DIR", root.join("data"))
        .env("PARLO_MODELS_DIR", root.join("models"))
        .env("PARLO_APP_PROBE", &address)
        .env("PARLO_SELFTEST_TRANSCRIPT", "Bom dia.")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("the release app did not start: {error}"))?;
    step("app started");
    let pid = running.id();
    let mut guard = Cleanup {
        app: running,
        field: None,
    };

    let port = win::wait_for_address(&address, Duration::from_secs(180))?;
    let token = win::read_token(&address)?;
    step("address written");
    win::wait_for_health(port, Duration::from_secs(30))?;
    step("engine answers");
    let island = win::find_window(pid, "Parlo Dictate", Duration::from_secs(30))?;
    step("island window exists");

    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut field_host = Command::new(exe)
        .arg("field")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| error.to_string())?;
    let stdout = field_host
        .stdout
        .take()
        .ok_or("the field host has no output")?;
    guard.field = Some(field_host);
    let field_host = guard.field.as_mut().ok_or("the field host vanished")?;
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
    if win::foreground() != field {
        return Err(format!(
            "the field is not in front before the hold: 0x{:x}",
            win::foreground()
        ));
    }

    step("field is in front");
    let before = win::capture_window(island);
    step("captured the hidden island");
    win::press(0x11);
    win::press(0x12);
    win::press(0x78);
    step("shortcut held");
    for _ in 0..15 {
        std::thread::sleep(Duration::from_millis(200));
        step(&format!("state {}", win::state(port, &token)));
    }
    let shown = Instant::now();
    while shown.elapsed() < Duration::from_secs(10) {
        if win::visible(island) && win::client_size(island) != (0, 0) {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    if !win::visible(island) {
        return Err("the island never became visible while the shortcut was held".to_string());
    }
    std::thread::sleep(Duration::from_millis(400));
    let (width, height) = win::client_size(island);
    if width == 0 || height == 0 {
        return Err("the island is visible but has no client area".to_string());
    }
    step("island is visible");
    let after = win::capture_window(island);
    let painted = win::changed(&before, &after);
    if painted < 200 {
        return Err(format!(
            "the island window changed only {painted} pixels, so nothing was painted (captured {} and {} pixels)",
            before.len(),
            after.len()
        ));
    }
    let focused = win::foreground();
    if focused != field {
        return Err(format!(
            "the island took the focus: 0x{focused:x} instead of the field 0x{field:x} (island 0x{island:x})"
        ));
    }
    println!(
        "island visible and painted: window=0x{island:x} client={width}x{height} changed_pixels={painted} foreground=0x{focused:x}"
    );

    step("releasing the shortcut");
    win::lift(0x78);
    win::lift(0x12);
    win::lift(0x11);
    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(250));
        step(&format!(
            "after release state {} raw {:?}",
            win::state(port, &token),
            win::get(port, "/state", Some(&token))
        ));
    }
    let typed = win::wait_for_text(field_host, &lines, "Bom dia.", Duration::from_secs(30))?;
    win::lift(0x78);
    win::lift(0x12);
    println!("recording ended on release: the modifier lifted, no stop click");
    println!("text inserted into the field: {typed}");

    let _ = win::ask(field_host, &lines, "quit");
    Ok(())
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    use std::fs;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpStream;
    use std::path::Path;
    use std::process::{Child, ChildStdout};
    use std::sync::atomic::{AtomicIsize, AtomicU32, AtomicUsize, Ordering};
    use std::sync::mpsc::{channel, Receiver};
    use std::time::{Duration, Instant};

    pub type Hwnd = *mut c_void;

    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
    const WS_VISIBLE: u32 = 0x10000000;
    const WS_CHILD: u32 = 0x40000000;
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const WS_BORDER: u32 = 0x00800000;
    const SW_SHOW: i32 = 5;
    const WM_DESTROY: u32 = 0x0002;
    const SRCCOPY: u32 = 0x00CC0020;
    const DIB_RGB_COLORS: u32 = 0;

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
    #[derive(Clone, Copy, Default)]
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
    struct BitmapInfoHeader {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: u32,
        size_image: u32,
        x_pels: i32,
        y_pels: i32,
        colors_used: u32,
        important: u32,
    }

    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [u32; 1],
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
        fn GetWindowRect(window: Hwnd, rect: *mut Rect) -> i32;
        fn MoveWindow(window: Hwnd, x: i32, y: i32, width: i32, height: i32, repaint: i32) -> i32;
        fn SetFocus(window: Hwnd) -> Hwnd;
        fn GetWindowTextW(window: Hwnd, text: *mut u16, length: i32) -> i32;
        fn SetWindowTextW(window: Hwnd, text: *const u16) -> i32;
        fn keybd_event(vk: u8, scan: u8, flags: u32, extra: usize);
        fn EnumWindows(
            procedure: unsafe extern "system" fn(Hwnd, isize) -> i32,
            param: isize,
        ) -> i32;
        fn GetDC(window: Hwnd) -> Hwnd;
        fn ReleaseDC(window: Hwnd, dc: Hwnd) -> i32;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateCompatibleDC(dc: Hwnd) -> Hwnd;
        fn CreateCompatibleBitmap(dc: Hwnd, width: i32, height: i32) -> Hwnd;
        fn SelectObject(dc: Hwnd, object: Hwnd) -> Hwnd;
        fn BitBlt(
            target: Hwnd,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            source: Hwnd,
            source_x: i32,
            source_y: i32,
            flags: u32,
        ) -> i32;
        fn GetDIBits(
            dc: Hwnd,
            bitmap: Hwnd,
            start: u32,
            lines: u32,
            bits: *mut c_void,
            info: *mut BitmapInfo,
            usage: u32,
        ) -> i32;
        fn DeleteObject(object: Hwnd) -> i32;
        fn DeleteDC(dc: Hwnd) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> Hwnd;
        fn GetCurrentThreadId() -> u32;
    }

    static EDIT: AtomicIsize = AtomicIsize::new(0);
    static WANTED: AtomicU32 = AtomicU32::new(0);
    static FOUND: AtomicUsize = AtomicUsize::new(0);

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

    unsafe extern "system" fn each_window(window: Hwnd, _param: isize) -> i32 {
        let mut process = 0u32;
        GetWindowThreadProcessId(window, &mut process);
        if process != WANTED.load(Ordering::SeqCst) {
            return 1;
        }
        if title(window) != "Parlo Dictate" {
            return 1;
        }
        FOUND.store(window as usize, Ordering::SeqCst);
        0
    }

    fn title(window: Hwnd) -> String {
        let mut buffer = vec![0u16; 256];
        let length = unsafe { GetWindowTextW(window, buffer.as_mut_ptr(), 256) };
        String::from_utf16_lossy(&buffer[..length.max(0) as usize])
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
        let mut rect = Rect::default();
        unsafe {
            GetClientRect(top, &mut rect);
            MoveWindow(edit, 0, 0, rect.right, rect.bottom, 1);
        }
        EDIT.store(edit as isize, Ordering::SeqCst);
        println!("HWND {:x}", top as usize);
        let _ = std::io::stdout().flush();
        force_foreground(top as usize);
        unsafe {
            SetFocus(edit);
        }
        let focused = foreground() == top as usize;
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
                "quit" => std::process::exit(0),
                _ => println!("OK"),
            }
            let _ = std::io::stdout().flush();
        }
        std::process::exit(0);
    }

    pub fn press(vk: u8) {
        unsafe {
            keybd_event(vk, 0, 0, 0);
        }
    }

    pub fn lift(vk: u8) {
        unsafe {
            keybd_event(vk, 0, 0x0002, 0);
        }
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

    pub fn foreground() -> usize {
        unsafe { GetForegroundWindow() as usize }
    }

    pub fn visible(window: usize) -> bool {
        unsafe { IsWindowVisible(window as Hwnd) != 0 }
    }

    pub fn client_size(window: usize) -> (i32, i32) {
        let mut rect = Rect::default();
        unsafe {
            GetClientRect(window as Hwnd, &mut rect);
        }
        (rect.right - rect.left, rect.bottom - rect.top)
    }

    pub fn capture_window(window: usize) -> Vec<u32> {
        let mut rect = Rect::default();
        unsafe {
            GetWindowRect(window as Hwnd, &mut rect);
        }
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return Vec::new();
        }
        unsafe {
            let screen = GetDC(std::ptr::null_mut());
            if screen.is_null() {
                return Vec::new();
            }
            let memory = CreateCompatibleDC(screen);
            let bitmap = CreateCompatibleBitmap(screen, width, height);
            let previous = SelectObject(memory, bitmap);
            BitBlt(
                memory, 0, 0, width, height, screen, rect.left, rect.top, SRCCOPY,
            );
            let mut info = BitmapInfo {
                header: BitmapInfoHeader {
                    size: std::mem::size_of::<BitmapInfoHeader>() as u32,
                    width,
                    height: -height,
                    planes: 1,
                    bit_count: 32,
                    compression: 0,
                    size_image: 0,
                    x_pels: 0,
                    y_pels: 0,
                    colors_used: 0,
                    important: 0,
                },
                colors: [0],
            };
            let mut pixels = vec![0u32; (width * height) as usize];
            let copied = GetDIBits(
                memory,
                bitmap,
                0,
                height as u32,
                pixels.as_mut_ptr() as *mut c_void,
                &mut info,
                DIB_RGB_COLORS,
            );
            SelectObject(memory, previous);
            DeleteObject(bitmap);
            DeleteDC(memory);
            ReleaseDC(std::ptr::null_mut(), screen);
            if copied == 0 {
                Vec::new()
            } else {
                pixels
            }
        }
    }

    pub fn changed(before: &[u32], after: &[u32]) -> usize {
        if before.len() != after.len() || before.is_empty() {
            return 0;
        }
        before
            .iter()
            .zip(after.iter())
            .filter(|(left, right)| **left != **right)
            .filter(|(left, right)| {
                let (left, right) = (**left, **right);
                let spread = |shift: u32| {
                    let a = (left >> shift) & 0xFF;
                    let b = (right >> shift) & 0xFF;
                    a.abs_diff(b)
                };
                spread(0) > 12 || spread(8) > 12 || spread(16) > 12
            })
            .count()
    }

    pub fn get(port: u16, path: &str, token: Option<&str>) -> Result<String, String> {
        let mut stream =
            TcpStream::connect(("127.0.0.1", port)).map_err(|error| error.to_string())?;
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let auth = token
            .map(|token| format!("Authorization: Bearer {token}\r\n"))
            .unwrap_or_default();
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{auth}Connection: close\r\n\r\n");
        stream
            .write_all(request.as_bytes())
            .map_err(|error| error.to_string())?;
        let mut answer = String::new();
        stream
            .read_to_string(&mut answer)
            .map_err(|error| error.to_string())?;
        Ok(answer)
    }

    pub fn wait_for_address(path: &Path, limit: Duration) -> Result<u16, String> {
        let start = Instant::now();
        while start.elapsed() < limit {
            if let Ok(text) = fs::read_to_string(path) {
                if let Some(port) = text
                    .split("\"port\":")
                    .nth(1)
                    .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
                    .and_then(|digits| digits.parse::<u16>().ok())
                {
                    return Ok(port);
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err("the release app never reported its engine address".to_string())
    }

    pub fn read_token(path: &Path) -> Result<String, String> {
        let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
        text.split("\"token\":\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .map(|token| token.to_string())
            .ok_or_else(|| "the address file carries no token".to_string())
    }

    pub fn state(port: u16, token: &str) -> String {
        match get(port, "/state", Some(token)) {
            Ok(answer) => answer
                .split("\"state\":\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .unwrap_or("unknown")
                .to_string(),
            Err(error) => error,
        }
    }

    pub fn wait_for_health(port: u16, limit: Duration) -> Result<(), String> {
        let start = Instant::now();
        while start.elapsed() < limit {
            if let Ok(answer) = get(port, "/health", None) {
                if answer.starts_with("HTTP/1.1 200") || answer.starts_with("HTTP/1.0 200") {
                    return Ok(());
                }
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        Err("the engine never answered the health check".to_string())
    }

    pub fn find_window(pid: u32, wanted: &str, limit: Duration) -> Result<usize, String> {
        WANTED.store(pid, Ordering::SeqCst);
        FOUND.store(0, Ordering::SeqCst);
        let start = Instant::now();
        while start.elapsed() < limit {
            FOUND.store(0, Ordering::SeqCst);
            unsafe {
                EnumWindows(each_window, 0);
            }
            let found = FOUND.load(Ordering::SeqCst);
            if found != 0 {
                let _ = wanted;
                return Ok(found);
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        Err(format!("no window titled {wanted:?} appeared"))
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
            std::thread::sleep(Duration::from_millis(200));
        }
        Err(format!("the field never held {wanted:?}"))
    }
}
