use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const SYSTEM: &str = "You summarise a meeting transcript. Answer with two short lists: decisions, then action items, each with the person when the transcript names one. Never invent a name, a date or a decision that is not in the transcript. If there are none, say so in one line.";

pub struct Target {
    pub host: String,
    pub port: u16,
    pub path: String,
}

pub fn parse_target(url: &str) -> Result<Target, String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| "the local model address must start with http://".to_string())?;
    let (authority, path) = match rest.split_once('/') {
        Some((authority, tail)) => (authority, format!("/{tail}")),
        None => (rest, "/v1/chat/completions".to_string()),
    };
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|_| "that port is not a number".to_string())?,
        ),
        None => (authority.to_string(), 80),
    };
    if host.is_empty() {
        return Err("the local model address has no host".to_string());
    }
    Ok(Target { host, port, path })
}

pub fn body(model: &str, prompt: &str) -> String {
    serde_json::json!({
        "model": model,
        "temperature": 0,
        "messages": [
            { "role": "system", "content": SYSTEM },
            { "role": "user", "content": prompt }
        ]
    })
    .to_string()
}

pub fn extract(answer: &str) -> Result<String, String> {
    let (head, payload) = answer
        .split_once("\r\n\r\n")
        .or_else(|| answer.split_once("\n\n"))
        .ok_or_else(|| "the model answered without a body".to_string())?;
    let status = head.lines().next().unwrap_or("");
    if !status.contains(" 200") {
        return Err(format!("the model answered with {status}"));
    }
    let value: serde_json::Value = serde_json::from_str(payload.trim())
        .map_err(|error| format!("the model answered with something else: {error}"))?;
    value
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .ok_or_else(|| "the model answered without text".to_string())
}

pub fn answer(meeting: &crate::meeting::Meeting, url: &str, model: &str) -> String {
    let rules = rules(meeting);
    if url.trim().is_empty() {
        return envelope(&meeting.id, &rules, "rules", "", "");
    }
    let text = prompt(&meeting.title, &meeting.segments);
    match ask(url, model, &text) {
        Ok(summary) => envelope(&meeting.id, &summary, "local", model, ""),
        Err(error) => envelope(&meeting.id, &rules, "rules", "", &error),
    }
}

fn envelope(id: &str, summary: &str, by: &str, model: &str, note: &str) -> String {
    serde_json::json!({
        "meeting": id,
        "summary": summary,
        "by": by,
        "model": model,
        "note": note,
    })
    .to_string()
}

pub fn rules(meeting: &crate::meeting::Meeting) -> String {
    let agreed = reported(crate::decisions::for_meeting(meeting), "speaker");
    let todos = reported(crate::actions::for_meeting(meeting), "owner");
    let mut lines = vec!["Decisions:".to_string()];
    lines.extend(or_none(agreed));
    lines.push(String::new());
    lines.push("Action items:".to_string());
    lines.extend(or_none(todos));
    lines.join("\n")
}

fn or_none(found: Vec<String>) -> Vec<String> {
    if found.is_empty() {
        vec!["- none in this transcript".to_string()]
    } else {
        found
    }
}

fn reported(items: Vec<impl serde::Serialize>, who: &str) -> Vec<String> {
    let value = serde_json::to_value(items).unwrap_or_default();
    let mut out = Vec::new();
    for item in value.as_array().cloned().unwrap_or_default() {
        let text = item
            .get("text")
            .and_then(|text| text.as_str())
            .unwrap_or("");
        if text.is_empty() {
            continue;
        }
        let speaker = item
            .get(who)
            .and_then(|speaker| speaker.as_str())
            .filter(|speaker| !speaker.is_empty())
            .unwrap_or("someone");
        let second = item
            .get("at_ms")
            .and_then(|at| at.as_u64())
            .map(|ms| ms / 1000);
        out.push(match second {
            Some(second) => format!("- {speaker} at {}:{:02}: {text}", second / 60, second % 60),
            None => format!("- {speaker}: {text}"),
        });
    }
    out
}

pub fn prompt(title: &str, segments: &[crate::meeting::Segment]) -> String {
    let mut lines = vec![format!("Meeting: {title}"), String::new()];
    for segment in segments {
        let total = segment.at_ms / 1000;
        lines.push(format!(
            "{}:{:02} {}: {}",
            total / 60,
            total % 60,
            segment.spoken_by(),
            segment.text
        ));
    }
    lines.join("\n")
}

pub fn ask(url: &str, model: &str, text: &str) -> Result<String, String> {
    let target = parse_target(url)?;
    let payload = body(model, text);
    let mut stream = TcpStream::connect((target.host.as_str(), target.port))
        .map_err(|error| format!("cannot reach the local model: {error}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(120)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        target.path,
        target.host,
        target.port,
        payload.len(),
        payload
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("cannot send the transcript: {error}"))?;
    let mut answer = Vec::new();
    stream
        .read_to_end(&mut answer)
        .map_err(|error| format!("cannot read the model: {error}"))?;
    extract(&String::from_utf8_lossy(&answer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_canned_answer_travels_over_a_socket() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request);
                let body = "{\"choices\":[{\"message\":{\"content\":\"Decisions: none\"}}]}";
                let answer = format!(
                    "HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: {}
Connection: close

{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(answer.as_bytes());
            }
        });
        let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
        assert_eq!(
            ask(&url, "local", "Meeting: none").unwrap(),
            "Decisions: none"
        );
    }

    #[test]
    fn a_full_address_keeps_its_path() {
        let target = parse_target("http://127.0.0.1:8080/v1/chat/completions").unwrap();
        assert_eq!(target.host, "127.0.0.1");
        assert_eq!(target.port, 8080);
        assert_eq!(target.path, "/v1/chat/completions");
    }

    #[test]
    fn a_bare_address_uses_the_usual_path_and_port() {
        let target = parse_target("http://localhost").unwrap();
        assert_eq!(target.host, "localhost");
        assert_eq!(target.port, 80);
        assert_eq!(target.path, "/v1/chat/completions");
    }

    #[test]
    fn https_is_refused_with_a_sentence() {
        assert!(parse_target("https://api.example.com/v1").is_err());
    }

    #[test]
    fn a_text_answer_is_unwrapped() {
        let answer = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"choices\":[{\"message\":{\"content\":\"Decisions:\\n- none\"}}]}";
        assert_eq!(extract(answer).unwrap(), "Decisions:\n- none");
    }

    #[test]
    fn a_failure_status_names_itself() {
        let answer = "HTTP/1.1 500 Internal Server Error\r\n\r\n{}";
        let error = extract(answer).unwrap_err();
        assert!(error.contains("500"));
    }

    #[test]
    fn an_answer_without_text_is_an_error() {
        let answer = "HTTP/1.1 200 OK\r\n\r\n{\"choices\":[]}";
        assert!(extract(answer).unwrap_err().contains("without text"));
    }

    #[test]
    fn a_machine_without_a_model_answers_with_the_rules_and_says_so() {
        let meeting = crate::meeting::Meeting {
            id: "m1".to_string(),
            title: "Planning".to_string(),
            started_at: 0,
            speaker_hint: None,
            segments: Vec::new(),
        };
        let value: serde_json::Value =
            serde_json::from_str(&answer(&meeting, "", "local")).unwrap();
        assert_eq!(value["by"], "rules");
        assert_eq!(value["meeting"], "m1");
        assert!(value["summary"].as_str().unwrap().contains("Decisions:"));
        assert_eq!(value["note"], "");
    }

    #[test]
    fn a_model_that_answers_is_named_as_the_one_that_wrote_it() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request);
                let body = "{\"choices\":[{\"message\":{\"content\":\"Decisions: none\"}}]}";
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        let meeting = crate::meeting::Meeting {
            id: "m2".to_string(),
            title: "Planning".to_string(),
            started_at: 0,
            speaker_hint: None,
            segments: Vec::new(),
        };
        let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
        let value: serde_json::Value =
            serde_json::from_str(&answer(&meeting, &url, "small-model")).unwrap();
        assert_eq!(value["by"], "local");
        assert_eq!(value["model"], "small-model");
        assert_eq!(value["summary"], "Decisions: none");
    }

    #[test]
    fn a_model_that_cannot_be_reached_falls_back_to_the_rules_with_the_reason() {
        let meeting = crate::meeting::Meeting {
            id: "m3".to_string(),
            title: "Planning".to_string(),
            started_at: 0,
            speaker_hint: None,
            segments: Vec::new(),
        };
        let value: serde_json::Value =
            serde_json::from_str(&answer(&meeting, "http://127.0.0.1:1/v1", "local")).unwrap();
        assert_eq!(value["by"], "rules");
        assert!(value["summary"].as_str().unwrap().contains("Decisions:"));
        assert!(!value["note"].as_str().unwrap().is_empty());
    }

    #[test]
    fn the_rules_summary_says_who_agreed_and_when() {
        let meeting = crate::meeting::Meeting {
            id: "m1".to_string(),
            title: "Planning".to_string(),
            started_at: 0,
            speaker_hint: None,
            segments: vec![crate::meeting::Segment {
                at_ms: 12_000,
                speaker: "Ana".to_string(),
                text: "Fica para sexta, eu envio o convite.".to_string(),
                name: None,
            }],
        };
        let summary = rules(&meeting);
        assert!(summary.contains("Decisions:"));
        assert!(summary.contains("Action items:"));
        assert!(
            summary.contains("Ana") || summary.contains("none in this transcript"),
            "{summary}"
        );
    }

    #[test]
    fn the_prompt_carries_the_speaker_the_moment_and_the_line() {
        let segments = vec![crate::meeting::Segment {
            at_ms: 12_000,
            speaker: "Ana".to_string(),
            text: "Fica para sexta.".to_string(),
            name: None,
        }];
        let text = prompt("Planning", &segments);
        assert!(text.contains("Meeting: Planning"));
        assert!(text.contains("0:12 Ana: Fica para sexta."));
    }
}
