//! Inline observers for Pia's already-computed RTT. Nothing is sent.
//!
//! Skyline's inline hook saves every general and SIMD register, calls this
//! callback, restores those registers, then runs the copied instruction.
//! The callback borrows `InlineCtx` and does not write registers. The only
//! pointer read is `this+0x38` while that call is on the stack. The address
//! is kept as an integer identity and is not loaded again.
//!
//! The diagnostic poll is the only publication writer. Hooks update the locked
//! session and do not publish. A missed leave or cleanup only increments the
//! epoch. The UI compares that epoch with the published one and ages a
//! measurement from the sample tick, so a failed poll cannot leave a number
//! up or make an old sample look new.
use super::{record, Code};
use crate::pia_rtt::{
    expire_publication, Observation, Publication, Session, REASON_OFF,
    RTT_ARITHMETIC_GUARD, RTT_ARITHMETIC_OFFSET, RTT_SAMPLE_GUARD, RTT_SAMPLE_OFFSET,
    SESSION_END_GUARD, SESSION_END_OFFSET, STATION_LEAVE_GUARD, STATION_LEAVE_OFFSET,
};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, TryLockError};

const LOCAL_INDEX_OFFSET: usize = 0x38;

static INSTALLED: AtomicBool = AtomicBool::new(false);
static EPOCH: AtomicU64 = AtomicU64::new(1);
/// Bumped around an accepted leave, cleanup, or protocol adoption. Hooks do
/// not write `PUB_*`. The diagnostic poll is the only publication writer.
static DRAW_GEN: AtomicU64 = AtomicU64::new(0);
static PUBLISHED_GEN: AtomicU64 = AtomicU64::new(0);
static STATE: Mutex<Session> = Mutex::new(Session::new());
static PUB_SEQ: AtomicU64 = AtomicU64::new(0);
static PUB_BITS: AtomicU64 = AtomicU64::new((1u64 << 63) | (REASON_OFF as u64));
static PUB_TICK: AtomicU64 = AtomicU64::new(0);
static PUB_PER: AtomicU64 = AtomicU64::new(0);
static PUB_EPOCH: AtomicU64 = AtomicU64::new(1);
static PUB_PEER: AtomicU64 = AtomicU64::new(crate::pia_rtt::NO_PEER);
static SAMPLES: AtomicU64 = AtomicU64::new(0);
static FOREIGN: AtomicU64 = AtomicU64::new(0);
static CLEANUPS: AtomicU64 = AtomicU64::new(0);
static DROPPED: AtomicU64 = AtomicU64::new(0);

/// Last poll's publication, the session epoch, and the two generations.
/// A missed read is an unavailable publication. A generation move during the
/// read is reported as the newer draw generation so the old bytes stay hidden.
pub(super) fn current_publication() -> (Publication, u64, u64, u64) {
    let epoch = EPOCH.load(Ordering::Acquire);
    let published_gen = PUBLISHED_GEN.load(Ordering::Acquire);
    let publication = match shown_publication() {
        Some(publication) => publication,
        None => Publication::unavailable(epoch),
    };
    let draw = DRAW_GEN.load(Ordering::Acquire);
    (publication, epoch, published_gen, draw)
}

/// Step the draw generation around a station-map change. Pair both calls.
fn begin_publication_retirement() {
    DRAW_GEN.fetch_add(1, Ordering::Release);
}

fn end_publication_retirement() {
    DRAW_GEN.fetch_add(1, Ordering::Release);
}

pub(super) fn poll() {
    if !INSTALLED.load(Ordering::Acquire) {
        write_publication(Publication {
            bits: (1u64 << 63) | (REASON_OFF as u64),
            tick: 0,
            ticks_per_ms: 0,
            epoch: EPOCH.load(Ordering::Acquire),
            peer: crate::pia_rtt::NO_PEER,
        });
        return;
    }
    let now = unsafe { skyline::nn::os::GetSystemTick() as u64 };
    let generation = DRAW_GEN.load(Ordering::Acquire);
    let station = station_pick();
    let Ok(mut session) = lock_session() else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
        return;
    };
    let epoch = EPOCH.load(Ordering::Acquire);
    session.accept_poll_for(now, epoch, station);
    store_counters(&session);
    let publication = session.publication();
    drop(session);
    write_publication(publication);
    if DRAW_GEN.load(Ordering::Acquire) == generation {
        PUBLISHED_GEN.store(generation, Ordering::Release);
    }
}

pub(super) fn log_counters() {
    record(
        Code::PiaRtt,
        SAMPLES.load(Ordering::Relaxed),
        CLEANUPS.load(Ordering::Relaxed),
    );
    record(
        Code::PiaRtt,
        FOREIGN.load(Ordering::Relaxed),
        DROPPED.load(Ordering::Relaxed),
    );
}

pub(super) fn install() {
    let guards = [
        (RTT_ARITHMETIC_OFFSET, RTT_ARITHMETIC_GUARD.as_slice()),
        (RTT_SAMPLE_OFFSET, RTT_SAMPLE_GUARD.as_slice()),
        (STATION_LEAVE_OFFSET, STATION_LEAVE_GUARD.as_slice()),
        (SESSION_END_OFFSET, SESSION_END_GUARD.as_slice()),
    ];
    for (offset, expected) in guards {
        if !unsafe { guard_matches(offset, expected) } {
            record(Code::HookRejected, offset as u64, 9);
            skyline::println!(
                "[Deluxe] Pia RTT guard mismatch at {offset:#x}; observer not installed"
            );
            return;
        }
    }
    skyline::install_hooks!(
        deluxe_rewrite_observe_rtt_sample,
        deluxe_rewrite_observe_station_left,
        deluxe_rewrite_observe_session_end
    );
    INSTALLED.store(true, Ordering::Release);
    record(Code::HookInstalled, RTT_SAMPLE_OFFSET as u64, 0);
    skyline::println!("[Deluxe] Pia RTT observer installed");
}

/// Leave and session-end hooks are installed. Guard checks stay in `install`.
pub(super) fn ready() -> bool {
    INSTALLED.load(Ordering::Acquire)
}

fn with_session<T>(update: impl FnOnce(&mut Session, u64) -> T) -> Option<T> {
    let Ok(mut session) = lock_session() else {
        return None;
    };
    let epoch = EPOCH.load(Ordering::Acquire);
    let value = update(&mut session, epoch);
    store_counters(&session);
    Some(value)
}

fn store_counters(session: &Session) {
    SAMPLES.store(session.samples(), Ordering::Relaxed);
    FOREIGN.store(session.foreign_samples(), Ordering::Relaxed);
    CLEANUPS.store(session.cleanups(), Ordering::Relaxed);
}

fn station_pick() -> crate::pia_rtt::StationPick {
    match crate::peer_transport::selection_snapshot() {
        crate::peer_transport::Selection::Station(index) => {
            crate::pia_rtt::StationPick::Index(index)
        }
        crate::peer_transport::Selection::None => crate::pia_rtt::StationPick::Absent,
        crate::peer_transport::Selection::Busy => crate::pia_rtt::StationPick::Busy,
    }
}

/// The RTT lock was busy, so this call cannot say which protocol left.
/// The publication epoch still drops the old ping. An enabled peer observer
/// clears every mapped id instead of keeping a station that might already be gone.
fn lifecycle_missed() {
    EPOCH.fetch_add(1, Ordering::Release);
    DROPPED.fetch_add(1, Ordering::Relaxed);
    super::peer_observer::invalidate_session();
    crate::peer_transport::clear_session();
}

/// One SeqCst publication snapshot, then epoch and age checks. No retry and no write.
fn shown_publication() -> Option<Publication> {
    let published = read_publication()?;
    let now = unsafe { skyline::nn::os::GetSystemTick() as u64 };
    let epoch = EPOCH.load(Ordering::Acquire);
    Some(expire_publication(published, now, epoch))
}

fn lock_session() -> Result<std::sync::MutexGuard<'static, Session>, ()> {
    match STATE.try_lock() {
        Ok(guard) => Ok(guard),
        Err(TryLockError::WouldBlock) => Err(()),
        Err(TryLockError::Poisoned(poisoned)) => Ok(poisoned.into_inner()),
    }
}

fn write_publication(publication: Publication) {
    PUB_SEQ.fetch_add(1, Ordering::SeqCst);
    PUB_BITS.store(publication.bits, Ordering::SeqCst);
    PUB_TICK.store(publication.tick, Ordering::SeqCst);
    PUB_PER.store(publication.ticks_per_ms, Ordering::SeqCst);
    PUB_EPOCH.store(publication.epoch, Ordering::SeqCst);
    PUB_PEER.store(publication.peer, Ordering::SeqCst);
    PUB_SEQ.fetch_add(1, Ordering::SeqCst);
}

fn read_publication() -> Option<Publication> {
    let start = PUB_SEQ.load(Ordering::SeqCst);
    if start & 1 != 0 {
        return None;
    }
    let publication = Publication {
        bits: PUB_BITS.load(Ordering::SeqCst),
        tick: PUB_TICK.load(Ordering::SeqCst),
        ticks_per_ms: PUB_PER.load(Ordering::SeqCst),
        epoch: PUB_EPOCH.load(Ordering::SeqCst),
        peer: PUB_PEER.load(Ordering::SeqCst),
    };
    if PUB_SEQ.load(Ordering::SeqCst) != start {
        return None;
    }
    Some(publication)
}

unsafe fn guard_matches(offset: usize, expected: &[u8]) -> bool {
    let base = getRegionAddress(Region::Text) as usize;
    let end = getRegionAddress(Region::Rodata) as usize;
    let Some(address) = base.checked_add(offset) else {
        return false;
    };
    let Some(stop) = address.checked_add(expected.len()) else {
        return false;
    };
    if address % 4 != 0 || stop > end {
        return false;
    }
    let bytes = std::slice::from_raw_parts(address as *const u8, expected.len());
    bytes == expected
}

#[skyline::hook(offset = RTT_SAMPLE_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_rtt_sample(ctx: &InlineCtx) {
    let protocol = ctx.registers[19].x();
    if protocol < 0x10000 {
        return;
    }
    let local = *((protocol as *const u8).add(LOCAL_INDEX_OFFSET));
    let sample = Observation {
        protocol,
        remote: ctx.registers[20].w(),
        local,
        rtt_ms: ctx.registers[1].x() as i64,
        now_tick: ctx.registers[22].x(),
        ticks_per_ms: ctx.registers[8].x(),
    };
    let station = station_pick();
    match with_session(|session, epoch| session.accept_sample_for(sample, epoch, station)) {
        // Existing path: a stale protocol table was replaced. Drop every
        // mapped id. This does not name the old or new protocol.
        Some(true) => {
            begin_publication_retirement();
            super::peer_observer::invalidate_session();
            crate::peer_transport::clear_session();
            crate::peer_transport::note_local(u32::from(local));
            end_publication_retirement();
        }
        Some(false) => crate::peer_transport::note_local(u32::from(local)),
        None => {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[skyline::hook(offset = STATION_LEAVE_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_station_left(ctx: &InlineCtx) {
    let protocol = ctx.registers[0].x();
    let remote = ctx.registers[1].w();
    let station = station_pick();
    match with_session(|session, epoch| session.accept_leave_for(protocol, remote, epoch, station)) {
        Some(true) => {
            begin_publication_retirement();
            super::peer_observer::invalidate_index(remote);
            crate::peer_transport::clear_station(remote);
            end_publication_retirement();
        }
        Some(false) => {}
        None => lifecycle_missed(),
    }
}

#[skyline::hook(offset = SESSION_END_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_session_end(ctx: &InlineCtx) {
    let protocol = ctx.registers[0].x();
    let station = station_pick();
    match with_session(|session, epoch| session.accept_cleanup_for(protocol, epoch, station)) {
        Some(true) => {
            begin_publication_retirement();
            super::peer_observer::invalidate_session();
            crate::peer_transport::clear_session();
            end_publication_retirement();
        }
        Some(false) => {}
        None => lifecycle_missed(),
    }
}
