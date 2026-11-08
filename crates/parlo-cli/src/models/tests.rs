use super::*;

#[test]
fn every_offered_language_has_a_shipped_pack() {
    let table: serde_json::Value =
        serde_json::from_str(include_str!("../../web/languages.json")).expect("a language table");
    for pair in table["pairs"].as_array().expect("language pairs") {
        let from = pair[0].as_str().expect("a source language");
        let to = pair[1].as_str().expect("a target language");
        let id = format!("mt-{from}-{to}");
        let spec = crate::registry::model(&id).unwrap_or_else(|| panic!("{id} is not shipped"));
        match spec.source {
            crate::registry::Source::Translation(source, target) => {
                assert_eq!((source, target), (from, to), "{id} points at another pair")
            }
            _ => panic!("{id} is not a translation pack"),
        }
        let paths = paths_for(&id).expect("translation paths");
        assert_eq!(
            paths[0],
            models_root().join("mt").join(format!("{from}-{to}")),
            "{id} resolves to another folder"
        );
    }
}
use super::*;

fn held(units: &[&str]) -> Vec<UnitState> {
    units
        .iter()
        .map(|unit| UnitState {
            unit: (*unit).to_string(),
            loaded: true,
            idle_seconds: Some(45),
            policy: 30,
            policy_origin: "global".to_string(),
        })
        .collect()
}

#[test]
fn dictation_needs_the_recogniser_the_engine_will_use() {
    let models = vec![
        Model {
            id: "whisper",
            name: "Whisper base".into(),
            kind: "recogniser",
            feature: "Dictation",
            installed: true,
            bytes: 145,
        },
        Model {
            id: "parakeet",
            name: "Parakeet".into(),
            kind: "recogniser",
            feature: "Dictation",
            installed: false,
            bytes: 0,
        },
    ];
    assert!(!dictation_ready_for(&models, "parakeet"));
    assert!(dictation_ready_for(&models, "whisper"));
    assert!(!ready(&models));
}

#[test]
fn whisper_is_installed_only_when_the_weights_are_there() {
    let root = std::env::temp_dir().join(format!("parlo-whisper-{}", std::process::id()));
    let cache = root.join("hub");
    let folder = cache
        .join("models--Systran--faster-whisper-base")
        .join("snapshots")
        .join("abc");
    fs::create_dir_all(&folder).unwrap();
    assert_eq!(whisper_installed(&cache), None);
    fs::write(folder.join("model.bin"), vec![0u8; 2048]).unwrap();
    assert_eq!(whisper_installed(&cache), Some(2048));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn every_model_maps_to_the_unit_the_engine_reports() {
    assert_eq!(unit_of("parakeet"), Some("recognition"));
    assert_eq!(unit_of("voice-en"), Some("voice"));
    assert_eq!(unit_of("voice-de"), Some("voice"));
    assert_eq!(unit_of("voice-fr"), Some("voice"));
    assert_eq!(unit_of("mt-en-pt"), Some("translation"));
    assert_eq!(unit_of("nonsense"), None);
}

#[test]
fn a_previous_copy_lives_beside_the_model() {
    let path = std::path::Path::new("C:/models/parakeet");
    assert_eq!(
        previous_of(path),
        std::path::Path::new("C:/models/parakeet.previous")
    );
}

#[test]
fn a_folder_is_adoptable_only_when_every_file_matches() {
    let root = std::env::temp_dir().join(format!("parlo-adopt-{}", std::process::id()));
    let left = root.join("left");
    let right = root.join("right");
    fs::create_dir_all(left.join("inner")).unwrap();
    fs::create_dir_all(right.join("inner")).unwrap();
    fs::write(left.join("inner/model.bin"), vec![1u8; 64]).unwrap();
    assert!(!same_tree(&left, &right));
    fs::write(right.join("inner/model.bin"), vec![1u8; 64]).unwrap();
    assert!(same_tree(&left, &right));
    fs::write(right.join("inner/model.bin"), vec![2u8; 64]).unwrap();
    assert!(!same_tree(&left, &right));
    fs::write(right.join("inner/model.bin"), vec![1u8; 32]).unwrap();
    assert!(!same_tree(&left, &right));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_relative_folder_is_refused_before_anything_moves() {
    let error = relocate("models-here").unwrap_err();
    assert!(error.contains("full path"), "{error}");
}

#[test]
fn both_translation_pairs_are_listed_and_removable() {
    let ids: Vec<&str> = status().iter().map(|model| model.id).collect();
    assert!(ids.contains(&"mt-pt-en"));
    assert!(ids.contains(&"mt-en-pt"));
    assert!(paths_for("mt-pt-en").is_some());
    assert!(paths_for("mt-en-pt").is_some());
}

#[test]
fn cancelling_without_a_download_says_so() {
    assert!(cancel().unwrap_err().contains("no download"));
}

#[test]
fn every_model_says_why_it_is_the_one_offered() {
    let models = status();
    let parsed: serde_json::Value =
        serde_json::from_str(&as_json(&models, &Lifecycle::default(), "pt-en", false)).unwrap();
    let rows = parsed["models"].as_array().unwrap();
    assert!(!rows.is_empty());
    for row in rows {
        let reason = row["reason"].as_str().unwrap();
        assert!(reason.contains("this machine has"), "{reason}");
    }
    assert!(parsed["machine"]["cores"].as_u64().unwrap() >= 1);
}

#[test]
fn installed_models_are_never_downloading() {
    let models = vec![Model {
        id: "voice-en",
        name: "voice".to_string(),
        kind: "voice",
        feature: "Read aloud",
        installed: true,
        bytes: 1024,
    }];
    assert!(!downloading(&models));
    let body = as_json(&models, &Lifecycle::default(), "pt-en", false);
    assert!(body.contains("\"downloading\":false"));
}

#[test]
fn an_unknown_model_is_refused_before_any_file_is_touched() {
    assert!(remove("nonsense").is_err());
}

#[test]
fn every_removable_model_names_the_switch_it_turns_off() {
    assert_eq!(feature_of("voice-pt"), Some("voice"));
    assert_eq!(feature_of("voice-en"), Some("voice"));
    assert_eq!(feature_of("mt-pt-en"), Some("translation"));
    assert_eq!(feature_of("mt-en-pt"), Some("translation"));
    assert_eq!(feature_of("parakeet"), None);
}

#[test]
fn every_row_carries_its_own_id_and_feature() {
    let models = status();
    assert!(models.iter().all(|model| !model.id.is_empty()));
    assert!(models.iter().any(|model| model.feature == "Dictation"));
    let body = as_json(&models, &Lifecycle::default(), "pt-en", false);
    assert!(body.contains("\"dictation\":"));
}

#[test]
fn the_models_root_is_absolute_after_resolution() {
    assert!(models_root().is_absolute() || models_root().starts_with("engines"));
}

#[test]
fn the_portuguese_voice_is_european() {
    let voice = crate::registry::model("voice-pt").expect("a Portuguese voice is offered");
    let file = match voice.source {
        crate::registry::Source::Voice(file) => file,
        _ => panic!("the Portuguese voice must point at a file"),
    };
    assert!(file.starts_with("pt_PT-"), "{file}");
    let paths = paths_for("voice-pt").unwrap();
    assert_eq!(paths[0].file_name().unwrap(), file);
}

#[test]
fn a_model_says_whether_it_sits_in_memory() {
    let models = vec![
        Model {
            id: "parakeet",
            name: "recogniser".to_string(),
            kind: "recogniser",
            feature: "Dictation",
            installed: true,
            bytes: 10,
        },
        Model {
            id: "mt-pt-en",
            name: "pair".to_string(),
            kind: "translation",
            feature: "Translation",
            installed: true,
            bytes: 10,
        },
        Model {
            id: "voice-pt",
            name: "voice".to_string(),
            kind: "voice",
            feature: "Read aloud",
            installed: false,
            bytes: 0,
        },
    ];
    let lifecycle = Lifecycle {
        manage: true,
        units: held(&["recognition", "dictation translation"]),
    };
    let parsed: serde_json::Value =
        serde_json::from_str(&as_json(&models, &lifecycle, "pt-en", false)).unwrap();
    let rows = parsed["models"].as_array().unwrap();
    assert_eq!(rows[0]["state"], "loaded");
    assert_eq!(rows[1]["state"], "loaded");
    assert_eq!(rows[2]["state"], "not installed");
    assert_eq!(rows[0]["idle_seconds"], 45);
    assert_eq!(rows[0]["policy"], 30);
    assert_eq!(rows[0]["policy_origin"], "global");
    assert!(rows[2]["idle_seconds"].is_null());
    assert!(rows[2]["policy"].is_null());
}

#[test]
fn a_model_that_was_not_asked_for_reads_as_on_disk() {
    let models = vec![Model {
        id: "voice-en",
        name: "voice".to_string(),
        kind: "voice",
        feature: "Read aloud",
        installed: true,
        bytes: 10,
    }];
    let parsed: serde_json::Value =
        serde_json::from_str(&as_json(&models, &Lifecycle::default(), "pt-en", false)).unwrap();
    assert_eq!(parsed["models"][0]["state"], "installed");
}

#[test]
fn the_recogniser_reads_in_use_while_the_engine_works() {
    let models = vec![
        Model {
            id: "parakeet",
            name: "recogniser".to_string(),
            kind: "recogniser",
            feature: "Dictation",
            installed: true,
            bytes: 10,
        },
        Model {
            id: "voice-en",
            name: "voice".to_string(),
            kind: "voice",
            feature: "Read aloud",
            installed: true,
            bytes: 10,
        },
    ];
    let lifecycle = Lifecycle {
        manage: true,
        units: held(&["recognition"]),
    };
    let parsed: serde_json::Value =
        serde_json::from_str(&as_json(&models, &lifecycle, "pt-en", true)).unwrap();
    assert_eq!(parsed["models"][0]["state"], "in use");
    assert_eq!(parsed["models"][1]["state"], "installed");
    let idle: serde_json::Value =
        serde_json::from_str(&as_json(&models, &lifecycle, "pt-en", false)).unwrap();
    assert_eq!(idle["models"][0]["state"], "loaded");
}

#[test]
fn every_row_states_a_number_or_says_there_is_none() {
    let models = status();
    let parsed: serde_json::Value =
        serde_json::from_str(&as_json(&models, &Lifecycle::default(), "pt-en", false)).unwrap();
    for row in parsed["models"].as_array().unwrap() {
        let quality = row["quality"].as_str().unwrap();
        assert!(!quality.is_empty(), "{}", row["id"]);
    }
    let voice = models.iter().find(|model| model.id == "voice-en").unwrap();
    assert!(quality_for(voice).contains("no quality number measured"));
}
