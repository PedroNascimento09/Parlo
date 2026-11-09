use super::*;

#[test]
fn edited_segments_survive_the_store_and_invalidate_the_translation() {
    let mut store = crate::store::Store::open_memory().unwrap();
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: Some("Good morning".to_string()),
        state: "dubbed".to_string(),
    });
    store.save_dub_project(&mut project).unwrap();

    apply_project_edits(
        &mut project,
        &[SegmentEdit {
            id: "p1-0".to_string(),
            start_ms: 500,
            end_ms: 2_750,
            speaker: Some("Beatriz".to_string()),
            voice_id: None,
            pace: None,
            source_text: "Bom dia a todos".to_string(),
        }],
        &std::collections::BTreeMap::new(),
    )
    .unwrap();
    store.save_dub_project(&mut project).unwrap();
    let loaded = store.load_dub_project("p1").unwrap().unwrap();

    assert_eq!(loaded.segments[0].start_ms, 500);
    assert_eq!(loaded.segments[0].end_ms, 2_750);
    assert_eq!(loaded.segments[0].speaker.as_deref(), Some("Beatriz"));
    assert_eq!(loaded.segments[0].source_text, "Bom dia a todos");
    assert_eq!(loaded.segments[0].original_text.as_deref(), Some("Bom dia"));
    assert!(loaded.segments[0].target_text.is_none());
    assert_eq!(loaded.segments[0].state, "edited");

    apply_project_edits(
        &mut project,
        &[SegmentEdit {
            id: "p1-0".to_string(),
            start_ms: 500,
            end_ms: 2_750,
            speaker: Some("Beatriz".to_string()),
            voice_id: None,
            pace: None,
            source_text: "Bom dia a todos, bom ano".to_string(),
        }],
        &std::collections::BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(
        project.segments[0].revisions,
        ["Bom dia", "Bom dia a todos"]
    );
    assert_eq!(project.segments[0].source_text, "Bom dia a todos, bom ano");
    assert_eq!(
        project.segments[0].original_text.as_deref(),
        Some("Bom dia")
    );

    apply_project_edits(
        &mut project,
        &[SegmentEdit {
            id: "p1-0".to_string(),
            start_ms: 500,
            end_ms: 2_750,
            speaker: Some("Beatriz".to_string()),
            voice_id: None,
            pace: None,
            source_text: "Bom dia".to_string(),
        }],
        &std::collections::BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(
        project.segments[0].revisions,
        ["Bom dia", "Bom dia a todos", "Bom dia a todos, bom ano"]
    );
    assert_eq!(project.segments[0].source_text, "Bom dia");
    assert_eq!(
        project.segments[0].original_text.as_deref(),
        Some("Bom dia")
    );

    let published = serde_json::to_value(&project).unwrap();
    assert_eq!(published["segments"][0]["source_text"], "Bom dia");
    assert_eq!(published["segments"][0]["original_text"], "Bom dia");
    assert_eq!(
        published["segments"][0]["revisions"]
            .as_array()
            .map(|versions| versions.len()),
        Some(3)
    );

    store.save_dub_project(&mut project).unwrap();
    let reloaded = store.load_dub_project("p1").unwrap().unwrap();
    assert_eq!(
        reloaded.segments[0].revisions,
        project.segments[0].revisions
    );
    assert_eq!(reloaded.segments[0].source_text, "Bom dia");
}

#[test]
fn cue_documents_keep_segment_identity_and_timing() {
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: Some(1.1),
        target_text: None,
        state: "imported".to_string(),
    });

    let json = cue_document(&project).unwrap();
    assert!(json.contains("\"pace\":1.1"), "{json}");
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(document["cues"][0]["id"], "p1-0");
    assert_eq!(document["cues"][0]["pace"], 1.1);
    assert_eq!(document["cues"][0]["start_ms"], 1_250);
    assert_eq!(document["cues"][0]["end_ms"], 3_500);
    assert_eq!(document["cues"][0]["source"], "Bom dia");
}

#[test]
fn casting_and_overrides_decide_the_voice_each_line_uses() {
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project
        .cast
        .insert("Ana".to_string(), "voice-en".to_string());
    project
        .cast
        .insert("Bob".to_string(), "voice-de".to_string());
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });
    project.segments.push(crate::store::DubSegment {
        id: "p1-1".to_string(),
        position: 1,
        start_ms: 4_000,
        end_ms: 6_000,
        speaker: Some("Bob".to_string()),
        voice_id: Some("voice-fr".to_string()),
        translations: std::collections::BTreeMap::new(),
        source_text: "Até logo".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });
    project.segments.push(crate::store::DubSegment {
        id: "p1-2".to_string(),
        position: 2,
        start_ms: 7_000,
        end_ms: 8_000,
        speaker: None,
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Sem voz".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });

    let document: serde_json::Value =
        serde_json::from_str(&cue_document(&project).unwrap()).unwrap();

    assert_eq!(document["cues"][0]["voice"], "voice-en");
    assert_eq!(document["cues"][1]["voice"], "voice-fr");
    assert!(document["cues"][2]["voice"].is_null());
}

#[test]
fn project_edits_keep_the_cast_and_refuse_an_unknown_voice() {
    let mut store = crate::store::Store::open_memory().unwrap();
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: Some("Good morning".to_string()),
        state: "dubbed".to_string(),
    });
    store.save_dub_project(&mut project).unwrap();
    let edit = [SegmentEdit {
        id: "p1-0".to_string(),
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: Some("voice-fr".to_string()),
        pace: Some(0.9),
        source_text: "Bom dia".to_string(),
    }];
    let mut cast = std::collections::BTreeMap::new();
    cast.insert("Ana".to_string(), "voice-en".to_string());

    apply_project_edits(&mut project, &edit, &cast).unwrap();
    store.save_dub_project(&mut project).unwrap();
    let loaded = store.load_dub_project("p1").unwrap().unwrap();

    assert_eq!(loaded.cast.get("Ana").map(String::as_str), Some("voice-en"));
    assert_eq!(loaded.segments[0].voice_id.as_deref(), Some("voice-fr"));
    assert_eq!(loaded.segments[0].pace, Some(0.9));
    assert_eq!(
        loaded.segments[0].target_text.as_deref(),
        Some("Good morning")
    );
    assert_eq!(loaded.segments[0].state, "edited");

    let mut wrong = std::collections::BTreeMap::new();
    wrong.insert("Ana".to_string(), "voice-klingon".to_string());
    let refused = apply_project_edits(&mut project, &edit, &wrong).unwrap_err();
    assert!(refused.contains("is not a voice"), "{refused}");

    let too_fast = [SegmentEdit {
        id: "p1-0".to_string(),
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        pace: Some(1.5),
        source_text: "Bom dia".to_string(),
    }];
    let refused = apply_project_edits(&mut project, &too_fast, &cast).unwrap_err();
    assert!(refused.contains("pace"), "{refused}");
}

#[test]
fn a_finished_dub_updates_matching_project_segments() {
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: None,
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });
    let lines = vec![
        dub::Line {
            start: 1.25,
            end: 2.0,
            source: "Bom dia".to_string(),
            translated: "Good morning".to_string(),
            id: Some("p1-0".to_string()),
            voice: None,
            ..Default::default()
        },
        dub::Line {
            start: 4.0,
            end: 5.0,
            source: "Até logo".to_string(),
            translated: "See you".to_string(),
            id: Some("missing".to_string()),
            voice: None,
            ..Default::default()
        },
    ];

    apply_dub(&mut project, &lines, "en");

    assert_eq!(
        project.segments[0].target_text.as_deref(),
        Some("Good morning")
    );
    assert_eq!(project.segments[0].state, "dubbed");
}

#[test]
fn a_project_keeps_one_translation_per_target_language() {
    let mut project =
        crate::store::DubProject::new("p1".to_string(), "launch".to_string(), "pt", "en", None);
    project.segments.push(crate::store::DubSegment {
        id: "p1-0".to_string(),
        position: 0,
        start_ms: 1_250,
        end_ms: 3_500,
        speaker: Some("Ana".to_string()),
        voice_id: Some("voice-en".to_string()),
        translations: std::collections::BTreeMap::new(),
        source_text: "Bom dia".to_string(),
        original_text: None,
        revisions: Vec::new(),
        pace: None,
        target_text: None,
        state: "imported".to_string(),
    });
    let line = |text: &str, voice: &str| dub::Line {
        start: 1.25,
        end: 2.0,
        source: "Bom dia".to_string(),
        translated: text.to_string(),
        id: Some("p1-0".to_string()),
        voice: Some(voice.to_string()),
        ..Default::default()
    };

    apply_dub(&mut project, &[line("Good morning", "voice-en")], "en");
    assert_eq!(project.targets, ["en"]);
    apply_dub(&mut project, &[line("Guten Morgen", "voice-de")], "de");

    assert_eq!(project.targets, ["en", "de"]);
    assert_eq!(
        project.segments[0]
            .translations
            .get("en")
            .map(String::as_str),
        Some("Good morning")
    );
    assert_eq!(
        project.segments[0]
            .translations
            .get("de")
            .map(String::as_str),
        Some("Guten Morgen")
    );
    assert_eq!(
        project.segments[0].target_text.as_deref(),
        Some("Good morning")
    );
}

#[test]
fn reads_the_filename_header_encoded() {
    assert_eq!(percent_decode("my%20clip.mp4"), "my clip.mp4");
    assert_eq!(percent_decode("a+b.mov"), "a b.mov");
    assert_eq!(percent_decode("plain.mp4"), "plain.mp4");
    assert_eq!(percent_decode("broken%zz.mp4"), "broken%zz.mp4");
}

#[test]
fn dubbed_names_keep_stem_and_suffix() {
    assert_eq!(dubbed_name("video.mp4", "en"), "video.en.mp4");
    assert_eq!(dubbed_name("talk.mkv", "pt"), "talk.pt.mkv");
    assert_eq!(dubbed_name("recording", "en"), "recording.en.wav");
}

#[test]
fn content_types_follow_the_suffix() {
    assert_eq!(content_type("a.en.mp4"), "video/mp4");
    assert_eq!(content_type("story.m4b"), "audio/mp4");
    assert_eq!(content_type("a.en.srt"), "text/plain; charset=utf-8");
    assert_eq!(content_type("a.en.weird"), "application/octet-stream");
}

#[test]
fn a_third_party_profile_cannot_skip_the_authorisation() {
    let refused = consent_for(
        Some("third-party"),
        false,
        "translation".into(),
        "local".into(),
        1,
    );
    assert!(refused.is_err());
    let allowed = consent_for(
        Some("third-party"),
        true,
        "translation".into(),
        "local".into(),
        1,
    )
    .unwrap();
    assert_eq!(allowed.origin.as_str(), "third-party");
    let own = consent_for(None, false, "translation".into(), "local".into(), 1).unwrap();
    assert_eq!(own.origin.as_str(), "own-voice");
}

#[test]
fn a_stream_piece_carries_its_state_and_text() {
    let pieces = [parlo_core::engine::StreamPiece {
        state: "partial".to_string(),
        text: "bom dia".to_string(),
        audio_ms: 640,
        latency_ms: 210,
    }];
    let answer = pieces_json(&pieces);
    assert!(answer.contains("\"partial\""));
    assert!(answer.contains("bom dia"));
}

#[test]
fn tokens_are_hex_and_unique_enough() {
    let token = new_token().unwrap();
    assert_eq!(token.len(), 48);
    assert!(token.chars().all(|character| character.is_ascii_hexdigit()));
    assert_ne!(token, new_token().unwrap());
}
