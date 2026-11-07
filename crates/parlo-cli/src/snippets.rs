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
    let wanted = normalise(text);
    if wanted.is_empty() {
        return text.to_string();
    }
    saved
        .iter()
        .find(|(cue, _)| normalise(cue) == wanted)
        .map(|(_, value)| value.trim().to_string())
        .unwrap_or_else(|| text.to_string())
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
    fn a_saved_phrase_inside_a_sentence_is_left_alone() {
        assert_eq!(
            apply("send my address to her", &saved()),
            "send my address to her"
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
