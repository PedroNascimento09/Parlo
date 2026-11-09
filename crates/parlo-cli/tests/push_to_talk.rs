#[cfg(windows)]
#[test]
fn the_push_to_talk_path_lands_the_text_in_the_field() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_push_to_talk"))
        .output()
        .expect("the push to talk probe runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the probe failed:
{stdout}
{stderr}"
    );
    for needle in [
        "field captured:",
        "island visible without focus theft:",
        "hold ended on release:",
        "text inserted into the field: Bom dia.",
        "target remembered:",
    ] {
        assert!(
            stdout.contains(needle),
            "missing {needle:?} in:
{stdout}"
        );
    }
    assert!(
        !stdout.contains("does not know which field"),
        "the probe reported the missing field: {stdout}"
    );
}

#[cfg(not(windows))]
#[test]
fn the_push_to_talk_probe_reports_that_it_needs_windows() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_push_to_talk"))
        .output()
        .expect("the push to talk probe runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{stdout}");
    assert!(stdout.contains("needs windows"), "{stdout}");
}
