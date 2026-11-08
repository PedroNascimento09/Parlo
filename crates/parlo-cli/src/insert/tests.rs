use super::*;

#[cfg(windows)]
#[test]
fn the_keyboard_input_matches_the_system_size() {
    assert_eq!(std::mem::size_of::<win::Input>(), 40);
}

#[cfg(windows)]
#[test]
fn erasing_refuses_a_window_that_is_not_in_front() {
    let error = erase(usize::MAX, 3).unwrap_err();
    assert!(error.contains("not in front"), "{error}");
}

#[cfg(windows)]
#[test]
fn a_letter_becomes_one_sixteen_bit_unit() {
    let encoded: Vec<u16> = "ã".encode_utf16().collect();
    assert_eq!(encoded, vec![0x00E3]);
}

#[cfg(windows)]
#[test]
fn one_press_and_one_release_are_queued_per_unit() {
    let pressed = win::Input::unicode(0x00E3, false);
    let released = win::Input::unicode(0x00E3, true);
    assert_eq!(pressed.kind, 1);
    assert_ne!(unsafe { pressed.union.keyboard.flags }, unsafe {
        released.union.keyboard.flags
    });
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
#[test]
fn other_platforms_say_what_is_missing() {
    let error = type_text(0, "olá").unwrap_err();
    assert!(error.contains("no way to type"));
}

#[test]
fn an_applescript_keystroke_escapes_quotes_and_backslashes() {
    let script = applescript_for("Diz \"olá\" \\ adeus");
    assert!(script.starts_with("tell application \"System Events\""));
    assert!(script.contains("keystroke \"Diz \\\"olá\\\" \\\\ adeus\""));
}

#[cfg(windows)]
#[test]
fn a_paste_is_control_then_v() {
    let keys = [
        win::Input::key(0x11, false),
        win::Input::key(0x56, false),
        win::Input::key(0x56, true),
        win::Input::key(0x11, true),
    ];
    assert_eq!(unsafe { keys[0].union.keyboard.vk }, 0x11);
    assert_eq!(unsafe { keys[1].union.keyboard.vk }, 0x56);
    assert_eq!(unsafe { keys[3].union.keyboard.flags } & 0x0002, 0x0002);
}

#[cfg(windows)]
#[test]
fn a_backspace_is_one_key_down_and_one_up() {
    let pressed = win::Input::backspace(false);
    let released = win::Input::backspace(true);
    assert_eq!(unsafe { pressed.union.keyboard.vk }, 0x08);
    assert_eq!(unsafe { released.union.keyboard.flags } & 0x0002, 0x0002);
}

#[test]
fn taking_back_text_repeats_the_delete_key() {
    let script = erase_script(3);
    assert!(script.contains("repeat 3 times"));
    assert!(script.contains("key code 51"));
}

#[test]
fn the_common_prefix_keeps_the_words_that_did_not_change() {
    assert_eq!(common_prefix("bom dia", "bom dia tudo"), 7);
    assert_eq!(common_prefix("bom dia", "boa tarde"), 2);
    assert_eq!(common_prefix("", "bom dia"), 0);
}

#[test]
fn a_newline_becomes_a_return_key() {
    let script = applescript_for("primeira\nsegunda");
    assert_eq!(script.matches("keystroke").count(), 2);
    assert!(script.contains("key code 36"));
}
