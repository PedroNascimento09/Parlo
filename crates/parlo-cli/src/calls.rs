const APPS: &[(&str, &str)] = &[
    ("discord", "Discord"),
    ("zoom", "Zoom"),
    ("microsoft teams", "Teams"),
    ("google meet", "Google Meet"),
    ("webex", "Webex"),
    ("skype", "Skype"),
    ("whereby", "Whereby"),
    ("jitsi", "Jitsi"),
    ("slack call", "Slack"),
];

pub fn app_for(title: &str) -> Option<&'static str> {
    let title = title.to_lowercase();
    APPS.iter()
        .find(|(needle, _)| title.contains(needle))
        .map(|(_, app)| *app)
}

#[cfg(windows)]
pub fn detected() -> Option<(&'static str, String)> {
    let titles = windows::visible_titles();
    titles
        .into_iter()
        .find_map(|title| app_for(&title).map(|app| (app, title)))
}

#[cfg(not(windows))]
pub fn detected() -> Option<(&'static str, String)> {
    None
}

#[cfg(windows)]
mod windows {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    type Handle = isize;

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(callback: extern "system" fn(Handle, isize) -> i32, parameter: isize)
            -> i32;
        fn IsWindowVisible(window: Handle) -> i32;
        fn GetWindowTextLengthW(window: Handle) -> i32;
        fn GetWindowTextW(window: Handle, buffer: *mut u16, length: i32) -> i32;
    }

    extern "system" fn collect(window: Handle, parameter: isize) -> i32 {
        let titles = unsafe { &mut *(parameter as *mut Vec<String>) };
        if unsafe { IsWindowVisible(window) } == 0 {
            return 1;
        }
        let length = unsafe { GetWindowTextLengthW(window) };
        if length <= 0 {
            return 1;
        }
        let mut buffer = vec![0u16; length as usize + 1];
        let written = unsafe { GetWindowTextW(window, buffer.as_mut_ptr(), buffer.len() as i32) };
        if written > 0 {
            buffer.truncate(written as usize);
            let title = OsString::from_wide(&buffer).to_string_lossy().into_owned();
            if !title.trim().is_empty() {
                titles.push(title);
            }
        }
        1
    }

    pub fn visible_titles() -> Vec<String> {
        let mut titles: Vec<String> = Vec::new();
        unsafe { EnumWindows(collect, &mut titles as *mut Vec<String> as isize) };
        titles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_call_window_names_its_application() {
        assert_eq!(app_for("Discord | friends"), Some("Discord"));
        assert_eq!(app_for("Zoom Meeting"), Some("Zoom"));
        assert_eq!(app_for("Microsoft Teams"), Some("Teams"));
        assert_eq!(app_for("Google Meet - Meeting"), Some("Google Meet"));
        assert_eq!(app_for("Slack call with Ana"), Some("Slack"));
    }

    #[test]
    fn an_ordinary_window_names_nothing() {
        assert_eq!(app_for("Parlo"), None);
        assert_eq!(app_for("main.rs - Visual Studio Code"), None);
    }

    #[test]
    fn the_match_ignores_case() {
        assert_eq!(app_for("DISCORD"), Some("Discord"));
        assert_eq!(app_for("zoom"), Some("Zoom"));
    }
}
