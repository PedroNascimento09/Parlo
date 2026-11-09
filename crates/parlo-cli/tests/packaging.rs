use std::fs;
use std::path::{Path, PathBuf};

const CHANNELS: [(&str, &str, &str); 7] = [
    (
        "winget",
        "packaging/winget/PedroNascimento09.Parlo.installer.yaml",
        "PackageVersion: {version}",
    ),
    (
        "homebrew",
        "packaging/homebrew/parlo.rb",
        "version \"{version}\"",
    ),
    ("aur", "packaging/aur/PKGBUILD", "pkgver={version}"),
    (
        "snap",
        "packaging/snap/snapcraft.yaml",
        "version: '{version}'",
    ),
    ("deb", "packaging/deb/debian/rules", "VERSION := {version}"),
    (
        "appimage",
        "packaging/appimage/parlo.appimage.yml",
        "version: {version}",
    ),
    (
        "nix",
        "packaging/nix/default.nix",
        "version = \"{version}\"",
    ),
];

const DOCKERFILE: &str = "packaging/docker/Dockerfile";
const ENTRYPOINT: &str = "packaging/docker/entrypoint.sh";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root is reachable from the crate")
}

fn read(relative: &str) -> String {
    let path = repo().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn workspace_version() -> String {
    for line in read("Cargo.toml").lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("version = ") {
            return value.trim_matches('"').to_string();
        }
    }
    panic!("Cargo.toml: no version line");
}

fn served_port() -> String {
    for line in read("crates/parlo-core/src/settings.rs").lines() {
        if let Some(value) = line.trim().strip_prefix("port: ") {
            return value.trim_end_matches(',').to_string();
        }
    }
    panic!("crates/parlo-core/src/settings.rs: no default port");
}

fn release_artefacts() -> Vec<String> {
    let workflow = read(".github/workflows/release.yml");
    let mut names = Vec::new();
    for line in workflow.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("archive: ") {
            names.push(value.trim().to_string());
        }
    }
    for fixed in ["Parlo-Setup.exe", "sbom.cdx.json", "SHA256SUMS.txt"] {
        if workflow.contains(fixed) {
            names.push(fixed.to_string());
        }
    }
    names
}

fn named_artefacts(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find("releases/download/") {
        rest = &rest[index + "releases/download/".len()..];
        let slash = match rest.find('/') {
            Some(slash) => slash,
            None => continue,
        };
        let tail = &rest[slash + 1..];
        let end = tail
            .find(|character: char| {
                character == '"' || character == '\'' || character.is_whitespace()
            })
            .unwrap_or(tail.len());
        names.push(tail[..end].to_string());
    }
    names
}

fn sha256_hexes(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        if character.is_ascii_hexdigit() {
            current.push(character);
        } else {
            if current.len() == 64 {
                found.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
        }
    }
    if current.len() == 64 {
        found.push(current);
    }
    found
}

#[test]
fn every_channel_ships_a_manifest_at_the_crate_version() {
    let version = workspace_version();
    for (channel, path, marker) in CHANNELS {
        let expected = marker.replace("{version}", &version);
        let text = read(path);
        assert!(
            text.contains(&expected),
            "{channel} ({path}): missing {expected:?}, while Cargo.toml is at {version}"
        );
    }
}

#[test]
fn every_manifest_names_an_artefact_the_release_produces() {
    let produced = release_artefacts();
    assert_eq!(
        produced.len(),
        6,
        "release.yml should publish six files, found {produced:?}"
    );
    for (channel, path, _) in CHANNELS {
        let text = read(path);
        let named = named_artefacts(&text);
        assert!(
            !named.is_empty(),
            "{channel} ({path}): names no release download"
        );
        for artefact in named {
            assert!(
                produced.contains(&artefact),
                "{channel} ({path}): names {artefact}, which release.yml does not produce (it produces {produced:?})"
            );
        }
    }
}

#[test]
fn every_manifest_pins_a_checksum() {
    for (channel, path, _) in CHANNELS {
        let text = read(path);
        assert!(
            !sha256_hexes(&text).is_empty() || text.contains("sha256-"),
            "{channel} ({path}): no sha256 checksum is pinned"
        );
    }
}

#[test]
fn the_container_builds_from_the_release_archive_at_the_crate_version() {
    let version = workspace_version();
    let file = read(DOCKERFILE);
    assert!(
        file.contains(&format!("ARG PARLO_VERSION={version}")),
        "{DOCKERFILE}: no ARG PARLO_VERSION={version}, while Cargo.toml is at {version}"
    );
    let named = named_artefacts(&file);
    assert!(!named.is_empty(), "{DOCKERFILE}: names no release download");
    let produced = release_artefacts();
    for artefact in named {
        assert!(
            produced.contains(&artefact),
            "{DOCKERFILE}: downloads {artefact}, which release.yml does not produce (it produces {produced:?})"
        );
    }
}

#[test]
fn the_container_checks_the_published_hash_before_it_unpacks() {
    let file = read(DOCKERFILE);
    assert!(
        file.contains("${PARLO_SHA256}  parlo-linux-x86_64.tar.gz\" | sha256sum -c -"),
        "{DOCKERFILE}: the archive is unpacked without a sha256sum -c - check"
    );
    let line = file
        .lines()
        .find(|line| line.trim_start().starts_with("ARG PARLO_SHA256="))
        .unwrap_or_else(|| panic!("{DOCKERFILE}: no ARG PARLO_SHA256 line"));
    let value = line.split('=').nth(1).unwrap_or_default().trim();
    assert_eq!(
        value.len(),
        64,
        "{DOCKERFILE}: PARLO_SHA256 is {value:?}, not a full sha256"
    );
    assert!(
        sha256_hexes(line).len() == 1,
        "{DOCKERFILE}: PARLO_SHA256 is not hexadecimal"
    );
}

#[test]
fn the_container_never_runs_as_root() {
    let file = read(DOCKERFILE);
    let user = file
        .lines()
        .map(str::trim)
        .rfind(|line| line.starts_with("USER "))
        .unwrap_or_else(|| panic!("{DOCKERFILE}: no USER line, so the server would run as root"));
    let value = user.trim_start_matches("USER ").trim();
    assert!(
        !value.eq_ignore_ascii_case("root") && !value.starts_with('0'),
        "{DOCKERFILE}: USER {value} is root"
    );
}

#[test]
fn the_container_serves_the_port_the_binary_uses() {
    let port = served_port();
    let file = read(DOCKERFILE);
    assert!(
        file.contains(&format!("EXPOSE {port}")),
        "{DOCKERFILE}: EXPOSE does not name {port}, the port parlo serves"
    );
    assert!(
        file.contains(&format!("PARLO_PORT={port}")),
        "{DOCKERFILE}: PARLO_PORT default is not {port}"
    );
    let entrypoint = read(ENTRYPOINT);
    assert!(
        entrypoint.contains(&format!("PARLO_PORT:-{port}")),
        "{ENTRYPOINT}: the default port is not {port}"
    );
    assert!(
        entrypoint.contains("exec /opt/parlo/parlo serve --port \"$port\""),
        "{ENTRYPOINT}: does not exec parlo serve on that port"
    );
    let document = read("packaging/README.md");
    assert!(
        document.contains(DOCKERFILE) && document.contains(ENTRYPOINT),
        "packaging/README.md: the docker channel is not documented"
    );
}

#[test]
fn every_channel_is_documented_with_its_state() {
    let document = read("packaging/README.md");
    assert!(
        document.contains("Partial"),
        "packaging/README.md: no channel carries its honest state"
    );
    for (channel, path, _) in CHANNELS {
        assert!(
            document.contains(path),
            "packaging/README.md: {channel} does not name {path}"
        );
    }
}

#[test]
fn the_installer_takes_its_version_from_the_project() {
    let iss = read("installer/parlo.iss");
    assert!(
        !iss.contains("#define AppVersion "),
        "installer/parlo.iss: AppVersion must arrive from the build, not from a literal"
    );
    assert!(
        iss.contains("#error AppVersion is missing"),
        "installer/parlo.iss: a build without a version must fail"
    );
    assert!(iss.contains("AppVersion={#AppVersion}"));
    assert!(iss.contains("VersionInfoProductVersion={#AppVersion}"));
    let workflow = read(".github/workflows/release.yml");
    assert!(
        workflow.contains("Select-String -Path Cargo.toml"),
        "release.yml: the installer version must be read from Cargo.toml"
    );
    assert!(
        workflow.contains("does not match the Cargo.toml version"),
        "release.yml: a tag that disagrees with Cargo.toml must fail the build"
    );
}
