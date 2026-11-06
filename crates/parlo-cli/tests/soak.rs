use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_parlo"))
}

#[test]
fn a_short_soak_reports_memory_and_sessions() {
    let data = std::env::temp_dir().join(format!("parlo-soak-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    let output = binary()
        .args(["soak", "--seconds", "4", "--interval", "1"])
        .env("PARLO_DATA_DIR", &data)
        .output()
        .expect("parlo soak runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}{stderr}");
    assert!(stdout.contains("4 s asked"), "{stdout}");
    assert!(stdout.contains(" MB first, "), "{stdout}");
    assert!(stdout.contains(" samples every 1 s"), "{stdout}");
    assert!(
        stdout.contains("1 recorded, 0 crashed (100.0% crash free)"),
        "{stdout}"
    );
    let _ = std::fs::remove_dir_all(&data);
}

#[test]
fn the_soak_refuses_a_run_without_time() {
    let output = binary()
        .args(["soak", "--seconds", "0"])
        .output()
        .expect("parlo soak runs");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--seconds must be at least 1"), "{stderr}");
}
