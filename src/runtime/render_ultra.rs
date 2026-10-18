//! Present-entry and late-flush hooks. Installed for every startup mode.
//! Steady Immediate runs the audited early flush and skips the late flush.
//! An index swap clears that skip, so early and late may both run until commit.
//! The caller site at 0x374812c is never patched.
use super::{logging, memory, record, renderer, Code};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

const ENTRY: usize = 0x38701e0;
const LATE_FLUSH: usize = 0x3749b78;
const OWNER_SLOT: usize = 0x5335e90;
const CALLER: usize = 0x374812c;
static IN_PREFLUSH: AtomicBool = AtomicBool::new(false);
pub(super) static ENTRIES: AtomicU64 = AtomicU64::new(0);
pub(super) static PREFLUSHES: AtomicU64 = AtomicU64::new(0);
pub(super) static LATE_SKIPS: AtomicU64 = AtomicU64::new(0);
pub(super) static ORIGINAL_FLUSHES: AtomicU64 = AtomicU64::new(0);
pub(super) static BAD_CALLERS: AtomicU64 = AtomicU64::new(0);
pub(super) static ALIAS_MISMATCHES: AtomicU64 = AtomicU64::new(0);
pub(super) static LAST_TARGET: AtomicU64 = AtomicU64::new(0);

unsafe fn base() -> usize {
    getRegionAddress(Region::Text) as usize
}
type FlushFn = unsafe extern "C" fn(*mut u8, u8);

#[skyline::hook(offset = ENTRY, inline)]
unsafe fn deluxe_rewrite_pre_present(ctx: &InlineCtx) {
    super::frame_delay::present_entry(
        ctx.registers[1].x() as usize,
        ctx.registers[30].x() as usize,
    );
    let decision = crate::render_policy::load_decision();
    let immediate = crate::render_policy::run_early(
        decision,
        if decision.live {
            crate::config::RenderMode::Vanilla
        } else {
            renderer::mode()
        },
    );
    super::api_trace::mark_thread(crate::capture::KIND_PRESENT_ENTRY, i32::from(immediate));
    if !immediate {
        ENTRIES.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let n = ENTRIES.fetch_add(1, Ordering::Relaxed);
    // Skyline armutils saves LR before calling the inline callback. Unlike an
    // ordinary Rust function's live LR, this is the game caller's return address.
    let caller = ctx.registers[30].x() as usize;
    let owner = ctx.registers[1].x() as *mut u8;
    let known_caller = caller == base() + CALLER;
    if !known_caller {
        BAD_CALLERS.fetch_add(1, Ordering::Relaxed);
    }
    assert!(!owner.is_null(), "present owner is null");
    let sub = *owner.add(0x88).cast::<*mut u8>();
    assert!(!sub.is_null(), "present flush subobject is null");
    let table = *sub.cast::<*const usize>();
    assert!(!table.is_null(), "present flush vtable is null");
    let target = *table.add(3);
    LAST_TARGET.store(target as u64, Ordering::Relaxed);

    // Compare the legacy singleton with the live argument without caching it.
    let root = *((base() + OWNER_SLOT) as *const *const u8);
    let alias = if root.is_null() {
        std::ptr::null_mut()
    } else {
        let slot = *root.add(8).cast::<*const *mut u8>();
        if slot.is_null() {
            std::ptr::null_mut()
        } else {
            *slot
        }
    };
    let matches = alias == owner;
    if !matches {
        ALIAS_MISMATCHES.fetch_add(1, Ordering::Relaxed);
    }
    if n == 0 || !known_caller || !matches {
        record(Code::RenderCaller, caller as u64, n);
        record(Code::RenderOwner, owner as u64, alias as u64);
        record(
            Code::RenderFlushTarget,
            target as u64,
            *owner.add(0xec) as u64,
        );
    }
    assert!(
        known_caller && matches,
        "Immediate present route invariant failed"
    );
    assert_eq!(
        target,
        base() + 0x384fa10,
        "Immediate flush target differs from audited function"
    );
    assert!(
        !IN_PREFLUSH.swap(true, Ordering::AcqRel),
        "recursive Immediate preflush"
    );
    let flush: FlushFn = std::mem::transmute(target);
    flush(sub, *owner.add(0xec));
    *owner.add(0xec) = 0;
    PREFLUSHES.fetch_add(1, Ordering::Relaxed);
    IN_PREFLUSH.store(false, Ordering::Release);
    // No manual original call: the inline trampoline restores the original
    // arguments/LR and executes the displaced prologue exactly once.
}

#[skyline::hook(offset = LATE_FLUSH, inline)]
unsafe fn deluxe_rewrite_later_flush(ctx: &InlineCtx) {
    let decision = crate::render_policy::load_decision();
    let skip = crate::render_policy::skip_late(
        decision,
        if decision.live {
            crate::config::RenderMode::Vanilla
        } else {
            renderer::mode()
        },
    );
    super::api_trace::mark_thread(crate::capture::KIND_LATE_FLUSH, i32::from(skip));
    if skip {
        LATE_SKIPS.fetch_add(1, Ordering::Relaxed);
    } else {
        let flush: FlushFn = std::mem::transmute(ctx.registers[8].x() as usize);
        flush(ctx.registers[0].x() as *mut u8, ctx.registers[1].w() as u8);
        *((ctx.registers[20].x() as *mut u8).add(0xec)) = 0;
    }
}

pub(super) fn preflight() -> bool {
    let words = [
        (CALLER - 4, 0x9404a02e),
        (ENTRY, 0xf81b0ff9),
        (ENTRY + 4, 0xa9015ff8),
        (ENTRY + 8, 0xa90257f6),
        (ENTRY + 12, 0xa9034ff4),
        (ENTRY + 16, 0xa9047bfd),
        (LATE_FLUSH, 0xd63f0100),
        (LATE_FLUSH + 4, 0x3903b29f),
    ];
    unsafe {
        words.iter().all(|&(a, w)| renderer::expect(a, w))
            && memory::accessible(base() + OWNER_SLOT, 8, false)
    }
}
pub(super) unsafe fn install() {
    logging::startup_phase("renderer_ultra_install");
    let caller_word = *((base() + CALLER - 4) as *const u32);
    let overlay_word = *((base() + CALLER) as *const u32);
    skyline::patching::Patch::in_text(LATE_FLUSH)
        .nop()
        .expect("late flush patch failed");
    skyline::patching::Patch::in_text(LATE_FLUSH + 4)
        .nop()
        .expect("late flush clear patch failed");
    skyline::install_hook!(deluxe_rewrite_later_flush);
    let installed = *((base() + LATE_FLUSH) as *const u32);
    logging::install_checkpoint(Code::HookInstalled, LATE_FLUSH as u64, installed as u64);
    assert!(
        installed & 0xfc00_0000 == 0x1400_0000,
        "late flush inline hook failed"
    );
    assert_eq!(
        *((base() + LATE_FLUSH + 4) as *const u32),
        0xd503201f,
        "late flush clear was not suppressed"
    );
    skyline::install_hook!(deluxe_rewrite_pre_present);
    let word = *((base() + ENTRY) as *const u32);
    record(Code::HookInstalled, ENTRY as u64, word as u64);
    logging::install_checkpoint(Code::HookInstalled, ENTRY as u64, word as u64);
    assert!(
        word & 0xfc00_0000 == 0x1400_0000,
        "present entry inline hook failed"
    );
    assert_eq!(
        *((base() + CALLER - 4) as *const u32),
        caller_word,
        "present caller changed"
    );
    assert_eq!(
        *((base() + CALLER) as *const u32),
        overlay_word,
        "overlay hook changed"
    );
    logging::startup_phase("renderer_ultra_ready");
}
