//! Dispatcher idle backoff: one inline hook at the point where the game's task
//! dispatcher has found all six queues empty and is about to rescan its
//! pending dependencies. When every pending record is still blocked, the
//! main thread sleeps briefly instead of spinning through six more
//! lock/unlock pairs. Nothing else changes: the game's own scan runs
//! unchanged after the hook returns, so no task, counter or lock is skipped.
//!
//! Decision logic lives in [`crate::dispatcher_backoff`] and is host-tested.
//! This file only reads the dispatcher and sleeps. Layout facts come from the
//! 13.0.5 source audit in the tangent evidence (`css-budget/astra`,
//! `CSS_SLOW_FRAME_ATTRIBUTION.md` "Production contract").
//!
//! 0.3.1 adds an eligibility gate (no sleep before a scene hook has fired and
//! while the tracked window is not presenting) and per-frame sleep aggregates
//! for the CSS capture (`css-budget/hand/CSS_CAPTURE_CONTRACT.md`).

use super::{memory, record, Code};
use crate::dispatcher_backoff::{decide, Policy, Snapshot, Streak};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering};

/// `main+0x354ab48`: `ldp x8, x9, [x24, #0xe0]`, the first instruction after
/// the sixth queue probe fell through without a job and released its lock.
/// x24 is the dispatcher.
const SITE: usize = 0x354ab48;
/// The five original words from `SITE` (13.0.5): `ldp x8, x9, [x24, #0xe0]`;
/// `cmp x8, x9`; `b.ne` to the pending rescan; `ldr x0, [sp]`;
/// `bl nn::os::WaitLightEvent`. All must match before the hook is published;
/// a changed first word with the rest intact means another plugin already
/// owns this site.
const SITE_WORDS: [u32; 5] = [
    0xa94e_2708,
    0xeb09_011f,
    0x5400_0d01,
    0xf940_03e0,
    0x9411_f072,
];
/// Pending-record vector `[begin, end)` at these dispatcher offsets, stride
/// 0x20; `record + 8` holds the 32-bit dependency counter pointer.
const PENDING_BEGIN: usize = 0xe0;
const PENDING_END: usize = 0xe8;
const PENDING_STRIDE: usize = 0x20;
const PENDING_MAX: usize = 4096;
/// Ready-to-resume fiber vector the game checks before polling; `(begin, end)`
/// dispatcher offsets. Non-empty means a fiber is ready to resume.
const RESUME_VECTORS: [(usize, usize); 1] = [(0xf8, 0x100)];
const LOCAL_SIZES: [usize; 3] = [0x38, 0x68, 0x98];
const SHARED_SIZES: [usize; 3] = [0x28, 0x58, 0x88];
const CONTEXT_ENABLED: usize = 0x58;
const DISPATCHER_SPAN: usize = 0x158;
const CONTEXT_SPAN: usize = 0x60;
const SHARED_SPAN: usize = 0xd0;
/// `nn::os::ThreadType + 0x1a0` holds the thread-name pointer
/// (`nn::os::GetThreadNamePointer` is `ldr x0, [x0, #0x1a0]; ret`).
const THREAD_NAME_POINTER: usize = 0x1a0;
const MAIN_THREAD_NAME: &[u8] = b"MainThread\0";
const SLOTS: usize = 8;

/// A tracked present older than this means the renderer is not in its steady
/// loop (startup, window re-creation, scene teardown): no sleep until it is.
const PRESENT_STALE_TICKS: u64 = crate::timing::TICK_HZ / 4;

static INSTALLED: AtomicBool = AtomicBool::new(false);
/// 0 unknown, 1 installed, 2 external owner, 3 words differ, 4 disabled.
static OUTCOME: AtomicU8 = AtomicU8::new(0);
/// Eligibility gate (0.3.1). Both must hold before the first sleep: a scene
/// hook has fired since boot (the game is past its initialization and on a
/// real scene), and the tracked window presented within the last 250 ms.
/// Startup, the logo/movie/title sequence and a stalled renderer never sleep.
static SCENE_SEEN: AtomicBool = AtomicBool::new(false);
static LAST_PRESENT_TICK: AtomicU64 = AtomicU64::new(0);
/// Latched the first time the gate passed; shown as `Armed` until then.
static ELIGIBLE: AtomicBool = AtomicBool::new(false);
static GATED_SKIPS: AtomicU64 = AtomicU64::new(0);
/// Per-frame aggregates for the CSS capture, swapped out at each tracked
/// present. Sleep ticks are measured only while a finite API trace is active.
static FRAME_SLEEPS: AtomicU64 = AtomicU64::new(0);
static FRAME_SLEEP_TICKS: AtomicU64 = AtomicU64::new(0);
static FRAME_MAX_SLEEP_TICKS: AtomicU64 = AtomicU64::new(0);
static FRAME_STREAKS: AtomicU64 = AtomicU64::new(0);
static SPIN_PASSES: AtomicU32 = AtomicU32::new(Policy::DEFAULT.spin_passes);
static EVERY_PASSES: AtomicU32 = AtomicU32::new(Policy::DEFAULT.every_passes);
static BASE_NS: AtomicU32 = AtomicU32::new(Policy::DEFAULT.base_ns);
static MAX_NS: AtomicU32 = AtomicU32::new(Policy::DEFAULT.max_ns);
static COOLDOWN_US: AtomicU32 = AtomicU32::new(Policy::DEFAULT.cooldown_us);
static COOLDOWN_TICKS: AtomicU64 = AtomicU64::new(0);
static MAIN_THREAD_ONLY: AtomicBool = AtomicBool::new(Policy::DEFAULT.main_thread_only);
/// The `ThreadType` whose SDK name is `MainThread`, once seen at the site.
static MAIN_THREAD: AtomicUsize = AtomicUsize::new(0);
static IDLE_PASSES: AtomicU64 = AtomicU64::new(0);
static SLEEPS: AtomicU64 = AtomicU64::new(0);
static COOLDOWN_SKIPS: AtomicU64 = AtomicU64::new(0);
static LAST_NS: AtomicU32 = AtomicU32::new(0);
static LAST_SLEEP_END: AtomicU64 = AtomicU64::new(0);
static RATE: AtomicU32 = AtomicU32::new(0);
static PUBLISHED_SLEEPS: AtomicU64 = AtomicU64::new(0);
static PUBLISH_CALLS: AtomicU64 = AtomicU64::new(0);
/// Dispatcher pointers seen so far and their packed streaks
/// (`idle_passes | last_ns << 32`). A pointer is validated once on first sight.
static KEYS: [AtomicUsize; SLOTS] = [const { AtomicUsize::new(0) }; SLOTS];
static STREAKS: [AtomicU64; SLOTS] = [const { AtomicU64::new(0) }; SLOTS];

fn policy() -> Policy {
    Policy {
        spin_passes: SPIN_PASSES.load(Ordering::Relaxed),
        every_passes: EVERY_PASSES.load(Ordering::Relaxed),
        base_ns: BASE_NS.load(Ordering::Relaxed),
        max_ns: MAX_NS.load(Ordering::Relaxed),
        cooldown_us: COOLDOWN_US.load(Ordering::Relaxed),
        main_thread_only: MAIN_THREAD_ONLY.load(Ordering::Relaxed),
    }
}

fn store_policy(policy: Policy) {
    SPIN_PASSES.store(policy.spin_passes, Ordering::Relaxed);
    EVERY_PASSES.store(policy.every_passes, Ordering::Relaxed);
    BASE_NS.store(policy.base_ns, Ordering::Relaxed);
    MAX_NS.store(policy.max_ns, Ordering::Relaxed);
    COOLDOWN_US.store(policy.cooldown_us, Ordering::Relaxed);
    COOLDOWN_TICKS.store(policy.cooldown_ticks(), Ordering::Relaxed);
    MAIN_THREAD_ONLY.store(policy.main_thread_only, Ordering::Relaxed);
}

/// Optional `dispatcher_backoff.ini` in the data directory, read once.
fn load_policy() -> Policy {
    let path = std::path::Path::new(super::DATA_DIR).join("dispatcher_backoff.ini");
    match std::fs::read_to_string(path) {
        Ok(text) => Policy::parse(&text, Policy::DEFAULT),
        Err(_) => Policy::DEFAULT,
    }
}

/// The calling thread is the SDK `MainThread`. Its `ThreadType` pointer is
/// resolved once by bounded name comparison and then compared directly.
unsafe fn on_main_thread() -> bool {
    let thread = skyline::nn::os::GetCurrentThread() as usize;
    if thread == 0 {
        return false;
    }
    let known = MAIN_THREAD.load(Ordering::Relaxed);
    if known != 0 {
        return known == thread;
    }
    if !memory::accessible(thread + THREAD_NAME_POINTER, 8, false) {
        return false;
    }
    let name = *((thread + THREAD_NAME_POINTER) as *const usize);
    if name == 0 || !memory::accessible(name, MAIN_THREAD_NAME.len(), false) {
        return false;
    }
    if core::slice::from_raw_parts(name as *const u8, MAIN_THREAD_NAME.len()) != MAIN_THREAD_NAME {
        return false;
    }
    MAIN_THREAD.store(thread, Ordering::Relaxed);
    true
}

/// Slot for this dispatcher, claiming a free one on first sight after the
/// one-time layout validation. `None` means the table is full or the
/// pointer failed validation; the pass then falls through untouched.
unsafe fn slot(dispatcher: usize) -> Option<usize> {
    for (index, key) in KEYS.iter().enumerate() {
        let current = key.load(Ordering::Relaxed);
        if current == dispatcher {
            return Some(index);
        }
        if current != 0 {
            continue;
        }
        if !validated(dispatcher) {
            return None;
        }
        return match key.compare_exchange(0, dispatcher, Ordering::AcqRel, Ordering::Relaxed) {
            Ok(_) => Some(index),
            Err(existing) if existing == dispatcher => Some(index),
            Err(_) => None,
        };
    }
    None
}

unsafe fn validated(dispatcher: usize) -> bool {
    if !dispatcher.is_multiple_of(8) || !memory::accessible(dispatcher, DISPATCHER_SPAN, false) {
        return false;
    }
    let context = *(dispatcher as *const usize);
    if !context.is_multiple_of(8) || !memory::accessible(context, CONTEXT_SPAN, false) {
        return false;
    }
    let shared = *((context + 8) as *const usize);
    shared.is_multiple_of(8) && memory::accessible(shared, SHARED_SPAN, false)
}

/// Reads what the game itself is about to read in its rescan. The layout was
/// validated once per dispatcher; the record vector is bounds-checked here and
/// every counter pointer must be a plausible aligned address.
unsafe fn snapshot(dispatcher: usize) -> Option<Snapshot> {
    let read = |address: usize| (address as *const u64).read_volatile();
    let context = read(dispatcher) as usize;
    let shared = read(context + 8) as usize;
    let enabled = ((context + CONTEXT_ENABLED) as *const u8).read_volatile() & 1 != 0;
    let mut queue_sizes = [0u64; 6];
    for (slot, offset) in queue_sizes.iter_mut().zip(LOCAL_SIZES) {
        *slot = read(dispatcher + offset);
    }
    for (slot, offset) in queue_sizes[3..].iter_mut().zip(SHARED_SIZES) {
        *slot = read(shared + offset);
    }
    let begin = read(dispatcher + PENDING_BEGIN) as usize;
    let end = read(dispatcher + PENDING_END) as usize;
    if begin == 0
        || end < begin
        || !begin.is_multiple_of(8)
        || !(end - begin).is_multiple_of(PENDING_STRIDE)
    {
        return None;
    }
    let pending_records = (end - begin) / PENDING_STRIDE;
    if pending_records > PENDING_MAX {
        return None;
    }
    let mut all_pending_blocked = true;
    let mut record_address = begin;
    while record_address < end {
        let counter = read(record_address + 8) as usize;
        if counter < 0x10000 || !counter.is_multiple_of(4) {
            return None;
        }
        if (counter as *const u32).read_volatile() == 0 {
            all_pending_blocked = false;
            break;
        }
        record_address += PENDING_STRIDE;
    }
    let resume_vectors_empty = RESUME_VECTORS
        .iter()
        .all(|&(first, last)| read(dispatcher + first) == read(dispatcher + last));
    Some(Snapshot {
        enabled,
        queue_sizes,
        pending_records,
        all_pending_blocked,
        resume_vectors_empty,
    })
}

/// Integer-only; no allocation, file or log access; no lock is held here.
#[skyline::hook(offset = SITE, inline)]
unsafe fn dispatcher_idle_pass(ctx: &InlineCtx) {
    if MAIN_THREAD_ONLY.load(Ordering::Relaxed) && !on_main_thread() {
        return;
    }
    let dispatcher = ctx.registers[24].x() as usize;
    let Some(index) = slot(dispatcher) else {
        return;
    };
    let Some(snapshot) = snapshot(dispatcher) else {
        return;
    };
    let packed = STREAKS[index].load(Ordering::Relaxed);
    let mut streak = Streak {
        idle_passes: packed as u32,
        last_ns: (packed >> 32) as u32,
    };
    let decision = decide(policy(), &snapshot, &mut streak);
    STREAKS[index].store(
        u64::from(streak.idle_passes) | (u64::from(streak.last_ns) << 32),
        Ordering::Relaxed,
    );
    if streak.idle_passes == 1 {
        IDLE_PASSES.fetch_add(1, Ordering::Relaxed);
        FRAME_STREAKS.fetch_add(1, Ordering::Relaxed);
    }
    let Ok(nanoseconds) = decision else {
        return;
    };
    // The clock is read only on passes that would sleep.
    let now = skyline::nn::os::GetSystemTick() as u64;
    if !SCENE_SEEN.load(Ordering::Relaxed)
        || now.wrapping_sub(LAST_PRESENT_TICK.load(Ordering::Relaxed)) > PRESENT_STALE_TICKS
    {
        GATED_SKIPS.fetch_add(1, Ordering::Relaxed);
        return;
    }
    if !ELIGIBLE.swap(true, Ordering::AcqRel) {
        record(Code::DispatcherBackoff, 5, now);
    }
    let cooldown = COOLDOWN_TICKS.load(Ordering::Relaxed);
    if cooldown != 0 && now.wrapping_sub(LAST_SLEEP_END.load(Ordering::Relaxed)) < cooldown {
        COOLDOWN_SKIPS.fetch_add(1, Ordering::Relaxed);
        return;
    }
    SLEEPS.fetch_add(1, Ordering::Relaxed);
    FRAME_SLEEPS.fetch_add(1, Ordering::Relaxed);
    LAST_NS.store(nanoseconds, Ordering::Relaxed);
    let timed = super::api_trace::active();
    skyline::nn::os::SleepThread(skyline::nn::TimeSpan {
        nanoseconds: u64::from(nanoseconds),
    });
    if timed || cooldown != 0 {
        let end = skyline::nn::os::GetSystemTick() as u64;
        if cooldown != 0 {
            LAST_SLEEP_END.store(end, Ordering::Relaxed);
        }
        if timed {
            let slept = end.wrapping_sub(now);
            FRAME_SLEEP_TICKS.fetch_add(slept, Ordering::Relaxed);
            FRAME_MAX_SLEEP_TICKS.fetch_max(slept, Ordering::Relaxed);
        }
    }
}

/// Any scene hook fired: the game is past initialization. Called by
/// `auto_driver::set_scene` on every scene observation, including unchanged ones.
pub(super) fn note_scene() {
    SCENE_SEEN.store(true, Ordering::Relaxed);
}

/// The tracked NVN window presented. Called from the present wrapper.
pub(super) fn note_present(tick: u64) {
    LAST_PRESENT_TICK.store(tick, Ordering::Relaxed);
}

/// Present wrapper, only while a finite API trace is active: emit the two
/// per-frame backoff rows and start the next frame's aggregates.
pub(super) fn frame_rows(tick: u64) {
    let sleeps = FRAME_SLEEPS.swap(0, Ordering::Relaxed);
    let sleep_ticks = FRAME_SLEEP_TICKS.swap(0, Ordering::Relaxed);
    let max_sleep = FRAME_MAX_SLEEP_TICKS.swap(0, Ordering::Relaxed);
    let streaks = FRAME_STREAKS.swap(0, Ordering::Relaxed);
    for event in
        crate::render_trace::backoff_frame_events(tick, sleeps, sleep_ticks, max_sleep, streaks)
    {
        super::api_trace::TRACE.observe(event);
    }
}

pub(super) fn install() {
    if INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let policy = load_policy();
    store_policy(policy);
    unsafe {
        let base = getRegionAddress(Region::Text) as usize;
        let end = getRegionAddress(Region::Rodata) as usize;
        let address = base + SITE;
        if address + SITE_WORDS.len() * 4 > end {
            OUTCOME.store(3, Ordering::Release);
            record(Code::DispatcherBackoff, 3, 0);
            return;
        }
        let mut words = [0u32; 5];
        for (index, word) in words.iter_mut().enumerate() {
            *word = ((address + index * 4) as *const u32).read_volatile();
        }
        if words != SITE_WORDS {
            let external = words[0] != SITE_WORDS[0] && words[1..] == SITE_WORDS[1..];
            let outcome = if external { 2 } else { 3 };
            OUTCOME.store(outcome, Ordering::Release);
            record(
                Code::DispatcherBackoff,
                u64::from(outcome),
                u64::from(words[0]),
            );
            skyline::println!(
                "[Deluxe] dispatcher backoff skipped: {}",
                if external {
                    "site already owned by another plugin"
                } else {
                    "site words differ from 13.0.5"
                }
            );
            return;
        }
        if !super::hooks::preflight(SITE) {
            OUTCOME.store(3, Ordering::Release);
            return;
        }
        skyline::install_hook!(dispatcher_idle_pass);
    }
    INSTALLED.store(true, Ordering::Release);
    OUTCOME.store(1, Ordering::Release);
    record(Code::HookInstalled, SITE as u64, 0);
    record(Code::DispatcherBackoff, 1, policy.packed());
}

/// Called by `mod.rs` when the `DISABLE_BACKOFF` flag is present.
pub(super) fn note_disabled() {
    OUTCOME.store(4, Ordering::Release);
    record(Code::DispatcherBackoff, 4, 0);
}

/// Diagnostics thread, every 500 ms. Rate is sleeps per second over the last
/// poll; the event record is written at most every 5 s and only on change.
pub(super) fn publish() {
    if !INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let sleeps = SLEEPS.load(Ordering::Relaxed);
    let previous = PUBLISHED_SLEEPS.swap(sleeps, Ordering::AcqRel);
    let delta = sleeps.saturating_sub(previous);
    RATE.store(
        u32::try_from(delta.saturating_mul(2)).unwrap_or(u32::MAX),
        Ordering::Relaxed,
    );
    let calls = PUBLISH_CALLS.fetch_add(1, Ordering::Relaxed) + 1;
    if delta != 0 && calls.is_multiple_of(10) {
        let last_us = u64::from(LAST_NS.load(Ordering::Relaxed) / 1000).min(0xffff);
        record(
            Code::DispatcherBackoffStats,
            IDLE_PASSES.load(Ordering::Relaxed),
            (sleeps & 0xffff_ffff_ffff) | (last_us << 48),
        );
    }
}

pub(super) fn status_label() -> String {
    match OUTCOME.load(Ordering::Acquire) {
        1 if !ELIGIBLE.load(Ordering::Acquire) => "Armed".to_string(),
        1 => crate::dispatcher_backoff::status_label(
            true,
            Some(RATE.load(Ordering::Relaxed)),
            LAST_NS.load(Ordering::Relaxed),
        ),
        2 => "External".to_string(),
        3 => "Unavailable".to_string(),
        _ => "Off".to_string(),
    }
}
