const TARGET_PEAK: f32 = 0.89;
const MAX_BOOST_DB: f32 = 24.0;

pub fn peak(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0f32, |highest, sample| highest.max(sample.abs()))
}

pub fn normalize(samples: &mut [f32]) -> f32 {
    let current = peak(samples);
    if current <= 0.0 {
        return 1.0;
    }
    let ceiling = 10f32.powf(MAX_BOOST_DB / 20.0);
    let gain = (TARGET_PEAK / current).min(ceiling);
    if gain <= 1.0 {
        return 1.0;
    }
    for sample in samples.iter_mut() {
        *sample = (*sample * gain).clamp(-1.0, 1.0);
    }
    gain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boosts_quiet_speech_towards_full_scale() {
        let mut samples = vec![0.1, -0.1, 0.04];
        let gain = normalize(&mut samples);
        assert!((peak(&samples) - TARGET_PEAK).abs() < 1e-3);
        assert!((gain - TARGET_PEAK / 0.1).abs() < 1e-3);
    }

    #[test]
    fn never_boosts_more_than_the_ceiling() {
        let mut samples = vec![0.0001, -0.0001];
        normalize(&mut samples);
        assert!(peak(&samples) <= 0.0001 * 10f32.powf(MAX_BOOST_DB / 20.0) + 1e-6);
    }

    #[test]
    fn leaves_loud_audio_untouched() {
        let mut samples = vec![0.95, -0.95];
        assert_eq!(normalize(&mut samples), 1.0);
        assert_eq!(samples, vec![0.95, -0.95]);
    }

    #[test]
    fn leaves_silence_untouched() {
        let mut samples = vec![0.0, 0.0];
        assert_eq!(normalize(&mut samples), 1.0);
        assert_eq!(samples, vec![0.0, 0.0]);
    }
}
