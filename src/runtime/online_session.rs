//! Switch side of [`crate::online_session`] (0.9.0): the diagnostics poll
//! feeds Pia's cumulative RTT sample count every 500 ms; any thread asks
//! whether the online session is live. Without the Pia observer there is no
//! evidence and the session is never live.

use crate::online_session::{live_since, Session};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Written only by the diagnostics thread; the lock never contends.
static SESSION: Mutex<Session> = Mutex::new(Session::new());
/// The last evidence tick, for readers on other threads (0 never).
static EVIDENCE_TICK: AtomicU64 = AtomicU64::new(0);

pub(super) fn tick() -> u64 {
    unsafe { skyline::nn::os::GetSystemTick() as u64 }
}

/// Diagnostics thread, right after `pia_rtt::poll`.
pub(super) fn poll() {
    if !super::pia_rtt::ready() {
        return;
    }
    let Ok(mut session) = SESSION.try_lock() else {
        return;
    };
    if session.observe(super::pia_rtt::evidence_count(), tick()) {
        EVIDENCE_TICK.store(session.evidence_tick(), Ordering::Release);
    }
}

pub(super) fn live_at(now: u64) -> bool {
    live_since(EVIDENCE_TICK.load(Ordering::Acquire), now)
}
