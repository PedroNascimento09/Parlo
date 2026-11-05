use std::path::Path;

use crate::store::{DubProject, DubSegment};

const MAX_CUE_TEXT: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cue {
    pub start_ms: u64,
    pub end_ms: u64,
    pub speaker: Option<String>,
    pub text: String,
}

pub fn project(
    id: String,
    name: &str,
    bytes: &[u8],
    from: &str,
    to: &str,
) -> Result<DubProject, String> {
    let cues = parse(name, bytes)?;
    let title = Path::new(name)
        .file_stem()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Subtitles");
    let mut project = DubProject::new(id, title.to_string(), from, to, Some(name.to_string()));
    for (position, cue) in cues.into_iter().enumerate() {
        project.segments.push(DubSegment {
            id: format!("{}-{position}", project.id),
            position,
            start_ms: cue.start_ms,
            end_ms: cue.end_ms,
            speaker: cue.speaker,
            voice_id: None,
            translations: std::collections::BTreeMap::new(),
            source_text: cue.text,
            original_text: None,
            target_text: None,
            state: "imported".to_string(),
        });
    }
    Ok(project)
}

pub fn parse(name: &str, bytes: &[u8]) -> Result<Vec<Cue>, String> {
    let lower = name.to_ascii_lowercase();
    let vtt = lower.ends_with(".vtt");
    if !vtt && !lower.ends_with(".srt") {
        return Err("choose an .srt or .vtt subtitle file".to_string());
    }
    let text = decode(bytes)?;
    let normalized = normalize(&text);
    let body = if vtt {
        strip_vtt_header(&normalized)
    } else {
        normalized.as_str()
    };
    let mut cues = Vec::new();
    for (number, block) in body.split("\n\n").enumerate() {
        if block.trim().is_empty() || (vtt && is_metadata(block)) {
            continue;
        }
        if let Some(cue) = parse_block(block, number + 1)? {
            cues.push(cue);
        }
    }
    if cues.is_empty() {
        return Err("the subtitle file has no cues with text".to_string());
    }
    cues.sort_by_key(|cue| cue.start_ms);
    Ok(cues)
}

fn decode(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("the subtitle file is empty".to_string());
    }
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8(rest.to_vec())
            .map_err(|_| "the subtitle file is not valid UTF-8 text".to_string());
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return decode_utf16(rest, true);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return decode_utf16(rest, false);
    }
    String::from_utf8(bytes.to_vec())
        .map_err(|_| "the subtitle file is not valid UTF-8 text".to_string())
}

fn decode_utf16(bytes: &[u8], little: bool) -> Result<String, String> {
    if bytes.len() % 2 != 0 {
        return Err("the subtitle file has an incomplete UTF-16 character".to_string());
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| {
            if little {
                u16::from_le_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_be_bytes([chunk[0], chunk[1]])
            }
        })
        .collect();
    String::from_utf16(&units).map_err(|_| "the subtitle file is not valid UTF-16 text".to_string())
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

fn strip_vtt_header(text: &str) -> &str {
    let after_header = text
        .find('\n')
        .map(|index| &text[index + 1..])
        .unwrap_or("");
    let after_header = after_header.trim_start_matches('\n');
    if after_header
        .lines()
        .next()
        .map(|line| line.contains("-->") || line.trim().is_empty())
        .unwrap_or(true)
    {
        return after_header;
    }
    after_header
        .split_once("\n\n")
        .map(|(_, body)| body)
        .unwrap_or(after_header)
}

fn is_metadata(block: &str) -> bool {
    let first = block.lines().next().unwrap_or_default().trim_start();
    first.starts_with("NOTE")
        || first.starts_with("STYLE")
        || first.starts_with("REGION")
        || first.starts_with("WEBVTT")
}

fn parse_block(block: &str, number: usize) -> Result<Option<Cue>, String> {
    let lines: Vec<&str> = block.lines().collect();
    let Some(time_index) = lines.iter().position(|line| line.contains("-->")) else {
        return Err(format!("cue {number} has no start and end time"));
    };
    if time_index > 1 {
        return Err(format!(
            "cue {number} has more than one line before its time"
        ));
    }
    let time_line = lines[time_index].trim();
    let Some((start_text, end_text)) = time_line.split_once("-->") else {
        return Err(format!("cue {number} has no start and end time"));
    };
    let start_ms =
        parse_time(start_text.trim()).map_err(|error| format!("cue {number}: {error}"))?;
    let end_ms = end_text
        .split_whitespace()
        .next()
        .map(parse_time)
        .transpose()
        .map_err(|error| format!("cue {number}: {error}"))?
        .ok_or_else(|| format!("cue {number} has no end time"))?;
    if end_ms <= start_ms {
        return Err(format!("cue {number} ends before it starts"));
    }
    let mut body: Vec<&str> = lines[time_index + 1..].to_vec();
    while body
        .last()
        .map(|line| line.trim().is_empty())
        .unwrap_or(false)
    {
        body.pop();
    }
    let (speaker, text) = split_speaker(&body);
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    if text.chars().count() > MAX_CUE_TEXT {
        return Err(format!(
            "cue {number} is longer than {MAX_CUE_TEXT} characters"
        ));
    }
    Ok(Some(Cue {
        start_ms,
        end_ms,
        speaker,
        text: text.to_string(),
    }))
}

fn parse_time(value: &str) -> Result<u64, String> {
    let (clock, fraction) = match value.rsplit_once('.') {
        Some((clock, fraction)) => (clock, Some(fraction)),
        None => match value.rsplit_once(',') {
            Some((clock, fraction)) => (clock, Some(fraction)),
            None => (value, None),
        },
    };
    let parts: Vec<&str> = clock.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err(format!("{value} is not a timestamp"));
    }
    let mut numbers = Vec::with_capacity(parts.len());
    for part in parts {
        if part.is_empty() || !part.chars().all(|character| character.is_ascii_digit()) {
            return Err(format!("{value} is not a timestamp"));
        }
        numbers.push(
            part.parse::<u64>()
                .map_err(|_| format!("{value} is not a timestamp"))?,
        );
    }
    let (hours, minutes, seconds) = match numbers.as_slice() {
        [minutes, seconds] => (0, *minutes, *seconds),
        [hours, minutes, seconds] => (*hours, *minutes, *seconds),
        _ => return Err(format!("{value} is not a timestamp")),
    };
    if minutes >= 60 || seconds >= 60 {
        return Err(format!("{value} is not a timestamp"));
    }
    let millis = match fraction {
        Some(fraction)
            if !fraction.is_empty()
                && fraction.len() <= 3
                && fraction.chars().all(|character| character.is_ascii_digit()) =>
        {
            let padded = format!("{fraction:0<3}");
            padded
                .parse::<u64>()
                .map_err(|_| format!("{value} is not a timestamp"))?
        }
        Some(_) => return Err(format!("{value} is not a timestamp")),
        None => 0,
    };
    Ok(((hours * 60 + minutes) * 60 + seconds) * 1000 + millis)
}

fn split_speaker(lines: &[&str]) -> (Option<String>, String) {
    let mut rows: Vec<&str> = lines.to_vec();
    while rows
        .first()
        .map(|line| line.trim().is_empty())
        .unwrap_or(false)
    {
        rows.remove(0);
    }
    if rows.is_empty() {
        return (None, String::new());
    }
    let (speaker, first_text) = extract_prefix(rows[0]);
    let mut body = Vec::new();
    if !first_text.trim().is_empty() {
        body.push(first_text.trim().to_string());
    }
    body.extend(rows[1..].iter().map(|line| strip_tags(line)));
    (speaker, body.join("\n").trim().to_string())
}

fn extract_prefix(line: &str) -> (Option<String>, String) {
    if let Some(found) = bracket_speaker(line) {
        return found;
    }
    if let Some(found) = voice_tag(line) {
        return found;
    }
    if let Some(found) = labelled_speaker(line) {
        return found;
    }
    (None, strip_tags(line))
}

fn bracket_speaker(line: &str) -> Option<(Option<String>, String)> {
    let line = line.trim();
    let close = line.strip_prefix('[')?.find(']')?;
    let name = line[1..close + 1].trim();
    if name.is_empty() {
        return None;
    }
    let rest = line[close + 2..].trim_start();
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim_start();
    Some((Some(name.to_string()), strip_tags(rest)))
}

fn voice_tag(line: &str) -> Option<(Option<String>, String)> {
    let line = line.trim();
    let lower = line.to_ascii_lowercase();
    if !lower.starts_with("<v") {
        return None;
    }
    let after_v = line[2..].chars().next()?;
    if !after_v.is_whitespace() && after_v != '.' {
        return None;
    }
    let close = line.find('>')?;
    let name = line[2..close].split_whitespace().last()?.trim();
    if name.is_empty() {
        return None;
    }
    Some((Some(name.to_string()), strip_tags(&line[close + 1..])))
}

fn labelled_speaker(line: &str) -> Option<(Option<String>, String)> {
    let (name, rest) = line.split_once(':')?;
    let name = name.trim();
    if name.eq_ignore_ascii_case("voice") || name.eq_ignore_ascii_case("speaker") {
        let value = rest.trim();
        if value.is_empty() {
            return None;
        }
        if let Some((inner, text)) = value.split_once(':') {
            if is_name(inner) && !text.trim().is_empty() {
                return Some((Some(inner.trim().to_string()), strip_tags(text)));
            }
        }
        return Some((Some(value.to_string()), String::new()));
    }
    if is_name(name) && !rest.trim().is_empty() {
        return Some((Some(name.to_string()), strip_tags(rest)));
    }
    None
}

fn is_name(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.chars().count() <= 40
        && value.split_whitespace().count() <= 4
        && value
            .chars()
            .next()
            .map(|character| character.is_uppercase())
            .unwrap_or(false)
        && value
            .chars()
            .all(|character| character.is_alphanumeric() || " -'._".contains(character))
}

fn strip_tags(value: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    #[test]
    fn reads_srt_with_bom_crlf_speakers_and_multiple_lines() {
        let text = "\u{feff}1\r\n00:00:01,250 --> 00:00:03,500\r\n[Ana]: Hello\r\nand welcome\r\n\r\n2\r\n00:00:04.000 --> 00:00:06,000\r\nVoice: Bob\r\nGoodbye\r\n";
        let cues = parse("sample.srt", text.as_bytes()).unwrap();
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].start_ms, 1_250);
        assert_eq!(cues[0].end_ms, 3_500);
        assert_eq!(cues[0].speaker.as_deref(), Some("Ana"));
        assert_eq!(cues[0].text, "Hello\nand welcome");
        assert_eq!(cues[1].start_ms, 4_000);
        assert_eq!(cues[1].end_ms, 6_000);
        assert_eq!(cues[1].speaker.as_deref(), Some("Bob"));
        assert_eq!(cues[1].text, "Goodbye");
    }

    #[test]
    fn reads_vtt_voice_tags_out_of_order_and_short_timestamps() {
        let text = "WEBVTT\n\n00:01.000 --> 00:02.500\n<v Ana>Hello\nfrom the team\n\ncue-2\n00:00:00.500 --> 00:00:01.000\n[Bob]: Earlier\n";
        let cues = parse("sample.vtt", text.as_bytes()).unwrap();
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].speaker.as_deref(), Some("Bob"));
        assert_eq!(cues[0].start_ms, 500);
        assert_eq!(cues[1].speaker.as_deref(), Some("Ana"));
        assert_eq!(cues[1].start_ms, 1_000);
        assert_eq!(cues[1].end_ms, 2_500);
        assert_eq!(cues[1].text, "Hello\nfrom the team");
    }

    #[test]
    fn skips_empty_cues_and_rejects_a_file_without_text() {
        let text = "1\n00:00:00,000 --> 00:00:01,000\n\n2\n00:00:01,000 --> 00:00:02,000\n";
        assert!(parse("empty.srt", text.as_bytes())
            .unwrap_err()
            .contains("no cues with text"));
    }

    #[test]
    fn rejects_a_cue_without_a_time_range() {
        let error = parse("broken.srt", b"1\nnot a time\nhello\n").unwrap_err();
        assert!(error.contains("no start and end time"), "{error}");
        assert!(parse("broken.srt", &[0xFF, 0xFE, 0x00]).is_err());
    }

    #[test]
    fn an_imported_project_survives_a_store_round_trip() {
        let mut store = Store::open_memory().unwrap();
        let mut project = project(
            "p1".to_string(),
            "launch.vtt",
            b"WEBVTT\n\n00:01.000 --> 00:02.000\n[Ana]: Hello\n",
            "pt",
            "en",
        )
        .unwrap();
        store.save_dub_project(&mut project).unwrap();
        let loaded = store.load_dub_project("p1").unwrap().unwrap();
        assert_eq!(loaded.title, "launch");
        assert_eq!(loaded.segments.len(), 1);
        assert_eq!(loaded.segments[0].speaker.as_deref(), Some("Ana"));
        assert_eq!(loaded.segments[0].start_ms, 1_000);
        assert_eq!(loaded.segments[0].source_text, "Hello");
        assert_eq!(loaded.segments[0].state, "imported");
    }
}
