use super::*;

struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("parlo-voice-{name}-{}", std::process::id()));
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
fn a_sample_is_stored_recorded_and_read_back() {
    let sandbox = Sandbox::new("sample");
    let store = sandbox.store();
    let profile = store
        .create(
            "Pedro",
            "pt-PT",
            Consent::own_voice("translation", "local", 1),
        )
        .unwrap();
    let saved = store.save_sample(&profile.id, b"RIFFwav").unwrap();
    assert_eq!(saved.artefacts, vec![SAMPLE.to_string()]);
    assert_eq!(store.sample(&profile.id).unwrap(), b"RIFFwav");
}

#[test]
fn a_sample_for_an_unknown_profile_is_refused() {
    let sandbox = Sandbox::new("nosample");
    let store = sandbox.store();
    assert!(matches!(
        store.save_sample("missing-00000000", b"RIFFwav"),
        Err(VoiceError::NotFound(_))
    ));
}

#[test]
fn deleting_removes_the_sample_too() {
    let sandbox = Sandbox::new("deletesample");
    let store = sandbox.store();
    let profile = store
        .create(
            "Pedro",
            "pt-PT",
            Consent::own_voice("translation", "local", 1),
        )
        .unwrap();
    store.save_sample(&profile.id, b"RIFFwav").unwrap();
    store.delete(&profile.id).unwrap();
    assert!(matches!(
        store.sample(&profile.id),
        Err(VoiceError::NotFound(_))
    ));
}

#[cfg(windows)]
#[test]
fn a_sealed_profile_keeps_its_sample_private_and_gives_it_back() {
    let sandbox = Sandbox::new("sealed");
    let store = sandbox.store();
    let profile = store
        .create(
            "Pedro",
            "pt-PT",
            Consent::own_voice("translation", "local", 1),
        )
        .unwrap();
    let sealed = store.set_encryption(&profile.id, true).unwrap();
    assert!(sealed.encrypted);
    store.save_sample(&profile.id, b"RIFFwav").unwrap();
    let on_disk = fs::read(store.root().join(&profile.id).join(SAMPLE)).unwrap();
    assert!(crypto::sealed(&on_disk));
    assert_eq!(store.sample(&profile.id).unwrap(), b"RIFFwav");
    let plain = store.set_encryption(&profile.id, false).unwrap();
    assert!(!plain.encrypted);
    assert_eq!(store.sample(&profile.id).unwrap(), b"RIFFwav");
}

#[cfg(windows)]
#[test]
fn a_sealed_profile_is_refused_when_the_sample_is_not_sealed() {
    let sandbox = Sandbox::new("unsealed");
    let store = sandbox.store();
    let profile = store
        .create(
            "Pedro",
            "pt-PT",
            Consent::own_voice("translation", "local", 1),
        )
        .unwrap();
    store.save_sample(&profile.id, b"RIFFwav").unwrap();
    store.set_encryption(&profile.id, true).unwrap();
    let path = store.root().join(&profile.id).join(SAMPLE);
    fs::write(&path, b"RIFFwav").unwrap();
    assert!(matches!(
        store.sample(&profile.id),
        Err(VoiceError::Crypto(_))
    ));
}

#[cfg(windows)]
#[test]
fn a_profile_stays_inside_its_own_folder_and_leaves_nothing_behind() {
    let sandbox = Sandbox::new("contained");
    let store = sandbox.store();
    let profile = store
        .create(
            "Pedro",
            "pt-PT",
            Consent::own_voice("translation", "local", 1),
        )
        .unwrap();
    store.save_sample(&profile.id, b"RIFFwav").unwrap();
    store.set_encryption(&profile.id, true).unwrap();
    store.set_encryption(&profile.id, false).unwrap();
    let roots: Vec<String> = fs::read_dir(store.root())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(roots, vec![profile.id.clone()]);
    let mut inside: Vec<String> = fs::read_dir(store.root().join(&profile.id))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    inside.sort();
    assert_eq!(
        inside,
        vec!["profile.json".to_string(), "sample.wav".to_string()]
    );
    store.delete(&profile.id).unwrap();
    assert_eq!(fs::read_dir(store.root()).unwrap().count(), 0);
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
fn revoking_deletes_the_sample_and_keeps_the_profile() {
    let sandbox = Sandbox::new("revoke-sample");
    let store = sandbox.store();
    let profile = store
        .create(
            "Pedro",
            "pt-PT",
            Consent::own_voice("translation", "local", 1),
        )
        .unwrap();
    store.save_sample(&profile.id, b"RIFFwav").unwrap();
    let revoked = store.revoke(&profile.id, 99).unwrap();
    assert_eq!(revoked.consent.revoked_at, Some(99));
    assert!(!revoked.usable());
    assert!(!revoked.artefacts.iter().any(|item| item == SAMPLE));
    assert!(!store.root().join(&profile.id).join(SAMPLE).exists());
    assert!(matches!(
        store.sample(&profile.id),
        Err(VoiceError::NotFound(_))
    ));
    assert_eq!(store.list().unwrap().len(), 1);
}

#[test]
fn revoking_leaves_the_other_profiles_and_their_samples_alone() {
    let sandbox = Sandbox::new("revoke-others");
    let store = sandbox.store();
    let kept = store
        .create(
            "Ana",
            "pt-PT",
            Consent::own_voice("translation", "local", 1),
        )
        .unwrap();
    let removed = store
        .create(
            "Rui",
            "pt-PT",
            Consent::own_voice("translation", "local", 2),
        )
        .unwrap();
    store.save_sample(&kept.id, b"RIFFkept").unwrap();
    store.save_sample(&removed.id, b"RIFFremoved").unwrap();
    store.revoke(&removed.id, 3).unwrap();
    assert_eq!(store.sample(&kept.id).unwrap(), b"RIFFkept");
    assert!(store.get(&kept.id).unwrap().usable());
    assert_eq!(store.list().unwrap().len(), 2);
}

#[test]
fn a_revoked_profile_cannot_take_a_new_sample() {
    let sandbox = Sandbox::new("revoke-sample-again");
    let store = sandbox.store();
    let profile = store
        .create(
            "Pedro",
            "pt-PT",
            Consent::own_voice("translation", "local", 1),
        )
        .unwrap();
    store.revoke(&profile.id, 5).unwrap();
    assert!(matches!(
        store.save_sample(&profile.id, b"RIFFwav"),
        Err(VoiceError::Revoked(_))
    ));
    assert!(!store.root().join(&profile.id).join(SAMPLE).exists());
}

#[test]
fn revoking_twice_changes_nothing() {
    let sandbox = Sandbox::new("revoke-twice");
    let store = sandbox.store();
    let profile = store
        .create(
            "Pedro",
            "pt-PT",
            Consent::own_voice("translation", "local", 1),
        )
        .unwrap();
    store.save_sample(&profile.id, b"RIFFwav").unwrap();
    store.revoke(&profile.id, 7).unwrap();
    let again = store.revoke(&profile.id, 8).unwrap();
    assert_eq!(again.consent.revoked_at, Some(8));
    assert_eq!(store.list().unwrap().len(), 1);
    assert!(!store.root().join(&profile.id).join(SAMPLE).exists());
}

#[test]
fn a_profile_file_written_before_revocation_existed_still_reads_the_same() {
    let sandbox = Sandbox::new("old-profile");
    let store = sandbox.store();
    let folder = store.root().join("old-00000000");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join(SAMPLE), b"RIFFwav").unwrap();
    fs::write(
        folder.join("profile.json"),
        r#"{"id":"old-00000000","label":"Old","language":"pt-PT","created_at":1,"consent":{"origin":"own-voice","purpose":"translation","scope":"local","notice_version":"1","declared_authorisation":false,"accepted_at":1},"artefacts":["sample.wav"]}"#,
    )
    .unwrap();
    let profile = store.get("old-00000000").unwrap();
    assert!(profile.usable());
    assert_eq!(profile.consent.revoked_at, None);
    assert_eq!(store.sample("old-00000000").unwrap(), b"RIFFwav");
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
