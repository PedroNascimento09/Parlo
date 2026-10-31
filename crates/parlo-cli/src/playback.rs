use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::FromSample;
use parlo_core::resample::Resampler;

const MAX_BUFFER_SECONDS: f32 = 20.0;

pub fn devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host.output_devices().map_err(|error| error.to_string())?;
    Ok(devices
        .map(|device| device.name().unwrap_or_else(|_| "unnamed".to_string()))
        .collect())
}

type Queue = Arc<Mutex<VecDeque<f32>>>;

pub struct Speaker {
    _stream: cpal::Stream,
    queue: Queue,
    resampler: Resampler,
    rate: u32,
}

impl Speaker {
    pub fn open(device_index: Option<usize>) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = match device_index {
            Some(index) => host
                .output_devices()
                .map_err(|error| error.to_string())?
                .nth(index)
                .ok_or_else(|| format!("output device {index} does not exist"))?,
            None => host
                .default_output_device()
                .ok_or_else(|| "no output device available".to_string())?,
        };
        let supported = device
            .default_output_config()
            .map_err(|error| format!("cannot read the output format: {error}"))?;
        let config: cpal::StreamConfig = supported.config();
        let rate = config.sample_rate.0;
        let queue: Queue = Arc::new(Mutex::new(VecDeque::new()));

        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => build::<f32>(&device, &config, queue.clone())?,
            cpal::SampleFormat::I16 => build::<i16>(&device, &config, queue.clone())?,
            cpal::SampleFormat::U16 => build::<u16>(&device, &config, queue.clone())?,
            other => return Err(format!("unsupported output sample format: {other}")),
        };
        stream
            .play()
            .map_err(|error| format!("cannot start playback: {error}"))?;

        Ok(Self {
            _stream: stream,
            queue,
            resampler: Resampler::new(16_000, rate),
            rate,
        })
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn push(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        let mut translated = Vec::with_capacity(samples.len() * 2);
        self.resampler.process(samples, &mut translated);
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        let limit = (self.rate as f32 * MAX_BUFFER_SECONDS) as usize;
        for sample in translated {
            queue.push_back(sample);
        }
        while queue.len() > limit {
            queue.pop_front();
        }
    }

    pub fn buffered_seconds(&self) -> f32 {
        self.queue
            .lock()
            .map(|queue| queue.len() as f32 / self.rate as f32)
            .unwrap_or(0.0)
    }
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    queue: Queue,
) -> Result<cpal::Stream, String>
where
    T: cpal::SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    device
        .build_output_stream(
            config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                let mut guard = queue.lock().ok();
                for frame in output.chunks_mut(channels) {
                    let value = guard
                        .as_mut()
                        .and_then(|queue| queue.pop_front())
                        .unwrap_or(0.0);
                    let sample = T::from_sample(value);
                    for output in frame {
                        *output = sample;
                    }
                }
            },
            move |error| eprintln!("audio output error: {error}"),
            None,
        )
        .map_err(|error| format!("cannot open the output stream: {error}"))
}
