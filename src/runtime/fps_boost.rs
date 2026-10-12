//! Native-window swap interval, owned by the tracked NVN window thread.
//!
//! `nvnWindowInitialize`, `nvnWindowFinalize`, and a bounded
//! `nvnQueuePresentTexture` cadence are the only callers that walk or write
//! the pointer chain. The diagnostics poll publishes the last outcome and
//! does not touch guest memory.
//!
//! `arm` latches `sd:/ultimate/ssbu_online_deluxe_rewrite/ALLOW_FPS_BOOST`
//! once. The saved `fps_boost` flag is left unchanged. Without the marker
//! an emulator request stays Unsupported and this owner does not write
//! `10000`.

use super::{memory, record, settings, Code};
use crate::config::Platform;
use crate::fps_boost::{self, OwnerEdge, Phase, Request, Schedule, State};
use skyline::hooks::{getRegionAddress, Region};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;

static ARMED: AtomicBool = AtomicBool::new(false);
static DIAGNOSTIC: AtomicBool = AtomicBool::new(false);
static APPLIED: AtomicBool = AtomicBool::new(false);
static PHASE: AtomicU8 = AtomicU8::new(0);
static PACKED: AtomicU64 = AtomicU64::new(u64::MAX);
static LAST_TICK: AtomicU64 = AtomicU64::new(u64::MAX);
static WANTS_CODE: AtomicU8 = AtomicU8::new(0);
static PUBLISHED: AtomicU64 = AtomicU64::new(u64::MAX);
static LIFE: Mutex<Life> = Mutex::new(Life::new());

struct Life {
    state: State,
    schedule: Schedule,
}

impl Life {
    const fn new() -> Self {
        Self {
            state: State::new(),
            schedule: Schedule::new(),
        }
    }
}

pub(super) fn arm() {
    if ARMED.load(Ordering::Acquire) {
        return;
    }
    let present = std::path::Path::new(super::DATA_DIR)
        .join("ALLOW_FPS_BOOST")
        .exists();
    DIAGNOSTIC.store(present, Ordering::Release);
    ARMED.store(true, Ordering::Release);
}

pub(super) fn applied() -> bool {
    ARMED.load(Ordering::Acquire) && APPLIED.load(Ordering::Acquire)
}

pub(super) fn status_label() -> &'static str {
    let config = settings();
    fps_boost::status_label(phase(), request_from(&config))
}

pub(super) fn on_initialize(tick: u64) {
    owner(OwnerEdge::Initialize, tick);
}

pub(super) fn on_finalize(tick: u64) {
    owner(OwnerEdge::Finalize, tick);
}

pub(super) fn on_present(tick: u64) {
    if !ARMED.load(Ordering::Acquire) {
        return;
    }
    let wants = request_from(&settings()).wants_boost();
    let cached = Schedule::from_cache(
        LAST_TICK.load(Ordering::Acquire),
        WANTS_CODE.load(Ordering::Acquire),
    );
    if !cached.due(OwnerEdge::Present, tick, wants) {
        return;
    }
    owner(OwnerEdge::Present, tick);
}

/// Diagnostics thread only. Copies the owner's last packed outcome into the
/// event log. It does not walk or write the native window.
pub(super) fn publish() {
    if !ARMED.load(Ordering::Acquire) {
        return;
    }
    let packed = PACKED.load(Ordering::Acquire);
    if packed == u64::MAX {
        return;
    }
    if PUBLISHED.swap(packed, Ordering::AcqRel) != packed {
        record(Code::FpsBoost, packed & 0xffff, (packed >> 16) & 1);
    }
}

fn owner(edge: OwnerEdge, tick: u64) {
    if !ARMED.load(Ordering::Acquire) {
        return;
    }
    let mut request = request_from(&settings());
    // Restore our owned interval before the tracked NVN window is finalized.
    // Keep the saved preference for the next initialized window.
    if edge == OwnerEdge::Finalize {
        request.configured = false;
    }
    let wants = request.wants_boost();
    let main = unsafe { getRegionAddress(Region::Text) as u64 };
    let mut guest = SwitchMem;
    let outcome = {
        let mut life = LIFE.lock().unwrap_or_else(|poison| poison.into_inner());
        if !life.schedule.due(edge, tick, wants) {
            return;
        }
        let outcome = fps_boost::step(&mut life.state, &mut guest, main, request);
        life.schedule.note(tick, wants);
        let (cached_tick, cached_code) = life.schedule.cache();
        LAST_TICK.store(cached_tick, Ordering::Release);
        WANTS_CODE.store(cached_code, Ordering::Release);
        outcome
    };
    APPLIED.store(outcome.applied, Ordering::Release);
    PHASE.store(outcome.phase as u8, Ordering::Release);
    let packed = u64::from(outcome.phase as u8)
        | (u64::from(outcome.note as u8) << 8)
        | (u64::from(outcome.applied) << 16);
    PACKED.store(packed, Ordering::Release);
}

fn request_from(config: &crate::config::Config) -> Request {
    Request {
        emulator: config.platform == Platform::Emulator,
        configured: config.fps_boost,
        diagnostic: DIAGNOSTIC.load(Ordering::Acquire),
    }
}

fn phase() -> Phase {
    match PHASE.load(Ordering::Acquire) {
        1 => Phase::Applied,
        2 => Phase::Unavailable,
        3 => Phase::Refused,
        4 => Phase::Unsupported,
        5 => Phase::Diagnostic,
        _ => Phase::Off,
    }
}

struct SwitchMem;

impl fps_boost::GuestMemory for SwitchMem {
    fn read_u32(&self, address: u64) -> Option<u32> {
        let address = usize::try_from(address).ok()?;
        if !address.is_multiple_of(4) || unsafe { !memory::accessible(address, 4, false) } {
            return None;
        }
        Some(unsafe { (address as *const u32).read() })
    }

    fn read_u64(&self, address: u64) -> Option<u64> {
        let address = usize::try_from(address).ok()?;
        if !address.is_multiple_of(8) || unsafe { !memory::accessible(address, 8, false) } {
            return None;
        }
        Some(unsafe { (address as *const u64).read() })
    }

    fn write_u32(&mut self, address: u64, value: u32) -> bool {
        let Some(address) = usize::try_from(address).ok() else {
            return false;
        };
        if !address.is_multiple_of(4) || unsafe { !memory::accessible(address, 4, true) } {
            return false;
        }
        unsafe { (address as *mut u32).write(value) };
        true
    }
}
