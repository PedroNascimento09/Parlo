use std::process::Command;

fn parlo(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_parlo"))
        .args(arguments)
        .output()
        .expect("the binary runs")
}

#[test]
fn the_command_respells_a_sentence_before_it_is_spoken() {
    let output = parlo(&["pronounce", "--text", "Parlo reads JSON."]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "PAR-loh reads JAY-son."
    );
}

#[test]
fn the_command_names_the_forms_of_a_word() {
    let output = parlo(&["pronounce", "Piper"]);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{text}");
    assert!(text.contains("PY-per"), "{text}");
    assert!(text.contains("ˈpaɪpər"), "{text}");
    assert!(text.contains("P AY1 P ER0"), "{text}");
}

#[test]
fn an_unknown_word_is_refused() {
    let output = parlo(&["pronounce", "pardal"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("pardal is not in the dictionary"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
