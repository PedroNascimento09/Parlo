use std::fs;
use std::time::Duration;

use super::super::auth::{authorised, new_token};
use super::super::dub_exports::{
    dub_body, dub_exports, dub_files, projects_body, recall, remember, DubFile,
};
use super::super::http::{error_json, header, percent_decode, respond_json, Stream};
use super::super::{
    apply_dub, apply_project_edits, project_json, respond_file, Flow, ProjectEdit, DUB_TIMEOUT,
};
use crate::dub;
use crate::dub_jobs;
use crate::store::Store as ProjectStore;
use crate::subtitles;

pub(crate) fn route(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Option<Result<Flow, String>> {
    if !(path == "/dub/import" || path == "/dub/projects" || path.starts_with("/dub/")) {
        return None;
    }
    Some(handle(method, path, stream, headers, token, body))
}

fn handle(
    method: &str,
    path: &str,
    stream: &mut dyn Stream,
    headers: &[(String, String)],
    token: &str,
    body: &[u8],
) -> Result<Flow, String> {
    if method == "POST" && path == "/dub/import" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let requested = header(headers, "x-parlo-filename")
            .map(|value| percent_decode(&value))
            .unwrap_or_else(|| "subtitles.srt".to_string());
        let name = dub::safe_name(&requested);
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".srt") && !lower.ends_with(".vtt") {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("choose an .srt or .vtt subtitle file"),
            )
            .map(|()| Flow::Serve);
        }
        let stored = crate::config::load();
        let from = header(headers, "x-parlo-from")
            .unwrap_or_else(|| stored.from.clone())
            .to_ascii_lowercase();
        let to = header(headers, "x-parlo-to")
            .unwrap_or_else(|| stored.to.clone())
            .to_ascii_lowercase();
        if let Err(error) = crate::config::check_pair(&from, &to) {
            return respond_json(stream, "400 Bad Request", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        let id = match new_token() {
            Ok(id) => id,
            Err(error) => {
                return respond_json(stream, "500 Internal Server Error", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        let mut project = match subtitles::project(id, &name, body, &from, &to) {
            Ok(project) => project,
            Err(error) => {
                return respond_json(stream, "400 Bad Request", &error_json(&error))
                    .map(|()| Flow::Serve);
            }
        };
        let mut store = match ProjectStore::open_default() {
            Ok(store) => store,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot open the project store: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        if let Err(error) = store.save_dub_project(&mut project) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot save the imported project: {error}")),
            )
            .map(|()| Flow::Serve);
        }
        return match project_json(&project) {
            Ok(answer) => respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot encode the imported project: {error}")),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "GET" && path.starts_with("/dub/project/") {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = path
            .trim_start_matches("/dub/project/")
            .trim_end_matches('/');
        if id.is_empty() {
            return respond_json(stream, "404 Not Found", &error_json("project not found"))
                .map(|()| Flow::Serve);
        }
        let store = match ProjectStore::open_default() {
            Ok(store) => store,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot open the project store: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        return match store.load_dub_project(id) {
            Ok(Some(project)) => match project_json(&project) {
                Ok(answer) => respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve),
                Err(error) => respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot encode the project: {error}")),
                )
                .map(|()| Flow::Serve),
            },
            Ok(None) => respond_json(stream, "404 Not Found", &error_json("project not found"))
                .map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot load the project: {error}")),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "GET" && path == "/dub/projects" {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        return match projects_body() {
            Ok(body) => respond_json(stream, "200 OK", &body).map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot list the projects: {error}")),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "DELETE" && path.starts_with("/dub/project/") {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = path
            .trim_start_matches("/dub/project/")
            .trim_end_matches('/');
        if id.is_empty() || id.contains('/') {
            return respond_json(stream, "404 Not Found", &error_json("project not found"))
                .map(|()| Flow::Serve);
        }
        let store = match ProjectStore::open_default() {
            Ok(store) => store,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot open the project store: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        let deleted = match store.delete_dub_project(id) {
            Ok(deleted) => deleted,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot remove the project: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        let directory = dub_jobs::directory(id).ok();
        let existed = deleted || directory.as_ref().is_some_and(|path| path.is_dir());
        if let Some(directory) = directory {
            fs::remove_dir_all(directory).ok();
        }
        if let Ok(mut files) = dub_files().lock() {
            files.retain(|file| file.id != id);
        }
        if !existed {
            return respond_json(stream, "404 Not Found", &error_json("project not found"))
                .map(|()| Flow::Serve);
        }
        return respond_json(stream, "200 OK", "{\"removed\":true}").map(|()| Flow::Serve);
    }

    if method == "POST" && path.starts_with("/dub/project/") {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let id = path
            .trim_start_matches("/dub/project/")
            .trim_end_matches('/');
        if id.is_empty() {
            return respond_json(stream, "404 Not Found", &error_json("project not found"))
                .map(|()| Flow::Serve);
        }
        if let Some(project_id) = id.strip_suffix("/recover") {
            if project_id.is_empty() || project_id.contains('/') {
                return respond_json(stream, "404 Not Found", &error_json("project not found"))
                    .map(|()| Flow::Serve);
            }
            let mut store = match ProjectStore::open_default() {
                Ok(store) => store,
                Err(error) => {
                    return respond_json(
                        stream,
                        "500 Internal Server Error",
                        &error_json(&format!("cannot open the project store: {error}")),
                    )
                    .map(|()| Flow::Serve);
                }
            };
            let mut project = match store.load_dub_project(project_id) {
                Ok(Some(project)) if !project.segments.is_empty() => project,
                Ok(Some(_)) => {
                    return respond_json(
                        stream,
                        "409 Conflict",
                        &error_json("the project has no cues to recover"),
                    )
                    .map(|()| Flow::Serve);
                }
                Ok(None) => {
                    return respond_json(stream, "404 Not Found", &error_json("project not found"))
                        .map(|()| Flow::Serve);
                }
                Err(error) => {
                    return respond_json(
                        stream,
                        "500 Internal Server Error",
                        &error_json(&format!("cannot load the project: {error}")),
                    )
                    .map(|()| Flow::Serve);
                }
            };
            let directory = match dub_jobs::directory(project_id) {
                Ok(directory) => directory,
                Err(error) => {
                    return respond_json(stream, "400 Bad Request", &error_json(&error))
                        .map(|()| Flow::Serve);
                }
            };
            let mut job = match dub_jobs::read(&directory) {
                Ok(job) => job,
                Err(error) => {
                    return respond_json(
                        stream,
                        "409 Conflict",
                        &error_json(&format!("the render cannot be recovered: {error}")),
                    )
                    .map(|()| Flow::Serve);
                }
            };
            if job.state == dub_jobs::DONE {
                return respond_json(
                    stream,
                    "409 Conflict",
                    &error_json("the render is already complete"),
                )
                .map(|()| Flow::Serve);
            }
            let Some(source) = dub_jobs::name(&directory, &job.source) else {
                return respond_json(
                    stream,
                    "409 Conflict",
                    &error_json("the saved source file is missing"),
                )
                .map(|()| Flow::Serve);
            };
            let cues = job
                .cues
                .as_deref()
                .and_then(|name| dub_jobs::name(&directory, name));
            if job.cues.is_some() && cues.is_none() {
                return respond_json(
                    stream,
                    "409 Conflict",
                    &error_json("the saved cues file is missing"),
                )
                .map(|()| Flow::Serve);
            }
            let Some(out) = dub_jobs::path(&directory, &job.out) else {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json("the saved output name is invalid"),
                )
                .map(|()| Flow::Serve);
            };
            let Some(srt) = dub_jobs::path(&directory, &job.srt) else {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json("the saved subtitle name is invalid"),
                )
                .map(|()| Flow::Serve);
            };
            dub_jobs::clean_outputs(&directory);
            job.state = dub_jobs::RENDERING.to_string();
            job.stage = "synthesizing".to_string();
            job.progress = 0.05;
            job.error = None;
            if let Err(error) = dub_jobs::write(&directory, &mut job) {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot save the recovery: {error}")),
                )
                .map(|()| Flow::Serve);
            }
            let track = directory.join(format!("{}-track.wav", job.id));
            let lines_dir = directory.join("lines");
            let report = directory.join(format!("{}-report.json", job.id));
            let work = directory.join("work");
            let _ = stream.set_write_timeout(Some(DUB_TIMEOUT));
            let result = dub::run(&dub::Request {
                source: &source,
                from: &job.from,
                to: &job.to,
                out: Some(out.clone()),
                srt: Some(srt.clone()),
                cues: cues.clone(),
                track: Some(track.clone()),
                lines_dir: Some(lines_dir.clone()),
                work: Some(work.clone()),
                report: Some(report.clone()),
                previous: None,
                retry: None,
                skip: None,
                keep_background: job.keep_background,
                timing: &job.timing,
                fastest: job.fastest,
                slowest: job.slowest,
            });
            let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
            return match result {
                Ok(result) => {
                    apply_dub(&mut project, &result.lines, &job.to);
                    if let Err(error) = store.save_dub_project(&mut project) {
                        return respond_json(
                            stream,
                            "500 Internal Server Error",
                            &error_json(&format!("the project could not be saved: {error}")),
                        )
                        .map(|()| Flow::Serve);
                    }
                    let written = dub_exports(&result.exports);
                    job.state = dub_jobs::DONE.to_string();
                    job.stage = "done".to_string();
                    job.progress = 1.0;
                    job.seconds = result.seconds;
                    job.lines = result.lines.len();
                    job.separation = result.separation.clone();
                    job.separation_note = result.separation_note.clone();
                    job.error = None;
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
                            &error_json(&format!("cannot save the recovered render: {error}")),
                        )
                        .map(|()| Flow::Serve);
                    }
                    fs::remove_dir_all(&work).ok();
                    let file = DubFile {
                        id: job.id.clone(),
                        job_dir: directory.clone(),
                        upload: source,
                        dubbed: out,
                        srt,
                        track,
                        lines_dir,
                        report,
                        cues,
                        name: job.out.clone(),
                        from: job.from.clone(),
                        to: job.to.clone(),
                        timing: job.timing.clone(),
                        fastest: job.fastest,
                        slowest: job.slowest,
                        project_id: Some(project.id.clone()),
                        keep_background: job.keep_background,
                        separation: job.separation.clone(),
                        separation_note: job.separation_note.clone(),
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
                        eprintln!("cannot save the failed recovery: {write_error}");
                    }
                    respond_json(stream, "500 Internal Server Error", &error_json(&error))
                        .map(|()| Flow::Serve)
                }
            };
        }
        let edit: ProjectEdit = match serde_json::from_slice(body) {
            Ok(edit) => edit,
            Err(error) => {
                return respond_json(
                    stream,
                    "400 Bad Request",
                    &error_json(&format!("the project edit is not valid JSON: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        let mut store = match ProjectStore::open_default() {
            Ok(store) => store,
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot open the project store: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        let mut project = match store.load_dub_project(id) {
            Ok(Some(project)) => project,
            Ok(None) => {
                return respond_json(stream, "404 Not Found", &error_json("project not found"))
                    .map(|()| Flow::Serve);
            }
            Err(error) => {
                return respond_json(
                    stream,
                    "500 Internal Server Error",
                    &error_json(&format!("cannot load the project: {error}")),
                )
                .map(|()| Flow::Serve);
            }
        };
        if let Err(error) = apply_project_edits(&mut project, &edit.segments, &edit.cast) {
            return respond_json(stream, "400 Bad Request", &error_json(&error))
                .map(|()| Flow::Serve);
        }
        if let Err(error) = store.save_dub_project(&mut project) {
            return respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot save the project: {error}")),
            )
            .map(|()| Flow::Serve);
        }
        return match project_json(&project) {
            Ok(answer) => respond_json(stream, "200 OK", &answer).map(|()| Flow::Serve),
            Err(error) => respond_json(
                stream,
                "500 Internal Server Error",
                &error_json(&format!("cannot encode the project: {error}")),
            )
            .map(|()| Flow::Serve),
        };
    }

    if method == "POST" && path.starts_with("/dub/") {
        if !authorised(headers, token) {
            return respond_json(stream, "401 Unauthorized", "{\"error\":\"wrong token\"}")
                .map(|()| Flow::Serve);
        }
        let rest = path.trim_start_matches("/dub/");
        let Some((id, action)) = rest.rsplit_once('/') else {
            return respond_json(stream, "404 Not Found", &error_json("not found"))
                .map(|()| Flow::Serve);
        };
        if action != "retry" && action != "skip" {
            return respond_json(stream, "404 Not Found", &error_json("not found"))
                .map(|()| Flow::Serve);
        }
        let Some(line_id) = header(headers, "x-parlo-line")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        else {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("a line id is required"),
            )
            .map(|()| Flow::Serve);
        };
        let state = match dub_files().lock() {
            Ok(files) => files.iter().find(|file| file.id == id).cloned(),
            Err(_) => None,
        };
        let Some(state) = state else {
            return respond_json(stream, "404 Not Found", &error_json("not found"))
                .map(|()| Flow::Serve);
        };
        if !state
            .lines
            .iter()
            .any(|line| line.id.as_deref() == Some(line_id.as_str()))
        {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json(&format!("line {line_id} is not part of this render")),
            )
            .map(|()| Flow::Serve);
        }
        if state.cues.is_none() {
            return respond_json(
                stream,
                "400 Bad Request",
                &error_json("retrying a line needs imported subtitles"),
            )
            .map(|()| Flow::Serve);
        }
        let retry = (action == "retry").then(|| line_id.clone());
        let skip = (action == "skip").then(|| line_id.clone());
        let _ = stream.set_write_timeout(Some(DUB_TIMEOUT));
        let result = dub::run(&dub::Request {
            source: &state.upload,
            from: &state.from,
            to: &state.to,
            out: Some(state.dubbed.clone()),
            srt: Some(state.srt.clone()),
            cues: state.cues.clone(),
            track: Some(state.track.clone()),
            lines_dir: Some(state.lines_dir.clone()),
            work: Some(state.job_dir.join("work")),
            report: Some(state.report.clone()),
            previous: Some(state.report.clone()),
            retry,
            skip,
            keep_background: state.keep_background,
            timing: &state.timing,
            fastest: state.fastest,
            slowest: state.slowest,
        });
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
        return match result {
            Ok(result) => {
                let mut updated = state;
                updated.lines = result.lines;
                updated.seconds = result.seconds;
                updated.exports = dub_exports(&result.exports);
                if let Ok(mut job) = dub_jobs::read(&updated.job_dir) {
                    job.state = dub_jobs::DONE.to_string();
                    job.stage = "done".to_string();
                    job.progress = 1.0;
                    job.seconds = updated.seconds;
                    job.lines = updated.lines.len();
                    job.separation = updated.separation.clone();
                    job.separation_note = updated.separation_note.clone();
                    job.error = None;
                    job.exports = updated
                        .exports
                        .iter()
                        .map(|item| dub_jobs::Export {
                            format: item.format.clone(),
                            label: item.label.clone(),
                            name: item.name.clone(),
                        })
                        .collect();
                    if let Err(error) = dub_jobs::write(&updated.job_dir, &mut job) {
                        eprintln!("cannot save the line change: {error}");
                    }
                }
                if let Ok(mut files) = dub_files().lock() {
                    if let Some(slot) = files.iter_mut().find(|file| file.id == updated.id) {
                        *slot = updated.clone();
                    }
                }
                if let Some(project_id) = updated.project_id.as_deref() {
                    if let Ok(mut store) = ProjectStore::open_default() {
                        if let Ok(Some(mut project)) = store.load_dub_project(project_id) {
                            apply_dub(&mut project, &updated.lines, &updated.to);
                            if let Err(error) = store.save_dub_project(&mut project) {
                                return respond_json(
                                    stream,
                                    "500 Internal Server Error",
                                    &error_json(&format!(
                                        "the line changed, but the project could not be saved: {error}"
                                    )),
                                )
                                .map(|()| Flow::Serve);
                            }
                        }
                    }
                }
                respond_json(stream, "200 OK", &dub_body(&updated)).map(|()| Flow::Serve)
            }
            Err(error) => respond_json(stream, "500 Internal Server Error", &error_json(&error))
                .map(|()| Flow::Serve),
        };
    }

    if method == "GET" && path.starts_with("/dub/") {
        let rest = path.trim_start_matches("/dub/");
        if let Some((id, format)) = rest.split_once("/export/") {
            let found = match dub_files().lock() {
                Ok(files) => files.iter().find(|file| file.id == id).and_then(|file| {
                    file.exports
                        .iter()
                        .find(|item| item.format == format)
                        .map(|item| (item.path.clone(), item.name.clone()))
                }),
                Err(_) => None,
            };
            let found = found.or_else(|| {
                dub_jobs::directory(id)
                    .ok()
                    .and_then(|directory| dub_jobs::export_path(&directory, format))
            });
            let Some((file, name)) = found else {
                return respond_json(stream, "404 Not Found", &error_json("not found"))
                    .map(|()| Flow::Serve);
            };
            return respond_file(stream, &file, &name).map(|()| Flow::Serve);
        }
        let id = rest;
        let Some((file, name)) = recall(id) else {
            return respond_json(stream, "404 Not Found", "{\"error\":\"not found\"}")
                .map(|()| Flow::Serve);
        };
        return respond_file(stream, &file, &name).map(|()| Flow::Serve);
    }
    respond_json(stream, "404 Not Found", "{\"error\":\"not found\"}").map(|()| Flow::Serve)
}
