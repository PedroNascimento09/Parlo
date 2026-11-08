use std::env;
use std::fs;
use std::path::PathBuf;

use parlo_core::wav::{read_pcm16, write_pcm16};
use parlo_core::{AudioFormat, Pipeline, PipelineConfig, Wav};

use crate::config;
use crate::models;
use crate::{build_offline_engine, set_direction};

pub(crate) fn translate(args: &[String]) -> Result<(), String> {
    let (input_path, output_path, engine_name) = parse_translate_args(args)?;

    let bytes = fs::read(input_path).map_err(|e| format!("{input_path}: {e}"))?;
    let wav = read_pcm16(&bytes).map_err(|e| format!("{input_path}: {e}"))?;
    if wav.format.channels() != 1 {
        return Err("input must be mono pcm16 audio".to_string());
    }

    let output = AudioFormat::mono(16_000).map_err(|e| e.to_string())?;
    let mut pipeline = Pipeline::new(
        PipelineConfig::new(wav.format, output),
        build_offline_engine(&engine_name)?,
    )
    .map_err(|e| e.to_string())?;

    let mut translated = Vec::new();
    let mut voiced = 0;
    for out in pipeline.translate_file(&wav.samples) {
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

pub(crate) fn pair(args: &[String]) -> Result<(), String> {
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

pub(crate) fn stream(args: &[String]) -> Result<(), String> {
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
    let mut engine =
        crate::sidecar::SidecarEngine::spawn(&dir).map_err(|error| error.to_string())?;

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
