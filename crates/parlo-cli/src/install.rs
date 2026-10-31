use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;
use crate::sidecar;

const SHORTCUT_NAME: &str = "Parlo";
const MARKER: &str = "install.json";

pub fn install_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        let base = env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| config::data_root());
        base.join("Programs").join("Parlo")
    } else if cfg!(target_os = "macos") {
        env::var("HOME")
            .map(|home| PathBuf::from(home).join("Applications").join("Parlo"))
            .unwrap_or_else(|_| config::data_root().join("app"))
    } else {
        env::var("HOME")
            .map(|home| PathBuf::from(home).join(".local").join("opt").join("parlo"))
            .unwrap_or_else(|_| config::data_root().join("app"))
    }
}

pub fn window_binary() -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "parlo-app.exe"
    } else {
        "parlo-app"
    };
    let beside = env::current_exe().ok()?.parent()?.join(name);
    beside.is_file().then_some(beside)
}

fn marker_path() -> PathBuf {
    config::data_root().join(MARKER)
}

fn shortcut_path() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        env::var("APPDATA").ok().map(|appdata| {
            PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join(format!("{SHORTCUT_NAME}.lnk"))
        })
    } else if cfg!(target_os = "linux") {
        env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                env::var("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
            })
            .ok()
            .map(|base| base.join("applications").join("parlo.desktop"))
    } else {
        None
    }
}

pub fn install(dir: Option<PathBuf>, shortcut: bool) -> Result<Vec<String>, String> {
    let target = dir.unwrap_or_else(install_dir);
    let target =
        std::path::absolute(&target).map_err(|error| format!("{}: {error}", target.display()))?;
    let binary = env::current_exe().map_err(|error| error.to_string())?;
    let name = binary
        .file_name()
        .ok_or_else(|| "the running binary has no name".to_string())?
        .to_owned();

    fs::create_dir_all(&target).map_err(|error| format!("{}: {error}", target.display()))?;
    let installed = target.join(&name);
    if installed != binary {
        fs::copy(&binary, &installed)
            .map_err(|error| format!("cannot copy {}: {error}", binary.display()))?;
    }

    let mut app = None;
    if let Some(source) = window_binary() {
        let destination = target.join(
            source
                .file_name()
                .ok_or_else(|| "the window has no name".to_string())?,
        );
        if destination != source {
            fs::copy(&source, &destination)
                .map_err(|error| format!("cannot copy {}: {error}", source.display()))?;
        }
        app = Some(destination);
    }

    let mut lines = vec![format!("installed   {}", installed.display())];
    if let Some(app) = &app {
        lines.push(format!("window      {}", app.display()));
    }
    if let Some(engine) = sidecar::bundled_engine() {
        let destination = match engine.parent().and_then(|parent| parent.file_name()) {
            Some(folder) if folder == "parlo-engine" => target.join("parlo-engine"),
            _ => target.join(
                engine
                    .file_name()
                    .ok_or_else(|| "the engine has no name".to_string())?,
            ),
        };
        if engine.parent() != Some(target.as_path()) {
            copy_tree(engine.parent().unwrap_or(&engine), &destination)?;
        }
        lines.push(format!("engine      {}", destination.display()));
    } else {
        lines.push(
            "engine      none beside this binary, install from a release archive to get one"
                .to_string(),
        );
    }

    if let Some(source_dir) = binary.parent() {
        for icon in ["parlo.ico", "parlo.png"] {
            let source = source_dir.join(icon);
            if source.is_file() && !target.join(icon).is_file() {
                fs::copy(&source, target.join(icon)).ok();
            }
        }
    }

    let marker = format!(
        "{{\"dir\":\"{}\",\"binary\":\"{}\"}}\n",
        json(&target.display().to_string()),
        json(&installed.display().to_string())
    );
    if let Some(parent) = marker_path().parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    fs::write(marker_path(), marker)
        .map_err(|error| format!("{}: {error}", marker_path().display()))?;

    let launcher = app.as_ref().unwrap_or(&installed).clone();
    if shortcut {
        match write_shortcut(&launcher, &target) {
            Ok(Some(path)) => lines.push(format!("shortcut    {}", path.display())),
            Ok(None) => lines.push(
                "shortcut    not created on this system, open the app with the installed binary"
                    .to_string(),
            ),
            Err(error) => lines.push(format!("shortcut    not created: {error}")),
        }
    }
    lines.push(format!("open it     \"{}\"", launcher.display()));
    Ok(lines)
}

pub fn uninstall(purge: bool) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    if let Some(path) = shortcut_path() {
        if path.is_file() && fs::remove_file(&path).is_ok() {
            lines.push(format!("removed     {}", path.display()));
        }
    }

    let marker = marker_path();
    let dir = fs::read_to_string(&marker)
        .ok()
        .and_then(|text| marker_dir(&text))
        .map(PathBuf::from)
        .unwrap_or_else(install_dir);

    let running_inside = env::current_exe()
        .ok()
        .and_then(|binary| binary.parent().map(Path::to_path_buf))
        .is_some_and(|parent| parent == dir);

    if running_inside && cfg!(target_os = "windows") {
        let script = format!(
            "ping -n 3 127.0.0.1 > nul & rmdir /s /q \"{}\"",
            dir.display()
        );
        Command::new("cmd")
            .args(["/c", &script])
            .spawn()
            .map_err(|error| format!("cannot schedule the removal: {error}"))?;
        lines.push(format!(
            "removing    {} in a few seconds, after this program exits",
            dir.display()
        ));
    } else if running_inside {
        lines.push(format!(
            "remove      {} yourself, this binary is running from there",
            dir.display()
        ));
    } else if dir.is_dir() {
        fs::remove_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        lines.push(format!("removed     {}", dir.display()));
    }

    if purge {
        let data = config::data_root();
        if data.is_dir() {
            fs::remove_dir_all(&data).map_err(|error| format!("{}: {error}", data.display()))?;
            lines.push(format!("removed     {} and every model", data.display()));
        }
    } else {
        lines.push(format!(
            "kept        {}, models included, remove it with --purge",
            config::data_root().display()
        ));
    }
    fs::remove_file(&marker).ok();
    Ok(lines)
}

fn marker_dir(text: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    value.get("dir")?.as_str().map(str::to_string)
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    if source.is_file() {
        fs::copy(source, target)
            .map(|_| ())
            .map_err(|error| format!("cannot copy {}: {error}", source.display()))?;
        return Ok(());
    }
    fs::create_dir_all(target).map_err(|error| format!("{}: {error}", target.display()))?;
    for entry in fs::read_dir(source).map_err(|error| format!("{}: {error}", source.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        copy_tree(&entry.path(), &target.join(entry.file_name()))?;
    }
    Ok(())
}

fn icon_for(dir: &Path, binary: &Path) -> PathBuf {
    let name = if cfg!(target_os = "windows") {
        "parlo.ico"
    } else {
        "parlo.png"
    };
    let candidate = dir.join(name);
    if candidate.is_file() {
        candidate
    } else {
        binary.to_path_buf()
    }
}

fn write_shortcut(binary: &Path, dir: &Path) -> Result<Option<PathBuf>, String> {
    let Some(path) = shortcut_path() else {
        return Ok(None);
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let icon = icon_for(dir, binary);
    if cfg!(target_os = "windows") {
        let script = format!(
            "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}');\
             $s.TargetPath='{}';$s.Arguments='app';$s.WorkingDirectory='{}';\
             $s.IconLocation='{}';$s.Save()",
            path.display(),
            binary.display(),
            dir.display(),
            icon.display()
        );
        let status = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .map_err(|error| error.to_string())?;
        if !status.success() {
            return Err("powershell refused to write the shortcut".to_string());
        }
        return Ok(Some(path));
    }
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName={SHORTCUT_NAME}\n\
         Comment=Speak your language, be heard in theirs\n\
         Exec=\"{}\" app\nPath={}\nIcon={}\nTerminal=false\n\
         Categories=AudioVideo;Utility;\n",
        binary.display(),
        dir.display(),
        icon.display()
    );
    fs::write(&path, entry).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(Some(path))
}

fn json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_directories_are_per_user() {
        let dir = install_dir();
        assert!(dir.is_absolute());
        assert!(dir.to_string_lossy().to_lowercase().contains("parlo"));
    }

    #[test]
    fn reads_the_install_marker() {
        assert_eq!(
            marker_dir("{\"dir\":\"C:\\\\Parlo\"}"),
            Some("C:\\Parlo".to_string())
        );
        assert_eq!(marker_dir("not json"), None);
        assert_eq!(marker_dir("{\"other\":1}"), None);
    }

    #[test]
    fn escapes_json_values() {
        assert_eq!(json("C:\\a\"b"), "C:\\\\a\\\"b");
    }
}
