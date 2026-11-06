use std::path::Path;
use std::process::Command;

const README: &str = include_str!("../../../README.md");
const START: &str = include_str!("../../../docs/START.md");

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_parlo"))
}

fn help_text() -> String {
    let output = binary().arg("--help").output().expect("parlo --help runs");
    assert!(output.status.success(), "parlo --help failed");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn documented_commands(text: &str) -> Vec<String> {
    let mut inside = false;
    let mut commands = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            inside = !inside;
            continue;
        }
        if !inside {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("parlo ") else {
            continue;
        };
        let line = rest.split('#').next().unwrap_or(rest).trim();
        if let Some(name) = line.split_whitespace().next() {
            if !name.starts_with('-') {
                commands.push(line.to_string());
            }
        }
    }
    commands
}

fn links(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find("](") {
        rest = &rest[index + 2..];
        let end = rest.find(')').unwrap_or(rest.len());
        found.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    found
}

#[test]
fn every_command_the_start_page_documents_is_a_command_the_binary_has() {
    let help = help_text();
    let listed: Vec<&str> = help.lines().map(str::trim_start).collect();
    for page in [START, README] {
        let commands = documented_commands(page);
        assert!(
            commands.len() >= 3,
            "the documentation names {} commands",
            commands.len()
        );
        for command in &commands {
            let name = command.split_whitespace().next().unwrap_or_default();
            assert!(
                listed
                    .iter()
                    .any(|line| *line == name || line.starts_with(&format!("{name} "))),
                "{command} is documented but the binary has no {name} command"
            );
            for flag in command
                .split_whitespace()
                .filter(|word| word.starts_with("--"))
            {
                let flag = flag.trim_end_matches(',');
                assert!(
                    help.contains(flag),
                    "{flag} is documented but no command takes it"
                );
            }
        }
    }
}

#[test]
fn the_documented_doctor_command_runs_on_the_real_binary() {
    let data = std::env::temp_dir().join(format!("parlo-quickstart-{}", std::process::id()));
    let output = binary()
        .arg("doctor")
        .env("PARLO_DATA_DIR", &data)
        .output()
        .expect("parlo doctor runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("latency budget"), "{stdout}");
    assert!(
        matches!(output.status.code(), Some(0) | Some(1)),
        "doctor exited with {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("no telemetry"), "{stdout}");
    let _ = std::fs::remove_dir_all(&data);
}

#[test]
fn the_doctor_reports_the_sessions_it_finds_on_disk() {
    let data = std::env::temp_dir().join(format!("parlo-sessions-doctor-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(
        data.join("sessions.jsonl"),
        concat!(
            "{\"event\":\"start\",\"id\":1,\"at\":1000}\n",
            "{\"event\":\"end\",\"id\":1,\"at\":1010}\n",
            "{\"event\":\"start\",\"id\":2,\"at\":2000}\n",
            "{\"event\":\"crash\",\"id\":2,\"at\":2005,\"message\":\"boom\",\"location\":\"src/x.rs:1:1\"}\n",
            "{\"event\":\"start\",\"id\":3,\"at\":3000}\n",
            "{\"event\":\"end\",\"id\":3,\"at\":3030}\n",
        ),
    )
    .unwrap();
    let output = binary()
        .arg("doctor")
        .env("PARLO_DATA_DIR", &data)
        .output()
        .expect("parlo doctor runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("3 recorded, 2 clean, 1 crashed (66.7% crash free)"),
        "{stdout}"
    );
    assert!(stdout.contains("last ran 30 s and ended clean"), "{stdout}");
    let _ = std::fs::remove_dir_all(&data);
}

#[test]
fn every_page_the_start_page_and_the_readme_link_to_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for page in ["docs/START.md", "docs/BENCHMARKS.md", "docs/PRIVACY.md"] {
        assert!(
            README.contains(&format!("]({page})")),
            "the README does not link {page}"
        );
        assert!(root.join(page).is_file(), "{page} does not exist");
    }
    for (text, base) in [(README, root.clone()), (START, root.join("docs"))] {
        for link in links(text) {
            let path = link.split('#').next().unwrap_or_default();
            if path.is_empty()
                || path.starts_with("http")
                || path.starts_with("mailto")
                || path.starts_with('/')
            {
                continue;
            }
            assert!(
                base.join(path).exists(),
                "{path} is linked from the docs but does not exist"
            );
        }
    }
}
