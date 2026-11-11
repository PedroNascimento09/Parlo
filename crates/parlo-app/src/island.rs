use parlo_core::settings;
#[cfg(windows)]
use parlo_core::shortcut::Shortcut;
use tauri::{AppHandle, Manager};

pub const ISLAND: &str = "island";

pub fn show(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window(ISLAND) else {
        return;
    };
    if !window.is_visible().unwrap_or(false) {
        crate::place(&window);
        let _ = window.eval("window.parloResizeRecovery && window.parloResizeRecovery()");
    }
    #[cfg(windows)]
    {
        let raw = window.hwnd().ok().map(|handle| handle.0 as usize);
        if let Some(raw) = raw {
            parlo_core::focus::no_activate(raw, true);
            if parlo_core::focus::reveal(raw) {
                return;
            }
        }
    }
    if window.show().is_err() {
        crate::note("the dictation island did not appear");
    }
}

pub fn hide(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window(ISLAND) else {
        return;
    };
    let _ = window.hide();
}

pub fn review_recovery(handle: &AppHandle) {
    show(handle);
    if let Some(window) = handle.get_webview_window(ISLAND) {
        #[cfg(windows)]
        if let Ok(raw) = window.hwnd() {
            parlo_core::focus::no_activate(raw.0 as usize, false);
        }
        if let Err(error) = window.set_focus() {
            crate::note(&format!(
                "the dictation window could not receive focus: {error}"
            ));
        }
        tell(
            handle,
            "window.parloReviewRecovery && window.parloReviewRecovery()",
        );
    }
}

pub fn copy_recovery(handle: &AppHandle) {
    show(handle);
    tell(
        handle,
        "window.parloCopyRecovery && window.parloCopyRecovery()",
    );
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
    use std::time::Duration;
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
    const VK_ESCAPE: u32 = 0x1B;
    const MOD_ALT: u32 = 0x0001;
    const MOD_CONTROL: u32 = 0x0002;
    const MOD_SHIFT: u32 = 0x0004;
    const MOD_WIN: u32 = 0x0008;
    const HOLD_MS: u128 = 250;

    static APP: Mutex<Option<AppHandle>> = Mutex::new(None);
    static KEY: AtomicU32 = AtomicU32::new(0);
    static MODS: AtomicU32 = AtomicU32::new(0);
    static FREE_KEY: AtomicU32 = AtomicU32::new(0);
    static FREE_MODS: AtomicU32 = AtomicU32::new(0);
    static CAPTIONS_KEY: AtomicU32 = AtomicU32::new(0);
    static CAPTIONS_MODS: AtomicU32 = AtomicU32::new(0);
    static COMMAND_KEY: AtomicU32 = AtomicU32::new(0);
    static COMMAND_MODS: AtomicU32 = AtomicU32::new(0);
    static COPY_SHORTCUT: AtomicU32 = AtomicU32::new(0);
    static HELD: AtomicBool = AtomicBool::new(false);
    static RECORDING: AtomicBool = AtomicBool::new(false);
    static ESCAPE_LATCH: AtomicBool = AtomicBool::new(false);
    static FREE_LATCH: AtomicBool = AtomicBool::new(false);
    static CAPTIONS_LATCH: AtomicBool = AtomicBool::new(false);
    static COMMAND_LATCH: AtomicBool = AtomicBool::new(false);
    static COPY_LATCH: AtomicBool = AtomicBool::new(false);
    static COPY_HELD_KEY: AtomicU32 = AtomicU32::new(0);
    static PRESSED_AT: Mutex<Option<Instant>> = Mutex::new(None);
    static PRESS: AtomicU32 = AtomicU32::new(0);

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

    fn island_visible(handle: &AppHandle) -> bool {
        handle
            .get_webview_window(ISLAND)
            .and_then(|window| window.is_visible().ok())
            .unwrap_or(false)
    }

    fn dismiss_overlay() -> bool {
        let Some(handle) = APP.lock().ok().and_then(|slot| slot.clone()) else {
            return false;
        };
        if !island_visible(&handle) {
            return false;
        }
        tell(&handle, "window.parloEscape && window.parloEscape()");
        true
    }

    #[cfg(windows)]
    fn remember_target(handle: &AppHandle) {
        let address = handle
            .try_state::<crate::Address>()
            .and_then(|address| address.0.lock().ok().and_then(|slot| slot.clone()));
        if let Some((port, token)) = address {
            let _ = crate::post(port, &token, "/dictate/begin");
        }
    }

    fn open(handle: &AppHandle, script: &'static str) {
        let handle = handle.clone();
        std::thread::spawn(move || {
            show(&handle);
            tell(&handle, script);
            remember_target(&handle);
        });
    }

    #[cfg(windows)]
    fn hold(handle: &AppHandle, press: u32) {
        let handle = handle.clone();
        std::thread::spawn(move || {
            let current = || PRESS.load(Ordering::Relaxed) == press;
            let still_down = || {
                current()
                    && PRESSED_AT
                        .lock()
                        .map(|slot| slot.is_some())
                        .unwrap_or(false)
            };
            if !current() {
                return;
            }
            show(&handle);
            tell(&handle, "window.parloArm && window.parloArm()");
            remember_target(&handle);
            let elapsed = PRESSED_AT
                .lock()
                .ok()
                .and_then(|slot| *slot)
                .map(|start| start.elapsed().as_millis())
                .unwrap_or(HOLD_MS);
            if elapsed < HOLD_MS {
                std::thread::sleep(Duration::from_millis((HOLD_MS - elapsed) as u64));
            }
            if !still_down() {
                if current() {
                    tell(&handle, "window.parloReady && window.parloReady()");
                }
                return;
            }
            if !current() || RECORDING.swap(true, Ordering::Relaxed) {
                return;
            }
            if !still_down() {
                RECORDING.store(false, Ordering::Relaxed);
                if current() {
                    tell(&handle, "window.parloReady && window.parloReady()");
                }
                return;
            }
            tell(&handle, "window.parloBegin && window.parloBegin()");
            if !still_down() {
                RECORDING.store(false, Ordering::Relaxed);
            }
        });
    }

    #[cfg(windows)]
    fn armed() {
        let press = PRESS.fetch_add(1, Ordering::Relaxed) + 1;
        if let Ok(mut slot) = PRESSED_AT.lock() {
            *slot = Some(Instant::now());
        }
        with_app(|handle| hold(handle, press));
    }

    fn hands_free() {
        with_app(|handle| {
            if island_visible(handle) {
                tell(handle, "window.parloHandsFree && window.parloHandsFree()");
            } else {
                open(handle, "window.parloHandsFree && window.parloHandsFree()");
            }
        });
    }

    fn command() {
        with_app(|handle| {
            if island_visible(handle) {
                tell(handle, "window.parloCommand && window.parloCommand()");
            } else {
                open(handle, "window.parloCommand && window.parloCommand()");
            }
        });
    }

    fn key_up() {
        if let Ok(mut slot) = PRESSED_AT.lock() {
            *slot = None;
        }
        with_app(|handle| {
            if RECORDING.swap(false, Ordering::Relaxed) {
                tell(handle, "window.parloStop && window.parloStop()");
            } else {
                tell(handle, "window.parloReady && window.parloReady()");
            }
        });
    }

    unsafe extern "system" fn callback(code: i32, w_param: usize, l_param: isize) -> isize {
        if code >= 0 && l_param != 0 {
            let entry = &*(l_param as *const Key);
            let pressed = w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN;
            let released = w_param == WM_KEYUP || w_param == WM_SYSKEYUP;
            if entry.vk == VK_ESCAPE {
                if pressed {
                    if ESCAPE_LATCH.load(Ordering::Relaxed) {
                        return 1;
                    }
                    if dismiss_overlay() {
                        ESCAPE_LATCH.store(true, Ordering::Relaxed);
                        return 1;
                    }
                } else if released && ESCAPE_LATCH.swap(false, Ordering::Relaxed) {
                    return 1;
                }
            }
            if released {
                if entry.vk == FREE_KEY.load(Ordering::Relaxed) {
                    FREE_LATCH.store(false, Ordering::Relaxed);
                }
                if entry.vk == CAPTIONS_KEY.load(Ordering::Relaxed) {
                    CAPTIONS_LATCH.store(false, Ordering::Relaxed);
                }
                if entry.vk == COMMAND_KEY.load(Ordering::Relaxed) {
                    COMMAND_LATCH.store(false, Ordering::Relaxed);
                }
                if entry.vk == COPY_HELD_KEY.load(Ordering::Relaxed)
                    && COPY_LATCH.swap(false, Ordering::Relaxed)
                {
                    COPY_HELD_KEY.store(0, Ordering::Relaxed);
                    return 1;
                }
            }
            if pressed
                && COPY_LATCH.load(Ordering::Relaxed)
                && entry.vk == COPY_HELD_KEY.load(Ordering::Relaxed)
            {
                return 1;
            }
            if pressed || released {
                let held = modifiers_held();
                let copy = COPY_SHORTCUT.load(Ordering::Relaxed);
                let matches = |key: u32, modifiers: u32| {
                    parlo_core::hotkey::matches(entry.vk, held, key, modifiers)
                };
                if released
                    && HELD.load(Ordering::Relaxed)
                    && parlo_core::hotkey::ends_hold(
                        entry.vk,
                        KEY.load(Ordering::Relaxed),
                        MODS.load(Ordering::Relaxed),
                    )
                {
                    HELD.store(false, Ordering::Relaxed);
                    key_up();
                } else if matches(KEY.load(Ordering::Relaxed), MODS.load(Ordering::Relaxed)) {
                    if pressed && !HELD.swap(true, Ordering::Relaxed) {
                        armed();
                    } else if released && HELD.swap(false, Ordering::Relaxed) {
                        key_up();
                    }
                } else if matches(
                    FREE_KEY.load(Ordering::Relaxed),
                    FREE_MODS.load(Ordering::Relaxed),
                ) {
                    if pressed && !FREE_LATCH.swap(true, Ordering::Relaxed) {
                        hands_free();
                    } else if released {
                        FREE_LATCH.store(false, Ordering::Relaxed);
                    }
                } else if matches(
                    CAPTIONS_KEY.load(Ordering::Relaxed),
                    CAPTIONS_MODS.load(Ordering::Relaxed),
                ) {
                    if pressed && !CAPTIONS_LATCH.swap(true, Ordering::Relaxed) {
                        with_app(crate::captions_toggle);
                    } else if released {
                        CAPTIONS_LATCH.store(false, Ordering::Relaxed);
                    }
                } else if matches(
                    COMMAND_KEY.load(Ordering::Relaxed),
                    COMMAND_MODS.load(Ordering::Relaxed),
                ) {
                    if pressed && !COMMAND_LATCH.swap(true, Ordering::Relaxed) {
                        command();
                    } else if released {
                        COMMAND_LATCH.store(false, Ordering::Relaxed);
                    }
                } else if matches(copy & 0xffff, copy >> 16)
                    && pressed
                    && !COPY_LATCH.swap(true, Ordering::Relaxed)
                {
                    COPY_HELD_KEY.store(entry.vk, Ordering::Relaxed);
                    with_app(|handle| {
                        let handle = handle.clone();
                        std::thread::spawn(move || copy_recovery(&handle));
                    });
                    return 1;
                }
            }
        }
        CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param)
    }

    let optional = |label: &str, text: &str, fallback: &str| match Shortcut::parse(text) {
        Ok(shortcut) => shortcut,
        Err(error) => {
            crate::note(&format!(
                "the {label} shortcut {text} is unusable: {error}, so {fallback} is used"
            ));
            Shortcut::parse(fallback).unwrap_or(Shortcut {
                modifiers: parlo_core::shortcut::CONTROL | parlo_core::shortcut::ALT,
                key: 0x44,
            })
        }
    };

    let stored = settings::load();
    let shortcut = match Shortcut::parse(&stored.shortcut) {
        Ok(shortcut) => shortcut,
        Err(error) => {
            crate::note(&format!(
                "the dictation shortcut {} is unusable: {error}",
                stored.shortcut
            ));
            return;
        }
    };
    let free = optional(
        "hands free",
        &stored.hands_free,
        parlo_core::shortcut::HANDS_FREE,
    );
    let captions = optional("captions", &stored.captions, parlo_core::shortcut::CAPTIONS);
    let command = optional("command", &stored.command, parlo_core::shortcut::COMMAND);
    let set_copy = |text: &str| {
        let parsed = Shortcut::parse(text).ok();
        COPY_SHORTCUT.store(
            parsed.map_or(0, |key| key.key | (key.modifiers << 16)),
            Ordering::Relaxed,
        );
    };
    set_copy(&stored.copy_shortcut);
    let unavailable_undo = stored.undo.clone();
    KEY.store(shortcut.key, Ordering::Relaxed);
    MODS.store(shortcut.modifiers, Ordering::Relaxed);
    FREE_KEY.store(free.key, Ordering::Relaxed);
    FREE_MODS.store(free.modifiers, Ordering::Relaxed);
    CAPTIONS_KEY.store(captions.key, Ordering::Relaxed);
    CAPTIONS_MODS.store(captions.modifiers, Ordering::Relaxed);
    COMMAND_KEY.store(command.key, Ordering::Relaxed);
    COMMAND_MODS.store(command.modifiers, Ordering::Relaxed);
    std::thread::spawn(move || {
        let mut seen = (
            stored.shortcut.clone(),
            stored.hands_free.clone(),
            stored.captions.clone(),
            stored.undo.clone(),
            stored.copy_shortcut.clone(),
            stored.command.clone(),
        );
        loop {
            std::thread::sleep(Duration::from_millis(750));
            let latest = settings::load();
            let now = (
                latest.shortcut.clone(),
                latest.hands_free.clone(),
                latest.captions.clone(),
                latest.undo.clone(),
                latest.copy_shortcut.clone(),
                latest.command.clone(),
            );
            if now == seen {
                continue;
            }
            seen = now;
            set_copy(&latest.copy_shortcut);
            if let Some((dictation, free, captions, _undo, command)) = parlo_core::shortcut::resolve(
                &latest.shortcut,
                &latest.hands_free,
                &latest.captions,
                &latest.undo,
                &latest.command,
            ) {
                KEY.store(dictation.key, Ordering::Relaxed);
                MODS.store(dictation.modifiers, Ordering::Relaxed);
                FREE_KEY.store(free.key, Ordering::Relaxed);
                FREE_MODS.store(free.modifiers, Ordering::Relaxed);
                CAPTIONS_KEY.store(captions.key, Ordering::Relaxed);
                CAPTIONS_MODS.store(captions.modifiers, Ordering::Relaxed);
                COMMAND_KEY.store(command.key, Ordering::Relaxed);
                COMMAND_MODS.store(command.modifiers, Ordering::Relaxed);
            }
            crate::note(&format!(
                "the shortcuts changed: {} dictates, {} stays listening, {} toggles captions, {} runs a command, {} is unavailable",
                latest.shortcut,
                free.text(),
                captions.text(),
                latest.command,
                latest.undo
            ));
        }
    });
    if let Ok(mut slot) = APP.lock() {
        *slot = Some(handle);
    }

    std::thread::spawn(move || unsafe {
        if SetWindowsHookExW(WH_KEYBOARD_LL, callback, std::ptr::null_mut(), 0).is_null() {
            crate::note("the keyboard hook did not install, so push to talk is unavailable");
            return;
        }
        crate::note(&format!(
            "shortcuts listening: {} opens the panel on a tap and records while held, {} stays listening, {} toggles captions, {} runs a command, {} is unavailable",
            shortcut.text(),
            free.text(),
            captions.text(),
            command.text(),
            unavailable_undo
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
    let stored = settings::load();
    crate::note(&format!(
        "the global shortcuts ({} to dictate, {} for hands free, {} for captions, {} to undo) are windows only for now",
        stored.shortcut, stored.hands_free, stored.captions, stored.undo
    ));
}

#[cfg(test)]
mod bridge_tests {
    #[test]
    fn shortcut_modifiers_match_exactly_and_releases_reset_latches() {
        let hook = include_str!("island.rs");
        assert!(hook.contains("parlo_core::hotkey::matches(entry.vk, held, key, modifiers)"));
        assert!(hook.contains("if entry.vk == FREE_KEY.load(Ordering::Relaxed)"));
        assert!(hook.contains("FREE_LATCH.store(false, Ordering::Relaxed)"));
    }

    #[test]
    fn the_hook_drives_the_overlay_through_its_bridge() {
        let hook = include_str!("island.rs");
        let page = include_str!("../../parlo-cli/web/island.html");
        for call in [
            "parloArm",
            "parloBegin",
            "parloStop",
            "parloReady",
            "parloEscape",
        ] {
            assert!(
                hook.contains(&format!("window.{call} && window.{call}()")),
                "the hook never fires {call}"
            );
            assert!(
                page.contains(&format!("window.{call} =")),
                "the overlay never exposes {call}"
            );
        }
    }

    #[test]
    fn the_island_is_revealed_without_activating() {
        let hook = include_str!("island.rs");
        assert!(
            hook.contains("parlo_core::focus::reveal(raw)"),
            "the island is shown through an activating call"
        );
        assert!(
            hook.contains("parlo_core::focus::no_activate(raw, true)"),
            "the island is not marked as non activating before it appears"
        );
        let start = hook.find("fn open(").expect("there is no open hook");
        let end = hook
            .find("fn hands_free()")
            .expect("there is no hands free hook");
        let body = &hook[start..end];
        let shown = body
            .find("show(&handle);")
            .expect("the open hook never shows the island");
        let remembered = body
            .find("remember_target(&handle);")
            .expect("the open hook never remembers the target");
        assert!(
            shown < remembered,
            "the target is captured before the island is revealed, which delays the island"
        );
    }

    #[test]
    fn the_hold_thread_arms_the_island_before_it_begins() {
        let hook = include_str!("island.rs");
        let start = hook.find("fn hold(").expect("there is no hold thread");
        let end = hook.find("fn armed()").expect("there is no armed handler");
        let body = &hook[start..end];
        let armed = body
            .find("window.parloArm && window.parloArm()")
            .expect("the hold thread never arms the island");
        let begin = body
            .find("window.parloBegin && window.parloBegin()")
            .expect("the hold thread never begins a dictation");
        assert!(
            armed < begin,
            "the begin signal overtakes the arm signal, so a slow first show drops the dictation"
        );
    }

    #[test]
    fn the_recording_key_starts_without_a_visible_wait() {
        let hook = include_str!("island.rs");
        let line = hook
            .lines()
            .find(|line| line.contains("const HOLD_MS"))
            .expect("the hold threshold moved");
        assert!(
            line.contains("250"),
            "a tap and a hold are told apart after too long: {line}"
        );
    }

    #[test]
    fn the_island_is_placed_before_it_is_shown() {
        let hook = include_str!("island.rs");
        let start = hook.find("pub fn show(").expect("there is no show");
        let end = hook.find("pub fn hide(").expect("there is no hide");
        let body = &hook[start..end];
        let placed = body
            .find("crate::place(&window);")
            .expect("show never places the island");
        let shown = body
            .find("window.show()")
            .expect("show never shows the island");
        assert!(placed < shown, "the island is shown before it is placed");
    }

    #[test]
    fn a_visible_island_is_not_repositioned() {
        let hook = include_str!("island.rs");
        let start = hook.find("pub fn show(").expect("there is no show");
        let end = hook.find("pub fn hide(").expect("there is no hide");
        let body = &hook[start..end];
        assert!(body.contains("if !window.is_visible().unwrap_or(false)"));
    }

    #[test]
    fn escape_dismisses_the_overlay_from_the_keyboard_hook() {
        let hook = include_str!("island.rs");
        let start = hook.find("VK_ESCAPE").expect("escape has no key code");
        let call = hook
            .find("window.parloEscape && window.parloEscape()")
            .expect("the hook never dismisses the overlay");
        assert!(
            start < call,
            "escape is read after the hook has already returned"
        );
    }

    #[test]
    fn the_island_can_be_dragged_from_the_page_the_engine_serves() {
        let page = include_str!("../../parlo-cli/web/island.html");
        assert!(
            page.contains("startDragging") && page.contains("pointerdown"),
            "the island has no way to be moved"
        );
    }
}
