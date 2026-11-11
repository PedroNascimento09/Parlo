use super::*;

#[cfg(windows)]
#[test]
fn the_keyboard_input_matches_the_system_size() {
    assert_eq!(std::mem::size_of::<win::Input>(), 40);
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

#[test]
fn replacement_without_a_matching_record_changes_nothing() {
    assert!(replace_recorded(0, "previous", "next").is_err());
}

#[test]
fn undo_with_nothing_typed_says_so() {
    assert!(undo_last().is_err());
}

#[test]
fn undo_over_a_cleanup_restores_the_words_before_it() {
    let cleaned = Record {
        target: 7,
        text: "Clean sentence.".to_string(),
        previous: Some("raw  words, uh sentence".to_string()),
    };
    let (gone, back, next) = undo_step(&cleaned);
    assert_eq!(gone, "Clean sentence.");
    assert_eq!(back.as_deref(), Some("raw  words, uh sentence"));
    let record = next.expect("undo over a cleanup keeps the cleaned text for redo");
    assert_eq!(record.text, "raw  words, uh sentence");
    let (gone, back, next) = undo_step(&record);
    assert_eq!(gone, "raw  words, uh sentence");
    assert_eq!(back.as_deref(), Some("Clean sentence."));
    assert_eq!(
        next.expect("a second toggle stays available").text,
        "Clean sentence."
    );
}

#[test]
fn undo_over_a_plain_insert_only_deletes() {
    let fresh = Record {
        target: 7,
        text: "Typed words.".to_string(),
        previous: None,
    };
    let (gone, back, next) = undo_step(&fresh);
    assert_eq!(gone, "Typed words.");
    assert!(back.is_none());
    assert!(next.is_none());
}

#[test]
fn a_newline_becomes_a_return_key() {
    let script = applescript_for("primeira\nsegunda");
    assert_eq!(script.matches("keystroke").count(), 2);
    assert!(script.contains("key code 36"));
}
