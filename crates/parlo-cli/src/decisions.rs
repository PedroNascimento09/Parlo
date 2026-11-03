use crate::ask::fold;
use crate::meeting::Meeting;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct Decision {
    pub at_ms: u64,
    pub speaker: String,
    pub text: String,
    pub supersedes: Option<u64>,
}

const CLOSES: &[&str] = &[
    "combinad",
    "decidid",
    "decidimos",
    "aprovad",
    "acordad",
    "entao fica",
    "decision",
    "agreed",
];

const OPEN: &[&str] = &[
    "talvez",
    "acho que",
    "nao sei",
    "decidimos depois",
    "maybe",
    "not sure",
];

const CLOSING_WORDS: &[&str] = &[
    "decid", "combin", "aprov", "acord", "agreed", "decision", "entao", "fica",
];

const STOP: &str = "para como onde quando sobre este esta isto isso the of to in on for with and or is are was were that this we us our they them you your";

pub fn for_meeting(meeting: &Meeting) -> Vec<Decision> {
    let mut kept: Vec<(Decision, Vec<String>)> = Vec::new();
    for segment in &meeting.segments {
        let folded = fold(&segment.text);
        if !CLOSES.iter().any(|mark| folded.contains(mark))
            || OPEN.iter().any(|block| folded.contains(block))
        {
            continue;
        }
        let words = topics(&folded);
        let superseded: Vec<usize> = kept
            .iter()
            .enumerate()
            .filter(|(_, (_, earlier))| words.iter().any(|word| earlier.contains(word)))
            .map(|(index, _)| index)
            .collect();
        let replaces = superseded.iter().map(|index| kept[*index].0.at_ms).max();
        for index in superseded.iter().rev() {
            kept.remove(*index);
        }
        kept.push((
            Decision {
                at_ms: segment.at_ms,
                speaker: segment.spoken_by().to_string(),
                text: segment.text.clone(),
                supersedes: replaces,
            },
            words,
        ));
    }
    kept.into_iter().map(|(decision, _)| decision).collect()
}

fn topics(folded: &str) -> Vec<String> {
    folded
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|word| word.chars().count() >= 4)
        .filter(|word| !STOP.split(' ').any(|stop| stop == *word))
        .filter(|word| !CLOSING_WORDS.iter().any(|prefix| word.starts_with(prefix)))
        .map(str::to_string)
        .collect()
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
    fn a_closed_decision_is_kept() {
        let decisions = for_meeting(&meeting(&[(
            0,
            "Jorge",
            "Então fica combinado: sexta-feira.",
        )]));
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].speaker, "Jorge");
        assert_eq!(decisions[0].supersedes, None);
    }

    #[test]
    fn a_maybe_is_not_a_decision() {
        let decisions = for_meeting(&meeting(&[(0, "Miguel", "Talvez fosse melhor outra cor.")]));
        assert!(decisions.is_empty());
    }

    #[test]
    fn a_later_decision_on_the_same_topic_replaces_the_earlier_one() {
        let decisions = for_meeting(&meeting(&[
            (0, "Miguel", "Decidimos que o lançamento é em março."),
            (
                5_000,
                "Jorge",
                "Afinal decidimos adiar o lançamento para abril.",
            ),
        ]));
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].at_ms, 5_000);
        assert_eq!(decisions[0].supersedes, Some(0));
    }

    #[test]
    fn decisions_on_different_topics_both_stay() {
        let decisions = for_meeting(&meeting(&[
            (0, "Miguel", "Decidimos aprovar o orçamento."),
            (5_000, "Jorge", "Decidimos marcar a reunião em abril."),
        ]));
        assert_eq!(decisions.len(), 2);
        assert!(decisions
            .iter()
            .all(|decision| decision.supersedes.is_none()));
    }

    #[test]
    fn an_english_agreement_is_a_decision() {
        let decisions = for_meeting(&meeting(&[(
            0,
            "Miguel",
            "Agreed: we ship the release on Friday.",
        )]));
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].supersedes, None);
    }

    #[test]
    fn decisions_keep_the_order_of_the_meeting() {
        let decisions = for_meeting(&meeting(&[
            (0, "Miguel", "Decidimos aprovar o orçamento."),
            (5_000, "Jorge", "Decidimos marcar a reunião em abril."),
        ]));
        assert!(decisions[0].at_ms < decisions[1].at_ms);
    }
}
