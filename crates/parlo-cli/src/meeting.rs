use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT: AtomicU64 = AtomicU64::new(0);

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
    pub evidence: String,
    pub inferred: bool,
    pub applied_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub at_ms: u64,
    pub speaker: String,
    pub text: String,
    #[serde(default)]
    pub name: Option<Name>,
}

impl Segment {
    pub fn spoken_by(&self) -> &str {
        self.name
            .as_ref()
            .map(|name| name.name.as_str())
            .unwrap_or(&self.speaker)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub started_at: u64,
    #[serde(default)]
    pub segments: Vec<Segment>,
    #[serde(default)]
    pub speaker_hint: Option<usize>,
}

impl Meeting {
    pub fn speakers(&self) -> Vec<String> {
        let mut speakers: Vec<String> = Vec::new();
        for segment in &self.segments {
            let speaker = segment.spoken_by().to_string();
            if !speakers.contains(&speaker) {
                speakers.push(speaker);
            }
        }
        speakers
    }
}

pub fn folder() -> PathBuf {
    crate::config::data_root().join("meetings")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default()
}

fn new_id() -> String {
    let counter = NEXT.fetch_add(1, Ordering::Relaxed);
    format!("m{}-{}-{counter}", now(), std::process::id())
}

fn path_for(folder: &Path, id: &str) -> Result<PathBuf, String> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-');
    if !valid {
        return Err("a meeting id is made of letters, digits and dashes".to_string());
    }
    Ok(folder.join(format!("{id}.json")))
}

pub fn save(meeting: &Meeting) -> Result<(), String> {
    save_in(&folder(), meeting)
}

pub fn load(id: &str) -> Result<Meeting, String> {
    load_in(&folder(), id)
}

pub fn list() -> Vec<Meeting> {
    list_in(&folder())
}

pub fn delete(id: &str) -> Result<(), String> {
    delete_in(&folder(), id)
}

pub fn clear() -> Result<(), String> {
    clear_in(&folder())
}

pub fn name_speaker(
    meeting: &mut Meeting,
    speaker: &str,
    name: Option<&str>,
    evidence: &str,
) -> Result<usize, String> {
    let label = name.map(|value| value.trim()).unwrap_or("");
    let spoken = meeting
        .segments
        .iter()
        .any(|segment| segment.speaker == speaker);
    if !spoken {
        return Err(format!("no line is spoken by {speaker}"));
    }
    if !label.is_empty() && evidence.trim().is_empty() {
        return Err("a name needs the sentence that supports it".to_string());
    }
    let applied = if label.is_empty() {
        None
    } else {
        Some(Name {
            name: label.to_string(),
            evidence: evidence.trim().to_string(),
            inferred: true,
            applied_at: now(),
        })
    };
    let mut touched = 0;
    for segment in meeting
        .segments
        .iter_mut()
        .filter(|segment| segment.speaker == speaker)
    {
        segment.name = applied.clone();
        touched += 1;
    }
    Ok(touched)
}

pub fn import(title: &str, text: &str) -> Result<Meeting, String> {
    let segments = parse(text);
    if segments.is_empty() {
        return Err("no line carries a speaker and a sentence".to_string());
    }
    let started_at = now();
    Ok(Meeting {
        id: new_id(),
        title: title.trim().to_string(),
        started_at,
        segments,
        speaker_hint: None,
    })
}

pub fn start_with(title: &str, hint: Option<usize>) -> Result<Meeting, String> {
    if let Some(count) = hint {
        if !(1..=20).contains(&count) {
            return Err("a speaker count must be between 1 and 20".to_string());
        }
    }
    Ok(build(title, hint))
}

fn build(title: &str, speaker_hint: Option<usize>) -> Meeting {
    let label = title.trim();
    Meeting {
        id: new_id(),
        title: if label.is_empty() {
            "Meeting".to_string()
        } else {
            label.to_string()
        },
        started_at: now(),
        segments: Vec::new(),
        speaker_hint,
    }
}

pub fn append(
    meeting: &mut Meeting,
    speaker: &str,
    text: &str,
    at_ms: u64,
) -> Result<usize, String> {
    let spoken = text.trim();
    if spoken.is_empty() {
        return Err("a segment needs a sentence".to_string());
    }
    let label = speaker.trim();
    meeting.segments.push(Segment {
        at_ms,
        speaker: if label.is_empty() {
            "Me".to_string()
        } else {
            label.to_string()
        },
        text: spoken.to_string(),
        name: None,
    });
    Ok(meeting.segments.len())
}

fn parse(text: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut last: Option<String> = None;
    for line in text.lines() {
        let clean = line.trim();
        if clean.is_empty() {
            continue;
        }
        let (speaker, rest) = match clean.split_once(':') {
            Some((speaker, rest)) if !speaker.trim().is_empty() && speaker.trim().len() <= 40 => {
                (speaker.trim().to_string(), rest.trim())
            }
            _ => (
                last.clone().unwrap_or_else(|| "Speaker 1".to_string()),
                clean,
            ),
        };
        let (at_ms, spoken) = split_time(rest);
        last = Some(speaker.clone());
        segments.push(Segment {
            at_ms,
            speaker,
            text: spoken,
            name: None,
        });
    }
    segments
}

fn split_time(text: &str) -> (u64, String) {
    let Some(rest) = text.strip_prefix('[') else {
        return (0, text.to_string());
    };
    let Some((stamp, spoken)) = rest.split_once(']') else {
        return (0, text.to_string());
    };
    let parts: Vec<u64> = stamp
        .split(':')
        .map(|part| part.trim().parse::<u64>())
        .map(|part| part.map_err(|_| ()))
        .collect::<Result<Vec<u64>, ()>>()
        .unwrap_or_default();
    let total = match parts.as_slice() {
        [minutes, seconds] if *seconds < 60 => minutes * 60 + seconds,
        [hours, minutes, seconds] if *seconds < 60 && *minutes < 60 => {
            hours * 3600 + minutes * 60 + seconds
        }
        _ => return (0, text.to_string()),
    };
    (total * 1000, spoken.trim().to_string())
}

fn save_in(folder: &Path, meeting: &Meeting) -> Result<(), String> {
    fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    let path = path_for(folder, &meeting.id)?;
    let text = serde_json::to_string_pretty(meeting).map_err(|error| error.to_string())?;
    fs::write(&path, text + "\n").map_err(|error| format!("{}: {error}", path.display()))
}

fn load_in(folder: &Path, id: &str) -> Result<Meeting, String> {
    let path = path_for(folder, id)?;
    let text = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("{} is not a readable meeting: {error}", path.display()))
}

fn list_in(folder: &Path) -> Vec<Meeting> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut meetings: Vec<Meeting> = entries
        .flatten()
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|extension| extension == "json")
                .unwrap_or(false)
        })
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .filter_map(|text| serde_json::from_str::<Meeting>(&text).ok())
        .collect();
    meetings.sort_by_key(|meeting| std::cmp::Reverse(meeting.started_at));
    meetings
}

fn delete_in(folder: &Path, id: &str) -> Result<(), String> {
    let path = path_for(folder, id)?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}

fn clear_in(folder: &Path) -> Result<(), String> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .map(|extension| extension == "json")
            .unwrap_or(false)
        {
            fs::remove_file(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("parlo-meeting-{name}-{}", new_id()));
        fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn sample() -> Meeting {
        import(
            "Product Sync",
            "Jorge: Olá, tudo bem, Miguel?\n\
             Miguel: Sim, tudo bem.\n\
             Miguel: [00:12] Posso tratar disso amanhã.\n\
             sem marcador nenhum",
        )
        .unwrap()
    }

    #[test]
    fn two_imports_never_share_an_id() {
        let first = sample();
        let second = sample();
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn the_parser_keeps_the_markers_and_the_timestamp() {
        let meeting = sample();
        assert_eq!(meeting.segments.len(), 4);
        assert_eq!(meeting.segments[0].speaker, "Jorge");
        assert_eq!(meeting.segments[2].at_ms, 12_000);
        assert_eq!(meeting.segments[3].speaker, "Miguel");
    }

    #[test]
    fn a_name_applies_to_every_earlier_segment() {
        let mut meeting = sample();
        let touched =
            name_speaker(&mut meeting, "Miguel", Some("Miguel"), "Sim, tudo bem.").unwrap();
        assert_eq!(touched, 3);
        assert_eq!(meeting.speakers(), vec!["Jorge", "Miguel"]);
        assert_eq!(meeting.segments[1].spoken_by(), "Miguel");
        assert_eq!(meeting.segments[1].speaker, "Miguel");
        assert_eq!(
            meeting.segments[1].name.as_ref().unwrap().evidence,
            "Sim, tudo bem."
        );
    }

    #[test]
    fn a_name_without_evidence_is_refused() {
        let mut meeting = sample();
        assert!(name_speaker(&mut meeting, "Miguel", Some("Miguel"), "  ").is_err());
        assert!(meeting.segments[1].name.is_none());
    }

    #[test]
    fn clearing_a_name_goes_back_to_the_marker() {
        let mut meeting = sample();
        name_speaker(&mut meeting, "Jorge", Some("Jorge Silva"), "Olá").unwrap();
        name_speaker(&mut meeting, "Jorge", None, "").unwrap();
        assert_eq!(meeting.segments[0].spoken_by(), "Jorge");
        assert!(meeting.segments[0].name.is_none());
    }

    #[test]
    fn naming_a_speaker_that_never_spoke_is_an_error() {
        let mut meeting = sample();
        assert!(name_speaker(&mut meeting, "Speaker 9", Some("Ana"), "olá").is_err());
    }

    #[test]
    fn a_meeting_round_trips_through_the_folder() {
        let folder = scratch("roundtrip");
        let meeting = sample();
        save_in(&folder, &meeting).unwrap();
        assert_eq!(load_in(&folder, &meeting.id).unwrap(), meeting);
        assert_eq!(list_in(&folder).len(), 1);
        assert_eq!(list_in(&folder)[0].title, "Product Sync");
        delete_in(&folder, &meeting.id).unwrap();
        assert!(list_in(&folder).is_empty());
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn clearing_removes_every_meeting() {
        let folder = scratch("clear");
        save_in(&folder, &sample()).unwrap();
        save_in(&folder, &sample()).unwrap();
        assert_eq!(list_in(&folder).len(), 2);
        clear_in(&folder).unwrap();
        assert!(list_in(&folder).is_empty());
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn a_live_meeting_starts_empty_and_named() {
        let meeting = start_with("  Design review  ", None).unwrap();
        assert_eq!(meeting.title, "Design review");
        assert!(meeting.segments.is_empty());
        assert_eq!(start_with("   ", None).unwrap().title, "Meeting");
    }

    #[test]
    fn a_meeting_records_the_speaker_count_it_was_given() {
        assert_eq!(start_with("Retro", Some(3)).unwrap().speaker_hint, Some(3));
        assert_eq!(start_with("Retro", None).unwrap().speaker_hint, None);
        assert!(start_with("Retro", Some(0)).is_err());
        assert!(start_with("Retro", Some(21)).is_err());
    }

    #[test]
    fn the_speaker_count_survives_a_restart() {
        let folder = scratch("hint");
        let meeting = start_with("Planning", Some(4)).unwrap();
        save_in(&folder, &meeting).unwrap();
        assert_eq!(load_in(&folder, &meeting.id).unwrap().speaker_hint, Some(4));
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn a_segment_is_appended_with_the_speaker_and_the_moment() {
        let mut meeting = start_with("Retro", None).unwrap();
        assert_eq!(
            append(&mut meeting, "", "Primeira frase.", 1500).unwrap(),
            1
        );
        append(&mut meeting, "Ana", "Segunda frase.", 4200).unwrap();
        assert_eq!(meeting.segments[0].speaker, "Me");
        assert_eq!(meeting.segments[0].at_ms, 1500);
        assert_eq!(meeting.segments[1].speaker, "Ana");
        assert_eq!(meeting.speakers(), vec!["Me", "Ana"]);
    }

    #[test]
    fn an_empty_segment_is_refused() {
        let mut meeting = start_with("Retro", None).unwrap();
        assert!(append(&mut meeting, "Me", "   ", 0).is_err());
        assert!(meeting.segments.is_empty());
    }

    #[test]
    fn a_live_meeting_round_trips_through_the_folder() {
        let folder = scratch("live");
        let mut meeting = start_with("Planning", None).unwrap();
        append(&mut meeting, "Me", "Fica para sexta.", 2000).unwrap();
        save_in(&folder, &meeting).unwrap();
        let kept = load_in(&folder, &meeting.id).unwrap();
        assert_eq!(kept.segments.len(), 1);
        assert_eq!(kept.segments[0].text, "Fica para sexta.");
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn an_id_cannot_walk_out_of_the_folder() {
        let folder = scratch("id");
        assert!(path_for(&folder, "../escape").is_err());
        assert!(path_for(&folder, "a/b").is_err());
        assert!(path_for(&folder, "").is_err());
        assert!(path_for(&folder, "m123-abc").is_ok());
        fs::remove_dir_all(&folder).unwrap();
    }
}
