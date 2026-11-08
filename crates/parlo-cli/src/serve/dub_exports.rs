use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use super::auth;
use super::{header, json_string, origin_allowed, KEPT_DUBS};
use crate::dub;
use crate::dub_jobs;
use crate::store::Store as ProjectStore;

#[derive(Clone)]
pub(super) struct DubExport {
    pub(super) format: String,
    pub(super) label: String,
    pub(super) name: String,
    pub(super) path: PathBuf,
}

#[derive(Clone)]
pub(super) struct DubFile {
    pub(super) id: String,
    pub(super) job_dir: PathBuf,
    pub(super) upload: PathBuf,
    pub(super) dubbed: PathBuf,
    pub(super) srt: PathBuf,
    pub(super) track: PathBuf,
    pub(super) lines_dir: PathBuf,
    pub(super) report: PathBuf,
    pub(super) cues: Option<PathBuf>,
    pub(super) name: String,
    pub(super) from: String,
    pub(super) to: String,
    pub(super) timing: String,
    pub(super) fastest: f64,
    pub(super) slowest: f64,
    pub(super) project_id: Option<String>,
    pub(super) keep_background: bool,
    pub(super) separation: String,
    pub(super) separation_note: Option<String>,
    pub(super) seconds: f64,
    pub(super) lines: Vec<dub::Line>,
    pub(super) exports: Vec<DubExport>,
}

pub(super) fn dub_files() -> &'static Mutex<Vec<DubFile>> {
    static FILES: OnceLock<Mutex<Vec<DubFile>>> = OnceLock::new();
    FILES.get_or_init(|| Mutex::new(Vec::new()))
}

pub(super) fn remember(file: DubFile) {
    let mut files = match dub_files().lock() {
        Ok(files) => files,
        Err(_) => return,
    };
    files.push(file);
    while files.len() > KEPT_DUBS {
        let oldest = files.remove(0);
        forget_dub(&oldest);
    }
}

pub(super) fn recall(id: &str) -> Option<(PathBuf, String)> {
    if let Ok(files) = dub_files().lock() {
        if let Some(file) = files.iter().find(|file| file.id == id) {
            return Some((file.dubbed.clone(), file.name.clone()));
        }
    }
    let directory = dub_jobs::directory(id).ok()?;
    let job = dub_jobs::read(&directory).ok()?;
    if job.state != dub_jobs::DONE {
        return None;
    }
    let path = dub_jobs::name(&directory, &job.out)?;
    Some((path, job.out))
}

pub(super) fn dub_exports(items: &[dub::Export]) -> Vec<DubExport> {
    items
        .iter()
        .filter_map(|item| {
            let path = PathBuf::from(&item.path);
            let name = path.file_name()?.to_string_lossy().to_string();
            Some(DubExport {
                format: item.format.clone(),
                label: item.label.clone(),
                name,
                path,
            })
        })
        .collect()
}

pub(super) fn dub_body(file: &DubFile) -> String {
    let lines: Vec<String> = file
        .lines
        .iter()
        .map(|line| serde_json::to_string(line).unwrap_or_else(|_| "null".to_string()))
        .collect();
    let project = file
        .project_id
        .as_ref()
        .map(|id| format!(",\"project_id\":\"{}\"", json_string(id)))
        .unwrap_or_default();
    let written: Vec<String> = file
        .exports
        .iter()
        .map(|item| {
            format!(
                "{{\"format\":\"{}\",\"label\":\"{}\",\"name\":\"{}\",\"url\":\"/dub/{}/export/{}\"}}",
                json_string(&item.format),
                json_string(&item.label),
                json_string(&item.name),
                json_string(&file.id),
                json_string(&item.format)
            )
        })
        .collect();
    format!(
        "{{\"id\":\"{}\",\"name\":\"{}\",\"to\":\"{}\",\"timing\":\"{}\",\"fastest\":{},\"slowest\":{},\"seconds\":{:.2},\"separation\":\"{}\",\"separation_note\":{},\"lines\":[{}],\"exports\":[{}]{}}}",
        json_string(&file.id),
        json_string(&file.name),
        json_string(&file.to),
        json_string(&file.timing),
        file.fastest,
        file.slowest,
        file.seconds,
        json_string(&file.separation),
        file.separation_note
            .as_ref()
            .map(|note| format!("\"{}\"", json_string(note)))
            .unwrap_or_else(|| "null".to_string()),
        lines.join(","),
        written.join(","),
        project
    )
}

pub(super) fn projects_body() -> Result<String, String> {
    let store = ProjectStore::open_default()?;
    let projects = store.list_dub_projects()?;
    let mut values = Vec::new();
    for project in projects {
        let mut status = "draft";
        let mut error = project.error.clone();
        let mut job_id = None;
        let mut seconds = 0.0;
        let mut exports = Vec::new();
        if project.invalid {
            status = "invalid";
        } else {
            match dub_jobs::directory(&project.id) {
                Err(reason) => {
                    status = "invalid";
                    error = Some(reason);
                }
                Ok(directory) => match dub_jobs::read(&directory) {
                    Ok(job) => {
                        job_id = Some(job.id.clone());
                        if job.state == dub_jobs::RENDERING {
                            status = "interrupted";
                            error = Some("the render stopped before it finished".to_string());
                        } else if job.state == dub_jobs::FAILED {
                            status = "failed";
                            error = job.error.clone();
                        } else if job.state == dub_jobs::DONE
                            && dub_jobs::name(&directory, &job.out).is_some()
                        {
                            status = "done";
                            seconds = job.seconds;
                            exports = job
                                .exports
                                .iter()
                                .map(|item| {
                                    serde_json::json!({
                                        "format": item.format,
                                        "label": item.label,
                                        "name": item.name,
                                        "url": format!("/dub/{}/export/{}", job.id, item.format),
                                    })
                                })
                                .collect();
                        } else {
                            status = "invalid";
                            error = Some("the finished render is missing its file".to_string());
                        }
                    }
                    Err(reason) if dub_jobs::has_manifest(&directory) => {
                        status = "invalid";
                        error = Some(reason);
                    }
                    Err(_) => {}
                },
            }
        }
        values.push(serde_json::json!({
            "id": project.id,
            "job_id": job_id,
            "title": project.title,
            "from": project.from,
            "to": project.to,
            "status": status,
            "seconds": seconds,
            "segments": project.segments,
            "updated_at": project.updated_at,
            "error": error,
            "exports": exports,
        }));
    }
    Ok(serde_json::json!({ "projects": values }).to_string())
}

pub(super) fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

pub(super) fn forget_dub(file: &DubFile) {
    if file.project_id.is_some() {
        return;
    }
    discard_dub(file);
}

pub(super) fn discard_dub(file: &DubFile) {
    fs::remove_dir_all(&file.job_dir).ok();
}

pub(super) struct Share {
    pub(super) pin: String,
    pub(super) port: u16,
}

pub(super) fn share_target(config: &parlo_core::settings::Config) -> Option<(String, u16, String)> {
    if !config.share || config.share_port < 1024 {
        return None;
    }
    let pin = parlo_core::settings::check_pin(&config.share_pin).ok()?;
    Some(("0.0.0.0".to_string(), config.share_port, pin))
}

pub(super) fn host_is_share(headers: &[(String, String)], port: u16) -> bool {
    let Some(host) = header(headers, "host") else {
        return false;
    };
    let host = host.trim();
    let (name, found) = match host.rsplit_once(':') {
        Some((name, value)) => (name, value.parse::<u16>().ok()),
        None => (host, None),
    };
    if found != Some(port) {
        return false;
    }
    let bare = name.trim_start_matches('[').trim_end_matches(']');
    bare.parse::<std::net::IpAddr>().is_ok()
        || bare.eq_ignore_ascii_case("localhost")
        || bare.to_ascii_lowercase().ends_with(".local")
}

pub(super) fn pin_offered(headers: &[(String, String)], query: &str, pin: &str) -> bool {
    let offered = header(headers, "x-parlo-pin").or_else(|| {
        query.split('&').find_map(|pair| {
            let (name, value) = pair.split_once('=')?;
            (name == "pin").then(|| value.to_string())
        })
    });
    offered.is_some_and(|value| auth::constant_time_eq(value.trim(), pin))
}

pub(super) fn origin_allowed_for(headers: &[(String, String)], port: u16, open: bool) -> bool {
    if !open {
        return origin_allowed(headers, port);
    }
    let Some(origin) = header(headers, "origin") else {
        return true;
    };
    let Some(host) = header(headers, "host") else {
        return false;
    };
    let origin = origin.trim_end_matches('/');
    origin == format!("http://{host}") || origin == format!("https://{host}")
}
