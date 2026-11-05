use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::dub;

pub const DRAFT: &str = "draft";
pub const RENDERING: &str = "rendering";
pub const DONE: &str = "done";
pub const FAILED: &str = "failed";
pub const PENDING: &str = "pending";

const VOICES: [&str; 3] = [
    "pt_PT-tugão-medium",
    "pt_BR-cadu-medium",
    "pt_BR-faber-medium",
];
const MAX_CHARACTERS: usize = 12;
const MAX_CHAPTERS: usize = 50;
const MAX_LINES: usize = 500;
const MAX_TEXT: usize = 1_000;
const MAX_AUTHOR: usize = 80;
const MAX_SERIES: usize = 80;
const MAX_COMMENT: usize = 500;

static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Character {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub voice: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoryLine {
    #[serde(default)]
    pub id: String,
    pub speaker: String,
    pub text: String,
    #[serde(default = "one")]
    pub pace: f64,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub out: Option<String>,
    #[serde(default)]
    pub seconds: f64,
    #[serde(default)]
    pub start: f64,
    #[serde(default)]
    pub end: f64,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chapter {
    #[serde(default)]
    pub id: String,
    pub title: String,
    pub lines: Vec<StoryLine>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Story {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub year: Option<u16>,
    #[serde(default)]
    pub series: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    pub language: String,
    pub state: String,
    #[serde(default)]
    pub batch: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub rendered: usize,
    #[serde(default)]
    pub cached: usize,
    #[serde(default)]
    pub failed: usize,
    pub created_at: u64,
    pub updated_at: u64,
    pub characters: Vec<Character>,
    pub chapters: Vec<Chapter>,
}

#[derive(Deserialize)]
pub struct Draft {
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub year: Option<u16>,
    #[serde(default)]
    pub series: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    pub language: String,
    pub characters: Vec<Character>,
    pub chapters: Vec<Chapter>,
}

pub struct StoryRender {
    pub seconds: f64,
    pub lines: usize,
}

fn one() -> f64 {
    1.0
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

fn new_id() -> String {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();
    format!("s{stamp}-{}", NEXT.fetch_add(1, Ordering::Relaxed))
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

fn clean(value: &str, field: &str, max: usize) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{field} cannot be empty"));
    }
    if value.chars().count() > max {
        return Err(format!("{field} is too long"));
    }
    Ok(value.to_string())
}

fn optional(value: Option<String>, field: &str, max: usize) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max {
        return Err(format!("{field} is too long"));
    }
    Ok(Some(value.to_string()))
}

pub fn voice_installed(voice: &str) -> bool {
    crate::models::models_root()
        .join(format!("{voice}.onnx"))
        .is_file()
}

pub fn voices() -> Vec<serde_json::Value> {
    VOICES
        .iter()
        .map(|voice| {
            serde_json::json!({
                "id": voice,
                "installed": voice_installed(voice),
                "label": voice.replace(['-', '_'], " "),
            })
        })
        .collect()
}

fn normalize(draft: Draft) -> Result<Story, String> {
    let title = clean(&draft.title, "the story title", 80)?;
    let author = optional(draft.author, "the author", MAX_AUTHOR)?;
    let series = optional(draft.series, "the series", MAX_SERIES)?;
    let comment = optional(draft.comment, "the comment", MAX_COMMENT)?;
    let year = match draft.year {
        Some(0) | Some(10_000..) => {
            return Err("the story year must be between 1 and 9999".to_string())
        }
        year => year,
    };
    let language = draft.language.trim().to_ascii_lowercase();
    if language.len() != 2
        || !language
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        return Err("choose a two-letter language".to_string());
    }
    if draft.characters.is_empty() {
        return Err("a story needs at least one character".to_string());
    }
    if draft.characters.len() > MAX_CHARACTERS {
        return Err(format!("a story takes at most {MAX_CHARACTERS} characters"));
    }
    let mut character_ids = HashSet::new();
    let mut names = HashSet::new();
    let mut characters = Vec::new();
    for (index, character) in draft.characters.into_iter().enumerate() {
        let id = if character.id.trim().is_empty() {
            format!("c{}", index + 1)
        } else {
            character.id.trim().to_string()
        };
        if !valid_id(&id) {
            return Err(format!("the character id {id} is not valid"));
        }
        if !character_ids.insert(id.clone()) {
            return Err(format!("the character id {id} appears twice"));
        }
        let name = clean(&character.name, "a character name", 40)?;
        if !names.insert(name.to_lowercase()) {
            return Err(format!("the character name {name} appears twice"));
        }
        let voice = character.voice.trim().to_string();
        if !VOICES.contains(&voice.as_str()) {
            return Err(format!("{voice} is not a story voice"));
        }
        characters.push(Character { id, name, voice });
    }
    if draft.chapters.is_empty() {
        return Err("a story needs at least one chapter".to_string());
    }
    if draft.chapters.len() > MAX_CHAPTERS {
        return Err(format!("a story takes at most {MAX_CHAPTERS} chapters"));
    }
    let mut line_ids = HashSet::new();
    let mut chapters = Vec::new();
    let mut total = 0;
    let mut number = 0;
    for (index, chapter) in draft.chapters.into_iter().enumerate() {
        let id = if chapter.id.trim().is_empty() {
            format!("ch{}", index + 1)
        } else {
            chapter.id.trim().to_string()
        };
        if !valid_id(&id) {
            return Err(format!("the chapter id {id} is not valid"));
        }
        let title = clean(&chapter.title, "a chapter title", 80)?;
        if chapter.lines.is_empty() {
            return Err(format!("the chapter {title} has no lines"));
        }
        let mut lines = Vec::new();
        for line in chapter.lines {
            number += 1;
            total += 1;
            if total > MAX_LINES {
                return Err(format!("a story takes at most {MAX_LINES} lines"));
            }
            let line_id = if line.id.trim().is_empty() {
                format!("l{number}")
            } else {
                line.id.trim().to_string()
            };
            if !valid_id(&line_id) {
                return Err(format!("the line id {line_id} is not valid"));
            }
            if !line_ids.insert(line_id.clone()) {
                return Err(format!("the line id {line_id} appears twice"));
            }
            let speaker = line.speaker.trim().to_string();
            if !character_ids.contains(&speaker) {
                return Err(format!("the line {line_id} has an unknown speaker"));
            }
            let text = clean(&line.text, "a line", MAX_TEXT)?;
            if !line.pace.is_finite() || !(0.8..=1.25).contains(&line.pace) {
                return Err(format!("the line {line_id} has an invalid pace"));
            }
            lines.push(StoryLine {
                id: line_id,
                speaker,
                text,
                pace: line.pace,
                state: PENDING.to_string(),
                out: None,
                seconds: 0.0,
                start: 0.0,
                end: 0.0,
                error: None,
            });
        }
        chapters.push(Chapter { id, title, lines });
    }
    let stamp = now();
    Ok(Story {
        id: new_id(),
        title,
        author,
        year,
        series,
        comment,
        language,
        state: DRAFT.to_string(),
        batch: None,
        error: None,
        rendered: 0,
        cached: 0,
        failed: 0,
        created_at: stamp,
        updated_at: stamp,
        characters,
        chapters,
    })
}

pub fn create(draft: Draft) -> Result<Story, String> {
    let mut story = normalize(draft)?;
    write(&mut story)?;
    Ok(story)
}

pub fn update(id: &str, draft: Draft) -> Result<Story, String> {
    let existing = read(id)?;
    let mut story = normalize(draft)?;
    story.id = existing.id;
    story.created_at = existing.created_at;
    write(&mut story)?;
    Ok(story)
}

pub fn get(id: &str) -> Result<Story, String> {
    let mut story = read(id)?;
    sync(&mut story);
    Ok(story)
}

pub fn list() -> Result<Vec<Story>, String> {
    let root = root()?;
    let mut stories = Vec::new();
    for entry in
        fs::read_dir(&root).map_err(|error| format!("cannot read {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if let Ok(mut story) = read(&id) {
            sync(&mut story);
            stories.push(story);
        }
    }
    stories.sort_by_key(|story| std::cmp::Reverse(story.created_at));
    Ok(stories)
}

pub fn start(id: &str) -> Result<Story, String> {
    let mut story = read(id)?;
    if story.state == RENDERING {
        return Err("this story is already rendering".to_string());
    }
    for character in &story.characters {
        if !voice_installed(&character.voice) {
            return Err(format!(
                "the voice {} is not installed for {}",
                character.voice, character.name
            ));
        }
    }
    let batch = crate::batch::create_story(&story.id, &story.title, &story.language)?;
    story.batch = Some(batch.id.clone());
    story.state = RENDERING.to_string();
    story.error = None;
    story.rendered = 0;
    story.cached = 0;
    story.failed = 0;
    write(&mut story)?;
    if let Err(error) = crate::batch::start(&batch.id) {
        story.state = FAILED.to_string();
        story.error = Some(error);
        write(&mut story)?;
        return Err("the story could not be queued".to_string());
    }
    Ok(story)
}

pub fn render(
    id: &str,
    batch_dir: &Path,
    out_name: &str,
    srt_name: &str,
) -> Result<StoryRender, String> {
    let mut story = read(id)?;
    let story_dir = directory(id)?;
    let lines_dir = story_dir.join("lines");
    fs::create_dir_all(&lines_dir)
        .map_err(|error| format!("cannot create {}: {error}", lines_dir.display()))?;
    let voices: std::collections::HashMap<&str, &str> = story
        .characters
        .iter()
        .map(|character| (character.id.as_str(), character.voice.as_str()))
        .collect();
    let mut lines = Vec::new();
    for chapter in &story.chapters {
        for line in &chapter.lines {
            let voice = voices
                .get(line.speaker.as_str())
                .ok_or_else(|| format!("the line {} has an unknown speaker", line.id))?;
            lines.push(serde_json::json!({
                "id": line.id,
                "text": line.text,
                "voice": voice,
                "pace": line.pace,
            }));
        }
    }
    let input = story_dir.join("render-request.json");
    let request = serde_json::json!({ "language": story.language, "lines": lines });
    fs::write(&input, request.to_string())
        .map_err(|error| format!("cannot write {}: {error}", input.display()))?;
    let report_path = batch_dir.join("story-report.json");
    let result = dub::run_lines(&dub::LinesRequest {
        input: input.clone(),
        out: batch_dir.join(out_name),
        srt: batch_dir.join(srt_name),
        report: report_path,
        lines_dir: lines_dir.clone(),
    });
    let report = match result {
        Ok(report) => report,
        Err(error) => {
            story.state = FAILED.to_string();
            story.error = Some(error.clone());
            write(&mut story)?;
            fs::remove_file(&input).ok();
            return Err(error);
        }
    };
    let rendered: std::collections::HashMap<&str, &dub::Line> = report
        .lines
        .iter()
        .filter_map(|line| line.id.as_deref().map(|id| (id, line)))
        .collect();
    for chapter in &mut story.chapters {
        for line in &mut chapter.lines {
            if let Some(message) = rendered.get(line.id.as_str()) {
                line.out = message.file.clone();
                line.seconds = (message.end - message.start).max(0.0);
                line.start = message.start;
                line.end = message.end;
                line.state = if message.failed { FAILED } else { DONE }.to_string();
                line.error = message.failure.clone();
            } else {
                line.state = FAILED.to_string();
                line.error = Some("the render did not return this line".to_string());
            }
        }
    }
    story.rendered = report.rendered;
    story.cached = report.cached;
    story.failed = report.failed;
    story.state = if report.failed == 0 {
        DONE.to_string()
    } else {
        FAILED.to_string()
    };
    story.error = None;
    write(&mut story)?;
    fs::remove_file(&input).ok();
    Ok(StoryRender {
        seconds: report.seconds,
        lines: report.lines.len(),
    })
}

pub fn audiobook(id: &str) -> Result<(PathBuf, String), String> {
    let story = read(id)?;
    if story.state != DONE {
        return Err("render the story before packaging the audiobook".to_string());
    }
    let batch = story
        .batch
        .as_deref()
        .ok_or_else(|| "the story has no render to package".to_string())?;
    let (source, _) = crate::batch::download(batch, "story")?;
    let directory = directory(id)?;
    let target = directory.join("story.m4b");
    let metadata = directory.join("story.ffmeta");
    crate::audiobook::package(&story, &source, &metadata, &target)?;
    fs::remove_file(&metadata).ok();
    Ok((target, "story.m4b".to_string()))
}

pub fn line_audio(id: &str, line_id: &str) -> Result<(PathBuf, String), String> {
    let story = read(id)?;
    let line = story
        .chapters
        .iter()
        .flat_map(|chapter| &chapter.lines)
        .find(|line| line.id == line_id)
        .ok_or_else(|| "this line is not in the story".to_string())?;
    let name = line
        .out
        .as_deref()
        .ok_or_else(|| "this line has no audio yet".to_string())?;
    if !valid_id(name) && !name.ends_with(".wav") {
        return Err("the story contains an unsafe audio name".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("the story contains an unsafe audio name".to_string());
    }
    let path = directory(id)?.join("lines").join(name);
    if !path.is_file() {
        return Err("the audio for this line is missing".to_string());
    }
    Ok((path, format!("{}-{name}", story.id)))
}

fn sync(story: &mut Story) {
    let Some(batch_id) = story.batch.clone() else {
        return;
    };
    let Ok(batch) = crate::batch::get(&batch_id) else {
        return;
    };
    let state = match batch.state.as_str() {
        crate::batch::DONE => DONE,
        crate::batch::FAILED => FAILED,
        _ => RENDERING,
    };
    if story.state != state {
        story.state = state.to_string();
        if state != FAILED {
            story.error = None;
        }
        write(story).ok();
    }
}

fn root() -> Result<PathBuf, String> {
    let root = crate::config::data_root().join("stories");
    fs::create_dir_all(&root)
        .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
    Ok(root)
}

fn directory(id: &str) -> Result<PathBuf, String> {
    if !valid_id(id) {
        return Err("the story id is not valid".to_string());
    }
    let directory = root()?.join(id);
    fs::create_dir_all(&directory)
        .map_err(|error| format!("cannot create {}: {error}", directory.display()))?;
    Ok(directory)
}

fn read(id: &str) -> Result<Story, String> {
    let path = directory(id)?.join("story.json");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn write(story: &mut Story) -> Result<(), String> {
    story.updated_at = now();
    let directory = directory(&story.id)?;
    let text = serde_json::to_string_pretty(story)
        .map_err(|error| format!("cannot encode the story: {error}"))?;
    let path = directory.join("story.json");
    fs::write(&path, text).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> Draft {
        Draft {
            title: "A story".to_string(),
            author: None,
            year: None,
            series: None,
            comment: None,
            language: "pt".to_string(),
            characters: vec![
                Character {
                    id: String::new(),
                    name: "Ana".to_string(),
                    voice: VOICES[0].to_string(),
                },
                Character {
                    id: String::new(),
                    name: "Beto".to_string(),
                    voice: VOICES[1].to_string(),
                },
                Character {
                    id: String::new(),
                    name: "Carla".to_string(),
                    voice: VOICES[2].to_string(),
                },
            ],
            chapters: vec![Chapter {
                id: String::new(),
                title: "One".to_string(),
                lines: vec![
                    StoryLine {
                        id: String::new(),
                        speaker: "c1".to_string(),
                        text: "Olá.".to_string(),
                        pace: 1.0,
                        state: String::new(),
                        out: None,
                        seconds: 0.0,
                        start: 0.0,
                        end: 0.0,
                        error: None,
                    },
                    StoryLine {
                        id: String::new(),
                        speaker: "c2".to_string(),
                        text: "Olá.".to_string(),
                        pace: 1.0,
                        state: String::new(),
                        out: None,
                        seconds: 0.0,
                        start: 0.0,
                        end: 0.0,
                        error: None,
                    },
                ],
            }],
        }
    }

    #[test]
    fn cast_ids_and_speakers_are_resolved() {
        let story = normalize(draft()).unwrap();
        assert_eq!(story.characters[0].id, "c1");
        assert_eq!(story.chapters[0].lines[1].speaker, "c2");
        assert_eq!(story.chapters[0].lines[1].id, "l2");
    }

    #[test]
    fn an_unknown_speaker_is_rejected() {
        let mut request = draft();
        request.chapters[0].lines[0].speaker = "c9".to_string();
        assert!(normalize(request).is_err());
    }

    #[test]
    fn a_repeated_line_id_is_rejected() {
        let mut request = draft();
        request.chapters[0].lines[1].id = "l1".to_string();
        assert!(normalize(request).is_err());
    }

    #[test]
    fn a_voice_outside_the_cast_list_is_rejected() {
        let mut request = draft();
        request.characters[0].voice = "missing".to_string();
        assert!(normalize(request).is_err());
    }
}
