//! Chained NVN observations. The active ImGui backend remains in the call path:
//! this NRO's copy when it is the host, or the standalone plugin's copy when
//! that host was selected. This module only wraps the pointer the bootstrap
//! hook returns.
//! Counts and timestamps describe API calls, never scanout or input latency.
use super::DATA_DIR;
use skyline::libc::{c_char, c_void};
use std::{
    ffi::CStr,
    fs::File,
    io::Write,
    path::Path,
    sync::atomic::{AtomicI32, AtomicU64, AtomicUsize, Ordering},
};
type Ptr = *mut c_void;
type Getter = unsafe extern "C" fn(Ptr, *const c_char) -> *const c_void;
static GETTER: AtomicUsize = AtomicUsize::new(0);
static DEVICE: AtomicUsize = AtomicUsize::new(0);
static INITIALIZE: AtomicUsize = AtomicUsize::new(0);
static FINALIZE: AtomicUsize = AtomicUsize::new(0);
static PRESENT: AtomicUsize = AtomicUsize::new(0);
static ACQUIRE: AtomicUsize = AtomicUsize::new(0);
static SYNC_WAIT: AtomicUsize = AtomicUsize::new(0);
static TEXTURE_SYNC: AtomicUsize = AtomicUsize::new(0);
static GET_ACTIVE: AtomicUsize = AtomicUsize::new(0);
static GET_TOTAL: AtomicUsize = AtomicUsize::new(0);
static GET_INTERVAL: AtomicUsize = AtomicUsize::new(0);
static SET_ACTIVE: AtomicUsize = AtomicUsize::new(0);
static WINDOW: AtomicUsize = AtomicUsize::new(0);
static ACTIVE: AtomicI32 = AtomicI32::new(-1);
static TOTAL: AtomicI32 = AtomicI32::new(-1);
static INTERVAL: AtomicI32 = AtomicI32::new(-1);
static PRESENTS: AtomicU64 = AtomicU64::new(0);
static ACQUIRES: AtomicU64 = AtomicU64::new(0);
static PRESENT_MASK: AtomicU64 = AtomicU64::new(0);
static ACQUIRE_MASK: AtomicU64 = AtomicU64::new(0);
static PRESENT_TICK: AtomicU64 = AtomicU64::new(0);
static ACQUIRE_TICK: AtomicU64 = AtomicU64::new(0);

pub(super) fn present_serial() -> u64 {
    PRESENTS.load(Ordering::Relaxed)
}

pub(super) unsafe fn bootstrap(name: *const c_char, next: *const c_void) -> *const c_void {
    if !next.is_null() && CStr::from_ptr(name).to_bytes() == b"nvnDeviceGetProcAddress" {
        GETTER.store(next as usize, Ordering::Release);
        get_proc as *const c_void
    } else {
        next
    }
}
unsafe extern "C" fn get_proc(device: Ptr, name: *const c_char) -> *const c_void {
    let getter: Getter = std::mem::transmute(GETTER.load(Ordering::Acquire));
    let next = getter(device, name);
    if !device.is_null() {
        DEVICE.store(device as usize, Ordering::Release);
    }
    if next.is_null() {
        return next;
    }
    let (slot, replacement) = match CStr::from_ptr(name).to_bytes() {
        b"nvnWindowInitialize" => (&INITIALIZE, window_initialize as *const c_void),
        b"nvnWindowFinalize" => (&FINALIZE, window_finalize as *const c_void),
        b"nvnSyncWait" => (&SYNC_WAIT, sync_wait as *const c_void),
        b"nvnQueuePresentTexture" => (&PRESENT, queue_present as *const c_void),
        b"nvnWindowAcquireTexture" => (&ACQUIRE, window_acquire as *const c_void),
        b"nvnDeviceGetProcAddress" => return get_proc as *const c_void,
        _ => return next,
    };
    slot.store(next as usize, Ordering::Release);
    replacement
}
unsafe fn resolve(name: &'static [u8]) -> usize {
    let getter: Getter = std::mem::transmute(GETTER.load(Ordering::Acquire));
    getter(DEVICE.load(Ordering::Acquire) as Ptr, name.as_ptr().cast()) as usize
}
unsafe fn readback(window: Ptr) {
    for (proc, value) in [
        (&GET_ACTIVE, &ACTIVE),
        (&GET_TOTAL, &TOTAL),
        (&GET_INTERVAL, &INTERVAL),
    ] {
        let address = proc.load(Ordering::Acquire);
        let n = if address == 0 {
            -1
        } else {
            let f: unsafe extern "C" fn(Ptr) -> i32 = std::mem::transmute(address);
            f(window)
        };
        value.store(n, Ordering::Relaxed);
    }
}
unsafe extern "C" fn window_initialize(window: Ptr, builder: *const c_void) -> u8 {
    let f: unsafe extern "C" fn(Ptr, *const c_void) -> u8 =
        std::mem::transmute(INITIALIZE.load(Ordering::Acquire));
    let result = f(window, builder);
    if result != 0 && !window.is_null() {
        if WINDOW
            .compare_exchange(0, window as usize, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            super::record(super::Code::Compatibility, 0x301, window as usize as u64);
            return result; // Only the first game's window belongs to this experiment.
        }
        for (slot, name) in [
            (&GET_ACTIVE, b"nvnWindowGetNumActiveTextures\0".as_slice()),
            (&GET_TOTAL, b"nvnWindowGetNumTextures\0".as_slice()),
            (&GET_INTERVAL, b"nvnWindowGetPresentInterval\0".as_slice()),
            (&SET_ACTIVE, b"nvnWindowSetNumActiveTextures\0".as_slice()),
        ] {
            slot.store(resolve(name), Ordering::Release);
        }
        readback(window);
        if let Some(count) = super::renderer::requested_textures() {
            let setter = SET_ACTIVE.load(Ordering::Acquire);
            if setter != 0
                && DEVICE.load(Ordering::Acquire) != 0
                && GET_ACTIVE.load(Ordering::Acquire) != 0
                && TOTAL.load(Ordering::Relaxed) == 3
                && ACTIVE.load(Ordering::Relaxed) == 3
            {
                let set: unsafe extern "C" fn(Ptr, i32) = std::mem::transmute(setter);
                set(window, count);
                readback(window);
                if ACTIVE.load(Ordering::Relaxed) == count {
                    super::renderer::arm_mode(count);
                } else {
                    set(window, 3);
                    readback(window);
                    // Continuing with an unknown texture ring is not safe.
                    if ACTIVE.load(Ordering::Relaxed) != 3 {
                        panic!("NVN active-texture rollback failed");
                    }
                    super::renderer::arm_mode(-1);
                }
            } else {
                super::renderer::arm_mode(-1);
            }
        }
    }
    result
}
unsafe extern "C" fn window_finalize(window: Ptr) {
    if WINDOW
        .compare_exchange(window as usize, 0, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        super::renderer::arm_mode(-1);
        ACTIVE.store(-1, Ordering::Relaxed);
        TOTAL.store(-1, Ordering::Relaxed);
    }
    let f: unsafe extern "C" fn(Ptr) = std::mem::transmute(FINALIZE.load(Ordering::Acquire));
    f(window);
}
fn mark_index(mask: &AtomicU64, index: i32) {
    if (0..64).contains(&index) {
        mask.fetch_or(1 << index, Ordering::Relaxed);
    }
}
unsafe extern "C" fn queue_present(queue: Ptr, window: Ptr, index: i32) {
    PRESENT_TICK.store(skyline::nn::os::GetSystemTick() as u64, Ordering::Relaxed);
    let n = PRESENTS.fetch_add(1, Ordering::Relaxed) + 1;
    mark_index(&PRESENT_MASK, index);
    let f: unsafe extern "C" fn(Ptr, Ptr, i32) =
        std::mem::transmute(PRESENT.load(Ordering::Acquire));
    let begin = super::api_trace::start();
    f(queue, window, index);
    super::api_trace::end(begin, 1, index, -1);
    // Read objects on their owning render thread, never the logger.
    if n.is_multiple_of(60) && WINDOW.load(Ordering::Acquire) == window as usize {
        readback(window);
    }
}
// The pinned backend's acquire wrapper returns void despite the native API
// returning an enum. Preserve its ABI and do not fabricate a result.
unsafe extern "C" fn window_acquire(window: Ptr, sync: Ptr, index: *mut i32) {
    let f: unsafe extern "C" fn(Ptr, Ptr, *mut i32) =
        std::mem::transmute(ACQUIRE.load(Ordering::Acquire));
    TEXTURE_SYNC.store(sync as usize, Ordering::Release);
    let begin = super::api_trace::start();
    f(window, sync, index);
    super::api_trace::end(begin, 2, if index.is_null() { -1 } else { *index }, -1);
    ACQUIRE_TICK.store(skyline::nn::os::GetSystemTick() as u64, Ordering::Relaxed);
    ACQUIRES.fetch_add(1, Ordering::Relaxed);
    if !index.is_null() {
        mark_index(&ACQUIRE_MASK, *index);
    }
}
/// Diagnostic thread only. Independent cumulative observations, not a snapshot
/// asserting that every value was sampled at the same GPU boundary.
pub(super) fn write_status() {
    if let Ok(mut file) = File::create(Path::new(DATA_DIR).join("renderer-status.txt")) {
        let _ = writeln!(file,
            "build={}\nmetric=nvn_api_observation\nmode={}\nengine_status={}\nindex_hook_calls={}\nindex_pair=0x{:x}\nskipped_render_waits={}\nflush_calls={}\nsequence_frames={}\nmodels_moved={}\nsequence_dispatches={}\nultra_entries={}\nultra_preflushes={}\nultra_late_skips={}\noriginal_flushes={}\nultra_bad_callers={}\nultra_alias_mismatches={}\nultra_last_target=0x{:x}\nwindow=0x{:x}\nget_active_resolved={}\nset_active_resolved={}\nallocated_textures={}\nactive_textures={}\npresent_interval={}\npresent_calls={}\nacquire_calls={}\npresent_index_mask=0x{:x}\nacquire_index_mask=0x{:x}\nlast_present_call_tick={}\nlast_acquire_return_tick={}\nnot_measured=displayed_frames,input_to_photon",
            crate::build_info::BUILD_ID, super::renderer::mode().key(), super::renderer::status(),
            super::renderer::INDEX_CALLS.load(Ordering::Relaxed), super::renderer::INDEX_PAIR.load(Ordering::Relaxed),
            super::renderer::SKIPPED_WAITS.load(Ordering::Relaxed), super::renderer::FLUSH_CALLS.load(Ordering::Relaxed),
            super::render_sequence::FRAMES.load(Ordering::Relaxed), super::render_sequence::MODELS_MOVED.load(Ordering::Relaxed), super::render_sequence::DISPATCHES.load(Ordering::Relaxed),
            super::render_ultra::ENTRIES.load(Ordering::Relaxed), super::render_ultra::PREFLUSHES.load(Ordering::Relaxed), super::render_ultra::LATE_SKIPS.load(Ordering::Relaxed), super::render_ultra::ORIGINAL_FLUSHES.load(Ordering::Relaxed), super::render_ultra::BAD_CALLERS.load(Ordering::Relaxed), super::render_ultra::ALIAS_MISMATCHES.load(Ordering::Relaxed), super::render_ultra::LAST_TARGET.load(Ordering::Relaxed),
            WINDOW.load(Ordering::Acquire),
            GET_ACTIVE.load(Ordering::Acquire) != 0, SET_ACTIVE.load(Ordering::Acquire) != 0,
            TOTAL.load(Ordering::Relaxed), ACTIVE.load(Ordering::Relaxed), INTERVAL.load(Ordering::Relaxed),
            PRESENTS.load(Ordering::Relaxed), ACQUIRES.load(Ordering::Relaxed),
            PRESENT_MASK.load(Ordering::Relaxed), ACQUIRE_MASK.load(Ordering::Relaxed),
            PRESENT_TICK.load(Ordering::Relaxed), ACQUIRE_TICK.load(Ordering::Relaxed));
        let _ = file.sync_all();
    }
}

unsafe extern "C" fn sync_wait(sync: Ptr, timeout: u64) -> i32 {
    let f: unsafe extern "C" fn(Ptr, u64) -> i32 =
        std::mem::transmute(SYNC_WAIT.load(Ordering::Acquire));
    let kind = if timeout == u64::MAX && TEXTURE_SYNC.load(Ordering::Acquire) == sync as usize {
        3
    } else if timeout == 0 {
        4
    } else {
        0
    };
    let begin = if kind != 0 {
        super::api_trace::start()
    } else {
        0
    };
    let result = f(sync, timeout);
    super::api_trace::end(begin, kind, -1, result);
    result
}
