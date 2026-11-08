#[derive(Debug, Clone)]
pub struct Resampler {
    step: f64,
    position: f64,
    previous: f32,
}

impl Resampler {
    pub fn new(from_rate: u32, to_rate: u32) -> Self {
        let step = if from_rate == 0 {
            1.0
        } else {
            from_rate as f64 / to_rate.max(1) as f64
        };
        Self {
            step,
            position: 0.0,
            previous: 0.0,
        }
    }

    pub fn is_passthrough(&self) -> bool {
        (self.step - 1.0).abs() < f64::EPSILON
    }

    pub fn reset(&mut self) {
        self.position = 0.0;
        self.previous = 0.0;
    }

    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        if input.is_empty() {
            return;
        }
        let len = input.len() as f64;
        let mut position = self.position;
        while position < len {
            let index = position.floor();
            let fraction = (position - index) as f32;
            let left = if index < 0.0 {
                self.previous
            } else {
                input[index as usize]
            };
            let next = index as isize + 1;
            let right = if next < 0 {
                self.previous
            } else if (next as usize) < input.len() {
                input[next as usize]
            } else {
                left
            };
            out.push(left + (right - left) * fraction);
            position += self.step;
        }
        self.position = position - len;
        self.previous = input[input.len() - 1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsamples_by_ratio() {
        let mut resampler = Resampler::new(48_000, 16_000);
        let mut out = Vec::new();
        resampler.process(&vec![0.0; 480], &mut out);
        assert_eq!(out.len(), 160);
    }

    #[test]
    fn keeps_stream_continuous_across_blocks() {
        let mut resampler = Resampler::new(48_000, 16_000);
        let mut out = Vec::new();
        for _ in 0..3 {
            resampler.process(&vec![1.0; 160], &mut out);
        }
        assert_eq!(out.len(), 160);
        assert!(out.iter().all(|s| (*s - 1.0).abs() < 1e-6));
    }

    #[test]
    fn interpolates_between_samples() {
        let mut resampler = Resampler::new(2, 3);
        let mut out = Vec::new();
        resampler.process(&[0.0, 1.0], &mut out);
        assert_eq!(out.len(), 3);
        assert!((out[1] - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn detects_passthrough() {
        assert!(Resampler::new(16_000, 16_000).is_passthrough());
        assert!(!Resampler::new(48_000, 16_000).is_passthrough());
    }

    #[test]
    fn upsamples_by_ratio() {
        let mut resampler = Resampler::new(16_000, 48_000);
        let mut out = Vec::new();
        resampler.process(&vec![0.0; 160], &mut out);
        assert_eq!(out.len(), 480);
    }

    #[test]
    fn zero_input_produces_no_output() {
        let mut resampler = Resampler::new(48_000, 16_000);
        let mut out = Vec::new();
        resampler.process(&[], &mut out);
        assert!(out.is_empty());
    }
}
