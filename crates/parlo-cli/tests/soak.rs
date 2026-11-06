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
fn a_soak_reports_its_samples_and_draws_them() {
    let folder = std::env::temp_dir().join(format!("parlo-soak-files-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    let report = folder.join("soak.json");
    let chart = folder.join("soak.svg");
    let output = binary()
        .args(["soak", "--seconds", "4", "--interval", "1", "--report"])
        .arg(&report)
        .arg("--chart")
        .arg(&chart)
        .env("PARLO_DATA_DIR", folder.join("data"))
        .output()
        .expect("parlo soak runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}{stderr}");
    let body = std::fs::read_to_string(&report).expect("a report");
    assert!(body.contains("\"interval\":1"), "{body}");
    assert!(body.contains("\"memory_bytes\":["), "{body}");
    let svg = std::fs::read_to_string(&chart).expect("a chart");
    assert!(svg.starts_with("<svg"), "{svg}");
    assert!(svg.contains("samples every 1 s"), "{svg}");
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn a_report_without_a_path_is_refused() {
    let output = binary()
        .args(["soak", "--report"])
        .output()
        .expect("parlo soak runs");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--report needs a path"), "{stderr}");
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
