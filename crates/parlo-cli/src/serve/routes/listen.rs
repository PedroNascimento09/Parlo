use super::super::http::{error_json, respond_json, Stream};
use parlo_core::wav::read_pcm16;

use super::super::{authorised, header, pieces_json, Flow, Session};

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Option<Result<Flow, String>> {
    if method == "POST" && (path == "/listen/stream" || path == "/listen/stream-end") {
        if !authorised(headers, token) {
            return Some(
                respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                    .map(|()| Flow::Serve),
            );
        }
        if path == "/listen/stream-end" {
            return Some(match session.finish_stream() {
                Ok(pieces) => {
                    respond_json(stream, "200 OK", &pieces_json(&pieces)).map(|()| Flow::Serve)
                }
                Err(error) => {
                    respond_json(stream, "500 Internal Server Error", &error_json(&error))
                        .map(|()| Flow::Serve)
                }
            });
        }
        let wav = match read_pcm16(body) {
            Ok(wav) => wav,
            Err(error) => {
                return Some(
                    respond_json(
                        stream,
                        "400 Bad Request",
                        &format!("{{\"error\":\"bad wav: {error}\"}}"),
                    )
                    .map(|()| Flow::Serve),
                );
            }
        };
        if wav.format.channels() != 1 || wav.format.sample_rate() != 16_000 {
            return Some(
                respond_json(
                    stream,
                    "400 Bad Request",
                    "{\"error\":\"input must be 16 kHz mono pcm16\"}",
                )
                .map(|()| Flow::Serve),
            );
        }
        if wav.samples.is_empty() {
            return Some(respond_json(stream, "200 OK", "[]").map(|()| Flow::Serve));
        }
        let segment = match header(headers, "x-parlo-window") {
            Some(raw) => {
                let parsed = raw.trim().parse::<f32>();
                match parsed {
                    Ok(value) if (0.4..=10.0).contains(&value) => Some(value),
                    _ => {
                        return Some(
                            respond_json(
                                stream,
                                "400 Bad Request",
                                "{\"error\":\"x-parlo-window needs seconds between 0.4 and 10\"}",
                            )
                            .map(|()| Flow::Serve),
                        );
                    }
                }
            }
            None => None,
        };
        return Some(match session.stream_window(&wav.samples, segment) {
            Ok(pieces) => {
                respond_json(stream, "200 OK", &pieces_json(&pieces)).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        });
    }

    None
}
