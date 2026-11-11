pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_WIN: u32 = 0x0008;

pub fn modifier_bit(key: u32) -> u32 {
    match key {
        0x10 | 0xA0 | 0xA1 => MOD_SHIFT,
        0x11 | 0xA2 | 0xA3 => MOD_CONTROL,
        0x12 | 0xA4 | 0xA5 => MOD_ALT,
        0x5B | 0x5C => MOD_WIN,
        _ => 0,
    }
}

pub fn matches(key: u32, held: u32, trigger: u32, modifiers: u32) -> bool {
    key == trigger && held == modifiers
}

pub fn ends_hold(key: u32, trigger: u32, modifiers: u32) -> bool {
    key == trigger || modifier_bit(key) & modifiers != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extra_modifiers_select_the_correct_shortcut() {
        let held = MOD_CONTROL | MOD_ALT | MOD_SHIFT;
        assert!(!matches(0x44, held, 0x44, MOD_CONTROL | MOD_ALT));
        assert!(matches(0x44, held, 0x44, held));
        assert!(!matches(0x43, held, 0x44, held));
    }

    #[test]
    fn lifting_the_shortcut_key_ends_the_dictation() {
        assert!(ends_hold(0x44, 0x44, MOD_CONTROL | MOD_ALT));
    }

    #[test]
    fn lifting_any_modifier_of_the_shortcut_ends_the_dictation() {
        assert!(ends_hold(0xA2, 0x44, MOD_CONTROL | MOD_ALT));
        assert!(ends_hold(0x11, 0x44, MOD_CONTROL | MOD_ALT));
        assert!(ends_hold(0x10, 0x44, MOD_SHIFT));
        assert!(ends_hold(0x5B, 0x44, MOD_WIN));
    }

    #[test]
    fn an_unrelated_key_leaves_the_dictation_running() {
        assert!(!ends_hold(0x41, 0x44, MOD_CONTROL));
        assert!(!ends_hold(0x10, 0x44, MOD_CONTROL));
        assert!(!ends_hold(0xA2, 0x44, 0));
    }
}
