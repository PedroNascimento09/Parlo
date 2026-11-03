const COMMANDS: &[(&str, &str)] = &[
    ("new line", "\n"),
    ("nova linha", "\n"),
    ("new paragraph", "\n\n"),
    ("novo paragrafo", "\n\n"),
    ("full stop", "."),
    ("ponto final", "."),
    ("period", "."),
    ("ponto", "."),
    ("comma", ","),
    ("virgula", ","),
    ("question mark", "?"),
    ("ponto de interrogacao", "?"),
    ("exclamation mark", "!"),
    ("exclamation point", "!"),
    ("ponto de exclamacao", "!"),
    ("semicolon", ";"),
    ("ponto e virgula", ";"),
    ("colon", ":"),
    ("dois pontos", ":"),
    ("dash", " - "),
    ("hyphen", " - "),
    ("scratch that", SCRATCH),
    ("risca isso", SCRATCH),
    ("apaga isso", SCRATCH),
];

const SCRATCH: &str = "\u{8}";

pub fn apply(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    while index < words.len() {
        if let Some((taken, piece)) = match_at(&words, index) {
            if piece == SCRATCH {
                scratch(&mut out);
            } else if piece.starts_with('\n') {
                while out.ends_with(' ') {
                    out.pop();
                }
                out.push_str(piece);
            } else if piece.starts_with(" - ") {
                out.push_str(piece);
            } else {
                while out.ends_with(' ') {
                    out.pop();
                }
                out.push_str(piece);
                out.push(' ');
            }
            index += taken;
            continue;
        }
        if !out.is_empty() && !out.ends_with(['\n', ' ']) {
            out.push(' ');
        }
        out.push_str(words[index]);
        index += 1;
    }
    out.trim_end().to_string()
}

fn match_at(words: &[&str], index: usize) -> Option<(usize, &'static str)> {
    let mut folded: Vec<String> = Vec::new();
    let mut best: Option<(usize, &'static str)> = None;
    for offset in 0..3 {
        let Some(word) = words.get(index + offset) else {
            break;
        };
        folded.push(fold(word));
        let phrase = folded.join(" ");
        if let Some((_, piece)) = COMMANDS.iter().find(|(command, _)| *command == phrase) {
            best = Some((offset + 1, piece));
        }
    }
    best
}

fn fold(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for character in word.chars() {
        let plain = match character {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ç' | 'Ç' => 'c',
            other => other.to_ascii_lowercase(),
        };
        if plain.is_alphanumeric() || plain == '\'' {
            out.push(plain);
        }
    }
    out
}

fn scratch(out: &mut String) {
    let boundary = out
        .char_indices()
        .filter(|(_, character)| matches!(character, '.' | '!' | '?' | '\n'))
        .map(|(at, character)| at + character.len_utf8())
        .next_back()
        .unwrap_or(0);
    out.truncate(boundary);
    while out.ends_with(' ') {
        out.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_comma_lands_against_the_word_before_it() {
        assert_eq!(apply("hello world comma yes"), "hello world, yes");
    }

    #[test]
    fn a_period_ends_the_sentence() {
        assert_eq!(apply("that is all full stop"), "that is all.");
        assert_eq!(apply("that is all period"), "that is all.");
    }

    #[test]
    fn portuguese_commands_work_with_or_without_accents() {
        assert_eq!(apply("olá vírgula tudo bem"), "olá, tudo bem");
        assert_eq!(apply("já está ponto final"), "já está.");
        assert_eq!(apply("não sei ponto de interrogação"), "não sei?");
    }

    #[test]
    fn a_new_line_breaks_without_a_space_before_it() {
        assert_eq!(apply("first new line second"), "first\nsecond");
        assert_eq!(apply("first new paragraph second"), "first\n\nsecond");
    }

    #[test]
    fn scratch_that_drops_the_sentence_it_closes() {
        assert_eq!(apply("keep this. wrong bit scratch that"), "keep this.");
        assert_eq!(apply("wrong bit scratch that"), "");
    }

    #[test]
    fn an_ordinary_word_is_left_alone() {
        assert_eq!(apply("the point of it"), "the point of it");
        assert_eq!(apply("commas are useful"), "commas are useful");
    }

    #[test]
    fn a_command_inside_a_longer_sentence_keeps_the_words_around_it() {
        assert_eq!(apply("send it colon now question mark"), "send it: now?");
    }
}
