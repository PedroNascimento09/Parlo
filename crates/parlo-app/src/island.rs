use parlo_core::settings;
#[cfg(windows)]
use parlo_core::shortcut::Shortcut;
use tauri::{AppHandle, Manager};

pub const ISLAND: &str = "island";

pub fn show(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window(ISLAND) else {
        return;
    };
    let _ = window.show();
}

pub fn hide(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window(ISLAND) else {
        return;
    };
    let _ = window.hide();
}

pub fn toggle(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window(ISLAND) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        hide(handle);
    } else {
        show(handle);
    }
}

#[cfg(windows)]
fn tell(handle: &AppHandle, script: &str) {
    if let Some(window) = handle.get_webview_window(ISLAND) {
        let _ = window.eval(script);
    }
}

#[cfg(windows)]
pub fn listen(handle: AppHandle) {
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Mutex;
    use std::time::Instant;

    const WH_KEYBOARD_LL: i32 = 13;
    const WM_KEYDOWN: usize = 0x0100;
    const WM_KEYUP: usize = 0x0101;
    const WM_SYSKEYDOWN: usize = 0x0104;
    const WM_SYSKEYUP: usize = 0x0105;
    const VK_SHIFT: i32 = 0x10;
    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12;
    const VK_LWIN: i32 = 0x5B;
    const MOD_ALT: u32 = 0x0001;
    const MOD_CONTROL: u32 = 0x0002;
    const MOD_SHIFT: u32 = 0x0004;
    const MOD_WIN: u32 = 0x0008;
    const TAP_MS: u128 = 250;

    static APP: Mutex<Option<AppHandle>> = Mutex::new(None);
    static KEY: AtomicU32 = AtomicU32::new(0);
    static MODS: AtomicU32 = AtomicU32::new(0);
    static HELD: AtomicBool = AtomicBool::new(false);
    static HANDS_FREE: AtomicBool = AtomicBool::new(false);
    static PRESSED_AT: Mutex<Option<Instant>> = Mutex::new(None);

    #[repr(C)]
    struct Key {
        vk: u32,
        scan: u32,
        flags: u32,
        time: u32,
        extra: usize,
    }

    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct Message {
        window: *mut c_void,
        id: u32,
        w_param: usize,
        l_param: isize,
        time: u32,
        point: Point,
        private: u32,
    }

    extern "system" {
        fn SetWindowsHookExW(
            id: i32,
            procedure: unsafe extern "system" fn(i32, usize, isize) -> isize,
            instance: *mut c_void,
            thread: u32,
        ) -> *mut c_void;
        fn CallNextHookEx(hook: *mut c_void, code: i32, w_param: usize, l_param: isize) -> isize;
        fn GetAsyncKeyState(key: i32) -> i16;
        fn GetMessageW(message: *mut Message, window: *mut c_void, first: u32, last: u32) -> i32;
        fn TranslateMessage(message: *const Message) -> i32;
        fn DispatchMessageW(message: *const Message) -> isize;
    }

    fn modifiers_held() -> u32 {
        let mut held = 0;
        unsafe {
            if GetAsyncKeyState(VK_CONTROL) < 0 {
                held |= MOD_CONTROL;
            }
            if GetAsyncKeyState(VK_MENU) < 0 {
                held |= MOD_ALT;
            }
            if GetAsyncKeyState(VK_SHIFT) < 0 {
                held |= MOD_SHIFT;
            }
            if GetAsyncKeyState(VK_LWIN) < 0 {
                held |= MOD_WIN;
            }
        }
        held
    }

    fn with_app(run: impl FnOnce(&AppHandle)) {
        if let Some(handle) = APP.lock().ok().and_then(|slot| slot.clone()) {
            run(&handle);
        }
    }

    fn began() {
        if let Ok(mut slot) = PRESSED_AT.lock() {
            *slot = Some(Instant::now());
        }
        with_app(|handle| {
            show(handle);
            tell(handle, "window.parloBegin && window.parloBegin()");
        });
    }

    fn ended() {
        let held = PRESSED_AT
            .lock()
            .ok()
            .and_then(|mut slot| slot.take())
            .map(|at| at.elapsed())
            .unwrap_or_default();
        with_app(|handle| {
            if held.as_millis() < TAP_MS {
                if HANDS_FREE.swap(false, Ordering::Relaxed) {
                    tell(handle, "window.parloStop && window.parloStop()");
                } else {
                    HANDS_FREE.store(true, Ordering::Relaxed);
                    tell(handle, "window.parloHandsFree && window.parloHandsFree()");
                }
            } else {
                HANDS_FREE.store(false, Ordering::Relaxed);
                tell(handle, "window.parloStop && window.parloStop()");
            }
        });
    }

    unsafe extern "system" fn callback(code: i32, w_param: usize, l_param: isize) -> isize {
        if code >= 0 && l_param != 0 {
            let entry = &*(l_param as *const Key);
            let ours = entry.vk == KEY.load(Ordering::Relaxed);
            let pressed = w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN;
            let released = w_param == WM_KEYUP || w_param == WM_SYSKEYUP;
            let wanted = MODS.load(Ordering::Relaxed);
            let modifiers_match = wanted == 0 || wanted & !modifiers_held() == 0;
            if ours && modifiers_match {
                if pressed && !HELD.swap(true, Ordering::Relaxed) {
                    began();
                } else if released && HELD.swap(false, Ordering::Relaxed) {
                    ended();
                }
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param)
    }

    let text = settings::load().shortcut;
    let shortcut = match Shortcut::parse(&text) {
        Ok(shortcut) => shortcut,
        Err(error) => {
            crate::note(&format!(
                "the dictation shortcut {text} is unusable: {error}"
            ));
            return;
        }
    };
    KEY.store(shortcut.key, Ordering::Relaxed);
    MODS.store(shortcut.modifiers, Ordering::Relaxed);
    if let Ok(mut slot) = APP.lock() {
        *slot = Some(handle);
    }

    std::thread::spawn(move || unsafe {
        if SetWindowsHookExW(WH_KEYBOARD_LL, callback, std::ptr::null_mut(), 0).is_null() {
            crate::note("the keyboard hook did not install, so push to talk is unavailable");
            return;
        }
        crate::note(&format!(
            "dictation shortcut listening on {} with press and release",
            shortcut.text()
        ));
        let mut message: Message = std::mem::zeroed();
        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    });
}

#[cfg(not(windows))]
pub fn listen(_handle: AppHandle) {
    let text = settings::load().shortcut;
    crate::note(&format!(
        "the global dictation shortcut ({text}) is windows only for now"
    ));
}
