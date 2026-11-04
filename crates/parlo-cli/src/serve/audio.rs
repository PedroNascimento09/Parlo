use parlo_core::resample::Resampler;
use parlo_core::wav::read_pcm16;

pub(crate) fn decode(data: &[u8]) -> Result<(Vec<u8>, f32), String> {
    let wav =
        read_pcm16(data).map_err(|error| format!("the file is not a 16-bit PCM wav: {error}"))?;
    if wav.samples.is_empty() {
        return Err("the file has no audio".to_string());
    }
    let channels = wav.format.channels() as usize;
    if channels == 0 {
        return Err("the file has no channels".to_string());
    }
    let seconds = wav.samples.len() as f32 / channels as f32 / wav.format.sample_rate() as f32;
    let mut samples: Vec<f32> = wav
        .samples
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect();
    if wav.format.sample_rate() != 16_000 {
        let mut resampler = Resampler::new(wav.format.sample_rate(), 16_000);
        let mut resampled = Vec::new();
        resampler.process(&samples, &mut resampled);
        samples = resampled;
    }
    let mut pcm16 = Vec::with_capacity(samples.len() * 2);
    for sample in &samples {
        pcm16
            .extend_from_slice(&((sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes());
    }
    Ok((pcm16, seconds))
}
