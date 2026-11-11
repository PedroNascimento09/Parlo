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

pub const COMMANDS: &[&str] = &[
    "formal",
    "casual",
    "excited",
    "uppercase",
    "lowercase",
    "title",
    "shorter",
    "bullets",
];

const FILLER: &[&str] = &[
    "um",
    "uh",
    "hmm",
    "ah",
    "like",
    "just",
    "very",
    "really",
    "basically",
    "actually",
    "so",
];

pub fn command(text: &str, command: &str) -> Result<String, String> {
    match command {
        "formal" | "casual" | "excited" => Ok(shape(text, command)),
        "uppercase" => Ok(text.to_uppercase()),
        "lowercase" => Ok(text.to_lowercase()),
        "title" => Ok(title(text)),
        "shorter" => Ok(shorter(text)),
        "bullets" => Ok(bullets(text)),
        name => Err(format!(
            "unknown command \"{name}\"; the choices are {}",
            COMMANDS.join(", ")
        )),
    }
}

fn title(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut characters = word.chars();
            match characters.next() {
                Some(first) => {
                    let mut titled = first.to_uppercase().to_string();
                    titled.push_str(&characters.as_str().to_lowercase());
                    titled
                }
                None => word.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn shorter(text: &str) -> String {
    text.split_whitespace()
        .filter(|word| {
            let bare = word.to_lowercase();
            !FILLER.contains(&bare.trim_matches(|mark| ".,!?;".contains(mark)))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn bullets(text: &str) -> String {
    text.split(|mark| ".?!\n".contains(mark))
        .map(str::trim)
        .filter(|sentence| !sentence.is_empty())
        .map(|sentence| format!("- {sentence}"))
        .collect::<Vec<_>>()
        .join("\n")
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

    #[test]
    fn a_command_mode_style_delegates_to_the_same_shaping() {
        assert_eq!(command("hello world", "uppercase").unwrap(), "HELLO WORLD");
        assert_eq!(command("HELLO WORLD", "lowercase").unwrap(), "hello world");
        assert_eq!(command("hello world", "title").unwrap(), "Hello World");
        assert_eq!(
            command("i think like we should just go", "shorter").unwrap(),
            "i think we should go"
        );
        assert_eq!(
            command("we ship. we test. we rest.", "bullets").unwrap(),
            "- we ship\n- we test\n- we rest"
        );
        assert_eq!(command("hey there", "formal").unwrap(), "Hey there.");
    }

    #[test]
    fn an_unknown_command_names_the_choices() {
        let error = command("hello", "whisper").unwrap_err();
        assert!(error.contains("unknown command"));
        assert!(error.contains("bullets"));
    }
}
