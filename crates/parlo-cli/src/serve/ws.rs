use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tungstenite::handshake::derive_accept_key;
use tungstenite::protocol::{Message, Role, WebSocket, WebSocketConfig};

use super::auth::authorised;
use super::http::{error_json, header, origin_allowed, respond_json};
use super::{Flow, Session};
use crate::serve::http::Stream;

pub(crate) const PROTOCOL: &str = "parlo.speech.v1";
pub(crate) const MAX_MESSAGE: usize = 4 * 1024 * 1024;
const SOCKET_TIMEOUT: Duration = Duration::from_secs(3_600);

static SESSIONS: AtomicU64 = AtomicU64::new(0);

pub(crate) fn stream(
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    query_token: &str,
    query_ticket: &str,
    port: u16,
    session: &Session,
) -> Result<Flow, String> {
    if !authorised(headers, token) && query_token != token && !super::ticket::redeem(query_ticket) {
        return respond_json(stream, "401 Unauthorized", &error_json("wrong token"))
            .map(|()| Flow::Serve);
    }
    if !origin_allowed(headers, port) {
        return respond_json(
            stream,
            "403 Forbidden",
            &error_json("origin is not the local app"),
        )
        .map(|()| Flow::Serve);
    }
    if !header(headers, "upgrade")
        .map(|value| value.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
    {
        return respond_json(
            stream,
            "400 Bad Request",
            &error_json("this endpoint needs a websocket upgrade"),
        )
        .map(|()| Flow::Serve);
    }
    let version = header(headers, "sec-websocket-version").unwrap_or_default();
    if version.trim() != "13" {
        return respond_json(
            stream,
            "400 Bad Request",
            &error_json("the websocket handshake needs version 13"),
        )
        .map(|()| Flow::Serve);
    }
    let key = header(headers, "sec-websocket-key").unwrap_or_default();
    if key.trim().is_empty() {
        return respond_json(
            stream,
            "400 Bad Request",
            &error_json("the websocket handshake needs a key"),
        )
        .map(|()| Flow::Serve);
    }

    let accept = derive_accept_key(key.trim().as_bytes());
    let answer = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n"
    );
    stream
        .write_all(answer.as_bytes())
        .and_then(|()| stream.flush())
        .map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(SOCKET_TIMEOUT))
        .map_err(|error| error.to_string())?;

    let config = WebSocketConfig {
        max_message_size: Some(MAX_MESSAGE),
        max_frame_size: Some(MAX_MESSAGE),
        ..Default::default()
    };
    let mut socket = WebSocket::from_raw_socket(stream, Role::Server, Some(config));
    let id = session_id();
    send(
        &mut socket,
        serde_json::json!({
            "type": "session.started",
            "protocol": PROTOCOL,
            "session_id": id,
        }),
    )?;

    let mut pending = Vec::new();
    loop {
        let message = match socket.read() {
            Ok(message) => message,
            Err(
                tungstenite::Error::ConnectionClosed
                | tungstenite::Error::Protocol(
                    tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
                ),
            ) => break,
            Err(error) => return Err(error.to_string()),
        };
        match message {
            Message::Binary(bytes) => {
                pending.extend_from_slice(&bytes);
                let usable = pending.len() - pending.len() % 2;
                if usable == 0 {
                    continue;
                }
                let samples: Vec<f32> = pending[..usable]
                    .chunks_exact(2)
                    .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32_768.0)
                    .collect();
                pending.drain(..usable);
                match session.stream_window(&samples, None) {
                    Ok(pieces) => emit(&mut socket, &id, &pieces)?,
                    Err(error) => send_error(&mut socket, &id, &error)?,
                }
            }
            Message::Text(text) => match control(&text) {
                Control::End => match session.finish_stream() {
                    Ok(pieces) => {
                        emit(&mut socket, &id, &pieces)?;
                        send(
                            &mut socket,
                            serde_json::json!({
                                "type": "input_audio.end",
                                "session_id": id,
                            }),
                        )?;
                    }
                    Err(error) => send_error(&mut socket, &id, &error)?,
                },
                Control::Ping => send(
                    &mut socket,
                    serde_json::json!({ "type": "pong", "session_id": id }),
                )?,
                Control::Unknown => send_error(
                    &mut socket,
                    &id,
                    "text frames carry input_audio.end or ping",
                )?,
            },
            Message::Close(_) => {
                let _ = socket.close(None);
                break;
            }
            Message::Ping(payload) => {
                socket
                    .send(Message::Pong(payload))
                    .map_err(|error| error.to_string())?;
            }
            Message::Pong(_) | Message::Frame(_) => {}
        }
    }
    Ok(Flow::Serve)
}

enum Control {
    End,
    Ping,
    Unknown,
}

fn control(text: &str) -> Control {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return Control::Unknown;
    };
    match value.get("type").and_then(serde_json::Value::as_str) {
        Some("input_audio.end") => Control::End,
        Some("ping") => Control::Ping,
        _ => Control::Unknown,
    }
}

fn emit(
    socket: &mut WebSocket<&mut dyn Stream>,
    id: &str,
    pieces: &[parlo_core::engine::StreamPiece],
) -> Result<(), String> {
    for piece in pieces {
        send(
            socket,
            serde_json::json!({
                "type": piece.state,
                "text": piece.text,
                "session_id": id,
                "t0": piece.audio_ms.saturating_sub(piece.latency_ms),
                "t1": piece.audio_ms,
            }),
        )?;
    }
    Ok(())
}

fn send_error(
    socket: &mut WebSocket<&mut dyn Stream>,
    id: &str,
    message: &str,
) -> Result<(), String> {
    send(
        socket,
        serde_json::json!({
            "type": "error",
            "message": message,
            "session_id": id,
        }),
    )
}

fn send(socket: &mut WebSocket<&mut dyn Stream>, value: serde_json::Value) -> Result<(), String> {
    socket
        .send(Message::Text(value.to_string()))
        .map_err(|error| error.to_string())
}

fn session_id() -> String {
    let count = SESSIONS.fetch_add(1, Ordering::Relaxed);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();
    format!("{millis:x}-{count:x}")
}
