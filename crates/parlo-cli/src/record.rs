use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::FromSample;
use parlo_core::gain::peak;
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

pub fn probe(seconds: f32) -> Result<Vec<(usize, String, f32)>, String> {
    let names = devices()?;
    let mut results = Vec::new();
    for (index, name) in names.into_iter().enumerate() {
        let level = capture(seconds, Some(index))
            .map(|recording| peak(&recording.samples))
            .unwrap_or(0.0);
        results.push((index, name, level));
    }
    Ok(results)
}

pub fn open_input(device_index: Option<usize>) -> Result<(cpal::Device, AudioFormat), String> {
    let (device, supported) = open(device_index)?;
    let format = AudioFormat::mono(supported.sample_rate().0).map_err(|error| error.to_string())?;
    Ok((device, format))
}

pub fn capture(seconds: f32, device_index: Option<usize>) -> Result<Recording, String> {
    let (device, supported) = open(device_index)?;
    let config: cpal::StreamConfig = supported.config();
    let rate = config.sample_rate.0;
    let seconds = seconds.max(0.5);
    let buffer = Arc::new(Mutex::new(Vec::with_capacity(
        rate as usize * seconds as usize,
    )));

    let failure: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => {
            build_stream::<f32>(&device, &config, buffer.clone(), failure.clone())?
        }
        cpal::SampleFormat::I16 => {
            build_stream::<i16>(&device, &config, buffer.clone(), failure.clone())?
        }
        cpal::SampleFormat::U16 => {
            build_stream::<u16>(&device, &config, buffer.clone(), failure.clone())?
        }
        cpal::SampleFormat::I32 => {
            build_stream::<i32>(&device, &config, buffer.clone(), failure.clone())?
        }
        other => return Err(format!("unsupported sample format: {other}")),
    };

    stream
        .play()
        .map_err(|error| format!("cannot start capture: {error}"))?;
    std::thread::sleep(Duration::from_secs_f32(seconds));
    drop(stream);

    if let Some(message) = failure.lock().ok().and_then(|guard| guard.clone()) {
        return Err(format!("the input device stopped: {message}"));
    }

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

pub fn chunks<F>(device: cpal::Device, seconds: f32, mut on_chunk: F) -> Result<(), String>
where
    F: FnMut(Vec<f32>) -> bool,
{
    let supported = device
        .default_input_config()
        .map_err(|error| format!("cannot read the input format: {error}"))?;
    let config: cpal::StreamConfig = supported.config();
    let (sender, receiver) = mpsc::channel();
    let (failure, failed) = mpsc::channel();

    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => build_block_stream::<f32>(&device, &config, sender, failure)?,
        cpal::SampleFormat::I16 => build_block_stream::<i16>(&device, &config, sender, failure)?,
        cpal::SampleFormat::U16 => build_block_stream::<u16>(&device, &config, sender, failure)?,
        cpal::SampleFormat::I32 => build_block_stream::<i32>(&device, &config, sender, failure)?,
        other => return Err(format!("unsupported sample format: {other}")),
    };

    stream
        .play()
        .map_err(|error| format!("cannot start capture: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs_f32(seconds.max(0.5));

    while Instant::now() < deadline {
        if let Ok(message) = failed.try_recv() {
            drop(stream);
            return Err(format!("the input device stopped: {message}"));
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(block) => {
                if !on_chunk(block) {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    drop(stream);
    Ok(())
}

fn open(
    device_index: Option<usize>,
) -> Result<(cpal::Device, cpal::SupportedStreamConfig), String> {
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
    Ok((device, supported))
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
    failure: Arc<Mutex<Option<String>>>,
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
                        let sum: f32 = frame.iter().map(|sample| f32::from_sample_(*sample)).sum();
                        samples.push(sum / frame.len() as f32);
                    }
                }
            },
            move |error| {
                eprintln!("audio input error: {error}");
                if let Ok(mut slot) = failure.lock() {
                    slot.get_or_insert_with(|| error.to_string());
                }
            },
            None,
        )
        .map_err(|error| format!("cannot open the input stream: {error}"))
}

fn build_block_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sender: mpsc::Sender<Vec<f32>>,
    failure: mpsc::Sender<String>,
) -> Result<cpal::Stream, String>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    let block = (config.sample_rate.0 as usize / 50).max(1);
    let mut buffer = Vec::with_capacity(block);
    device
        .build_input_stream(
            config,
            move |input: &[T], _: &cpal::InputCallbackInfo| {
                for frame in input.chunks(channels) {
                    let sum: f32 = frame.iter().map(|sample| f32::from_sample_(*sample)).sum();
                    buffer.push(sum / frame.len() as f32);
                    if buffer.len() >= block && sender.send(std::mem::take(&mut buffer)).is_err() {
                        return;
                    }
                }
            },
            move |error| {
                eprintln!("audio input error: {error}");
                let _ = failure.send(error.to_string());
            },
            None,
        )
        .map_err(|error| format!("cannot open the input stream: {error}"))
}

#[cfg(test)]
mod capture_tests {
    use super::*;

    fn default_device_and_config() -> Option<(cpal::Device, cpal::SupportedStreamConfig)> {
        let host = cpal::default_host();
        let device = host.default_input_device()?;
        let supported = device.default_input_config().ok()?;
        Some((device, supported))
    }

    #[test]
    fn devices_lists_what_the_system_has() {
        let devices = devices().unwrap_or_default();
        for name in devices {
            assert!(!name.is_empty(), "a device name must not be blank");
        }
    }

    #[test]
    fn default_input_can_be_opened_when_present() {
        if let Some((device, _supported)) = default_device_and_config() {
            let index = device_index_of(&device);
            let result = if let Some(index) = index {
                open_input(Some(index))
            } else {
                open_input(None)
            };
            match result {
                Ok((_device, format)) => {
                    assert!(format.sample_rate() > 0);
                }
                Err(error) => panic!("could not open the default input device: {error}"),
            }
        } else {
            let host_reports_a_device = cpal::default_host().default_input_device().is_some();
            match open_input(None) {
                Ok(_) => panic!("expected no input device to fail"),
                Err(error) if host_reports_a_device => {
                    assert!(!error.is_empty(), "a capture failure must carry a reason")
                }
                Err(error) => assert!(
                    error.contains("no input device"),
                    "expected a no-device error, got: {error}"
                ),
            }
        }
    }

    #[test]
    fn short_capture_does_not_hang() {
        if let Some((device, _supported)) = default_device_and_config() {
            let index = device_index_of(&device);
            let result = capture(0.5, index);
            match result {
                Ok(recording) => {
                    assert!(recording.format.sample_rate() > 0);
                }
                Err(error) => {
                    assert!(!error.is_empty(), "a capture failure must carry a reason");
                }
            }
        }
    }

    #[test]
    fn chunks_deliver_blocks_while_stream_is_open() {
        if let Some((device, _supported)) = default_device_and_config() {
            let mut received = 0;
            let result = chunks(device, 0.5, |block| {
                received += 1;
                !block.is_empty()
            });
            match result {
                Ok(()) => assert!(received > 0, "chunks stream delivered nothing"),
                Err(error) => {
                    assert!(!error.is_empty(), "a chunks failure must carry a reason");
                }
            }
        }
    }

    #[test]
    fn capture_with_zero_seconds_returns_quickly() {
        if let Some((device, _supported)) = default_device_and_config() {
            let index = device_index_of(&device);
            let result = capture(0.0, index);
            if let Ok(recording) = result {
                assert!(recording.format.sample_rate() > 0);
            }
        }
    }

    fn device_index_of(device: &cpal::Device) -> Option<usize> {
        let host = cpal::default_host();
        let target = device.name().unwrap_or_else(|_| String::new());
        for (index, candidate) in host.input_devices().ok()?.enumerate() {
            if candidate.name().unwrap_or_else(|_| String::new()) == target {
                return Some(index);
            }
        }
        None
    }
}
