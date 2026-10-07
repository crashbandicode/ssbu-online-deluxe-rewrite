//! 13.0.5 resolution function, its two 1080-arm inlines, and the four gate NOPs.
//!
//! The game calls `0x37cc130` twice in one pass. The dirty flag calls the
//! original on the first of those and returns on the second. Nothing here
//! adds a post-scene call. The handheld and notification branches are NOP'd
//! first, so the original falls through the 1080 arm; the inlines then replace
//! saved `x8`, `x21`, and `x22`. The 720 arm is not hooked.

use super::{logging, memory, record, Code};
use crate::resolution::{SiteKind, CALLBACK_INLINE, FUNCTION_OFFSET, NOP, OBJECT_INLINE, SITES};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use skyline::patching::Patch;
use std::sync::atomic::{AtomicU64, Ordering};

static PACK: AtomicU64 = AtomicU64::new(0);

fn read_word(offset: usize) -> Option<u32> {
    unsafe {
        let address = getRegionAddress(Region::Text) as usize + offset;
        if !memory::accessible(address, 4, false) {
            return None;
        }
        Some(*(address as *const u32))
    }
}

fn rollback_gates(done: usize) {
    let mut seen = 0usize;
    for site in SITES {
        if site.kind != SiteKind::Gate {
            continue;
        }
        if seen == done {
            break;
        }
        if Patch::in_text(site.offset).data(site.word).is_err() {
            panic!("resolution gate rollback failed at {:x}", site.offset);
        }
        seen += 1;
    }
}

#[skyline::hook(offset = FUNCTION_OFFSET)]
unsafe fn deluxe_rewrite_resolution(object: u64) {
    let Some(level) = crate::resolution::begin_original() else {
        return;
    };
    PACK.store(crate::resolution::canonical_pack(level), Ordering::Release);
    original!()(object);
    crate::resolution::commit_original(level);
}

#[skyline::hook(offset = OBJECT_INLINE, inline)]
unsafe fn deluxe_rewrite_resolution_object(ctx: &mut InlineCtx) {
    let pack = PACK.load(Ordering::Acquire);
    if pack != 0 {
        ctx.registers[8].set_x(crate::resolution::object_x8(pack));
    }
}

#[skyline::hook(offset = CALLBACK_INLINE, inline)]
unsafe fn deluxe_rewrite_resolution_callback(ctx: &mut InlineCtx) {
    let pack = PACK.load(Ordering::Acquire);
    if pack != 0 {
        ctx.registers[21].set_x(crate::resolution::callback_x21(pack));
        ctx.registers[22].set_x(crate::resolution::callback_x22(pack));
    }
}

pub(super) unsafe fn on_present(window: *mut skyline::libc::c_void) {
    super::fighter_main::sync_resolution();
    let delay =
        crate::resolution::delay_for_profile(crate::render_policy::load_decision().committed);
    let Some((width, height)) = crate::resolution::due_crop(delay) else {
        return;
    };
    if super::nvn::set_crop(window, width, height) {
        crate::resolution::note_crop(width, height);
    }
}

pub(super) fn install() {
    logging::startup_phase("resolution_preflight");
    for site in SITES {
        match read_word(site.offset) {
            Some(word) if word == site.word => {
                record(Code::HookEntryWord, site.offset as u64, word as u64);
            }
            Some(word) => {
                record(Code::HookRejected, site.offset as u64, word as u64);
                logging::startup_phase("resolution_guard_rejected");
                return;
            }
            None => {
                record(Code::HookRejected, site.offset as u64, 0);
                logging::startup_phase("resolution_guard_unmapped");
                return;
            }
        }
    }
    let mut patched = 0usize;
    for site in SITES {
        if site.kind != SiteKind::Gate {
            continue;
        }
        record(Code::HookInstalling, site.offset as u64, site.word as u64);
        if Patch::in_text(site.offset).nop().is_err() {
            rollback_gates(patched);
            record(Code::HookRejected, site.offset as u64, u64::from(NOP));
            logging::startup_phase("resolution_gate_patch_failed");
            return;
        }
        patched += 1;
    }
    skyline::install_hooks!(
        deluxe_rewrite_resolution,
        deluxe_rewrite_resolution_object,
        deluxe_rewrite_resolution_callback
    );
    for offset in [FUNCTION_OFFSET, OBJECT_INLINE, CALLBACK_INLINE] {
        let word = read_word(offset).unwrap_or(0);
        assert!(
            crate::resolution::installed_hook_word(offset, word),
            "resolution hook did not install at {offset:x}"
        );
        record(Code::HookInstalled, offset as u64, word as u64);
    }
    for site in SITES {
        if site.kind == SiteKind::Gate {
            record(Code::HookInstalled, site.offset as u64, u64::from(NOP));
        }
    }
    crate::resolution::set_hooks_ready(true);
    logging::startup_phase("resolution_hooks_ready");
}
