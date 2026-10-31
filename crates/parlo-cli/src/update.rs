use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LATEST: &str = "https://api.github.com/repos/PedroNascimento09/Parlo/releases/latest";

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

pub fn is_newer(current: &str, candidate: &str) -> bool {
    parse(current) < parse(candidate)
}

fn parse(version: &str) -> (u32, u32, u32) {
    let mut parts = version
        .trim()
        .trim_start_matches('v')
        .split(['.', '-', '+'])
        .filter_map(|part| part.parse::<u32>().ok())
        .chain(std::iter::repeat(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

pub fn latest() -> Result<Latest, String> {
    let body = fetch(LATEST)?;
    let document: serde_json::Value = serde_json::from_str(&body)
        .map_err(|error| format!("cannot read the release feed: {error}"))?;
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
    let directory = std::env::temp_dir().join(format!("parlo-{}", latest.version));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let archive = directory.join(archive_name());

    if cfg!(target_os = "windows") {
        run(
            "powershell",
            &[
                "-NoProfile",
                "-Command",
                &format!(
                    "Invoke-WebRequest -UseBasicParsing -Uri '{}' -OutFile '{}'",
                    url,
                    archive.display()
                ),
            ],
        )?;
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
        run(
            "curl",
            &["-fsSL", "-o", &archive.display().to_string(), url],
        )?;
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

    let binary = directory.join(binary_name());
    if !binary.is_file() {
        return Err(format!("the archive did not contain {}", binary_name()));
    }
    Ok(binary)
}

pub fn replace_running(new_binary: &Path) -> Result<(), String> {
    let current = std::env::current_exe().map_err(|error| error.to_string())?;
    let backup = current.with_extension("previous");
    let _ = fs::copy(&current, &backup);

    if cfg!(target_os = "windows") {
        let command = format!(
            "ping 127.0.0.1 -n 3 >nul & move /y \"{}\" \"{}\"",
            new_binary.display(),
            current.display()
        );
        Command::new("cmd")
            .args(["/c", &command])
            .spawn()
            .map_err(|error| error.to_string())?;
        return Ok(());
    }

    let command = format!(
        "sleep 2; mv -f '{}' '{}'",
        new_binary.display(),
        current.display()
    );
    Command::new("sh")
        .args(["-c", &command])
        .spawn()
        .map_err(|error| error.to_string())?;
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
