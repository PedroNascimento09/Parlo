#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyBudget {
    pub capture_ms: u32,
    pub engine_ms: u32,
    pub output_ms: u32,
}

impl LatencyBudget {
    pub const TARGET_MS: u32 = 3_000;
    pub const CEILING_MS: u32 = 5_000;

    pub fn new(capture_ms: u32, engine_ms: u32, output_ms: u32) -> Self {
        Self {
            capture_ms,
            engine_ms,
            output_ms,
        }
    }

    pub fn total_ms(&self) -> u32 {
        self.capture_ms
            .saturating_add(self.engine_ms)
            .saturating_add(self.output_ms)
    }

    pub fn is_within_target(&self) -> bool {
        self.total_ms() <= Self::TARGET_MS
    }

    pub fn is_within_ceiling(&self) -> bool {
        self.total_ms() <= Self::CEILING_MS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_the_stages() {
        let budget = LatencyBudget::new(80, 2_000, 20);
        assert_eq!(budget.total_ms(), 2_100);
        assert!(budget.is_within_target());
        assert!(budget.is_within_ceiling());
    }

    #[test]
    fn flags_a_slow_engine() {
        let budget = LatencyBudget::new(80, 5_400, 20);
        assert!(!budget.is_within_target());
        assert!(!budget.is_within_ceiling());
    }
}
