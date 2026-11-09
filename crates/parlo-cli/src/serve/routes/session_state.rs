use std::sync::Mutex;

use parlo_core::state::Machine;

use super::super::http::{error_json, respond_json, Stream};
use super::super::{authorised, json_string, ticket, ws, Flow, Session};

static STATE: Mutex<Machine> = Mutex::new(Machine::new());

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
    session: &Session,
) -> Option<Result<Flow, String>> {
    if method == "POST" && path == "/ticket" {
        if !authorised(headers, token) {
            return Some(
                respond_json(stream, "401 Unauthorized", &error_json("wrong token"))
                    .map(|()| Flow::Serve),
            );
        }
        let (granted, lifetime) = match ticket::issue() {
            Ok(ticket) => ticket,
            Err(error) => {
                return Some(
                    respond_json(stream, "500 Internal Server Error", &error_json(&error))
                        .map(|()| Flow::Serve),
                );
            }
        };
        let answer = serde_json::json!({
            "ticket": granted,
            "expires_in": lifetime,
            "protocol": ws::PROTOCOL,
        })
        .to_string();
        return Some(respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve));
    }

    if method == "GET" && path == "/target" {
        if !authorised(headers, token) {
            return Some(
                respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                    .map(|()| Flow::Serve),
            );
        }
        let answer = match session.target_app() {
            Some(app) => format!("{{\"insertable\":true,\"app\":\"{}\"}}", json_string(&app)),
            None => format!(
                "{{\"insertable\":false,\"reason\":\"{}\"}}",
                json_string("Parlo does not know which field to write into")
            ),
        };
        return Some(respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve));
    }

    if path == "/state" {
        let mut machine = match STATE.lock() {
            Ok(machine) => machine,
            Err(_) => {
                return Some(
                    respond_json(
                        stream,
                        "500 Internal Server Error",
                        "{\"error\":\"the state is unavailable\"}",
                    )
                    .map(|()| Flow::Serve),
                );
            }
        };
        if method == "POST" {
            let next = serde_json::from_slice::<serde_json::Value>(body)
                .ok()
                .and_then(|request| {
                    request
                        .get("state")
                        .and_then(serde_json::Value::as_str)
                        .and_then(parlo_core::state::State::parse)
                });
            let Some(next) = next else {
                return Some(
                    respond_json(stream, "400 Bad Request", "{\"error\":\"unknown state\"}")
                        .map(|()| Flow::Serve),
                );
            };
            if let Err(error) = machine.enter(next) {
                return Some(
                    respond_json(stream, "409 Conflict", &error_json(&error)).map(|()| Flow::Serve),
                );
            }
        }
        let answer = format!("{{\"state\":\"{}\"}}", machine.state().as_str());
        return Some(respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve));
    }

    if method == "GET" && path == "/engine" {
        let stored_compute = crate::config::load().compute;
        let gpu = crate::engines::detect_gpu_memory_mb();
        let choice = crate::engines::choose_preferred(
            crate::engines::detect_cores(),
            gpu,
            crate::engines::Fidelity::Cloned,
            &stored_compute,
            &parlo_core::device::profile().describe(),
        );
        let logged = crate::engines::log_choice(&choice);
        let steps: Vec<&str> = crate::engines::chain(choice.requested)
            .iter()
            .map(|fidelity| fidelity.label())
            .collect();
        let body = serde_json::json!({
            "profile": choice.profile.label(),
            "requested": choice.requested.label(),
            "chosen": choice.chosen.label(),
            "chain": steps,
            "reason": choice.reason,
            "compute": stored_compute,
            "device": crate::engines::device_label(&stored_compute, gpu),
            "log": logged.map(|path| path.display().to_string()),
            "switches": crate::engines::recent_switches(5),
        })
        .to_string();
        return Some(respond_json(stream, "200 OK", &body).map(|()| Flow::Serve));
    }

    None
}
