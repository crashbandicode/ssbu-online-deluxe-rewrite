//! Runtime half of `crate::net_watch` (0.8.2). The Pia hooks in `pia_rtt`
//! pass integers they already read; this file keeps per-station sample ticks
//! and a 5 s window in atomics, and the diagnostics poll adds the console's
//! `nifm` status. Read-only: nothing is sent, written to the game or retried.
use super::{record, Code};
use crate::net_watch::{gap_ms, ms_since, pack_link, Window, ACTIVE_MS, NEVER};
use std::sync::atomic::{AtomicU64, Ordering};

const STATIONS: usize = crate::pia_rtt::MAX_STATION_INDEX as usize + 1;

/// Tick of each station's last RTT sample, 0 before the first.
static LAST: [AtomicU64; STATIONS] = [const { AtomicU64::new(0) }; STATIONS];
/// Tick of the newest sample from any station.
static NEWEST: AtomicU64 = AtomicU64::new(0);
static COUNT: AtomicU64 = AtomicU64::new(0);
static SUM: AtomicU64 = AtomicU64::new(0);
static MIN: AtomicU64 = AtomicU64::new(u64::MAX);
static MAX: AtomicU64 = AtomicU64::new(0);
static MAX_GAP: AtomicU64 = AtomicU64::new(0);
/// Last recorded `LocalLink` `a` and `b`; `u64::MAX` before the first poll.
static LINK: AtomicU64 = AtomicU64::new(u64::MAX);
static LINK_RESULT: AtomicU64 = AtomicU64::new(u64::MAX);

fn tick() -> u64 {
    unsafe { skyline::nn::os::GetSystemTick() as u64 }
}

/// One RTT sample from the sample hook. The local station and Pia's sentinel
/// indices are skipped; a negative RTT still counts as traffic.
pub(super) fn sample(remote: u32, local: u8, rtt_ms: i64) {
    if remote >= STATIONS as u32 || remote == u32::from(local) {
        return;
    }
    let now = tick();
    let last = LAST[remote as usize].swap(now, Ordering::AcqRel);
    NEWEST.fetch_max(now, Ordering::AcqRel);
    if let Some(gap) = gap_ms(last, now) {
        MAX_GAP.fetch_max(gap, Ordering::Relaxed);
        record(Code::RttGap, u64::from(remote), gap);
    }
    if let Ok(rtt) = u64::try_from(rtt_ms) {
        COUNT.fetch_add(1, Ordering::Relaxed);
        SUM.fetch_add(rtt, Ordering::Relaxed);
        MIN.fetch_min(rtt, Ordering::Relaxed);
        MAX.fetch_max(rtt, Ordering::Relaxed);
    }
}

/// Pia's station-left hook, with whether the RTT table accepted the leave.
pub(super) fn left(remote: u32, accepted: bool) {
    let silent = match LAST.get(remote as usize) {
        Some(last) => ms_since(last.load(Ordering::Acquire), tick()),
        None => NEVER,
    };
    record(
        Code::PeerLeft,
        u64::from(remote & 0xff) | u64::from(accepted) << 8,
        silent,
    );
}

/// Pia's session cleanup, with whether the RTT table accepted it.
pub(super) fn session_ended(accepted: bool) {
    let silent = ms_since(NEWEST.load(Ordering::Acquire), tick());
    record(Code::SessionEnded, u64::from(accepted), silent);
}

/// Diagnostics poll, every 500 ms: the link edge each time, the RTT window
/// every tenth call while a session is live.
pub(super) fn poll(iteration: u64) {
    let (result, raw) = super::connection_transport::internet_status();
    let (a, b) = pack_link(result, raw);
    let previous = (
        LINK.swap(a, Ordering::AcqRel),
        LINK_RESULT.swap(b, Ordering::AcqRel),
    );
    if previous != (a, b) {
        record(Code::LocalLink, a, b);
    }
    if !iteration.is_multiple_of(10) {
        return;
    }
    let window = Window {
        count: COUNT.swap(0, Ordering::AcqRel),
        sum: SUM.swap(0, Ordering::AcqRel),
        min: MIN.swap(u64::MAX, Ordering::AcqRel),
        max: MAX.swap(0, Ordering::AcqRel),
        max_gap: MAX_GAP.swap(0, Ordering::AcqRel),
    };
    let silent = ms_since(NEWEST.load(Ordering::Acquire), tick());
    if window.count > 0 || silent < ACTIVE_MS {
        let (a, b) = window.pack(silent);
        record(Code::RttWindow, a, b);
    }
}
