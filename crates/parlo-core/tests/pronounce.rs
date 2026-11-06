use parlo_core::pronounce::{entries, lookup, respell};

#[test]
fn every_entry_carries_a_respelling_ipa_and_cmu() {
    let entries = entries();
    assert!(
        entries.len() >= 10,
        "the dictionary is thin: {}",
        entries.len()
    );
    for entry in entries {
        assert!(!entry.word.trim().is_empty(), "an entry has no word");
        assert!(
            !entry.respell.trim().is_empty(),
            "{} has no respelling",
            entry.word
        );
        assert!(!entry.ipa.trim().is_empty(), "{} has no ipa", entry.word);
        assert!(!entry.cmu.trim().is_empty(), "{} has no cmu", entry.word);
        assert!(
            entry
                .cmu
                .chars()
                .any(|character| character.is_ascii_digit()),
            "{} has no cmu stress digit: {}",
            entry.word,
            entry.cmu
        );
        assert_eq!(
            entry.word.to_lowercase(),
            lookup(&entry.word)
                .map(|found| found.word.to_lowercase())
                .unwrap_or_default(),
            "{} cannot be found by its own word",
            entry.word
        );
    }
}

#[test]
fn a_known_word_is_replaced_before_it_is_spoken() {
    assert_eq!(respell("Parlo speaks"), "PAR-loh speaks");
    assert_eq!(respell("Piper reads JSON"), "PY-per reads JAY-son");
}

#[test]
fn a_word_inside_a_longer_word_is_left_alone() {
    assert_eq!(respell("Pipering"), "Pipering");
    assert_eq!(respell("APIs"), "APIs");
    assert_eq!(respell("parlophone"), "parlophone");
}

#[test]
fn a_digit_after_a_word_makes_it_another_word() {
    assert_eq!(respell("SQLite3"), "SQLite3");
    assert_eq!(respell("WAV2"), "WAV2");
}

#[test]
fn case_does_not_stop_a_match_but_survives_for_unknown_words() {
    assert_eq!(respell("piper"), "PY-per");
    assert_eq!(respell("PIPER"), "PY-per");
    assert_eq!(respell("PiPeR"), "PY-per");
    assert_eq!(respell("pArLoPhone"), "pArLoPhone");
}

#[test]
fn punctuation_and_spacing_survive_the_replacement() {
    assert_eq!(respell("\"JSON\", isn't it?"), "\"JAY-son\", isn't it?");
    assert_eq!(respell("GUI  and\nAPI."), "GOO-ee  and\nay-pee-EYE.");
    assert_eq!(respell(""), "");
    assert_eq!(respell("   "), "   ");
}

#[test]
fn an_unknown_word_is_spoken_as_written() {
    assert!(lookup("pardal").is_none());
    assert_eq!(
        respell("O pardal comeu o API."),
        "O pardal comeu o ay-pee-EYE."
    );
}
