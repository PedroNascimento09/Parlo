use crate::meeting::Segment;

const DEFAULT_THRESHOLD: f32 = 0.72;
const MAX_SPEAKERS: usize = 8;
const BLEND_LINES: usize = 8;

const COMMON: &[&str] = &[
    "sim",
    "não",
    "nao",
    "ok",
    "okay",
    "claro",
    "bom",
    "boa",
    "bem",
    "então",
    "entao",
    "olá",
    "ola",
    "obrigado",
    "obrigada",
    "thanks",
    "thank",
    "yes",
    "no",
    "sure",
    "hey",
    "hi",
    "hello",
    "well",
    "so",
    "and",
    "but",
    "right",
    "certo",
    "pois",
    "agora",
    "depois",
    "hoje",
    "amanhã",
    "amanha",
    "ontem",
    "segunda",
    "terça",
    "terca",
    "quarta",
    "quinta",
    "sexta",
    "sábado",
    "sabado",
    "domingo",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
    "eu",
    "tu",
    "ele",
    "ela",
    "nós",
    "nos",
    "eles",
    "elas",
    "i",
    "you",
    "he",
    "she",
    "we",
    "they",
    "the",
    "um",
    "uma",
];

const AFFIRMATIONS: &[&str] = &[
    "sim",
    "yes",
    "claro",
    "combinado",
    "trato",
    "deixa comigo",
    "pode deixar",
    "vou tratar",
    "ok",
    "okay",
    "sure",
    "will do",
    "on it",
    "yep",
    "certo",
    "faço isso",
    "faco isso",
    "contigo",
];

pub struct Voice {
    centroid: Vec<f32>,
    label: String,
    lines: usize,
}

pub struct Room {
    voices: Vec<Voice>,
    threshold: f32,
}

impl Default for Room {
    fn default() -> Self {
        Self::new()
    }
}

impl Room {
    pub fn new() -> Self {
        Self {
            voices: Vec::new(),
            threshold: DEFAULT_THRESHOLD,
        }
    }

    #[cfg(test)]
    pub fn speakers(&self) -> Vec<String> {
        self.voices
            .iter()
            .map(|voice| voice.label.clone())
            .collect()
    }

    pub fn label(&mut self, embedding: &[f32]) -> Result<String, String> {
        if embedding.iter().any(|value| !value.is_finite()) {
            return Err("a voice vector carries a number that is not finite".to_string());
        }
        let unit = normalised(embedding);
        if unit.is_empty() {
            return Err("a voice vector is empty".to_string());
        }
        let closest = self
            .voices
            .iter()
            .enumerate()
            .map(|(index, voice)| (index, dot(&unit, &voice.centroid)))
            .max_by(|left, right| left.1.total_cmp(&right.1));
        if let Some((index, score)) = closest {
            if score >= self.threshold || self.voices.len() >= MAX_SPEAKERS {
                let label = self.voices[index].label.clone();
                self.absorb(index, &unit);
                return Ok(label);
            }
        }
        let label = format!("Speaker {}", self.voices.len() + 1);
        self.voices.push(Voice {
            centroid: unit,
            label: label.clone(),
            lines: 1,
        });
        Ok(label)
    }

    fn absorb(&mut self, index: usize, unit: &[f32]) {
        let voice = &mut self.voices[index];
        let weight = voice.lines.min(BLEND_LINES) as f32;
        for (slot, value) in voice.centroid.iter_mut().zip(unit) {
            *slot = (*slot * weight + value) / (weight + 1.0);
        }
        voice.centroid = normalised(&voice.centroid);
        voice.lines += 1;
    }
}

pub struct Named {
    pub speaker: String,
    pub name: String,
    pub evidence: String,
}

pub fn infer(segments: &[Segment]) -> Vec<Named> {
    let mut named: Vec<Named> = Vec::new();
    for pair in segments.windows(2) {
        let ask = &pair[0];
        let answer = &pair[1];
        if ask.speaker == answer.speaker || answer.speaker == "Note" {
            continue;
        }
        let Some(name) = vocative(&ask.text) else {
            continue;
        };
        if !affirms(&answer.text) {
            continue;
        }
        if named
            .iter()
            .any(|done| done.name.eq_ignore_ascii_case(&name) || done.speaker == answer.speaker)
        {
            continue;
        }
        named.push(Named {
            speaker: answer.speaker.clone(),
            name,
            evidence: format!("\"{}\" then \"{}\"", ask.text.trim(), answer.text.trim()),
        });
    }
    named
}

fn affirms(text: &str) -> bool {
    let plain = crate::ask::fold(text);
    AFFIRMATIONS.iter().any(|mark| plain.contains(mark))
}

fn vocative(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    for (index, raw) in words.iter().enumerate() {
        let trimmed = raw.trim_matches(|character: char| {
            !character.is_alphanumeric() && character != '-' && character != '\''
        });
        if trimmed.chars().count() < 2 {
            continue;
        }
        if !trimmed
            .chars()
            .next()
            .map(char::is_uppercase)
            .unwrap_or(false)
        {
            continue;
        }
        if !trimmed
            .chars()
            .all(|character| character.is_alphabetic() || character == '-' || character == '\'')
        {
            continue;
        }
        if COMMON.iter().any(|word| word.eq_ignore_ascii_case(trimmed)) {
            continue;
        }
        let called = raw.ends_with(',')
            || words
                .get(index + 1)
                .map(|next| next.starts_with(','))
                .unwrap_or(false)
            || (index > 0 && words[index - 1].ends_with(','));
        if !called {
            continue;
        }
        return Some(trimmed.to_string());
    }
    None
}

fn normalised(values: &[f32]) -> Vec<f32> {
    let length = values.iter().map(|value| value * value).sum::<f32>().sqrt();
    if length <= f32::EPSILON {
        return Vec::new();
    }
    values.iter().map(|value| value / length).collect()
}

fn dot(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right).map(|(one, other)| one * other).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(speaker: &str, text: &str) -> Segment {
        Segment {
            at_ms: 0,
            speaker: speaker.to_string(),
            text: text.to_string(),
            name: None,
        }
    }

    #[test]
    fn two_voices_get_their_own_number() {
        let mut room = Room::new();
        assert_eq!(room.label(&[1.0, 0.0, 0.0]).unwrap(), "Speaker 1");
        assert_eq!(room.label(&[0.0, 1.0, 0.0]).unwrap(), "Speaker 2");
        assert_eq!(room.speakers().len(), 2);
    }

    #[test]
    fn a_voice_that_returns_keeps_its_number() {
        let mut room = Room::new();
        let first = room.label(&[1.0, 0.0, 0.0]).unwrap();
        room.label(&[0.0, 1.0, 0.0]).unwrap();
        assert_eq!(room.label(&[0.98, 0.02, 0.0]).unwrap(), first);
        assert_eq!(room.speakers().len(), 2);
    }

    #[test]
    fn an_empty_or_broken_vector_is_refused() {
        let mut room = Room::new();
        assert!(room.label(&[]).is_err());
        assert!(room.label(&[f32::NAN, 1.0]).is_err());
        assert!(room.speakers().is_empty());
    }

    #[test]
    fn a_room_stops_opening_new_numbers_at_the_limit() {
        let mut room = Room::new();
        for index in 0..24 {
            let mut vector = vec![0.0; 24];
            vector[index] = 1.0;
            room.label(&vector).unwrap();
        }
        assert_eq!(room.speakers().len(), MAX_SPEAKERS);
    }

    #[test]
    fn a_name_called_out_is_learned_from_the_answer() {
        let lines = vec![
            segment("Speaker 1", "Consegues tratar disso, John?"),
            segment("Speaker 2", "Sim, trato disso hoje."),
        ];
        let named = infer(&lines);
        assert_eq!(named.len(), 1);
        assert_eq!(named[0].speaker, "Speaker 2");
        assert_eq!(named[0].name, "John");
        assert!(named[0].evidence.contains("John"));
    }

    #[test]
    fn a_hedged_answer_names_nobody() {
        let lines = vec![
            segment("Speaker 1", "Consegues tratar disso, John?"),
            segment("Speaker 2", "Não sei, talvez."),
        ];
        assert!(infer(&lines).is_empty());
    }

    #[test]
    fn an_affirmation_without_a_name_names_nobody() {
        let lines = vec![
            segment("Speaker 1", "Consegues tratar disso?"),
            segment("Speaker 2", "Sim."),
        ];
        assert!(infer(&lines).is_empty());
    }

    #[test]
    fn a_greeting_is_never_mistaken_for_a_name() {
        let lines = vec![
            segment("Speaker 1", "Sim, tudo bem."),
            segment("Speaker 2", "Sim, tudo bem."),
        ];
        assert!(infer(&lines).is_empty());
    }

    #[test]
    fn the_caller_cannot_name_themselves() {
        let lines = vec![
            segment("Speaker 1", "John, tratas disso?"),
            segment("Speaker 1", "Sim, trato."),
        ];
        assert!(infer(&lines).is_empty());
    }
}
