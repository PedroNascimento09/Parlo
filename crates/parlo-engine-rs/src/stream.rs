const SAMPLE_RATE: u64 = 16_000;
const DECODE_EVERY_MS: u64 = 400;
const MIN_AUDIO_MS: u64 = 600;
const SEGMENT_MS: u64 = 12_000;
const ROUNDS: usize = 2;

pub struct Event {
    pub state: &'static str,
    pub text: String,
    pub audio_ms: u64,
    pub latency_ms: u64,
}

fn event(state: &'static str, text: String, audio_ms: u64, latency_ms: u64) -> Event {
    Event {
        state,
        text,
        audio_ms,
        latency_ms,
    }
}

#[derive(Default)]
struct Policy {
    confirmed: usize,
    confirmed_words: Vec<String>,
    history: Vec<Vec<String>>,
}

impl Policy {
    fn reset(&mut self) {
        self.confirmed = 0;
        self.confirmed_words.clear();
        self.history.clear();
    }

    fn agreement(&self) -> usize {
        let Some(first) = self.history.first() else {
            return 0;
        };
        let mut shared = 0;
        for (index, word) in first.iter().enumerate() {
            if self
                .history
                .iter()
                .all(|words| words.get(index) == Some(word))
            {
                shared += 1;
            } else {
                break;
            }
        }
        shared
    }

    fn observe(&mut self, words: Vec<String>, audio_ms: u64, latency_ms: u64) -> Vec<Event> {
        let mut events = Vec::new();
        let text = words.join(" ");
        let previous = self
            .history
            .last()
            .map(|entry| entry.join(" "))
            .unwrap_or_default();
        if !text.is_empty() && text != previous {
            events.push(event("partial", text, audio_ms, latency_ms));
        }

        self.history.push(words);
        if self.history.len() > ROUNDS {
            self.history.remove(0);
        }

        let shared = self.agreement();
        if self.history.len() == ROUNDS && shared > self.confirmed {
            let last = self.history.last().cloned().unwrap_or_default();
            let fresh = last[self.confirmed..shared].join(" ");
            self.confirmed = shared;
            self.confirmed_words = last[..shared].to_vec();
            events.push(event("stable", fresh, audio_ms, latency_ms));
        }
        events
    }

    fn finalize(&mut self, words: Vec<String>, audio_ms: u64, latency_ms: u64) -> Vec<Event> {
        let text = words.join(" ");
        if text.trim().is_empty() {
            return Vec::new();
        }
        let mut events = Vec::new();
        if self.confirmed > 0
            && words.get(..self.confirmed).map(<[String]>::to_vec)
                != Some(self.confirmed_words.clone())
        {
            events.push(event("corrected", text.clone(), audio_ms, latency_ms));
        }
        events.push(event("final", text, audio_ms, latency_ms));
        events
    }
}

#[derive(Default)]
pub struct Session {
    samples: Vec<f32>,
    decoded_ms: u64,
    last_decode_ms: u64,
    policy: Policy,
}

impl Session {
    pub fn push(&mut self, samples: &[f32]) {
        self.samples.extend_from_slice(samples);
    }

    pub fn audio(&self) -> &[f32] {
        &self.samples
    }

    pub fn audio_ms(&self) -> u64 {
        (self.samples.len() as u64 * 1000) / SAMPLE_RATE
    }

    pub fn wants_decode(&self) -> bool {
        let audio = self.audio_ms();
        audio >= MIN_AUDIO_MS
            && (audio - self.decoded_ms >= self.interval_ms() || audio >= SEGMENT_MS)
    }

    fn interval_ms(&self) -> u64 {
        DECODE_EVERY_MS.max(self.last_decode_ms.saturating_mul(3) / 2)
    }

    pub fn reset(&mut self) {
        self.samples.clear();
        self.decoded_ms = 0;
        self.last_decode_ms = 0;
        self.policy.reset();
    }

    pub fn observe(&mut self, text: &str, latency_ms: u64) -> Vec<Event> {
        self.decoded_ms = self.audio_ms();
        self.last_decode_ms = latency_ms;
        let words = text.split_whitespace().map(str::to_string).collect();
        self.policy.observe(words, self.audio_ms(), latency_ms)
    }

    pub fn close(&mut self, text: &str, latency_ms: u64) -> Vec<Event> {
        if self.audio_ms() < MIN_AUDIO_MS {
            self.reset();
            return Vec::new();
        }
        let words = text.split_whitespace().map(str::to_string).collect();
        let events = self.policy.finalize(words, self.audio_ms(), latency_ms);
        self.reset();
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_with(seconds: f32) -> Session {
        let mut session = Session::default();
        session.push(&vec![0.1; (SAMPLE_RATE as f32 * seconds) as usize]);
        session
    }

    #[test]
    fn short_audio_is_not_decoded() {
        let session = session_with(0.3);
        assert!(!session.wants_decode());
    }

    #[test]
    fn audio_is_decoded_every_four_hundred_milliseconds() {
        let mut session = session_with(1.0);
        assert!(session.wants_decode());
        session.observe("hello", 5);
        assert!(!session.wants_decode());
        session.push(&vec![0.1; (SAMPLE_RATE as f32 * 0.5) as usize]);
        assert!(session.wants_decode());
    }

    #[test]
    fn a_slow_decode_waits_longer_before_the_next_one() {
        let mut session = session_with(1.0);
        assert!(session.wants_decode());
        session.observe("hello", 900);
        session.push(&vec![0.1; (SAMPLE_RATE as f32 * 0.5) as usize]);
        assert!(!session.wants_decode());
        session.push(&vec![0.1; (SAMPLE_RATE as f32 * 0.9) as usize]);
        assert!(session.wants_decode());
    }

    #[test]
    fn the_first_hypothesis_is_only_provisional() {
        let mut session = session_with(1.0);
        let events = session.observe("hello there", 5);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].state, "partial");
    }

    #[test]
    fn agreement_between_two_rounds_confirms_the_shared_words() {
        let mut session = session_with(1.0);
        session.observe("hello there", 5);
        session.push(&vec![0.1; (SAMPLE_RATE as f32 * 0.5) as usize]);
        let events = session.observe("hello there friend", 5);
        let stable: Vec<&Event> = events.iter().filter(|e| e.state == "stable").collect();
        assert_eq!(stable.len(), 1);
        assert_eq!(stable[0].text, "hello there");
    }

    #[test]
    fn disagreeing_rounds_confirm_nothing() {
        let mut session = session_with(1.0);
        session.observe("um dois", 5);
        session.push(&vec![0.1; (SAMPLE_RATE as f32 * 0.5) as usize]);
        let events = session.observe("outra coisa", 5);
        assert!(events.iter().all(|event| event.state != "stable"));
    }

    #[test]
    fn closing_reports_final_and_clears_the_audio() {
        let mut session = session_with(1.0);
        session.observe("hello there", 5);
        let events = session.close("hello there friend", 6);
        assert_eq!(events.last().map(|event| event.state), Some("final"));
        assert_eq!(session.audio().len(), 0);
    }

    #[test]
    fn a_changed_prefix_is_reported_as_a_correction() {
        let mut session = session_with(1.0);
        session.observe("hello there", 5);
        session.push(&vec![0.1; (SAMPLE_RATE as f32 * 0.5) as usize]);
        session.observe("hello there friend", 5);
        let events = session.close("ahoy there friend", 6);
        assert_eq!(events.first().map(|event| event.state), Some("corrected"));
    }

    #[test]
    fn closing_short_audio_reports_nothing() {
        let mut session = session_with(0.2);
        assert!(session.close("anything", 4).is_empty());
    }
}
