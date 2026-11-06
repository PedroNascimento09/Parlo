use std::fs;
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "parlo-voice-pack";
pub const VERSION: u64 = 1;
pub const MAX_VOICES: usize = 8;
pub const NOTICE: &str = "Nothing in this pack leaves this computer.";

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Pack {
    pub format: String,
    pub version: u64,
    pub name: String,
    pub voices: Vec<String>,
    pub notice: String,
}

pub fn root() -> PathBuf {
    crate::config::data_root().join("voice-packs")
}

pub fn list() -> Result<Vec<Pack>, String> {
    list_at(&root())
}

pub fn read(name: &str) -> Result<Pack, String> {
    read_at(&root(), name)
}

pub fn import<F>(bytes: &[u8], revoked: F) -> Result<Pack, String>
where
    F: Fn(&str) -> bool,
{
    import_at(&root(), bytes, revoked, ensure_installed)
}

pub fn export(name: &str) -> Result<Vec<u8>, String> {
    let pack = read(name)?;
    let mut text = serde_json::to_string_pretty(&pack).map_err(|error| error.to_string())?;
    text.push('\n');
    Ok(text.into_bytes())
}

pub fn list_at(root: &Path) -> Result<Vec<Pack>, String> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut packs: Vec<Pack> = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let path = entry
            .map_err(|error| format!("{}: {error}", root.display()))?
            .path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let text =
            fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        packs.push(serde_json::from_str(&text).map_err(|_| {
            format!(
                "{} is not a pack this app can read; import it again",
                path.display()
            )
        })?);
    }
    packs.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(packs)
}

pub fn read_at(root: &Path, name: &str) -> Result<Pack, String> {
    let slug = slug(name)?;
    let path = root.join(format!("{slug}.json"));
    let text = fs::read_to_string(&path)
        .map_err(|_| format!("there is no voice pack called {}", name.trim()))?;
    serde_json::from_str(&text)
        .map_err(|_| format!("the pack called {} is not readable", name.trim()))
}

pub fn import_at<F, G>(root: &Path, bytes: &[u8], revoked: F, ensure: G) -> Result<Pack, String>
where
    F: Fn(&str) -> bool,
    G: FnOnce(&[String]) -> Result<(), String>,
{
    let pack: Pack = serde_json::from_slice(bytes)
        .map_err(|_| "the file is not a Parlo voice pack".to_string())?;
    if pack.format != FORMAT {
        return Err("the file is not a Parlo voice pack".to_string());
    }
    if pack.version != VERSION {
        return Err("this voice pack has a version this app does not read".to_string());
    }
    let name = pack.name.trim();
    let slug = slug(name)?;
    if !(2..=MAX_VOICES).contains(&pack.voices.len()) {
        return Err(format!(
            "a voice pack carries between 2 and {MAX_VOICES} voices"
        ));
    }
    let mut seen: Vec<String> = Vec::new();
    for voice in &pack.voices {
        let id = voice.trim();
        if revoked(id) {
            return Err(format!("{id} was revoked, so it cannot join a pack"));
        }
        match crate::registry::model(id) {
            Some(spec) if spec.kind == "voice" => {}
            _ => return Err(format!("{id} is not a voice this app can read aloud with")),
        }
        if seen.contains(&id.to_string()) {
            return Err(format!("a pack lists {id} only once"));
        }
        seen.push(id.to_string());
    }
    ensure(&seen)?;
    let stored = Pack {
        format: FORMAT.to_string(),
        version: VERSION,
        name: name.to_string(),
        voices: seen,
        notice: NOTICE.to_string(),
    };
    fs::create_dir_all(root).map_err(|error| format!("{}: {error}", root.display()))?;
    let path = root.join(format!("{slug}.json"));
    let mut text = serde_json::to_string_pretty(&stored).map_err(|error| error.to_string())?;
    text.push('\n');
    fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(stored)
}

fn ensure_installed(voices: &[String]) -> Result<(), String> {
    let models = crate::models::status();
    let missing = missing_units(voices, |id| {
        models.iter().any(|model| model.id == id && model.installed)
    });
    crate::models::install_blocking(&missing)
}

fn missing_units<F>(voices: &[String], installed: F) -> Vec<&'static str>
where
    F: Fn(&str) -> bool,
{
    let mut units: Vec<&'static str> = Vec::new();
    for voice in voices {
        if installed(voice) {
            continue;
        }
        if let Some(unit) = crate::models::unit_of(voice) {
            if !units.contains(&unit) {
                units.push(unit);
            }
        }
    }
    units
}

fn slug(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.chars().count() > 48 {
        return Err("a pack name is at most 48 characters".to_string());
    }
    let mut slug = String::new();
    for character in trimmed.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        return Err("a pack needs a name with a letter or a digit".to_string());
    }
    Ok(slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("parlo-packs-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn pack(name: &str, voices: &[&str]) -> Vec<u8> {
        serde_json::json!({
            "format": FORMAT,
            "version": VERSION,
            "name": name,
            "voices": voices,
            "notice": NOTICE,
        })
        .to_string()
        .into_bytes()
    }

    fn writes_nothing(root: &Path) -> bool {
        fs::read_dir(root)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true)
    }

    #[test]
    fn a_pack_imports_and_comes_back_with_its_voices() {
        let root = scratch("happy");
        let imported = import_at(
            &root,
            &pack("Studio pair", &["voice-pt", "voice-en"]),
            |_| false,
            |_| Ok(()),
        )
        .unwrap();
        assert_eq!(imported.name, "Studio pair");
        assert_eq!(imported.voices, ["voice-pt", "voice-en"]);
        assert_eq!(imported.notice, NOTICE);
        let listed = list_at(&root).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(read_at(&root, "studio pair").unwrap(), imported);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_pack_exports_as_the_file_it_imported() {
        let root = scratch("export");
        import_at(
            &root,
            &pack("Travel", &["voice-de", "voice-fr"]),
            |_| false,
            |_| Ok(()),
        )
        .unwrap();
        let written = read_at(&root, "Travel").unwrap();
        let text = serde_json::to_vec_pretty(&written).unwrap();
        let again = serde_json::from_slice::<Pack>(&text).unwrap();
        assert_eq!(again, written);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_valid_pack_installs_its_voices_before_it_writes_itself() {
        let root = scratch("install");
        let asked = std::cell::Cell::new(Vec::<String>::new());
        let imported = import_at(
            &root,
            &pack("Studio pair", &["voice-pt", "voice-en"]),
            |_| false,
            |voices| {
                assert!(
                    writes_nothing(&root),
                    "the pack wrote itself before installing its voices"
                );
                asked.set(voices.to_vec());
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(asked.take(), ["voice-pt", "voice-en"]);
        assert_eq!(list_at(&root).unwrap(), [imported]);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_pack_whose_voices_cannot_be_installed_writes_nothing() {
        let root = scratch("install-fail");
        let error = import_at(
            &root,
            &pack("Studio pair", &["voice-pt", "voice-en"]),
            |_| false,
            |_| Err("the voices could not be downloaded".to_string()),
        )
        .unwrap_err();
        assert!(error.contains("could not be downloaded"), "{error}");
        assert!(writes_nothing(&root));
        assert!(list_at(&root).unwrap().is_empty());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn importing_the_same_pack_twice_lists_it_once_and_leaves_no_orphans() {
        let root = scratch("twice");
        let calls = std::cell::Cell::new(0usize);
        for _ in 0..2 {
            import_at(
                &root,
                &pack("Studio pair", &["voice-pt", "voice-en"]),
                |_| false,
                |_| {
                    calls.set(calls.get() + 1);
                    Ok(())
                },
            )
            .unwrap();
        }
        assert_eq!(calls.get(), 2);
        assert_eq!(list_at(&root).unwrap().len(), 1);
        let mut files: Vec<String> = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        files.sort();
        assert_eq!(files, ["studio-pair.json"]);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn only_the_units_still_missing_are_installed() {
        let voices = vec![
            "voice-pt".to_string(),
            "voice-en".to_string(),
            "voice-de".to_string(),
        ];
        assert_eq!(missing_units(&voices, |_| false), ["voice"]);
        assert!(missing_units(&voices, |_| true).is_empty());
        assert!(missing_units(&voices, |id| id == "voice-pt"
            || id == "voice-en"
            || id == "voice-de")
        .is_empty());
    }

    #[test]
    fn a_pack_with_an_unknown_or_revoked_voice_is_refused_and_nothing_is_written() {
        let root = scratch("refuse");
        let unknown = import_at(
            &root,
            &pack("Broken", &["voice-pt", "parakeet"]),
            |_| false,
            |_| panic!("an invalid pack must not install anything"),
        )
        .unwrap_err();
        assert!(unknown.contains("parakeet"), "{unknown}");
        let revoked = import_at(
            &root,
            &pack("Revoked", &["voice-pt", "voice-en"]),
            |id| id == "voice-en",
            |_| panic!("a revoked pack must not install anything"),
        )
        .unwrap_err();
        assert!(revoked.contains("voice-en"), "{revoked}");
        assert!(revoked.contains("revoked"), "{revoked}");
        assert!(list_at(&root).unwrap().is_empty());
        assert!(writes_nothing(&root));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_pack_needs_two_distinct_voices_and_a_name() {
        let root = scratch("shape");
        let refused = |root: &Path, bytes: Vec<u8>| {
            import_at(
                root,
                &bytes,
                |_| false,
                |_| panic!("a refused pack must not install anything"),
            )
        };
        let single = refused(&root, pack("One", &["voice-pt"])).unwrap_err();
        assert!(single.contains("between 2"), "{single}");
        let twice = refused(&root, pack("Twice", &["voice-pt", "voice-pt"])).unwrap_err();
        assert!(twice.contains("only once"), "{twice}");
        let nameless = refused(&root, pack("  ", &["voice-pt", "voice-en"])).unwrap_err();
        assert!(nameless.contains("name"), "{nameless}");
        let foreign = refused(&root, b"{\"format\":\"other\"}".to_vec()).unwrap_err();
        assert!(foreign.contains("not a Parlo voice pack"), "{foreign}");
        assert!(list_at(&root).unwrap().is_empty());
        assert!(read_at(&root, "One").unwrap_err().contains("no voice pack"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_store_without_packs_lists_nothing_and_creates_no_folder() {
        let root = std::env::temp_dir().join(format!("parlo-packs-none-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        assert!(list_at(&root).unwrap().is_empty());
        assert!(!root.exists());
    }
}
