//! Startup rendering policies. The hook superset is installed for every mode,
//! including Vanilla. A confirmed window can move among Vanilla, LessLag, and
//! LLUltra. Until that count is known, Vanilla keeps the original index, wait,
//! and flush behavior.
use super::{logging, memory, record, Code};
use crate::config::RenderMode;
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use skyline::patching::Patch;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};

const INDEX: usize = 0x386b0fc;
const WAIT: usize = 0x387029c;
const FLUSH: usize = 0x384fa10;
const INDEX_WORD: u32 = 0x11000929;
const WAIT_WORD: u32 = 0xd63f0100;
static REQUEST: AtomicU8 = AtomicU8::new(0);
static ARMED: AtomicBool = AtomicBool::new(false);
static MODE: AtomicU8 = AtomicU8::new(0);
static STATE: AtomicU8 = AtomicU8::new(0);
pub(super) static INDEX_CALLS: AtomicU64 = AtomicU64::new(0);
pub(super) static INDEX_PAIR: AtomicU64 = AtomicU64::new(0);
pub(super) static SKIPPED_WAITS: AtomicU64 = AtomicU64::new(0);
pub(super) static FLUSH_CALLS: AtomicU64 = AtomicU64::new(0);

pub(super) fn mode() -> RenderMode {
    RenderMode::from_byte(MODE.load(Ordering::Acquire))
}

pub(crate) fn store_request(mode: RenderMode) {
    REQUEST.store(mode as u8, Ordering::Release);
}

pub(super) fn status() -> &'static str {
    match STATE.load(Ordering::Acquire) {
        1 => "Awaiting window",
        2 => crate::render_policy::engine_status(
            mode(),
            super::auto_driver::snapshot().request,
            super::nvn::active_count(),
            crate::render_policy::load_machine(),
        ),
        3 => "Refused; see log",
        _ => "Vanilla",
    }
}

/// Called on the window's initializing thread before its first acquisition.
/// A failed dependency check must leave the actual window at its original count.
pub(super) fn arm_mode(active: i32) {
    let requested = RenderMode::from_byte(REQUEST.load(Ordering::Acquire));
    if active < 0 {
        crate::render_policy::arm(requested);
        MODE.store(RenderMode::Vanilla as u8, Ordering::Release);
        STATE.store(if requested == RenderMode::Vanilla { 2 } else { 3 }, Ordering::Release);
        return;
    }
    if ARMED.load(Ordering::Acquire)
        && active == requested.textures()
        && matches!(active, 2 | 3)
    {
        crate::render_policy::confirm(requested, active);
        MODE.store(requested as u8, Ordering::Release);
        STATE.store(2, Ordering::Release);
    } else if requested != RenderMode::Vanilla {
        crate::render_policy::arm(RenderMode::Vanilla);
        MODE.store(RenderMode::Vanilla as u8, Ordering::Release);
        STATE.store(3, Ordering::Release);
    }
}
pub(super) fn requested_textures() -> Option<i32> {
    if !ARMED.load(Ordering::Acquire) {
        return None;
    }
    let requested = RenderMode::from_byte(REQUEST.load(Ordering::Acquire));
    // Vanilla keeps the game's original 3-texture window. Do not call the setter.
    if requested == RenderMode::Vanilla {
        None
    } else {
        Some(requested.textures())
    }
}

#[skyline::hook(offset = INDEX, inline)]
unsafe fn deluxe_rewrite_select_index(ctx: &mut InlineCtx) {
    let input = ctx.registers[9].w();
    let decision = crate::render_policy::load_decision();
    let armed = if decision.live {
        RenderMode::Vanilla
    } else {
        mode()
    };
    // Vanilla's following instructions perform %3. Preserve add's u32 wrap.
    // A live two-texture policy emits 0 or 1 so that later %3 stays an identity.
    let output = crate::render_policy::index_value(input, decision, armed);
    ctx.registers[9].set_x(output as u64);
    super::api_trace::mark_thread(crate::render_trace::KIND_INDEX, output as i32);
    INDEX_CALLS.fetch_add(1, Ordering::Relaxed);
    INDEX_PAIR.store(((input as u64) << 32) | output as u64, Ordering::Relaxed);
}

#[skyline::hook(offset = WAIT, inline)]
unsafe fn deluxe_rewrite_render_wait(ctx: &mut InlineCtx) {
    let decision = crate::render_policy::load_decision();
    let armed = if decision.live {
        RenderMode::Vanilla
    } else {
        mode()
    };
    let value = if crate::render_policy::skip_wait(decision, armed) {
        SKIPPED_WAITS.fetch_add(1, Ordering::Relaxed);
        0
    } else {
        let f: unsafe extern "C" fn(u64, u64) -> u64 =
            std::mem::transmute(ctx.registers[8].x() as usize);
        f(ctx.registers[0].x(), ctx.registers[1].x())
    };
    ctx.registers[0].set_x(value);
}

#[skyline::hook(offset = FLUSH)]
unsafe fn deluxe_rewrite_flush_dispatches(object: *mut u8, flag: u32) {
    let (calls, fields, arm_drain, generation) =
        if let Some(plan) = crate::render_policy::observe_flush() {
            (
                plan.calls,
                plan.lesslag_fields,
                plan.arm_drain,
                plan.generation,
            )
        } else {
            let current = mode();
            (
                if current.immediate() { 3 } else { 1 },
                current != RenderMode::Vanilla && !current.immediate(),
                false,
                0,
            )
        };
    if fields && !object.is_null() {
        // Sizes independently checked against host ASM: u8, u32, u64, u8.
        // This is the engine-owned object argument, never a guessed global.
        if *object.add(0x1d18) != 0 {
            let previous = *object.add(0x1d20).cast::<u32>();
            *object.add(0x1d30).cast::<u64>() = u64::from((!previous) & 1);
            *object.add(0x1d28) = 1;
        }
    }
    FLUSH_CALLS.fetch_add(1, Ordering::Relaxed);
    let begin = super::api_trace::start();
    for _ in 0..calls {
        original!()(object, flag);
    }
    if arm_drain {
        crate::render_policy::publish_drain(generation);
    }
    super::render_ultra::ORIGINAL_FLUSHES.fetch_add(u64::from(calls), Ordering::Relaxed);
    super::api_trace::end(begin, 5, -1, -1);
}

pub(crate) unsafe fn expect(offset: usize, expected: u32) -> bool {
    let address = getRegionAddress(Region::Text) as usize + offset;
    if !memory::accessible(address, 4, false) {
        record(Code::HookRejected, offset as u64, 0);
        return false;
    }
    let actual = *(address as *const u32);
    record(Code::HookEntryWord, offset as u64, actual as u64);
    logging::install_checkpoint(Code::HookEntryWord, offset as u64, actual as u64);
    if actual != expected {
        record(Code::HookRejected, offset as u64, actual as u64);
        return false;
    }
    true
}

pub(super) fn install() {
    let requested = super::auto_driver::current_request();
    logging::startup_phase("renderer_preflight");
    let signatures = [
        (INDEX, INDEX_WORD),
        (WAIT, WAIT_WORD),
        (FLUSH, 0xa9bc5ff8),
        (FLUSH + 4, 0xa90157f6),
        (FLUSH + 8, 0xa9024ff4),
        (FLUSH + 12, 0xa9037bfd),
        (FLUSH + 16, 0x9100c3fd),
    ];
    if !unsafe {
        signatures
            .iter()
            .all(|&(offset, word)| expect(offset, word))
    } || !super::render_sequence::preflight()
        || !super::render_ultra::preflight()
    {
        STATE.store(3, Ordering::Release);
        return;
    }
    {
        if Patch::in_text(INDEX).nop().is_err() {
            STATE.store(3, Ordering::Release);
            logging::startup_phase("renderer_index_patch_failed");
            return;
        }
        if Patch::in_text(WAIT).nop().is_err() {
            // No hook has been installed yet: restore the first original word.
            if Patch::in_text(INDEX).data(INDEX_WORD).is_err() {
                panic!("renderer patch rollback failed; refusing partially patched pipeline");
            }
            STATE.store(3, Ordering::Release);
            logging::startup_phase("renderer_wait_patch_failed");
            return;
        }
        skyline::install_hooks!(
            deluxe_rewrite_select_index,
            deluxe_rewrite_render_wait,
            deluxe_rewrite_flush_dispatches
        );
    }
    for offset in [INDEX, WAIT, FLUSH] {
        let word = unsafe { *((getRegionAddress(Region::Text) as usize + offset) as *const u32) };
        record(Code::HookInstalled, offset as u64, word as u64);
        let branch = word & 0xfc00_0000 == 0x1400_0000;
        let far_entry = offset == FLUSH && word == 0x5800_0051;
        assert!(
            branch || far_entry,
            "renderer hook did not install at {offset:x}"
        );
    }
    unsafe { super::render_sequence::install() };
    unsafe { super::render_ultra::install() };
    ARMED.store(true, Ordering::Release);
    if requested == RenderMode::Vanilla {
        crate::render_policy::arm(RenderMode::Vanilla);
        MODE.store(RenderMode::Vanilla as u8, Ordering::Release);
        STATE.store(2, Ordering::Release);
    } else {
        STATE.store(1, Ordering::Release);
    }
    logging::startup_phase("renderer_hooks_ready");
}
