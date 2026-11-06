#[cfg(windows)]
use parlo_core::AudioFormat;

use crate::record::Recording;

#[cfg(windows)]
const RATE: u32 = 16_000;

#[cfg(windows)]
pub fn capture(seconds: f32) -> Result<Recording, String> {
    use std::time::{Duration, Instant};

    use wasapi::{initialize_mta, DeviceEnumerator, Direction, SampleType, StreamMode, WaveFormat};

    initialize_mta().ok().map_err(|error| error.to_string())?;
    let enumerator = DeviceEnumerator::new().map_err(|error| error.to_string())?;
    let device = enumerator
        .get_default_device(&Direction::Render)
        .map_err(|error| format!("no speakers to capture: {error}"))?;
    let mut client = device
        .get_iaudioclient()
        .map_err(|error| error.to_string())?;

    let desired = WaveFormat::new(32, 32, &SampleType::Float, RATE as usize, 1, None);
    let mode = StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns: 200_000,
    };
    client
        .initialize_client(&desired, &Direction::Capture, &mode)
        .map_err(|error| format!("cannot capture the speakers: {error}"))?;

    let event = client
        .set_get_eventhandle()
        .map_err(|error| error.to_string())?;
    let capture = client
        .get_audiocaptureclient()
        .map_err(|error| error.to_string())?;
    client
        .start_stream()
        .map_err(|error| format!("cannot start capture: {error}"))?;

    let seconds = seconds.max(0.5);
    let deadline = Instant::now() + Duration::from_secs_f32(seconds);
    let mut samples: Vec<f32> = Vec::with_capacity((RATE as f32 * seconds) as usize);
    let mut buffer = vec![0u8; 4 * 4096];

    while Instant::now() < deadline {
        if event.wait_for_event(100_000).is_err() {
            continue;
        }
        loop {
            let frames = capture
                .get_next_packet_size()
                .map_err(|error| error.to_string())?
                .unwrap_or(0) as usize;
            if frames == 0 {
                break;
            }
            let needed = frames * 4;
            if buffer.len() < needed {
                buffer.resize(needed, 0);
            }
            let (read, _) = capture
                .read_from_device(&mut buffer)
                .map_err(|error| error.to_string())?;
            for chunk in buffer[..read as usize * 4].chunks_exact(4) {
                samples.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
        }
    }

    client
        .stop_stream()
        .map_err(|error| format!("cannot stop capture: {error}"))?;
    samples.truncate((RATE as f32 * seconds) as usize);

    Ok(Recording {
        format: AudioFormat::mono(RATE).map_err(|error| error.to_string())?,
        samples,
    })
}

#[cfg(not(windows))]
pub fn capture(_seconds: f32) -> Result<Recording, String> {
    Err("capturing what the computer plays is only available on Windows".to_string())
}
