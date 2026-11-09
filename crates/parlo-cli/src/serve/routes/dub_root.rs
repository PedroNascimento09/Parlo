use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::super::auth::{authorised, new_token};
use super::super::dub_exports::{dub_body, dub_exports, remember, unix_now, DubFile};
use super::super::http::{error_json, header, percent_decode, respond_json, Stream};
use super::super::{
    apply_dub, dubbed_name, json_string, load_requested_project, write_cues, Flow, DUB_TIMEOUT,
};
use crate::dub;
use crate::dub_jobs;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    if !(method == "POST" && path == "/dub") {
        return None;
    }
    Some(handle_root(stream, headers, token, body))
}

fn handle_root(
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Result<Flow, String> {
    if !authorised(headers, token) {
        return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
            .map(|()| Flow::Serve);
    }
    if !crate::config::load().features.translation {
        return respond_json(
            stream,
            "409 Conflict",
            "{\"error\":\"dubbing needs translation, and translation is switched off in Settings\"}",
        )
        .map(|()| Flow::Serve);
    }
    let requested_url = header(headers, "x-parlo-url")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let cookies = header(headers, "x-parlo-cookies")
        .map(|value| PathBuf::from(percent_decode(&value)))
        .filter(|path| !path.as_os_str().is_empty());
    if requested_url.is_some() && !body.is_empty() {
        return respond_json(
            stream,
            "400 Bad Request",
            &error_json("send either a file or a link, not both"),
        )
        .map(|()| Flow::Serve);
    }
    if body.is_empty() && requested_url.is_none() {
        return respond_json(
            stream,
            "400 Bad Request",
            "{\"error\":\"the file is empty\"}",
        )
        .map(|()| Flow::Serve);
    }

    let stored = crate::config::load();
    let (mut project, mut project_store) = match load_requested_project(headers) {
        Ok(Some(loaded)) => (Some(loaded.project), Some(loaded.store)),
        Ok(None) => (None, None),
        Err((reason, message)) => {
            return respond_json(stream, reason, &error_json(&message)).map(|()| Flow::Serve);
        }
    };
    let (from, to) = match project.as_ref() {
        Some(project) => (
            project.from.clone(),
            header(headers, "x-parlo-to")
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| project.to.clone()),
        ),
        None => (
            header(headers, "x-parlo-from")
                .unwrap_or_else(|| stored.from.clone())
                .to_ascii_lowercase(),
            header(headers, "x-parlo-to")
                .unwrap_or_else(|| stored.to.clone())
                .to_ascii_lowercase(),
        ),
    };
    if let Err(error) = crate::config::check_pair(&from, &to) {
        return respond_json(
            stream,
            "400 Bad Request",
            &format!("{{\"error\":\"{}\"}}", json_string(&error)),
        )
        .map(|()| Flow::Serve);
    }
    let timing = header(headers, "x-parlo-timing")
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "natural".to_string());
    let fastest = match header(headers, "x-parlo-fastest") {
        Some(value) => match value.trim().parse::<f64>() {
            Ok(value) => value,
            Err(_) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json("fastest must be a number"),
                )
                .map(|()| Flow::Serve);
            }
        },
        None => 1.25,
    };
    let slowest = match header(headers, "x-parlo-slowest") {
        Some(value) => match value.trim().parse::<f64>() {
            Ok(value) => value,
            Err(_) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json("slowest must be a number"),
                )
                .map(|()| Flow::Serve);
            }
        },
        None => 0.9,
    };
    if let Err(error) = dub::check_timing(&timing, fastest, slowest) {
        return respond_json(stream, "400 Bad Request", &error_json(&error)).map(|()| Flow::Serve);
    }

    let requested = header(headers, "x-parlo-filename")
        .map(|value| percent_decode(&value))
        .unwrap_or_else(|| match &requested_url {
            Some(url) => crate::ingest::name_from_url(url),
            None => "upload.mp4".to_string(),
        });
    let name = dub::safe_name(&requested);
    let project_id = project.as_ref().map(|project| project.id.clone());
    let id = match project_id.clone() {
        Some(id) => id,
        None => match new_token() {
            Ok(id) => id,
            Err(error) => {
                return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        },
    };
    let directory = match dub_jobs::directory(&id) {
        Ok(directory) => directory,
        Err(error) => {
            return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve);
        }
    };
    if let Err(error) = fs::create_dir_all(&directory) {
        return respond_json(
            stream,
            "500 Internal Server Error",
            &format!(
                "{{\"error\":\"cannot create {}: {}\"}}",
                json_string(&directory.display().to_string()),
                json_string(&error.to_string())
            ),
        )
        .map(|()| Flow::Serve);
    }
    dub_jobs::clean_outputs(&directory);
    let source_name = dub_jobs::source_name(&name);

    let cues = match write_cues(&directory, &id, project.as_ref()) {
        Ok(cues) => cues,
        Err(error) => {
            return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve);
        }
    };

    let upload = directory.join(&source_name);
    if let Some(url) = &requested_url {
        if let Err(failure) = crate::ingest::download(url, &upload, cookies.as_deref()) {
            if let Some(path) = &cues {
                fs::remove_file(path).ok();
            }
            return respond_json(stream, failure.status, &error_json(&failure.message))
                .map(|()| Flow::Serve);
        }
        if let Err(reason) = crate::ingest::probe(&upload) {
            fs::remove_file(&upload).ok();
            if let Some(path) = &cues {
                fs::remove_file(path).ok();
            }
            return respond_json(stream, "400 Bad Request", &error_json(&reason))
                .map(|()| Flow::Serve);
        }
    } else if let Err(error) = fs::write(&upload, body) {
        return respond_json(
            stream,
            "500 Internal Server Error",
            &format!(
                "{{\"error\":\"cannot store the upload: {}\"}}",
                json_string(&error.to_string())
            ),
        )
        .map(|()| Flow::Serve);
    }

    if timing == "lip-sync" {
        if let Ok(false) = crate::ingest::has_video(&upload) {
            fs::remove_file(&upload).ok();
            if let Some(path) = &cues {
                fs::remove_file(path).ok();
            }
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("lip sync needs a video file with a picture"),
            )
            .map(|()| Flow::Serve);
        }
    }

    let out_name = dubbed_name(&name, &to);
    let dubbed = directory.join(format!("{id}-{out_name}"));
    let subtitles = dubbed.with_extension("srt");
    let track = directory.join(format!("{id}-track.wav"));
    let lines_dir = directory.join(format!("{id}-lines"));
    let report = directory.join(format!("{id}-report.json"));
    let work = directory.join("work");
    let keep_background = header(headers, "x-parlo-background").is_some();
    let mut job = dub_jobs::Job {
        id: id.clone(),
        project_id: project_id.clone(),
        title: project
            .as_ref()
            .map(|project| project.title.clone())
            .unwrap_or_else(|| name.clone()),
        source: source_name.clone(),
        cues: cues
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string()),
        out: dubbed
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        srt: subtitles
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default(),
        state: dub_jobs::RENDERING.to_string(),
        stage: "synthesizing".to_string(),
        progress: 0.05,
        error: None,
        from: from.clone(),
        to: to.clone(),
        timing: timing.clone(),
        fastest,
        slowest,
        keep_background,
        separation: "off".to_string(),
        separation_note: None,
        seconds: 0.0,
        lines: 0,
        created_at: unix_now(),
        updated_at: 0,
        exports: Vec::new(),
    };
    if let Err(error) = dub_jobs::write(&directory, &mut job) {
        return respond_json(
            stream,
            "500 Internal Server Error",
            &error_json(&format!("cannot save the render: {error}")),
        )
        .map(|()| Flow::Serve);
    }
    let _ = stream.set_write_timeout(Some(DUB_TIMEOUT));
    println!("dubbing {name} from {from} to {to}");
    let result = dub::run(&dub::Request {
        source: &upload,
        from: &from,
        to: &to,
        out: Some(dubbed.clone()),
        srt: Some(subtitles.clone()),
        cues: cues.clone(),
        track: Some(track.clone()),
        lines_dir: Some(lines_dir.clone()),
        work: Some(work.clone()),
        report: Some(report.clone()),
        previous: None,
        retry: None,
        skip: None,
        keep_background,
        timing: &timing,
        fastest,
        slowest,
    });
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

    match result {
        Ok(result) => {
            if let (Some(project), Some(store)) = (project.as_mut(), project_store.as_mut()) {
                apply_dub(project, &result.lines, &to);
                if let Err(error) = store.save_dub_project(project) {
                    return respond_json(
                        stream,
                        "500 Internal Server Error",
                        &error_json(&format!(
                            "the dub finished, but the project could not be saved: {error}"
                        )),
                    )
                    .map(|()| Flow::Serve);
                }
            }
            let written = dub_exports(&result.exports);
            fs::remove_dir_all(&work).ok();
            job.state = dub_jobs::DONE.to_string();
            job.stage = "done".to_string();
            job.progress = 1.0;
            job.seconds = result.seconds;
            job.lines = result.lines.len();
            job.separation = result.separation.clone();
            job.separation_note = result.separation_note.clone();
            job.exports = written
                .iter()
                .map(|item| dub_jobs::Export {
                    format: item.format.clone(),
                    label: item.label.clone(),
                    name: item.name.clone(),
                })
                .collect();
            if let Err(error) = dub_jobs::write(&directory, &mut job) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot save the finished render: {error}")),
                )
                .map(|()| Flow::Serve);
            }
            let file = DubFile {
                id: id.clone(),
                job_dir: directory.clone(),
                upload,
                dubbed,
                srt: subtitles,
                track,
                lines_dir,
                report,
                cues,
                name: out_name,
                from,
                to,
                timing,
                fastest,
                slowest,
                project_id,
                keep_background,
                separation: result.separation.clone(),
                separation_note: result.separation_note.clone(),
                seconds: result.seconds,
                lines: result.lines,
                exports: written,
            };
            let body = dub_body(&file);
            remember(file);
            respond_json(stream, "200 OK", &body).map(|()| Flow::Serve)
        }
        Err(error) => {
            job.state = dub_jobs::FAILED.to_string();
            job.stage = "stopped".to_string();
            job.error = Some(error.clone());
            if let Err(write_error) = dub_jobs::write(&directory, &mut job) {
                eprintln!("cannot save the failed render: {write_error}");
            }
            respond_json(
                stream,
                "500 Internal Server Error",
                &format!("{{\"error\":\"{}\"}}", json_string(&error)),
            )
            .map(|()| Flow::Serve)
        }
    }
}
