const MIN_AUDIO_MS: u64 = 800;

pub struct Event {
    pub state: &'static str,
    pub text: String,
    pub audio_ms: u64,
    pub latency_ms: u64,
}

#[derive(Default)]
pub struct Session {
    samples: Vec<f32>,
    last: String,
}

impl Session {
    pub fn push(&mut self, samples: &[f32]) {
        self.samples.extend_from_slice(samples);
    }

    pub fn audio_ms(&self) -> u64 {
        (self.samples.len() as u64 * 1000) / 16_000
    }

    pub fn audio(&self) -> &[f32] {
        &self.samples
    }

    pub fn reset(&mut self) {
        self.samples.clear();
        self.last.clear();
    }

    pub fn observe(&mut self, text: &str, latency_ms: u64) -> Vec<Event> {
        let text = text.trim();
        if text.is_empty() || text == self.last || self.audio_ms() < MIN_AUDIO_MS {
            return Vec::new();
        }
        self.last = text.to_string();
        vec![Event {
            state: "partial",
            text: text.to_string(),
            audio_ms: self.audio_ms(),
            latency_ms,
        }]
    }

    pub fn finish(&mut self, text: &str, latency_ms: u64) -> Vec<Event> {
        let text = text.trim();
        let mut events = Vec::new();
        if !text.is_empty() && text != self.last {
            events.push(Event {
                state: "stable",
                text: text.to_string(),
                audio_ms: self.audio_ms(),
                latency_ms,
            });
        }
        self.reset();
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_audio_is_not_announced() {
        let mut session = Session::default();
        session.push(&vec![0.1; 1_600]);
        assert!(session.observe("hello", 5).is_empty());
    }

    #[test]
    fn the_same_text_is_announced_once() {
        let mut session = Session::default();
        session.push(&vec![0.1; 32_000]);
        assert_eq!(session.observe("hello", 5).len(), 1);
        assert!(session.observe("hello", 5).is_empty());
    }

    #[test]
    fn closing_reports_the_last_text_and_clears_the_audio() {
        let mut session = Session::default();
        session.push(&vec![0.1; 32_000]);
        let events = session.finish("a sentence", 7);
        assert_eq!(events[0].state, "stable");
        assert_eq!(session.audio().len(), 0);
    }
}
