use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const NOTICE_VERSION: &str = "1";

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
    NotFound(String),
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
            VoiceError::NotFound(id) => write!(formatter, "voice profile {id} does not exist"),
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

    pub fn revoke(&self, id: &str, at: u64) -> Result<Profile, VoiceError> {
        let folder = self.folder(id)?;
        if !folder.is_dir() {
            return Err(VoiceError::NotFound(id.to_string()));
        }
        let mut profile = read(&folder)?;
        profile.consent.revoked_at = Some(at);
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
mod tests {
    use super::*;

    struct Sandbox {
        root: PathBuf,
    }

    impl Sandbox {
        fn new(name: &str) -> Self {
            let root =
                std::env::temp_dir().join(format!("parlo-voice-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            Sandbox { root }
        }

        fn store(&self) -> Store {
            Store::new(&self.root)
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn creating_a_profile_writes_its_consent() {
        let sandbox = Sandbox::new("create");
        let store = sandbox.store();
        let profile = store
            .create(
                "Pedro",
                "pt-PT",
                Consent::own_voice("translation", "local sessions", 1_700_000_000),
            )
            .unwrap();
        assert_eq!(profile.consent.notice_version, NOTICE_VERSION);
        assert!(profile.usable());
        let stored = store.get(&profile.id).unwrap();
        assert_eq!(stored, profile);
    }

    #[test]
    fn a_third_party_voice_needs_a_declared_authorisation() {
        let sandbox = Sandbox::new("third");
        let store = sandbox.store();
        let mut consent = Consent::third_party("translation", "local sessions", 1);
        consent.declared_authorisation = false;
        let error = store.create("Someone", "en", consent).unwrap_err();
        assert!(matches!(error, VoiceError::ThirdPartyWithoutAuthorisation));
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn an_empty_purpose_is_rejected() {
        let sandbox = Sandbox::new("purpose");
        let store = sandbox.store();
        let error = store
            .create("Pedro", "pt-PT", Consent::own_voice("  ", "local", 1))
            .unwrap_err();
        assert!(matches!(error, VoiceError::EmptyField("purpose")));
    }

    #[test]
    fn two_labels_that_differ_keep_different_ids() {
        let sandbox = Sandbox::new("ids");
        let store = sandbox.store();
        let first = store
            .create(
                "Pedro",
                "pt-PT",
                Consent::own_voice("translation", "local", 1),
            )
            .unwrap();
        let second = store
            .create(
                "Pedro N",
                "pt-PT",
                Consent::own_voice("translation", "local", 2),
            )
            .unwrap();
        assert_ne!(first.id, second.id);
        assert_eq!(store.list().unwrap().len(), 2);
    }

    #[test]
    fn the_same_label_twice_is_refused() {
        let sandbox = Sandbox::new("duplicate");
        let store = sandbox.store();
        store
            .create(
                "Pedro",
                "pt-PT",
                Consent::own_voice("translation", "local", 1),
            )
            .unwrap();
        let error = store
            .create(
                "Pedro",
                "pt-PT",
                Consent::own_voice("translation", "local", 1),
            )
            .unwrap_err();
        assert!(matches!(error, VoiceError::Duplicate(_)));
    }

    #[test]
    fn deleting_removes_the_folder_and_every_artefact() {
        let sandbox = Sandbox::new("delete");
        let store = sandbox.store();
        let profile = store
            .create(
                "Pedro",
                "pt-PT",
                Consent::own_voice("translation", "local", 1),
            )
            .unwrap();
        let folder = store.root().join(&profile.id);
        fs::write(folder.join("sample.wav"), b"audio").unwrap();
        let updated = store.record_artefact(&profile.id, "sample.wav").unwrap();
        assert_eq!(updated.artefacts, vec!["sample.wav".to_string()]);
        store.delete(&profile.id).unwrap();
        assert!(!folder.exists());
        assert!(matches!(
            store.get(&profile.id),
            Err(VoiceError::NotFound(_))
        ));
    }

    #[test]
    fn revoking_keeps_the_profile_and_marks_it_unusable() {
        let sandbox = Sandbox::new("revoke");
        let store = sandbox.store();
        let profile = store
            .create(
                "Pedro",
                "pt-PT",
                Consent::own_voice("translation", "local", 1),
            )
            .unwrap();
        let revoked = store.revoke(&profile.id, 99).unwrap();
        assert_eq!(revoked.consent.revoked_at, Some(99));
        assert!(!revoked.usable());
        assert_eq!(store.list().unwrap().len(), 1);
    }

    #[test]
    fn an_artefact_outside_the_folder_is_refused() {
        let sandbox = Sandbox::new("escape");
        let store = sandbox.store();
        let profile = store
            .create(
                "Pedro",
                "pt-PT",
                Consent::own_voice("translation", "local", 1),
            )
            .unwrap();
        assert!(store
            .record_artefact(&profile.id, "../profile.json")
            .is_err());
    }

    #[test]
    fn an_unreadable_folder_is_reported_not_skipped() {
        let sandbox = Sandbox::new("corrupt");
        let store = sandbox.store();
        fs::create_dir_all(store.root().join("broken-00000000")).unwrap();
        assert!(matches!(store.list(), Err(VoiceError::Corrupt(_))));
    }
}
