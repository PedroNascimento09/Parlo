use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parlo_core::AudioFormat;

pub struct Recording {
    pub format: AudioFormat,
    pub samples: Vec<f32>,
}

pub fn devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host.input_devices().map_err(|error| error.to_string())?;
    Ok(devices
        .map(|device| device.name().unwrap_or_else(|_| "unnamed".to_string()))
        .collect())
}

pub fn capture(seconds: f32, device_index: Option<usize>) -> Result<Recording, String> {
    let host = cpal::default_host();
    let device = match device_index {
        Some(index) => host
            .input_devices()
            .map_err(|error| error.to_string())?
            .nth(index)
            .ok_or_else(|| format!("input device {index} does not exist"))?,
        None => host
            .default_input_device()
            .ok_or_else(|| "no input device available".to_string())?,
    };

    let supported = device
        .default_input_config()
        .map_err(|error| format!("cannot read the input format: {error}"))?;
    let config: cpal::StreamConfig = supported.config();
    let rate = config.sample_rate.0;
    let seconds = seconds.max(0.5);
    let buffer = Arc::new(Mutex::new(Vec::with_capacity(
        rate as usize * seconds as usize,
    )));

    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, buffer.clone())?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, buffer.clone())?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, buffer.clone())?,
        cpal::SampleFormat::I32 => build_stream::<i32>(&device, &config, buffer.clone())?,
        other => return Err(format!("unsupported sample format: {other}")),
    };

    stream
        .play()
        .map_err(|error| format!("cannot start capture: {error}"))?;
    std::thread::sleep(Duration::from_secs_f32(seconds));
    drop(stream);

    let samples = buffer
        .lock()
        .map(|guard| guard.clone())
        .map_err(|_| "capture buffer was poisoned".to_string())?;
    if samples.is_empty() {
        return Err("the microphone returned no audio".to_string());
    }

    Ok(Recording {
        format: AudioFormat::mono(rate).map_err(|error| error.to_string())?,
        samples,
    })
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream, String>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    device
        .build_input_stream(
            config,
            move |input: &[T], _: &cpal::InputCallbackInfo| {
                if let Ok(mut samples) = buffer.lock() {
                    for frame in input.chunks(channels) {
                        let sum: f32 = frame
                            .iter()
                            .map(|sample| f32::from_sample(*sample))
                            .sum();
                        samples.push(sum / frame.len() as f32);
                    }
                }
            },
            |error| eprintln!("audio input error: {error}"),
            None,
        )
        .map_err(|error| format!("cannot open the input stream: {error}"))
}
