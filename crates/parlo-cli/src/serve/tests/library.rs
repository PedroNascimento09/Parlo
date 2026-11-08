use super::*;

#[test]
fn the_library_payloads_carry_the_fields_the_panels_read() {
    let meeting = crate::meeting::Meeting {
        id: "m1".to_string(),
        title: "Review".to_string(),
        started_at: 1,
        segments: vec![crate::meeting::Segment {
            at_ms: 0,
            speaker: "Speaker 1".to_string(),
            text: "We ship on Friday.".to_string(),
            name: None,
        }],
        speaker_hint: None,
    };
    let payload = meeting_json(meeting);
    for key in [
        "id",
        "title",
        "started_at",
        "speakers",
        "segments",
        "actions",
        "decisions",
    ] {
        assert!(payload.get(key).is_some(), "meetings lost {key}");
    }
    let segment = &payload["segments"][0];
    for key in ["at_ms", "speaker", "spoken_by", "text", "named"] {
        assert!(segment.get(key).is_some(), "a meeting line lost {key}");
    }

    let batch = crate::batch::Batch {
        id: "b1".to_string(),
        from: "en".to_string(),
        targets: vec!["pt".to_string()],
        timing: "natural".to_string(),
        fastest: 1.0,
        slowest: 2.0,
        keep_background: false,
        state: crate::batch::RUNNING.to_string(),
        created_at: 1,
        updated_at: 1,
        story: None,
        items: vec![crate::batch::Item {
            id: "i1".to_string(),
            source: "a.wav".to_string(),
            title: "a.wav".to_string(),
            to: "pt".to_string(),
            state: crate::batch::DONE.to_string(),
            stage: "ready".to_string(),
            progress: 1.0,
            error: None,
            out: Some("a.pt.wav".to_string()),
            srt: None,
            seconds: 1.0,
            lines: 1,
            started_at: None,
            ended_at: None,
        }],
    };
    let payload = serde_json::to_value(&batch).expect("a batch payload");
    for key in ["id", "state", "from", "targets", "story", "items"] {
        assert!(payload.get(key).is_some(), "batches lost {key}");
    }
    for key in ["id", "title", "to", "state", "error"] {
        assert!(
            payload["items"][0].get(key).is_some(),
            "a batch item lost {key}"
        );
    }

    let story = crate::story::Story {
        id: "s1".to_string(),
        title: "Coast".to_string(),
        author: None,
        year: None,
        series: None,
        comment: None,
        language: "pt".to_string(),
        state: crate::story::DRAFT.to_string(),
        batch: None,
        error: None,
        rendered: 0,
        cached: 0,
        failed: 0,
        created_at: 1,
        updated_at: 1,
        characters: vec![crate::story::Character {
            id: "c1".to_string(),
            name: "Keeper".to_string(),
            voice: "pt_PT-tugão-medium".to_string(),
        }],
        chapters: vec![crate::story::Chapter {
            id: "ch1".to_string(),
            title: "Chapter 1".to_string(),
            lines: Vec::new(),
        }],
    };
    let payload = serde_json::to_value(&story).expect("a story payload");
    for key in ["id", "title", "state", "characters", "chapters"] {
        assert!(payload.get(key).is_some(), "stories lost {key}");
    }

    let app = include_str!("../../../web/app.js");
    for token in [
        "meeting.segments",
        "meeting.speakers",
        "meeting.decisions",
        "meeting.actions",
        "batch.targets",
        "story.characters",
        "project.exports",
        "entry.translation",
    ] {
        assert!(app.contains(token), "the panels no longer read {token}");
    }
}

#[test]
fn an_empty_write_choice_follows_the_pair() {
    let table: serde_json::Value =
        serde_json::from_str(include_str!("../../../web/languages.json"))
            .expect("a language table");
    for pair in table["pairs"].as_array().expect("language pairs") {
        let from = pair[0].as_str().expect("a source language");
        let to = pair[1].as_str().expect("a target language");
        assert!(
            crate::config::check_language(to),
            "{to} would be refused with a 400"
        );
        assert_eq!(dictation_target("", from, to, true), to);
        assert_eq!(dictation_target("", from, to, false), from);
        assert_eq!(dictation_target(to, from, to, true), to);
    }
}
