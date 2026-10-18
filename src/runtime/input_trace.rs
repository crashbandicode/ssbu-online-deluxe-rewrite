//! Startup-only, opt-in input-manager/scene trace points. These hooks only
//! record finite API-trace points while a capture is active; they do not read
//! or change gameplay input except for the separately guarded cache diagnostic.
use super::{api_trace, memory, record, Code};
use crate::capture::{
    KIND_HID_AFTER, KIND_HID_BEFORE, KIND_INPUT_CACHE_STATE, KIND_SCENE_AFTER, KIND_SCENE_BEFORE,
};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicU8, Ordering};

const HID_BEFORE: usize = 0x374cab8;
const HID_AFTER: usize = 0x374cabc;
const SCENE_BEFORE: usize = 0x374cbd4;
const SCENE_AFTER: usize = 0x374cbd8;
const INPUT_CACHE_OWNER: usize = 0x52c50f0;

// Skyline keeps its inline-handler pool adjacent to main text. These offsets
// are within the A64 B range, so each hook replaces one word; neighboring
// before/after sites do not overlap.
const SITES: [(usize, u32); 4] = [
    (HID_BEFORE, 0x97fc6c76),   // BL 0x3667c90 is displaced by the inline hook.
    (HID_AFTER, 0x94004959),    // BL 0x375f020.
    (SCENE_BEFORE, 0x97ff6117), // BL 0x3725030.
    (SCENE_AFTER, 0xf0010f68),  // ADRP.
];

const OFF: u8 = 0;
const PREFLIGHT_FAILED: u8 = 1;
const INSTALLING: u8 = 2;
const INSTALLED: u8 = 3;
static STATE: AtomicU8 = AtomicU8::new(OFF);

/// State for the startup diagnostics file. The request value is reported by
/// the configuration/status path separately.
pub(super) fn state() -> &'static str {
    match STATE.load(Ordering::Acquire) {
        OFF => "off",
        PREFLIGHT_FAILED => "preflight_failed",
        INSTALLING => "installing",
        INSTALLED => "installed",
        _ => "unknown",
    }
}

#[skyline::hook(offset = HID_BEFORE, inline)]
unsafe fn deluxe_rewrite_input_hid_before(_: &InlineCtx) {
    api_trace::mark_thread(KIND_HID_BEFORE, -1);
}

#[skyline::hook(offset = HID_AFTER, inline)]
unsafe fn deluxe_rewrite_input_hid_after(_: &InlineCtx) {
    api_trace::mark_thread(KIND_HID_AFTER, -1);
}

/// A short-lived, guarded read of the current input cache metadata. Neither
/// the owner pointer nor any field pointer escapes this call.
unsafe fn input_cache_state() -> Option<(i32, i32)> {
    let text = getRegionAddress(Region::Text) as usize;
    let global = text.checked_add(INPUT_CACHE_OWNER)?;
    if global % core::mem::align_of::<usize>() != 0
        || !memory::accessible(global, core::mem::size_of::<usize>(), false)
    {
        return None;
    }
    let owner = core::ptr::read_volatile(global as *const usize);
    if owner < 0x10000 || owner % core::mem::align_of::<u64>() != 0 {
        return None;
    }

    let mode_address = owner.checked_add(8)?;
    let flags_address = owner.checked_add(0x580)?;
    if mode_address % core::mem::align_of::<i32>() != 0
        || !memory::accessible(mode_address, core::mem::size_of::<i32>(), false)
        || !memory::accessible(flags_address, 8, false)
    {
        return None;
    }

    let mode = core::ptr::read_volatile(mode_address as *const i32);
    let mut mask = 0i32;
    for bit in 0..8 {
        if core::ptr::read_volatile((flags_address + bit) as *const u8) != 0 {
            mask |= 1 << bit;
        }
    }
    Some((mode, mask))
}

#[skyline::hook(offset = SCENE_BEFORE, inline)]
unsafe fn deluxe_rewrite_input_scene_before(_: &InlineCtx) {
    let begin = api_trace::start();
    if begin == 0 {
        return;
    }
    api_trace::point(begin, KIND_SCENE_BEFORE, -1, api_trace::thread_label());

    // The cache fields are sampled only during an active trace, and unknown
    // or inaccessible state simply omits kind 84 without affecting kind 82.
    if api_trace::active() {
        if let Some((mode, overrides)) = input_cache_state() {
            let sample_tick = api_trace::start();
            api_trace::point(sample_tick, KIND_INPUT_CACHE_STATE, mode, overrides);
        }
    }
}

#[skyline::hook(offset = SCENE_AFTER, inline)]
unsafe fn deluxe_rewrite_input_scene_after(_: &InlineCtx) {
    api_trace::mark_thread(KIND_SCENE_AFTER, -1);
}

/// Install only when the startup-only INI request is true. All four source
/// words are checked before any of this module's hooks are installed.
pub(super) fn install() {
    if !super::input_trace_enabled() {
        STATE.store(OFF, Ordering::Release);
        return;
    }

    super::logging::startup_phase("input_trace_preflight");
    let mut valid = true;
    unsafe {
        for &(offset, word) in &SITES {
            // Keep checking later sites even after a mismatch so the complete
            // opt-in preflight is logged before the fail-closed decision.
            let site_valid = super::renderer::expect(offset, word);
            valid = site_valid & valid;
        }
    }
    if !valid {
        STATE.store(PREFLIGHT_FAILED, Ordering::Release);
        super::logging::startup_phase("input_trace_preflight_failed");
        return;
    }

    STATE.store(INSTALLING, Ordering::Release);
    unsafe {
        skyline::install_hooks!(
            deluxe_rewrite_input_hid_before,
            deluxe_rewrite_input_hid_after,
            deluxe_rewrite_input_scene_before,
            deluxe_rewrite_input_scene_after
        );
    }

    let text = unsafe { getRegionAddress(Region::Text) as usize };
    for (offset, _) in SITES {
        let word = unsafe { core::ptr::read_volatile((text + offset) as *const u32) };
        record(Code::HookInstalled, offset as u64, word as u64);
        let branch = word & 0xfc00_0000 == 0x1400_0000;
        assert!(
            branch,
            "input trace inline hook did not install at {offset:x}"
        );
    }
    STATE.store(INSTALLED, Ordering::Release);
    super::logging::startup_phase("input_trace_hooks_ready");
}
