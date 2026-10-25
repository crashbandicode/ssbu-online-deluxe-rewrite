//! Match-scoped observation of the automatic input-buffer length.
//!
//! The producer takes [`AutoBufferState::observation_token`] before it reads the
//! game byte, then publishes with [`AutoBufferState::observe`]. The word stores
//! no pointer. A sample from an older generation, a closed match, or a manual
//! selection cannot change the count still shown for the current result.

use std::sync::atomic::{AtomicU64, Ordering};

/// Largest frame count the game keeps. Anything above this stays unknown.
const MAX_FRAMES: u8 = 25;
/// Count byte meaning "do not show a number". Zero is a real sample.
const UNKNOWN_COUNT: u8 = u8::MAX;
const ACTIVE: u64 = 1 << 8;
const ELIGIBLE: u64 = 1 << 9;
const GENERATION_SHIFT: u32 = 16;
const GENERATION_MASK: u64 = (1 << 48) - 1;

#[derive(Clone, Copy)]
struct Sample {
    generation: u64,
    active: bool,
    eligible: bool,
    count: Option<u8>,
}

const fn known_count(frames: u8) -> Option<u8> {
    if frames <= MAX_FRAMES {
        Some(frames)
    } else {
        None
    }
}

const fn next_generation(generation: u64) -> u64 {
    generation.wrapping_add(1) & GENERATION_MASK
}

const fn encode(sample: Sample) -> u64 {
    let count = match sample.count {
        Some(count) => count,
        None => UNKNOWN_COUNT,
    };
    let mut word = count as u64;
    if sample.active {
        word |= ACTIVE;
    }
    if sample.eligible {
        word |= ELIGIBLE;
    }
    word | ((sample.generation & GENERATION_MASK) << GENERATION_SHIFT)
}

const fn decode(word: u64) -> Sample {
    Sample {
        generation: (word >> GENERATION_SHIFT) & GENERATION_MASK,
        active: (word & ACTIVE) != 0,
        eligible: (word & ELIGIBLE) != 0,
        count: known_count((word & 0xff) as u8),
    }
}

/// Epoch captured before the producer reads the game byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservationToken {
    generation: u64,
}

/// One atomic word: generation, active, eligible, and a known-count sentinel.
#[derive(Debug)]
pub struct AutoBufferState {
    word: AtomicU64,
}

impl Default for AutoBufferState {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoBufferState {
    pub const fn new() -> Self {
        Self {
            word: AtomicU64::new(UNKNOWN_COUNT as u64),
        }
    }

    /// Opens a match. Only `selected == -1` (Auto) may be sampled. The previous
    /// count is hidden until this match publishes one.
    pub fn begin_match(&self, selected: i8) {
        let eligible = selected == -1;
        self.publish(|sample| {
            Some(Sample {
                generation: next_generation(sample.generation),
                active: true,
                eligible,
                count: None,
            })
        });
    }

    /// Closes sampling and drops in-flight tokens. A known count stays visible
    /// for results until the next match, clear, or manual invalidation.
    /// A second call leaves that count and generation alone.
    pub fn end_match(&self) {
        self.publish(|sample| {
            if !sample.active {
                None
            } else {
                Some(Sample {
                    generation: next_generation(sample.generation),
                    active: false,
                    eligible: sample.eligible,
                    count: sample.count,
                })
            }
        });
    }

    /// Offline, Training, or an explicit reset. The count becomes unknown.
    pub fn clear(&self) {
        self.publish(|sample| {
            Some(Sample {
                generation: next_generation(sample.generation),
                active: false,
                eligible: false,
                count: None,
            })
        });
    }

    /// Drops the current sample and refuses further ones until the next
    /// [`Self::begin_match`]. Repeated calls, once already blocked and unknown,
    /// do not advance the generation.
    pub fn invalidate_manual(&self) {
        self.publish(|sample| {
            if !sample.eligible && sample.count.is_none() {
                None
            } else {
                Some(Sample {
                    generation: next_generation(sample.generation),
                    active: sample.active,
                    eligible: false,
                    count: None,
                })
            }
        });
    }

    /// `Some` only while this match is still accepting Auto samples.
    pub fn observation_token(&self) -> Option<ObservationToken> {
        let sample = decode(self.word.load(Ordering::Acquire));
        if sample.active && sample.eligible {
            Some(ObservationToken {
                generation: sample.generation,
            })
        } else {
            None
        }
    }

    /// Publishes `frames` for `token`'s generation. True only when the visible
    /// count changes, including the first sample of a match.
    pub fn observe(&self, token: ObservationToken, frames: u8) -> bool {
        if known_count(frames).is_none() {
            return false;
        }
        let mut current = self.word.load(Ordering::Acquire);
        loop {
            let sample = decode(current);
            if sample.generation != token.generation || !sample.active || !sample.eligible {
                return false;
            }
            if sample.count == Some(frames) {
                return false;
            }
            let next = encode(Sample {
                generation: sample.generation,
                active: sample.active,
                eligible: sample.eligible,
                count: Some(frames),
            });
            match self
                .word
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return true,
                Err(observed) => current = observed,
            }
        }
    }

    /// Last accepted count, including one retained after the match ends.
    pub fn value(&self) -> Option<u8> {
        decode(self.word.load(Ordering::Acquire)).count
    }

    /// Current epoch. Logging only; sampling uses [`ObservationToken`].
    pub fn generation(&self) -> u64 {
        decode(self.word.load(Ordering::Acquire)).generation
    }

    fn publish(&self, step: impl Fn(Sample) -> Option<Sample>) {
        let mut current = self.word.load(Ordering::Acquire);
        loop {
            let Some(next) = step(decode(current)) else {
                return;
            };
            match self.word.compare_exchange(
                current,
                encode(next),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(observed) => current = observed,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_state_is_unknown() {
        for state in [AutoBufferState::new(), AutoBufferState::default()] {
            assert_eq!(state.value(), None);
            assert!(state.observation_token().is_none());
            assert_eq!(state.generation(), 0);
            assert!(!state.observe(ObservationToken { generation: 0 }, 0));
            assert_eq!(state.value(), None);
        }
    }

    #[test]
    fn one_match_publishes_changes_and_rejects_out_of_range() {
        let state = AutoBufferState::new();
        state.begin_match(-1);
        assert_eq!(state.value(), None);
        let token = state.observation_token().expect("auto match");
        assert!(state.observe(token, 4));
        assert_eq!(state.value(), Some(4));
        assert!(!state.observe(token, 4));
        assert_eq!(state.value(), Some(4));
        assert!(state.observe(token, 0));
        assert_eq!(state.value(), Some(0));
        assert!(state.observe(token, 25));
        assert_eq!(state.value(), Some(25));
        assert!(state.observe(token, 3));
        assert_eq!(state.value(), Some(3));
        let generation = state.generation();
        assert!(!state.observe(token, 26));
        assert!(!state.observe(token, 255));
        assert_eq!(state.value(), Some(3));
        assert_eq!(state.generation(), generation);
    }

    #[test]
    fn next_match_stays_unknown_until_a_new_sample_and_end_keeps_it() {
        let state = AutoBufferState::new();
        state.begin_match(-1);
        let first = state.observation_token().expect("first match");
        assert!(state.observe(first, 4));
        assert_eq!(state.value(), Some(4));

        state.begin_match(-1);
        assert_eq!(state.value(), None);
        assert!(!state.observe(first, 4));
        assert_eq!(state.value(), None);
        let second = state.observation_token().expect("second match");
        assert!(state.observe(second, 3));
        assert_eq!(state.value(), Some(3));

        let late = state.observation_token().expect("still in match");
        state.end_match();
        assert_eq!(state.value(), Some(3));
        assert!(state.observation_token().is_none());
        assert!(!state.observe(late, 0));
        assert!(!state.observe(second, 25));
        assert_eq!(state.value(), Some(3));
    }

    #[test]
    fn duplicate_end_keeps_the_generation_and_the_count() {
        let state = AutoBufferState::new();
        state.begin_match(-1);
        let token = state.observation_token().expect("auto match");
        assert!(state.observe(token, 3));
        state.end_match();
        let generation = state.generation();
        state.end_match();
        assert_eq!(state.generation(), generation);
        assert_eq!(state.value(), Some(3));
        assert!(state.observation_token().is_none());
        assert!(!state.observe(token, 9));

        let idle = AutoBufferState::new();
        let generation = idle.generation();
        idle.end_match();
        idle.end_match();
        assert_eq!(idle.generation(), generation);
        assert_eq!(idle.value(), None);
    }

    #[test]
    fn clear_hides_a_count_kept_for_results() {
        let state = AutoBufferState::new();
        state.begin_match(-1);
        let token = state.observation_token().expect("auto match");
        assert!(state.observe(token, 3));
        state.end_match();
        assert_eq!(state.value(), Some(3));
        let generation = state.generation();
        state.clear();
        assert_ne!(state.generation(), generation);
        assert_eq!(state.value(), None);
        assert!(state.observation_token().is_none());
        assert!(!state.observe(token, 3));
        assert_eq!(state.value(), None);
    }

    #[test]
    fn manual_block_drops_the_sample_and_does_not_echo_auto() {
        let state = AutoBufferState::new();
        state.begin_match(-1);
        let before = state.observation_token().expect("auto match");
        assert!(state.observe(before, 4));
        let in_flight = state.observation_token().expect("still sampling");
        state.invalidate_manual();
        assert_eq!(state.value(), None);
        assert!(state.observation_token().is_none());
        assert!(!state.observe(in_flight, 4));
        assert!(!state.observe(before, 4));
        assert_eq!(state.value(), None);

        let blocked = state.generation();
        state.invalidate_manual();
        state.invalidate_manual();
        assert_eq!(state.generation(), blocked);
        assert_eq!(state.value(), None);
        assert!(state.observation_token().is_none());

        state.begin_match(-1);
        assert_eq!(state.value(), None);
        let fresh = state.observation_token().expect("next auto match");
        assert!(state.observe(fresh, 3));
        assert_eq!(state.value(), Some(3));
        assert!(!state.observe(in_flight, 4));
        assert_eq!(state.value(), Some(3));
    }

    #[test]
    fn invalid_selections_are_ineligible_and_repeat_marks_do_not_churn() {
        for selected in [0_i8, 4, 25, -2, i8::MAX, i8::MIN] {
            let state = AutoBufferState::new();
            state.begin_match(-1);
            let token = state.observation_token().expect("auto match");
            assert!(state.observe(token, 25));
            state.begin_match(selected);
            assert_eq!(state.value(), None);
            assert!(state.observation_token().is_none());
            assert!(!state.observe(token, 25));
            let generation = state.generation();
            state.invalidate_manual();
            state.invalidate_manual();
            assert_eq!(state.generation(), generation);
            assert_eq!(state.value(), None);
        }
    }

    #[test]
    fn stale_token_cannot_publish_over_a_newer_generation() {
        let state = AutoBufferState::new();
        state.begin_match(-1);
        let stale = state.observation_token().expect("open");
        state.clear();
        assert!(!state.observe(stale, 4));
        assert_eq!(state.value(), None);

        state.begin_match(-1);
        let in_flight = state.observation_token().expect("open");
        state.end_match();
        assert!(!state.observe(in_flight, 1));
        assert_eq!(state.value(), None);

        state.begin_match(-1);
        let previous = state.observation_token().expect("open");
        state.begin_match(-1);
        assert!(!state.observe(previous, 25));
        let current = state.observation_token().expect("new match");
        assert!(state.observe(current, 3));
        assert!(!state.observe(previous, 8));
        assert!(!state.observe(stale, 8));
        assert_eq!(state.value(), Some(3));
    }

    #[test]
    fn same_generation_samples_replace_the_visible_count() {
        let state = AutoBufferState::new();
        state.begin_match(-1);
        let first = state.observation_token().expect("open");
        let second = state.observation_token().expect("same match");
        assert_eq!(first, second);
        assert!(state.observe(first, 4));
        assert!(state.observe(second, 3));
        assert_eq!(state.value(), Some(3));
        assert!(state.observe(first, 4));
        assert_eq!(state.value(), Some(4));
        assert!(!state.observe(second, 4));
        assert_eq!(state.value(), Some(4));
        assert_eq!(state.generation(), first.generation);
    }

    #[test]
    fn contending_samples_publish_a_legal_count_without_a_timing_assumption() {
        let state = AutoBufferState::new();
        state.begin_match(-1);
        let token = state.observation_token().expect("open");
        let state = &state;
        std::thread::scope(|scope| {
            for frames in [0_u8, 3, 26, 25, 255, 4] {
                scope.spawn(move || {
                    let _ = state.observe(token, frames);
                });
            }
        });
        let value = state.value().expect("in-range sample");
        assert!([0, 3, 25, 4].contains(&value));
        let retained = state.value();
        state.end_match();
        state.end_match();
        assert_eq!(state.value(), retained);
        assert!(!state.observe(token, 1));
        assert_eq!(state.value(), retained);
    }

    #[test]
    fn manual_before_the_first_sample_rejects_the_in_flight_token() {
        let state = AutoBufferState::new();
        state.begin_match(-1);
        let in_flight = state.observation_token().expect("open");
        state.invalidate_manual();
        assert_eq!(state.value(), None);
        assert!(state.observation_token().is_none());
        assert!(!state.observe(in_flight, 4));
        let generation = state.generation();
        state.invalidate_manual();
        assert_eq!(state.generation(), generation);
        assert_eq!(state.value(), None);
    }
}
