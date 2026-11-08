use std::env;
use std::process::Command;

use crate::playback;
pub const SINK_NAME: &str = "parlo";
pub const SINK_DESCRIPTION: &str = "Parlo";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicCommand {
    pub program: &'static str,
    pub args: Vec<String>,
}

impl MicCommand {
    pub fn run(&self) -> Result<String, String> {
        let output = Command::new(self.program)
            .args(&self.args)
            .output()
            .map_err(|error| format!("{}: {error}", self.program))?;
        if !output.status.success() {
            let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(format!("{} failed: {reason}", self.program));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

pub fn setup_command() -> MicCommand {
    MicCommand {
        program: "pactl",
        args: vec![
            "load-module".to_string(),
            "module-null-sink".to_string(),
            format!("sink_name={SINK_NAME}"),
            format!("sink_properties=device.description={SINK_DESCRIPTION}"),
        ],
    }
}

pub fn remove_command(module_id: u32) -> MicCommand {
    MicCommand {
        program: "pactl",
        args: vec!["unload-module".to_string(), module_id.to_string()],
    }
}

pub fn list_command() -> MicCommand {
    MicCommand {
        program: "pactl",
        args: vec![
            "list".to_string(),
            "short".to_string(),
            "modules".to_string(),
        ],
    }
}

pub fn find_module(listing: &str) -> Option<u32> {
    let sink = format!("sink_name={SINK_NAME}");
    listing.lines().find_map(|line| {
        let mut fields = line.split('\t');
        let id = fields.next()?.trim().parse().ok()?;
        let name = fields.next()?.trim();
        let args = fields.next().unwrap_or_default();
        (name == "module-null-sink" && args.split_whitespace().any(|arg| arg == sink)).then_some(id)
    })
}

pub fn cable_for(device: &str) -> Option<&'static str> {
    let folded = device.to_lowercase();
    if folded.contains("cable input") {
        return Some("VB-CABLE");
    }
    if folded.contains("voicemeeter input") || folded.contains("voicemeeter aux input") {
        return Some("VoiceMeeter");
    }
    if folded.contains("scream") {
        return Some("Scream");
    }
    None
}

pub fn windows_guidance(devices: &[String]) -> String {
    for (index, device) in devices.iter().enumerate() {
        if let Some(cable) = cable_for(device) {
            return format!(
                "virtual microphone  {device} ({cable}), output device {index}\noutput              parlo listen --play --speaker {index} sends the translation into it\nmicrophone          the calling app picks the {cable} recording side, not the one above"
            );
        }
    }
    "virtual microphone  none installed\nnext                Windows gives no program a microphone of its own without a signed driver. Install VB-CABLE, restart, then run parlo mic status again; Parlo never installs a driver by itself".to_string()
}

pub const WINDOWS_REMOVE: &str = "Parlo installed no driver on this computer: remove the virtual cable with its own uninstaller, and Parlo stops using it";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vb_cable_output_is_named_with_its_index() {
        let devices = vec![
            "Speakers (Realtek(R) Audio)".to_string(),
            "CABLE Input (VB-Audio Virtual Cable)".to_string(),
        ];
        let guidance = windows_guidance(&devices);
        assert!(
            guidance.contains("CABLE Input (VB-Audio Virtual Cable) (VB-CABLE)"),
            "{guidance}"
        );
        assert!(guidance.contains("--speaker 1"), "{guidance}");
    }

    #[test]
    fn voicemeeter_is_recognised_too() {
        let devices = vec!["VoiceMeeter Input (VB-Audio VoiceMeeter VAIO)".to_string()];
        assert_eq!(cable_for(&devices[0]), Some("VoiceMeeter"));
        assert!(windows_guidance(&devices).contains("output device 0"));
    }

    #[test]
    fn without_a_cable_the_guide_names_the_driver_and_what_parlo_does_not_do() {
        let devices = vec!["Speakers (Realtek(R) Audio)".to_string()];
        let guidance = windows_guidance(&devices);
        assert!(guidance.contains("none installed"), "{guidance}");
        assert!(guidance.contains("VB-CABLE"), "{guidance}");
        assert!(guidance.contains("never installs a driver"), "{guidance}");
    }

    #[test]
    fn a_plain_speaker_is_not_a_virtual_microphone() {
        assert_eq!(cable_for("Speakers (Realtek(R) Audio)"), None);
        assert_eq!(cable_for("Headphones"), None);
    }

    #[test]
    fn setup_creates_a_named_null_sink() {
        let command = setup_command();
        assert_eq!(command.program, "pactl");
        assert!(command.args.contains(&"load-module".to_string()));
        assert!(command.args.contains(&"sink_name=parlo".to_string()));
        assert!(command
            .args
            .contains(&"sink_properties=device.description=Parlo".to_string()));
    }

    #[test]
    fn finds_the_parlo_sink_among_others() {
        let listing = "12\tmodule-null-sink\tsink_name=discord\n13\tmodule-null-sink\tsink_name=parlo sink_properties=device.description=Parlo\n14\tmodule-echo-cancel\n";
        assert_eq!(find_module(listing), Some(13));
    }

    #[test]
    fn ignores_other_virtual_sinks() {
        let listing = "12\tmodule-null-sink\tsink_name=discord\n15\tmodule-null-sink\tsink_name=parlo-extra\n";
        assert_eq!(find_module(listing), None);
    }
    #[test]
    fn builds_an_unload_command_with_the_module_id() {
        let command = remove_command(13);
        assert_eq!(command.program, "pactl");
        assert_eq!(
            command.args,
            vec!["unload-module".to_string(), "13".to_string()]
        );
    }
}

fn require_linux() -> Result<(), String> {
    if cfg!(target_os = "linux") {
        Ok(())
    } else {
        Err(format!(
            "virtual microphone automation is linux only for now, see docs/VIRTUAL_MIC.md for {}",
            env::consts::OS
        ))
    }
}

pub(crate) fn command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("setup") => setup(),
        Some("status") => status(),
        Some("remove") => remove(),
        _ => Err("usage: parlo mic <setup|status|remove>".to_string()),
    }
}

pub(crate) fn setup() -> Result<(), String> {
    if cfg!(windows) {
        let devices = playback::devices()?;
        println!("{}", windows_guidance(&devices));
        return Ok(());
    }
    require_linux()?;
    let listing = list_command().run()?;
    if let Some(id) = find_module(&listing) {
        println!("Parlo microphone already loaded as module {id}");
        return Ok(());
    }
    let module = setup_command().run()?;
    println!("Parlo microphone ready (module {})", module.trim());
    println!("select the Parlo monitor as the microphone in your calling app");
    Ok(())
}

pub(crate) fn status() -> Result<(), String> {
    if cfg!(windows) {
        let devices = playback::devices()?;
        println!("{}", windows_guidance(&devices));
        return Ok(());
    }
    require_linux()?;
    let listing = list_command().run()?;
    match find_module(&listing) {
        Some(id) => println!("Parlo microphone active (module {id})"),
        None => println!("Parlo microphone not loaded, run: parlo mic setup"),
    }
    Ok(())
}

pub(crate) fn remove() -> Result<(), String> {
    if cfg!(windows) {
        println!("{}", WINDOWS_REMOVE);
        return Ok(());
    }
    require_linux()?;
    let listing = list_command().run()?;
    let id = find_module(&listing).ok_or("Parlo microphone is not loaded")?;
    remove_command(id).run()?;
    println!("removed the Parlo microphone (module {id})");
    Ok(())
}
