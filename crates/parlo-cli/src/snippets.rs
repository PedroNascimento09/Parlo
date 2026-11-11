use std::collections::BTreeMap;

pub fn command(args: &[String]) -> Result<(), String> {
    let mut stored = crate::config::load();
    match args.first().map(String::as_str) {
        None | Some("list") => {
            if stored.snippets.is_empty() {
                println!(
                    "no saved phrases yet: parlo snippet add \"my address\" \"12 Rua das Flores, Lisboa\""
                );
            }
            for (cue, value) in &stored.snippets {
                println!("{cue}  ->  {value}");
            }
            Ok(())
        }
        Some("add") => {
            let cue = check(
                args.get(1).map(String::as_str).unwrap_or(" "),
                120,
                "the phrase to say",
            )?;
            let value = check(
                args.get(2).map(String::as_str).unwrap_or(" "),
                2000,
                "the text it types",
            )?;
            stored.snippets.insert(normalise(&cue), value);
            crate::config::save(&stored).map(|_| ())
        }
        Some("remove") => {
            let cue = normalise(args.get(1).map(String::as_str).unwrap_or(""));
            if stored.snippets.remove(&cue).is_none() {
                return Err(format!("no saved phrase called {cue}"));
            }
            crate::config::save(&stored).map(|_| ())
        }
        Some(other) => Err(format!("snippet takes add, list or remove, not {other}")),
    }
}

pub fn apply(text: &str, saved: &BTreeMap<String, String>) -> String {
    if saved.is_empty() {
        return text.to_string();
    }
    let mut cues: Vec<(Vec<String>, String)> = saved
        .iter()
        .map(|(cue, value)| (fold_words(cue), value.trim().to_string()))
        .filter(|(cue, _)| !cue.is_empty())
        .collect();
    cues.sort_by_key(|left| std::cmp::Reverse(left.0.len()));
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return text.to_string();
    }
    let folded: Vec<String> = words.iter().map(|word| fold_word(word)).collect();
    let mut out: Vec<String> = Vec::new();
    let mut index = 0;
    while index < words.len() {
        let hit = cues.iter().find(|(cue, _)| {
            index + cue.len() <= words.len() && folded[index..index + cue.len()] == cue[..]
        });
        match hit {
            Some((cue, value)) => {
                out.push(value.clone());
                index += cue.len();
            }
            None => {
                out.push(words[index].to_string());
                index += 1;
            }
        }
    }
    out.join(" ")
}

fn fold_words(text: &str) -> Vec<String> {
    text.split_whitespace().map(fold_word).collect()
}

fn fold_word(word: &str) -> String {
    word.trim_matches(|mark: char| ".,!?;:".contains(mark))
        .to_lowercase()
}

pub fn check(value: &str, max: usize, what: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{what} cannot be empty"));
    }
    if trimmed.chars().count() > max {
        return Err(format!("{what} is longer than {max} characters"));
    }
    Ok(trimmed.to_string())
}

pub fn normalise(text: &str) -> String {
    text.trim()
        .trim_end_matches(['.', ',', '!', '?', ';', ':'])
        .trim()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved() -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        map.insert(
            "my address".to_string(),
            "12 Rua das Flores, Lisboa".to_string(),
        );
        map
    }

    #[test]
    fn a_saved_phrase_replaces_the_whole_dictation() {
        assert_eq!(apply("My address.", &saved()), "12 Rua das Flores, Lisboa");
        assert_eq!(
            apply("  my ADDRESS ", &saved()),
            "12 Rua das Flores, Lisboa"
        );
    }

    #[test]
    fn a_saved_phrase_inside_a_sentence_expands_inline() {
        assert_eq!(
            apply("send my address to her", &saved()),
            "send 12 Rua das Flores, Lisboa to her"
        );
    }

    #[test]
    fn the_longest_cue_wins_and_punctuation_does_not_block_a_match() {
        let mut map = saved();
        map.insert("address".to_string(), "the address".to_string());
        assert_eq!(
            apply("my address, please", &map),
            "12 Rua das Flores, Lisboa please"
        );
    }

    #[test]
    fn no_saved_phrases_change_nothing() {
        assert_eq!(
            apply("anything at all", &BTreeMap::new()),
            "anything at all"
        );
    }

    #[test]
    fn a_cue_and_a_value_have_to_carry_something() {
        assert!(check("   ", 120, "the phrase to say").is_err());
        assert!(check("ok", 120, "the phrase to say").is_ok());
        assert!(check(&"a".repeat(121), 120, "the phrase to say").is_err());
    }
}
