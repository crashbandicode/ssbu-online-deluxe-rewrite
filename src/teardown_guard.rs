//! Pure arithmetic for the render-sequence teardown guard.
//!
//! The render-opts path moves the game's fighter model update and fighter
//! command recording after the scene update. When that scene update tears a
//! fighter scene down (modules unload on the way from Training or a match to
//! the character select), the relocated model update runs on objects the
//! game has already destroyed; on 2026-10-14 that aborted four Eden runs in
//! `nn::os::LockMutex` (kernel `InvalidHandle`) from `deluxe_rewrite_post_scene`.
//! The guard skips the relocated work on a frame whose scene update unloaded a
//! module and hands the next `QUIET_FRAMES` frames back to the game's own
//! ordering, which is what every scene transition ran before the relocation.
//! The runtime statics live in `runtime::render_sequence`; this module keeps
//! the decisions host-testable.

/// Frames after a detected unload during which render opts stay off.
pub const QUIET_FRAMES: u64 = 180;

/// First frame on which the relocation may run again after an unload seen on
/// `frame`.
pub fn quiet_until(frame: u64) -> u64 {
    frame.saturating_add(QUIET_FRAMES)
}

/// Whether the relocation may run on `frame` given the stored quiet deadline.
pub fn relocation_allowed(frame: u64, quiet_until: u64) -> bool {
    frame >= quiet_until
}

/// Whether a module unloaded between the frame's fighter recording (which
/// snapshotted `epoch_at_record`) and its post-scene hook (`epoch_now`).
pub fn teardown_detected(epoch_at_record: u64, epoch_now: u64) -> bool {
    epoch_at_record != epoch_now
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_window_covers_the_next_three_seconds_of_frames() {
        let until = quiet_until(1_000);
        assert_eq!(until, 1_180);
        assert!(!relocation_allowed(1_000, until));
        assert!(!relocation_allowed(1_179, until));
        assert!(relocation_allowed(1_180, until));
        assert!(relocation_allowed(5_000, until));
    }

    #[test]
    fn fresh_start_allows_relocation() {
        assert!(relocation_allowed(0, 0));
        assert!(relocation_allowed(7, 0));
    }

    #[test]
    fn quiet_deadline_saturates_instead_of_wrapping() {
        let until = quiet_until(u64::MAX - 10);
        assert_eq!(until, u64::MAX);
        assert!(!relocation_allowed(u64::MAX - 1, until));
    }

    #[test]
    fn teardown_is_an_epoch_change_within_the_frame() {
        assert!(!teardown_detected(3, 3));
        assert!(teardown_detected(3, 4));
        assert!(teardown_detected(3, 9));
    }
}
