use std::process::Command;

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

#[cfg(test)]
mod tests {
    use super::*;

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
