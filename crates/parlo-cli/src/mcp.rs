use std::io::{BufRead, Write};

use serde_json::{json, Value};

const PROTOCOL: &str = "2024-11-05";

pub fn handle(request: &Value) -> Option<Value> {
    let id = request.get("id")?.clone();
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    if method == "initialize" {
        let client = request
            .pointer("/params/clientInfo/name")
            .and_then(Value::as_str)
            .unwrap_or("");
        let _ = crate::agents::remember(client);
    }
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
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    serve_with(stdin.lock(), stdout.lock())
}

fn serve_with<R: BufRead, W: Write>(input: R, mut output: W) -> Result<(), String> {
    for line in input.lines() {
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
            writeln!(output, "{reply}").map_err(|error| error.to_string())?;
            output.flush().map_err(|error| error.to_string())?;
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
        {
            "name": "list_models",
            "description": "Every model in the catalogue with its licence, download size, languages, devices and whether it is installed.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
        },
        {
            "name": "list_engines",
            "description": "The speech and translation engines Parlo knows, with their licence, hardware, fidelity and availability.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
        },
        {
            "name": "capabilities",
            "description": "What this Parlo server can do, which transports it speaks, how it authenticates and where audio stays.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
        },
        {
            "name": "search_dictations",
            "description": "Search past dictations by their text, translation or program, newest first.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The words to look for." },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 50 },
                },
                "required": ["query"],
                "additionalProperties": false,
            },
        },
        {
            "name": "favourite_dictations",
            "description": "The dictations marked as favourites, newest first.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "minimum": 1, "maximum": 50 },
                },
                "additionalProperties": false,
            },
        },
        {
            "name": "meeting_transcript",
            "description": "One meeting's transcript as structured lines, with millisecond times, speaker labels and the names applied to them.",
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
            "name": "meeting_actions",
            "description": "The action items found in one meeting, with the owner, the due phrase and the line that supports each one.",
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
            "name": "bind_voice",
            "description": "Bind this client to one of the voices on this computer, so every later call from the same client resolves to it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "voice": { "type": "string", "description": "A voice id from list_models, such as voice-pt or voice-en." },
                },
                "required": ["voice"],
                "additionalProperties": false,
            },
        },
        {
            "name": "agent_voice",
            "description": "The voice bound to the client that is connected, and whether it has one.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
        },
        {
            "name": "list_voice_packs",
            "description": "The voice packs kept on this computer, each with the voices it carries.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
        },
        {
            "name": "meeting_decisions",
            "description": "The decisions found in one meeting, with the speaker, the time and the decision each one replaces.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "The meeting id from list_meetings." },
                },
                "required": ["id"],
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
        "list_models" => list_models(),
        "list_engines" => list_engines(),
        "capabilities" => capabilities(),
        "search_dictations" => match arguments.get("query").and_then(Value::as_str) {
            Some(query) if !query.trim().is_empty() => search_dictations(query, &arguments),
            _ => return failure("search_dictations needs a query"),
        },
        "favourite_dictations" => favourite_dictations(&arguments),
        "meeting_transcript" => match render_meeting(&arguments, name, transcript_json) {
            Ok(text) => text,
            Err(error) => return failure(&error),
        },
        "meeting_actions" => match render_meeting(&arguments, name, actions_json) {
            Ok(text) => text,
            Err(error) => return failure(&error),
        },
        "meeting_decisions" => match render_meeting(&arguments, name, decisions_json) {
            Ok(text) => text,
            Err(error) => return failure(&error),
        },
        "bind_voice" => match arguments.get("voice").and_then(Value::as_str) {
            Some(voice) => match crate::agents::current() {
                Some(client) => match crate::agents::bind(&client, voice) {
                    Ok(_) => json!({ "client": client, "voice": voice.trim(), "bound": true })
                        .to_string(),
                    Err(error) => return failure(&error),
                },
                None => return failure("this client did not say its name when it connected"),
            },
            None => return failure("bind_voice needs a voice id"),
        },
        "agent_voice" => match crate::agents::current() {
            Some(client) => match crate::agents::voice_for(&client) {
                Some(voice) => {
                    json!({ "client": client, "voice": voice, "bound": true }).to_string()
                }
                None => json!({ "client": client, "voice": "", "bound": false }).to_string(),
            },
            None => json!({ "client": "", "voice": "", "bound": false }).to_string(),
        },
        "list_voice_packs" => match crate::packs::list() {
            Ok(packs) => json!({ "packs": packs }).to_string(),
            Err(error) => return failure(&error),
        },
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
    let entries: Vec<Value> = crate::history::recent(limit_of(arguments))
        .iter()
        .map(entry_json)
        .collect();
    json!(entries).to_string()
}

fn search_dictations(query: &str, arguments: &Value) -> String {
    let entries: Vec<Value> = crate::history::search(query, limit_of(arguments))
        .iter()
        .map(entry_json)
        .collect();
    json!(entries).to_string()
}

fn favourite_dictations(arguments: &Value) -> String {
    let entries: Vec<Value> = crate::history::favourites(limit_of(arguments))
        .iter()
        .map(entry_json)
        .collect();
    json!(entries).to_string()
}

fn limit_of(arguments: &Value) -> usize {
    arguments
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(10)
        .clamp(1, 50) as usize
}

fn entry_json(entry: &crate::history::Entry) -> Value {
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
        "seconds": entry.seconds,
    })
}

fn list_models() -> String {
    let models: Vec<Value> = crate::models::status()
        .iter()
        .map(|model| {
            let spec = crate::registry::model(model.id);
            json!({
                "id": model.id,
                "name": model.name,
                "kind": model.kind,
                "feature": model.feature,
                "installed": model.installed,
                "installed_bytes": model.bytes,
                "download_bytes": spec.map(|spec| spec.bytes).unwrap_or(0),
                "licence": spec.map(|spec| spec.licence).unwrap_or(""),
                "languages": spec.map(|spec| spec.languages).unwrap_or(&[]),
                "devices": spec.map(|spec| spec.devices).unwrap_or(&[]),
                "capabilities": spec.map(|spec| spec.capabilities).unwrap_or(&[]),
                "default": spec.map(|spec| spec.default).unwrap_or(false),
            })
        })
        .collect();
    json!(models).to_string()
}

fn list_engines() -> String {
    let engines: Vec<Value> = crate::registry::PIPELINES
        .iter()
        .map(|spec| {
            json!({
                "id": spec.id,
                "name": spec.name,
                "licence": spec.licence,
                "hardware": spec.hardware,
                "requires_gpu": spec.requires_gpu,
                "available": spec.available,
                "default": spec.default,
                "fidelity": spec.fidelity,
            })
        })
        .collect();
    json!(engines).to_string()
}

fn capabilities() -> String {
    json!({
        "name": "Parlo",
        "version": env!("CARGO_PKG_VERSION"),
        "api": "v1",
        "local": true,
        "transports": ["http", "websocket", "mcp-http", "mcp-stdio"],
        "authentication": "x-parlo-token or Authorization: Bearer",
        "capabilities": ["dictation", "translation", "meetings", "voices", "dubbing", "diarisation"],
        "privacy": "audio stays on this computer",
    })
    .to_string()
}

fn render_meeting(
    arguments: &Value,
    tool: &str,
    render: fn(&crate::meeting::Meeting) -> String,
) -> Result<String, String> {
    let id = arguments
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{tool} needs a meeting id"))?;
    let meeting = crate::meeting::load(id)?;
    Ok(render(&meeting))
}

fn transcript_json(meeting: &crate::meeting::Meeting) -> String {
    let segments: Vec<Value> = meeting
        .segments
        .iter()
        .map(|segment| {
            json!({
                "at_ms": segment.at_ms,
                "speaker": segment.speaker,
                "name": segment.name.as_ref().map(|name| name.name.as_str()),
                "text": segment.text,
            })
        })
        .collect();
    json!({
        "id": meeting.id,
        "title": meeting.title,
        "started_at": meeting.started_at,
        "segments": segments,
    })
    .to_string()
}

fn actions_json(meeting: &crate::meeting::Meeting) -> String {
    let actions: Vec<Value> = crate::actions::for_meeting(meeting)
        .iter()
        .map(|action| {
            json!({
                "at_ms": action.at_ms,
                "speaker": action.speaker,
                "owner": action.owner,
                "due": action.due,
                "text": action.text,
            })
        })
        .collect();
    json!(actions).to_string()
}

fn decisions_json(meeting: &crate::meeting::Meeting) -> String {
    let decisions: Vec<Value> = crate::decisions::for_meeting(meeting)
        .iter()
        .map(|decision| {
            json!({
                "at_ms": decision.at_ms,
                "speaker": decision.speaker,
                "text": decision.text,
                "supersedes": decision.supersedes,
            })
        })
        .collect();
    json!(decisions).to_string()
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
        assert_eq!(tools.len(), 15);
        for name in [
            "list_meetings",
            "ask_meetings",
            "meeting_notes",
            "recent_dictations",
            "list_models",
            "list_engines",
            "capabilities",
            "search_dictations",
            "favourite_dictations",
            "meeting_transcript",
            "meeting_actions",
            "meeting_decisions",
            "bind_voice",
            "agent_voice",
            "list_voice_packs",
        ] {
            assert!(names.contains(&name), "{name} is missing");
        }
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
            ("list_models", json!({})),
            ("list_engines", json!({})),
            ("capabilities", json!({})),
            ("search_dictations", json!({ "query": "test" })),
            ("favourite_dictations", json!({ "limit": 5 })),
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
    fn the_client_that_says_its_name_resolves_to_its_voice() {
        let initialized = handle(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "clientInfo": { "name": "claude-code" } },
        }))
        .expect("a reply");
        assert_eq!(initialized["result"]["serverInfo"]["name"], "parlo");
        let answer = call_tool("agent_voice", json!({}));
        assert_eq!(answer["result"]["isError"], false);
        let detail: Value = serde_json::from_str(&text_of(&answer)).expect("json");
        assert_eq!(detail["client"], "claude-code");
        assert_eq!(detail["bound"], false);
        let refused = call_tool("bind_voice", json!({ "voice": "parakeet" }));
        assert_eq!(refused["result"]["isError"], true);
        assert!(text_of(&refused).contains("not a voice"));
    }

    #[test]
    fn every_action_and_decision_names_its_speaker_and_minute() {
        use crate::meeting::{Meeting, Segment};

        let meeting = Meeting {
            id: "m1".to_string(),
            title: "Weekly".to_string(),
            started_at: 0,
            segments: vec![
                Segment {
                    at_ms: 65_000,
                    speaker: "speaker-1".to_string(),
                    text: "Vou enviar o relatorio amanha.".to_string(),
                    name: None,
                },
                Segment {
                    at_ms: 185_000,
                    speaker: "speaker-2".to_string(),
                    text: "Fica combinado que a reuniao passa para as dez.".to_string(),
                    name: None,
                },
            ],
            speaker_hint: None,
        };

        let actions: Value = serde_json::from_str(&actions_json(&meeting)).unwrap();
        let actions = actions.as_array().unwrap();
        assert_eq!(actions.len(), 1, "{actions:?}");
        assert_eq!(actions[0]["speaker"], "speaker-1");
        assert_eq!(actions[0]["at_ms"], 65_000);
        assert_eq!(actions[0]["at_ms"].as_u64().unwrap() / 60_000, 1);

        let decisions: Value = serde_json::from_str(&decisions_json(&meeting)).unwrap();
        let decisions = decisions.as_array().unwrap();
        assert_eq!(decisions.len(), 1, "{decisions:?}");
        assert_eq!(decisions[0]["speaker"], "speaker-2");
        assert_eq!(decisions[0]["at_ms"], 185_000);
        assert_eq!(decisions[0]["at_ms"].as_u64().unwrap() / 60_000, 3);
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

    #[test]
    fn stdio_answers_requests_and_ignores_notifications() {
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
        );
        let mut output = Vec::new();
        serve_with(input.as_bytes(), &mut output).unwrap();
        let lines: Vec<&str> = std::str::from_utf8(&output).unwrap().lines().collect();
        assert_eq!(lines.len(), 2);
        let initialized: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(initialized["result"]["serverInfo"]["name"], "parlo");
        let listed: Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(listed["result"]["tools"].as_array().unwrap().len(), 15);
    }
}
