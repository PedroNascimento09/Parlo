use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_LINES: usize = 5_000;
const DEFAULT_HOLD_MS: u64 = 2_500;

pub fn srt(lines: &[(u64, String)]) -> String {
    let mut out = String::new();
    let mut index_out = 0;
    for (index, (at_ms, text)) in lines.iter().enumerate() {
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let end = lines
            .get(index + 1)
            .map(|(next, _)| *next)
            .unwrap_or(at_ms + DEFAULT_HOLD_MS);
        index_out += 1;
        out.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            index_out,
            clock(*at_ms),
            clock(end.max(at_ms + 500)),
            text
        ));
    }
    out
}

fn clock(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms / 60_000) % 60;
    let seconds = (ms / 1_000) % 60;
    let millis = ms % 1_000;
    format!("{hours:02}:{minutes:02}:{seconds:02},{millis:03}")
}

pub fn save(lines: &[(u64, String)]) -> Result<PathBuf, String> {
    let body = srt(lines);
    if body.is_empty() {
        return Err("there are no caption lines to save yet".to_string());
    }
    if lines.len() > MAX_LINES {
        return Err(format!(
            "this session has more than {MAX_LINES} lines; save it in parts"
        ));
    }
    let folder = crate::config::data_root().join("captions");
    fs::create_dir_all(&folder)
        .map_err(|error| format!("{} could not be created: {error}", folder.display()))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    let path = folder.join(format!("captions-{stamp}.srt"));
    fs::write(&path, body)
        .map_err(|error| format!("{} could not be written: {error}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_holds_until_the_next_one_starts() {
        let lines = vec![(0, "bom dia".to_string()), (3_000, "até logo".to_string())];
        let text = srt(&lines);
        assert!(
            text.starts_with("1\n00:00:00,000 --> 00:00:03,000\nbom dia\n\n"),
            "{text}"
        );
        assert!(
            text.contains("2\n00:00:03,000 --> 00:00:05,500\naté logo\n"),
            "{text}"
        );
    }

    #[test]
    fn empty_lines_are_left_out_and_numbering_stays_continuous() {
        let lines = vec![
            (0, "primeira".to_string()),
            (1_000, "   ".to_string()),
            (2_000, "segunda".to_string()),
        ];
        let text = srt(&lines);
        assert_eq!(text.matches(" --> ").count(), 2);
        assert!(text.contains("2\n00:00:02,000"), "{text}");
    }

    #[test]
    fn a_session_without_lines_is_refused() {
        assert!(save(&[]).unwrap_err().contains("no caption lines"));
    }
}
