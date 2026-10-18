//! Default-off pre-HID timing experiment. Never changes game state or update counts.
use super::{api_trace, memory};
use crate::frame_delay::Budget;
use skyline::hooks::{getRegionAddress, Region};
use std::io::Write;
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Mutex,
};

struct State {
    budget: Budget,
    end: u64,
    window: usize,
    generation: u64,
    mode: u8,
    hid: bool,
    updates: u64,
    sleep_us: u64,
}
static STATE: Mutex<State> = Mutex::new(State {
    budget: Budget::new(),
    end: 0,
    window: 0,
    generation: 0,
    mode: 0,
    hid: false,
    updates: 0,
    sleep_us: 0,
});
static PRESENT_OWNER: AtomicUsize = AtomicUsize::new(0);
static PAIRED_SLEEP: AtomicU64 = AtomicU64::new(0);
static PRESENT_OBSERVATIONS: AtomicU64 = AtomicU64::new(0);
static PRESENT_INVALID: AtomicU64 = AtomicU64::new(0);
static MISSED_FRAMES: AtomicU64 = AtomicU64::new(0);
static MISSED_EVENTS: AtomicU64 = AtomicU64::new(0);
static DELAYED_PRESENTS: AtomicU64 = AtomicU64::new(0);
static DELAYED_MISSED: AtomicU64 = AtomicU64::new(0);
static APPLIED: AtomicU64 = AtomicU64::new(0);
static TOTAL_US: AtomicU64 = AtomicU64::new(0);
static LAST_US: AtomicU64 = AtomicU64::new(0);
static RESETS: AtomicU64 = AtomicU64::new(0);
static CADENCE_OUTLIERS: AtomicU64 = AtomicU64::new(0);
static OVERSLEEPS: AtomicU64 = AtomicU64::new(0);

pub(super) fn clock() -> u64 {
    if super::frame_delay_enabled() {
        unsafe { skyline::nn::os::GetSystemTick() as u64 }
    } else {
        0
    }
}
fn us(ticks: u64) -> u64 {
    ticks.saturating_mul(5) / 96
}
pub(super) fn acquired(begin: u64, window: usize, valid: bool) {
    if begin == 0 {
        return;
    }
    let end = clock();
    let auto = super::auto_driver::snapshot();
    let mode = super::renderer::mode() as u8;
    if let Ok(mut s) = STATE.lock() {
        let period = us(end.saturating_sub(s.end));
        let clean = valid
            && s.end != 0
            && s.window == window
            && s.generation == auto.selection_generation
            && s.mode == mode
            && s.hid
            && s.updates == 1
            && auto.ready;
        if s.end != 0 && !(14_000..=19_000).contains(&period) {
            CADENCE_OUTLIERS.fetch_add(1, Ordering::Relaxed);
        }
        let old_count = s.budget.count;
        let sleep = s.sleep_us;
        s.budget
            .observe(period, us(end.saturating_sub(begin)), sleep, clean);
        if old_count != 0 && s.budget.count == 0 {
            RESETS.fetch_add(1, Ordering::Relaxed);
        }
        let tick = api_trace::start();
        api_trace::point(
            tick,
            96,
            period.min(i32::MAX as u64) as i32,
            us(end.saturating_sub(begin)).min(i32::MAX as u64) as i32,
        );
        s.end = end;
        s.window = window;
        s.generation = auto.selection_generation;
        s.mode = mode;
        s.hid = false;
        s.updates = 0;
        s.sleep_us = 0;
    }
}
pub(super) fn note_update() {
    if !super::frame_delay_enabled() {
        return;
    }
    if let Ok(mut s) = STATE.lock() {
        s.updates = s.updates.saturating_add(1);
    }
}
pub(super) unsafe fn before_hid() {
    if !super::frame_delay_enabled() {
        return;
    }
    let now = clock();
    let auto = super::auto_driver::snapshot();
    let mode = super::renderer::mode();
    let credits = getRegionAddress(Region::Text) as usize + 0x593b440;
    let ready = auto.ready
        && !auto.cleanup_hold
        && super::renderer::status() == "Active"
        && !super::research_present_interval_zero_enabled()
        && (!mode.render_opts() || super::render_sequence::frame_active())
        && memory::accessible(credits, 4, false)
        && (*(credits as *const std::sync::atomic::AtomicU32)).load(Ordering::Acquire) == 0;
    let request = if let Ok(mut s) = STATE.lock() {
        if !ready
            || s.end == 0
            || us(now.saturating_sub(s.end)) > 6_000
            || s.generation != auto.selection_generation
            || s.mode != mode as u8
        {
            if s.budget.count != 0 {
                RESETS.fetch_add(1, Ordering::Relaxed);
            }
            s.budget.reset();
            s.hid = false;
            return;
        }
        s.hid = true;
        s.budget.delay()
    } else {
        0
    };
    if request == 0 {
        return;
    }
    let begin = clock();
    skyline::nn::os::SleepThread(skyline::nn::TimeSpan {
        nanoseconds: request * 1_000,
    });
    let end = clock();
    let actual = us(end.saturating_sub(begin));
    if let Ok(mut s) = STATE.lock() {
        s.sleep_us = actual;
        if actual > request + 500 {
            s.budget.reset();
            OVERSLEEPS.fetch_add(1, Ordering::Relaxed);
            RESETS.fetch_add(1, Ordering::Relaxed);
        }
    }
    APPLIED.fetch_add(1, Ordering::Relaxed);
    TOTAL_US.fetch_add(actual, Ordering::Relaxed);
    LAST_US.store(actual, Ordering::Relaxed);
    if api_trace::active() {
        api_trace::point(
            begin,
            95,
            request as i32,
            actual.min(i32::MAX as u64) as i32,
        );
    }
}
pub(super) fn write_status(out: &mut impl Write) -> std::io::Result<()> {
    writeln!(out,"research_frame_start_delay_present_observations={}\nresearch_frame_start_delay_present_invalid={}\nresearch_frame_start_delay_missed_frames={}\nresearch_frame_start_delay_missed_events={}\nresearch_frame_start_delay_delayed_presents={}\nresearch_frame_start_delay_delayed_missed_frames={}\nresearch_frame_start_delay_miss_semantics=game_reported_extra_vsync_events_not_physical_scanout",PRESENT_OBSERVATIONS.load(Ordering::Relaxed),PRESENT_INVALID.load(Ordering::Relaxed),MISSED_FRAMES.load(Ordering::Relaxed),MISSED_EVENTS.load(Ordering::Relaxed),DELAYED_PRESENTS.load(Ordering::Relaxed),DELAYED_MISSED.load(Ordering::Relaxed))?;
    let samples = STATE.lock().map(|s| s.budget.count).unwrap_or(0);
    writeln!(out,"research_frame_start_delay_enabled={}\nresearch_frame_start_delay_cap_us=4000\nresearch_frame_start_delay_margin_us=3000\nresearch_frame_start_delay_samples={}\nresearch_frame_start_delay_applied={}\nresearch_frame_start_delay_total_us={}\nresearch_frame_start_delay_last_us={}\nresearch_frame_start_delay_resets={}\nresearch_frame_start_delay_cadence_outliers={}\nresearch_frame_start_delay_oversleeps={}",
    super::frame_delay_enabled(),samples,APPLIED.load(Ordering::Relaxed),TOTAL_US.load(Ordering::Relaxed),LAST_US.load(Ordering::Relaxed),RESETS.load(Ordering::Relaxed),CADENCE_OUTLIERS.load(Ordering::Relaxed),OVERSLEEPS.load(Ordering::Relaxed))
}

// Read the game's final missed-VSync output at the audited known-caller ABI.
// This observes game accounting, not scanout/photons or GPU deadline attribution.
pub(super) fn present_entry(owner: usize, caller: usize) {
    if !super::frame_delay_enabled() {
        return;
    }
    let base = unsafe { getRegionAddress(Region::Text) as usize };
    PRESENT_OWNER.store(
        if caller == base + 0x374812c { owner } else { 0 },
        Ordering::Relaxed,
    );
    let sleep = STATE.lock().map(|s| s.sleep_us).unwrap_or(0);
    PAIRED_SLEEP.store(sleep, Ordering::Relaxed);
}
pub(super) unsafe fn present_completed(ctx: &skyline::hooks::InlineCtx) {
    if !super::frame_delay_enabled() {
        return;
    }
    let sp = ctx.sp.x() as usize;
    let base = getRegionAddress(Region::Text) as usize;
    let owner = PRESENT_OWNER.swap(0, Ordering::Relaxed);
    if owner == 0
        || ctx.registers[19].x() as usize != owner
        || sp % 16 != 0
        || !memory::accessible(sp + 0x48, 8, false)
        || !memory::accessible(sp + 0x2c0, 8, false)
        || !memory::accessible(sp + 0x2440, 4, false)
        || core::ptr::read_volatile((sp + 0x48) as *const usize) != base + 0x374812c
    {
        PRESENT_INVALID.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let missed = core::ptr::read_volatile((sp + 0x2c0) as *const u64);
    let status = core::ptr::read_volatile((sp + 0x2440) as *const i32);
    if missed > 1000 {
        PRESENT_INVALID.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let sleep = PAIRED_SLEEP.swap(0, Ordering::Relaxed);
    PRESENT_OBSERVATIONS.fetch_add(1, Ordering::Relaxed);
    if missed != 0 {
        MISSED_FRAMES.fetch_add(1, Ordering::Relaxed);
        MISSED_EVENTS.fetch_add(missed, Ordering::Relaxed);
    }
    if sleep != 0 {
        DELAYED_PRESENTS.fetch_add(1, Ordering::Relaxed);
        if missed != 0 {
            DELAYED_MISSED.fetch_add(1, Ordering::Relaxed);
        }
    }
    let tick = api_trace::start();
    api_trace::point(tick, 97, missed as i32, status);
    api_trace::point(tick, 98, sleep.min(i32::MAX as u64) as i32, missed as i32);
}
