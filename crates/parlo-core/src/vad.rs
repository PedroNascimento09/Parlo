pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

#[derive(Debug, Clone)]
pub struct VoiceGate {
    floor: f32,
    threshold: f32,
    hangover: u32,
    remaining: u32,
}

impl VoiceGate {
    pub fn new(threshold_db: f32, hangover_frames: u32) -> Self {
        Self {
            floor: 0.0,
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
        self.floor = 0.0;
    }

    pub fn process(&mut self, samples: &[f32]) -> bool {
        let level = rms(samples);
        if self.floor == 0.0 {
            self.floor = level;
        } else {
            self.floor = self.floor * 0.98 + level * 0.02;
        }
        let adaptive = self.floor.max(self.threshold);
        let open = level >= adaptive;
        if open {
            self.remaining = self.hangover.saturating_add(1);
        } else if self.remaining > 0 {
            self.remaining = self.remaining.saturating_sub(1);
        }
        self.remaining > 0
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

    #[test]
    fn floor_adapts_to_background_so_speech_still_opens() {
        let mut gate = VoiceGate::new(-45.0, 3);
        for _ in 0..50 {
            gate.process(&[0.01; 160]);
        }
        assert!(gate.process(&[0.3; 160]));
        assert!(gate.is_open());
    }
}
