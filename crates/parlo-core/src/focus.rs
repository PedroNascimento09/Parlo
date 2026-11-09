#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    const SW_SHOWNOACTIVATE: i32 = 4;
    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOACTIVATE: u32 = 0x0010;
    const HWND_TOPMOST: isize = -1;

    extern "system" {
        fn ShowWindow(window: *mut c_void, command: i32) -> i32;
        fn SetWindowPos(
            window: *mut c_void,
            insert: *mut c_void,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            flags: u32,
        ) -> i32;
        fn GetForegroundWindow() -> *mut c_void;
        fn GetWindowThreadProcessId(window: *mut c_void, process: *mut u32) -> u32;
        fn GetCurrentProcessId() -> u32;
        fn SetForegroundWindow(window: *mut c_void) -> i32;
        fn IsWindow(window: *mut c_void) -> i32;
        fn AttachThreadInput(mine: u32, theirs: u32, attach: i32) -> i32;
        fn GetCurrentThreadId() -> u32;
        fn GetWindowLongPtrW(window: *mut c_void, index: i32) -> isize;
        fn SetWindowLongPtrW(window: *mut c_void, index: i32, value: isize) -> isize;
    }

    pub fn foreground() -> Option<usize> {
        let window = unsafe { GetForegroundWindow() };
        if window.is_null() {
            return None;
        }
        let mut process = 0u32;
        unsafe {
            GetWindowThreadProcessId(window, &mut process);
        }
        if process == unsafe { GetCurrentProcessId() } {
            None
        } else {
            Some(window as usize)
        }
    }

    pub fn reveal(window: usize) -> bool {
        if window == 0 || unsafe { IsWindow(window as *mut c_void) } == 0 {
            return false;
        }
        let raw = window as *mut c_void;
        unsafe {
            ShowWindow(raw, SW_SHOWNOACTIVATE);
            SetWindowPos(
                raw,
                HWND_TOPMOST as *mut c_void,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
        true
    }

    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_NOACTIVATE: isize = 0x0800_0000;

    pub fn no_activate(window: usize, on: bool) {
        if window == 0 || unsafe { IsWindow(window as *mut c_void) } == 0 {
            return;
        }
        let raw = window as *mut c_void;
        unsafe {
            let style = GetWindowLongPtrW(raw, GWL_EXSTYLE);
            let wanted = if on {
                style | WS_EX_NOACTIVATE
            } else {
                style & !WS_EX_NOACTIVATE
            };
            SetWindowLongPtrW(raw, GWL_EXSTYLE, wanted);
        }
    }

    pub fn give_back(field: usize) {
        if field == 0 || unsafe { IsWindow(field as *mut c_void) } == 0 {
            return;
        }
        let current = unsafe { GetForegroundWindow() };
        if current.is_null() {
            return;
        }
        let mut process = 0u32;
        unsafe {
            GetWindowThreadProcessId(current, &mut process);
        }
        if process != unsafe { GetCurrentProcessId() } {
            return;
        }
        unsafe {
            let mine = GetCurrentThreadId();
            let current_thread = GetWindowThreadProcessId(current, std::ptr::null_mut());
            let field_thread = GetWindowThreadProcessId(field as *mut c_void, std::ptr::null_mut());
            AttachThreadInput(mine, current_thread, 1);
            AttachThreadInput(mine, field_thread, 1);
            SetForegroundWindow(field as *mut c_void);
            AttachThreadInput(mine, field_thread, 0);
            AttachThreadInput(mine, current_thread, 0);
        }
    }
}

#[cfg(windows)]
pub use win::{foreground, give_back, no_activate, reveal};

#[cfg(not(windows))]
pub fn foreground() -> Option<usize> {
    None
}

#[cfg(not(windows))]
pub fn give_back(_field: usize) {}

#[cfg(not(windows))]
pub fn no_activate(_window: usize, _on: bool) {}

#[cfg(not(windows))]
pub fn reveal(_window: usize) -> bool {
    false
}
