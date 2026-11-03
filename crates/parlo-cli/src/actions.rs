use crate::meeting::{Meeting, Segment};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct Action {
    pub at_ms: u64,
    pub speaker: String,
    pub owner: String,
    pub due: Option<String>,
    pub text: String,
}

const MARKS: &[&str] = &[
    "vou ",
    "vamos ",
    "fico de",
    "fica de",
    "ficou de",
    "trato ",
    "trata ",
    "envio ",
    "envia ",
    "mando ",
    "manda ",
    "preparo ",
    "prepara ",
    "reviso ",
    "revisa ",
    "i will",
    "i'll",
    "will send",
    "will prepare",
    "will review",
    "take care of",
    "action item",
];

const NEGATIONS: &[&str] = &[
    "nao vou",
    "nao envio",
    "nao mando",
    "nao trato",
    "nao preparo",
    "not going to",
    "won't",
    "will not",
];

const DUES: &[(&str, &str)] = &[
    ("depois de amanha", "depois de amanhã"),
    ("amanha", "amanhã"),
    ("hoje", "hoje"),
    ("semana que vem", "semana que vem"),
    ("proxima semana", "próxima semana"),
    ("fim de semana", "fim de semana"),
    ("fim do mes", "fim do mês"),
    ("esta semana", "esta semana"),
    ("segunda", "segunda"),
    ("terca", "terça"),
    ("quarta", "quarta"),
    ("quinta", "quinta"),
    ("sexta", "sexta"),
    ("sabado", "sábado"),
    ("domingo", "domingo"),
    ("next week", "next week"),
    ("this week", "this week"),
    ("tomorrow", "tomorrow"),
    ("today", "today"),
    ("end of day", "end of day"),
    ("friday", "friday"),
    ("monday", "monday"),
    ("tuesday", "tuesday"),
    ("wednesday", "wednesday"),
    ("thursday", "thursday"),
];

pub fn for_meeting(meeting: &Meeting) -> Vec<Action> {
    let mut found: Vec<Action> = Vec::new();
    for segment in &meeting.segments {
        let folded = fold(&segment.text);
        if NEGATIONS.iter().any(|mark| folded.contains(mark)) {
            continue;
        }
        let Some(at) = first_mark(&folded) else {
            continue;
        };
        if found.iter().any(|action| action.text == segment.text) {
            continue;
        }
        found.push(Action {
            at_ms: segment.at_ms,
            speaker: segment.spoken_by().to_string(),
            owner: owner_of(&segment.text, at, meeting, segment),
            due: due_of(&folded),
            text: segment.text.clone(),
        });
    }
    found
}

fn first_mark(folded: &str) -> Option<usize> {
    let mut best: Option<usize> = None;
    for mark in MARKS {
        if let Some(position) = folded.find(mark) {
            let after_word = folded[..position]
                .chars()
                .next_back()
                .map(|character| !character.is_alphanumeric())
                .unwrap_or(true);
            if !after_word {
                continue;
            }
            let characters = folded[..position].chars().count();
            if best.map(|seen| characters < seen).unwrap_or(true) {
                best = Some(characters);
            }
        }
    }
    best
}

fn owner_of(text: &str, at: usize, meeting: &Meeting, segment: &Segment) -> String {
    let prefix: String = text.chars().take(at).collect();
    let folded = fold(&prefix);
    let mut best: Option<(usize, String)> = None;
    for speaker in meeting.speakers() {
        if let Some(position) = folded.rfind(&fold(&speaker)) {
            if best
                .as_ref()
                .map(|(seen, _)| position > *seen)
                .unwrap_or(true)
            {
                best = Some((position, speaker));
            }
        }
    }
    if let Some((position, name)) = best {
        if position + 40 >= folded.len() {
            return name;
        }
    }
    if let Some(word) = capitalised(&prefix) {
        return word;
    }
    segment.spoken_by().to_string()
}

fn capitalised(prefix: &str) -> Option<String> {
    let word = prefix.split_whitespace().last()?;
    let clean: String = word
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect();
    if clean.chars().count() < 2 {
        return None;
    }
    let first = clean.chars().next()?;
    let refused = ["eu", "i", "o", "a", "os", "as", "nos", "we", "the"];
    if first.is_uppercase() && !refused.iter().any(|stop| clean.eq_ignore_ascii_case(stop)) {
        return Some(clean);
    }
    None
}

fn due_of(folded: &str) -> Option<String> {
    if let Some(rest) = folded.split("ate dia ").nth(1) {
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if !digits.is_empty() {
            return Some(format!("até dia {digits}"));
        }
    }
    DUES.iter()
        .find(|(when, _)| folded.contains(when))
        .map(|(_, display)| (*display).to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_promise_from_the_speaker_names_its_due_date() {
        let actions = for_meeting(&meeting(&[(
            3_000,
            "Miguel",
            "Eu envio o relatório amanhã.",
        )]));
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].owner, "Miguel");
        assert_eq!(actions[0].due, Some("amanhã".to_string()));
        assert_eq!(actions[0].at_ms, 3_000);
    }

    #[test]
    fn a_promise_about_someone_else_names_that_person() {
        let actions = for_meeting(&meeting(&[(
            1_000,
            "Jorge",
            "A Maria fica de preparar a proposta até sexta.",
        )]));
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].owner, "Maria");
        assert_eq!(actions[0].due, Some("sexta".to_string()));
    }

    #[test]
    fn a_named_speaker_becomes_the_owner() {
        let actions = for_meeting(&meeting(&[(
            0,
            "Miguel",
            "O João envia o contrato até dia 5.",
        )]));
        assert_eq!(actions[0].owner, "João");
        assert_eq!(actions[0].due, Some("até dia 5".to_string()));
    }

    #[test]
    fn a_denial_is_not_a_promise() {
        let actions = for_meeting(&meeting(&[(
            0,
            "Miguel",
            "Não vou enviar nada esta semana.",
        )]));
        assert!(actions.is_empty());
    }

    #[test]
    fn english_promises_are_read_too() {
        let actions = for_meeting(&meeting(&[(0, "Miguel", "I'll send the deck tomorrow.")]));
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].owner, "Miguel");
        assert_eq!(actions[0].due, Some("tomorrow".to_string()));
    }

    #[test]
    fn a_promise_without_a_date_says_so() {
        let actions = for_meeting(&meeting(&[(
            0,
            "Miguel",
            "Eu trato disso depois do almoço.",
        )]));
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].due, None);
    }

    #[test]
    fn a_plain_statement_is_not_an_action() {
        let actions = for_meeting(&meeting(&[(0, "Miguel", "O ecrã inicial está pronto.")]));
        assert!(actions.is_empty());
    }

    #[test]
    fn the_same_sentence_is_kept_once() {
        let actions = for_meeting(&meeting(&[
            (0, "Miguel", "Eu envio amanhã."),
            (1_000, "Miguel", "Eu envio amanhã."),
        ]));
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn actions_keep_the_order_of_the_meeting() {
        let actions = for_meeting(&meeting(&[
            (0, "Miguel", "Eu envio amanhã."),
            (5_000, "Jorge", "Eu reviso o texto na segunda."),
        ]));
        assert_eq!(actions.len(), 2);
        assert!(actions[0].at_ms < actions[1].at_ms);
        assert_eq!(actions[1].owner, "Jorge");
    }
}
