use serde_json::{json, Value};

const PROTOCOL: &str = "2024-11-05";

pub fn handle(request: &Value) -> Option<Value> {
    let id = request.get("id")?.clone();
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": PROTOCOL,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "parlo", "version": env!("CARGO_PKG_VERSION") },
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools() })),
        "tools/call" => Ok(call(request.get("params").unwrap_or(&Value::Null))),
        other => Err((-32601, format!("this server has no method {other}"))),
    };
    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, message)) => {
            json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
        }
    })
}

pub fn serve() -> Result<(), String> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(request) => handle(&request),
            Err(error) => Some(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32700, "message": format!("this is not JSON: {error}") },
            })),
        };
        if let Some(reply) = reply {
            writeln!(stdout, "{reply}").map_err(|error| error.to_string())?;
            stdout.flush().map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn tools() -> Value {
    json!([
        {
            "name": "list_meetings",
            "description": "Every meeting kept on this computer, with its id, title, time, line count and speakers.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
        },
        {
            "name": "ask_meetings",
            "description": "Answer a question from the transcripts and quote the lines used, with the meeting title, the speaker and the minute.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "question": { "type": "string", "description": "What to look for in the meetings." },
                },
                "required": ["question"],
                "additionalProperties": false,
            },
        },
        {
            "name": "meeting_notes",
            "description": "One meeting as Markdown: its decisions, action items, notes and the transcript.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "The meeting id from list_meetings." },
                },
                "required": ["id"],
                "additionalProperties": false,
            },
        },
        {
            "name": "recent_dictations",
            "description": "The most recent dictations kept in the library, newest first.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "minimum": 1, "maximum": 50 },
                },
                "additionalProperties": false,
            },
        },
    ])
}

fn call(params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let text = match name {
        "list_meetings" => list_meetings(),
        "ask_meetings" => match arguments.get("question").and_then(Value::as_str) {
            Some(question) if !question.trim().is_empty() => ask_meetings(question),
            _ => return failure("ask_meetings needs a question"),
        },
        "meeting_notes" => match arguments.get("id").and_then(Value::as_str) {
            Some(id) => match crate::meeting::load(id) {
                Ok(meeting) => crate::notes::markdown(&meeting),
                Err(error) => return failure(&error),
            },
            None => return failure("meeting_notes needs a meeting id"),
        },
        "recent_dictations" => recent_dictations(&arguments),
        other => return failure(&format!("this server has no tool called {other}")),
    };
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}

fn failure(message: &str) -> Value {
    json!({ "content": [{ "type": "text", "text": message }], "isError": true })
}

fn list_meetings() -> String {
    let meetings: Vec<Value> = crate::meeting::list()
        .iter()
        .map(|meeting| {
            json!({
                "id": meeting.id,
                "title": meeting.title,
                "started_at": meeting.started_at,
                "lines": meeting.segments.len(),
                "speakers": meeting.speakers(),
            })
        })
        .collect();
    json!(meetings).to_string()
}

fn ask_meetings(question: &str) -> String {
    let answer = crate::ask::ask(question, &crate::meeting::list());
    json!({
        "found": answer.found,
        "answer": answer.answer,
        "citations": answer.citations,
    })
    .to_string()
}

fn recent_dictations(arguments: &Value) -> String {
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(10)
        .clamp(1, 50) as usize;
    let entries: Vec<Value> = crate::history::recent(limit)
        .iter()
        .map(|entry| {
            json!({
                "id": entry.id,
                "at": entry.at,
                "text": entry.text,
                "translation": entry.translation,
                "from": entry.from,
                "to": entry.to,
                "app": entry.app,
                "mode": entry.mode,
                "favourite": entry.favourite,
            })
        })
        .collect();
    json!(entries).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call_tool(name: &str, arguments: Value) -> Value {
        handle(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments },
        }))
        .expect("a reply")
    }

    fn text_of(reply: &Value) -> String {
        reply["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string()
    }

    #[test]
    fn initialize_names_the_server_and_its_tools() {
        let reply = handle(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        }))
        .expect("a reply");
        assert_eq!(reply["result"]["serverInfo"]["name"], "parlo");
        assert_eq!(reply["result"]["protocolVersion"], PROTOCOL);
        assert!(reply["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn the_tool_list_covers_the_library_and_the_meetings() {
        let reply = handle(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {},
        }))
        .expect("a reply");
        let tools = reply["result"]["tools"].as_array().expect("tools");
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect();
        assert!(names.contains(&"list_meetings"));
        assert!(names.contains(&"ask_meetings"));
        assert!(names.contains(&"meeting_notes"));
        assert!(names.contains(&"recent_dictations"));
        for tool in tools {
            assert!(tool["inputSchema"]["type"] == "object");
            assert!(!tool["description"].as_str().unwrap_or("").is_empty());
        }
    }

    #[test]
    fn every_documented_tool_is_callable() {
        let calls = [
            ("list_meetings", json!({})),
            ("recent_dictations", json!({ "limit": 5 })),
            ("ask_meetings", json!({ "question": "decisions" })),
        ];
        for (name, arguments) in calls {
            let reply = call_tool(name, arguments);
            assert_eq!(
                reply["result"]["isError"], false,
                "{name} should not fail on a well formed call"
            );
        }
    }

    #[test]
    fn a_question_is_required_and_answered_with_citations() {
        let empty = call_tool("ask_meetings", json!({ "question": "  " }));
        assert_eq!(empty["result"]["isError"], true);
        assert!(text_of(&empty).contains("needs a question"));

        let reply = call_tool("ask_meetings", json!({ "question": "what did we decide" }));
        assert_eq!(reply["result"]["isError"], false);
        let answer: Value = serde_json::from_str(&text_of(&reply)).expect("json");
        assert!(answer["answer"].is_string());
        assert!(answer["citations"].is_array());
    }

    #[test]
    fn a_missing_meeting_is_an_error_result_not_a_crash() {
        let reply = call_tool("meeting_notes", json!({ "id": "nope" }));
        assert_eq!(reply["result"]["isError"], true);
        assert!(!text_of(&reply).is_empty());
        let missing_id = call_tool("meeting_notes", json!({}));
        assert!(text_of(&missing_id).contains("needs a meeting id"));
    }

    #[test]
    fn an_unknown_tool_says_so() {
        let reply = call_tool("delete_everything", json!({}));
        assert_eq!(reply["result"]["isError"], true);
        assert!(text_of(&reply).contains("delete_everything"));
    }

    #[test]
    fn an_unknown_method_is_an_error() {
        let reply = handle(&json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/subscribe",
        }))
        .expect("a reply");
        assert_eq!(reply["error"]["code"], -32601);
        assert_eq!(reply["id"], 7);
    }

    #[test]
    fn a_notification_gets_no_reply() {
        assert!(handle(&json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
        }))
        .is_none());
    }
}
