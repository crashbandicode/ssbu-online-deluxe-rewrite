//! Startup-only rendering policies. Full LessLag also moves fighter recording.
//! Dynamic resolution and runtime profile transitions remain separate work.
use super::{logging, memory, record, settings, Code};
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
pub(super) fn status() -> &'static str {
    match STATE.load(Ordering::Acquire) {
        1 => "Awaiting window",
        2 if mode().render_opts() => "Active; restart to change",
        2 => "Core only; restart",
        3 => "Refused; see log",
        _ => "Vanilla",
    }
}

/// Called on the window's initializing thread before its first acquisition.
/// A failed dependency check must leave the actual window at its original count.
pub(super) fn arm_mode(active: i32) {
    let requested = RenderMode::from_byte(REQUEST.load(Ordering::Acquire));
    if ARMED.load(Ordering::Acquire) && active == requested.textures() {
        MODE.store(requested as u8, Ordering::Release);
        STATE.store(2, Ordering::Release);
    } else if requested != RenderMode::Vanilla {
        MODE.store(RenderMode::Vanilla as u8, Ordering::Release);
        STATE.store(3, Ordering::Release);
    }
}
pub(super) fn requested_textures() -> Option<i32> {
    ARMED
        .load(Ordering::Acquire)
        .then(|| RenderMode::from_byte(REQUEST.load(Ordering::Acquire)).textures())
}

#[skyline::hook(offset = INDEX, inline)]
unsafe fn deluxe_rewrite_select_index(ctx: &mut InlineCtx) {
    let input = ctx.registers[9].w();
    let current = mode();
    let output = if current == RenderMode::Vanilla {
        // The following original instructions perform %3. Preserve add's u32
        // wrap and zero-extension exactly in the inactive path.
        input.wrapping_add(2)
    } else {
        input.wrapping_add(u32::from(!current.immediate())) % current.textures() as u32
    };
    ctx.registers[9].set_x(output as u64);
    INDEX_CALLS.fetch_add(1, Ordering::Relaxed);
    INDEX_PAIR.store(((input as u64) << 32) | output as u64, Ordering::Relaxed);
}

#[skyline::hook(offset = WAIT, inline)]
unsafe fn deluxe_rewrite_render_wait(ctx: &mut InlineCtx) {
    let value = if mode() == RenderMode::Vanilla {
        let f: unsafe extern "C" fn(u64, u64) -> u64 =
            std::mem::transmute(ctx.registers[8].x() as usize);
        f(ctx.registers[0].x(), ctx.registers[1].x())
    } else {
        SKIPPED_WAITS.fetch_add(1, Ordering::Relaxed);
        0
    };
    ctx.registers[0].set_x(value);
}

#[skyline::hook(offset = FLUSH)]
unsafe fn deluxe_rewrite_flush_dispatches(object: *mut u8, flag: u32) {
    let current = mode();
    if current != RenderMode::Vanilla && !current.immediate() && !object.is_null() {
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
    let calls = if current.immediate() { 3 } else { 1 };
    for _ in 0..calls {
        original!()(object, flag);
    }
    super::render_ultra::ORIGINAL_FLUSHES.fetch_add(calls, Ordering::Relaxed);
    super::api_trace::end(begin, 5, -1, -1);
}

pub(super) unsafe fn expect(offset: usize, expected: u32) -> bool {
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
    if settings().render_mode == RenderMode::Vanilla {
        return; // Vanilla has no rendering text patches.
    }
    REQUEST.store(settings().render_mode as u8, Ordering::Release);
    let sequence = settings().render_mode.render_opts();
    let ultra = super::render_ultra::requested();
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
    } || (sequence && !super::render_sequence::preflight())
        || (ultra && !super::render_ultra::preflight())
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
    if sequence {
        unsafe { super::render_sequence::install() };
    }
    if ultra {
        unsafe { super::render_ultra::install() };
    }
    ARMED.store(true, Ordering::Release);
    STATE.store(1, Ordering::Release);
    logging::startup_phase("renderer_hooks_ready");
}
