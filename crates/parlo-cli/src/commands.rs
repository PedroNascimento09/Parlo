const ENGLISH: &[(&str, &str)] = &[
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
];

const SPANISH: &[(&str, &str)] = &[
    ("nueva linea", "\n"),
    ("nuevo parrafo", "\n\n"),
    ("vineta", "\n• "),
    ("punto", "."),
    ("punto final", "."),
    ("coma", ","),
    ("signo de interrogacion", "?"),
    ("signo de exclamacion", "!"),
    ("borra eso", SCRATCH),
    ("numero siguiente", NEXT_NUMBER),
];

const FRENCH: &[(&str, &str)] = &[
    ("nouvelle ligne", "\n"),
    ("nouveau paragraphe", "\n\n"),
    ("puce", "\n• "),
    ("point final", "."),
    ("virgule", ","),
    ("point d'interrogation", "?"),
    ("point d'exclamation", "!"),
    ("efface ca", SCRATCH),
    ("liste numerotee", NUMBERED_LIST),
    ("numero suivant", NEXT_NUMBER),
];

const GERMAN: &[(&str, &str)] = &[
    ("neue zeile", "\n"),
    ("neuer absatz", "\n\n"),
    ("punkt", "."),
    ("punkt final", "."),
    ("komma", ","),
    ("fragezeichen", "?"),
    ("ausrufezeichen", "!"),
    ("loesch das", SCRATCH),
    ("nummerierte liste", NUMBERED_LIST),
    ("nachste nummer", NEXT_NUMBER),
    ("naechste nummer", NEXT_NUMBER),
];

const PORTUGUESE: &[(&str, &str)] = &[
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
];

const NUMBERED_LIST: &str = "\u{1}";
const NEXT_NUMBER: &str = "\u{2}";
const SCRATCH: &str = "\u{8}";

pub fn apply(text: &str, language: &str) -> String {
    let commands = match language.split(['-', '_']).next().unwrap_or("") {
        "en" => ENGLISH,
        "es" => SPANISH,
        "fr" => FRENCH,
        "de" => GERMAN,
        "pt" => PORTUGUESE,
        _ => return text.to_string(),
    };
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = String::with_capacity(text.len());
    let mut index = 0;
    let mut number = 0u32;
    while index < words.len() {
        if let Some((taken, piece)) = match_at(&words, index, commands) {
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

fn match_at(
    words: &[&str],
    index: usize,
    commands: &'static [(&str, &str)],
) -> Option<(usize, &'static str)> {
    let mut folded: Vec<String> = Vec::new();
    let mut best: Option<(usize, &'static str)> = None;
    for offset in 0..3 {
        let Some(word) = words.get(index + offset) else {
            break;
        };
        folded.push(fold(word));
        let phrase = folded.join(" ");
        if let Some((_, piece)) = commands.iter().find(|(command, _)| *command == phrase) {
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
            'ñ' | 'Ñ' => 'n',
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
    use super::apply;

    #[test]
    fn a_comma_lands_against_the_word_before_it() {
        assert_eq!(apply("hello world comma yes", "en"), "hello world, yes");
    }

    #[test]
    fn a_period_ends_the_sentence() {
        assert_eq!(apply("that is all full stop", "en"), "that is all.");
        assert_eq!(apply("that is all period", "en"), "that is all.");
    }

    #[test]
    fn english_commands_work() {
        assert_eq!(apply("yes comma no question mark", "en"), "yes, no?");
        assert_eq!(apply("stop that exclamation mark", "en"), "stop that!");
    }

    #[test]
    fn spanish_commands_work_with_or_without_accents() {
        assert_eq!(apply("hola coma todo bien", "es"), "hola, todo bien");
        assert_eq!(apply("ya está punto final", "es"), "ya está.");
        assert_eq!(apply("no se signo de interrogacion", "es"), "no se?");
    }

    #[test]
    fn french_commands_work_with_or_without_accents() {
        assert_eq!(
            apply("bonjour virgule tout va bien", "fr"),
            "bonjour, tout va bien"
        );
        assert_eq!(apply("c'est fini point final", "fr"), "c'est fini.");
        assert_eq!(apply("quoi point d'interrogation", "fr"), "quoi?");
    }

    #[test]
    fn german_commands_work() {
        assert_eq!(apply("hallo komma alles gut", "de"), "hallo, alles gut");
        assert_eq!(apply("es ist vorbei punkt", "de"), "es ist vorbei.");
        assert_eq!(apply("was fragezeichen", "de"), "was?");
    }

    #[test]
    fn portuguese_commands_still_work() {
        assert_eq!(apply("olá vírgula tudo bem", "pt"), "olá, tudo bem");
        assert_eq!(apply("já está ponto final", "pt"), "já está.");
        assert_eq!(apply("não sei ponto de interrogação", "pt"), "não sei?");
    }

    #[test]
    fn a_new_line_breaks_without_a_space_before_it() {
        assert_eq!(apply("first new line second", "en"), "first\nsecond");
        assert_eq!(apply("first new paragraph second", "en"), "first\n\nsecond");
    }

    #[test]
    fn scratch_that_drops_the_sentence_it_closes() {
        assert_eq!(
            apply("keep this. wrong bit scratch that", "en"),
            "keep this."
        );
        assert_eq!(apply("wrong bit scratch that", "en"), "");
    }

    #[test]
    fn an_ordinary_word_is_left_alone() {
        assert_eq!(apply("the point of it", "en"), "the point of it");
        assert_eq!(apply("commas are useful", "en"), "commas are useful");
    }

    #[test]
    fn a_bullet_starts_a_list_on_its_own_line() {
        assert_eq!(
            apply(
                "shopping bullet point milk new line bullet point bread",
                "en"
            ),
            "shopping\n• milk\n• bread"
        );
        assert_eq!(apply("lista marcador pao", "pt"), "lista\n• pao");
    }

    #[test]
    fn a_numbered_list_counts_up_on_its_own_lines() {
        assert_eq!(
            apply("plan numbered list milk next number bread", "en"),
            "plan\n1. milk\n2. bread"
        );
        assert_eq!(
            apply("lista numerada pão numero seguinte leite", "pt"),
            "1. pão\n2. leite"
        );
    }

    #[test]
    fn commands_are_limited_to_the_spoken_language() {
        assert_eq!(apply("the punto is comma here", "en"), "the punto is, here");
        assert_eq!(
            apply("o bullet é útil vírgula sim", "pt-PT"),
            "o bullet é útil, sim"
        );
        assert_eq!(
            apply("leave  this untouched", "it"),
            "leave  this untouched"
        );
    }

    #[test]
    fn accented_commands_match_their_normalized_entries() {
        assert_eq!(apply("une erreur efface ça", "fr"), "");
        assert_eq!(apply("lista viñeta pan", "es"), "lista\n• pan");
    }

    #[test]
    fn a_command_inside_a_longer_sentence_keeps_the_words_around_it() {
        assert_eq!(
            apply("send it colon now question mark", "en"),
            "send it: now?"
        );
    }
}
