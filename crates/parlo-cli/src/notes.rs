use crate::meeting::Meeting;

pub fn markdown(meeting: &Meeting) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", meeting.title.trim()));
    let decisions = crate::decisions::for_meeting(meeting);
    if !decisions.is_empty() {
        out.push_str("## Decisions\n\n");
        for decision in &decisions {
            out.push_str(&format!(
                "- {} · {} · {}{}\n",
                stamp(decision.at_ms),
                decision.speaker,
                decision.text.trim(),
                match decision.supersedes {
                    Some(replaced) => format!(" (replaces the one at {})", stamp(replaced)),
                    None => String::new(),
                }
            ));
        }
        out.push('\n');
    }
    let actions = crate::actions::for_meeting(meeting);
    if !actions.is_empty() {
        out.push_str("## Action items\n\n");
        for action in &actions {
            out.push_str(&format!(
                "- {} · {}{} · {}\n",
                action.owner,
                action.due.as_deref().unwrap_or("no date"),
                String::new(),
                action.text.trim()
            ));
        }
        out.push('\n');
    }
    let notes: Vec<_> = meeting
        .segments
        .iter()
        .filter(|segment| segment.speaker == "Note")
        .collect();
    if !notes.is_empty() {
        out.push_str("## Notes\n\n");
        for note in notes {
            out.push_str(&format!("- {} · {}\n", stamp(note.at_ms), note.text.trim()));
        }
        out.push('\n');
    }
    if !meeting.segments.is_empty() {
        out.push_str("## Transcript\n\n");
        for segment in &meeting.segments {
            out.push_str(&format!(
                "- [{}] {}: {}\n",
                stamp(segment.at_ms),
                segment.spoken_by(),
                segment.text.trim()
            ));
        }
    }
    out
}

pub fn file_name(meeting: &Meeting) -> String {
    let mut name: String = meeting
        .title
        .trim()
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect();
    while name.contains("--") {
        name = name.replace("--", "-");
    }
    let name = name.trim_matches('-');
    let name = if name.is_empty() { "meeting" } else { name };
    format!("{name}.md")
}

fn stamp(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms / 60_000) % 60;
    let seconds = (ms / 1_000) % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meeting::Segment;

    fn meeting(lines: &[(u64, &str, &str)]) -> Meeting {
        Meeting {
            id: "m1".to_string(),
            title: "Product Sync".to_string(),
            started_at: 0,
            segments: lines
                .iter()
                .map(|(at_ms, speaker, text)| Segment {
                    at_ms: *at_ms,
                    speaker: speaker.to_string(),
                    text: text.to_string(),
                    name: None,
                })
                .collect(),
        }
    }

    #[test]
    fn the_title_opens_the_file() {
        let text = markdown(&meeting(&[]));
        assert_eq!(text, "# Product Sync\n\n");
    }

    #[test]
    fn decisions_come_before_the_transcript() {
        let text = markdown(&meeting(&[
            (0, "Miguel", "Bom dia a todos."),
            (5_000, "Miguel", "Decidimos aprovar o orçamento."),
        ]));
        let decisions = text.find("## Decisions").expect("decisions");
        let transcript = text.find("## Transcript").expect("transcript");
        assert!(decisions < transcript);
        assert!(text.contains("- 00:05 · Miguel · Decidimos aprovar o orçamento.\n"));
    }

    #[test]
    fn a_replaced_decision_names_the_one_it_replaced() {
        let text = markdown(&meeting(&[
            (0, "Miguel", "Decidimos que o lançamento é em março."),
            (
                61_000,
                "Jorge",
                "Afinal decidimos adiar o lançamento para abril.",
            ),
        ]));
        let decisions = text.split("## Transcript").next().unwrap_or(&text);
        assert!(decisions.contains("(replaces the one at 00:00)"), "{text}");
        assert!(!decisions.contains("em março"), "{text}");
    }

    #[test]
    fn an_action_carries_its_owner_and_its_due_phrase() {
        let text = markdown(&meeting(&[(
            3_000,
            "Miguel",
            "Eu envio o relatório amanhã.",
        )]));
        assert!(
            text.contains("## Action items\n\n- Miguel · amanhã · Eu envio o relatório amanhã.\n"),
            "{text}"
        );
    }

    #[test]
    fn a_note_is_kept_apart_from_the_transcript() {
        let text = markdown(&meeting(&[(0, "Note", "Perguntar o preço.")]));
        assert!(
            text.contains("## Notes\n\n- 00:00 · Perguntar o preço.\n"),
            "{text}"
        );
        assert!(text.contains("## Transcript\n\n- [00:00] Note: Perguntar o preço.\n"));
    }

    #[test]
    fn every_line_of_the_transcript_names_who_spoke() {
        let text = markdown(&meeting(&[
            (0, "Speaker 1", "Bom dia."),
            (3_000, "Speaker 2", "Boa tarde."),
        ]));
        assert!(text.contains("- [00:00] Speaker 1: Bom dia.\n"));
        assert!(text.contains("- [00:03] Speaker 2: Boa tarde.\n"));
    }

    #[test]
    fn the_file_name_follows_the_title() {
        assert_eq!(file_name(&meeting(&[])), "product-sync.md");
        let mut untitled = meeting(&[]);
        untitled.title = "  ".to_string();
        assert_eq!(file_name(&untitled), "meeting.md");
    }
}
