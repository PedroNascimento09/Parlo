#[test]
fn the_first_dictation_note_is_proved_in_a_dom() {
    let suite = concat!(env!("CARGO_MANIFEST_DIR"), "/web/first-load.test.mjs");
    let status = std::process::Command::new("node")
        .arg("--test")
        .arg(suite)
        .status()
        .expect("node runs the island note test");
    assert!(status.success(), "the island note test failed");
}
