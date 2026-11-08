use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::crypto;

pub const NOTICE_VERSION: &str = "1";
pub const SAMPLE: &str = "sample.wav";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    OwnVoice,
    ThirdParty,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::OwnVoice => "own-voice",
            Origin::ThirdParty => "third-party",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consent {
    pub origin: Origin,
    pub purpose: String,
    pub scope: String,
    pub notice_version: String,
    pub declared_authorisation: bool,
    pub accepted_at: u64,
    #[serde(default)]
    pub revoked_at: Option<u64>,
}

impl Consent {
    pub fn own_voice(
        purpose: impl Into<String>,
        scope: impl Into<String>,
        accepted_at: u64,
    ) -> Self {
        Consent {
            origin: Origin::OwnVoice,
            purpose: purpose.into(),
            scope: scope.into(),
            notice_version: NOTICE_VERSION.to_string(),
            declared_authorisation: false,
            accepted_at,
            revoked_at: None,
        }
    }

    pub fn third_party(
        purpose: impl Into<String>,
        scope: impl Into<String>,
        accepted_at: u64,
    ) -> Self {
        Consent {
            origin: Origin::ThirdParty,
            purpose: purpose.into(),
            scope: scope.into(),
            notice_version: NOTICE_VERSION.to_string(),
            declared_authorisation: true,
            accepted_at,
            revoked_at: None,
        }
    }

    fn validate(&self) -> Result<(), VoiceError> {
        if self.purpose.trim().is_empty() {
            return Err(VoiceError::EmptyField("purpose"));
        }
        if self.scope.trim().is_empty() {
            return Err(VoiceError::EmptyField("scope"));
        }
        if self.notice_version.trim().is_empty() {
            return Err(VoiceError::EmptyField("notice_version"));
        }
        if self.origin == Origin::ThirdParty && !self.declared_authorisation {
            return Err(VoiceError::ThirdPartyWithoutAuthorisation);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub label: String,
    pub language: String,
    pub created_at: u64,
    pub consent: Consent,
    #[serde(default)]
    pub encrypted: bool,
    #[serde(default)]
    pub artefacts: Vec<String>,
}

impl Profile {
    pub fn usable(&self) -> bool {
        self.consent.revoked_at.is_none()
    }
}

#[derive(Debug)]
pub enum VoiceError {
    EmptyField(&'static str),
    ThirdPartyWithoutAuthorisation,
    Duplicate(String),
    Crypto(String),
    NotFound(String),
    Revoked(String),
    Corrupt(PathBuf),
    Io(io::Error),
}

impl fmt::Display for VoiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VoiceError::EmptyField(field) => write!(formatter, "{field} cannot be empty"),
            VoiceError::ThirdPartyWithoutAuthorisation => write!(
                formatter,
                "a third party voice profile requires a declared authorisation"
            ),
            VoiceError::Duplicate(id) => write!(formatter, "voice profile {id} already exists"),
            VoiceError::Crypto(reason) => write!(formatter, "{reason}"),
            VoiceError::NotFound(id) => write!(formatter, "voice profile {id} does not exist"),
            VoiceError::Revoked(id) => write!(
                formatter,
                "voice profile {id} was revoked, so it cannot take a new sample"
            ),
            VoiceError::Corrupt(path) => {
                write!(
                    formatter,
                    "{} is not a readable voice profile",
                    path.display()
                )
            }
            VoiceError::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for VoiceError {}

impl From<io::Error> for VoiceError {
    fn from(error: io::Error) -> Self {
        VoiceError::Io(error)
    }
}

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Store { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn create(
        &self,
        label: &str,
        language: &str,
        consent: Consent,
    ) -> Result<Profile, VoiceError> {
        if label.trim().is_empty() {
            return Err(VoiceError::EmptyField("label"));
        }
        if language.trim().is_empty() {
            return Err(VoiceError::EmptyField("language"));
        }
        consent.validate()?;
        let id = derive_id(label);
        let folder = self.root.join(&id);
        if folder.exists() {
            return Err(VoiceError::Duplicate(id));
        }
        let profile = Profile {
            id: id.clone(),
            label: label.trim().to_string(),
            language: language.trim().to_string(),
            created_at: consent.accepted_at,
            consent,
            encrypted: false,
            artefacts: Vec::new(),
        };
        fs::create_dir_all(&folder)?;
        if let Err(error) = write(&folder, &profile) {
            let _ = fs::remove_dir_all(&folder);
            return Err(error);
        }
        Ok(profile)
    }

    pub fn list(&self) -> Result<Vec<Profile>, VoiceError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut profiles = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if !path.is_dir() {
                continue;
            }
            profiles.push(read(&path)?);
        }
        profiles.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(profiles)
    }

    pub fn get(&self, id: &str) -> Result<Profile, VoiceError> {
        let folder = self.folder(id)?;
        if !folder.is_dir() {
            return Err(VoiceError::NotFound(id.to_string()));
        }
        read(&folder)
    }

    pub fn record_artefact(&self, id: &str, name: &str) -> Result<Profile, VoiceError> {
        if name.trim().is_empty() || name.contains('/') || name.contains('\\') {
            return Err(VoiceError::EmptyField("artefact"));
        }
        let folder = self.folder(id)?;
        if !folder.join(name).exists() {
            return Err(VoiceError::NotFound(name.to_string()));
        }
        let mut profile = read(&folder)?;
        if !profile.artefacts.iter().any(|item| item == name) {
            profile.artefacts.push(name.to_string());
        }
        write(&folder, &profile)?;
        Ok(profile)
    }

    pub fn save_sample(&self, id: &str, bytes: &[u8]) -> Result<Profile, VoiceError> {
        let folder = self.folder(id)?;
        if !folder.is_dir() {
            return Err(VoiceError::NotFound(id.to_string()));
        }
        let profile = read(&folder)?;
        if !profile.usable() {
            return Err(VoiceError::Revoked(id.to_string()));
        }
        let stored = if profile.encrypted {
            crypto::seal(bytes).map_err(|error| VoiceError::Crypto(error.to_string()))?
        } else {
            bytes.to_vec()
        };
        fs::write(folder.join(SAMPLE), stored)?;
        self.record_artefact(id, SAMPLE)
    }

    pub fn sample(&self, id: &str) -> Result<Vec<u8>, VoiceError> {
        let folder = self.folder(id)?;
        let path = folder.join(SAMPLE);
        if !path.is_file() {
            return Err(VoiceError::NotFound(id.to_string()));
        }
        let stored = fs::read(path)?;
        if read(&folder)?.encrypted {
            return crypto::open(&stored).map_err(|error| VoiceError::Crypto(error.to_string()));
        }
        Ok(stored)
    }

    pub fn set_encryption(&self, id: &str, on: bool) -> Result<Profile, VoiceError> {
        let folder = self.folder(id)?;
        if !folder.is_dir() {
            return Err(VoiceError::NotFound(id.to_string()));
        }
        let mut profile = read(&folder)?;
        if profile.encrypted != on {
            let path = folder.join(SAMPLE);
            if path.is_file() {
                let stored = fs::read(&path)?;
                let converted = if on {
                    crypto::seal(&stored).map_err(|error| VoiceError::Crypto(error.to_string()))?
                } else {
                    crypto::open(&stored).map_err(|error| VoiceError::Crypto(error.to_string()))?
                };
                fs::write(&path, converted)?;
            }
            profile.encrypted = on;
            write(&folder, &profile)?;
        }
        Ok(profile)
    }

    pub fn revoke(&self, id: &str, at: u64) -> Result<Profile, VoiceError> {
        let folder = self.folder(id)?;
        if !folder.is_dir() {
            return Err(VoiceError::NotFound(id.to_string()));
        }
        let mut profile = read(&folder)?;
        profile.consent.revoked_at = Some(at);
        let sample = folder.join(SAMPLE);
        if sample.is_file() {
            fs::remove_file(&sample)?;
        }
        profile.artefacts.retain(|artefact| artefact != SAMPLE);
        write(&folder, &profile)?;
        Ok(profile)
    }

    pub fn delete(&self, id: &str) -> Result<(), VoiceError> {
        let folder = self.folder(id)?;
        if !folder.is_dir() {
            return Err(VoiceError::NotFound(id.to_string()));
        }
        fs::remove_dir_all(folder)?;
        Ok(())
    }

    fn folder(&self, id: &str) -> Result<PathBuf, VoiceError> {
        if id.is_empty()
            || !id.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
        {
            return Err(VoiceError::EmptyField("id"));
        }
        Ok(self.root.join(id))
    }
}

fn derive_id(label: &str) -> String {
    let mut slug = String::new();
    for character in label.trim().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
        if slug.len() >= 24 {
            break;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    let mut hash: u32 = 0x811c_9dc5;
    for byte in label.trim().as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    let stem = if slug.is_empty() { "voice" } else { &slug };
    format!("{stem}-{hash:08x}")
}

fn read(folder: &Path) -> Result<Profile, VoiceError> {
    let path = folder.join("profile.json");
    let text = fs::read_to_string(&path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            VoiceError::Corrupt(folder.to_path_buf())
        } else {
            VoiceError::Io(error)
        }
    })?;
    serde_json::from_str(&text).map_err(|_| VoiceError::Corrupt(path))
}

fn write(folder: &Path, profile: &Profile) -> Result<(), VoiceError> {
    let text = serde_json::to_string_pretty(profile)
        .map_err(|_| VoiceError::Corrupt(folder.to_path_buf()))?;
    let temporary = folder.join("profile.json.tmp");
    fs::write(&temporary, text)?;
    fs::rename(temporary, folder.join("profile.json"))?;
    Ok(())
}

#[cfg(test)]
mod tests;
