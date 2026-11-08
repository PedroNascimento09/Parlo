use crate::{config, locales, remote, set_direction, update};

pub(crate) fn command(args: &[String]) -> Result<(), String> {
    let mut config = config::load();
    let mut written = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index].as_str();
        index += 1;
        let value = args
            .get(index)
            .cloned()
            .ok_or(format!("{option} needs a value"))?;
        match option {
            "--engine" => config.engine = value,
            "--stt" => {
                config::check_stt(&value)?;
                config.stt = value
            }
            "--from" => config.from = value,
            "--to" => config.to = value,
            "--shortcut" => config.shortcut = parlo_core::Shortcut::parse(&value)?.text(),
            "--port" => {
                config.port = value
                    .parse()
                    .map_err(|_| "port must be a number".to_string())?
            }
            "--channel" => config.update_channel = update::check_channel(&value)?,
            "--share" => {
                config.share = match value.as_str() {
                    "on" | "true" | "yes" => true,
                    "off" | "false" | "no" => false,
                    _ => return Err("share is either on or off".to_string()),
                }
            }
            "--pin" => config.share_pin = parlo_core::settings::check_pin(&value)?,
            "--remote-url" => config.remote_url = remote::check_url(&value)?,
            "--remote-token" => config.remote_token = value,
            "--remote-ca" => config.remote_ca = value,
            "--locale" => config.locale = locales::check(&value)?,
            "--share-port" => {
                config.share_port = value
                    .parse()
                    .map_err(|_| "the share port must be a number".to_string())?;
                if config.share_port < 1024 {
                    return Err("the share port must be 1024 or higher".to_string());
                }
            }
            "--device" => {
                config.device = Some(
                    value
                        .parse()
                        .map_err(|_| "device must be a number".to_string())?,
                )
            }
            other => return Err(format!("unknown option: {other}")),
        }
        written = true;
        index += 1;
    }

    if written {
        if config.share && config.share_pin.is_empty() {
            return Err("set a PIN with --pin before sharing this computer".to_string());
        }
        set_direction(&config.from, &config.to)?;
        let path = config::save(&config)?;
        println!("written {}", path.display());
    } else {
        println!("file       {}", config::path().display());
    }
    println!("  engine   {}", config.engine);
    println!("  stt      {}", config.stt);
    println!("  from     {}", config.from);
    println!("  to       {}", config.to);
    println!("  shortcut {}", config.shortcut);
    println!("  port     {}", config.port);
    println!("  channel  {}", config.update_channel);
    if config.share {
        println!("  share    on, port {}", config.share_port);
    } else {
        println!("  share    off");
    }
    if config.remote_url.is_empty() {
        println!("  remote   off");
    } else {
        println!("  remote   {}", config.remote_url);
    }
    println!("  locale   {}", config.locale);
    match config.device {
        Some(device) => println!("  device   {device}"),
        None => println!("  device   the system default"),
    }
    Ok(())
}
