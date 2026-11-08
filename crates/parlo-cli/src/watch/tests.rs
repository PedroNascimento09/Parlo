use super::*;

#[test]
fn supported_files_are_the_dubbing_formats() {
    assert!(supported(Path::new("clip.MP4")));
    assert!(supported(Path::new("voice.wav")));
    assert!(!supported(Path::new("notes.txt")));
    assert!(!supported(Path::new("clip")));
}

#[test]
fn batch_ids_are_stable_and_do_not_repeat_for_different_keys() {
    let first = FileKey {
        volume: 1,
        index: 2,
        size: 3,
    };
    let second = FileKey {
        volume: 1,
        index: 2,
        size: 4,
    };
    assert_eq!(batch_id("w1", &first), batch_id("w1", &first));
    assert_ne!(batch_id("w1", &first), batch_id("w1", &second));
}

#[test]
fn a_truncated_journal_keeps_every_complete_claim() {
    let root = std::env::temp_dir().join(format!("parlo-watch-test-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("journal.jsonl");
    let key = FileKey {
        volume: 1,
        index: 2,
        size: 3,
    };
    let claim = Claim {
        key: key.clone(),
        path: "C:/inbox/clip.mp4".to_string(),
        batch: "w-test".to_string(),
        at: 1,
    };
    append_claim_at(&path, &claim).unwrap();
    let mut file = OpenOptions::new().append(true).open(&path).unwrap();
    file.write_all(b"{\"key\":").unwrap();
    drop(file);
    let later = Claim {
        key: FileKey {
            volume: 4,
            index: 5,
            size: 6,
        },
        path: "C:/inbox/later.mp4".to_string(),
        batch: "w-later".to_string(),
        at: 2,
    };
    append_claim_at(&path, &later).unwrap();
    let claims = read_claims_at(&path).unwrap();
    assert!(claims.contains_key(&key));
    assert!(claims.contains_key(&later.key));
    fs::remove_dir_all(&root).ok();
}
