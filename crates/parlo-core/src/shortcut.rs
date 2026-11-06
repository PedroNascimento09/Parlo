use std::fmt;

pub const DEFAULT: &str = "Ctrl+Alt+D";
pub const HANDS_FREE: &str = "Ctrl+Alt+Shift+D";
pub const CAPTIONS: &str = "Ctrl+Alt+C";
pub const UNDO: &str = "Ctrl+Alt+Z";

pub const ALT: u32 = 0x0001;
pub const CONTROL: u32 = 0x0002;
pub const SHIFT: u32 = 0x0004;
pub const WIN: u32 = 0x0008;
pub const NO_REPEAT: u32 = 0x4000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: u32,
    pub key: u32,
}

impl Shortcut {
    pub fn parse(text: &str) -> Result<Shortcut, String> {
        let parts: Vec<&str> = text
            .split('+')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect();
        let (key, modifiers) = match parts.split_last() {
            Some((key, modifiers)) => (*key, modifiers),
            None => return Err("a shortcut needs at least a modifier and a key".to_string()),
        };

        let mut bits = 0;
        for modifier in modifiers {
            let bit = match modifier.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => CONTROL,
                "alt" => ALT,
                "shift" => SHIFT,
                "win" | "meta" | "super" => WIN,
                other => return Err(format!("unknown modifier {other}")),
            };
            if bits & bit != 0 {
                return Err(format!("{modifier} is listed twice"));
            }
            bits |= bit;
        }
        if bits == 0 {
            return Err("a global shortcut needs Ctrl, Alt, Shift or Win".to_string());
        }

        Ok(Shortcut {
            modifiers: bits,
            key: virtual_key(key)?,
        })
    }

    pub fn text(&self) -> String {
        let mut parts = Vec::new();
        for (bit, name) in [
            (CONTROL, "Ctrl"),
            (ALT, "Alt"),
            (SHIFT, "Shift"),
            (WIN, "Win"),
        ] {
            if self.modifiers & bit != 0 {
                parts.push(name.to_string());
            }
        }
        parts.push(key_name(self.key));
        parts.join("+")
    }
}

#[cfg(windows)]
pub fn taken_by_another_program(shortcut: &Shortcut) -> bool {
    use std::ffi::c_void;

    extern "system" {
        fn RegisterHotKey(window: *mut c_void, id: i32, modifiers: u32, key: u32) -> i32;
        fn UnregisterHotKey(window: *mut c_void, id: i32) -> i32;
    }

    const PROBE: i32 = 0x5041;
    let registered = unsafe {
        RegisterHotKey(
            std::ptr::null_mut(),
            PROBE,
            shortcut.modifiers,
            shortcut.key,
        )
    };
    if registered == 0 {
        return true;
    }
    unsafe {
        UnregisterHotKey(std::ptr::null_mut(), PROBE);
    }
    false
}

#[cfg(not(windows))]
pub fn taken_by_another_program(_shortcut: &Shortcut) -> bool {
    false
}

impl fmt::Display for Shortcut {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text())
    }
}

pub fn resolve(
    dictation: &str,
    hands_free: &str,
    captions: &str,
    undo: &str,
) -> Option<(Shortcut, Shortcut, Shortcut, Shortcut)> {
    let fallback = |text: &str, default: &str| {
        Shortcut::parse(text)
            .or_else(|_| Shortcut::parse(default))
            .expect("the documented shortcut parses")
    };
    Some((
        Shortcut::parse(dictation).ok()?,
        fallback(hands_free, HANDS_FREE),
        fallback(captions, CAPTIONS),
        fallback(undo, UNDO),
    ))
}

fn virtual_key(name: &str) -> Result<u32, String> {
    let upper = name.to_ascii_uppercase();
    if upper.len() == 1 {
        let letter = upper.as_bytes()[0];
        if letter.is_ascii_uppercase() {
            return Ok(letter as u32);
        }
        if letter.is_ascii_digit() {
            return Ok(letter as u32);
        }
    }
    if upper == "SPACE" {
        return Ok(0x20);
    }
    if let Some(number) = upper.strip_prefix('F') {
        if let Ok(index) = number.parse::<u32>() {
            if (1..=24).contains(&index) {
                return Ok(0x70 + index - 1);
            }
        }
    }
    Err(format!("{name} is not a key this shortcut can use"))
}

fn key_name(key: u32) -> String {
    if (0x30..=0x39).contains(&key) || (0x41..=0x5A).contains(&key) {
        return char::from_u32(key).unwrap_or('?').to_string();
    }
    if key == 0x20 {
        return "Space".to_string();
    }
    if (0x70..=0x87).contains(&key) {
        return format!("F{}", key - 0x70 + 1);
    }
    format!("0x{key:02X}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_documented_default() {
        let shortcut = Shortcut::parse(DEFAULT).unwrap();
        assert_eq!(shortcut.modifiers, CONTROL | ALT);
        assert_eq!(shortcut.key, 0x44);
        assert_eq!(shortcut.text(), "Ctrl+Alt+D");
    }

    #[test]
    fn accepts_lower_case_and_extra_spaces() {
        let shortcut = Shortcut::parse(" ctrl + alt + d ").unwrap();
        assert_eq!(shortcut, Shortcut::parse(DEFAULT).unwrap());
    }

    #[test]
    fn writes_modifiers_in_one_order_whatever_the_input_order() {
        assert_eq!(
            Shortcut::parse("Alt+Shift+Ctrl+C").unwrap().text(),
            "Ctrl+Alt+Shift+C"
        );
    }

    #[test]
    fn refuses_a_key_without_a_modifier() {
        assert!(Shortcut::parse("D").is_err());
        assert!(Shortcut::parse("").is_err());
    }

    #[test]
    fn refuses_unknown_modifiers_and_keys() {
        assert!(Shortcut::parse("Hyper+D").is_err());
        assert!(Shortcut::parse("Ctrl+Escape").is_err());
        assert!(Shortcut::parse("Ctrl+F25").is_err());
    }

    #[test]
    fn refuses_a_modifier_listed_twice() {
        assert!(Shortcut::parse("Ctrl+Control+D").is_err());
    }

    #[test]
    fn covers_digits_function_keys_and_space() {
        assert_eq!(Shortcut::parse("Ctrl+Shift+1").unwrap().key, 0x31);
        assert_eq!(Shortcut::parse("Ctrl+F12").unwrap().key, 0x7B);
        assert_eq!(Shortcut::parse("Alt+Space").unwrap().text(), "Alt+Space");
    }

    #[test]
    fn reads_the_documented_hands_free_and_captions_keys() {
        assert_eq!(
            Shortcut::parse(HANDS_FREE).unwrap().text(),
            "Ctrl+Alt+Shift+D"
        );
        assert_eq!(Shortcut::parse(CAPTIONS).unwrap().text(), "Ctrl+Alt+C");
    }

    #[test]
    fn formatting_round_trips_through_parsing() {
        for text in ["Ctrl+Alt+D", "Alt+Space", "Win+Shift+F5", "Ctrl+Shift+9"] {
            let shortcut = Shortcut::parse(text).unwrap();
            assert_eq!(Shortcut::parse(&shortcut.text()).unwrap(), shortcut);
        }
    }

    #[test]
    fn a_saved_change_reaches_the_keys_without_a_restart() {
        let before = resolve(DEFAULT, HANDS_FREE, CAPTIONS, UNDO).unwrap();
        let after = resolve("Ctrl+Shift+K", HANDS_FREE, CAPTIONS, UNDO).unwrap();
        assert_ne!(before.0, after.0);
        assert_eq!(after.0.key, 0x4B);
        assert_eq!(after.1, before.1);
    }

    #[test]
    fn an_unusable_dictation_keeps_the_keys_already_listening() {
        assert!(resolve("nonsense", HANDS_FREE, CAPTIONS, UNDO).is_none());
    }
}
