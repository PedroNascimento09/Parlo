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
    import_at(&root(), bytes, revoked)
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

pub fn import_at<F>(root: &Path, bytes: &[u8], revoked: F) -> Result<Pack, String>
where
    F: Fn(&str) -> bool,
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

    #[test]
    fn a_pack_imports_and_comes_back_with_its_voices() {
        let root = scratch("happy");
        let imported = import_at(
            &root,
            &pack("Studio pair", &["voice-pt", "voice-en"]),
            |_| false,
        )
        .unwrap();
        assert_eq!(imported.name, "Studio pair");
        assert_eq!(imported.voices, ["voice-pt", "voice-en"]);
        assert_eq!(imported.notice, NOTICE);
        let listed = list_at(&root).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(read_at(&root, "studio pair").unwrap(), imported);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_pack_exports_as_the_file_it_imported() {
        let root = scratch("export");
        import_at(&root, &pack("Travel", &["voice-de", "voice-fr"]), |_| false).unwrap();
        let written = read_at(&root, "Travel").unwrap();
        let text = serde_json::to_vec_pretty(&written).unwrap();
        let again = serde_json::from_slice::<Pack>(&text).unwrap();
        assert_eq!(again, written);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_pack_with_an_unknown_or_revoked_voice_is_refused_and_nothing_is_written() {
        let root = scratch("refuse");
        let unknown =
            import_at(&root, &pack("Broken", &["voice-pt", "parakeet"]), |_| false).unwrap_err();
        assert!(unknown.contains("parakeet"), "{unknown}");
        let revoked = import_at(&root, &pack("Revoked", &["voice-pt", "voice-en"]), |id| {
            id == "voice-en"
        })
        .unwrap_err();
        assert!(revoked.contains("voice-en"), "{revoked}");
        assert!(revoked.contains("revoked"), "{revoked}");
        assert!(list_at(&root).unwrap().is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_pack_needs_two_distinct_voices_and_a_name() {
        let root = scratch("shape");
        let single = import_at(&root, &pack("One", &["voice-pt"]), |_| false).unwrap_err();
        assert!(single.contains("between 2"), "{single}");
        let twice =
            import_at(&root, &pack("Twice", &["voice-pt", "voice-pt"]), |_| false).unwrap_err();
        assert!(twice.contains("only once"), "{twice}");
        let nameless =
            import_at(&root, &pack("  ", &["voice-pt", "voice-en"]), |_| false).unwrap_err();
        assert!(nameless.contains("name"), "{nameless}");
        let foreign = import_at(&root, b"{\"format\":\"other\"}", |_| false).unwrap_err();
        assert!(foreign.contains("not a Parlo voice pack"), "{foreign}");
        assert!(list_at(&root).unwrap().is_empty());
        assert!(read_at(&root, "One").unwrap_err().contains("no voice pack"));
        std::fs::remove_dir_all(&root).unwrap();
    }
}
