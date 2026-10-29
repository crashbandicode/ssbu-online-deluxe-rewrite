//! Runtime half of `crate::match_state` (0.8.2): two inline observers. Each
//! callback reads registers (and the command word the next instruction loads
//! anyway) and records a row; no register, flag or game value is written.
use super::hooks::guard_matches;
use super::{record, Code};
use crate::match_state::{
    begins_sync_wait, id_of, STATE_COMMAND_GUARD, STATE_COMMAND_OFFSET, SYNC_STATUS_GUARD,
    SYNC_STATUS_OFFSET, UNKNOWN,
};
use skyline::hooks::InlineCtx;
use std::sync::atomic::{AtomicU64, Ordering};

/// Status + 1 of the last reported sync failure; 0 = none since the wait began.
static REPORTED: AtomicU64 = AtomicU64::new(0);
static FAILURES: AtomicU64 = AtomicU64::new(0);

pub(super) fn install() {
    for (offset, expected) in [
        (STATE_COMMAND_OFFSET, STATE_COMMAND_GUARD.as_slice()),
        (SYNC_STATUS_OFFSET, SYNC_STATUS_GUARD.as_slice()),
    ] {
        if !unsafe { guard_matches(offset, expected) } {
            record(Code::HookRejected, offset as u64, 12);
            skyline::println!("[Deluxe] match-state guard mismatch at {offset:#x}; not installed");
            return;
        }
    }
    skyline::install_hooks!(
        deluxe_rewrite_observe_state_command,
        deluxe_rewrite_observe_sync_status
    );
    record(Code::HookInstalled, STATE_COMMAND_OFFSET as u64, 0);
    skyline::println!("[Deluxe] match-state observer installed");
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
