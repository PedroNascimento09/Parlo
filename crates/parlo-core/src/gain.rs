const TARGET_PEAK: f32 = 0.89;
const MAX_BOOST_DB: f32 = 24.0;
const AGC_TARGET_PEAK: f32 = 0.5;
const AGC_RELEASE: f32 = 0.98;
const AGC_SMOOTHING: f32 = 0.3;

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

#[derive(Debug, Clone)]
pub struct AutoGain {
    envelope: f32,
    gain: f32,
}

impl AutoGain {
    pub fn new() -> Self {
        Self {
            envelope: 0.0,
            gain: 1.0,
        }
    }

    pub fn gain(&self) -> f32 {
        self.gain
    }

    pub fn process(&mut self, samples: &mut [f32]) -> f32 {
        self.envelope = peak(samples).max(self.envelope * AGC_RELEASE);
        let desired = if self.envelope > 0.0 {
            (AGC_TARGET_PEAK / self.envelope)
                .min(10f32.powf(MAX_BOOST_DB / 20.0))
                .max(1.0)
        } else {
            1.0
        };
        self.gain += (desired - self.gain) * AGC_SMOOTHING;
        for sample in samples.iter_mut() {
            *sample = (*sample * self.gain).clamp(-1.0, 1.0);
        }
        self.gain
    }
}

impl Default for AutoGain {
    fn default() -> Self {
        Self::new()
    }
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
    fn auto_gain_lifts_a_quiet_frame_over_several_blocks() {
        let mut gain = AutoGain::new();
        let mut frame = vec![0.03f32; 320];
        for _ in 0..20 {
            gain.process(&mut frame);
        }
        assert!(peak(&frame) > 0.2);
        assert!(gain.gain() <= 10f32.powf(MAX_BOOST_DB / 20.0) + 1e-3);
    }

    #[test]
    fn auto_gain_leaves_a_loud_frame_alone() {
        let mut gain = AutoGain::new();
        let mut frame = vec![0.8f32; 320];
        assert!((gain.process(&mut frame) - 1.0).abs() < 1e-6);
        assert_eq!(frame[0], 0.8);
    }

    #[test]
    fn auto_gain_keeps_silence_silent() {
        let mut gain = AutoGain::new();
        let mut frame = vec![0.0f32; 320];
        assert!((gain.process(&mut frame) - 1.0).abs() < 1e-6);
        assert_eq!(frame[0], 0.0);
    }

    #[test]
    fn leaves_silence_untouched() {
        let mut samples = vec![0.0, 0.0];
        assert_eq!(normalize(&mut samples), 1.0);
        assert_eq!(samples, vec![0.0, 0.0]);
    }
}
