use crate::meeting::{Meeting, Segment};
use serde::Serialize;

const REFUSAL: &str =
    "I cannot find that in these meetings. I can show the decisions on record or search another meeting.";

#[derive(Debug, PartialEq, Serialize)]
pub struct Citation {
    pub meeting: String,
    pub title: String,
    pub speaker: String,
    pub at_ms: u64,
    pub text: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Answer {
    pub found: bool,
    pub answer: String,
    pub citations: Vec<Citation>,
}

#[derive(Clone, Copy, PartialEq)]
enum Intent {
    Decisions,
    Actions,
    Anything,
}

const STOPWORDS: &str = "a o as os um uma uns umas de da do das dos em no na nos nas por para com que e ou se ao aos à às the of to in on for with what who when where how did does was were is are that this and or about said say say about me my lhe the";

const INTENT_WORDS: &str = "decidido decidida decisao decisoes decision decided combinado combinada agreed responsavel responsaveis responsability tarefa tarefas task tasks action actions pendente pendentes pending aberto abertas open quem who quando when latest ultima ultimo reuniao reunioes meeting meetings ficou ficaram";

const DECISION_MARKS: &[&str] = &[
    "combinad",
    "decidid",
    "decidimos",
    "aprovad",
    "acordad",
    "entao fica",
    "decision",
    "agreed",
];

const DECISION_BLOCKS: &[&str] = &["talvez", "acho que", "nao sei", "decidimos depois"];

const ACTION_MARKS: &[&str] = &[
    "vou ",
    "ficou de",
    "fica de",
    "trato",
    "trata",
    "responsavel",
    "ate ",
    "amanha",
    "action item",
    "i will",
];

pub fn ask(question: &str, meetings: &[Meeting]) -> Answer {
    let folded = fold(question);
    let keywords = content_words(&folded);
    let intent = intent_of(&folded);
    let mut hits: Vec<(i32, &Meeting, &Segment)> = Vec::new();
    for meeting in meetings {
        let speaker = speaker_in(&folded, meeting);
        for segment in &meeting.segments {
            if let Some(name) = &speaker {
                if fold(segment.spoken_by()) != fold(name) {
                    continue;
                }
            }
            let score = score(segment, &keywords, intent, speaker.as_deref());
            if score > 0 {
                hits.push((score, meeting, segment));
            }
        }
    }
    hits.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.2.at_ms.cmp(&right.2.at_ms))
    });
    let mut citations: Vec<Citation> = Vec::new();
    for (_, meeting, segment) in hits {
        if citations.len() == 3 {
            break;
        }
        if citations.iter().any(|citation| {
            citation.meeting == meeting.id
                && citation.at_ms == segment.at_ms
                && citation.text == segment.text
        }) {
            continue;
        }
        citations.push(Citation {
            meeting: meeting.id.clone(),
            title: meeting.title.clone(),
            speaker: segment.spoken_by().to_string(),
            at_ms: segment.at_ms,
            text: segment.text.clone(),
        });
    }
    if citations.is_empty() {
        return Answer {
            found: false,
            answer: REFUSAL.to_string(),
            citations,
        };
    }
    citations.sort_by_key(|citation| citation.at_ms);
    let quoted: Vec<String> = citations
        .iter()
        .map(|citation| {
            format!(
                "{} at {}: \"{}\"",
                citation.speaker,
                clock(citation.at_ms),
                citation.text
            )
        })
        .collect();
    let lead = match intent {
        Intent::Decisions => "The transcript closes this, in these words.",
        Intent::Actions => "The transcript names this, in these words.",
        Intent::Anything => "The transcript says this.",
    };
    Answer {
        found: true,
        answer: format!("{lead} {}", quoted.join(" ")),
        citations,
    }
}

fn fold(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    for character in text.chars() {
        let plain = match character {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            'ñ' | 'Ñ' => 'n',
            other => other,
        };
        for lower in plain.to_lowercase() {
            folded.push(lower);
        }
    }
    folded
}

fn tokenize(text: &str) -> Vec<String> {
    fold(text)
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
        .map(str::to_string)
        .collect()
}

fn content_words(folded: &str) -> Vec<String> {
    tokenize(folded)
        .into_iter()
        .filter(|word| word.chars().count() >= 3)
        .filter(|word| !STOPWORDS.split(' ').any(|stop| stop == word))
        .filter(|word| !INTENT_WORDS.split(' ').any(|stop| stop == word))
        .collect()
}

fn intent_of(folded: &str) -> Intent {
    if DECISION_MARKS.iter().any(|mark| folded.contains(mark))
        || ["decisao", "decision", "decidid", "decidido"]
            .iter()
            .any(|word| folded.contains(word))
    {
        return Intent::Decisions;
    }
    if ["respons", "tarefa", "pendent", "action", "quem", "who"]
        .iter()
        .any(|word| folded.contains(word))
    {
        return Intent::Actions;
    }
    Intent::Anything
}

fn closes(text: &str) -> bool {
    DECISION_MARKS.iter().any(|mark| text.contains(mark))
        && !DECISION_BLOCKS.iter().any(|block| text.contains(block))
}

fn promises(text: &str) -> bool {
    ACTION_MARKS.iter().any(|mark| text.contains(mark))
}

fn speaker_in(folded: &str, meeting: &Meeting) -> Option<String> {
    meeting
        .speakers()
        .into_iter()
        .find(|speaker| folded.contains(&fold(speaker)))
}

fn score(segment: &Segment, keywords: &[String], intent: Intent, speaker: Option<&str>) -> i32 {
    let text = fold(&segment.text);
    let hits = keywords
        .iter()
        .filter(|word| text.contains(word.as_str()))
        .count() as i32;
    let named = speaker
        .map(|name| fold(segment.spoken_by()) == fold(name))
        .unwrap_or(false);
    let marker = match intent {
        Intent::Decisions => closes(&text),
        Intent::Actions => promises(&text),
        Intent::Anything => false,
    };
    if hits == 0 && !named && !(keywords.is_empty() && marker) {
        return 0;
    }
    let mut value = hits * 2;
    if named {
        value += 4;
    }
    if marker {
        value += 3;
    }
    value
}

fn clock(at_ms: u64) -> String {
    format!("{}:{:02}", at_ms / 60_000, (at_ms / 1_000) % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meeting(id: &str, title: &str, lines: &[(u64, &str, &str)]) -> Meeting {
        Meeting {
            id: id.to_string(),
            title: title.to_string(),
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
    fn a_closed_decision_comes_back_with_its_line() {
        let meetings = vec![meeting(
            "m1",
            "Product Sync",
            &[
                (0, "Miguel", "Acho que podíamos lançar sexta-feira."),
                (5_000, "Jorge", "Então fica combinado: sexta-feira."),
            ],
        )];
        let answer = ask("O que ficou decidido?", &meetings);
        assert!(answer.found);
        assert_eq!(answer.citations.len(), 1);
        assert_eq!(answer.citations[0].speaker, "Jorge");
        assert!(answer.answer.contains("combinado"));
    }

    #[test]
    fn a_proposal_is_not_a_decision() {
        let meetings = vec![meeting(
            "m1",
            "Design",
            &[(0, "Miguel", "Talvez fosse melhor outra cor.")],
        )];
        let answer = ask("O que ficou decidido?", &meetings);
        assert!(!answer.found);
        assert!(answer.citations.is_empty());
        assert_eq!(answer.answer, REFUSAL);
    }

    #[test]
    fn a_question_without_evidence_is_refused() {
        let meetings = vec![meeting(
            "m1",
            "Product Sync",
            &[(0, "Miguel", "Vamos rever o ecrã inicial.")],
        )];
        let answer = ask("Quem falou do orçamento?", &meetings);
        assert!(!answer.found);
        assert_eq!(answer.answer, REFUSAL);
    }

    #[test]
    fn two_meetings_are_both_cited() {
        let meetings = vec![
            meeting("m1", "Março", &[(0, "Miguel", "O lançamento é em março.")]),
            meeting(
                "m2",
                "Abril",
                &[(0, "Jorge", "Adiamos o lançamento para abril.")],
            ),
        ];
        let answer = ask("O que foi dito sobre o lançamento?", &meetings);
        assert!(answer.found);
        assert_eq!(answer.citations.len(), 2);
        assert_eq!(answer.citations[0].title, "Março");
        assert_eq!(answer.citations[1].title, "Abril");
    }

    #[test]
    fn accents_in_the_question_match_plain_text() {
        let meetings = vec![meeting(
            "m1",
            "Product Sync",
            &[(3_000, "Miguel", "Eu trato do relatório amanhã.")],
        )];
        let answer = ask("Quem ficou responsável pelo relatório?", &meetings);
        assert!(answer.found);
        assert_eq!(answer.citations[0].speaker, "Miguel");
        assert_eq!(answer.citations[0].at_ms, 3_000);
        assert!(answer.answer.contains("0:03"));
    }

    #[test]
    fn a_named_speaker_filters_the_lines() {
        let meetings = vec![meeting(
            "m1",
            "Product Sync",
            &[
                (0, "Miguel", "O prazo é sexta."),
                (2_000, "Jorge", "O prazo é segunda."),
            ],
        )];
        let answer = ask("O que disse o Miguel sobre o prazo?", &meetings);
        assert!(answer.found);
        assert!(answer
            .citations
            .iter()
            .all(|citation| citation.speaker == "Miguel"));
    }
}
