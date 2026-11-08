use super::*;

fn folder(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let path = std::env::temp_dir().join(format!("parlo-adopt-{name}-{stamp}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn pair_of(here: &Path, there: &Path, left: &[u8], right: &[u8]) {
    fs::create_dir_all(here).unwrap();
    fs::create_dir_all(there).unwrap();
    fs::write(here.join("model.bin"), left).unwrap();
    fs::write(there.join("model.bin"), right).unwrap();
}

#[test]
fn a_copy_with_the_same_bytes_is_the_same_tree() {
    let root = folder("same");
    pair_of(
        &root.join("here"),
        &root.join("there"),
        b"weights",
        b"weights",
    );
    assert!(same_tree(&root.join("here"), &root.join("there")));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_same_size_with_different_bytes_is_not_adopted() {
    let root = folder("different");
    pair_of(
        &root.join("here"),
        &root.join("there"),
        b"weights",
        b"weightz",
    );
    assert!(!same_tree(&root.join("here"), &root.join("there")));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn an_empty_file_never_counts_as_adopted() {
    let root = folder("empty");
    pair_of(&root.join("here"), &root.join("there"), b"", b"");
    assert!(!same_tree(&root.join("here"), &root.join("there")));
    fs::remove_dir_all(&root).unwrap();
}
