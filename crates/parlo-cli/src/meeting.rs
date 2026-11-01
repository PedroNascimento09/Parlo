use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
        id: format!("m{started_at}"),
        title: title.trim().to_string(),
        started_at,
        segments,
    })
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
        let nanos = now();
        let folder = std::env::temp_dir().join(format!("parlo-meeting-{name}-{nanos}"));
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
    fn an_id_cannot_walk_out_of_the_folder() {
        let folder = scratch("id");
        assert!(path_for(&folder, "../escape").is_err());
        assert!(path_for(&folder, "a/b").is_err());
        assert!(path_for(&folder, "").is_err());
        assert!(path_for(&folder, "m123-abc").is_ok());
        fs::remove_dir_all(&folder).unwrap();
    }
}
