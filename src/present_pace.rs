//! Present pacing diagnostics (0.8.2), added after the 2026-10-28 report of
//! tearing at 120 Hz. That session's captures showed one present every
//! 16.67 ms and no acquire of the image just presented, but captures run only
//! with `CAPTURE_MATCH` and record CPU-side calls, not scanout. These checks
//! run on every present and log only the exceptions, within a row budget.

/// `nn::os::GetSystemTick` rate (19.2 MHz).
const TICKS_PER_SECOND: u64 = 19_200_000;

/// Spacing of one present per 16.67 ms (interval 1 at 60 Hz, interval 2 at
/// 120 Hz) stays inside these bounds; anything else is logged.
pub const PACE_LOW_US: u64 = 14_000;
pub const PACE_HIGH_US: u64 = 19_000;

/// Rows per second, shared by every pacing code, so a stage load or a
/// stall cannot flood `events.log`. The status counters stay exact.
pub const ROWS_PER_SECOND: u64 = 4;

/// System ticks to microseconds (19.2 ticks per microsecond).
pub const fn ticks_to_us(ticks: u64) -> u64 {
    ticks * 5 / 96
}

pub const fn pace_outlier(delta_us: u64) -> bool {
    delta_us < PACE_LOW_US || delta_us > PACE_HIGH_US
}

/// An acquire returned the image that was presented last. A two- or
/// three-image ring hands out a different image next, so this is the reuse
/// that could draw into an image still on screen.
pub const fn reacquired_last(last_presented: i32, acquired: i32) -> bool {
    last_presented >= 0 && acquired == last_presented
}

/// Row budget packed as `second << 8 | rows`. Returns the next state when a
/// row may be written at `tick`, `None` when this second's budget is spent.
pub const fn spend(state: u64, tick: u64) -> Option<u64> {
    let second = tick / TICKS_PER_SECOND;
    let rows = if state >> 8 == second {
        state & 0xff
    } else {
        0
    };
    if rows >= ROWS_PER_SECOND {
        None
    } else {
        Some(second << 8 | (rows + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_present_per_two_120hz_periods_is_normal() {
        let frame = 320_000; // 16.667 ms in ticks
        assert_eq!(ticks_to_us(frame), 16_666);
        assert!(!pace_outlier(ticks_to_us(frame)));
        // Half a frame (one 120 Hz period) and a dropped frame are outliers.
        assert!(pace_outlier(ticks_to_us(frame / 2)));
        assert!(pace_outlier(ticks_to_us(frame * 2)));
        assert!(!pace_outlier(PACE_LOW_US) && !pace_outlier(PACE_HIGH_US));
    }

    #[test]
    fn only_the_last_presented_index_counts_as_reuse() {
        assert!(reacquired_last(0, 0));
        assert!(!reacquired_last(0, 1));
        assert!(!reacquired_last(-1, -1));
        assert!(!reacquired_last(-1, 0));
    }

    #[test]
    fn the_budget_allows_a_few_rows_per_second_and_refills() {
        let mut state = 0;
        for _ in 0..ROWS_PER_SECOND {
            state = spend(state, 5).unwrap();
        }
        assert_eq!(spend(state, TICKS_PER_SECOND - 1), None);
        assert!(spend(state, TICKS_PER_SECOND).is_some());
    }
}
