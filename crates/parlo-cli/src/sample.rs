const FRAME_SECONDS: f64 = 0.02;
const SILENT_PEAK: f32 = 0.002;
const MIN_RMS: f32 = 0.01;
const SPEECH_PEAK: f32 = 0.02;
const MIN_SPEECH: f64 = 0.35;
const CLIPPED_PEAK: f32 = 0.995;
const MAX_CLIPPED: f64 = 0.01;

pub fn problem(samples: &[f32], seconds: f64) -> Option<String> {
    let peak = samples
        .iter()
        .fold(0f32, |highest, sample| highest.max(sample.abs()));
    if peak < SILENT_PEAK {
        return Some(
            "the sample is silent; speak into the microphone and record again".to_string(),
        );
    }

    let frames = ((seconds / FRAME_SECONDS).round() as usize).max(1);
    let width = (samples.len() / frames).max(1);
    let mut speech = 0usize;
    let mut frames_seen = 0usize;
    let mut clipped = 0usize;
    let mut energy = 0f64;
    let mut counted = 0usize;

    for chunk in samples.chunks(width) {
        let loudest = chunk
            .iter()
            .fold(0f32, |highest, sample| highest.max(sample.abs()));
        if loudest >= SPEECH_PEAK {
            speech += 1;
        }
        frames_seen += 1;
        clipped += chunk
            .iter()
            .filter(|sample| sample.abs() >= CLIPPED_PEAK)
            .count();
        energy += chunk
            .iter()
            .map(|sample| f64::from(*sample) * f64::from(*sample))
            .sum::<f64>();
        counted += chunk.len();
    }

    let clipped_share = clipped as f64 / counted as f64;
    if clipped_share > MAX_CLIPPED {
        return Some(format!(
            "{:.1}% of the sample is clipped; lower the microphone gain and record again",
            clipped_share * 100.0
        ));
    }

    let rms = (energy / counted as f64).sqrt();
    if rms < f64::from(MIN_RMS) {
        return Some(format!(
            "the sample is too quiet at {:.0} dBFS; move closer to the microphone and record again",
            20.0 * rms.log10()
        ));
    }

    let speech_share = speech as f64 / frames_seen as f64;
    if speech_share < MIN_SPEECH {
        return Some(format!(
            "only {:.0}% of the sample carries speech; fill the ten seconds with talking",
            speech_share * 100.0
        ));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: usize = 16_000;

    fn tone(seconds: f64, level: f32) -> Vec<f32> {
        vec![level; (seconds * RATE as f64) as usize]
    }

    #[test]
    fn silence_is_refused() {
        let problem = problem(&tone(10.0, 0.0), 10.0).expect("a refusal");
        assert!(problem.contains("silent"), "{problem}");
    }

    #[test]
    fn a_quiet_recording_names_the_level() {
        let problem = problem(&tone(10.0, 0.004), 10.0).expect("a refusal");
        assert!(problem.contains("too quiet"), "{problem}");
        assert!(problem.contains("dBFS"), "{problem}");
    }

    #[test]
    fn a_clipped_recording_names_the_share() {
        let problem = problem(&tone(10.0, 1.0), 10.0).expect("a refusal");
        assert!(problem.contains("clipped"), "{problem}");
    }

    #[test]
    fn a_recording_that_is_mostly_pauses_is_refused() {
        let mut samples = tone(10.0, 0.0);
        let speech = (1.0 * RATE as f64) as usize;
        for sample in samples.iter_mut().take(speech) {
            *sample = 0.2;
        }
        let problem = problem(&samples, 10.0).expect("a refusal");
        assert!(problem.contains("carries speech"), "{problem}");
    }

    #[test]
    fn ten_seconds_of_speech_pass() {
        assert_eq!(problem(&tone(10.0, 0.2), 10.0), None);
    }

    #[test]
    fn the_frame_width_follows_the_duration() {
        let mut samples = tone(20.0, 0.2);
        let quiet = samples.len() * 9 / 10;
        for sample in samples.iter_mut().take(quiet) {
            *sample = 0.0;
        }
        let problem = problem(&samples, 20.0).expect("a refusal");
        assert!(problem.contains("carries speech"), "{problem}");
    }
}
