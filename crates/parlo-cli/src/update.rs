use std::cmp::Ordering;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

pub(crate) const LATEST: &str =
    "https://api.github.com/repos/PedroNascimento09/Parlo/releases/latest";
pub(crate) const RELEASES: &str =
    "https://api.github.com/repos/PedroNascimento09/Parlo/releases?per_page=30";
pub const CHANNELS: [&str; 2] = ["stable", "preview"];

pub struct Latest {
    pub version: String,
    pub notes_url: String,
    pub archive_url: Option<String>,
}

pub fn archive_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "parlo-windows-x86_64.zip"
    } else if cfg!(target_os = "macos") {
        "parlo-macos-aarch64.tar.gz"
    } else {
        "parlo-linux-x86_64.tar.gz"
    }
}

pub fn binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "parlo.exe"
    } else {
        "parlo"
    }
}

#[derive(Clone)]
pub struct Progress {
    pub running: bool,
    pub phase: String,
    pub percent: u8,
    pub version: String,
}

fn slot() -> &'static Mutex<Progress> {
    static SLOT: OnceLock<Mutex<Progress>> = OnceLock::new();
    SLOT.get_or_init(|| {
        Mutex::new(Progress {
            running: false,
            phase: "idle".to_string(),
            percent: 0,
            version: String::new(),
        })
    })
}

fn report(running: bool, phase: &str, percent: u8, version: &str) {
    if let Ok(mut state) = slot().lock() {
        state.running = running;
        state.phase = phase.to_string();
        state.percent = percent;
        state.version = version.to_string();
    }
}

pub fn progress_json() -> String {
    let state = match slot().lock() {
        Ok(state) => state.clone(),
        Err(_) => Progress {
            running: false,
            phase: "idle".to_string(),
            percent: 0,
            version: String::new(),
        },
    };
    serde_json::json!({
        "running": state.running,
        "phase": state.phase,
        "percent": state.percent,
        "version": state.version,
    })
    .to_string()
}

pub fn percent_from_curl(line: &str) -> Option<u8> {
    let index = line.find('%')?;
    let digits: String = line[..index]
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit() || *character == '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    digits
        .parse::<f32>()
        .ok()
        .map(|value| value.clamp(0.0, 100.0) as u8)
}

fn download(url: &str, archive: &Path, version: &str) -> Result<(), String> {
    let mut child = Command::new("curl")
        .args([
            "-L",
            "--progress-bar",
            "-o",
            &archive.display().to_string(),
            url,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start the download: {error}"))?;
    if let Some(stderr) = child.stderr.take() {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if let Some(percent) = percent_from_curl(&line) {
                report(true, "downloading", percent.saturating_mul(3) / 4, version);
            }
        }
    }
    let status = child.wait().map_err(|error| error.to_string())?;
    if !status.success() {
        return Err("the download did not finish".to_string());
    }
    Ok(())
}

pub fn is_newer(current: &str, candidate: &str) -> bool {
    compare(current, candidate) == Ordering::Less
}

struct Version<'a> {
    numbers: Vec<u32>,
    prerelease: Option<&'a str>,
}

impl<'a> Version<'a> {
    fn parse(text: &'a str) -> Self {
        let text = text.trim().trim_start_matches('v');
        let (numbers, prerelease) = match text.split_once('-') {
            Some((numbers, prerelease)) => (numbers, Some(prerelease)),
            None => (text, None),
        };
        let mut numbers: Vec<u32> = numbers
            .split(['.', '+'])
            .filter_map(|part| part.parse::<u32>().ok())
            .collect();
        while numbers.len() < 3 {
            numbers.push(0);
        }
        Self {
            numbers,
            prerelease,
        }
    }
}

fn compare(left: &str, right: &str) -> Ordering {
    let left = Version::parse(left);
    let right = Version::parse(right);
    for index in 0..left.numbers.len().max(right.numbers.len()) {
        let a = *left.numbers.get(index).unwrap_or(&0);
        let b = *right.numbers.get(index).unwrap_or(&0);
        if a != b {
            return a.cmp(&b);
        }
    }
    match (left.prerelease, right.prerelease) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a), Some(b)) => compare_prerelease(a, b),
    }
}

fn compare_prerelease(left: &str, right: &str) -> Ordering {
    let mut left = left.split('.');
    let mut right = right.split('.');
    loop {
        match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(a), Some(b)) => {
                let order = match (a.parse::<u32>(), b.parse::<u32>()) {
                    (Ok(a), Ok(b)) => a.cmp(&b),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => a.cmp(b),
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
        }
    }
}

pub fn check_channel(value: &str) -> Result<String, String> {
    let wanted = value.trim().to_ascii_lowercase();
    if CHANNELS.contains(&wanted.as_str()) {
        Ok(wanted)
    } else {
        Err(format!(
            "unknown update channel {value}, expected one of {}",
            CHANNELS.join(", ")
        ))
    }
}

pub fn feed_url(channel: &str) -> &'static str {
    if channel == "preview" {
        RELEASES
    } else {
        LATEST
    }
}

pub fn latest() -> Result<Latest, String> {
    latest_in(&crate::config::load().update_channel)
}

pub fn latest_in(channel: &str) -> Result<Latest, String> {
    let channel = check_channel(channel)?;
    let body = fetch(feed_url(&channel))?;
    let document: serde_json::Value = serde_json::from_str(&body)
        .map_err(|error| format!("cannot read the release feed: {error}"))?;
    select(&channel, &document)
}

fn select(channel: &str, document: &serde_json::Value) -> Result<Latest, String> {
    let items: Vec<&serde_json::Value> = match document {
        serde_json::Value::Array(items) => items.iter().collect(),
        single => vec![single],
    };
    let chosen = items
        .into_iter()
        .filter(|item| {
            !item
                .get("draft")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        })
        .filter(|item| {
            channel == "preview"
                || !item
                    .get("prerelease")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
        })
        .filter(|item| {
            item.get("tag_name")
                .and_then(serde_json::Value::as_str)
                .is_some()
        })
        .max_by(|left, right| compare(tag(left), tag(right)))
        .ok_or_else(|| {
            if channel == "preview" {
                "the release feed has no release".to_string()
            } else {
                "the release feed has no stable release".to_string()
            }
        })?;
    release(chosen)
}

fn tag(item: &serde_json::Value) -> &str {
    item.get("tag_name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("0")
}

fn release(document: &serde_json::Value) -> Result<Latest, String> {
    let version = document
        .get("tag_name")
        .and_then(serde_json::Value::as_str)
        .ok_or("the release feed has no tag")?
        .trim_start_matches('v')
        .to_string();
    let notes_url = document
        .get("html_url")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let archive_url = document
        .get("assets")
        .and_then(serde_json::Value::as_array)
        .and_then(|assets| {
            assets.iter().find_map(|asset| {
                let name = asset.get("name")?.as_str()?;
                if name == archive_name() {
                    asset
                        .get("browser_download_url")?
                        .as_str()
                        .map(str::to_string)
                } else {
                    None
                }
            })
        });
    Ok(Latest {
        version,
        notes_url,
        archive_url,
    })
}

pub fn install(latest: &Latest) -> Result<PathBuf, String> {
    let url = latest
        .archive_url
        .as_deref()
        .ok_or_else(|| format!("release {} has no build for this platform", latest.version))?;
    report(true, "downloading", 0, &latest.version);
    let directory = std::env::temp_dir().join(format!("parlo-{}", latest.version));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let archive = directory.join(archive_name());

    if cfg!(target_os = "windows") {
        download(url, &archive, &latest.version)?;
        report(true, "unpacking", 78, &latest.version);
        run(
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    archive.display(),
                    directory.display()
                ),
            ],
        )?;
    } else {
        download(url, &archive, &latest.version)?;
        report(true, "unpacking", 78, &latest.version);
        run(
            "tar",
            &[
                "-xzf",
                &archive.display().to_string(),
                "-C",
                &directory.display().to_string(),
            ],
        )?;
    }

    report(true, "checking", 92, &latest.version);
    let binary = directory.join(binary_name());
    if !binary.is_file() {
        return Err(format!("the archive did not contain {}", binary_name()));
    }
    report(true, "ready", 100, &latest.version);
    Ok(binary)
}

pub fn replacement_script(pid: u32, new_binary: &Path, current: &Path) -> String {
    format!(
        "@echo off\r\n:parlo_wait\r\ntasklist /FI \"PID eq {pid}\" 2>nul | find \"{pid}\" >nul\r\nif not errorlevel 1 (\r\n  timeout /t 1 /nobreak >nul\r\n  goto parlo_wait\r\n)\r\nmove /y \"{}\" \"{}\" >nul\r\nstart \"\" \"{}\" app\r\n",
        new_binary.display(),
        current.display(),
        current.display()
    )
}

pub fn schedule_replacement(new_binary: &Path) -> Result<PathBuf, String> {
    let current = std::env::current_exe().map_err(|error| error.to_string())?;
    let script = std::env::temp_dir().join(format!("parlo-update-{}.cmd", std::process::id()));
    let body = replacement_script(std::process::id(), new_binary, &current);
    fs::write(&script, body).map_err(|error| error.to_string())?;
    Command::new("cmd")
        .args(["/c", "start", "", &script.display().to_string()])
        .spawn()
        .map_err(|error| format!("cannot start the replacement: {error}"))?;
    Ok(script)
}

pub fn restart() -> Result<(), String> {
    let current = std::env::current_exe().map_err(|error| error.to_string())?;
    Command::new(&current)
        .arg("app")
        .spawn()
        .map_err(|error| format!("Parlo could not start again: {error}"))?;
    Ok(())
}

fn fetch(url: &str) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Invoke-WebRequest -UseBasicParsing -Uri '{url}').Content"),
            ])
            .output()
    } else {
        Command::new("curl").args(["-fsSL", url]).output()
    }
    .map_err(|error| format!("cannot reach the release feed: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "cannot read the release feed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("cannot run {program}: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fourth_number_is_read_and_compared() {
        assert!(is_newer("0.2.1", "0.2.1.1"));
        assert!(!is_newer("0.2.1+1", "0.2.1.1"));
        assert!(!is_newer("0.2.1+1", "0.2.1"));
        assert!(is_newer("0.2.1.1", "0.2.2"));
    }

    #[test]
    fn a_curl_progress_line_gives_a_percentage() {
        assert_eq!(percent_from_curl("####                    45.3%"), Some(45));
        assert_eq!(percent_from_curl("100.0%"), Some(100));
        assert_eq!(percent_from_curl("no meter here"), None);
    }

    #[test]
    fn the_replacement_waits_for_the_process_then_moves_and_starts_it() {
        let script = replacement_script(
            4242,
            Path::new("C:/temp/new/parlo.exe"),
            Path::new("C:/apps/parlo.exe"),
        );
        assert!(script.contains("PID eq 4242"));
        assert!(script.contains("move /y \"C:/temp/new/parlo.exe\" \"C:/apps/parlo.exe\""));
        assert!(script.contains("start \"\" \"C:/apps/parlo.exe\" app"));
    }

    #[test]
    fn newer_versions_are_detected() {
        assert!(is_newer("0.1.1", "0.1.2"));
        assert!(is_newer("0.1.11", "0.2.0"));
        assert!(is_newer("v0.1.1", "v0.2.0"));
        assert!(!is_newer("0.2.0", "0.1.9"));
        assert!(!is_newer("0.1.1", "0.1.1"));
    }

    #[test]
    fn short_versions_fill_missing_parts() {
        assert!(is_newer("0.2", "0.2.1"));
        assert!(!is_newer("0.2.1", "0.2"));
    }

    #[test]
    fn the_archive_matches_the_platform() {
        let name = archive_name();
        assert!(name.starts_with("parlo-"));
        if cfg!(target_os = "windows") {
            assert!(name.ends_with(".zip"));
        } else {
            assert!(name.ends_with(".tar.gz"));
        }
    }

    fn release(tag: &str, prerelease: bool) -> serde_json::Value {
        serde_json::json!({
            "tag_name": format!("v{tag}"),
            "html_url": format!("https://api.github.com/releases/{tag}"),
            "prerelease": prerelease,
            "draft": false,
            "assets": [{
                "name": archive_name(),
                "browser_download_url": format!("https://api.github.com/{tag}")
            }]
        })
    }

    #[test]
    fn a_preview_build_updates_to_the_newer_preview() {
        let feed = serde_json::json!([
            release("0.5.0-preview.1", true),
            release("0.5.0-preview.0", true),
            release("0.4.9", false)
        ]);
        let chosen = select("preview", &feed).unwrap();
        assert_eq!(chosen.version, "0.5.0-preview.1");
        assert!(is_newer("0.5.0-preview.0", &chosen.version));
        assert_eq!(
            chosen.archive_url.as_deref(),
            Some("https://api.github.com/0.5.0-preview.1")
        );
    }

    #[test]
    fn a_preview_build_updates_to_a_newer_stable() {
        let feed = serde_json::json!([
            release("0.4.0", false),
            release("0.3.9", false),
            release("0.3.9-preview.2", true)
        ]);
        let chosen = select("preview", &feed).unwrap();
        assert_eq!(chosen.version, "0.4.0");
        assert!(is_newer("0.4.0-preview.9", &chosen.version));
    }

    #[test]
    fn a_stable_build_never_jumps_to_a_preview() {
        let feed = serde_json::json!([release("0.9.0-preview.1", true), release("0.4.0", false)]);
        let chosen = select("stable", &feed).unwrap();
        assert_eq!(chosen.version, "0.4.0");
        assert!(!is_newer("0.4.0", &chosen.version));
    }

    #[test]
    fn a_draft_is_never_offered() {
        let mut draft = release("0.6.0", false);
        draft["draft"] = serde_json::Value::Bool(true);
        let feed = serde_json::json!([draft, release("0.4.0", false)]);
        assert_eq!(select("stable", &feed).unwrap().version, "0.4.0");
    }

    #[test]
    fn stable_and_preview_read_different_feeds() {
        assert_eq!(feed_url("stable"), LATEST);
        assert_eq!(feed_url("preview"), RELEASES);
        assert!(RELEASES.starts_with(LATEST.trim_end_matches("/latest")));
        assert!(RELEASES.contains("releases?"));
    }

    #[test]
    fn an_unknown_channel_is_refused() {
        assert!(check_channel("nightly").is_err());
        assert_eq!(check_channel(" Preview ").unwrap(), "preview");
        assert_eq!(check_channel("stable").unwrap(), "stable");
    }

    #[test]
    fn a_prerelease_sorts_below_its_stable_release() {
        assert!(is_newer("0.3.0-preview.1", "0.3.0"));
        assert!(!is_newer("0.3.0", "0.3.0-preview.1"));
        assert!(is_newer("0.3.0-preview.1", "0.3.0-preview.2"));
        assert!(is_newer("0.3.0-preview.2", "0.3.0-preview.10"));
        assert!(!is_newer("0.3.0-preview.2", "0.3.0-preview.2"));
    }

    #[test]
    fn a_missing_build_is_reported() {
        let latest = Latest {
            version: "0.9.9".to_string(),
            notes_url: String::new(),
            archive_url: None,
        };
        assert!(install(&latest).is_err());
    }
}
