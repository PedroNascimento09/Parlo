use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub word: String,
    pub respell: String,
    pub ipa: String,
    pub cmu: String,
    #[serde(default)]
    pub by_language: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Dictionary {
    schema: u32,
    entries: Vec<Entry>,
}

const SOURCE: &str = include_str!("../assets/pronounce.json");

fn dictionary() -> &'static Dictionary {
    static DICTIONARY: OnceLock<Dictionary> = OnceLock::new();
    DICTIONARY.get_or_init(|| {
        let parsed: Dictionary =
            serde_json::from_str(SOURCE).expect("the pronunciation dictionary parses");
        assert_eq!(
            parsed.schema, 1,
            "the pronunciation dictionary has an unknown schema"
        );
        parsed
    })
}

pub fn entries() -> &'static [Entry] {
    &dictionary().entries
}

pub fn lookup(word: &str) -> Option<&'static Entry> {
    entries()
        .iter()
        .find(|entry| entry.word.eq_ignore_ascii_case(word))
}

impl Entry {
    pub fn spoken_in(&self, language: Option<&str>) -> &str {
        language
            .and_then(|language| self.by_language.get(language))
            .map(String::as_str)
            .unwrap_or(&self.respell)
    }
}

pub fn respell(text: &str) -> String {
    respell_in(text, None)
}

pub fn respell_in(text: &str, language: Option<&str>) -> String {
    let mut spoken = String::with_capacity(text.len());
    let mut word = String::new();
    for character in text.chars() {
        if character.is_ascii_alphanumeric() {
            word.push(character);
            continue;
        }
        speak_word(&mut spoken, &word, language);
        word.clear();
        spoken.push(character);
    }
    speak_word(&mut spoken, &word, language);
    spoken
}

fn speak_word(spoken: &mut String, word: &str, language: Option<&str>) {
    if word.is_empty() {
        return;
    }
    match lookup(word) {
        Some(entry) => spoken.push_str(entry.spoken_in(language)),
        None => spoken.push_str(word),
    }
}
