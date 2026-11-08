use std::fs;
use std::path::PathBuf;

use parlo_core::wav::write_pcm16;
use parlo_core::{AudioFormat, EchoEngine, Pipeline, PipelineConfig, Wav};

use crate::{bench, engines, install, playback, synthetic_speech, ENGINE_LATENCY_MS};

pub(crate) fn list_engines() -> Result<(), String> {
    println!("engines");
    let licence_width = engines::ENGINES
        .iter()
        .map(|engine| engine.license.label().len())
        .max()
        .unwrap_or(0);
    for engine in engines::ENGINES.iter() {
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

pub(crate) fn install(args: &[String]) -> Result<(), String> {
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

pub(crate) fn uninstall(args: &[String]) -> Result<(), String> {
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

pub(crate) fn list_speakers() -> Result<(), String> {
    for (index, name) in playback::devices()?.iter().enumerate() {
        println!("{index}  {name}");
    }
    Ok(())
}

pub(crate) fn print_profile() -> Result<(), String> {
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

pub(crate) fn bench(args: &[String]) -> Result<(), String> {
    if args.first().map(String::as_str) == Some("dictate") {
        return bench::dictate(&args[1..]);
    }
    if args.iter().any(|argument| argument == "--json") {
        println!("{}", bench::as_json());
    } else {
        print!("{}", bench::as_markdown());
    }
    Ok(())
}

pub(crate) fn demo(path: Option<&str>) -> Result<(), String> {
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
