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
    expire_publication, Observation, Publication, Session, REASON_OFF, RTT_ARITHMETIC_GUARD,
    RTT_ARITHMETIC_OFFSET, RTT_SAMPLE_GUARD, RTT_SAMPLE_OFFSET, SESSION_END_GUARD,
    SESSION_END_OFFSET, STATION_LEAVE_GUARD, STATION_LEAVE_OFFSET,
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
/// The station table (about 107 KB) exists only after the first RTT sample
/// of the process. The sample hook asks for it through `WANT_SESSION`; the
/// diagnostics poll allocates it (`Session::new_boxed`) on its own thread, so
/// no Pia thread allocates. Until then samples are counted as dropped and the
/// publication is the same "no peer" an empty table gives. It is never freed.
static STATE: Mutex<Option<Box<Session>>> = Mutex::new(None);
static WANT_SESSION: AtomicBool = AtomicBool::new(false);
static PUB_SEQ: AtomicU64 = AtomicU64::new(0);
static PUB_BITS: AtomicU64 = AtomicU64::new((1u64 << 63) | (REASON_OFF as u64));
static PUB_TICK: AtomicU64 = AtomicU64::new(0);
static PUB_PER: AtomicU64 = AtomicU64::new(0);
static PUB_EPOCH: AtomicU64 = AtomicU64::new(1);
static PUB_PEER: AtomicU64 = AtomicU64::new(crate::pia_rtt::NO_PEER);
static BOARD_SEQ: AtomicU64 = AtomicU64::new(0);
static BOARD_LEN: AtomicU64 = AtomicU64::new(0);
static BOARD_EPOCH: AtomicU64 = AtomicU64::new(1);
static BOARD_BITS: [AtomicU64; crate::pia_rtt::BOARD_CAP] = board_zeros();
static BOARD_TICK: [AtomicU64; crate::pia_rtt::BOARD_CAP] = board_zeros();
static BOARD_PER: [AtomicU64; crate::pia_rtt::BOARD_CAP] = board_zeros();
static BOARD_SLOT_EPOCH: [AtomicU64; crate::pia_rtt::BOARD_CAP] = board_zeros();
static BOARD_PEER: [AtomicU64; crate::pia_rtt::BOARD_CAP] = board_zeros();

const fn board_zeros() -> [AtomicU64; crate::pia_rtt::BOARD_CAP] {
    [const { AtomicU64::new(0) }; crate::pia_rtt::BOARD_CAP]
}
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
        let epoch = EPOCH.load(Ordering::Acquire);
        write_publication(Publication {
            bits: (1u64 << 63) | (REASON_OFF as u64),
            tick: 0,
            ticks_per_ms: 0,
            epoch,
            peer: crate::pia_rtt::NO_PEER,
        });
        write_board(crate::pia_rtt::StationBoard::unavailable(epoch));
        return;
    }
    let now = unsafe { skyline::nn::os::GetSystemTick() as u64 };
    let generation = DRAW_GEN.load(Ordering::Acquire);
    let targets = crate::peer_transport::publication_targets();
    let Ok(mut slot) = lock_session() else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
        if targets.busy {
            write_board(crate::pia_rtt::StationBoard::busy(
                EPOCH.load(Ordering::Acquire),
            ));
        }
        return;
    };
    let epoch = EPOCH.load(Ordering::Acquire);
    if slot.is_none() && WANT_SESSION.swap(false, Ordering::AcqRel) {
        *slot = Some(Session::new_boxed());
        record(Code::PiaRtt, u64::MAX, 0);
        skyline::println!("[Deluxe] Pia RTT station table allocated");
    }
    let stations = &targets.stations[..usize::from(targets.len)];
    let (publication, board) = match slot.as_deref_mut() {
        Some(session) => {
            session.accept_poll_for(now, epoch, targets.selected);
            let board = if targets.busy {
                crate::pia_rtt::StationBoard::busy(epoch)
            } else {
                session.board_for(now, epoch, stations)
            };
            store_counters(session);
            (session.publication(), board)
        }
        None => {
            let board = if targets.busy {
                crate::pia_rtt::StationBoard::busy(epoch)
            } else {
                crate::pia_rtt::StationBoard::empty(epoch, stations)
            };
            (Publication::unavailable(epoch), board)
        }
    };
    drop(slot);
    write_publication(publication);
    write_board(board);
    if DRAW_GEN.load(Ordering::Acquire) == generation {
        PUBLISHED_GEN.store(generation, Ordering::Release);
    }
}

/// All-station snapshot from the last poll. A torn read is unavailable.
pub(super) fn current_board() -> (crate::pia_rtt::StationBoard, u64, u64, u64) {
    let epoch = EPOCH.load(Ordering::Acquire);
    let published_gen = PUBLISHED_GEN.load(Ordering::Acquire);
    let board = match shown_board() {
        Some(board) => board,
        None => crate::pia_rtt::StationBoard::unavailable(epoch),
    };
    let draw = DRAW_GEN.load(Ordering::Acquire);
    (board, epoch, published_gen, draw)
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

/// Outcome of a hook's table update.
enum Access<T> {
    Done(T),
    /// The lock was busy. The hook cannot say what the table would have done.
    Busy,
    /// No table has been allocated yet, so there is nothing mapped to update.
    NoSession,
}

fn with_session<T>(update: impl FnOnce(&mut Session, u64) -> T) -> Access<T> {
    let Ok(mut slot) = lock_session() else {
        return Access::Busy;
    };
    let Some(session) = slot.as_deref_mut() else {
        return Access::NoSession;
    };
    let epoch = EPOCH.load(Ordering::Acquire);
    let value = update(session, epoch);
    store_counters(session);
    Access::Done(value)
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
/// The publication epoch still drops the old ping, and the transport clears
/// its session rather than keep a station that might already be gone.
fn lifecycle_missed() {
    EPOCH.fetch_add(1, Ordering::Release);
    DROPPED.fetch_add(1, Ordering::Relaxed);
    crate::peer_transport::clear_session();
}

/// One SeqCst publication snapshot, then epoch and age checks. No retry and no write.
fn shown_publication() -> Option<Publication> {
    let published = read_publication()?;
    let now = unsafe { skyline::nn::os::GetSystemTick() as u64 };
    let epoch = EPOCH.load(Ordering::Acquire);
    Some(expire_publication(published, now, epoch))
}

fn lock_session() -> Result<std::sync::MutexGuard<'static, Option<Box<Session>>>, ()> {
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

fn write_board(board: crate::pia_rtt::StationBoard) {
    BOARD_SEQ.fetch_add(1, Ordering::SeqCst);
    let busy = u64::from(board.busy);
    let len = u64::from(board.len) | (busy << 8);
    BOARD_LEN.store(len, Ordering::SeqCst);
    BOARD_EPOCH.store(board.epoch, Ordering::SeqCst);
    for index in 0..crate::pia_rtt::BOARD_CAP {
        let slot = board.slots[index];
        BOARD_BITS[index].store(slot.bits, Ordering::SeqCst);
        BOARD_TICK[index].store(slot.tick, Ordering::SeqCst);
        BOARD_PER[index].store(slot.ticks_per_ms, Ordering::SeqCst);
        BOARD_SLOT_EPOCH[index].store(slot.epoch, Ordering::SeqCst);
        BOARD_PEER[index].store(slot.peer, Ordering::SeqCst);
    }
    BOARD_SEQ.fetch_add(1, Ordering::SeqCst);
}

fn shown_board() -> Option<crate::pia_rtt::StationBoard> {
    let start = BOARD_SEQ.load(Ordering::SeqCst);
    if start & 1 != 0 {
        return None;
    }
    let meta = BOARD_LEN.load(Ordering::SeqCst);
    let epoch = BOARD_EPOCH.load(Ordering::SeqCst);
    let mut board = crate::pia_rtt::StationBoard::unavailable(epoch);
    board.len = (meta & 0xff) as u8;
    board.busy = meta & 0x100 != 0;
    if usize::from(board.len) > crate::pia_rtt::BOARD_CAP {
        return None;
    }
    for index in 0..crate::pia_rtt::BOARD_CAP {
        board.slots[index] = Publication {
            bits: BOARD_BITS[index].load(Ordering::SeqCst),
            tick: BOARD_TICK[index].load(Ordering::SeqCst),
            ticks_per_ms: BOARD_PER[index].load(Ordering::SeqCst),
            epoch: BOARD_SLOT_EPOCH[index].load(Ordering::SeqCst),
            peer: BOARD_PEER[index].load(Ordering::SeqCst),
        };
    }
    if BOARD_SEQ.load(Ordering::SeqCst) != start {
        return None;
    }
    let now = unsafe { skyline::nn::os::GetSystemTick() as u64 };
    let live = EPOCH.load(Ordering::Acquire);
    Some(crate::pia_rtt::expire_board(board, now, live))
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
        Access::Done(true) => {
            begin_publication_retirement();
            crate::peer_transport::clear_session();
            crate::peer_transport::note_local(u32::from(local));
            end_publication_retirement();
        }
        Access::Done(false) => crate::peer_transport::note_local(u32::from(local)),
        Access::Busy => {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
        // First sample of the process: ask the diagnostics thread for a table.
        // This sample is lost; the next poll (at most 500 ms) allocates.
        Access::NoSession => {
            WANT_SESSION.store(true, Ordering::Release);
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[skyline::hook(offset = STATION_LEAVE_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_station_left(ctx: &InlineCtx) {
    let protocol = ctx.registers[0].x();
    let remote = ctx.registers[1].w();
    let station = station_pick();
    match with_session(|session, epoch| session.accept_leave_for(protocol, remote, epoch, station))
    {
        Access::Done(true) => {
            begin_publication_retirement();
            crate::peer_transport::clear_station(remote);
            end_publication_retirement();
        }
        // Without a table nothing was mapped, so there is nothing to clear.
        Access::Done(false) | Access::NoSession => {}
        Access::Busy => lifecycle_missed(),
    }
}

#[skyline::hook(offset = SESSION_END_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_session_end(ctx: &InlineCtx) {
    let protocol = ctx.registers[0].x();
    let station = station_pick();
    match with_session(|session, epoch| session.accept_cleanup_for(protocol, epoch, station)) {
        Access::Done(true) => {
            begin_publication_retirement();
            crate::peer_transport::clear_session();
            end_publication_retirement();
        }
        Access::Done(false) | Access::NoSession => {}
        Access::Busy => lifecycle_missed(),
    }
}
