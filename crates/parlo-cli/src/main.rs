use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use parlo_core::gain::{normalize, peak as peak_amplitude};
use parlo_core::wav::write_pcm16;
use parlo_core::{EchoEngine, Engine, Wav};

mod actions;
mod agents;
mod ask;
mod audiobook;
mod batch;
mod bench;
mod calls;
mod captions;
mod captions_cmd;
mod catalog;
mod clipboard;
mod commands;
mod compare;
mod config;
mod config_cmd;
mod decisions;
mod diarize;
mod dictate;
mod doctor;
mod dub;
mod dub_jobs;
mod engines;
mod engines_cmd;
mod hardware;
mod history;
mod ingest;
mod insert;
mod install;
mod intelligence;
#[cfg(test)]
mod interface;
mod jobs;
mod listen;
mod locales;
mod loopback;
mod mcp;
mod meeting;
mod mic;
mod models;
#[cfg(test)]
mod network;
mod notes;
mod packs;
mod playback;
mod pronounce;
mod recommend;
mod record;
mod registry;
mod remote;
mod retention;
mod sample;
mod serve;
mod sidecar;
mod snippets;
mod soak;
mod speakers;
mod store;
mod story;
mod style;
mod subtitles;
mod summary;
mod translate;
mod undo;
mod update;
mod voicefile;
mod watch;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const ENGINE_LATENCY_MS: u32 = 1_500;

fn main() -> ExitCode {
    let sessions = parlo_core::sessions::Log::default_log();
    parlo_core::sessions::install_hook(sessions.clone());
    let session = sessions.begin();
    let args: Vec<String> = env::args().skip(1).collect();
    let outcome = match args.first().map(String::as_str) {
        Some("doctor") => doctor::doctor(),
        Some("demo") => engines_cmd::demo(args.get(1).map(String::as_str)),
        Some("bench") => engines_cmd::bench(&args[1..]),
        Some("translate") => translate::translate(&args[1..]),
        Some("stream") => translate::stream(&args[1..]),
        Some("dub") => dub_command(&args[1..]),
        Some("mic") => mic::command(&args[1..]),
        Some("record") => record_command(&args[1..]),
        Some("listen") => listen::command(&args[1..]),
        Some("captions") => captions_cmd::captions(&args[1..]),
        Some("dictate") => captions_cmd::dictate(&args[1..]),
        Some("app") => app_command(&args[1..]),
        Some("mcp") => mcp::serve(),
        Some("retention") => retention_command(&args[1..]),
        Some("remote") => remote_command(&args[1..]),
        Some("serve") => serve_command(&args[1..]),
        Some("soak") => soak::command(&args[1..]),
        Some("pronounce") => pronounce::command(&args[1..]),
        Some("undo") => undo_command(),
        Some("update") => update_command(&args[1..]),
        Some("config") => config_cmd::command(&args[1..]),
        Some("snippet") => snippets::command(&args[1..]),
        Some("pair") => translate::pair(&args[1..]),
        Some("engines") => engines_cmd::list_engines(),
        Some("speakers") => engines_cmd::list_speakers(),
        Some("install") => engines_cmd::install(&args[1..]),
        Some("uninstall") => engines_cmd::uninstall(&args[1..]),
        Some("profile") => engines_cmd::print_profile(),
        Some("--version" | "-V") => {
            println!("parlo {VERSION}");
            Ok(())
        }
        Some("--help" | "-h") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("unknown command: {other}")),
    };

    session.finish();
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!("parlo {VERSION} - speak your language, be heard in theirs");
    println!();
    println!("usage: parlo <command>");
    println!();
    println!("commands:");
    println!("  doctor                             check the local audio setup");
    println!(
        "  soak [--seconds N] [--interval S] [--report PATH] [--chart PATH]  run the server and record its memory"
    );
    println!(
        "  pronounce [word] [--text TEXT]   show how a name is spoken before the engine says it"
    );
    println!("  undo                               take back what the last dictation wrote");
    println!("  demo [out.wav]                     run the pipeline on synthetic speech");
    println!("  bench [--json]                     measure every engine on this machine");
    println!("  bench dictate [--runs N] [--json]  measure end of audio to text, with the stages");
    println!("  app [--from X --to Y] [--stt NAME] open the Parlo window");
    println!("  translate <in.wav> <out.wav>       translate a mono pcm16 wave file");
    println!("            [--engine echo|sidecar|cascade]  pick the engine, default echo");
    println!("            [--from pt --to en]             language pair for the cascade");
    println!(
        "  stream <in.wav>                    show partial, stable and final text as it arrives"
    );
    println!("            [--from pt --to en] [--stt parakeet|whisper] [--chunk MS]");
    println!("  dub <video|audio>                  dub a file into another language, offline");
    println!("            [--from pt --to en] [--out PATH] [--srt PATH] [--keep-background]");
    println!("  mic setup|status|remove            the virtual microphone: linux sets it up, windows routes a cable");
    println!("  record [out.wav] [--seconds N]     capture the microphone (--list for devices)");
    println!("  listen [out.wav] [--seconds N]     translate the microphone live");
    println!("            [--play --speaker N]            play it to an output device");
    println!("  speakers                           list output devices by number");
    println!("  install [--dir PATH] [--no-shortcut]  install for this user with a menu entry");
    println!("  uninstall [--purge]                remove the install, keep models unless purged");
    println!("  captions [--seconds N]             translate the microphone as text, no voice");
    println!(
        "  dictate [--from pt] [--seconds N]  type with your voice, text lands in the clipboard"
    );
    println!("            [--window S] [--no-copy]        no translation and no voice");
    println!(
        "  snippet add|list|remove [cue] [text]  say a cue and Parlo types the phrase you saved"
    );
    println!("  serve [--port N] [--token T]       local authenticated api on 127.0.0.1");
    println!(
        "  mcp                                meetings and dictations for an MCP client, on stdio"
    );
    println!("  retention [--days N] [--apply]     delete meetings older than N days, dry run by default");
    println!("  update [--check]                   install the latest release in place");
    println!("  engines                            list engines, licenses and voice levels");
    println!("  config [--engine NAME] [--from X]  show or write the defaults in the config file");
    println!("         [--to X] [--port N] [--device N]");
    println!("  profile                            detect hardware and recommend a mode");
    println!();
    println!("audio never leaves this machine.");
}

fn dub_command(args: &[String]) -> Result<(), String> {
    let stored = config::load();
    let mut from = stored.from.clone();
    let mut to = stored.to.clone();
    let mut out = None;
    let mut srt = None;
    let mut keep_background = false;
    let mut source = None;
    let mut index = 0;

    while index < args.len() {
        let argument = args[index].as_str();
        index += 1;
        match argument {
            "--from" => from = args.get(index).cloned().ok_or("--from needs a value")?,
            "--to" => to = args.get(index).cloned().ok_or("--to needs a value")?,
            "--out" => {
                out = Some(PathBuf::from(
                    args.get(index).cloned().ok_or("--out needs a value")?,
                ))
            }
            "--srt" => {
                srt = Some(PathBuf::from(
                    args.get(index).cloned().ok_or("--srt needs a value")?,
                ))
            }
            "--keep-background" => {
                keep_background = true;
                continue;
            }
            other if other.starts_with("--") => return Err(format!("unknown option: {other}")),
            other => {
                if source.is_some() {
                    return Err("dub takes one file".to_string());
                }
                source = Some(PathBuf::from(other));
                continue;
            }
        }
        index += 1;
    }

    let source = source.ok_or(
        "usage: parlo dub <video|audio> [--from pt] [--to en] [--out PATH] [--srt PATH] \
         [--keep-background]",
    )?;
    let (from, to) = (from.to_ascii_lowercase(), to.to_ascii_lowercase());
    set_direction(&from, &to)?;

    let result = dub::run(&dub::Request {
        source: &source,
        from: &from,
        to: &to,
        out,
        srt,
        cues: None,
        track: None,
        lines_dir: None,
        work: None,
        report: None,
        previous: None,
        retry: None,
        skip: None,
        keep_background,
        timing: "natural",
        fastest: 1.25,
        slowest: 0.9,
    })?;

    println!();
    println!("dubbed {} into {}", source.display(), to);
    println!("  output      {}", result.out);
    println!("  subtitles   {}", result.srt);
    println!("  lines       {}", result.lines.len());
    Ok(())
}

pub(crate) fn set_direction(from: &str, to: &str) -> Result<(), String> {
    config::check_pair(from, to)?;
    env::set_var("PARLO_CASCADE_SOURCE", from.to_ascii_lowercase());
    env::set_var("PARLO_CASCADE_TARGET", to.to_ascii_lowercase());
    Ok(())
}

pub(crate) fn build_engine(name: &str) -> Result<Box<dyn Engine>, String> {
    match name {
        "echo" => Ok(Box::new(EchoEngine::new("echo", ENGINE_LATENCY_MS, 1.0))),
        "sidecar" => spawn_sidecar("PARLO_HIBIKI_DIR", "engines/hibiki"),
        "cascade" => spawn_sidecar("PARLO_CASCADE_DIR", "engines/cascade"),
        other => Err(format!(
            "unknown engine: {other}, expected echo, sidecar or cascade"
        )),
    }
}

pub(crate) fn build_offline_engine(name: &str) -> Result<Box<dyn Engine>, String> {
    match name {
        "echo" => Ok(Box::new(EchoEngine::new("echo", ENGINE_LATENCY_MS, 1.0))),
        "sidecar" => spawn_offline_sidecar("PARLO_HIBIKI_DIR", "engines/hibiki"),
        "cascade" => spawn_offline_sidecar("PARLO_CASCADE_DIR", "engines/cascade"),
        other => Err(format!(
            "unknown engine: {other}, expected echo, sidecar or cascade"
        )),
    }
}

fn spawn_offline_sidecar(variable: &str, fallback: &str) -> Result<Box<dyn Engine>, String> {
    let dir = env::var(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(fallback));
    let engine = sidecar::SidecarEngine::spawn_offline(&dir).map_err(|error| error.to_string())?;
    Ok(Box::new(engine))
}

fn spawn_sidecar(variable: &str, fallback: &str) -> Result<Box<dyn Engine>, String> {
    let dir = env::var(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(fallback));
    let engine = sidecar::SidecarEngine::spawn(&dir).map_err(|error| error.to_string())?;
    Ok(Box::new(engine))
}

fn record_command(args: &[String]) -> Result<(), String> {
    let mut path = PathBuf::from("out/recording.wav");
    let mut seconds = 5.0f32;
    let mut device = None;
    let mut list = false;
    let mut probe = false;
    let mut system = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--list" => list = true,
            "--probe" => probe = true,
            "--system" => system = true,
            "--seconds" => {
                index += 1;
                seconds = args
                    .get(index)
                    .ok_or("--seconds needs a value")?
                    .parse()
                    .map_err(|_| "seconds must be a number".to_string())?;
            }
            "--device" => {
                index += 1;
                device = Some(
                    args.get(index)
                        .ok_or("--device needs a value")?
                        .parse()
                        .map_err(|_| "device must be a number".to_string())?,
                );
            }
            other if other.starts_with("--") => return Err(format!("unknown option: {other}")),
            other => path = PathBuf::from(other),
        }
        index += 1;
    }

    if system && (list || probe) {
        return Err(
            "--system records the speakers, so --list and --probe do not apply".to_string(),
        );
    }

    if list {
        for (index, name) in record::devices()?.iter().enumerate() {
            println!("{index}  {name}");
        }
        return Ok(());
    }

    if probe {
        let seconds = seconds.max(1.0);
        println!("listening to every input for {seconds:.0} s, speak normally throughout");
        for (index, name, peak) in record::probe(seconds)? {
            let decibels = if peak > 0.0 {
                20.0 * peak.log10()
            } else {
                -120.0
            };
            println!("{index}  {name:<40} {decibels:>7.1} dBFS");
        }
        return Ok(());
    }

    let mut recording = if system {
        loopback::capture(seconds)?
    } else {
        record::capture(seconds, device)?
    };
    let rate = recording.format.sample_rate();
    let duration = recording.samples.len() as f32 / rate as f32;
    let peak = peak_amplitude(&recording.samples);
    let gain = normalize(&mut recording.samples);
    let wav = Wav {
        format: recording.format,
        samples: recording.samples,
    };
    fs::write(&path, write_pcm16(&wav)).map_err(|error| format!("{}: {error}", path.display()))?;
    let decibels = if peak > 0.0 {
        20.0 * peak.log10()
    } else {
        -120.0
    };
    println!(
        "recorded {duration:.1} s at {rate} Hz to {}",
        path.display()
    );
    println!("  peak             {decibels:.1} dBFS");
    if gain > 1.0 {
        println!("  gain             {:+.1} dB", 20.0 * gain.log10());
    }
    if peak < 0.001 {
        if system {
            println!("  warning          the speakers played almost nothing, start some audio and record again");
        } else {
            println!("  warning          the microphone captured almost nothing, check the input device, mute and gain");
        }
    }
    Ok(())
}

fn undo_command() -> Result<(), String> {
    let last = undo::take()?;
    if let Err(error) = insert::erase(last.window, last.characters) {
        undo::restore(&last);
        return Err(error);
    }
    println!("took back {} characters", last.characters);
    Ok(())
}

fn serve_command(args: &[String]) -> Result<(), String> {
    let stored = config::load();
    let mut port = stored.port;
    let mut token = None;
    let mut certificate = None;
    let mut key = None;
    let mut engine = stored.engine.clone();
    let mut stt = stored.stt.clone();
    let mut from = stored.from.clone();
    let mut to = stored.to.clone();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--from" => {
                index += 1;
                from = args.get(index).cloned().ok_or("--from needs a value")?;
            }
            "--to" => {
                index += 1;
                to = args.get(index).cloned().ok_or("--to needs a value")?;
            }
            "--port" => {
                index += 1;
                port = args
                    .get(index)
                    .ok_or("--port needs a value")?
                    .parse()
                    .map_err(|_| "port must be a number".to_string())?;
            }
            "--token" => {
                index += 1;
                token = Some(args.get(index).ok_or("--token needs a value")?.clone());
            }
            "--tls-cert" => {
                index += 1;
                certificate = Some(std::path::PathBuf::from(
                    args.get(index).ok_or("--tls-cert needs a value")?,
                ));
            }
            "--tls-key" => {
                index += 1;
                key = Some(std::path::PathBuf::from(
                    args.get(index).ok_or("--tls-key needs a value")?,
                ));
            }
            "--engine" => {
                index += 1;
                engine = args.get(index).cloned().ok_or("--engine needs a value")?;
            }
            "--stt" => {
                index += 1;
                stt = args.get(index).cloned().ok_or("--stt needs a value")?;
            }
            other => return Err(format!("unknown option: {other}")),
        }
        index += 1;
    }

    set_direction(&from, &to)?;
    config::check_stt(&stt)?;
    let token = match token {
        Some(token) => token,
        None => serve::new_token()?,
    };
    match (certificate, key) {
        (Some(certificate), Some(key)) => serve::run_tls(
            port,
            token,
            serve::TlsFiles { certificate, key },
            engine,
            stt,
            from,
            to,
        ),
        (None, None) => serve::run(port, token, engine, stt, from, to),
        _ => Err("--tls-cert and --tls-key go together".to_string()),
    }
}

fn remote_command(args: &[String]) -> Result<(), String> {
    let file = args.first().ok_or("remote needs a 16 kHz mono wav file")?;
    let bytes = std::fs::read(file).map_err(|error| format!("{file}: {error}"))?;
    let config = config::load();
    let answer = remote::post(&config, "/translate", "audio/wav", &bytes)?;
    if answer.status != 200 {
        return Err(format!(
            "the remote backend answered {}: {}",
            answer.status,
            String::from_utf8_lossy(&answer.body).trim()
        ));
    }
    let transcript = answer
        .headers
        .iter()
        .find(|(name, _)| name == "x-parlo-transcript")
        .and_then(|(_, value)| {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.decode(value).ok()
        })
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    if transcript.is_empty() {
        println!("{} bytes of audio returned", answer.body.len());
    } else {
        println!("{transcript}");
    }
    Ok(())
}

fn update_command(args: &[String]) -> Result<(), String> {
    let mut check = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            other => return Err(format!("unknown option: {other}")),
        }
    }

    let latest = update::latest()?;
    if !update::is_newer(VERSION, &latest.version) {
        println!(
            "parlo {VERSION} is up to date, the latest release is {}",
            latest.version
        );
        return Ok(());
    }

    println!("parlo {VERSION} can update to {}", latest.version);
    if !latest.notes_url.is_empty() {
        println!("  notes      {}", latest.notes_url);
    }
    if check {
        return Ok(());
    }

    let binary = update::install(&latest)?;
    println!("  staged     {}", binary.display());
    update::schedule_replacement(&binary)?;
    println!("  replacing this binary two seconds after it exits");
    Ok(())
}

fn app_command(args: &[String]) -> Result<(), String> {
    let stored = config::load();
    let mut engine = stored.engine.clone();
    let mut stt = stored.stt.clone();
    let mut from = stored.from.clone();
    let mut to = stored.to.clone();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--from" => {
                index += 1;
                from = args.get(index).cloned().ok_or("--from needs a value")?;
            }
            "--to" => {
                index += 1;
                to = args.get(index).cloned().ok_or("--to needs a value")?;
            }
            "--port" => {
                return Err(
                    "the Parlo window picks its own port; use parlo serve --port for the local api"
                        .to_string(),
                )
            }
            "--engine" => {
                index += 1;
                engine = args.get(index).cloned().ok_or("--engine needs a value")?;
            }
            "--stt" => {
                index += 1;
                stt = args.get(index).cloned().ok_or("--stt needs a value")?;
            }
            other => return Err(format!("unknown option: {other}")),
        }
        index += 1;
    }

    set_direction(&from, &to)?;
    config::check_stt(&stt)?;
    let mut current = config::load();
    if current.engine != engine || current.stt != stt {
        current.engine = engine;
        current.stt = stt;
        config::save(&current)?;
    }

    let window = install::window_binary().ok_or_else(|| {
        "the Parlo window is not beside this program; install the latest release".to_string()
    })?;
    Command::new(&window)
        .spawn()
        .map_err(|error| format!("cannot open {}: {error}", window.display()))?;
    Ok(())
}

fn retention_command(args: &[String]) -> Result<(), String> {
    let mut days: u64 = 30;
    let mut apply = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--days" => {
                index += 1;
                days = args
                    .get(index)
                    .ok_or("--days needs a number")?
                    .parse()
                    .map_err(|_| "--days needs a number".to_string())?;
            }
            "--apply" => apply = true,
            other => {
                return Err(format!(
                    "unknown option {other}; usage: parlo retention [--days N] [--apply]"
                ))
            }
        }
        index += 1;
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default();
    let cutoff = retention::cutoff_ms(days, now_ms)?;
    let stale = retention::stale(&crate::meeting::list(), cutoff);
    if !apply {
        println!(
            "{days} days or older: {} meeting(s) would be deleted",
            stale.len()
        );
        if !stale.is_empty() {
            println!("run the same command with --apply to delete them");
        }
        return Ok(());
    }
    let mut removed = 0;
    for id in &stale {
        crate::meeting::delete(id)?;
        removed += 1;
    }
    println!("{removed} meeting(s) older than {days} days were deleted");
    println!(
        "dictations were not touched: the library keeps the last {} entries",
        crate::history::LIMIT
    );
    Ok(())
}

pub(crate) fn synthetic_speech(sample_rate: u32, duration_ms: u32) -> Vec<f32> {
    let total = (sample_rate as u64 * duration_ms as u64 / 1000) as usize;
    let bursts = [(0.0, 600.0), (800.0, 700.0), (1_700.0, 600.0)];
    let mut samples = vec![0.0; total];

    for (start_ms, length_ms) in bursts {
        let start = (sample_rate as f64 * start_ms / 1000.0) as usize;
        let length = (sample_rate as f64 * length_ms / 1000.0) as usize;
        for offset in 0..length.min(total.saturating_sub(start)) {
            let t = offset as f32 / sample_rate as f32;
            let envelope = (std::f32::consts::PI * offset as f32 / length as f32).sin();
            let voice = (2.0 * std::f32::consts::PI * 140.0 * t).sin()
                + 0.5 * (2.0 * std::f32::consts::PI * 280.0 * t).sin();
            samples[start + offset] = 0.25 * envelope * voice;
        }
    }
    samples
}
