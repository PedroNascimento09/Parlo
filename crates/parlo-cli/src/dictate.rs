use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_tool() -> Option<(&'static str, Vec<&'static str>)> {
    if cfg!(target_os = "windows") {
        Some(("clip", Vec::new()))
    } else if cfg!(target_os = "macos") {
        Some(("pbcopy", Vec::new()))
    } else if env::var_os("WAYLAND_DISPLAY").is_some() {
        Some(("wl-copy", Vec::new()))
    } else if env::var_os("DISPLAY").is_some() {
        Some(("xclip", vec!["-selection", "clipboard"]))
    } else {
        None
    }
}

pub fn copy(text: &str) -> Result<String, String> {
    let (program, args) = copy_tool()
        .ok_or("no clipboard tool found, install wl-clipboard or xclip, or pass --no-copy")?;
    let mut child = Command::new(program)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot run {program}: {error}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| format!("{program} has no input stream"))?;
    stdin
        .write_all(text.as_bytes())
        .map_err(|error| format!("cannot write to {program}: {error}"))?;
    drop(stdin);
    let output = child
        .wait_with_output()
        .map_err(|error| format!("{program} did not finish: {error}"))?;
    if output.status.success() {
        Ok(program.to_string())
    } else {
        Err(format!(
            "{program} exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

pub fn join(parts: &[String]) -> String {
    let mut text = String::new();
    for part in parts {
        let clean = part.trim();
        if clean.is_empty() {
            continue;
        }
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(clean);
    }
    sentence(text)
}

pub fn sentence(text: String) -> String {
    let mut chars = text.chars();
    let capitalised = match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => return text,
    };
    match capitalised.chars().last() {
        Some('.') | Some('!') | Some('?') | Some(':') | Some(';') => capitalised,
        Some(_) => capitalised + ".",
        None => capitalised,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    #[test]
    fn segments_are_joined_with_one_space() {
        let text = join(&parts(&["good morning", "how are you"]));
        assert_eq!(text, "Good morning how are you.");
    }

    #[test]
    fn an_empty_segment_is_skipped() {
        let text = join(&parts(&["good morning", "  ", "thank you"]));
        assert_eq!(text, "Good morning thank you.");
    }

    #[test]
    fn nothing_becomes_nothing() {
        assert_eq!(join(&parts(&["", " "])), "");
    }

    #[test]
    fn an_existing_full_stop_is_kept() {
        assert_eq!(sentence("it is done.".to_string()), "It is done.");
    }

    #[test]
    fn a_question_mark_is_kept() {
        assert_eq!(sentence("it is done?".to_string()), "It is done?");
    }

    #[test]
    fn the_first_letter_is_capitalised() {
        assert_eq!(sentence("meeting at nine".to_string()), "Meeting at nine.");
    }

    #[test]
    fn a_clipboard_tool_is_named_on_every_desktop_platform() {
        if cfg!(target_os = "windows") {
            assert_eq!(copy_tool().map(|tool| tool.0), Some("clip"));
        }
        if cfg!(target_os = "macos") {
            assert_eq!(copy_tool().map(|tool| tool.0), Some("pbcopy"));
        }
    }
}
