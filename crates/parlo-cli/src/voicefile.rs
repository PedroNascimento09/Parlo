use parlo_core::voice::{Profile, Store};

pub const FORMAT: &str = "parlo-voice-profile";
pub const VERSION: u64 = 1;

pub struct Exchange {
    pub profile: Profile,
    pub sample: Vec<u8>,
}

pub fn export(profile: &Profile, sample: &[u8]) -> Vec<u8> {
    let header = serde_json::json!({
        "format": FORMAT,
        "version": VERSION,
        "synthetic": true,
        "notice": "Audio from this profile is synthetic voice. The person who authorised it is named in the consent record below.",
        "profile": profile,
    });
    let mut file = header.to_string().into_bytes();
    file.extend_from_slice(b"\n\n");
    file.extend_from_slice(sample);
    file
}

pub fn parse(bytes: &[u8]) -> Result<Exchange, String> {
    let split = bytes
        .windows(2)
        .position(|pair| pair == b"\n\n")
        .ok_or_else(|| "the file is not a Parlo voice file".to_string())?;
    let (header, rest) = bytes.split_at(split);
    let sample = &rest[2..];
    let value: serde_json::Value = serde_json::from_slice(header)
        .map_err(|_| "the file is not a Parlo voice file".to_string())?;
    if value["format"].as_str() != Some(FORMAT) {
        return Err("the file is not a Parlo voice file".to_string());
    }
    if value["version"].as_u64() != Some(VERSION) {
        return Err("this voice file has a version this app does not read".to_string());
    }
    if sample.len() < 44 || &sample[..4] != b"RIFF" {
        return Err("the voice file has no recorded sample".to_string());
    }
    let profile: Profile = serde_json::from_value(value["profile"].clone())
        .map_err(|_| "the voice file has no readable consent record".to_string())?;
    if !profile.usable() {
        return Err("that voice was revoked, so it cannot be imported".to_string());
    }
    Ok(Exchange {
        profile,
        sample: sample.to_vec(),
    })
}

pub fn import_into(store: &Store, bytes: &[u8]) -> Result<Profile, String> {
    let exchange = parse(bytes)?;
    let profile = store
        .create(
            &exchange.profile.label,
            &exchange.profile.language,
            exchange.profile.consent,
        )
        .map_err(|error| error.to_string())?;
    store
        .save_sample(&profile.id, &exchange.sample)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use parlo_core::voice::Consent;

    fn sample() -> Vec<u8> {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0u8; 40]);
        bytes
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let folder =
            std::env::temp_dir().join(format!("parlo-voicefile-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn recorded(store: &Store) -> Profile {
        let profile = store
            .create("Mine", "pt-PT", Consent::own_voice("playback", "here", 7))
            .unwrap();
        store.save_sample(&profile.id, &sample()).unwrap();
        profile
    }

    #[test]
    fn a_voice_travels_to_another_machine_as_one_file() {
        let source = scratch("source");
        let from = Store::new(&source);
        let profile = recorded(&from);
        let file = export(&profile, &from.sample(&profile.id).unwrap());

        let target = scratch("target");
        let to = Store::new(&target);
        let imported = import_into(&to, &file).unwrap();
        assert!(imported.id.starts_with("mine-"));
        assert_eq!(imported.label, "Mine");
        assert_eq!(imported.language, "pt-PT");
        assert_eq!(imported.consent.origin, profile.consent.origin);
        assert_eq!(imported.consent.purpose, profile.consent.purpose);
        assert_eq!(imported.consent.scope, profile.consent.scope);
        assert!(imported.usable());
        assert_eq!(to.sample(&imported.id).unwrap(), sample());
        assert_eq!(to.list().unwrap().len(), 1);
        std::fs::remove_dir_all(&source).unwrap();
        std::fs::remove_dir_all(&target).unwrap();
    }

    #[test]
    fn a_file_this_app_did_not_write_is_refused() {
        let target = scratch("refuse");
        let store = Store::new(&target);
        assert!(import_into(&store, b"hello").is_err());
        assert!(import_into(&store, b"{\"format\":\"other\"}\n\nRIFF0000").is_err());
        assert!(import_into(
            &store,
            b"{\"format\":\"parlo-voice-profile\",\"version\":9}\n\nRIFF0000"
        )
        .is_err());
        assert!(store.list().unwrap().is_empty());
        std::fs::remove_dir_all(&target).unwrap();
    }

    #[test]
    fn a_revoked_voice_does_not_come_back_through_a_file() {
        let source = scratch("revoked");
        let from = Store::new(&source);
        let profile = recorded(&from);
        from.revoke(&profile.id, 9).unwrap();
        let file = export(&from.get(&profile.id).unwrap(), &sample());
        let target = scratch("revoked-target");
        let to = Store::new(&target);
        let refused = import_into(&to, &file).unwrap_err();
        assert!(refused.contains("revoked"), "{refused}");
        std::fs::remove_dir_all(&source).unwrap();
        std::fs::remove_dir_all(&target).unwrap();
    }
}
