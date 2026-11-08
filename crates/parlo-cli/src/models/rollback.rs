use std::fs;
use std::path::{Path, PathBuf};

use super::*;

pub(super) fn previous_of(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    name.push(".previous");
    path.with_file_name(name)
}

pub fn has_previous(id: &str) -> bool {
    paths_for(id)
        .map(|paths| paths.iter().any(|path| previous_of(path).exists()))
        .unwrap_or(false)
}

pub fn rollback(id: &str) -> Result<String, String> {
    let paths = paths_for(id).ok_or_else(|| format!("{id} is not a model this app manages"))?;
    let mut restored = 0;
    for path in paths {
        let previous = previous_of(&path);
        if !previous.exists() {
            continue;
        }
        let removed = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
        if let Err(error) = removed {
            if path.exists() {
                return Err(format!(
                    "{} could not be replaced: {error}. Close Parlo and try again.",
                    path.display()
                ));
            }
        }
        fs::rename(&previous, &path).map_err(|error| {
            format!(
                "{} could not be restored: {error}. The previous copy is still there.",
                path.display()
            )
        })?;
        restored += 1;
    }
    if restored == 0 {
        return Err("there is no previous copy of this model to go back to".to_string());
    }
    let name = status()
        .into_iter()
        .find(|model| model.id == id)
        .map(|model| model.name)
        .unwrap_or_else(|| id.to_string());
    Ok(format!("{name} is back on the copy that was here before."))
}

pub(super) fn relocate_target(target: &str) -> Result<(PathBuf, PathBuf), String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return Err("give the folder the models should move to".to_string());
    }
    let destination = PathBuf::from(trimmed);
    if !destination.is_absolute() {
        return Err("that is a relative path; give the full path of the folder".to_string());
    }
    let source = models_root();
    if source == destination {
        return Err("the models are already there".to_string());
    }
    if destination.starts_with(&source) || source.starts_with(&destination) {
        return Err("the new folder cannot sit inside the old one, or around it".to_string());
    }
    if downloading(&status()) {
        return Err("a download is running; stop it before moving the models".to_string());
    }
    if !source.is_dir() {
        return Err(format!("there are no models in {} yet", source.display()));
    }
    Ok((source, destination))
}

#[cfg(windows)]
pub(super) fn free_bytes(path: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);
    let mut available: u64 = 0;
    let answered = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut available,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if answered == 0 {
        None
    } else {
        Some(available)
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetDiskFreeSpaceExW(
        directory: *const u16,
        available: *mut u64,
        total: *mut u64,
        free: *mut u64,
    ) -> i32;
}

#[cfg(not(windows))]
pub(super) fn free_bytes(_path: &Path) -> Option<u64> {
    None
}

pub const PAIR_FILES: &[&str] = &[
    "encoder_model.onnx",
    "decoder_model.onnx",
    "decoder_with_past_model.onnx",
    "config.json",
    "vocab.json",
    "source.spm",
    "target.spm",
];

pub fn install_pair(folder: &str, from: &str, to: &str) -> Result<String, String> {
    install_pair_in(&models_root(), folder, from, to)
}

pub(super) fn install_pair_in(
    root: &Path,
    folder: &str,
    from: &str,
    to: &str,
) -> Result<String, String> {
    let source = PathBuf::from(folder.trim());
    if !source.is_dir() {
        return Err(format!("{} is not a folder", source.display()));
    }
    let missing: Vec<&str> = PAIR_FILES
        .iter()
        .copied()
        .filter(|name| !source.join(*name).is_file())
        .collect();
    if !missing.is_empty() {
        return Err(format!("the export is missing {}", missing.join(", ")));
    }
    let destination = root.join("mt").join(format!("{from}-{to}-onnx"));
    if destination.exists() {
        return Err(format!(
            "{} already holds a pair; remove it before installing another",
            destination.display()
        ));
    }
    let bytes = copy_tree(&source, &destination)?;
    Ok(format!(
        "Installed {from} to {to} in {}, {}",
        destination.display(),
        describe_bytes(bytes)
    ))
}

pub fn free_space() -> Option<u64> {
    free_bytes(&models_root())
}

pub fn describe_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KB", bytes.div_ceil(1024))
    }
}

pub fn relocate_plan(target: &str) -> Result<(u64, bool), String> {
    let (source, destination) = relocate_target(target)?;
    Ok((directory_bytes(&source), same_tree(&source, &destination)))
}

pub fn relocate(target: &str) -> Result<String, String> {
    let (source, destination) = relocate_target(target)?;
    if same_tree(&source, &destination) {
        let size = describe_bytes(directory_bytes(&source));
        fs::remove_dir_all(&source).map_err(|error| {
            format!(
                "the copy in {} is complete but {} could not be removed: {error}",
                destination.display(),
                source.display()
            )
        })?;
        let mut config = crate::config::load();
        config.models_dir = destination.display().to_string();
        crate::config::save(&config).map_err(|error| {
            format!("the models were adopted but the folder was not saved: {error}")
        })?;
        return Ok(format!(
            "Adopted {size} already in {}. Nothing was copied.",
            destination.display()
        ));
    }
    let expected = directory_bytes(&source);
    let copied = copy_tree(&source, &destination)?;
    if copied != expected {
        return Err(format!(
            "the copy holds {copied} bytes but {} holds {expected}; nothing was deleted",
            source.display()
        ));
    }
    fs::remove_dir_all(&source).map_err(|error| {
        format!(
            "the copy is in place but {} could not be removed: {error}",
            source.display()
        )
    })?;
    let mut config = crate::config::load();
    config.models_dir = destination.display().to_string();
    crate::config::save(&config)
        .map_err(|error| format!("the models moved but the folder was not saved: {error}"))?;
    let mut message = format!(
        "Moved {} to {}. The old folder is gone.",
        describe_bytes(copied),
        destination.display()
    );
    if env::var("PARLO_MODELS_DIR").is_ok() {
        message.push_str(
            " This session still reads PARLO_MODELS_DIR, so start Parlo without it to use the new folder.",
        );
    }
    Ok(message)
}
