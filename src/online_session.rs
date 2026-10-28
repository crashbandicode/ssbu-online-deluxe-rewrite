//! Network-proven online session (0.8.1). The one answer to "is this console
//! playing online right now" for every consumer: the Horizon-OC game-aware
//! gate and Auto's carried-mark check. The evidence is Pia's RTT samples, which
//! exist only while the game is connected to another player over the internet
//! or local wireless; offline play produces none, so no scene hook (and no
//! stale Quickplay connection) can make offline play look online. The runtime
//! feeds the cumulative sample count from the 500 ms diagnostics poll.

/// `nn::os::GetSystemTick` rate (19.2 MHz).
pub const TICKS_PER_SECOND: u64 = 19_200_000;

/// The session stays live this long after the last sample, so it ends 15 s
/// after the last player disconnects unless one reconnects (operator
/// 2026-10-28: the candidate's 90 s kept the overclock on too long; never on
/// offline). Longer than the 10 s a connected peer may go without a sample
/// ([`crate::pia_rtt::LIVE_MS`]), which covers loading gaps.
pub const HOLD_TICKS: u64 = 15 * TICKS_PER_SECOND;

/// Last cumulative sample count seen and the tick it last changed (0 never).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Session {
    count: u64,
    evidence_tick: u64,
}

impl Session {
    pub const fn new() -> Self {
        Self {
            count: 0,
            evidence_tick: 0,
        }
    }

    /// One poll of the cumulative count. Any change is network activity at
    /// `now` (a reset table counts too: only a live session replaces it).
    /// Returns whether this poll found new evidence.
    pub fn observe(&mut self, count: u64, now: u64) -> bool {
        if count == self.count {
            return false;
        }
        self.count = count;
        self.evidence_tick = now.max(1);
        true
    }

    pub const fn evidence_tick(self) -> u64 {
        self.evidence_tick
    }

    pub const fn live(self, now: u64) -> bool {
        live_since(self.evidence_tick, now)
    }
}

/// Live within [`HOLD_TICKS`] of the last evidence; never before the first.
/// A reader whose `now` was sampled before another thread stored newer
/// evidence sees zero elapsed, not a wrapped age.
pub const fn live_since(evidence_tick: u64, now: u64) -> bool {
    evidence_tick != 0 && now.saturating_sub(evidence_tick) < HOLD_TICKS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_evidence_is_never_live() {
        let session = Session::new();
        assert!(!session.live(0));
        assert!(!session.live(HOLD_TICKS * 3));
    }

    #[test]
    fn a_changed_count_is_evidence_and_holds_for_the_window() {
        let mut session = Session::new();
        assert!(!session.observe(0, 100));
        assert!(session.observe(4, 1_000));
        assert_eq!(session.evidence_tick(), 1_000);
        assert!(session.live(1_000));
        assert!(session.live(1_000 + HOLD_TICKS - 1));
        assert!(!session.live(1_000 + HOLD_TICKS));
    }

    #[test]
    fn an_unchanged_count_does_not_renew() {
        let mut session = Session::new();
        session.observe(7, 10);
        assert!(!session.observe(7, 10 + HOLD_TICKS / 2));
        assert_eq!(session.evidence_tick(), 10);
        assert!(!session.live(10 + HOLD_TICKS));
    }

    #[test]
    fn a_reset_count_is_still_activity() {
        let mut session = Session::new();
        session.observe(50, 10);
        assert!(session.observe(3, 20));
        assert_eq!(session.evidence_tick(), 20);
    }

    #[test]
    fn evidence_at_tick_zero_is_kept_as_one() {
        let mut session = Session::new();
        assert!(session.observe(1, 0));
        assert_eq!(session.evidence_tick(), 1);
        assert!(session.live(0));
    }

    #[test]
    fn a_reader_behind_the_writer_sees_a_live_session() {
        assert!(live_since(1_000, 999));
        assert!(!live_since(0, 999));
    }

    #[test]
    fn hold_is_fifteen_seconds() {
        assert_eq!(HOLD_TICKS, 288_000_000);
    }
}
