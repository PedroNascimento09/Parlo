use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let name = path.to_string_lossy();
            let test_file = name.ends_with("tests.rs")
                || name.contains("/tests/")
                || name.contains("\\tests\\");
            if !test_file {
                found.push(path);
            }
        }
    }
}

#[test]
fn production_code_does_not_unwrap() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    assert!(!files.is_empty(), "no crate sources were found");

    let mut offenders = Vec::new();
    for file in &files {
        if file.to_string_lossy().contains("dictation_latency.rs") {
            continue;
        }
        let text = std::fs::read_to_string(file).expect("a source file reads");
        let test_start = text.find("#[cfg(test)]").unwrap_or(text.len());
        for (index, line) in text[..test_start].lines().enumerate() {
            if line.contains(".unwrap()") {
                offenders.push(format!("{}:{}", file.display(), index + 1));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "production code must not unwrap, use ? or expect with a reason: {}",
        offenders.join(", ")
    );
}
