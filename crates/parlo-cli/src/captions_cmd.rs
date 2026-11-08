use std::env;

use parlo_core::{AudioFormat, Pipeline, PipelineConfig};

use crate::{build_engine, config, dictate, insert, record, set_direction};

pub(crate) fn captions(args: &[String]) -> Result<(), String> {
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

pub(crate) fn dictate(args: &[String]) -> Result<(), String> {
    let stored = config::load();
    let mut seconds = 60.0f32;
    let mut window = 2.0f32;
    let mut engine_name = stored.engine.clone();
    let mut stt = stored.stt.clone();
    let mut device = stored.device;
    let mut from = stored.from.clone();
    let mut copy_text = true;
    let mut insert_text = false;
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
            "--insert" => insert_text = true,
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
    if insert_text {
        let (target, _app) = insert::foreground()?;
        match insert::type_text(target, &text) {
            Ok(()) => eprintln!("  inserted   typed into the focused window"),
            Err(error) => eprintln!("  inserted   not typed: {error}"),
        }
    } else if copy_text {
        match dictate::copy(&text) {
            Ok(tool) => eprintln!("  clipboard  copied with {tool}"),
            Err(error) => eprintln!("  clipboard  not copied: {error}"),
        }
    }
    Ok(())
}
