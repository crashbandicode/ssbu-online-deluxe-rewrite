//! Fighter rendering is recorded after scene updates, using a per-frame decision.
//! Persistent hooks follow the modern host's ordering; no runtime text rewrites.
use super::{logging, memory, record, Code};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};

const RECORD_FIGHTERS: usize = 0x374bb04;
const DISPATCH: usize = 0x374c600;
const POST_SCENE: usize = 0x374cd64;
const MODELS: usize = 0x3ab590;
const INIT_WORKER: usize = 0x3549720;
const WAIT_WORKER: usize = 0x354ccd0;
const SUBMIT_WORKER: usize = 0x35487f0;
const FIGHTER_WORKER: usize = 0x374f600;
const MANAGER: usize = 0x5333120;
const QUEUE: usize = 0x5333558;
const TLS: usize = 0x5333f58;

static FRAME_ACTIVE: AtomicBool = AtomicBool::new(false);
static MODEL_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
pub(super) static FRAMES: AtomicU64 = AtomicU64::new(0);
pub(super) static MODELS_MOVED: AtomicU64 = AtomicU64::new(0);
pub(super) static DISPATCHES: AtomicU64 = AtomicU64::new(0);

unsafe fn base() -> usize {
    getRegionAddress(Region::Text) as usize
}
type ModelFn = unsafe extern "C" fn(*mut u8);
type InitFn = unsafe extern "C" fn(*mut u8, *const (), u32, i32);
type WaitFn = unsafe extern "C" fn(u32, *mut u32);

unsafe fn initialize(record: *mut u8, function: *const (), a: u32, b: i32) {
    let f: InitFn = std::mem::transmute(base() + INIT_WORKER);
    f(record, function, a, b);
}

unsafe extern "C" fn model_hook(manager: *mut u8) {
    if !FRAME_ACTIVE.load(Ordering::Acquire) {
        update_models(manager);
    }
}
unsafe fn update_models(manager: *mut u8) {
    let address = MODEL_ORIGINAL.load(Ordering::Acquire);
    assert_ne!(address, 0, "model update trampoline unavailable");
    let original: ModelFn = std::mem::transmute(address);
    original(manager);
}

#[skyline::hook(offset = RECORD_FIGHTERS, inline)]
unsafe fn deluxe_rewrite_record_fighters(ctx: &InlineCtx) {
    let active = super::renderer::mode().render_opts();
    FRAME_ACTIVE.store(active, Ordering::Release);
    if !active {
        // Replaces precisely the original BL; the four ABI arguments survive.
        initialize(
            ctx.registers[0].x() as *mut u8,
            ctx.registers[1].x() as *const (),
            ctx.registers[2].w(),
            ctx.registers[3].w() as i32,
        );
    }
}

#[skyline::hook(offset = DISPATCH, inline)]
unsafe fn deluxe_rewrite_defer_dispatch(ctx: &mut InlineCtx) {
    if FRAME_ACTIVE.load(Ordering::Acquire) {
        // The displaced CMP and following BEQ remain intact.
        ctx.registers[8].set_x(1);
    }
}

unsafe fn submit_queue() {
    let queue = (base() + QUEUE) as *mut u8;
    // The original game reads +0x20. The archive's *mut ().add(0x20)
    // accidentally read +0 because () has zero size. Use the game semantics.
    if *queue.add(0x20) == 1 {
        return;
    }
    let busy = &*queue.add(8).cast::<AtomicU8>();
    // Match the original LDAXRB/STXRB exchange loop: bit0 is the lock.
    // A plain store (as in the archive) would bypass its mutual exclusion.
    while busy.swap(1, Ordering::Acquire) & 1 != 0 {
        std::hint::spin_loop();
    }
    let mut link = queue.cast::<*mut u8>();
    let mut node = *link;
    while !node.is_null() {
        let next = *node.add(0x48).cast::<*mut u8>();
        match *node.add(0x70) {
            2 => {
                *link = next;
                let vtable = *node.cast::<*const usize>();
                let destroy: unsafe extern "C" fn(*mut u8) = std::mem::transmute(*vtable.add(1));
                destroy(node);
                node = next;
                continue;
            }
            1 => {}
            _ => initialize(
                node.add(8),
                (base() + SUBMIT_WORKER) as *const (),
                *node.add(0x68).cast::<u32>(),
                *node.add(0x6c).cast::<i32>(),
            ),
        }
        link = node.add(0x48).cast();
        node = next;
    }
    busy.store(0, Ordering::Release);
    DISPATCHES.fetch_add(1, Ordering::Relaxed);
}

#[skyline::hook(offset = POST_SCENE, inline)]
unsafe fn deluxe_rewrite_post_scene(_: &InlineCtx) {
    if !FRAME_ACTIVE.load(Ordering::Acquire) {
        return;
    }
    let manager = *((base() + MANAGER) as *const *mut u8);
    if !manager.is_null() {
        update_models(manager);
        MODELS_MOVED.fetch_add(1, Ordering::Relaxed);
    }
    // The game's worker record is 64 bytes. It must outlive its awaited task.
    let mut worker = [0u32; 16];
    initialize(
        worker.as_mut_ptr().cast(),
        (base() + FIGHTER_WORKER) as *const (),
        0,
        0,
    );
    if worker[4] != 0 {
        let slots = (base() + TLS) as *const *const u8;
        let slot = *slots.add(worker[5] as usize);
        let wait: WaitFn = std::mem::transmute(base() + WAIT_WORKER);
        wait(*slot.add(0x10).cast::<u32>(), worker.as_mut_ptr().add(4));
    }
    // No dynamic-resolution subsystem is installed. Preserve the game's
    // original resolution updates rather than adding the host's DRS callback.
    submit_queue();
    FRAMES.fetch_add(1, Ordering::Relaxed);
}

pub(super) fn preflight() -> bool {
    let signatures = [
        (RECORD_FIGHTERS, 0x97f7f707),
        (0x374baf0, 0x90000021), // original fighter function address
        (0x374baf4, 0x91180021),
        (DISPATCH, 0x7100051f),
        (DISPATCH + 4, 0x54000520),
        (POST_SCENE, 0xb000df68),
        (MODELS, 0xd10343ff),
        (MODELS + 4, 0xa9076ffc),
        (MODELS + 8, 0xa90867fa),
        (MODELS + 12, 0xa9095ff8),
        (MODELS + 16, 0xa90a57f6),
        (INIT_WORKER, 0xa9bb67fa),
        (WAIT_WORKER, 0xd10183ff),
        (SUBMIT_WORKER, 0xa9bd57f6),
    ];
    unsafe {
        signatures
            .iter()
            .all(|&(offset, word)| super::renderer::expect(offset, word))
            && [MANAGER, QUEUE, TLS]
                .iter()
                .all(|offset| memory::accessible(base() + offset, 0x28, false))
    }
}

/// Called only after every family signature passes, while mode is still Vanilla.
pub(super) unsafe fn install() {
    logging::startup_phase("renderer_sequence_install");
    skyline::patching::Patch::in_text(RECORD_FIGHTERS)
        .nop()
        .expect("fighter recording patch failed");
    let mut original: *mut c_void = std::ptr::null_mut();
    skyline::hooks::A64HookFunction(
        (base() + MODELS) as *const c_void,
        model_hook as *const c_void,
        &mut original,
    );
    assert!(
        !original.is_null(),
        "model update hook returned no trampoline"
    );
    MODEL_ORIGINAL.store(original as usize, Ordering::Release);
    skyline::install_hooks!(
        deluxe_rewrite_record_fighters,
        deluxe_rewrite_defer_dispatch,
        deluxe_rewrite_post_scene
    );
    for offset in [MODELS, RECORD_FIGHTERS, DISPATCH, POST_SCENE] {
        let word = *((base() + offset) as *const u32);
        record(Code::HookInstalled, offset as u64, word as u64);
        logging::install_checkpoint(Code::HookInstalled, offset as u64, word as u64);
        assert!(
            word & 0xfc00_0000 == 0x1400_0000 || word == 0x5800_0051,
            "sequence hook did not install at {offset:x}"
        );
    }
    logging::startup_phase("renderer_sequence_ready");
}
