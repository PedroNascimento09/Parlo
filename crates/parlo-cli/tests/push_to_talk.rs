#[cfg(windows)]
#[test]
fn the_release_app_shows_the_island_and_lands_the_text() {
    if std::env::var("PARLO_REAL_APP_PROBE").as_deref() != Ok("1") {
        println!("skipped: set PARLO_REAL_APP_PROBE=1 in a desktop session");
        return;
    }
    let app = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/release/parlo-app.exe"
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_push_to_talk"))
        .env("PARLO_APP_EXE", app)
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
        "island visible and painted:",
        "recording ended on release:",
        "text inserted into the field: Bom dia.",
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
