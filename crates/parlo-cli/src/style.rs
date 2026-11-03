pub const STYLES: [&str; 3] = ["formal", "casual", "excited"];

pub fn is_known(style: &str) -> bool {
    STYLES.contains(&style)
}

pub fn shape(text: &str, style: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    match style {
        "formal" => formal(trimmed),
        "casual" => casual(trimmed),
        "excited" => excited(trimmed),
        _ => trimmed.to_string(),
    }
}

fn formal(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 1);
    let mut capitalise = true;
    for character in text.chars() {
        if character.is_alphanumeric() {
            if capitalise {
                out.extend(character.to_uppercase());
            } else {
                out.push(character);
            }
            capitalise = false;
            continue;
        }
        if matches!(character, '.' | '?' | '!') {
            capitalise = true;
        }
        out.push(character);
    }
    if out
        .chars()
        .last()
        .is_some_and(|last| last.is_alphanumeric())
    {
        out.push('.');
    }
    out
}

fn casual(text: &str) -> String {
    let first_word_is_i = text
        .split_whitespace()
        .next()
        .is_some_and(|word| word.trim_matches(|c: char| !c.is_alphanumeric()) == "I");
    let mut out = text.to_string();
    if !first_word_is_i {
        let mut chars = out.chars();
        if let Some(first) = chars.next() {
            let rest: String = chars.collect();
            out = first.to_lowercase().collect::<String>() + &rest;
        }
    }
    if out.ends_with('.') {
        out.pop();
    }
    out
}

fn excited(text: &str) -> String {
    let mut out = text.to_string();
    if out.ends_with('.') {
        out.pop();
    }
    if out
        .chars()
        .last()
        .is_some_and(|last| last.is_alphanumeric())
    {
        out.push('!');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_closes_the_sentence_and_capitalises_after_a_stop() {
        assert_eq!(
            shape("hey there. how are things", "formal"),
            "Hey there. How are things."
        );
    }

    #[test]
    fn casual_drops_the_first_cap_and_the_final_stop() {
        assert_eq!(
            shape("Hey, if you're free, let's talk.", "casual"),
            "hey, if you're free, let's talk"
        );
    }

    #[test]
    fn casual_keeps_a_lone_i_capitalised() {
        assert_eq!(shape("I am free.", "casual"), "I am free");
    }

    #[test]
    fn excited_turns_a_stop_into_an_exclamation_mark() {
        assert_eq!(
            shape("that is great. see you", "excited"),
            "that is great. see you!"
        );
    }

    #[test]
    fn unknown_styles_leave_the_text_alone() {
        assert_eq!(shape("  as dictated  ", "natural"), "as dictated");
    }
}
