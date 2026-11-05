use std::fs;
use std::path::Path;
use std::process::Command;

use crate::story::Story;

#[cfg(test)]
pub struct Inspection {
    pub chapters: usize,
    pub titles: Vec<String>,
    pub cover_codec: Option<String>,
    pub cover_attached: bool,
    pub attached_covers: usize,
    pub normal_video: bool,
    pub covr: bool,
    pub media_type: Option<String>,
    pub brand: Option<String>,
    pub chpl: bool,
    pub chapter_track: bool,
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('=', "\\=")
        .replace(';', "\\;")
        .replace('#', "\\#")
        .replace('\n', " ")
}

fn write_cover(path: &Path) -> Result<(), String> {
    let status = Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-y",
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=0x1f3a32:s=600x600",
            "-frames:v",
            "1",
            "-c:v",
            "png",
            "-f",
            "image2",
            &path.to_string_lossy(),
        ])
        .status()
        .map_err(|error| format!("cannot start ffmpeg for the cover: {error}"))?;
    if !status.success() {
        return Err(format!("ffmpeg could not write the cover: {status}"));
    }
    Ok(())
}

pub fn chapter_metadata(story: &Story) -> Result<String, String> {
    let mut text = String::from(";FFMETADATA1\n");
    text.push_str(&format!("title={}\n", escape(&story.title)));
    text.push_str("artist=Parlo\n");
    text.push_str("album=Parlo stories\n");
    let mut chapters = 0;
    for chapter in &story.chapters {
        let start = chapter
            .lines
            .first()
            .map(|line| line.start)
            .unwrap_or(0.0)
            .max(0.0);
        let end = chapter
            .lines
            .last()
            .map(|line| line.end)
            .unwrap_or(start)
            .max(start);
        if end <= start {
            continue;
        }
        chapters += 1;
        text.push_str("\n[CHAPTER]\n");
        text.push_str("TIMEBASE=1/1000\n");
        text.push_str(&format!("START={}\n", (start * 1000.0).round() as u64));
        text.push_str(&format!("END={}\n", (end * 1000.0).round() as u64));
        text.push_str(&format!("title={}\n", escape(&chapter.title)));
    }
    if chapters == 0 {
        return Err("the story has no timed chapter to package".to_string());
    }
    Ok(text)
}

pub fn package(story: &Story, source: &Path, metadata: &Path, target: &Path) -> Result<(), String> {
    if !source.is_file() {
        return Err("the rendered story audio is missing".to_string());
    }
    let chapters = chapter_metadata(story)?;
    fs::write(metadata, chapters)
        .map_err(|error| format!("cannot write {}: {error}", metadata.display()))?;
    let cover = metadata.with_file_name("story-cover.png");
    write_cover(&cover)?;
    let title = format!("title={}", story.title);
    let status = Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-y",
            "-v",
            "error",
            "-i",
            &source.to_string_lossy(),
            "-i",
            &metadata.to_string_lossy(),
            "-i",
            &cover.to_string_lossy(),
            "-map",
            "0:a:0",
            "-map",
            "2:v:0",
            "-c:v",
            "png",
            "-disposition:v:0",
            "attached_pic",
            "-map_metadata",
            "1",
            "-map_chapters",
            "1",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-ar",
            "44100",
            "-movflags",
            "+faststart",
            "-metadata",
            "media_type=2",
            "-metadata",
            &title,
            "-metadata",
            "artist=Parlo",
            "-brand",
            "M4B ",
            "-f",
            "mp4",
            &target.to_string_lossy(),
        ])
        .status()
        .map_err(|error| format!("cannot start ffmpeg: {error}"))?;
    fs::remove_file(&cover).ok();
    if !status.success() {
        return Err(format!("ffmpeg could not write the audiobook: {status}"));
    }
    Ok(())
}

#[cfg(test)]
pub fn inspect(path: &Path) -> Result<Inspection, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_chapters",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
            &path.to_string_lossy(),
        ])
        .output()
        .map_err(|error| format!("cannot start ffprobe: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let value: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("invalid ffprobe output: {error}"))?;
    let chapters = value
        .get("chapters")
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let titles = value
        .get("chapters")
        .and_then(serde_json::Value::as_array)
        .map(|chapters| {
            chapters
                .iter()
                .filter_map(|chapter| chapter.get("tags")?.get("title")?.as_str())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let streams = value.get("streams").and_then(serde_json::Value::as_array);
    let attached = |stream: &&serde_json::Value| {
        stream
            .get("disposition")
            .and_then(|disposition| disposition.get("attached_pic"))
            .and_then(serde_json::Value::as_u64)
            == Some(1)
    };
    let attached_covers = streams
        .map(|streams| streams.iter().filter(attached).count())
        .unwrap_or_default();
    let normal_video = streams
        .map(|streams| {
            streams.iter().any(|stream| {
                stream.get("codec_type").and_then(serde_json::Value::as_str) == Some("video")
                    && !attached(&stream)
            })
        })
        .unwrap_or_default();
    let cover = streams.and_then(|streams| streams.iter().find(attached));
    let cover_codec = cover
        .and_then(|stream| stream.get("codec_name"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let tags = value
        .get("format")
        .and_then(|format| format.get("tags"))
        .and_then(serde_json::Value::as_object);
    let tag = |name: &str| {
        tags.and_then(|tags| tags.get(name))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    };
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let has = |needle: &[u8]| bytes.windows(needle.len()).any(|window| window == needle);
    Ok(Inspection {
        chapters,
        titles,
        cover_codec,
        cover_attached: cover.is_some(),
        attached_covers,
        normal_video,
        covr: bytes.windows(4).any(|window| window == b"covr"),
        media_type: tag("media_type").or_else(|| tag("stik")),
        brand: tag("major_brand").or_else(|| tag("brand")),
        chpl: has(b"chpl"),
        chapter_track: has(b"chap"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::{Chapter, StoryLine};

    fn story() -> Story {
        Story {
            id: "s1".to_string(),
            title: "A book".to_string(),
            language: "pt".to_string(),
            state: crate::story::DONE.to_string(),
            batch: None,
            error: None,
            rendered: 2,
            cached: 0,
            failed: 0,
            created_at: 1,
            updated_at: 1,
            characters: Vec::new(),
            chapters: vec![
                Chapter {
                    id: "ch1".to_string(),
                    title: "One".to_string(),
                    lines: vec![StoryLine {
                        id: "l1".to_string(),
                        speaker: "c1".to_string(),
                        text: "Olá.".to_string(),
                        pace: 1.0,
                        state: crate::story::DONE.to_string(),
                        out: None,
                        seconds: 1.0,
                        error: None,
                        start: 0.0,
                        end: 1.0,
                    }],
                },
                Chapter {
                    id: "ch2".to_string(),
                    title: "Two".to_string(),
                    lines: vec![StoryLine {
                        id: "l2".to_string(),
                        speaker: "c1".to_string(),
                        text: "Adeus.".to_string(),
                        pace: 1.0,
                        state: crate::story::DONE.to_string(),
                        out: None,
                        seconds: 1.0,
                        error: None,
                        start: 1.0,
                        end: 2.0,
                    }],
                },
            ],
        }
    }

    #[test]
    fn metadata_names_every_chapter_in_milliseconds() {
        let text = chapter_metadata(&story()).unwrap();
        assert!(text.contains("title=A book"));
        assert_eq!(text.matches("[CHAPTER]").count(), 2);
        assert!(text.contains("START=0\nEND=1000"));
        assert!(text.contains("START=1000\nEND=2000"));
    }

    #[test]
    fn a_story_without_timed_lines_is_refused() {
        let mut story = story();
        story.chapters[0].lines[0].end = 0.0;
        story.chapters[1].lines[0].end = 0.0;
        assert!(chapter_metadata(&story).is_err());
    }

    fn silence(path: &Path) {
        let samples = 44_100u32 * 2;
        let data = samples * 2;
        let mut bytes = Vec::with_capacity(44 + data as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&44_100u32.to_le_bytes());
        bytes.extend_from_slice(&88_200u32.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data.to_le_bytes());
        bytes.resize(44 + data as usize, 0);
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn a_real_m4b_carries_chapters_and_audiobook_tags() {
        if Command::new("ffmpeg").arg("-version").output().is_err() {
            return;
        }
        let directory = std::env::temp_dir().join(format!("parlo-m4b-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let source = directory.join("story.wav");
        let target = directory.join("story.m4b");
        silence(&source);
        package(&story(), &source, &directory.join("story.ffmeta"), &target).unwrap();
        let result = inspect(&target).unwrap();
        assert_eq!(result.chapters, 2);
        assert_eq!(result.titles, vec!["One", "Two"]);
        assert_eq!(result.cover_codec.as_deref(), Some("png"));
        assert!(result.cover_attached);
        assert_eq!(result.attached_covers, 1);
        assert!(!result.normal_video);
        assert!(result.covr);
        assert_eq!(result.media_type.as_deref(), Some("2"));
        assert_eq!(result.brand.as_deref(), Some("M4B "));
        assert!(result.chpl);
        assert!(result.chapter_track);
        fs::remove_dir_all(&directory).ok();
    }
}
