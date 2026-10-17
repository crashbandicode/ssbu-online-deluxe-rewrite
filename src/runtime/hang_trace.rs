//! Device side of the hang trace (0.3.2): lock-free counters touched by the
//! hooks, and a text log written only by the diagnostics thread.
//!
//! `sd:/ultimate/ssbu_online_deluxe_rewrite/hang-trace.log` gets a
//! `[Heartbeat]` line every 5 s and a `[HangTrace] scene` line on each scene
//! change. A hook that entered and never returned shows as `ACTIVE` with a
//! growing age. Formatting lives in `crate::hang_trace`. No hook here
//! allocates, locks, or touches a file.

use super::{logging, DATA_DIR};
use crate::hang_trace::{
    header, heartbeat_line, scene_line, HookSample, Liveness, Memory, HOOK_COUNT,
};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;

const FILE: &str = "hang-trace.log";
const MAX_BYTES: u64 = 2 * 1024 * 1024;
/// Diagnostics polls are 500 ms apart; a heartbeat every tenth poll.
const HEARTBEAT_EVERY: u64 = 10;

/// Index into `HOOK_NAMES`.
#[derive(Clone, Copy)]
#[repr(usize)]
pub(super) enum Hook {
    Present = 0,
    BackoffPass = 1,
    BackoffSleep = 2,
    CssWait = 3,
    CssUiJob = 4,
    FighterLoad = 5,
    FighterUnload = 6,
    FighterMain = 7,
    CutIn = 8,
}

struct Slot {
    calls: AtomicU64,
    in_flight: AtomicU32,
    entry_tick: AtomicU64,
}

impl Slot {
    const fn new() -> Self {
        Self {
            calls: AtomicU64::new(0),
            in_flight: AtomicU32::new(0),
            entry_tick: AtomicU64::new(0),
        }
    }

    fn sample(&self) -> HookSample {
        HookSample {
            calls: self.calls.load(Ordering::Relaxed),
            in_flight: self.in_flight.load(Ordering::Relaxed),
            entry_tick: self.entry_tick.load(Ordering::Relaxed),
        }
    }
}

static SLOTS: [Slot; HOOK_COUNT] = [const { Slot::new() }; HOOK_COUNT];
static INSTALLED: AtomicBool = AtomicBool::new(false);
static BOOT_TICK: AtomicU64 = AtomicU64::new(0);
/// Packed scene byte as `auto_driver` stores it, and the one last written.
static SCENE: AtomicU8 = AtomicU8::new(0);
static WRITTEN_SCENE: AtomicU8 = AtomicU8::new(0);
static SCENE_CHANGES: AtomicU64 = AtomicU64::new(0);
static WRITTEN_CHANGES: AtomicU64 = AtomicU64::new(0);
static LOG: Mutex<Option<File>> = Mutex::new(None);
/// Latest kernel memory numbers, refreshed every poll for the overlay row.
static MEM_USED: AtomicU64 = AtomicU64::new(0);
static MEM_TOTAL: AtomicU64 = AtomicU64::new(0);
static MEM_HEAP: AtomicU64 = AtomicU64::new(0);
static MEM_VALID: AtomicBool = AtomicBool::new(false);

fn refresh_memory() -> Option<Memory> {
    let memory = super::memory::memory_snapshot();
    match memory {
        Some(m) => {
            MEM_USED.store(m.used, Ordering::Relaxed);
            MEM_TOTAL.store(m.total, Ordering::Relaxed);
            MEM_HEAP.store(m.heap, Ordering::Relaxed);
            MEM_VALID.store(true, Ordering::Release);
        }
        None => MEM_VALID.store(false, Ordering::Release),
    }
    memory
}

/// Overlay: the most recent poll's numbers, no system call on the UI path.
pub(super) fn memory() -> Option<Memory> {
    MEM_VALID.load(Ordering::Acquire).then(|| Memory {
        used: MEM_USED.load(Ordering::Relaxed),
        total: MEM_TOTAL.load(Ordering::Relaxed),
        heap: MEM_HEAP.load(Ordering::Relaxed),
    })
}

fn tick() -> u64 {
    unsafe { skyline::nn::os::GetSystemTick() as u64 }
}

fn slot(hook: Hook) -> &'static Slot {
    &SLOTS[hook as usize]
}

/// Count-only liveness signal (present, idle pass, per-fighter tick).
pub(super) fn count(hook: Hook) {
    slot(hook).calls.fetch_add(1, Ordering::Relaxed);
}

/// A hook body is about to run the original or sleep. Pair with [`exit`].
pub(super) fn enter(hook: Hook) -> u64 {
    let now = tick();
    enter_at(hook, now);
    now
}

/// Same as [`enter`] for callers that already read the clock.
pub(super) fn enter_at(hook: Hook, now: u64) {
    let slot = slot(hook);
    slot.calls.fetch_add(1, Ordering::Relaxed);
    slot.entry_tick.store(now, Ordering::Relaxed);
    slot.in_flight.fetch_add(1, Ordering::AcqRel);
}

pub(super) fn exit(hook: Hook) {
    // Saturate rather than wrap if an exit ever outnumbers its entry.
    let _ = slot(hook)
        .in_flight
        .fetch_update(Ordering::AcqRel, Ordering::Relaxed, |n| n.checked_sub(1));
}

/// `auto_driver::store_scene`: a changed scene is written at the next poll.
pub(super) fn note_scene(packed: u8) {
    if SCENE.swap(packed, Ordering::AcqRel) != packed {
        SCENE_CHANGES.fetch_add(1, Ordering::AcqRel);
    }
}

/// Rotates to `hang-trace.previous.log` (the same rule as `events.log`) and
/// writes the header.
fn open(root: &Path, boot: u64) -> Option<File> {
    let mut file = match logging::rotate(&root.join(FILE)) {
        Ok(file) => file,
        Err(error) => {
            skyline::println!("[Deluxe] hang trace unavailable: {}", error);
            return None;
        }
    };
    let _ = file
        .write_all(header(crate::build_info::BUILD_ID, boot).as_bytes())
        .and_then(|_| file.sync_data());
    Some(file)
}

/// Startup, right after the version check and before any hook installs, so a
/// hang during installation still leaves the header and the first heartbeat.
pub(super) fn install() {
    if INSTALLED.swap(true, Ordering::AcqRel) {
        return;
    }
    let boot = tick();
    BOOT_TICK.store(boot, Ordering::Release);
    let file = open(Path::new(DATA_DIR), boot);
    if let Ok(mut guard) = LOG.lock() {
        *guard = file;
    }
    logging::startup_phase("hang_trace_ready");
}

fn write_line(file: &mut File, line: &str) -> std::io::Result<()> {
    writeln!(file, "{line}")?;
    file.sync_data()
}

/// Diagnostics thread, every 500 ms. Writes scene changes as they appear and
/// a heartbeat every 5 s; rotates the file at 2 MiB.
pub(super) fn poll(iteration: u64) {
    if !INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let Ok(mut guard) = LOG.lock() else {
        return;
    };
    let Some(file) = guard.as_mut() else {
        return;
    };
    let boot = BOOT_TICK.load(Ordering::Relaxed);
    let now = tick();
    let memory = refresh_memory();
    let changes = SCENE_CHANGES.load(Ordering::Acquire);
    let scene = SCENE.load(Ordering::Acquire);
    if changes != WRITTEN_CHANGES.load(Ordering::Relaxed) {
        let previous = WRITTEN_SCENE.swap(scene, Ordering::AcqRel);
        WRITTEN_CHANGES.store(changes, Ordering::Relaxed);
        let _ = write_line(file, &scene_line(previous, scene, now, boot));
    }
    if iteration.is_multiple_of(HEARTBEAT_EVERY) {
        let samples: [HookSample; HOOK_COUNT] = std::array::from_fn(|n| SLOTS[n].sample());
        let live = Liveness {
            scene,
            ready: super::auto_driver::ready(),
            in_css: super::capture::in_css(),
            trace_active: super::api_trace::active(),
        };
        let _ = write_line(file, &heartbeat_line(now, boot, live, memory, &samples));
        if file
            .metadata()
            .map(|m| m.len() >= MAX_BYTES)
            .unwrap_or(false)
        {
            *guard = open(Path::new(DATA_DIR), boot);
        }
    }
}
