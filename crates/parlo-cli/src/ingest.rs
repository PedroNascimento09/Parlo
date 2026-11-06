use std::env;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const DEFAULT_ENGINE_DIR: &str = "engines/cascade";
const MAX_URL: usize = 2_048;
const MAX_COOKIES: u64 = 1_048_576;

#[derive(Debug)]
pub struct Failure {
    pub status: &'static str,
    pub message: String,
}

pub fn download(url: &str, destination: &Path, cookies: Option<&Path>) -> Result<(), Failure> {
    let allow_local = allow_local();
    check_url(url, allow_local)?;
    let engine_dir = engine_dir()?;
    download_in(&engine_dir, url, destination, cookies, allow_local)
}

fn download_in(
    engine_dir: &Path,
    url: &str,
    destination: &Path,
    cookies: Option<&Path>,
    allow_local: bool,
) -> Result<(), Failure> {
    let cookies = match cookies {
        Some(path) => Some(check_cookies(path)?),
        None => None,
    };
    let script = engine_dir.join("scripts").join("ingest.py");
    if !script.is_file() {
        return Err(internal(format!(
            "cannot find {}; run Parlo from the source checkout",
            script.display()
        )));
    }
    let python = crate::sidecar::python_for(engine_dir);
    let mut command = Command::new(&python);
    command
        .arg(&script)
        .args(["--url", url])
        .arg("--out")
        .arg(destination)
        .current_dir(engine_dir)
        .env("PYTHONPATH", engine_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if allow_local {
        command.arg("--allow-local");
    }
    if let Some(path) = &cookies {
        command.arg("--cookies").arg(path);
    }
    let output = command.output().map_err(|error| {
        internal(format!(
            "cannot start {python}: {error}; install the cascade requirements first"
        ))
    })?;
    if output.status.success() {
        return Ok(());
    }
    let message = reported_error(&output.stderr);
    let status = match output.status.code() {
        Some(2) => "400 Bad Request",
        Some(3) => "502 Bad Gateway",
        _ => "500 Internal Server Error",
    };
    Err(Failure { status, message })
}

pub fn probe(path: &Path) -> Result<(), String> {
    let output = match Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "default=nw=1:nk=1",
        ])
        .arg(path)
        .output()
    {
        Ok(output) => output,
        Err(_) => return Ok(()),
    };
    let found = String::from_utf8_lossy(&output.stdout);
    let media = found
        .lines()
        .map(str::trim)
        .any(|kind| kind == "audio" || kind == "video");
    if output.status.success() && media {
        Ok(())
    } else {
        Err("the link does not point at an audio or video file".to_string())
    }
}
pub fn has_video(path: &Path) -> Result<bool, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Ok(false);
    }
    Ok(String::from_utf8_lossy(&output.stdout).contains("video"))
}

pub fn name_from_url(url: &str) -> String {
    let trimmed = url.split(['?', '#']).next().unwrap_or(url);
    let path = trimmed
        .split_once("://")
        .and_then(|(_, rest)| rest.find('/').map(|index| &rest[index..]))
        .unwrap_or("");
    if path.ends_with('/') {
        return "download.mp4".to_string();
    }
    let last = path.rsplit('/').next().unwrap_or("");
    if last.is_empty() {
        return "download.mp4".to_string();
    }
    crate::dub::safe_name(&crate::serve::http::percent_decode(last))
}

pub fn check_url(url: &str, allow_local: bool) -> Result<(), Failure> {
    let url = url.trim();
    if url.is_empty() {
        return Err(refused("paste a link to a video or audio file"));
    }
    if url.len() > MAX_URL {
        return Err(refused("the link is too long"));
    }
    if url
        .chars()
        .any(|c| c.is_ascii_whitespace() || c.is_control())
    {
        return Err(refused("the link contains a space or a control character"));
    }
    let lower = url.to_ascii_lowercase();
    let rest = if lower.starts_with("http://") {
        &url[7..]
    } else if lower.starts_with("https://") {
        &url[8..]
    } else {
        return Err(refused("only http and https links can be dubbed"));
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() {
        return Err(refused("the link has no address"));
    }
    if authority.contains('@') {
        return Err(refused("links with a username or password are refused"));
    }
    let host = if let Some(stripped) = authority.strip_prefix('[') {
        stripped.split(']').next().unwrap_or("")
    } else {
        authority.split(':').next().unwrap_or("")
    };
    if host.is_empty() {
        return Err(refused("the link has no address"));
    }
    if !host.is_ascii() {
        return Err(refused("the link must use an ASCII address"));
    }
    let host_lower = host.to_ascii_lowercase();
    if (host_lower == "localhost" || host_lower.ends_with(".localhost")) && !allow_local {
        return Err(refused(
            "the link points at this computer; paste a public address",
        ));
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if !is_public(ip) {
            if allow_local && is_local(ip) {
                return Ok(());
            }
            return Err(refused(
                "the link points at a private address; paste a public address",
            ));
        }
    }
    Ok(())
}

pub fn check_cookies(path: &Path) -> Result<PathBuf, Failure> {
    if path.as_os_str().is_empty() {
        return Err(refused("the cookies file path is empty"));
    }
    if !path.is_file() {
        return Err(refused(format!(
            "there is no cookies file at {}",
            path.display()
        )));
    }
    let size = fs::metadata(path)
        .map(|meta| meta.len())
        .unwrap_or_default();
    if size > MAX_COOKIES {
        return Err(refused("the cookies file is larger than 1 MB"));
    }
    let text = fs::read_to_string(path).map_err(|_| {
        refused(format!(
            "the cookies file at {} cannot be read",
            path.display()
        ))
    })?;
    let looks_like_cookies = text.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("# Netscape HTTP Cookie File")
            || line.starts_with("# HTTP Cookie File")
            || (!line.starts_with('#') && !line.trim().is_empty() && line.split('\t').count() >= 7)
    });
    if !looks_like_cookies {
        return Err(refused(format!(
            "{} is not a cookies file; export it from the browser in Netscape format",
            path.display()
        )));
    }
    Ok(path.to_path_buf())
}

fn allow_local() -> bool {
    cfg!(debug_assertions) && env::var("PARLO_INGEST_ALLOW_LOCAL").is_ok_and(|value| value == "1")
}

fn engine_dir() -> Result<PathBuf, Failure> {
    env::var("PARLO_CASCADE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_ENGINE_DIR))
        .canonicalize()
        .map_err(|error| internal(format!("{DEFAULT_ENGINE_DIR}: {error}")))
}

fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(address) => {
            !(address.is_private()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_broadcast()
                || address.is_documentation()
                || address.is_unspecified()
                || address.is_multicast())
        }
        IpAddr::V6(address) => {
            !(address.is_loopback()
                || address.is_unspecified()
                || address.is_multicast()
                || address.is_unique_local()
                || address.is_unicast_link_local())
        }
    }
}

fn is_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(address) => {
            address.is_private() || address.is_loopback() || address.is_link_local()
        }
        IpAddr::V6(address) => {
            address.is_loopback() || address.is_unique_local() || address.is_unicast_link_local()
        }
    }
}

fn reported_error(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let text = text.trim();
    let parsed = serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|value| value.get("error")?.as_str().map(ToOwned::to_owned));
    parsed.unwrap_or_else(|| {
        if text.is_empty() {
            "the link could not be downloaded".to_string()
        } else {
            text.to_string()
        }
    })
}

fn refused(message: impl Into<String>) -> Failure {
    Failure {
        status: "400 Bad Request",
        message: message.into(),
    }
}

fn internal(message: String) -> Failure {
    Failure {
        status: "500 Internal Server Error",
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused_url(url: &str) -> String {
        check_url(url, false).unwrap_err().message
    }

    #[test]
    fn accepts_public_http_and_https_links() {
        assert!(check_url("https://example.com/video.mp4", false).is_ok());
        assert!(check_url("HTTPS://example.com/video.mp4", false).is_ok());
        assert!(check_url("http://93.184.216.34/video.mp4", false).is_ok());
    }

    #[test]
    fn refuses_other_schemes_and_credentials() {
        assert!(refused_url("ftp://example.com/video.mp4").contains("http"));
        assert!(refused_url("file:///tmp/video.mp4").contains("http"));
        assert!(refused_url("https://user:secret@example.com/video.mp4").contains("username"));
    }

    #[test]
    fn refuses_localhost_and_private_addresses() {
        assert!(refused_url("http://localhost:8000/video.mp4").contains("this computer"));
        assert!(refused_url("http://media.localhost/video.mp4").contains("this computer"));
        assert!(refused_url("http://127.0.0.1/video.mp4").contains("private"));
        assert!(refused_url("http://10.1.2.3/video.mp4").contains("private"));
        assert!(refused_url("http://192.168.1.10/video.mp4").contains("private"));
        assert!(refused_url("http://169.254.1.1/video.mp4").contains("private"));
        assert!(refused_url("http://[::1]/video.mp4").contains("private"));
        assert!(refused_url("http://0.0.0.0/video.mp4").contains("private"));
    }

    #[test]
    fn the_development_override_allows_local_addresses() {
        assert!(check_url("http://127.0.0.1:8000/video.mp4", true).is_ok());
        assert!(check_url("http://localhost:8000/video.mp4", true).is_ok());
        assert!(check_url("http://[::1]/video.mp4", true).is_ok());
        assert!(check_url("http://192.168.1.10/video.mp4", true).is_ok());
    }

    #[test]
    fn refuses_spaces_empty_and_overlong_links() {
        assert!(refused_url("https://example.com/a b.mp4").contains("space"));
        assert!(refused_url("").contains("paste a link"));
        let long = format!("https://example.com/{}", "a".repeat(MAX_URL));
        assert!(refused_url(&long).contains("too long"));
    }

    #[test]
    fn names_come_from_the_path_and_fall_back() {
        assert_eq!(name_from_url("https://example.com/a/clip.mp4"), "clip.mp4");
        assert_eq!(
            name_from_url("https://example.com/a/clip.mp4?x=1"),
            "clip.mp4"
        );
        assert_eq!(name_from_url("https://example.com/a/"), "download.mp4");
        assert_eq!(name_from_url("https://example.com"), "download.mp4");
        assert_eq!(
            name_from_url("https://example.com/a/my%20clip.mp4"),
            "my clip.mp4"
        );
    }
}

#[cfg(test)]
mod cookies_tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("parlo-ingest-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn export(folder: &Path) -> PathBuf {
        let path = folder.join("cookies.txt");
        fs::write(
            &path,
            "# Netscape HTTP Cookie File\n.youtube.com\tTRUE\t/\tTRUE\t0\tSID\tabc\n",
        )
        .unwrap();
        path
    }

    fn downloader(folder: &Path, record: &Path) -> PathBuf {
        let engine = folder.join("engine");
        fs::create_dir_all(engine.join("scripts")).unwrap();
        fs::write(
            engine.join("scripts").join("ingest.py"),
            format!(
                "import json, sys\nargs = sys.argv[1:]\nopen({record:?}, 'w').write(json.dumps(args))\nopen(args[args.index('--out') + 1], 'wb').write(b'media')\n",
                record = record.to_string_lossy()
            ),
        )
        .unwrap();
        engine
    }

    #[test]
    fn refuses_a_cookies_path_that_is_not_there() {
        let folder = scratch("missing");
        let error = check_cookies(&folder.join("none.txt")).unwrap_err();
        assert_eq!(error.status, "400 Bad Request");
        assert!(
            error.message.contains("no cookies file"),
            "{}",
            error.message
        );
        assert!(check_cookies(&folder).is_err());
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn refuses_a_file_that_is_not_a_cookies_export() {
        let folder = scratch("foreign");
        let path = folder.join("notes.txt");
        fs::write(&path, "buy milk").unwrap();
        let error = check_cookies(&path).unwrap_err();
        assert!(
            error.message.contains("not a cookies file"),
            "{}",
            error.message
        );
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn accepts_a_netscape_cookies_export() {
        let folder = scratch("valid");
        let path = export(&folder);
        assert_eq!(check_cookies(&path).unwrap(), path);
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn the_cookies_file_reaches_the_downloader_and_a_public_link_needs_none() {
        let folder = scratch("argv");
        let record = folder.join("argv.json");
        let engine = downloader(&folder, &record);
        let cookies = export(&folder);
        let with = download_in(
            &engine,
            "https://example.com/video.mp4",
            &folder.join("one.mp4"),
            Some(&cookies),
            false,
        );
        let recorded: Vec<String> =
            serde_json::from_str(&fs::read_to_string(&record).unwrap()).unwrap();
        let without = download_in(
            &engine,
            "https://example.com/video.mp4",
            &folder.join("two.mp4"),
            None,
            false,
        );
        let plain: Vec<String> =
            serde_json::from_str(&fs::read_to_string(&record).unwrap()).unwrap();
        assert!(
            with.is_ok(),
            "{:?}",
            with.err().map(|failure| failure.message)
        );
        assert!(
            without.is_ok(),
            "{:?}",
            without.err().map(|failure| failure.message)
        );
        assert!(
            recorded.iter().any(|arg| arg == "--cookies"),
            "{recorded:?}"
        );
        assert!(
            recorded
                .iter()
                .any(|arg| *arg == cookies.display().to_string()),
            "{recorded:?}"
        );
        assert!(!plain.iter().any(|arg| arg == "--cookies"), "{plain:?}");
        assert!(folder.join("one.mp4").is_file());
        fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn a_missing_cookies_file_stops_before_the_downloader_runs() {
        let folder = scratch("no-run");
        let record = folder.join("argv.json");
        let engine = downloader(&folder, &record);
        let error = download_in(
            &engine,
            "https://example.com/video.mp4",
            &folder.join("x.mp4"),
            Some(&folder.join("none.txt")),
            false,
        )
        .unwrap_err();
        assert_eq!(error.status, "400 Bad Request");
        assert!(
            error.message.contains("no cookies file"),
            "{}",
            error.message
        );
        assert!(!record.exists());
        assert!(!folder.join("x.mp4").exists());
        fs::remove_dir_all(&folder).unwrap();
    }
}
