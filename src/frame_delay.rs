//! Conservative budget from the blocking acquire following the delayed frame.
//! Units are microseconds. This estimates CPU pacing slack, not physical vblank.
pub const WINDOW: usize = 32;
pub const CAP_US: u64 = 4_000;
pub const MARGIN_US: u64 = 3_000;

pub struct Budget {
    samples: [u64; WINDOW],
    next: usize,
    pub count: usize,
}
impl Budget {
    pub const fn new() -> Self {
        Self {
            samples: [0; WINDOW],
            next: 0,
            count: 0,
        }
    }
    pub fn reset(&mut self) {
        self.next = 0;
        self.count = 0;
    }
    pub fn observe(&mut self, period: u64, blocked: u64, preceding_sleep: u64, clean: bool) {
        if !clean
            || !(14_000..=19_000).contains(&period)
            || blocked < 200
            || preceding_sleep > CAP_US + 500
        {
            self.reset();
            return;
        }
        self.samples[self.next] = blocked.saturating_add(preceding_sleep);
        self.next = (self.next + 1) % WINDOW;
        self.count = (self.count + 1).min(WINDOW);
    }
    pub fn delay(&self) -> u64 {
        if self.count < WINDOW {
            return 0;
        }
        self.samples
            .iter()
            .copied()
            .min()
            .unwrap_or(0)
            .saturating_sub(MARGIN_US)
            .min(CAP_US)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn warmup_and_compensated_feedback_do_not_oscillate() {
        let mut b = Budget::new();
        for _ in 0..31 {
            b.observe(16_667, 9_000, 0, true);
            assert_eq!(b.delay(), 0);
        }
        b.observe(16_667, 9_000, 0, true);
        assert_eq!(b.delay(), 4_000);
        for _ in 0..64 {
            b.observe(16_667, 5_000, 4_000, true);
            assert_eq!(b.delay(), 4_000);
        }
    }
    #[test]
    fn worst_recent_slack_and_margin_limit_sleep() {
        let mut b = Budget::new();
        for _ in 0..32 {
            b.observe(16_667, 9_000, 0, true);
        }
        b.observe(16_667, 4_000, 0, true);
        assert_eq!(b.delay(), 1_000);
        b.observe(16_667, 2_000, 0, true);
        assert_eq!(b.delay(), 0);
    }
    #[test]
    fn cadence_catchup_oversleep_and_zero_wait_reset() {
        for (period, blocked, sleep, clean) in [
            (33_333, 9_000, 0, true),
            (16_667, 9_000, 0, false),
            (16_667, 9_000, 4_501, true),
            (16_667, 0, 0, true),
        ] {
            let mut b = Budget::new();
            for _ in 0..32 {
                b.observe(16_667, 9_000, 0, true);
            }
            b.observe(period, blocked, sleep, clean);
            assert_eq!(b.count, 0);
            assert_eq!(b.delay(), 0);
        }
    }
}
