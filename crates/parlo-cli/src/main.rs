use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use parlo_core::wav::{read_pcm16, write_pcm16};
use parlo_core::{AudioFormat, EchoEngine, Engine, LatencyBudget, Pipeline, PipelineConfig, Wav};

mod mic;
mod sidecar;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const ENGINE_LATENCY_MS: u32 = 1_500;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let outcome = match args.first().map(String::as_str) {
        Some("doctor") => doctor(),
        Some("demo") => demo(args.get(1).map(String::as_str)),
        Some("translate") => translate(&args[1..]),
        Some("mic") => mic_command(&args[1..]),
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
    println!("  translate <in.wav> <out.wav>       translate a mono pcm16 wave file");
    println!("            [--engine echo|sidecar]  pick the engine, default echo");
    println!("  mic setup|status|remove            manage the virtual microphone (linux)");
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

fn parse_translate_args(args: &[String]) -> Result<(&String, &String, String), String> {
    let mut engine = "echo".to_string();
    let mut paths: Vec<&String> = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--engine" {
            index += 1;
            engine = args.get(index).cloned().ok_or("--engine needs a value")?;
        } else if arg.starts_with("--") {
            return Err(format!("unknown option: {arg}"));
        } else {
            paths.push(arg);
        }
        index += 1;
    }
    match paths.as_slice() {
        [input, output] => Ok((input, output, engine)),
        _ => Err("usage: parlo translate <in.wav> <out.wav> [--engine echo|sidecar]".to_string()),
    }
}

fn build_engine(name: &str) -> Result<Box<dyn Engine>, String> {
    match name {
        "echo" => Ok(Box::new(EchoEngine::new("echo", ENGINE_LATENCY_MS, 1.0))),
        "sidecar" => {
            let dir = env::var("PARLO_HIBIKI_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("engines/hibiki"));
            let engine = sidecar::SidecarEngine::spawn(&dir).map_err(|error| error.to_string())?;
            Ok(Box::new(engine))
        }
        other => Err(format!("unknown engine: {other}, expected echo or sidecar")),
    }
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
