const COMMANDS: &[(&str, &str)] = &[
    ("new line", "\n"),
    ("new paragraph", "\n\n"),
    ("numbered list", NUMBERED_LIST),
    ("next number", NEXT_NUMBER),
    ("bullet point", "\n• "),
    ("bullet", "\n• "),
    ("full stop", "."),
    ("period", "."),
    ("comma", ","),
    ("question mark", "?"),
    ("exclamation mark", "!"),
    ("exclamation point", "!"),
    ("semicolon", ";"),
    ("colon", ":"),
    ("dash", " - "),
    ("hyphen", " - "),
    ("scratch that", SCRATCH),
    ("nueva linea", "\n"),
    ("nuevo parrafo", "\n\n"),
    ("viñeta", "\n• "),
    ("punto", "."),
    ("punto final", "."),
    ("coma", ","),
    ("signo de interrogacion", "?"),
    ("signo de exclamacion", "!"),
    ("borra eso", SCRATCH),
    ("nouvelle ligne", "\n"),
    ("nouveau paragraphe", "\n\n"),
    ("puce", "\n• "),
    ("point final", "."),
    ("virgule", ","),
    ("point d'interrogation", "?"),
    ("point d'exclamation", "!"),
    ("efface ça", SCRATCH),
    ("neue zeile", "\n"),
    ("neuer absatz", "\n\n"),
    ("punkt", "."),
    ("punkt final", "."),
    ("komma", ","),
    ("fragezeichen", "?"),
    ("ausrufezeichen", "!"),
    ("loesch das", SCRATCH),
    ("nova linha", "\n"),
    ("novo paragrafo", "\n\n"),
    ("ponto de lista", "\n• "),
    ("marcador", "\n• "),
    ("ponto final", "."),
    ("ponto", "."),
    ("virgula", ","),
    ("ponto de interrogacao", "?"),
    ("ponto de exclamacao", "!"),
    ("ponto e virgula", ";"),
    ("dois pontos", ":"),
    ("risca isso", SCRATCH),
    ("apaga isso", SCRATCH),
    ("lista numerada", NUMBERED_LIST),
    ("numero seguinte", NEXT_NUMBER),
    ("numero siguiente", NEXT_NUMBER),
    ("liste numerotee", NUMBERED_LIST),
    ("numero suivant", NEXT_NUMBER),
    ("nummerierte liste", NUMBERED_LIST),
    ("nachste nummer", NEXT_NUMBER),
    ("naechste nummer", NEXT_NUMBER),
];

const NUMBERED_LIST: &str = "\u{1}";
const NEXT_NUMBER: &str = "\u{2}";
const SCRATCH: &str = "\u{8}";

pub fn apply(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    let mut number = 0u32;
    while index < words.len() {
        if let Some((taken, piece)) = match_at(&words, index) {
            if piece == SCRATCH {
                scratch(&mut out);
            } else if piece == NUMBERED_LIST || piece == NEXT_NUMBER {
                number = if piece == NUMBERED_LIST {
                    1
                } else {
                    number.saturating_add(1)
                };
                while out.ends_with([' ', '\n']) {
                    out.pop();
                }
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&format!("{number}. "));
            } else if piece.starts_with('\n') {
                while out.ends_with([' ', '\n']) {
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
    fn english_commands_work() {
        assert_eq!(apply("yes comma no question mark"), "yes, no?");
        assert_eq!(apply("stop that exclamation mark"), "stop that!");
    }

    #[test]
    fn spanish_commands_work_with_or_without_accents() {
        assert_eq!(apply("hola coma todo bien"), "hola, todo bien");
        assert_eq!(apply("ya está punto final"), "ya está.");
        assert_eq!(apply("no se signo de interrogacion"), "no se?");
    }

    #[test]
    fn french_commands_work_with_or_without_accents() {
        assert_eq!(
            apply("bonjour virgule tout va bien"),
            "bonjour, tout va bien"
        );
        assert_eq!(apply("c'est fini point final"), "c'est fini.");
        assert_eq!(apply("quoi point d'interrogation"), "quoi?");
    }

    #[test]
    fn german_commands_work() {
        assert_eq!(apply("hallo komma alles gut"), "hallo, alles gut");
        assert_eq!(apply("es ist vorbei punkt"), "es ist vorbei.");
        assert_eq!(apply("was fragezeichen"), "was?");
    }

    #[test]
    fn portuguese_commands_still_work() {
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
    fn a_bullet_starts_a_list_on_its_own_line() {
        assert_eq!(
            apply("shopping bullet point milk new line bullet point bread"),
            "shopping\n• milk\n• bread"
        );
        assert_eq!(apply("lista marcador pao"), "lista\n• pao");
    }

    #[test]
    fn a_numbered_list_counts_up_on_its_own_lines() {
        assert_eq!(
            apply("plan numbered list milk next number bread"),
            "plan\n1. milk\n2. bread"
        );
        assert_eq!(
            apply("lista numerada pão numero seguinte leite"),
            "1. pão\n2. leite"
        );
    }

    #[test]
    fn a_command_inside_a_longer_sentence_keeps_the_words_around_it() {
        assert_eq!(apply("send it colon now question mark"), "send it: now?");
    }
}
