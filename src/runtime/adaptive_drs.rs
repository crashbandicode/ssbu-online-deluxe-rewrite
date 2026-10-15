//! Adaptive dynamic-resolution trigger (0.5.2 experiment), present-thread side.
//!
//! `queue_present` reports every tracked present tick here. The pure state
//! machine in `drs_triggers::Adaptive` counts frames over and under the
//! configured budget; its `Push`/`Pop` decisions move one resolution report
//! through `perf_scaler`, the same multiset the cut-in and move triggers use.
//! Disabled in the INI: one atomic load per present and nothing else.

use super::{record, Code};
use crate::drs_triggers::{Adaptive, AdaptiveEvent, TICKS_PER_HUNDREDTH_MS};
use crate::perf_scaler::{self, Action};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

static STATE: Mutex<Adaptive> = Mutex::new(Adaptive::new());
static LAST_TICK: AtomicU64 = AtomicU64::new(0);
/// Mirrors the state machine's active flag so a disabled setting can still
/// release a held report without taking the lock on every present.
static ACTIVE: AtomicBool = AtomicBool::new(false);
static OBSERVED: AtomicU64 = AtomicU64::new(0);

pub(super) fn on_present(tick: u64) {
    let settings = super::adaptive_settings();
    let last = LAST_TICK.swap(tick, Ordering::AcqRel);
    if !settings.enabled && !ACTIVE.load(Ordering::Acquire) {
        return;
    }
    if last == 0 || tick <= last {
        return;
    }
    let interval = ((tick - last) / TICKS_PER_HUNDREDTH_MS).min(u64::from(u32::MAX)) as u32;
    OBSERVED.fetch_add(1, Ordering::Relaxed);
    let event = {
        let mut state = STATE.lock().unwrap_or_else(|err| err.into_inner());
        let event = state.observe(interval, &settings);
        ACTIVE.store(state.active(), Ordering::Release);
        event
    };
    let level = perf_scaler::intensive_level(perf_scaler::global().base_level()) as u64;
    match event {
        Some(AdaptiveEvent::Push) => {
            if perf_scaler::global().adaptive_push() == Action::Started {
                record(
                    Code::DrsTrigger,
                    2,
                    1 | (level << 8) | (u64::from(interval) << 16),
                );
            }
        }
        Some(AdaptiveEvent::Pop) => {
            if perf_scaler::global().adaptive_pop() == Action::Ended {
                record(
                    Code::DrsTrigger,
                    2,
                    2 | (level << 8) | (u64::from(interval) << 16),
                );
            }
        }
        None => {}
    }
}

/// Scene reset or retarget: the scaler already dropped the report token; the
/// counters start over so the next scene needs a fresh run of late frames.
pub(super) fn reset() {
    let mut state = STATE.lock().unwrap_or_else(|err| err.into_inner());
    state.reset();
    ACTIVE.store(false, Ordering::Release);
    LAST_TICK.store(0, Ordering::Release);
}

pub(super) fn observed() -> u64 {
    OBSERVED.load(Ordering::Relaxed)
}
