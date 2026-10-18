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
    expire_board, expire_publication, Observation, Publication, Session, Shown, StationBoard,
    StationPick, RTT_ARITHMETIC_GUARD, RTT_ARITHMETIC_OFFSET, RTT_SAMPLE_GUARD, RTT_SAMPLE_OFFSET,
    SESSION_END_GUARD, SESSION_END_OFFSET, STATION_LEAVE_GUARD, STATION_LEAVE_OFFSET,
};
use skyline::hooks::InlineCtx;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError, TryLockError};

const LOCAL_INDEX_OFFSET: usize = 0x38;

static INSTALLED: AtomicBool = AtomicBool::new(false);
static EPOCH: AtomicU64 = AtomicU64::new(1);
/// Bumped around an accepted leave, cleanup, or protocol adoption.
static DRAW_GEN: AtomicU64 = AtomicU64::new(0);
/// The station table (sixteen keyed slots, about 7 KB). Hooks and the poll
/// take it with `try_lock`; a busy lock drops the sample or marks the
/// lifecycle event missed.
static STATE: Mutex<Session> = Mutex::new(Session::new());
/// What the UI reads. The poll writes it under `lock`; readers `try_lock`
/// and treat a busy lock as an unavailable publication or board.
static SHOWN: Mutex<Published> = Mutex::new(Published {
    publication: Publication::off(1),
    board: StationBoard::unavailable(1),
    published_gen: 0,
});
static SAMPLES: AtomicU64 = AtomicU64::new(0);
static FOREIGN: AtomicU64 = AtomicU64::new(0);
static CLEANUPS: AtomicU64 = AtomicU64::new(0);
static DROPPED: AtomicU64 = AtomicU64::new(0);
/// Live-peer tracking (0.5.5). `LIVE_STATION` is the selected station the poll
/// last saw, as index + 1 (0 = none). `LIVE_TICK` is the tick of that
/// station's last measured publication, or of its binding (the grace start).
/// `TICKS_PER_MS` is the rate the last RTT sample reported.
static LIVE_STATION: AtomicU64 = AtomicU64::new(0);
static LIVE_TICK: AtomicU64 = AtomicU64::new(0);
static TICKS_PER_MS: AtomicU64 = AtomicU64::new(0);
static WAS_LIVE: AtomicBool = AtomicBool::new(false);

struct Published {
    publication: Publication,
    board: StationBoard,
    published_gen: u64,
}

fn tick() -> u64 {
    unsafe { skyline::nn::os::GetSystemTick() as u64 }
}

fn try_lock<T>(mutex: &'static Mutex<T>) -> Option<MutexGuard<'static, T>> {
    match mutex.try_lock() {
        Ok(guard) => Some(guard),
        Err(TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
        Err(TryLockError::WouldBlock) => None,
    }
}

/// One draw's reading of the last poll, aged to now. A busy lock is an
/// unavailable value under generation 0, which no draw generation shows.
fn read_shown<T>(
    read: impl FnOnce(&Published, u64) -> T,
    unavailable: impl FnOnce(u64) -> T,
) -> Shown<T> {
    let epoch = EPOCH.load(Ordering::Acquire);
    let (value, published_gen) = match try_lock(&SHOWN) {
        Some(shown) => (read(&shown, epoch), shown.published_gen),
        None => (unavailable(epoch), 0),
    };
    Shown {
        value,
        epoch,
        published_gen,
        draw_gen: DRAW_GEN.load(Ordering::Acquire),
    }
}

/// Last poll's publication.
pub(super) fn current_publication() -> Shown<Publication> {
    read_shown(
        |shown, epoch| expire_publication(shown.publication, tick(), epoch),
        Publication::unavailable,
    )
}

/// All-station snapshot from the last poll.
pub(super) fn current_board() -> Shown<StationBoard> {
    read_shown(
        |shown, epoch| expire_board(shown.board, tick(), epoch),
        StationBoard::unavailable,
    )
}

/// Poll-side bookkeeping for the live-peer rule: a newly selected station
/// starts its grace period now; a measured publication renews it.
fn track_liveness(selection: crate::peer_transport::Selection, publication: Publication, now: u64) {
    let bound = match selection {
        crate::peer_transport::Selection::Station(index) => u64::from(index) + 1,
        _ => 0,
    };
    let previous = LIVE_STATION.swap(bound, Ordering::AcqRel);
    if bound == 0 {
        return;
    }
    if previous != bound || publication.measured() {
        LIVE_TICK.store(now, Ordering::Release);
    }
}

/// The live-peer rule (0.5.5) for the selected station: bound, and heard from
/// (a measured RTT publication) within [`crate::pia_rtt::LIVE_MS`]. A station
/// the poll has not seen yet counts as live so a fresh binding shows at once.
/// Records `PeerStale` once when a bound station goes silent.
pub(super) fn station_live(selection: crate::peer_transport::Selection) -> bool {
    let crate::peer_transport::Selection::Station(index) = selection else {
        WAS_LIVE.store(false, Ordering::Release);
        return false;
    };
    if LIVE_STATION.load(Ordering::Acquire) != u64::from(index) + 1 {
        WAS_LIVE.store(true, Ordering::Release);
        return true;
    }
    let per_ms = match TICKS_PER_MS.load(Ordering::Relaxed) {
        0 => crate::pia_rtt::SYSTEM_TICKS_PER_MS,
        rate => rate,
    };
    let last = LIVE_TICK.load(Ordering::Acquire);
    let now = tick();
    let live = crate::pia_rtt::station_live(last, now, per_ms);
    if live {
        WAS_LIVE.store(true, Ordering::Release);
    } else if WAS_LIVE.swap(false, Ordering::AcqRel) {
        let silence_ms = now.saturating_sub(last) / per_ms;
        record(Code::PeerStale, u64::from(index), silence_ms);
    }
    live
}

/// Step the draw generation around a station-map change.
fn retire_publication(change: impl FnOnce()) {
    DRAW_GEN.fetch_add(1, Ordering::Release);
    change();
    DRAW_GEN.fetch_add(1, Ordering::Release);
}

fn shown() -> MutexGuard<'static, Published> {
    SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(super) fn poll() {
    if !INSTALLED.load(Ordering::Acquire) {
        let epoch = EPOCH.load(Ordering::Acquire);
        let mut shown = shown();
        shown.publication = Publication::off(epoch);
        shown.board = StationBoard::unavailable(epoch);
        return;
    }
    let now = tick();
    let generation = DRAW_GEN.load(Ordering::Acquire);
    let (frame, roster) = crate::peer_transport::banner_peers();
    let Some(mut session) = try_lock(&STATE) else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
        if roster.busy {
            shown().board = StationBoard::busy(EPOCH.load(Ordering::Acquire));
        }
        return;
    };
    let epoch = EPOCH.load(Ordering::Acquire);
    session.accept(epoch, station_pick(frame.selection), Some(now), |_| false);
    let board = if roster.busy {
        StationBoard::busy(epoch)
    } else {
        let peers = roster.peers[..usize::from(roster.len)].iter();
        session.board_for(now, epoch, peers.filter_map(|peer| peer.station))
    };
    store_counters(&session);
    let publication = session.publication();
    drop(session);
    track_liveness(frame.selection, publication, now);
    let mut shown = shown();
    shown.publication = publication;
    shown.board = board;
    if DRAW_GEN.load(Ordering::Acquire) == generation {
        shown.published_gen = generation;
    }
}

pub(super) fn log_counters() {
    let counter = |counter: &AtomicU64| counter.load(Ordering::Relaxed);
    record(Code::PiaRtt, counter(&SAMPLES), counter(&CLEANUPS));
    record(Code::PiaRtt, counter(&FOREIGN), counter(&DROPPED));
}

pub(super) fn install() {
    let guards = [
        (RTT_ARITHMETIC_OFFSET, RTT_ARITHMETIC_GUARD.as_slice()),
        (RTT_SAMPLE_OFFSET, RTT_SAMPLE_GUARD.as_slice()),
        (STATION_LEAVE_OFFSET, STATION_LEAVE_GUARD.as_slice()),
        (SESSION_END_OFFSET, SESSION_END_GUARD.as_slice()),
    ];
    for (offset, expected) in guards {
        if !unsafe { super::hooks::guard_matches(offset, expected) } {
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

/// `None` when the lock was busy: the hook cannot say what the table would
/// have done.
fn with_session<T>(update: impl FnOnce(&mut Session, u64) -> T) -> Option<T> {
    let mut session = try_lock(&STATE)?;
    let epoch = EPOCH.load(Ordering::Acquire);
    let value = update(&mut session, epoch);
    store_counters(&session);
    Some(value)
}

fn store_counters(session: &Session) {
    let table = session.table();
    SAMPLES.store(table.samples, Ordering::Relaxed);
    FOREIGN.store(table.foreign_samples, Ordering::Relaxed);
    CLEANUPS.store(table.cleanups, Ordering::Relaxed);
}

fn station_pick(selection: crate::peer_transport::Selection) -> StationPick {
    match selection {
        crate::peer_transport::Selection::Station(index) => StationPick::Index(index),
        crate::peer_transport::Selection::None => StationPick::Absent,
        crate::peer_transport::Selection::Busy => StationPick::Busy,
    }
}

/// The RTT lock was busy, so this call cannot say which protocol left.
/// The publication epoch still drops the old ping, and the transport clears
/// its session rather than keep a station that might already be gone.
fn lifecycle_missed() {
    EPOCH.fetch_add(1, Ordering::Release);
    DROPPED.fetch_add(1, Ordering::Relaxed);
    crate::peer_transport::clear_session();
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
    if sample.ticks_per_ms != 0 {
        TICKS_PER_MS.store(sample.ticks_per_ms, Ordering::Relaxed);
    }
    let station = station_pick(crate::peer_transport::selection_snapshot());
    match with_session(|session, epoch| {
        session.accept(epoch, station, Some(sample.now_tick), |table| {
            table.observe_rtt(sample)
        })
    }) {
        // A stale protocol table was replaced. Drop every mapped id. This
        // does not name the old or new protocol.
        Some(true) => retire_publication(|| {
            crate::peer_transport::clear_session();
            crate::peer_transport::note_local(u32::from(local));
        }),
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
    let station = station_pick(crate::peer_transport::selection_snapshot());
    match with_session(|session, epoch| {
        session.accept(epoch, station, None, |table| {
            table.station_left(protocol, remote)
        })
    }) {
        Some(true) => retire_publication(|| crate::peer_transport::clear_station(remote)),
        Some(false) => {}
        None => lifecycle_missed(),
    }
}

#[skyline::hook(offset = SESSION_END_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_session_end(ctx: &InlineCtx) {
    let protocol = ctx.registers[0].x();
    let station = station_pick(crate::peer_transport::selection_snapshot());
    match with_session(|session, epoch| {
        session.accept(epoch, station, None, |table| table.session_end(protocol))
    }) {
        Some(true) => retire_publication(crate::peer_transport::clear_session),
        Some(false) => {}
        None => lifecycle_missed(),
    }
}
