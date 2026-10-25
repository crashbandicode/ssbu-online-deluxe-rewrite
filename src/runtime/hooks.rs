//! Source-verified 13.0.5 offsets with conservative coexistence checks.
use super::{record, Code, INPUT_BUFFER};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicBool, Ordering};

const ONLINE_LATENCY: usize = 0x16ccab8;
// 13.0.5 network-update consumer: `ldrb w9, [x19, #0x41]`.
// The calculation hook above writes owner+0x41, then advances x19 to that byte.
// Here x19 is still the owner. Observe only while the game itself owns/reads it.
const ONLINE_BUFFER_READ: usize = 0x16d81a8;
const ONLINE_BUFFER_GUARD: [u8; 20] = [
    0x6a, 0xf2, 0x40, 0x39, // ldrb w10, [x19, #0x3c]
    0x69, 0x06, 0x41, 0x39, // ldrb w9, [x19, #0x41]
    0x68, 0x0a, 0x40, 0x79, // ldrh w8, [x19, #4]
    0x2a, 0x01, 0x00, 0x34, // cbz w10, 0x16d81d4
    0x6a, 0x06, 0x40, 0x79, // ldrh w10, [x19, #2]
];
static LATENCY_INSTALLED: AtomicBool = AtomicBool::new(false);
static BUFFER_OBSERVER_INSTALLED: AtomicBool = AtomicBool::new(false);
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
    if (0..=25).contains(&selected) {
        super::invalidate_manual_auto_buffer();
    }
    let automatic = *pointer;
    let applied = crate::policy::apply_input_buffer(automatic, selected);
    if applied != automatic {
        *pointer = applied;
    }
    record(Code::LatencyApplied, automatic as u64, applied as u64);
}

/// Read-only sampling at the live input-buffer consumer, rather than a retained
/// calculation-time snapshot. The original instruction still performs its read;
/// no registers or game memory are changed and no pointer is retained.
#[skyline::hook(offset = ONLINE_BUFFER_READ, inline)]
unsafe fn deluxe_rewrite_observe_online_buffer(ctx: &InlineCtx) {
    if !BUFFER_ENABLED.load(Ordering::Relaxed) {
        return;
    }
    // Capture the generation before borrowing game memory. A stage transition
    // or manual choice during this callback makes publication fail closed.
    let Some(token) = super::auto_buffer_observation() else {
        return;
    };
    let owner = ctx.registers[19].x() as usize;
    let Some(address) = owner.checked_add(0x41).filter(|_| owner != 0) else {
        return;
    };
    let automatic = std::ptr::read_volatile(address as *const u8);
    if let Some(frames) =
        crate::policy::observed_auto_buffer(automatic, INPUT_BUFFER.load(Ordering::Acquire))
    {
        super::observe_auto_buffer(token, frames);
    }
}

fn install_buffer_observer() {
    if BUFFER_OBSERVER_INSTALLED.load(Ordering::Relaxed) {
        return;
    }
    unsafe {
        if !guard_matches(ONLINE_BUFFER_READ - 4, &ONLINE_BUFFER_GUARD) {
            record(Code::HookRejected, ONLINE_BUFFER_READ as u64, 0);
            return;
        }
        if !preflight(ONLINE_BUFFER_READ) {
            return;
        }
        skyline::install_hook!(deluxe_rewrite_observe_online_buffer);
    }
    BUFFER_OBSERVER_INSTALLED.store(true, Ordering::Relaxed);
    record(Code::HookInstalled, ONLINE_BUFFER_READ as u64, 0);
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
        install_buffer_observer();
    }
}
