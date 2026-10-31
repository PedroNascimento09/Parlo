use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct RingBuffer {
    queue: VecDeque<f32>,
    capacity: usize,
}

impl RingBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn push(&mut self, samples: &[f32]) -> usize {
        let overflow = samples.len().saturating_sub(self.capacity);
        let incoming = &samples[overflow..];
        let evicted = (self.queue.len() + incoming.len()).saturating_sub(self.capacity);
        for _ in 0..evicted {
            self.queue.pop_front();
        }
        self.queue.extend(incoming.iter().copied());
        evicted + overflow
    }

    pub fn pop_into(&mut self, out: &mut [f32]) -> usize {
        let count = out.len().min(self.queue.len());
        for slot in out.iter_mut().take(count) {
            *slot = self.queue.pop_front().unwrap_or(0.0);
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_oldest_samples_when_full() {
        let mut ring = RingBuffer::with_capacity(4);
        assert_eq!(ring.push(&[1.0, 2.0, 3.0]), 0);
        assert_eq!(ring.push(&[4.0, 5.0]), 1);
        let mut out = [0.0; 4];
        assert_eq!(ring.pop_into(&mut out), 4);
        assert_eq!(out, [2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn keeps_newest_samples_for_oversized_push() {
        let mut ring = RingBuffer::with_capacity(2);
        assert_eq!(ring.push(&[1.0, 2.0, 3.0]), 1);
        let mut out = [0.0; 2];
        ring.pop_into(&mut out);
        assert_eq!(out, [2.0, 3.0]);
    }

    #[test]
    fn pops_partially_into_short_buffer() {
        let mut ring = RingBuffer::with_capacity(8);
        ring.push(&[1.0, 2.0, 3.0]);
        let mut out = [0.0; 2];
        assert_eq!(ring.pop_into(&mut out), 2);
        assert_eq!(ring.len(), 1);
    }
}
