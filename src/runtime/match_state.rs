//! Runtime half of `crate::match_state` (0.8.2): inline observers. Each
//! callback reads registers (and memory the next instructions read anyway) and
//! records a row; no register, flag or game value is written.
use super::hooks::guard_matches;
use super::{record, Code};
use crate::match_state::{
    begins_sync_wait, id_of, pack_sync_end, DESYNC_GUARD, DESYNC_OFFSET, DESYNC_ROWS,
    PEER_ABORT_GUARD, PEER_ABORT_OFFSET, STATE_COMMAND_GUARD, STATE_COMMAND_OFFSET, SYNC_END_GUARD,
    SYNC_END_OFFSET, SYNC_STATUS_GUARD, SYNC_STATUS_OFFSET, UNKNOWN,
};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicU64, Ordering};

/// Status + 1 of the last reported sync failure; 0 = none since the wait began.
static REPORTED: AtomicU64 = AtomicU64::new(0);
static FAILURES: AtomicU64 = AtomicU64::new(0);
/// Mismatch rows in the current 10 s window and that window's start tick.
static DESYNC_ROWS_USED: AtomicU64 = AtomicU64::new(0);
static DESYNC_WINDOW: AtomicU64 = AtomicU64::new(0);
const DESYNC_WINDOW_TICKS: u64 = 10 * 19_200_000;

fn all_match(sites: &[(usize, &[u8])]) -> bool {
    sites.iter().all(|&(offset, expected)| {
        let matched = unsafe { guard_matches(offset, expected) };
        if !matched {
            record(Code::HookRejected, offset as u64, 12);
            skyline::println!("[Deluxe] match-state guard mismatch at {offset:#x}; not installed");
        }
        matched
    })
}

pub(super) fn install() {
    if all_match(&[
        (STATE_COMMAND_OFFSET, STATE_COMMAND_GUARD.as_slice()),
        (SYNC_STATUS_OFFSET, SYNC_STATUS_GUARD.as_slice()),
    ]) {
        skyline::install_hooks!(
            deluxe_rewrite_observe_state_command,
            deluxe_rewrite_observe_sync_status
        );
        record(Code::HookInstalled, STATE_COMMAND_OFFSET as u64, 0);
    }
    // The lockstep's own end, desync and abort paths, as one family.
    if all_match(&[
        (SYNC_END_OFFSET, SYNC_END_GUARD.as_slice()),
        (DESYNC_OFFSET, DESYNC_GUARD.as_slice()),
        (PEER_ABORT_OFFSET, PEER_ABORT_GUARD.as_slice()),
    ]) {
        skyline::install_hooks!(
            deluxe_rewrite_observe_sync_end,
            deluxe_rewrite_observe_desync,
            deluxe_rewrite_observe_peer_abort
        );
        record(Code::HookInstalled, SYNC_END_OFFSET as u64, 0);
    }
    skyline::println!("[Deluxe] match-state observers checked");
}

fn text_base() -> u64 {
    unsafe { getRegionAddress(Region::Text) as u64 }
}

/// A byte of the lockstep session object the hooked function is reading.
unsafe fn session_byte(session: u64, offset: u64) -> u8 {
    std::ptr::read_volatile((session + offset) as *const u8)
}

unsafe fn session_u16(session: u64, offset: u64) -> u16 {
    std::ptr::read_unaligned((session + offset) as *const u16)
}

#[skyline::hook(offset = STATE_COMMAND_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_state_command(ctx: &InlineCtx) {
    let Some(id) = id_of(ctx.registers[26].x()) else {
        return;
    };
    // x23 is the command `{u32 kind, u64 hash}` the next instruction reads.
    let command = ctx.registers[23].x() as *const u32;
    let kind = if command.is_null() { 0 } else { *command };
    let previous = id_of(ctx.registers[8].x()).map_or(UNKNOWN, u64::from);
    if begins_sync_wait(id) {
        REPORTED.store(0, Ordering::Release);
        FAILURES.store(0, Ordering::Release);
    }
    record(
        Code::MatchState,
        u64::from(id) | u64::from(kind & 0xff) << 8,
        previous,
    );
}

#[skyline::hook(offset = SYNC_STATUS_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_sync_status(ctx: &InlineCtx) {
    let status = u64::from(ctx.registers[0].w() & 0xff);
    let failures = FAILURES.fetch_add(1, Ordering::AcqRel) + 1;
    if REPORTED.swap(status + 1, Ordering::AcqRel) != status + 1 {
        record(Code::SyncCheckFailed, status, failures);
    }
}

/// Entry of `.text+0x16ce7c0(S, kind)`; the function reads these fields next.
#[skyline::hook(offset = SYNC_END_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_sync_end(ctx: &InlineCtx) {
    let session = ctx.registers[0].x();
    if session < 0x10000 {
        return;
    }
    let a = pack_sync_end(
        ctx.registers[1].w(),
        session_byte(session, 0x5f) & 1 != 0,
        session_byte(session, 0x63) != 0,
        session_byte(session, 0x1),
        session_u16(session, 0x2),
    );
    let caller = ctx.registers[30].x().wrapping_sub(text_base());
    record(Code::SyncEnd, a, caller);
}

/// The mismatch branch of `.text+0x16cf1d0`.
#[skyline::hook(offset = DESYNC_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_desync(ctx: &InlineCtx) {
    let now = skyline::nn::os::GetSystemTick() as u64;
    let start = DESYNC_WINDOW.load(Ordering::Acquire);
    if now.wrapping_sub(start) > DESYNC_WINDOW_TICKS {
        DESYNC_WINDOW.store(now, Ordering::Release);
        DESYNC_ROWS_USED.store(0, Ordering::Release);
    }
    if DESYNC_ROWS_USED.fetch_add(1, Ordering::AcqRel) >= DESYNC_ROWS {
        return;
    }
    let record_ptr = ctx.registers[2].x();
    let frame = if record_ptr < 0x10000 {
        0
    } else {
        u64::from(std::ptr::read_unaligned(record_ptr as *const u16))
    };
    let ours = u64::from(ctx.registers[8].w());
    let peer = u64::from(ctx.registers[1].w() & 0xff);
    let theirs = u64::from(ctx.registers[9].w());
    record(Code::DesyncSeen, ours | peer << 32, theirs | frame << 32);
}

/// The type-7 (abort) case of the lockstep control-message dispatcher.
#[skyline::hook(offset = PEER_ABORT_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_peer_abort(ctx: &InlineCtx) {
    let session = ctx.registers[19].x();
    let message = ctx.registers[20].x();
    if session < 0x10000 || message < 0x10000 {
        return;
    }
    let sender = u64::from(session_byte(message, 0) >> 4);
    let reason = u64::from(u16::from_be_bytes([
        session_byte(message, 1),
        session_byte(message, 2),
    ]));
    record(
        Code::PeerAbort,
        sender | reason << 8,
        u64::from(session_u16(session, 0x2)),
    );
}
