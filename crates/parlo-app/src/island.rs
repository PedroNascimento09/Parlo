use tauri::AppHandle;

#[cfg(windows)]
use tauri::Manager;

pub const SHORTCUT: &str = "Ctrl+Alt+D";

#[cfg(windows)]
pub fn toggle(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window("island") else {
        return;
    };
    match window.is_visible() {
        Ok(true) => {
            let _ = window.hide();
        }
        _ => {
            let _ = window.show();
        }
    }
}

#[cfg(windows)]
pub fn listen(handle: AppHandle) {
    std::thread::spawn(move || unsafe {
        use std::ffi::c_void;

        const WM_HOTKEY: u32 = 0x0312;
        const MOD_ALT: u32 = 0x0001;
        const MOD_CONTROL: u32 = 0x0002;
        const MOD_NOREPEAT: u32 = 0x4000;
        const VK_D: u32 = 0x44;
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

        let modifiers = MOD_CONTROL | MOD_ALT | MOD_NOREPEAT;
        if RegisterHotKey(std::ptr::null_mut(), ID, modifiers, VK_D) == 0 {
            crate::note("the dictation shortcut is taken by another program");
            return;
        }
        crate::note(&format!("dictation shortcut ready on {SHORTCUT}"));

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
    crate::note(&format!(
        "the global dictation shortcut ({SHORTCUT}) is windows only for now"
    ));
}
