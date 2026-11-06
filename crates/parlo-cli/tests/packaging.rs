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
