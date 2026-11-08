use super::*;

fn write_export(folder: &Path, skip: Option<&str>) {
    fs::create_dir_all(folder).unwrap();
    for name in PAIR_FILES {
        if Some(*name) == skip {
            continue;
        }
        fs::write(folder.join(*name), b"weights").unwrap();
    }
}

fn scratch(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let root = std::env::temp_dir().join(format!("parlo-pair-{name}-{stamp}"));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn a_complete_export_is_installed() {
    let root = scratch("ok");
    let folder = root.join("export");
    write_export(&folder, None);
    let answer = install_pair_in(&root, folder.to_str().unwrap(), "pt", "de").unwrap();
    assert!(answer.contains("pt to de"));
    assert!(root
        .join("mt")
        .join("pt-de-onnx")
        .join("encoder_model.onnx")
        .is_file());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_missing_file_is_named() {
    let root = scratch("missing");
    let folder = root.join("export");
    write_export(&folder, Some("vocab.json"));
    let error = install_pair_in(&root, folder.to_str().unwrap(), "pt", "de").unwrap_err();
    assert!(error.contains("vocab.json"));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_folder_that_is_not_there_is_refused() {
    let root = scratch("absent");
    let absent = root.join("nope");
    let error = install_pair_in(&root, absent.to_str().unwrap(), "pt", "de").unwrap_err();
    assert!(error.contains("not a folder"));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn an_existing_pair_is_not_overwritten() {
    let root = scratch("exists");
    let folder = root.join("export");
    write_export(&folder, None);
    install_pair_in(&root, folder.to_str().unwrap(), "pt", "de").unwrap();
    let error = install_pair_in(&root, folder.to_str().unwrap(), "pt", "de").unwrap_err();
    assert!(error.contains("already holds"));
    fs::remove_dir_all(&root).unwrap();
}
