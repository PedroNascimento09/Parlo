use std::path::Path;

const NAME: &str = "Parlo";

pub fn quote(path: &Path) -> String {
    let text = path.display().to_string();
    if text.contains(' ') {
        format!("\"{text}\"")
    } else {
        text
    }
}

pub fn entry() -> Option<String> {
    platform::entry()
}

pub fn enable(command: &str) -> Result<(), String> {
    platform::enable(command)
}

pub fn disable() -> Result<(), String> {
    platform::disable()
}

pub fn launch_agent(program: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n\
<dict>\n\
  <key>Label</key>\n\
  <string>dev.parlo.Parlo</string>\n\
  <key>ProgramArguments</key>\n\
  <array>\n\
    <string>{}</string>\n\
  </array>\n\
  <key>RunAtLoad</key>\n\
  <true/>\n\
  <key>ProcessType</key>\n\
  <string>Interactive</string>\n\
</dict>\n\
</plist>\n",
        escape(program)
    )
}

pub fn desktop_entry(program: &str) -> String {
    format!(
        "[Desktop Entry]\n\
Type=Application\n\
Name={NAME}\n\
Comment=Speak your language, be heard in theirs\n\
Exec={program}\n\
Terminal=false\n\
X-GNOME-Autostart-enabled=true\n"
    )
}

pub fn agent_program(contents: &str) -> Option<String> {
    let start = contents.find("<array>")?;
    let rest = &contents[start..];
    let open = rest.find("<string>")? + "<string>".len();
    let close = rest[open..].find("</string>")? + open;
    Some(rest[open..close].to_string())
}

pub fn desktop_program(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let value = line.strip_prefix("Exec=")?.trim();
        Some(value.trim_matches('"').to_string())
    })
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(target_os = "windows")]
mod platform {
    use std::process::Command;

    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

    pub fn entry() -> Option<String> {
        let output = Command::new("reg")
            .args(["query", RUN_KEY, "/v", super::NAME])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        parse(&String::from_utf8_lossy(&output.stdout))
    }

    pub fn enable(command: &str) -> Result<(), String> {
        run(&[
            "add",
            RUN_KEY,
            "/v",
            super::NAME,
            "/t",
            "REG_SZ",
            "/d",
            command,
            "/f",
        ])
    }

    pub fn disable() -> Result<(), String> {
        run(&["delete", RUN_KEY, "/v", super::NAME, "/f"])
    }

    fn run(args: &[&str]) -> Result<(), String> {
        let output = Command::new("reg")
            .args(args)
            .output()
            .map_err(|error| format!("reg: {error}"))?;
        if output.status.success() {
            return Ok(());
        }
        let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if reason.is_empty() {
            "the registry refused the change".to_string()
        } else {
            reason
        })
    }

    fn parse(output: &str) -> Option<String> {
        output.lines().find_map(|line| {
            let mut fields = line.split_whitespace();
            if fields.next()? != super::NAME || fields.next()? != "REG_SZ" {
                return None;
            }
            let value = fields.collect::<Vec<_>>().join(" ");
            Some(value.trim_matches('"').to_string())
        })
    }

    #[cfg(test)]
    mod tests {
        use super::parse;

        #[test]
        fn the_registry_value_is_read_through_its_quotes_and_spaces() {
            let output = "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run\r\n    Parlo    REG_SZ    \"C:\\Program Files\\Parlo\\parlo-app.exe\"\r\n\r\n";
            assert_eq!(
                parse(output).as_deref(),
                Some("C:\\Program Files\\Parlo\\parlo-app.exe")
            );
        }

        #[test]
        fn another_value_is_not_mistaken_for_parlo() {
            let output = "    Other    REG_SZ    C:\\Other\\app.exe";
            assert_eq!(parse(output), None);
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::fs;
    use std::path::PathBuf;

    fn file() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from).map(|home| {
            home.join("Library")
                .join("LaunchAgents")
                .join("dev.parlo.Parlo.plist")
        })
    }

    pub fn entry() -> Option<String> {
        let contents = fs::read_to_string(file()?).ok()?;
        super::agent_program(&contents)
    }

    pub fn enable(command: &str) -> Result<(), String> {
        let path = file().ok_or_else(|| "no home folder".to_string())?;
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
        }
        fs::write(&path, super::launch_agent(command))
            .map_err(|error| format!("{}: {error}", path.display()))
    }

    pub fn disable() -> Result<(), String> {
        let path = file().ok_or_else(|| "no home folder".to_string())?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("{}: {error}", path.display())),
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use std::fs;
    use std::path::PathBuf;

    fn file() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(base.join("autostart").join("parlo.desktop"))
    }

    pub fn entry() -> Option<String> {
        let contents = fs::read_to_string(file()?).ok()?;
        super::desktop_program(&contents)
    }

    pub fn enable(command: &str) -> Result<(), String> {
        let path = file().ok_or_else(|| "no config folder".to_string())?;
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
        }
        fs::write(&path, super::desktop_entry(command))
            .map_err(|error| format!("{}: {error}", path.display()))
    }

    pub fn disable() -> Result<(), String> {
        let path = file().ok_or_else(|| "no config folder".to_string())?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("{}: {error}", path.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_with_a_space_is_quoted_for_a_command_line() {
        assert_eq!(
            quote(Path::new("C:\\Program Files\\Parlo\\parlo-app.exe")),
            "\"C:\\Program Files\\Parlo\\parlo-app.exe\""
        );
        assert_eq!(quote(Path::new("/usr/bin/parlo")), "/usr/bin/parlo");
    }

    #[test]
    fn the_agent_launches_the_program_on_login() {
        let agent = launch_agent("/Applications/Parlo/parlo-app");
        assert!(agent.contains("<string>/Applications/Parlo/parlo-app</string>"));
        assert!(agent.contains("<key>RunAtLoad</key>"));
        assert_eq!(
            agent_program(&agent).as_deref(),
            Some("/Applications/Parlo/parlo-app")
        );
    }

    #[test]
    fn an_ampersand_in_the_program_does_not_break_the_agent() {
        let agent = launch_agent("/Users/a & b/parlo-app");
        assert!(agent.contains("/Users/a &amp; b/parlo-app"));
        assert_eq!(
            agent_program(&agent).as_deref(),
            Some("/Users/a &amp; b/parlo-app")
        );
    }

    #[test]
    fn the_desktop_entry_runs_the_program_on_login() {
        let entry = desktop_entry("/home/ana/.local/opt/parlo/parlo-app");
        assert!(entry.contains("Type=Application"));
        assert!(entry.contains("X-GNOME-Autostart-enabled=true"));
        assert_eq!(
            desktop_program(&entry).as_deref(),
            Some("/home/ana/.local/opt/parlo/parlo-app")
        );
    }

    #[test]
    fn a_desktop_entry_without_exec_reads_as_absent() {
        assert_eq!(desktop_program("[Desktop Entry]\nType=Application\n"), None);
    }
}
