use std::fs;
use std::path::{Path, PathBuf};

use crate::dub;
use crate::store::Store as ProjectStore;

use super::header;

pub(super) struct LoadedProject {
    pub(super) project: crate::store::DubProject,
    pub(super) store: ProjectStore,
}

pub(super) fn load_requested_project(
    headers: &[(String, String)],
) -> Result<Option<LoadedProject>, (&'static str, String)> {
    let id = header(headers, "x-parlo-project").unwrap_or_default();
    if id.is_empty() {
        return Ok(None);
    }
    let store = ProjectStore::open_default().map_err(|error| {
        (
            "Internal Server Error",
            format!("cannot open the project store: {error}"),
        )
    })?;
    match store.load_dub_project(&id) {
        Ok(Some(project)) => {
            if project.segments.is_empty() {
                return Err((
                    "Bad Request",
                    "the imported project has no cues".to_string(),
                ));
            }
            Ok(Some(LoadedProject { project, store }))
        }
        Ok(None) => Err(("Not Found", "project not found".to_string())),
        Err(error) => Err((
            "Internal Server Error",
            format!("cannot load the project: {error}"),
        )),
    }
}

pub(super) fn write_cues(
    directory: &Path,
    id: &str,
    project: Option<&crate::store::DubProject>,
) -> Result<Option<PathBuf>, String> {
    let Some(project) = project else {
        return Ok(None);
    };
    let path = directory.join(format!("{id}-cues.json"));
    let document =
        cue_document(project).map_err(|error| format!("cannot encode the cues: {error}"))?;
    fs::write(&path, document).map_err(|error| format!("cannot store the cues: {error}"))?;
    Ok(Some(path))
}

const MAX_SEGMENT_TEXT: usize = 10_000;

#[derive(serde::Deserialize)]
pub(super) struct ProjectEdit {
    #[serde(default)]
    pub(super) cast: std::collections::BTreeMap<String, String>,
    pub(super) segments: Vec<SegmentEdit>,
}

#[derive(serde::Deserialize)]
pub(super) struct SegmentEdit {
    pub(super) id: String,
    pub(super) start_ms: u64,
    pub(super) end_ms: u64,
    #[serde(default)]
    pub(super) speaker: Option<String>,
    #[serde(default)]
    pub(super) voice_id: Option<String>,
    #[serde(default)]
    pub(super) pace: Option<f64>,
    pub(super) source_text: String,
}

pub(super) fn apply_project_edits(
    project: &mut crate::store::DubProject,
    edits: &[SegmentEdit],
    cast: &std::collections::BTreeMap<String, String>,
) -> Result<(), String> {
    if project.segments.is_empty() {
        return Err("the project has no segments to edit".to_string());
    }
    if edits.len() != project.segments.len() {
        return Err("send every segment exactly once".to_string());
    }
    for (speaker, voice) in cast {
        if speaker.trim().is_empty() || speaker.chars().count() > 120 {
            return Err("a speaker needs a name of up to 120 characters".to_string());
        }
        if !known_voice(voice) {
            return Err(format!("{voice} is not a voice Parlo can cast"));
        }
    }
    let mut seen = std::collections::HashSet::new();
    for edit in edits {
        if edit.id.is_empty() {
            return Err("a segment has no id".to_string());
        }
        if !seen.insert(edit.id.as_str()) {
            return Err(format!("segment {} appears twice", edit.id));
        }
        if edit.start_ms >= edit.end_ms {
            return Err(format!("segment {} ends before it starts", edit.id));
        }
        if edit.source_text.trim().is_empty() {
            return Err(format!("segment {} has no text", edit.id));
        }
        if edit.source_text.chars().count() > MAX_SEGMENT_TEXT {
            return Err(format!("segment {} has too much text", edit.id));
        }
        if edit
            .speaker
            .as_deref()
            .map(str::trim)
            .map(|speaker| speaker.chars().count() > 120)
            .unwrap_or(false)
        {
            return Err(format!(
                "segment {} has a speaker name that is too long",
                edit.id
            ));
        }
        if let Some(voice) = edit.voice_id.as_deref().filter(|voice| !voice.is_empty()) {
            if !known_voice(voice) {
                return Err(format!("{voice} is not a voice Parlo can cast"));
            }
        }
        if let Some(pace) = edit.pace {
            if !pace.is_finite() || !(0.8..=1.25).contains(&pace) {
                return Err(format!(
                    "segment {} needs a pace between 0.8 and 1.25",
                    edit.id
                ));
            }
        }
        if !project.segments.iter().any(|segment| segment.id == edit.id) {
            return Err(format!("segment {} is not in this project", edit.id));
        }
    }
    let before: std::collections::HashMap<String, Option<String>> = project
        .segments
        .iter()
        .map(|segment| {
            (
                segment.id.clone(),
                effective_voice(project, segment).map(ToOwned::to_owned),
            )
        })
        .collect();
    for segment in &mut project.segments {
        let edit = edits
            .iter()
            .find(|edit| edit.id == segment.id)
            .ok_or_else(|| format!("segment {} is missing", segment.id))?;
        let speaker = edit
            .speaker
            .as_deref()
            .map(str::trim)
            .filter(|speaker| !speaker.is_empty())
            .map(ToOwned::to_owned);
        let source_changed = segment.source_text != edit.source_text;
        let pace = match edit.pace {
            Some(pace) if (pace - 1.0).abs() > f64::EPSILON => Some(pace),
            Some(_) => None,
            None => segment.pace,
        };
        let pace_changed = segment.pace != pace;
        let text_changed = segment.start_ms != edit.start_ms
            || segment.end_ms != edit.end_ms
            || segment.speaker != speaker
            || source_changed;
        segment.start_ms = edit.start_ms;
        segment.end_ms = edit.end_ms;
        segment.speaker = speaker;
        segment.pace = pace;
        if source_changed {
            let previous = segment.source_text.clone();
            if segment.revisions.last() != Some(&previous) {
                segment.revisions.push(previous);
            }
            segment.source_text = edit.source_text.clone();
            segment.original_text = segment.revisions.first().cloned();
        }
        segment.voice_id = edit
            .voice_id
            .clone()
            .filter(|voice| !voice.trim().is_empty());
        let after = segment.voice_id.clone().or_else(|| {
            segment
                .speaker
                .as_deref()
                .and_then(|speaker| cast.get(speaker))
                .cloned()
        });
        if text_changed || pace_changed || before.get(&segment.id).cloned().flatten() != after {
            if text_changed {
                segment.target_text = None;
                segment.translations.clear();
            }
            segment.state = "edited".to_string();
        }
    }
    project.cast = cast.clone();
    Ok(())
}

#[derive(serde::Serialize)]
pub(super) struct DubCue<'a> {
    id: &'a str,
    start_ms: u64,
    end_ms: u64,
    source: &'a str,
    speaker: Option<&'a str>,
    voice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pace: Option<f64>,
}

#[derive(serde::Serialize)]
pub(super) struct DubCueDocument<'a> {
    cues: Vec<DubCue<'a>>,
}

pub(super) fn cue_document(
    project: &crate::store::DubProject,
) -> Result<String, serde_json::Error> {
    let cues: Vec<DubCue<'_>> = project
        .segments
        .iter()
        .map(|segment| DubCue {
            id: segment.id.as_str(),
            start_ms: segment.start_ms,
            end_ms: segment.end_ms,
            source: segment.source_text.as_str(),
            speaker: segment.speaker.as_deref(),
            voice: effective_voice(project, segment),
            pace: segment.pace,
        })
        .collect();
    serde_json::to_string(&DubCueDocument { cues })
}

pub(super) fn known_voice(id: &str) -> bool {
    crate::registry::model(id)
        .map(|spec| spec.kind == "voice")
        .unwrap_or(false)
}

pub(super) fn effective_voice<'a>(
    project: &'a crate::store::DubProject,
    segment: &'a crate::store::DubSegment,
) -> Option<&'a str> {
    segment.voice_id.as_deref().or_else(|| {
        segment
            .speaker
            .as_deref()
            .and_then(|speaker| project.cast.get(speaker))
            .map(String::as_str)
    })
}

pub(super) fn project_json(
    project: &crate::store::DubProject,
) -> Result<String, serde_json::Error> {
    let installed: std::collections::HashSet<&str> = crate::models::status()
        .into_iter()
        .filter(|model| model.installed)
        .map(|model| model.id)
        .collect();
    let voices: Vec<serde_json::Value> = crate::registry::MODELS
        .iter()
        .filter(|spec| spec.kind == "voice")
        .map(|spec| {
            serde_json::json!({
                "id": spec.id,
                "name": spec.name,
                "installed": installed.contains(spec.id),
            })
        })
        .collect();
    let mut languages: Vec<&str> = Vec::new();
    for spec in crate::registry::MODELS
        .iter()
        .filter(|spec| spec.kind == "translation")
    {
        if spec.languages.first() == Some(&project.from.as_str()) {
            if let Some(target) = spec.languages.get(1) {
                if !languages.contains(target) {
                    languages.push(target);
                }
            }
        }
    }
    if !languages.contains(&project.to.as_str()) {
        languages.push(&project.to);
    }
    let mut document = serde_json::to_value(project)?;
    document["voices"] = serde_json::Value::Array(voices);
    document["languages"] = serde_json::json!(languages);
    serde_json::to_string(&document)
}

pub(super) fn apply_dub(
    project: &mut crate::store::DubProject,
    lines: &[dub::Line],
    language: &str,
) {
    let primary = language == project.to;
    for line in lines {
        let Some(id) = line.id.as_deref() else {
            continue;
        };
        if let Some(segment) = project.segments.iter_mut().find(|segment| segment.id == id) {
            if !line.translated.is_empty() {
                segment
                    .translations
                    .insert(language.to_string(), line.translated.clone());
                if primary {
                    segment.target_text = Some(line.translated.clone());
                }
            }
            segment.state = if line.failed {
                "failed".to_string()
            } else if line.skipped {
                "skipped".to_string()
            } else {
                "dubbed".to_string()
            };
        }
    }
    if !project.targets.iter().any(|target| target == language) {
        project.targets.push(language.to_string());
    }
}
