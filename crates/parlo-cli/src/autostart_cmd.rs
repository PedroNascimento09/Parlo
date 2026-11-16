use std::env;

use parlo_core::autostart;

use crate::install;

pub(crate) fn command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        None | Some("status") => status(),
        Some("on") => enable(),
        Some("off") => disable(),
        Some(other) => Err(format!("unknown autostart action: {other}")),
    }
}

fn status() -> Result<(), String> {
    match autostart::entry() {
        Some(entry) => println!("start at login    on    {entry}"),
        None => println!("start at login    off"),
    }
    Ok(())
}

fn enable() -> Result<(), String> {
    let command = window_command()?;
    autostart::enable(&command)?;
    println!("start at login    on    {command}");
    Ok(())
}

fn disable() -> Result<(), String> {
    autostart::disable()?;
    println!("start at login    off");
    Ok(())
}

fn window_command() -> Result<String, String> {
    let running =
        env::current_exe().map_err(|error| format!("cannot read this program: {error}"))?;
    let program = install::window_binary().unwrap_or_else(|| running.clone());
    let quoted = autostart::quote(&program);
    if program == running {
        Ok(format!("{quoted} app --hidden"))
    } else {
        Ok(format!("{quoted} --hidden"))
    }
}

pub(crate) fn summary() -> String {
    match autostart::entry() {
        Some(entry) => format!("on ({entry})"),
        None => "off".to_string(),
    }
}
