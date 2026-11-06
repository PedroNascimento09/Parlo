use std::fs;
use std::path::{Path, PathBuf};

const EVIDENCE: &str = "docs/benchmarks/machine-2026-11-06.json";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_stored_per_engine_run_names_what_it_measured() {
    let path = root().join(EVIDENCE);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let document: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|error| panic!("{EVIDENCE}: {error}"));
    assert_eq!(document["schema"], 1, "{EVIDENCE}: schema");
    let rows = document["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("{EVIDENCE}: no rows"));
    assert_eq!(rows.len(), 8, "{EVIDENCE}: one row per engine and device");
    let mut measured = 0;
    for row in rows {
        let engine = row["engine"].as_str().unwrap_or_default();
        let device = row["device"].as_str().unwrap_or_default();
        assert!(
            !engine.is_empty() && !device.is_empty(),
            "{EVIDENCE}: a row has no engine or device"
        );
        let hardware = row["hardware"].as_str().unwrap_or_default();
        assert!(
            hardware.contains("cores"),
            "{engine} on {device}: hardware {hardware:?} names no core count"
        );
        let commit = row["commit"].as_str().unwrap_or_default();
        assert_eq!(
            commit.len(),
            7,
            "{engine} on {device}: commit {commit:?} is not a short hash"
        );
        assert!(
            commit
                .chars()
                .all(|character| character.is_ascii_hexdigit()),
            "{engine} on {device}: commit {commit:?} is not hexadecimal"
        );
        let state = row["state"].as_str().unwrap_or_default();
        assert!(
            !state.is_empty(),
            "{engine} on {device}: no state explains the row"
        );
        match row["rtf"].as_f64() {
            Some(rtf) => {
                measured += 1;
                assert!(rtf > 0.0, "{engine} on {device}: rtf {rtf} is not positive");
                assert!(
                    row["seconds"].as_f64().unwrap_or(0.0) > 0.0,
                    "{engine} on {device}: no seconds beside a real time factor"
                );
                assert!(
                    row["frames"].as_u64().unwrap_or(0) > 0,
                    "{engine} on {device}: no frames beside a real time factor"
                );
                assert_eq!(
                    state, "measured on this machine",
                    "{engine} on {device}: a measured row must say so"
                );
            }
            None => {
                assert!(
                    row["seconds"].is_null(),
                    "{engine} on {device}: seconds without a run"
                );
                assert_ne!(
                    state, "measured on this machine",
                    "{engine} on {device}: claims a run it did not make"
                );
            }
        }
    }
    assert!(
        measured >= 1,
        "{EVIDENCE}: no row was measured on this machine"
    );
    assert!(
        rows.iter()
            .any(|row| row["engine"] == "echo" && row["rtf"].as_f64().is_some()),
        "{EVIDENCE}: the echo engine has no measured row"
    );
    let document = fs::read_to_string(root().join("docs/BENCHMARKS.md")).unwrap();
    assert!(
        document.contains(EVIDENCE),
        "docs/BENCHMARKS.md: the stored run is not linked"
    );
}
