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
    sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU64, AtomicU8, AtomicUsize, Ordering},
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
static SET_CROP: AtomicUsize = AtomicUsize::new(0);
static INTERVAL_ZERO_ATTEMPTED: AtomicBool = AtomicBool::new(false);
/// 0 = not checked, 1 = raw setter resolved, 2 = raw setter unavailable.
static INTERVAL_ZERO_AVAILABILITY: AtomicU8 = AtomicU8::new(0);
/// -1 = not requested; zero is the only requested value in this experiment.
static INTERVAL_ZERO_REQUESTED: AtomicI32 = AtomicI32::new(-1);
/// True means the void NVN setter was called; it is not an acknowledgement.
static INTERVAL_ZERO_APPLIED: AtomicBool = AtomicBool::new(false);
/// One-time API-reported getter result after the setter call (or failed lookup).
static INTERVAL_ZERO_READBACK: AtomicI32 = AtomicI32::new(-1);
static WINDOW: AtomicUsize = AtomicUsize::new(0);
static ACTIVE: AtomicI32 = AtomicI32::new(-1);
/// The game's queue, from its first tracked present. Used by `present_at_flush`.
static QUEUE: AtomicUsize = AtomicUsize::new(0);
/// Index returned by the latest tracked acquire and not yet presented early.
static PENDING_INDEX: AtomicI32 = AtomicI32::new(-1);
/// Index this module already presented at the flush; the game's own present
/// of that index is then skipped. -1 when nothing is outstanding.
static EARLY_INDEX: AtomicI32 = AtomicI32::new(-1);
pub(super) static EARLY_PRESENTS: AtomicU64 = AtomicU64::new(0);
pub(super) static EARLY_SKIPS: AtomicU64 = AtomicU64::new(0);
pub(super) static EARLY_MISMATCHES: AtomicU64 = AtomicU64::new(0);

pub(super) fn active_count() -> i32 {
    ACTIVE.load(Ordering::Relaxed)
}
static TOTAL: AtomicI32 = AtomicI32::new(-1);
static INTERVAL: AtomicI32 = AtomicI32::new(-1);
static PRESENTS: AtomicU64 = AtomicU64::new(0);
static ACQUIRES: AtomicU64 = AtomicU64::new(0);
static PRESENT_MASK: AtomicU64 = AtomicU64::new(0);
static ACQUIRE_MASK: AtomicU64 = AtomicU64::new(0);
static PRESENT_TICK: AtomicU64 = AtomicU64::new(0);
static ACQUIRE_TICK: AtomicU64 = AtomicU64::new(0);
static TRACKED_ACQUIRES: AtomicU64 = AtomicU64::new(0);
static TRACKED_PRESENTS: AtomicU64 = AtomicU64::new(0);
static OUTSTANDING: AtomicI64 = AtomicI64::new(0);

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
        b"nvnWindowSetCrop" => {
            // `next` came from the original getter. Never replace a raw proc
            // with this wrapper. `resolve` uses that getter, not `get_proc`.
            remember_raw_crop(next as usize);
            let raw = SET_CROP.load(Ordering::Acquire);
            if raw == 0 || raw == crop_wrapper_address() {
                return next;
            }
            return window_set_crop as *const c_void;
        }
        _ => return next,
    };
    slot.store(next as usize, Ordering::Release);
    replacement
}

pub(super) unsafe fn resolve(name: &'static [u8]) -> usize {
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
            (&SET_CROP, b"nvnWindowSetCrop\0".as_slice()),
        ] {
            let address = resolve(name);
            if std::ptr::eq(slot, &SET_CROP) && address == crop_wrapper_address() {
                continue;
            }
            slot.store(address, Ordering::Release);
        }
        readback(window);
        let _ = super::auto_driver::current_request();
        if let Some(count) = super::renderer::requested_textures() {
            let setter = SET_ACTIVE.load(Ordering::Acquire);
            if setter != 0
                && DEVICE.load(Ordering::Acquire) != 0
                && GET_ACTIVE.load(Ordering::Acquire) != 0
                && TOTAL.load(Ordering::Relaxed) == 3
                && ACTIVE.load(Ordering::Relaxed) == 3
            {
                // A saved 3-texture mode is already the window's active count.
                // Skip that no-op setter. A real 3→2 change still sets, and both
                // paths read the count back before arming.
                if count != TOTAL.load(Ordering::Relaxed) {
                    let set: unsafe extern "C" fn(Ptr, i32) = std::mem::transmute(setter);
                    set(window, count);
                }
                readback(window);
                if ACTIVE.load(Ordering::Relaxed) == count {
                    super::renderer::arm_mode(count);
                } else {
                    let set: unsafe extern "C" fn(Ptr, i32) = std::mem::transmute(setter);
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
        } else if TOTAL.load(Ordering::Relaxed) == 3 && ACTIVE.load(Ordering::Relaxed) == 3 {
            super::renderer::arm_mode(3);
        }
        apply_present_interval_zero(window);
    }
    result
}

unsafe fn apply_present_interval_zero(window: Ptr) {
    if !super::research_present_interval_zero_enabled()
        || INTERVAL_ZERO_ATTEMPTED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
    {
        return;
    }

    INTERVAL_ZERO_REQUESTED.store(0, Ordering::Release);
    let setter = resolve(b"nvnWindowSetPresentInterval\0");
    if setter == 0 {
        INTERVAL_ZERO_AVAILABILITY.store(2, Ordering::Release);
        INTERVAL_ZERO_READBACK.store(INTERVAL.load(Ordering::Relaxed), Ordering::Release);
        return;
    }

    INTERVAL_ZERO_AVAILABILITY.store(1, Ordering::Release);
    let set: unsafe extern "C" fn(Ptr, i32) = std::mem::transmute(setter);
    super::interval_observer::observe(window as usize, setter, false);
    set(window, 0);
    super::interval_observer::observe(window as usize, setter, true);
    INTERVAL_ZERO_APPLIED.store(true, Ordering::Release);
    readback(window);
    INTERVAL_ZERO_READBACK.store(INTERVAL.load(Ordering::Relaxed), Ordering::Release);
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
fn tracked_window(window: Ptr) -> bool {
    !window.is_null() && WINDOW.load(Ordering::Acquire) == window as usize
}
unsafe fn fresh_counts(window: Ptr) -> (i32, i32, bool) {
    let mut active = -1;
    let mut total = TOTAL.load(Ordering::Relaxed);
    let active_fn = GET_ACTIVE.load(Ordering::Acquire);
    if active_fn != 0 {
        let read: unsafe extern "C" fn(Ptr) -> i32 = std::mem::transmute(active_fn);
        active = read(window);
        ACTIVE.store(active, Ordering::Relaxed);
    }
    let total_fn = GET_TOTAL.load(Ordering::Acquire);
    if total_fn != 0 {
        let read: unsafe extern "C" fn(Ptr) -> i32 = std::mem::transmute(total_fn);
        total = read(window);
        TOTAL.store(total, Ordering::Relaxed);
    }
    (active, total, true)
}
fn apply_count(window: Ptr, event: crate::render_policy::StepEvent) {
    if let crate::render_policy::StepEvent::SetTextures { count, generation } = event {
        let setter = SET_ACTIVE.load(Ordering::Acquire);
        if setter == 0 {
            let _ = crate::render_policy::release_unclaimed(generation);
            super::record(
                super::Code::ProfileRefused,
                crate::render_policy::RefuseReason::Unresolved as u64,
                count as u64,
            );
            return;
        }
        note_policy(event);
        unsafe {
            let set: unsafe extern "C" fn(Ptr, i32) = std::mem::transmute(setter);
            set(window, count);
            if !crate::render_policy::setter_returned(generation) {
                panic!("NVN active-texture setter lost its ring generation");
            }
        }
        return;
    }
    note_policy(event);
    if let crate::render_policy::StepEvent::CountStop { seen, expected } = event {
        panic!("NVN active-texture count {seen} after setter, expected {expected}");
    }
}
fn note_policy(event: crate::render_policy::StepEvent) {
    if let Some((raw, a, b)) = crate::render_policy::diagnostic(&event) {
        let code = if raw == crate::render_policy::TRACE_REFUSED {
            super::Code::ProfileRefused
        } else {
            super::Code::ProfileTransition
        };
        super::record(code, a, b);
    }
}
fn mark_index(mask: &AtomicU64, index: i32) {
    if (0..64).contains(&index) {
        mask.fetch_or(1 << index, Ordering::Relaxed);
    }
}
pub(super) fn tracked_presents() -> u64 {
    TRACKED_PRESENTS.load(Ordering::Relaxed)
}

/// Calls the raw `nvnWindowSetCrop` stored from the original device getter.
/// A stored wrapper address is refused so main cannot recurse into the probe.
/// Accepted crops stay on code 80.
pub(super) unsafe fn set_crop(window: Ptr, width: u32, height: u32) -> bool {
    let address = SET_CROP.load(Ordering::Acquire);
    if address == 0 || window.is_null() || address == crop_wrapper_address() {
        return false;
    }
    let crop: unsafe extern "C" fn(Ptr, i32, i32, i32, i32) = std::mem::transmute(address);
    crop(window, 0, 0, width as i32, height as i32);
    true
}

fn crop_wrapper_address() -> usize {
    window_set_crop as *const () as usize
}

fn remember_raw_crop(address: usize) {
    if address == 0 || address == crop_wrapper_address() {
        return;
    }
    SET_CROP.store(address, Ordering::Release);
}

/// Game SetCrop entry. Forwards the original arguments once. A fullscreen
/// reset is reported to the resolution policy, which does not call [`set_crop`].
#[inline(never)]
unsafe extern "C" fn window_set_crop(window: Ptr, x: i32, y: i32, w: i32, h: i32) {
    let address = SET_CROP.load(Ordering::Acquire);
    if address == 0 || address == crop_wrapper_address() {
        return;
    }
    let crop: unsafe extern "C" fn(Ptr, i32, i32, i32, i32) = std::mem::transmute(address);
    crop(window, x, y, w, h);
    let delay =
        crate::resolution::delay_for_profile(crate::render_policy::load_decision().committed);
    let _ = crate::resolution::rearm_fullscreen_reset(
        delay,
        crate::resolution::ObservedCrop {
            tracked: tracked_window(window),
            hooks_ready: crate::resolution::hooks_ready(),
            x,
            y,
            width: w,
            height: h,
        },
    );
}

/// Present the image acquired this frame now, at the end of the flush that
/// submitted it. Called from the renderer's flush hook on the game's render
/// thread when `render_policy::present_at_flush` allows it. The game's own
/// present of the same index, which comes after the next frame's update, is
/// then skipped in `queue_present`; its loop bookkeeping still runs there.
/// The per-present crop moves with the real present so it is applied once.
pub(super) unsafe fn present_at_flush() -> bool {
    let window = WINDOW.load(Ordering::Acquire);
    let queue = QUEUE.load(Ordering::Acquire);
    let present = PRESENT.load(Ordering::Acquire);
    if window == 0 || queue == 0 || present == 0 || EARLY_INDEX.load(Ordering::Acquire) >= 0 {
        return false;
    }
    let index = PENDING_INDEX.swap(-1, Ordering::AcqRel);
    if index < 0 {
        return false;
    }
    let window = window as Ptr;
    super::resolution::on_present(window);
    let f: unsafe extern "C" fn(Ptr, Ptr, i32) = std::mem::transmute(present);
    let begin = super::api_trace::start();
    f(queue as Ptr, window, index);
    super::api_trace::end(begin, 1, index, 2);
    EARLY_INDEX.store(index, Ordering::Release);
    EARLY_PRESENTS.fetch_add(1, Ordering::Relaxed);
    true
}

unsafe extern "C" fn queue_present(queue: Ptr, window: Ptr, index: i32) {
    let tick = skyline::nn::os::GetSystemTick() as u64;
    let tracked = tracked_window(window);
    if tracked {
        QUEUE.store(queue as usize, Ordering::Release);
    }
    // An index already presented at the flush is not presented twice. Any
    // other outstanding early index means the game presented something else:
    // let its call through and count the mismatch.
    let early = EARLY_INDEX.swap(-1, Ordering::AcqRel);
    let presented_early = tracked && early >= 0 && early == index;
    if early >= 0 && !presented_early {
        EARLY_MISMATCHES.fetch_add(1, Ordering::Relaxed);
        super::record(super::Code::EarlyPresent, 2, index as u64);
    }
    if tracked {
        if !presented_early {
            super::resolution::on_present(window);
        }
        super::adaptive_drs::on_present(tick);
        super::dispatcher_backoff::note_present(tick);
        super::hang_trace::count(super::hang_trace::Hook::Present);
    }
    PRESENT_TICK.store(tick, Ordering::Relaxed);
    let n = PRESENTS.fetch_add(1, Ordering::Relaxed) + 1;
    mark_index(&PRESENT_MASK, index);
    let f: unsafe extern "C" fn(Ptr, Ptr, i32) =
        std::mem::transmute(PRESENT.load(Ordering::Acquire));
    let begin = super::api_trace::start();
    if presented_early {
        EARLY_SKIPS.fetch_add(1, Ordering::Relaxed);
        super::api_trace::end(begin, 1, index, 1);
    } else {
        f(queue, window, index);
        super::api_trace::end(begin, 1, index, -1);
    }
    if tracked {
        TRACKED_PRESENTS.fetch_add(1, Ordering::Relaxed);
        let before = OUTSTANDING.fetch_sub(1, Ordering::AcqRel);
        let after = crate::render_policy::outstanding_after_present(before);
        let shown = if after > i64::from(i32::MAX) {
            i32::MAX
        } else if after < i64::from(i32::MIN) {
            i32::MIN
        } else {
            after as i32
        };
        super::api_trace::mark_thread(crate::capture::KIND_OUTSTANDING, shown);
        let saved = super::auto_driver::current_request();
        let snap = crate::render_policy::snapshot();
        let (active, allocated, fresh) = if snap.needs_fresh(saved, index, after) {
            fresh_counts(window)
        } else {
            (
                ACTIVE.load(Ordering::Relaxed),
                TOTAL.load(Ordering::Relaxed),
                false,
            )
        };
        apply_count(
            window,
            crate::render_policy::apply(
                snap,
                crate::render_policy::PresentInput {
                    saved,
                    presented_index: index,
                    outstanding: after,
                    active,
                    allocated,
                    fresh,
                    setter_ready: SET_ACTIVE.load(Ordering::Acquire) != 0,
                },
            ),
        );
    }
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
    let delay_begin = super::frame_delay::clock();
    let begin = super::api_trace::start();
    f(window, sync, index);
    super::frame_delay::acquired(
        delay_begin,
        window as usize,
        tracked_window(window) && !index.is_null() && *index >= 0,
    );
    super::api_trace::end(begin, 2, if index.is_null() { -1 } else { *index }, -1);
    ACQUIRE_TICK.store(skyline::nn::os::GetSystemTick() as u64, Ordering::Relaxed);
    ACQUIRES.fetch_add(1, Ordering::Relaxed);
    if tracked_window(window) {
        TRACKED_ACQUIRES.fetch_add(1, Ordering::Relaxed);
        OUTSTANDING.fetch_add(1, Ordering::AcqRel);
        if !index.is_null() {
            PENDING_INDEX.store(*index, Ordering::Release);
        }
    }
    if !index.is_null() {
        mark_index(&ACQUIRE_MASK, *index);
    }
}
/// Diagnostic thread only. Independent cumulative observations, not a snapshot
/// asserting that every value was sampled at the same GPU boundary.
pub(super) fn write_status() {
    if let Ok(mut file) = File::create(Path::new(DATA_DIR).join("renderer-status.txt")) {
        let policy = crate::render_policy::labels(crate::render_policy::load_machine());
        let auto = super::auto_driver::snapshot();
        let interval_zero_enabled = if super::research_present_interval_zero_enabled() {
            "true"
        } else {
            "false"
        };
        let interval_zero_availability = match INTERVAL_ZERO_AVAILABILITY.load(Ordering::Acquire) {
            1 => "available",
            2 => "unavailable",
            _ => "not_checked",
        };
        let interval_zero_requested = match INTERVAL_ZERO_REQUESTED.load(Ordering::Acquire) {
            0 => "0".to_owned(),
            _ => "not_requested".to_owned(),
        };
        let interval_zero_applied = if INTERVAL_ZERO_APPLIED.load(Ordering::Acquire) {
            "setter_called"
        } else {
            "not_applied"
        };
        let auto_profile = if auto.auto_profile { "true" } else { "false" };
        let cleanup_hold = if auto.cleanup_hold { "true" } else { "false" };
        let scene_ready = if auto.ready { "true" } else { "false" };
        let _ = writeln!(file,
            "build={}\nmetric=nvn_api_observation\nmode={}\nengine_status={}\npolicy_committed={}\npolicy_effective={}\npolicy_requested={}\npolicy_known_active={}\npolicy_count_target={}\npolicy_buffer={}\npolicy_index={}\npolicy_swapping={}\npolicy_failed={}\npolicy_force_wait={}\nselected_profile={}\nauto_profile={}\nauto_requested={}\nplatform={}\nscene_ready={}\nscene_connection={}\nscene_status={}\ncleanup_hold={}\nselection_generation={}\nindex_hook_calls={}\nindex_pair=0x{:x}\nskipped_render_waits={}\nflush_calls={}\nsequence_frames={}\nmodels_moved={}\nsequence_dispatches={}\nteardown_guard_skips={}\nultra_entries={}\nultra_preflushes={}\nultra_late_skips={}\noriginal_flushes={}\nultra_bad_callers={}\nultra_alias_mismatches={}\nultra_last_target=0x{:x}\nwindow=0x{:x}\nget_active_resolved={}\nset_active_resolved={}\nallocated_textures={}\nactive_textures={}\npresent_interval={}\npresent_calls={}\nacquire_calls={}\ntracked_acquires={}\ntracked_presents={}\ntracked_outstanding={}\npresent_index_mask=0x{:x}\nacquire_index_mask=0x{:x}\nlast_present_call_tick={}\nlast_acquire_return_tick={}\nnot_measured=displayed_frames,input_to_photon",
            crate::build_info::BUILD_ID, super::renderer::mode().key(), super::renderer::status(),
            policy.committed, policy.effective, policy.requested, policy.known_active, policy.count_target, policy.buffer, policy.index, policy.swapping, policy.failed, policy.force_wait,
            auto.selected.key(), auto_profile, auto.request.key(), auto.platform.key(), scene_ready, auto.connection.key(),
            crate::auto_policy::scene_status_key(auto.ready, auto.status), cleanup_hold, auto.selection_generation,
            super::renderer::INDEX_CALLS.load(Ordering::Relaxed), super::renderer::INDEX_PAIR.load(Ordering::Relaxed),
            super::renderer::SKIPPED_WAITS.load(Ordering::Relaxed), super::renderer::FLUSH_CALLS.load(Ordering::Relaxed),
            super::render_sequence::FRAMES.load(Ordering::Relaxed), super::render_sequence::MODELS_MOVED.load(Ordering::Relaxed), super::render_sequence::DISPATCHES.load(Ordering::Relaxed),
            super::render_sequence::GUARD_SKIPS.load(Ordering::Relaxed),
            super::render_ultra::ENTRIES.load(Ordering::Relaxed), super::render_ultra::PREFLUSHES.load(Ordering::Relaxed), super::render_ultra::LATE_SKIPS.load(Ordering::Relaxed), super::render_ultra::ORIGINAL_FLUSHES.load(Ordering::Relaxed), super::render_ultra::BAD_CALLERS.load(Ordering::Relaxed), super::render_ultra::ALIAS_MISMATCHES.load(Ordering::Relaxed), super::render_ultra::LAST_TARGET.load(Ordering::Relaxed),
            WINDOW.load(Ordering::Acquire),
            GET_ACTIVE.load(Ordering::Acquire) != 0, SET_ACTIVE.load(Ordering::Acquire) != 0,
            TOTAL.load(Ordering::Relaxed), ACTIVE.load(Ordering::Relaxed), INTERVAL.load(Ordering::Relaxed),
            PRESENTS.load(Ordering::Relaxed), ACQUIRES.load(Ordering::Relaxed),
            TRACKED_ACQUIRES.load(Ordering::Relaxed), TRACKED_PRESENTS.load(Ordering::Relaxed),
            OUTSTANDING.load(Ordering::Relaxed),
            PRESENT_MASK.load(Ordering::Relaxed), ACQUIRE_MASK.load(Ordering::Relaxed),
            PRESENT_TICK.load(Ordering::Relaxed), ACQUIRE_TICK.load(Ordering::Relaxed));
        let _ = writeln!(
            file,
            "research_present_interval_zero_enabled={}\nresearch_present_interval_zero_availability={}\nresearch_present_interval_zero_requested={}\nresearch_present_interval_zero_applied={}\nresearch_present_interval_zero_api_reported_readback={}\nresearch_present_interval_zero_readback_semantics=api_reported_requested_interval_only",
            interval_zero_enabled,
            interval_zero_availability,
            interval_zero_requested,
            interval_zero_applied,
            INTERVAL_ZERO_READBACK.load(Ordering::Acquire),
        );
        let _ = super::interval_observer::write_status(&mut file);
        let _ = super::frame_delay::write_status(&mut file);
        let _ = writeln!(
            file,
            "research_input_trace_requested={}\nresearch_input_trace_state={}",
            if super::input_trace_enabled() {
                "true"
            } else {
                "false"
            },
            super::input_trace::state(),
        );
        let _ = writeln!(
            file,
            "lesslag_fields_skip={}",
            u8::from(super::renderer::lesslag_fields_skipped())
        );
        let _ = writeln!(
            file,
            "{}",
            crate::clock_probe::status_lines(super::clock_probe::mhz(), super::clock_probe::hoc())
        );
        let _ = writeln!(
            file,
            "early_presents={}\nearly_present_skips={}\nearly_present_mismatches={}",
            EARLY_PRESENTS.load(Ordering::Relaxed),
            EARLY_SKIPS.load(Ordering::Relaxed),
            EARLY_MISMATCHES.load(Ordering::Relaxed)
        );
        super::imgui_stats::write_status(&mut file);
        let _ = write!(
            file,
            "{}",
            crate::resolution::format_status(
                crate::resolution::hooks_ready(),
                {
                    let view = super::fighter_main::view();
                    crate::resolution::triggers_available(view.callback_armed, view.count_only)
                },
                crate::resolution::status()
            )
        );
        let settings = super::settings();
        let counters = crate::perf_scaler::global().counters();
        let _ = writeln!(
            file,
            "drs_moves={}\ndrs_adaptive={}\ndrs_adaptive_threshold_ms={}\ndrs_adaptive_trigger_frames={}\ndrs_adaptive_release_frames={}\nadaptive_active={}\nadaptive_observed={}\nadaptive_pushes={}\nadaptive_pops={}\nmove_active={}\nmove_pushes={}\nmove_pops={}",
            settings.drs_moves.serialize(),
            if settings.drs_adaptive.enabled { "true" } else { "false" },
            crate::drs_triggers::format_threshold(settings.drs_adaptive.threshold_hundredths_ms),
            settings.drs_adaptive.trigger_frames,
            settings.drs_adaptive.release_frames,
            if counters.adaptive_active { "true" } else { "false" },
            super::adaptive_drs::observed(),
            counters.adaptive_pushes,
            counters.adaptive_pops,
            counters
                .move_active
                .and_then(|index| crate::drs_triggers::MOVES.get(usize::from(index)))
                .map_or("none", |entry| entry.name),
            counters.move_pushes,
            counters.move_pops,
        );
        let _ = file.sync_all();
    }
}

unsafe extern "C" fn sync_wait(sync: Ptr, timeout: u64) -> i32 {
    let f: unsafe extern "C" fn(Ptr, u64) -> i32 =
        std::mem::transmute(SYNC_WAIT.load(Ordering::Acquire));
    let kind = if timeout == u64::MAX && TEXTURE_SYNC.load(Ordering::Acquire) == sync as usize {
        3
    } else if timeout == u64::MAX {
        crate::capture::KIND_SYNC_INFINITE_OTHER
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
