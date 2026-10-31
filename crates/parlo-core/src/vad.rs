pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

#[derive(Debug, Clone)]
pub struct VoiceGate {
    threshold: f32,
    hangover: u32,
    remaining: u32,
}

impl VoiceGate {
    pub fn new(threshold_db: f32, hangover_frames: u32) -> Self {
        Self {
            threshold: 10f32.powf(threshold_db / 20.0),
            hangover: hangover_frames,
            remaining: 0,
        }
    }

    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    pub fn is_open(&self) -> bool {
        self.remaining > 0
    }

    pub fn reset(&mut self) {
        self.remaining = 0;
    }

    pub fn process(&mut self, samples: &[f32]) -> bool {
        if rms(samples) >= self.threshold {
            self.remaining = self.hangover.saturating_add(1);
        }
        let open = self.remaining > 0;
        self.remaining = self.remaining.saturating_sub(1);
        open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closes_on_silence() {
        let mut gate = VoiceGate::new(-45.0, 3);
        assert!(!gate.process(&[0.0; 160]));
    }

    #[test]
    fn opens_on_speech_level_signal() {
        let mut gate = VoiceGate::new(-45.0, 3);
        assert!(gate.process(&[0.3; 160]));
        assert!(gate.is_open());
    }

    #[test]
    fn holds_open_for_hangover_then_closes() {
        let mut gate = VoiceGate::new(-45.0, 3);
        gate.process(&[0.3; 160]);
        assert!(gate.process(&[0.0; 160]));
        assert!(gate.process(&[0.0; 160]));
        assert!(gate.process(&[0.0; 160]));
        assert!(!gate.process(&[0.0; 160]));
    }

    #[test]
    fn ignores_quiet_noise() {
        let mut gate = VoiceGate::new(-45.0, 3);
        assert!(!gate.process(&[0.0001; 160]));
    }
}
