//! Source-verified 13.0.5 offsets with conservative coexistence checks.
use super::{record, Code, INPUT_BUFFER};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicBool, Ordering};

const ONLINE_LATENCY: usize = 0x16ccab8;
static LATENCY_INSTALLED: AtomicBool = AtomicBool::new(false);
static BUFFER_ENABLED: AtomicBool = AtomicBool::new(false);

// This detects common trampolines, NOT an original-instruction signature.
pub(super) unsafe fn preflight(offset: usize) -> bool {
    let base = getRegionAddress(Region::Text) as usize;
    let end = getRegionAddress(Region::Rodata) as usize;
    let Some(address) = base.checked_add(offset) else {
        return false;
    };
    if address % 4 != 0 || address.checked_add(20).is_none_or(|v| v > end) {
        record(Code::HookRejected, offset as u64, 1);
        return false;
    }
    let word = *(address as *const u32);
    let branch = word & 0x7c00_0000 == 0x1400_0000;
    let indirect = word & 0xffff_fc1f == 0xd61f_0000;
    let literal = word & 0xff00_0000 == 0x5800_0000;
    let call = word & 0xfc00_0000 == 0x9400_0000;
    if (branch && !call) || indirect || literal || word == 0 {
        record(Code::HookRejected, offset as u64, word as u64);
        return false;
    }
    record(Code::HookInstalling, offset as u64, word as u64);
    true
}

/// True when the 4-byte-aligned `.text+offset` window holds exactly `expected`.
pub(super) unsafe fn guard_matches(offset: usize, expected: &[u8]) -> bool {
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

#[skyline::hook(offset = ONLINE_LATENCY, inline)]
unsafe fn deluxe_rewrite_set_online_latency(ctx: &InlineCtx) {
    super::online::mark_match();
    if !BUFFER_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    let pointer = ctx.registers[19].x() as *mut u8;
    // Retain fork's Quickplay null guard and unrestricted online hook.
    if pointer.is_null() {
        record(Code::NullLatencyPointer, 0, 0);
        return;
    }
    let selected = INPUT_BUFFER.load(Ordering::Acquire);
    let automatic = *pointer;
    let applied = crate::policy::apply_input_buffer(automatic, selected);
    if applied != automatic {
        *pointer = applied;
    }
    record(Code::LatencyApplied, automatic as u64, applied as u64);
}

/// Shared positive online-match signal, independent of buffer mutation.
pub(super) fn install_match_setup() -> bool {
    if LATENCY_INSTALLED.load(Ordering::Relaxed) {
        return true;
    }
    unsafe {
        if !preflight(ONLINE_LATENCY) {
            return false;
        }
        skyline::install_hook!(deluxe_rewrite_set_online_latency);
    }
    LATENCY_INSTALLED.store(true, Ordering::Relaxed);
    record(Code::HookInstalled, ONLINE_LATENCY as u64, 0);
    true
}

pub(super) fn install_latency() {
    if install_match_setup() {
        BUFFER_ENABLED.store(true, Ordering::Relaxed);
    }
}
