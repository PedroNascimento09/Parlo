use std::env;
use std::fs;
use std::path::PathBuf;

use parlo_core::wav::write_pcm16;
use parlo_core::{AudioFormat, Pipeline, PipelineConfig, Wav};

use crate::config;
use crate::playback;
use crate::record;
use crate::{build_engine, set_direction};

pub(crate) fn command(args: &[String]) -> Result<(), String> {
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
