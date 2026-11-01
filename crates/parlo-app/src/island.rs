use parlo_core::settings;
use parlo_core::shortcut::{Shortcut, NO_REPEAT};
use tauri::{AppHandle, Manager};

pub fn show(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window("island") else {
        return;
    };
    let _ = window.show();
}

pub fn hide(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window("island") else {
        return;
    };
    let _ = window.hide();
}

pub fn toggle(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window("island") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        hide(handle);
    } else {
        show(handle);
    }
}

#[cfg(windows)]
pub fn listen(handle: AppHandle) {
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
    std::thread::spawn(move || unsafe {
        use std::ffi::c_void;

        const WM_HOTKEY: u32 = 0x0312;
        const ID: i32 = 0x5041;

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
            fn RegisterHotKey(window: *mut c_void, id: i32, modifiers: u32, key: u32) -> i32;
            fn GetMessageW(
                message: *mut Message,
                window: *mut c_void,
                first: u32,
                last: u32,
            ) -> i32;
            fn TranslateMessage(message: *const Message) -> i32;
            fn DispatchMessageW(message: *const Message) -> isize;
        }

        let modifiers = shortcut.modifiers | NO_REPEAT;
        if RegisterHotKey(std::ptr::null_mut(), ID, modifiers, shortcut.key) == 0 {
            crate::note(&format!(
                "the dictation shortcut {} is taken by another program",
                shortcut.text()
            ));
            return;
        }
        crate::note(&format!("dictation shortcut ready on {}", shortcut.text()));

        let mut message: Message = std::mem::zeroed();
        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
            if message.id == WM_HOTKEY && message.w_param as i32 == ID {
                toggle(&handle);
            }
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
