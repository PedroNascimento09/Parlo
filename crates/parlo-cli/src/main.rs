use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use parlo_core::gain::{normalize, peak as peak_amplitude};
use parlo_core::wav::{read_pcm16, write_pcm16};
use parlo_core::{AudioFormat, EchoEngine, Engine, LatencyBudget, Pipeline, PipelineConfig, Wav};

mod actions;
mod ask;
mod captions;
mod catalog;
mod config;
mod decisions;
mod dictate;
mod dub;
mod engines;
mod history;
mod insert;
mod install;
mod intelligence;
mod meeting;
mod mic;
mod models;
mod playback;
mod record;
mod serve;
mod sidecar;
mod summary;
mod update;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const ENGINE_LATENCY_MS: u32 = 1_500;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let outcome = match args.first().map(String::as_str) {
        Some("doctor") => doctor(),
        Some("demo") => demo(args.get(1).map(String::as_str)),
        Some("translate") => translate(&args[1..]),
        Some("stream") => stream_command(&args[1..]),
        Some("dub") => dub_command(&args[1..]),
        Some("mic") => mic_command(&args[1..]),
        Some("record") => record_command(&args[1..]),
        Some("listen") => listen_command(&args[1..]),
        Some("captions") => captions_command(&args[1..]),
        Some("dictate") => dictate_command(&args[1..]),
        Some("app") => app_command(&args[1..]),
        Some("serve") => serve_command(&args[1..]),
        Some("update") => update_command(&args[1..]),
        Some("config") => config_command(&args[1..]),
        Some("pair") => pair_command(&args[1..]),
        Some("engines") => list_engines(),
        Some("speakers") => list_speakers(),
        Some("install") => install_command(&args[1..]),
        Some("uninstall") => uninstall_command(&args[1..]),
        Some("profile") => print_profile(),
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
    println!("  demo [out.wav]                     run the pipeline on synthetic speech");
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
    println!("  mic setup|status|remove            manage the virtual microphone (linux)");
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
    println!("  serve [--port N] [--token T]       local authenticated api on 127.0.0.1");
    println!("  update [--check]                   install the latest release in place");
    println!("  engines                            list engines, licenses and voice levels");
    println!("  config [--engine NAME] [--from X]  show or write the defaults in the config file");
    println!("         [--to X] [--port N] [--device N]");
    println!("  profile                            detect hardware and recommend a mode");
    println!();
    println!("audio never leaves this machine.");
}

fn doctor() -> Result<(), String> {
    let input = AudioFormat::mono(48_000).map_err(|e| e.to_string())?;
    let output = AudioFormat::mono(16_000).map_err(|e| e.to_string())?;
    let budget = LatencyBudget::new(20, ENGINE_LATENCY_MS, 5);

    println!("parlo {VERSION} doctor");
    println!();
    println!(
        "platform          {} {}",
        env::consts::OS,
        env::consts::ARCH
    );
    println!(
        "capture format    {} Hz mono, 20 ms frames",
        input.sample_rate()
    );
    println!("engine format     {} Hz mono", output.sample_rate());
    println!("engines           echo, sidecar (needs python and engines/hibiki)");
    println!();
    println!("latency budget");
    println!("  capture         {:>5} ms", budget.capture_ms);
    println!("  engine          {:>5} ms", budget.engine_ms);
    println!("  output          {:>5} ms", budget.output_ms);
    println!(
        "  total           {:>5} ms (target {} ms)",
        budget.total_ms(),
        LatencyBudget::TARGET_MS
    );
    println!();
    if budget.is_within_target() {
        println!("status            within target");
    } else if budget.is_within_ceiling() {
        println!("status            above target, within ceiling");
    } else {
        println!("status            above the usable ceiling");
    }
    println!("privacy           local only, no network calls, no telemetry");
    println!("engine binary     {}", engine_binary());
    println!("models            {}", models_summary());
    println!("config            {}", config_summary());
    println!("data              {}", config::data_root().display());
    println!("ffmpeg            {}", ffmpeg_version());

    let cores = engines::detect_cores();
    let profile = engines::profile_for_cores(cores);
    println!(
        "profile           {} ({} cores), voice level {}",
        profile.label(),
        cores,
        engines::fidelity_for(profile)
    );
    println!();
    println!("first session");
    let checks = first_session_checks();
    let mut failed = 0;
    for (ok, line) in &checks {
        if !ok {
            failed += 1;
        }
        println!("  {:<4} {line}", if *ok { "ok" } else { "fail" });
    }
    println!();
    if failed > 0 {
        println!("status            not ready: {failed} checks need attention");
        std::process::exit(1);
    }
    println!("status            ready for a first session");
    Ok(())
}

fn first_session_checks() -> Vec<(bool, String)> {
    let models = models::status();
    let mut checks = Vec::new();
    let missing: Vec<String> = models
        .iter()
        .filter(|model| !model.installed)
        .map(|model| model.name.clone())
        .collect();
    checks.push((
        missing.is_empty(),
        if missing.is_empty() {
            "every model this build knows is on disk".to_string()
        } else {
            format!("missing {}", missing.join(", "))
        },
    ));
    let chosen = config::load().stt;
    let dictation = models::dictation_ready_for(&models, &chosen);
    checks.push((
        dictation,
        format!(
            "dictation uses {chosen}, and that recogniser is {}",
            if dictation { "on disk" } else { "not on disk" }
        ),
    ));
    let shortcut = config::load().shortcut;
    checks.push((
        !shortcut.trim().is_empty(),
        format!("dictation shortcut {shortcut}"),
    ));
    let settings = config::path();
    checks.push((
        settings.is_file(),
        format!("settings at {}", settings.display()),
    ));
    let data = config::data_root();
    let probe = data.join(".first-session-check");
    let writable = std::fs::create_dir_all(&data)
        .and_then(|()| std::fs::write(&probe, b"ok"))
        .is_ok();
    let _ = std::fs::remove_file(&probe);
    checks.push((
        writable,
        format!(
            "data folder {} is {}",
            data.display(),
            if writable { "writable" } else { "not writable" }
        ),
    ));
    checks.push(match sidecar::bundled_engine() {
        Some(path) => (true, format!("engine at {}", path.display())),
        None => (
            true,
            "no bundled engine; a source checkout starts python instead".to_string(),
        ),
    });
    checks
}

fn engine_binary() -> String {
    match sidecar::bundled_engine() {
        Some(path) => path.display().to_string(),
        None => "none, this build uses python from a source checkout".to_string(),
    }
}

fn models_summary() -> String {
    let models = models::status();
    let installed = models.iter().filter(|model| model.installed).count();
    format!(
        "{} ({} of {} installed)",
        models::models_root().display(),
        installed,
        models.len()
    )
}

fn config_summary() -> String {
    let path = config::path();
    if path.is_file() {
        path.display().to_string()
    } else {
        format!("{} (not written yet, defaults in use)", path.display())
    }
}

fn ffmpeg_version() -> String {
    let output = Command::new("ffmpeg").arg("-version").output();
    match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("found, version unknown")
            .trim()
            .to_string(),
        _ => "not found, dubbing needs it".to_string(),
    }
}

fn list_engines() -> Result<(), String> {
    println!("engines");
    let licence_width = engines::ENGINES
        .iter()
        .map(|engine| engine.license.label().len())
        .max()
        .unwrap_or(0);
    for engine in engines::ENGINES {
        let state = if engine.available {
            "available"
        } else {
            "planned"
        };
        let needs = if engine.requires_gpu { "gpu" } else { "cpu ok" };
        println!(
            "  {:<8} {:<9} {:<width$} {:<24} {:<8} {}",
            engine.id,
            state,
            engine.license.label(),
            engine.hardware,
            needs,
            engine.license.commercial_note(),
            width = licence_width
        );
        let levels: Vec<String> = engine
            .fidelity
            .iter()
            .map(|level| level.to_string())
            .collect();
        println!("           {} | {}", engine.name, levels.join(", "));
    }
    println!();
    println!(
        "default engine: {} ({})",
        engines::default_engine().id,
        engines::default_engine().license.label()
    );
    Ok(())
}

fn install_command(args: &[String]) -> Result<(), String> {
    let mut dir = None;
    let mut shortcut = true;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                index += 1;
                dir = Some(PathBuf::from(args.get(index).ok_or("--dir needs a value")?));
            }
            "--no-shortcut" => shortcut = false,
            other => return Err(format!("unknown option: {other}")),
        }
        index += 1;
    }
    for line in install::install(dir, shortcut)? {
        println!("{line}");
    }
    Ok(())
}

fn uninstall_command(args: &[String]) -> Result<(), String> {
    let mut purge = false;
    for argument in args {
        match argument.as_str() {
            "--purge" => purge = true,
            other => return Err(format!("unknown option: {other}")),
        }
    }
    for line in install::uninstall(purge)? {
        println!("{line}");
    }
    Ok(())
}

fn list_speakers() -> Result<(), String> {
    for (index, name) in playback::devices()?.iter().enumerate() {
        println!("{index}  {name}");
    }
    Ok(())
}

fn print_profile() -> Result<(), String> {
    let cores = engines::detect_cores();
    let gpu = engines::detect_gpu_memory_mb();
    let profile = engines::profile_for(cores, gpu);
    let fidelity = engines::fidelity_for(profile);
    let (min_ms, max_ms) = profile.target_ms();

    println!("hardware profile");
    println!("  cpu cores       {cores}");
    match gpu {
        Some(memory) => println!("  gpu memory      {memory} MB"),
        None => println!("  gpu memory      none detected, this is a cpu hint"),
    }
    println!("  profile         {}", profile.label());
    println!("  voice level     {fidelity}");
    if min_ms == 0 {
        println!("  latency target  under {max_ms} ms (captions)");
    } else {
        println!("  latency target  {min_ms} to {max_ms} ms");
    }
    println!(
        "  note            targets are design goals, the published table is what was measured"
    );
    Ok(())
}

fn demo(path: Option<&str>) -> Result<(), String> {
    let input = AudioFormat::mono(48_000).map_err(|e| e.to_string())?;
    let output = AudioFormat::mono(16_000).map_err(|e| e.to_string())?;
    let mut pipeline = Pipeline::new(
        PipelineConfig::new(input, output),
        Box::new(EchoEngine::new("echo", ENGINE_LATENCY_MS, 0.8)),
    )
    .map_err(|e| e.to_string())?;

    let speech = synthetic_speech(input.sample_rate(), 2_500);
    let mut translated = Vec::new();
    let mut frames = 0;
    let mut voiced = 0;

    for chunk in speech.chunks(480) {
        for out in pipeline.push(chunk) {
            frames += 1;
            if out.voice {
                voiced += 1;
            }
            if let Some(frame) = out.frame {
                translated.extend_from_slice(frame.samples());
            }
        }
    }

    println!("demo");
    println!(
        "  input           {:.2} s synthetic speech",
        speech.len() as f32 / 48_000.0
    );
    println!("  frames          {frames} processed, {voiced} voiced");
    println!(
        "  translated      {:.2} s at {} Hz",
        translated.len() as f32 / 16_000.0,
        output.sample_rate()
    );
    println!(
        "  latency         {} ms (declared by engine)",
        pipeline.budget().total_ms()
    );

    if let Some(path) = path {
        let wav = Wav {
            format: output,
            samples: translated,
        };
        fs::write(path, write_pcm16(&wav)).map_err(|e| format!("{path}: {e}"))?;
        println!("  written         {path}");
    }
    Ok(())
}

fn translate(args: &[String]) -> Result<(), String> {
    let (input_path, output_path, engine_name) = parse_translate_args(args)?;

    let bytes = fs::read(input_path).map_err(|e| format!("{input_path}: {e}"))?;
    let wav = read_pcm16(&bytes).map_err(|e| format!("{input_path}: {e}"))?;
    if wav.format.channels() != 1 {
        return Err("input must be mono pcm16 audio".to_string());
    }

    let output = AudioFormat::mono(16_000).map_err(|e| e.to_string())?;
    let mut pipeline = Pipeline::new(
        PipelineConfig::new(wav.format, output),
        build_engine(&engine_name)?,
    )
    .map_err(|e| e.to_string())?;

    let mut translated = Vec::new();
    let mut voiced = 0;
    for chunk in wav.samples.chunks(480) {
        for out in pipeline.push(chunk) {
            if let Some(error) = out.error {
                return Err(format!("engine failed: {error}"));
            }
            if out.voice {
                voiced += 1;
            }
            if let Some(frame) = out.frame {
                translated.extend_from_slice(frame.samples());
            }
        }
    }

    if translated.is_empty() {
        return Err(format!(
            "the {} engine returned text but no audio, so nothing was written to {output_path}: its voice is not available yet",
            pipeline.engine_id()
        ));
    }

    let result = Wav {
        format: output,
        samples: translated,
    };
    fs::write(output_path, write_pcm16(&result)).map_err(|e| format!("{output_path}: {e}"))?;

    println!("translated {input_path} -> {output_path}");
    println!(
        "  source          {} Hz, {:.2} s",
        wav.format.sample_rate(),
        wav.samples.len() as f32 / wav.format.sample_rate() as f32
    );
    println!(
        "  output          {} Hz, {:.2} s",
        output.sample_rate(),
        result.samples.len() as f32 / output.sample_rate() as f32
    );
    println!("  voiced frames   {voiced}");
    println!("  engine          {}", pipeline.engine_id());
    Ok(())
}

fn pair_command(args: &[String]) -> Result<(), String> {
    let mut from = "pt".to_string();
    let mut to = "de".to_string();
    let mut folder = None;
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
            other if other.starts_with("--") => return Err(format!("unknown option: {other}")),
            other => {
                if folder.is_some() {
                    return Err("pair takes one export folder".to_string());
                }
                folder = Some(other.to_string());
            }
        }
        index += 1;
    }
    let folder = folder.ok_or("usage: parlo pair <export folder> [--from pt] [--to de]")?;
    if !config::check_language(&from) {
        return Err(format!("{from} is not a language this build knows"));
    }
    if !config::check_language(&to) {
        return Err(format!("{to} is not a language this build knows"));
    }
    println!("{}", models::install_pair(&folder, &from, &to)?);
    Ok(())
}

fn stream_command(args: &[String]) -> Result<(), String> {
    let stored = config::load();
    let mut from = stored.from.clone();
    let mut to = stored.to.clone();
    let mut stt = stored.stt.clone();
    let mut chunk_ms = 100u32;
    let mut path = None;
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
            "--stt" => {
                index += 1;
                stt = args.get(index).cloned().ok_or("--stt needs a value")?;
            }
            "--chunk" => {
                index += 1;
                chunk_ms = args
                    .get(index)
                    .ok_or("--chunk needs a value")?
                    .parse()
                    .map_err(|_| "chunk must be milliseconds".to_string())?;
            }
            other if other.starts_with("--") => return Err(format!("unknown option: {other}")),
            other => {
                if path.is_some() {
                    return Err("stream takes one file".to_string());
                }
                path = Some(PathBuf::from(other));
            }
        }
        index += 1;
    }

    let path = path.ok_or(
        "usage: parlo stream <in.wav> [--from pt] [--to en] [--stt parakeet|whisper] \
         [--chunk MS]",
    )?;
    if chunk_ms < 20 {
        return Err("--chunk must be at least 20 ms".to_string());
    }
    set_direction(&from, &to)?;
    config::check_stt(&stt)?;
    env::set_var("PARLO_CASCADE_STT", &stt);

    let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let wav = read_pcm16(&bytes).map_err(|error| format!("{}: {error}", path.display()))?;
    if wav.format.channels() != 1 {
        return Err("input must be mono pcm16 audio".to_string());
    }
    let mut resampler = parlo_core::Resampler::new(wav.format.sample_rate(), 16_000);
    let mut samples = Vec::new();
    resampler.process(&wav.samples, &mut samples);

    let dir = env::var("PARLO_CASCADE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("engines/cascade"));
    let mut engine = sidecar::SidecarEngine::spawn(&dir).map_err(|error| error.to_string())?;

    let chunk = (16_000 * chunk_ms / 1000) as usize;
    let started = std::time::Instant::now();
    let mut counts = std::collections::BTreeMap::new();
    println!(
        "stream {from} to {to} with {stt}, {chunk_ms} ms chunks, {:.2} s of audio",
        samples.len() as f32 / 16_000.0
    );

    for block in samples.chunks(chunk) {
        for event in engine.stream(block).map_err(|error| error.to_string())? {
            *counts.entry(event.state.clone()).or_insert(0usize) += 1;
            println!(
                "[{:>6.3}s] {:<9} {:>6} ms audio {:>5} ms decode  {}",
                started.elapsed().as_secs_f32(),
                event.state,
                event.audio_ms,
                event.latency_ms,
                event.text
            );
        }
    }
    for event in engine.close_stream().map_err(|error| error.to_string())? {
        *counts.entry(event.state.clone()).or_insert(0usize) += 1;
        println!(
            "[{:>6.3}s] {:<9} {:>6} ms audio {:>5} ms decode  {}",
            started.elapsed().as_secs_f32(),
            event.state,
            event.audio_ms,
            event.latency_ms,
            event.text
        );
    }

    println!();
    println!("  mode         {}", engine.stream_mode());
    for (state, count) in counts {
        println!("  {state:<12} {count}");
    }
    println!("  wall         {:.2} s", started.elapsed().as_secs_f32());
    Ok(())
}

fn parse_translate_args(args: &[String]) -> Result<(&String, &String, String), String> {
    let mut engine = "echo".to_string();
    let mut paths: Vec<&String> = Vec::new();
    let mut direction = ("pt".to_string(), "en".to_string());
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--engine" {
            index += 1;
            engine = args.get(index).cloned().ok_or("--engine needs a value")?;
        } else if arg == "--from" || arg == "--to" {
            index += 1;
            let value = args
                .get(index)
                .cloned()
                .ok_or(format!("{arg} needs a value"))?;
            if arg == "--from" {
                direction.0 = value;
            } else {
                direction.1 = value;
            }
        } else if arg.starts_with("--") {
            return Err(format!("unknown option: {arg}"));
        } else {
            paths.push(arg);
        }
        index += 1;
    }
    set_direction(&direction.0, &direction.1)?;
    match paths.as_slice() {
        [input, output] => Ok((input, output, engine)),
        _ => Err(
            "usage: parlo translate <in.wav> <out.wav> [--engine echo|sidecar|cascade] [--from pt] [--to en]".to_string(),
        ),
    }
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
        keep_background,
    })?;

    println!();
    println!("dubbed {} into {}", source.display(), to);
    println!("  output      {}", result.out);
    println!("  subtitles   {}", result.srt);
    println!("  lines       {}", result.lines.len());
    Ok(())
}

fn config_command(args: &[String]) -> Result<(), String> {
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
    match config.device {
        Some(device) => println!("  device   {device}"),
        None => println!("  device   the system default"),
    }
    Ok(())
}

fn set_direction(from: &str, to: &str) -> Result<(), String> {
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
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--list" => list = true,
            "--probe" => probe = true,
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

    let mut recording = record::capture(seconds, device)?;
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
        println!("  warning          the microphone captured almost nothing, check the input device, mute and gain");
    }
    Ok(())
}

fn serve_command(args: &[String]) -> Result<(), String> {
    let stored = config::load();
    let mut port = stored.port;
    let mut token = None;
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
    serve::run(
        port,
        token.unwrap_or_else(serve::new_token),
        engine,
        stt,
        from,
        to,
    )
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

fn listen_command(args: &[String]) -> Result<(), String> {
    let stored = config::load();
    let mut path = PathBuf::from("out/translated-live.wav");
    let mut seconds = 10.0f32;
    let mut play = false;
    let mut speaker = None;
    let mut engine_name = stored.engine.clone();
    let mut stt = stored.stt.clone();
    let mut device = stored.device;
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
            "--seconds" => {
                index += 1;
                seconds = args
                    .get(index)
                    .ok_or("--seconds needs a value")?
                    .parse()
                    .map_err(|_| "seconds must be a number".to_string())?;
            }
            "--engine" => {
                index += 1;
                engine_name = args.get(index).cloned().ok_or("--engine needs a value")?;
            }
            "--stt" => {
                index += 1;
                stt = args.get(index).cloned().ok_or("--stt needs a value")?;
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
            "--play" => play = true,
            "--speaker" => {
                index += 1;
                speaker = Some(
                    args.get(index)
                        .ok_or("--speaker needs a value")?
                        .parse()
                        .map_err(|_| "speaker must be a number".to_string())?,
                );
                play = true;
            }
            other if other.starts_with("--") => return Err(format!("unknown option: {other}")),
            other => path = PathBuf::from(other),
        }
        index += 1;
    }

    set_direction(&from, &to)?;
    config::check_stt(&stt)?;
    env::set_var("PARLO_CASCADE_STT", &stt);
    let (microphone, input) = record::open_input(device)?;
    let output = AudioFormat::mono(16_000).map_err(|error| error.to_string())?;
    let mut pipeline = Pipeline::new(
        PipelineConfig::new(input, output),
        build_engine(&engine_name)?,
    )
    .map_err(|error| error.to_string())?;
    let mut speaker = match (play, speaker) {
        (true, index) => Some(playback::Speaker::open(index)?),
        (false, _) => None,
    };

    let mut translated = Vec::new();
    let mut voiced = 0usize;
    let mut transcripts = 0usize;
    let mut first_audio = None;
    let mut first_error = None;
    let started = std::time::Instant::now();

    record::chunks(microphone, seconds, |block| {
        for out in pipeline.push(&block) {
            if out.voice {
                voiced += 1;
            }
            if let Some(text) = out.transcript {
                transcripts += 1;
                println!("  translation      {text}");
            }
            if let Some(error) = out.error {
                first_error.get_or_insert(error);
            }
            if let Some(frame) = out.frame {
                first_audio.get_or_insert_with(|| started.elapsed().as_secs_f32());
                if let Some(speaker) = speaker.as_mut() {
                    speaker.push(frame.samples());
                }
                translated.extend_from_slice(frame.samples());
            }
        }
        true
    })?;

    let wall = started.elapsed().as_secs_f32();
    let audio = translated.len() as f32 / output.sample_rate() as f32;
    let wav = Wav {
        format: output,
        samples: translated,
    };
    fs::write(&path, write_pcm16(&wav)).map_err(|error| format!("{}: {error}", path.display()))?;

    println!("listened {wall:.1} s at {} Hz", input.sample_rate());
    println!("  voiced frames    {voiced}");
    let gain = pipeline.input_gain();
    if gain > 1.01 {
        println!("  input gain       {:+.1} dB", 20.0 * gain.log10());
    }
    println!("  translations     {transcripts}");
    println!("  translated audio {audio:.2} s");
    match first_audio {
        Some(seconds) => println!("  first audio      {seconds:.2} s after start"),
        None => println!("  first audio      never, nothing passed the voice gate"),
    }
    if let Some(speaker) = speaker.as_ref() {
        println!("  speaker          {} Hz", speaker.rate());
        println!("  still queued     {:.2} s", speaker.buffered_seconds());
    }
    if let Some(error) = first_error {
        println!("  engine error     {error}");
    }
    println!("  written          {}", path.display());
    Ok(())
}

fn captions_command(args: &[String]) -> Result<(), String> {
    let stored = config::load();
    let mut seconds = 10.0f32;
    let mut window = 1.2f32;
    let mut engine_name = stored.engine.clone();
    let mut stt = stored.stt.clone();
    let mut device = stored.device;
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
            "--seconds" => {
                index += 1;
                seconds = args
                    .get(index)
                    .ok_or("--seconds needs a value")?
                    .parse()
                    .map_err(|_| "seconds must be a number".to_string())?;
            }
            "--window" => {
                index += 1;
                window = args
                    .get(index)
                    .ok_or("--window needs a value")?
                    .parse()
                    .map_err(|_| "window must be a number of seconds".to_string())?;
            }
            "--engine" => {
                index += 1;
                engine_name = args.get(index).cloned().ok_or("--engine needs a value")?;
            }
            "--stt" => {
                index += 1;
                stt = args.get(index).cloned().ok_or("--stt needs a value")?;
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
            other => return Err(format!("unknown option: {other}")),
        }
        index += 1;
    }

    if window < 0.5 {
        return Err("--window must be at least half a second".to_string());
    }
    set_direction(&from, &to)?;
    config::check_stt(&stt)?;
    env::set_var("PARLO_CASCADE_STT", &stt);
    env::set_var("PARLO_CASCADE_CAPTIONS", "1");
    env::set_var("PARLO_CASCADE_SEGMENT", format!("{window}"));

    let (microphone, input) = record::open_input(device)?;
    let output = AudioFormat::mono(16_000).map_err(|error| error.to_string())?;
    let mut pipeline = Pipeline::new(
        PipelineConfig::new(input, output),
        build_engine(&engine_name)?,
    )
    .map_err(|error| error.to_string())?;

    println!("captions {from} to {to}, no audio is spoken, control c stops");
    let started = std::time::Instant::now();
    let mut captions = 0usize;
    let mut first_caption = None;
    let mut first_error = None;

    record::chunks(microphone, seconds, |block| {
        for out in pipeline.push(&block) {
            if let Some(text) = out.transcript {
                captions += 1;
                first_caption.get_or_insert_with(|| started.elapsed().as_secs_f32());
                println!("[{:>6.1}s] {text}", started.elapsed().as_secs_f32());
            }
            if let Some(error) = out.error {
                first_error.get_or_insert(error);
            }
        }
        true
    })?;

    println!("listened {:.1} s", started.elapsed().as_secs_f32());
    println!("  captions         {captions}");
    match first_caption {
        Some(at) => println!("  first caption    {at:.2} s after start"),
        None => println!("  first caption    never, nothing passed the voice gate"),
    }
    if let Some(error) = first_error {
        println!("  engine error     {error}");
    }
    Ok(())
}

fn dictate_command(args: &[String]) -> Result<(), String> {
    let stored = config::load();
    let mut seconds = 60.0f32;
    let mut window = 2.0f32;
    let mut engine_name = stored.engine.clone();
    let mut stt = stored.stt.clone();
    let mut device = stored.device;
    let mut from = stored.from.clone();
    let mut copy_text = true;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--from" => {
                index += 1;
                from = args.get(index).cloned().ok_or("--from needs a value")?;
            }
            "--seconds" => {
                index += 1;
                seconds = args
                    .get(index)
                    .ok_or("--seconds needs a value")?
                    .parse()
                    .map_err(|_| "seconds must be a number".to_string())?;
            }
            "--window" => {
                index += 1;
                window = args
                    .get(index)
                    .ok_or("--window needs a value")?
                    .parse()
                    .map_err(|_| "window must be a number of seconds".to_string())?;
            }
            "--engine" => {
                index += 1;
                engine_name = args.get(index).cloned().ok_or("--engine needs a value")?;
            }
            "--stt" => {
                index += 1;
                stt = args.get(index).cloned().ok_or("--stt needs a value")?;
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
            "--no-copy" => copy_text = false,
            other => return Err(format!("unknown option: {other}")),
        }
        index += 1;
    }

    if window < 0.5 {
        return Err("--window must be at least half a second".to_string());
    }
    if !matches!(from.as_str(), "pt" | "en") {
        return Err(format!("cannot dictate in {from}, expected pt or en"));
    }
    config::check_stt(&stt)?;
    env::set_var("PARLO_CASCADE_SOURCE", &from);
    env::set_var("PARLO_CASCADE_TARGET", &from);
    env::set_var("PARLO_CASCADE_STT", &stt);
    env::set_var("PARLO_CASCADE_DICTATE", "1");
    env::set_var("PARLO_CASCADE_SEGMENT", format!("{window}"));

    let (microphone, input) = record::open_input(device)?;
    let output = AudioFormat::mono(16_000).map_err(|error| error.to_string())?;
    let mut pipeline = Pipeline::new(
        PipelineConfig::new(input, output),
        build_engine(&engine_name)?,
    )
    .map_err(|error| error.to_string())?;

    eprintln!("dictating in {from} with {stt}, {window:.1} s windows, control c stops");
    let started = std::time::Instant::now();
    let mut segments: Vec<String> = Vec::new();
    let mut first_error = None;

    record::chunks(microphone, seconds, |block| {
        for out in pipeline.push(&block) {
            if let Some(text) = out.transcript {
                eprintln!("  {:>4.1} s  {text}", started.elapsed().as_secs_f32());
                segments.push(text);
            }
            if let Some(error) = out.error {
                first_error.get_or_insert(error);
            }
        }
        true
    })?;

    eprintln!();
    let text = dictate::join(&segments);
    if text.is_empty() {
        if let Some(error) = first_error {
            return Err(error);
        }
        return Err("nothing was recognised, check the microphone with parlo doctor".to_string());
    }

    println!("{text}");
    eprintln!("  listened   {:>5.1} s", started.elapsed().as_secs_f32());
    eprintln!("  segments   {:>5}", segments.len());
    if copy_text {
        match dictate::copy(&text) {
            Ok(tool) => eprintln!("  clipboard  copied with {tool}"),
            Err(error) => eprintln!("  clipboard  not copied: {error}"),
        }
    }
    Ok(())
}

fn mic_command(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("setup") => mic_setup(),
        Some("status") => mic_status(),
        Some("remove") => mic_remove(),
        _ => Err("usage: parlo mic <setup|status|remove>".to_string()),
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

fn mic_setup() -> Result<(), String> {
    require_linux()?;
    let listing = mic::list_command().run()?;
    if let Some(id) = mic::find_module(&listing) {
        println!("Parlo microphone already loaded as module {id}");
        return Ok(());
    }
    let module = mic::setup_command().run()?;
    println!("Parlo microphone ready (module {})", module.trim());
    println!("select the Parlo monitor as the microphone in your calling app");
    Ok(())
}

fn mic_status() -> Result<(), String> {
    require_linux()?;
    let listing = mic::list_command().run()?;
    match mic::find_module(&listing) {
        Some(id) => println!("Parlo microphone active (module {id})"),
        None => println!("Parlo microphone not loaded, run: parlo mic setup"),
    }
    Ok(())
}

fn mic_remove() -> Result<(), String> {
    require_linux()?;
    let listing = mic::list_command().run()?;
    let id = mic::find_module(&listing).ok_or("Parlo microphone is not loaded")?;
    mic::remove_command(id).run()?;
    println!("removed the Parlo microphone (module {id})");
    Ok(())
}

fn synthetic_speech(sample_rate: u32, duration_ms: u32) -> Vec<f32> {
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
