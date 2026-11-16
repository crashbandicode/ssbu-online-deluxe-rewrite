//! Native elapsed-match time compared with wall time, independent of presentation.
//!
//! Input frames are the verified native elapsed clock in 60-Hz units, not an
//! unconditional engine-tick serial. The owner can apply native rate/pause state.
//! Callers must supply current-stage eligibility and a fresh nonzero generation.
//! Hints use the paired calibration ruling: sustained slow progress and observed
//! no-advance gaps are separate; short-window quantization is not a spike.

use crate::fixed::FixedStr;
use std::sync::atomic::{AtomicU64, Ordering};

pub const SPEED_TICKS_PER_SECOND: u64 = 19_200_000;
pub const SPEED_NATIVE_SATURATION: u32 = 21_599_999;
const SPEED_BIN: u64 = SPEED_TICKS_PER_SECOND / 10;
const SPEED_WINDOW: u64 = SPEED_TICKS_PER_SECOND;
// Approved paired-calibration ruling, 2026-11-16 (normal-time REPORT af75258f).
const SPEED_NO_ADVANCE: u64 = SPEED_TICKS_PER_SECOND / 10;
const SPEED_LOW_PERCENT: u128 = 97;
const SPEED_CLEAR_PERCENT: u128 = 99;
const SPEED_LOW_HOLD: u64 = SPEED_TICKS_PER_SECOND * 2;
const SPEED_SPIKE_HOLD: u64 = SPEED_TICKS_PER_SECOND * 2;
const SPEED_CLEAR_HOLD: u64 = SPEED_TICKS_PER_SECOND;

/// Timing descriptions, not a diagnosis of the network or either peer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    None,
    DelayTooLow,
    LagSpikes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
    /// None until one full second of eligible observations has elapsed.
    pub percent: Option<u8>,
    pub cause: Cause,
}

/// A native clock read bound to its validated current match-object identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeedSample {
    pub object_identity: u64,
    pub elapsed_frames: u32,
}

pub fn speed_label(reading: Reading) -> FixedStr<48> {
    let Some(percent) = reading.percent else {
        return FixedStr::from("Game --");
    };
    match reading.cause {
        Cause::None => crate::fixed!("Game {}%", percent),
        Cause::DelayTooLow => crate::fixed!("Game {}% (delay too low)", percent),
        Cause::LagSpikes => crate::fixed!("Game {}% (lag spikes)", percent),
    }
}

/// Append the bounded fact to the strip's existing timing segment.
pub fn speed_timing_line(timing: &str, reading: Option<Reading>) -> crate::fixed::Line {
    use core::fmt::Write;
    let mut line = crate::fixed::Line::from(timing);
    if let Some(reading) = reading {
        let _ = write!(line, "   {}", speed_label(reading));
    }
    line
}

/// Native tuple alone is insufficient: CSS can retain it. The caller must also
/// prove the current stage and native ReadyGo state for the current generation.
pub fn speed_stage_eligible(
    initialized: bool,
    melee: u8,
    mode: u32,
    match_stage: bool,
    ready_go: bool,
) -> bool {
    initialized
        && match_stage
        && ready_go
        && ((mode & 0x7fff_0000 == 0x0701_0000
            && matches!(melee, 0x0e | 0x0f | 0x10 | 0x14 | 0x18 | 0x25))
            || (mode == 0x0201_0000 && melee == 0x01))
}

#[derive(Clone, Copy)]
struct SpeedPoint {
    tick: u64,
    frames: u32,
}

impl SpeedPoint {
    const EMPTY: Self = Self { tick: 0, frames: 0 };
}

/// Fixed storage; observation and formatting use no heap, locks or I/O.
pub struct SpeedModel {
    points: [SpeedPoint; 16],
    count: usize,
    next: usize,
    generation: u64,
    object_identity: u64,
    start_tick: u64,
    last_tick: u64,
    last_advance: u64,
    published_tick: u64,
    spike_until: u64,
    low_since: Option<u64>,
    clear_since: Option<u64>,
    last_frames: u32,
    sustained: bool,
    reading: Option<Reading>,
}

impl SpeedModel {
    pub const fn new() -> Self {
        Self {
            points: [SpeedPoint::EMPTY; 16],
            count: 0,
            next: 0,
            generation: 0,
            object_identity: 0,
            start_tick: 0,
            last_tick: 0,
            last_advance: 0,
            published_tick: 0,
            spike_until: 0,
            low_since: None,
            clear_since: None,
            last_frames: 0,
            sustained: false,
            reading: None,
        }
    }

    pub fn speed_reading(&self) -> Option<Reading> {
        self.reading
    }

    /// Bind each observation to the validated native object. Replacement resets
    /// even when the stage token and increasing counter would otherwise agree.
    pub fn observe_speed_sample(
        &mut self,
        generation: u64,
        tick: u64,
        sample: Option<SpeedSample>,
    ) {
        let Some(sample) = sample.filter(|sample| sample.object_identity != 0) else {
            *self = Self::new();
            return;
        };
        if sample.object_identity != self.object_identity {
            *self = Self::new();
        }
        self.observe_speed(generation, tick, Some(sample.elapsed_frames));
        // The primitive can restart the whole model on clock/counter/generation
        // changes, so restore identity after it, never before it.
        if self.reading.is_some() {
            self.object_identity = sample.object_identity;
        }
    }

    /// Observe actual elapsed time, including gaps when the UI could not draw.
    /// None/zero generation invalidates immediately. A rollback starts a new
    /// warmup; a saturated native clock is invalid rather than a false freeze.
    pub fn observe_speed(&mut self, generation: u64, tick: u64, elapsed_frames: Option<u32>) {
        let Some(frames) = elapsed_frames.filter(|f| *f < SPEED_NATIVE_SATURATION) else {
            *self = Self::new();
            return;
        };
        if generation == 0 {
            *self = Self::new();
            return;
        }
        if self.reading.is_none()
            || generation != self.generation
            || tick < self.last_tick
            || frames < self.last_frames
        {
            self.speed_restart(generation, tick, frames);
            return;
        }
        // Ignore an unorderable second observation, including a different
        // counter at that same timestamp; the next real interval retains it.
        if tick == self.last_tick {
            return;
        }
        if frames != self.last_frames {
            self.last_advance = tick;
        } else if tick - self.last_advance >= SPEED_NO_ADVANCE {
            self.spike_until = tick.saturating_add(SPEED_SPIKE_HOLD);
        }
        self.last_tick = tick;
        self.last_frames = frames;
        if tick - self.published_tick < SPEED_BIN {
            return;
        }
        self.published_tick = tick;
        let point = SpeedPoint { tick, frames };
        // Read references before retaining now: even a long blocked UI gap is
        // measured against the actual preceding observation, never one second
        // invented at the current counter.
        let reference = self.speed_reference(tick, SPEED_WINDOW);
        if tick - self.start_tick >= SPEED_WINDOW {
            if let Some(reference) = reference {
                self.speed_update_sustained(point, reference);
                let denominator = (tick - reference.tick) as u128 * 60;
                let numerator =
                    (frames - reference.frames) as u128 * SPEED_TICKS_PER_SECOND as u128 * 100;
                let percent = ((numerator + denominator / 2) / denominator).min(100) as u8;
                self.reading = Some(Reading {
                    percent: Some(percent),
                    cause: if tick < self.spike_until {
                        Cause::LagSpikes
                    } else if self.sustained {
                        Cause::DelayTooLow
                    } else {
                        Cause::None
                    },
                });
            }
        }
        self.speed_retain_point(point);
    }

    fn speed_restart(&mut self, generation: u64, tick: u64, frames: u32) {
        *self = Self::new();
        self.generation = generation;
        self.start_tick = tick;
        self.last_tick = tick;
        self.last_advance = tick;
        self.published_tick = tick;
        self.last_frames = frames;
        self.reading = Some(Reading {
            percent: None,
            cause: Cause::None,
        });
        self.speed_retain_point(SpeedPoint { tick, frames });
    }

    fn speed_retain_point(&mut self, point: SpeedPoint) {
        self.points[self.next] = point;
        self.next = (self.next + 1) % self.points.len();
        self.count = (self.count + 1).min(self.points.len());
    }

    /// Closest retained observation at or before the real wall-time cutoff.
    fn speed_reference(&self, tick: u64, window: u64) -> Option<SpeedPoint> {
        let cutoff = tick.checked_sub(window)?;
        self.points[..self.count]
            .iter()
            .copied()
            .filter(|p| p.tick <= cutoff)
            .max_by_key(|p| p.tick)
    }

    fn speed_below(now: SpeedPoint, before: SpeedPoint, percent: u128) -> bool {
        (now.frames - before.frames) as u128 * SPEED_TICKS_PER_SECOND as u128 * 100
            < (now.tick - before.tick) as u128 * 60 * percent
    }

    fn speed_update_sustained(&mut self, now: SpeedPoint, before: SpeedPoint) {
        if Self::speed_below(now, before, SPEED_LOW_PERCENT) {
            self.clear_since = None;
            let since = *self.low_since.get_or_insert(now.tick);
            if now.tick - since >= SPEED_LOW_HOLD {
                self.sustained = true;
            }
        } else {
            self.low_since = None;
            if !Self::speed_below(now, before, SPEED_CLEAR_PERCENT) {
                let since = *self.clear_since.get_or_insert(now.tick);
                if now.tick - since >= SPEED_CLEAR_HOLD {
                    self.sustained = false;
                }
            } else {
                self.clear_since = None;
            }
        }
    }
}

impl Default for SpeedModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Current stage attempt, with the low bit published only after acceptance.
/// An asynchronous offer cannot activate a ticket invalidated by another stage,
/// CSS or menu entry, including a rejected attempt in the same native scene.
pub struct StageEpoch {
    epoch: AtomicU64,
}

impl StageEpoch {
    pub const fn new() -> Self {
        Self {
            epoch: AtomicU64::new(0),
        }
    }

    pub fn speed_begin_stage(&self) -> u64 {
        let before = self
            .epoch
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |old| {
                Some(speed_next_ticket(old))
            })
            .unwrap();
        speed_next_ticket(before)
    }

    pub fn speed_invalidate_stage(&self) {
        self.speed_begin_stage();
    }

    pub fn speed_arm_stage(&self, ticket: u64) -> bool {
        ticket != 0
            && ticket & 1 == 0
            && self
                .epoch
                .compare_exchange(ticket, ticket | 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
    }

    pub fn speed_stage_token(&self) -> Option<u64> {
        let token = self.epoch.load(Ordering::Acquire);
        (token & 1 != 0).then_some(token)
    }
}

fn speed_next_ticket(old: u64) -> u64 {
    (old & !1).wrapping_add(2).max(2)
}

impl Default for StageEpoch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    // Only this thread's explicitly bracketed observations are counted. The
    // parallel test harness and other tests cannot contaminate the measurement.
    struct SpeedTestAllocator;
    thread_local! {
        static SPEED_COUNT_ENABLED: Cell<bool> = const { Cell::new(false) };
        static SPEED_ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    }

    fn speed_count_allocation() {
        if SPEED_COUNT_ENABLED.try_with(Cell::get).unwrap_or(false) {
            let _ = SPEED_ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
        }
    }

    unsafe impl GlobalAlloc for SpeedTestAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            speed_count_allocation();
            unsafe { System.alloc(layout) }
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            speed_count_allocation();
            unsafe { System.alloc_zeroed(layout) }
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            speed_count_allocation();
            unsafe { System.realloc(ptr, layout, size) }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe { System.dealloc(ptr, layout) }
        }
    }

    #[global_allocator]
    static SPEED_TEST_ALLOCATOR: SpeedTestAllocator = SpeedTestAllocator;

    fn speed_tick_ms(ms: u64) -> u64 {
        ms * (SPEED_TICKS_PER_SECOND / 1000)
    }

    fn speed_feed_rate(
        model: &mut SpeedModel,
        generation: u64,
        end_ms: u64,
        fps_tenths: u64,
        step_ms: u64,
    ) {
        for ms in (0..=end_ms).step_by(step_ms as usize) {
            let native_elapsed = (ms * fps_tenths / 10_000) as u32;
            model.observe_speed(generation, speed_tick_ms(ms), Some(native_elapsed));
        }
    }

    #[test]
    fn speed_uses_simulation_delta_and_elapsed_ticks() {
        for (fps_tenths, expected) in [(600, 100), (552, 92), (480, 80)] {
            let mut model = SpeedModel::new();
            speed_feed_rate(&mut model, 7, 6000, fps_tenths, 10);
            assert_eq!(model.speed_reading().unwrap().percent, Some(expected));
        }
        let mut model = SpeedModel::new();
        speed_feed_rate(&mut model, 1, 900, 600, 10);
        assert_eq!(model.speed_reading().unwrap().percent, None);
        model.observe_speed(1, speed_tick_ms(1000), Some(60));
        assert_eq!(model.speed_reading().unwrap().percent, Some(100));
        // Raw >100% must clear the sustained warning even though display caps.
        speed_feed_rate(&mut model, 2, 6000, 720, 10);
        assert_eq!(
            model.speed_reading(),
            Some(Reading {
                percent: Some(100),
                cause: Cause::None
            })
        );
    }

    #[test]
    fn speed_jitter_freeze_and_recovery() {
        let mut jittered = SpeedModel::new();
        for i in 0..400 {
            let ms = i * 17 + [0, 3, 1, 5][i as usize % 4];
            jittered.observe_speed(1, speed_tick_ms(ms), Some((ms * 60 / 1000) as u32));
        }
        assert!((98..=100).contains(&jittered.speed_reading().unwrap().percent.unwrap()));
        for pause_ms in [100, 250, 500] {
            let mut model = SpeedModel::new();
            speed_feed_rate(&mut model, 1, 3000, 600, 10);
            for ms in (3010..=3000 + pause_ms).step_by(10) {
                model.observe_speed(1, speed_tick_ms(ms), Some(180));
            }
            let stopped = model.speed_reading().unwrap();
            assert!(
                stopped.percent.unwrap() < 100,
                "pause {pause_ms}: {stopped:?}"
            );
            assert_eq!(stopped.cause, Cause::LagSpikes);
            for ms in (3010 + pause_ms..=7000).step_by(10) {
                model.observe_speed(
                    1,
                    speed_tick_ms(ms),
                    Some(((ms - pause_ms) * 60 / 1000) as u32),
                );
            }
            assert_eq!(
                model.speed_reading(),
                Some(Reading {
                    percent: Some(100),
                    cause: Cause::None
                })
            );
        }
    }

    #[test]
    fn sustained_slowdown_selects_delay_hint() {
        let mut model = SpeedModel::new();
        speed_feed_rate(&mut model, 1, 2900, 552, 10);
        assert_eq!(model.speed_reading().unwrap().cause, Cause::None);
        model.observe_speed(1, speed_tick_ms(3000), Some(165));
        assert_eq!(model.speed_reading().unwrap().cause, Cause::DelayTooLow);
        // An intermediate 98% plateau cannot clear the sustained state.
        let base = 165;
        for ms in (3010..=6000).step_by(10) {
            model.observe_speed(
                1,
                speed_tick_ms(ms),
                Some(base + ((ms - 3000) * 588 / 10_000) as u32),
            );
        }
        assert_eq!(model.speed_reading().unwrap().cause, Cause::DelayTooLow);
        let base = base + 176;
        for ms in (6010..=8500).step_by(10) {
            model.observe_speed(
                1,
                speed_tick_ms(ms),
                Some(base + ((ms - 6000) * 60 / 1000) as u32),
            );
        }
        assert_eq!(model.speed_reading().unwrap().cause, Cause::None);
    }

    #[test]
    fn spike_overrides_sustained_hint() {
        let mut model = SpeedModel::new();
        speed_feed_rate(&mut model, 1, 4000, 552, 10);
        assert_eq!(model.speed_reading().unwrap().cause, Cause::DelayTooLow);
        for ms in (4010..=4500).step_by(10) {
            model.observe_speed(1, speed_tick_ms(ms), Some(220));
        }
        assert_eq!(model.speed_reading().unwrap().cause, Cause::LagSpikes);
        // 92% resumes, so after spike hold the sustained hint returns.
        for ms in (4510..=8000).step_by(10) {
            model.observe_speed(
                1,
                speed_tick_ms(ms),
                Some(220 + ((ms - 4500) * 552 / 10_000) as u32),
            );
        }
        assert_eq!(model.speed_reading().unwrap().cause, Cause::DelayTooLow);
    }

    #[test]
    fn speed_blocked_ui_gap_preserves_actual_elapsed_time() {
        let mut model = SpeedModel::new();
        speed_feed_rate(&mut model, 1, 2000, 600, 10);
        model.observe_speed(1, speed_tick_ms(5000), Some(120));
        assert_eq!(
            model.speed_reading(),
            Some(Reading {
                percent: Some(0),
                cause: Cause::LagSpikes
            })
        );
        // The next draw advances six frames in 3.1 real seconds against the
        // closest retained pre-cutoff point; it must not report a healthy 100%.
        model.observe_speed(1, speed_tick_ms(5100), Some(126));
        assert_eq!(model.speed_reading().unwrap().percent, Some(3));
    }

    #[test]
    fn speed_draw_counts_duplicate_ticks_and_multi_tick_polls_do_not_define_rate() {
        let mut dense = SpeedModel::new();
        let mut sparse = SpeedModel::new();
        for ms in (0..=6000).step_by(10) {
            let frames = (ms * 60 / 1000) as u32;
            dense.observe_speed(1, speed_tick_ms(ms), Some(frames));
            dense.observe_speed(1, speed_tick_ms(ms), Some(frames + 1000));
            if ms % 100 == 0 {
                sparse.observe_speed(1, speed_tick_ms(ms), Some(frames));
            }
        }
        assert_eq!(dense.speed_reading(), sparse.speed_reading());
        assert_eq!(sparse.speed_reading().unwrap().percent, Some(100));
        // Fewer than 100ms since last publication leaves the displayed reading.
        let before = dense.speed_reading();
        dense.observe_speed(1, speed_tick_ms(6050), Some(360));
        assert_eq!(dense.speed_reading(), before);
    }

    #[test]
    fn speed_counter_clock_generation_and_saturation_restart_without_wrap() {
        let mut model = SpeedModel::new();
        speed_feed_rate(&mut model, 1, 2000, 600, 10);
        model.observe_speed(2, speed_tick_ms(2000), Some(120));
        assert_eq!(model.speed_reading().unwrap().percent, None);
        model.observe_speed(2, speed_tick_ms(2100), Some(0));
        assert_eq!(model.speed_reading().unwrap().percent, None);
        model.observe_speed(2, speed_tick_ms(100), Some(1));
        assert_eq!(model.speed_reading().unwrap().percent, None);
        model.observe_speed(2, speed_tick_ms(1100), Some(61));
        assert_eq!(model.speed_reading().unwrap().percent, Some(100));
        for cap in [SPEED_NATIVE_SATURATION, u32::MAX] {
            model.observe_speed(2, speed_tick_ms(1200), Some(cap));
            assert_eq!(model.speed_reading(), None);
        }
        model.observe_speed(2, speed_tick_ms(1300), Some(0));
        assert_eq!(model.speed_reading().unwrap().percent, None);
        model.observe_speed(0, speed_tick_ms(1400), Some(6));
        assert_eq!(model.speed_reading(), None);
        model.observe_speed(2, speed_tick_ms(1500), None);
        assert_eq!(model.speed_reading(), None);
    }

    #[test]
    fn speed_visibility_rejects_stale_online_context() {
        for melee in [0x0e, 0x0f, 0x10, 0x14, 0x18, 0x25] {
            for mode in [0x0701_0000, 0x0701_0200, 0x8701_ffff] {
                assert!(speed_stage_eligible(true, melee, mode, true, true));
            }
        }
        assert!(speed_stage_eligible(true, 1, 0x0201_0000, true, true));
        for (melee, mode) in [
            (0x0d, 0x0701_0000),
            (0x20, 0x0701_0000),
            (0, 0x0701_0000),
            (0x14, 0),
            (1, 0x0201_0001),
        ] {
            assert!(!speed_stage_eligible(true, melee, mode, true, true));
        }
        for (initialized, stage, ready) in [
            (false, true, true),
            (true, false, true),
            (true, true, false),
        ] {
            assert!(!speed_stage_eligible(
                initialized,
                0x14,
                0x0701_0200,
                stage,
                ready
            ));
        }
        let mut model = SpeedModel::new();
        speed_feed_rate(&mut model, 1, 2000, 600, 10);
        model.observe_speed(1, speed_tick_ms(2010), None);
        assert_eq!(model.speed_reading(), None);
        model.observe_speed(1, speed_tick_ms(5000), Some(300));
        assert_eq!(speed_label(model.speed_reading().unwrap()), "Game --");
    }

    #[test]
    fn speed_stage_tickets_reject_stale_offers_after_rematch_rejection_menu_and_css() {
        let epoch = StageEpoch::new();
        assert_eq!(epoch.speed_stage_token(), None);
        assert!(!epoch.speed_arm_stage(0));
        assert!(!epoch.speed_arm_stage(1));
        let first = epoch.speed_begin_stage();
        assert_eq!(first, 2);
        assert!(epoch.speed_arm_stage(first));
        assert_eq!(epoch.speed_stage_token(), Some(3));
        assert!(!epoch.speed_arm_stage(first));
        // Every attempt invalidates before eligibility, including rejection.
        let rejected = epoch.speed_begin_stage();
        assert_eq!(epoch.speed_stage_token(), None);
        let same_scene_rematch = epoch.speed_begin_stage();
        assert_ne!(rejected, same_scene_rematch);
        assert!(!epoch.speed_arm_stage(rejected));
        assert!(!epoch.speed_arm_stage(first));
        assert!(epoch.speed_arm_stage(same_scene_rematch));
        for _ in ["menu", "CSS"] {
            let offer = epoch.speed_begin_stage();
            epoch.speed_invalidate_stage();
            assert!(!epoch.speed_arm_stage(offer));
            assert_eq!(epoch.speed_stage_token(), None);
        }
    }

    #[test]
    fn speed_fact_fits_match_strip() {
        for cause in [Cause::None, Cause::DelayTooLow, Cause::LagSpikes] {
            for percent in [0, 92, 100] {
                let label = speed_label(Reading {
                    percent: Some(percent),
                    cause,
                });
                assert!(label.len() <= 48);
                assert!(!label.as_bytes().contains(&0));
                let reading = Some(Reading {
                    percent: Some(percent),
                    cause,
                });
                let timing = crate::menu::compact_timing_line(Some(16.67), Some(500.0));
                let timing = speed_timing_line(&timing, reading);
                assert!(timing.ends_with(label.as_str()));
                assert!(timing.len() < timing.capacity());
                for name in ["W".repeat(40), "界".repeat(40)] {
                    let opp = crate::menu::opp_line(&name, "LessLag", "25f", "999 ms", "Ethernet");
                    let opp: crate::menu::StripText = crate::fixed!("Opp: {}", opp);
                    let facts = [opp.as_str(), "* Stealth: reconnect", timing.as_str()];
                    let second: FixedStr<512> = crate::fixed::join(&facts, "   ");
                    assert!(second.ends_with(label.as_str()));
                    assert!(second.len() < second.capacity());
                    for scale in [0.5, 1.0] {
                        let size = crate::menu::compact_font_size(
                            "You: LessLag | 25f | Ethernet",
                            &second,
                            1280.0 * scale,
                            scale,
                            scale,
                        );
                        assert!(size.is_finite() && size > 0.0 && size <= 16.0 * scale);
                    }
                }
            }
        }
        assert_eq!(speed_timing_line("FPS:60.0", None), "FPS:60.0");
        assert_eq!(
            speed_label(Reading {
                percent: None,
                cause: Cause::LagSpikes
            }),
            "Game --"
        );
        assert_eq!(
            speed_label(Reading {
                percent: Some(92),
                cause: Cause::DelayTooLow
            }),
            "Game 92% (delay too low)"
        );
        assert_eq!(
            speed_label(Reading {
                percent: Some(0),
                cause: Cause::LagSpikes
            }),
            "Game 0% (lag spikes)"
        );
        assert!(std::mem::size_of::<SpeedModel>() + std::mem::size_of::<FixedStr<48>>() < 512);
    }

    #[test]
    fn speed_spike_hint_holds_for_two_seconds_after_detection() {
        let mut model = SpeedModel::new();
        speed_feed_rate(&mut model, 1, 2000, 600, 10);
        for ms in (2010..=2100).step_by(10) {
            model.observe_speed(1, speed_tick_ms(ms), Some(120));
        }
        assert_eq!(model.speed_reading().unwrap().cause, Cause::LagSpikes);
        for ms in (2110..=4000).step_by(10) {
            model.observe_speed(1, speed_tick_ms(ms), Some(((ms - 100) * 60 / 1000) as u32));
        }
        assert_eq!(model.speed_reading().unwrap().cause, Cause::LagSpikes);
        for ms in (4010..=5000).step_by(10) {
            model.observe_speed(1, speed_tick_ms(ms), Some(((ms - 100) * 60 / 1000) as u32));
        }
        assert_eq!(model.speed_reading().unwrap().cause, Cause::None);
    }

    // Retained calibration4 samples, every third poll; host monotonic ns are
    // projected to SystemTick units. This exercises model semantics, not UI
    // sampling or a fresh runtime. Full derivation and SHA256 receipts are kept
    // outside the repo in counter-fix/model/fixture-derivation.json.
    const SPEED_RETAINED_80: &[(u64, u32)] = &[
        (0, 0),
        (1188657, 4),
        (2379509, 7),
        (3568102, 11),
        (4761560, 13),
        (5950493, 17),
        (7135271, 19),
        (8324060, 22),
        (9509844, 25),
        (10707534, 29),
        (11893324, 31),
        (13077062, 34),
        (14264194, 37),
        (15451953, 39),
        (16637635, 44),
        (17822043, 47),
        (19010503, 49),
        (20204661, 52),
        (21398587, 55),
        (22583872, 58),
        (23770817, 60),
        (24959510, 64),
        (26152578, 67),
        (27340887, 70),
        (28532566, 73),
        (29732702, 76),
        (30922528, 79),
        (32108238, 81),
        (33296879, 85),
        (34489290, 87),
        (35675106, 91),
        (36860726, 94),
        (38045457, 97),
        (39232276, 99),
        (40428283, 102),
        (41615569, 105),
        (42805866, 109),
        (43990441, 112),
        (45177904, 114),
        (46367540, 118),
        (47554588, 121),
        (48746030, 123),
        (49952502, 126),
        (51139170, 130),
        (52330155, 133),
        (53517702, 135),
        (54704486, 138),
        (55890420, 142),
        (57077406, 144),
        (58266060, 147),
        (59467791, 150),
        (60659155, 153),
        (61843251, 155),
        (63035088, 159),
        (64227238, 162),
        (65417186, 165),
        (66605704, 168),
        (67791628, 171),
        (68979735, 174),
        (70181838, 176),
        (71377353, 179),
        (72559625, 183),
        (73750146, 187),
        (74939110, 189),
        (76139708, 191),
        (77327965, 195),
        (78514121, 198),
        (79717972, 200),
        (80902971, 204),
        (82089368, 207),
        (83274698, 210),
        (84463976, 212),
        (85647469, 216),
        (86840725, 219),
        (88025221, 222),
        (89226664, 225),
        (90409770, 228),
        (91596800, 231),
        (92787908, 234),
        (93976783, 236),
        (95168640, 239),
        (96367137, 242),
        (97555190, 245),
        (98747698, 248),
        (99942417, 252),
        (101128576, 255),
        (102314772, 258),
        (103501754, 260),
        (104684214, 262),
        (105869483, 267),
        (107055143, 269),
        (108241498, 272),
        (109437891, 275),
        (110628529, 278),
        (111819775, 281),
        (113005090, 284),
        (114193372, 286),
        (115383649, 290),
        (116573104, 292),
        (117761030, 295),
    ];
    const SPEED_RETAINED_250: &[(u64, u32)] = &[
        (0, 0),
        (1197709, 4),
        (2390732, 8),
        (3584049, 11),
        (4781543, 15),
        (5973312, 19),
        (7167098, 23),
        (8353737, 26),
        (9540197, 29),
        (10728904, 34),
        (11917334, 38),
        (13114207, 41),
        (14312288, 44),
        (15502013, 49),
        (16689770, 53),
        (17876179, 56),
        (19061121, 59),
        (20249807, 64),
        (21439816, 67),
        (22623914, 71),
        (23825890, 74),
        (25013016, 79),
        (26202140, 82),
        (27389267, 85),
        (28577919, 85),
        (29776012, 85),
        (30961890, 86),
        (32144775, 90),
        (33337510, 94),
        (34536982, 97),
        (35731201, 100),
        (36925831, 104),
        (38117003, 108),
        (39305569, 111),
        (40507985, 114),
        (41700883, 119),
        (42886268, 123),
        (44080814, 126),
        (45268016, 129),
        (46453720, 133),
        (47639181, 137),
        (48826549, 141),
        (50013713, 144),
        (51200491, 148),
        (52388490, 152),
        (53582509, 156),
        (54768316, 159),
        (55954712, 163),
        (57140257, 167),
        (58331485, 171),
        (59518245, 174),
        (60704527, 177),
        (61889148, 182),
        (63076097, 186),
        (64275497, 189),
        (65460281, 192),
        (66645641, 196),
        (67832573, 200),
        (69020984, 204),
        (70208092, 207),
        (71401825, 212),
        (72588718, 215),
        (73782640, 219),
        (74969874, 222),
        (76155588, 226),
        (77344283, 230),
        (78535603, 234),
        (79728462, 237),
        (80917280, 241),
        (82103749, 245),
        (83298512, 249),
        (84483238, 252),
        (85668503, 255),
        (86853197, 260),
        (88038956, 263),
        (89226760, 267),
        (90411488, 270),
        (91593627, 275),
        (92782441, 278),
        (93979289, 282),
        (95167757, 285),
        (96359784, 290),
        (97547999, 293),
        (98735619, 297),
        (99923267, 300),
        (101114088, 305),
        (102307081, 308),
        (103506485, 312),
        (104694400, 315),
        (105886478, 318),
        (107085034, 323),
        (108272250, 326),
        (109557770, 331),
        (110743106, 334),
        (111932031, 338),
        (113132790, 341),
        (114323652, 345),
        (115542432, 349),
        (116735838, 352),
        (117929335, 357),
        (119123878, 360),
    ];
    const SPEED_RETAINED_500: &[(u64, u32)] = &[
        (0, 0),
        (1188232, 3),
        (2381568, 7),
        (3578004, 11),
        (4769208, 15),
        (5953645, 18),
        (7151766, 22),
        (8344569, 26),
        (9531953, 29),
        (10717551, 34),
        (11907306, 37),
        (13109675, 41),
        (14299485, 44),
        (15489947, 49),
        (16683679, 52),
        (17872756, 56),
        (19058743, 59),
        (20242607, 64),
        (21426780, 67),
        (22624402, 71),
        (23806827, 74),
        (24996988, 77),
        (26183601, 82),
        (27366932, 86),
        (28554961, 89),
        (29745027, 92),
        (30933961, 97),
        (32126739, 97),
        (33330497, 97),
        (34520879, 97),
        (35705439, 97),
        (36898621, 97),
        (38092480, 97),
        (39280940, 97),
        (40471528, 98),
        (41659722, 102),
        (42860062, 106),
        (44049506, 110),
        (45235763, 113),
        (46424241, 116),
        (47610709, 121),
        (48799614, 125),
        (49987867, 129),
        (51172692, 132),
        (52371821, 135),
        (53561514, 140),
        (54752839, 143),
        (55937131, 147),
        (57122485, 150),
        (58306939, 154),
        (59490687, 158),
        (60672002, 162),
        (61854049, 165),
        (63051305, 168),
        (64236072, 173),
        (65421612, 176),
        (66609236, 180),
        (67795132, 183),
        (68984831, 188),
        (70170807, 192),
        (71361447, 195),
        (72555389, 198),
        (73745421, 203),
        (74934891, 207),
        (76116997, 210),
        (77307007, 213),
        (78494404, 218),
        (79677407, 221),
        (80861476, 225),
        (82060228, 228),
        (83250138, 232),
        (84441487, 236),
        (85627310, 240),
        (86816388, 243),
        (88005376, 246),
        (89188835, 251),
        (90381726, 255),
        (91566893, 258),
        (92765353, 261),
        (93950291, 266),
        (95134760, 270),
        (96319003, 273),
        (97513149, 276),
        (98698928, 281),
        (99887859, 284),
        (101073166, 288),
        (102271795, 291),
        (103457811, 295),
        (104653237, 299),
        (105839220, 303),
        (107024850, 306),
        (108218039, 310),
        (109421606, 314),
        (110616765, 318),
        (111820694, 321),
        (113010670, 325),
        (114202094, 329),
        (115389861, 333),
        (116574339, 336),
        (117758812, 339),
        (118942209, 344),
        (120138459, 347),
        (121325525, 351),
        (122532142, 354),
        (123723593, 359),
        (124937105, 362),
        (126128423, 365),
        (127320253, 370),
    ];

    #[test]
    fn speed_retained_steady80_has_delay_without_false_spikes() {
        let mut model = SpeedModel::new();
        for &(tick, frames) in SPEED_RETAINED_80 {
            model.observe_speed(1, tick, Some(frames));
            assert_ne!(model.speed_reading().unwrap().cause, Cause::LagSpikes);
        }
        let reading = model.speed_reading().unwrap();
        assert!((78..=82).contains(&reading.percent.unwrap()));
        assert_eq!(reading.cause, Cause::DelayTooLow);
    }

    #[test]
    fn speed_retained_250_and_500ms_gaps_spike_then_recover() {
        for samples in [SPEED_RETAINED_250, SPEED_RETAINED_500] {
            let mut model = SpeedModel::new();
            let mut saw_spike = false;
            for &(tick, frames) in samples {
                model.observe_speed(1, tick, Some(frames));
                saw_spike |= model.speed_reading().unwrap().cause == Cause::LagSpikes;
            }
            assert!(saw_spike);
            let reading = model.speed_reading().unwrap();
            assert_eq!(reading.cause, Cause::None);
            assert!((98..=100).contains(&reading.percent.unwrap()));
        }
    }

    #[test]
    fn speed_object_replacement_and_invalid_reads_restart_same_stage() {
        let mut model = SpeedModel::new();
        let sample = |object_identity, elapsed_frames| {
            Some(SpeedSample {
                object_identity,
                elapsed_frames,
            })
        };
        for ms in (0..=4000).step_by(10) {
            model.observe_speed_sample(
                3,
                speed_tick_ms(ms),
                sample(11, (ms * 552 / 10_000) as u32),
            );
        }
        assert_eq!(model.speed_reading().unwrap().cause, Cause::DelayTooLow);
        model.observe_speed_sample(3, speed_tick_ms(4100), sample(12, 300));
        assert_eq!(
            model.speed_reading().unwrap(),
            Reading {
                percent: None,
                cause: Cause::None
            }
        );
        model.observe_speed_sample(3, speed_tick_ms(5100), sample(12, 360));
        assert_eq!(model.speed_reading().unwrap().percent, Some(100));
        // Inner counter rollback resets Self, but must preserve the new identity.
        model.observe_speed_sample(3, speed_tick_ms(5200), sample(12, 1));
        model.observe_speed_sample(3, speed_tick_ms(6200), sample(12, 61));
        assert_eq!(model.speed_reading().unwrap().percent, Some(100));
        for invalid in [None, sample(0, 65)] {
            model.observe_speed_sample(3, speed_tick_ms(6300), invalid);
            assert_eq!(model.speed_reading(), None);
            model.observe_speed_sample(3, speed_tick_ms(6400), sample(12, 100));
            assert_eq!(model.speed_reading().unwrap().percent, None);
        }
    }

    // Proof3 A indices1955..2162 every third sample. Mirror remains7200 while
    // the owner's cumulative +0xc advances. Raw source SHA7662af70a665f1cb.
    #[test]
    fn speed_retained_proof3_cumulative_clock_continues_past_mirror_hold() {
        const SAMPLES: &[(u64, u32, u32)] = &[
            (0, 7201, 7200),
            (2934261, 7209, 7200),
            (5898814, 7219, 7200),
            (8836584, 7228, 7200),
            (11763459, 7237, 7200),
            (14695329, 7246, 7200),
            (17635824, 7255, 7200),
            (20563207, 7263, 7200),
            (23565587, 7270, 7200),
            (26630269, 7277, 7200),
            (29557990, 7285, 7200),
            (32499533, 7295, 7200),
            (35437104, 7303, 7200),
            (38367478, 7313, 7200),
            (41304748, 7322, 7200),
            (44261652, 7331, 7200),
            (47209630, 7340, 7200),
            (50139864, 7349, 7200),
            (53063879, 7359, 7200),
            (55999035, 7368, 7200),
            (58924199, 7376, 7200),
            (61857677, 7386, 7200),
            (64815391, 7395, 7200),
            (67748853, 7404, 7200),
            (70689186, 7414, 7200),
            (73615284, 7423, 7200),
            (76571064, 7432, 7200),
            (79506255, 7441, 7200),
            (82444639, 7450, 7200),
            (85402557, 7460, 7200),
            (88324032, 7468, 7200),
            (91268825, 7478, 7200),
            (94212452, 7487, 7200),
            (97162820, 7497, 7200),
            (100088513, 7506, 7200),
            (103041107, 7514, 7200),
            (105980174, 7524, 7200),
            (108906087, 7533, 7200),
            (111838180, 7542, 7200),
            (114804450, 7551, 7200),
            (117778769, 7561, 7200),
            (120711433, 7570, 7200),
            (123914869, 7578, 7200),
            (126834295, 7578, 7200),
            (129749631, 7578, 7200),
            (132665686, 7578, 7200),
            (135588157, 7578, 7200),
            (138502370, 7578, 7200),
            (141419683, 7578, 7200),
            (144948220, 7587, 7200),
            (147868918, 7596, 7200),
            (150791977, 7604, 7200),
            (153746755, 7610, 7200),
            (156691380, 7619, 7200),
            (159625175, 7629, 7200),
            (162564399, 7638, 7200),
            (165496366, 7647, 7200),
            (168421587, 7656, 7200),
            (171349669, 7665, 7200),
            (174297129, 7675, 7200),
            (177222728, 7684, 7200),
            (180149518, 7693, 7200),
            (183097049, 7702, 7200),
            (186030872, 7711, 7200),
            (188967311, 7720, 7200),
            (191893649, 7730, 7200),
            (194835417, 7738, 7200),
            (197762015, 7747, 7200),
            (200689578, 7757, 7200),
            (203639277, 7766, 7200),
        ];
        let mut cumulative = SpeedModel::new();
        let mut mirror = SpeedModel::new();
        for &(tick, frames, held) in SAMPLES {
            assert_eq!(held, 7200);
            cumulative.observe_speed(1, tick, Some(frames));
            mirror.observe_speed(1, tick, Some(held));
        }
        let reading = cumulative.speed_reading().unwrap();
        assert!((98..=100).contains(&reading.percent.unwrap()));
        assert_eq!(reading.cause, Cause::None);
        assert_eq!(mirror.speed_reading().unwrap().percent, Some(0));
        assert_eq!(mirror.speed_reading().unwrap().cause, Cause::LagSpikes);
    }

    #[test]
    fn speed_allocation_probe_detects_an_allocation() {
        SPEED_ALLOCATIONS.with(|count| count.set(0));
        SPEED_COUNT_ENABLED.with(|enabled| enabled.set(true));
        let allocation = std::hint::black_box(Box::new([7u8; 64]));
        SPEED_COUNT_ENABLED.with(|enabled| enabled.set(false));
        let allocations = SPEED_ALLOCATIONS.with(Cell::get);
        assert!(allocations >= 1);
        assert_eq!(allocation[0], 7);
    }

    #[test]
    fn speed_hook_allocation_and_lock_probe() {
        // This is the pure model/formatting probe. The unchanged source callgraph
        // gate and paired runtime allocation evidence remain separate checks.
        SPEED_ALLOCATIONS.with(|count| count.set(0));
        SPEED_COUNT_ENABLED.with(|enabled| enabled.set(true));
        let mut model = SpeedModel::new();
        let epoch = StageEpoch::new();
        let ticket = epoch.speed_begin_stage();
        let armed = epoch.speed_arm_stage(ticket);
        let mut labels = 0usize;
        for ms in (0..=10_000).step_by(10) {
            model.observe_speed_sample(
                epoch.speed_stage_token().unwrap(),
                speed_tick_ms(ms),
                Some(SpeedSample {
                    object_identity: 11,
                    elapsed_frames: (ms * 552 / 10_000) as u32,
                }),
            );
            if let Some(reading) = model.speed_reading() {
                labels += speed_label(reading).len();
            }
        }
        epoch.speed_invalidate_stage();
        model.observe_speed(0, speed_tick_ms(10_010), None);
        SPEED_COUNT_ENABLED.with(|enabled| enabled.set(false));
        let allocations = SPEED_ALLOCATIONS.with(Cell::get);
        assert!(armed && labels > 0);
        assert_eq!(
            allocations, 0,
            "pure speed observation/label heap allocations"
        );
        println!(
            "speed_model_bytes={} speed_label_bytes={} heap_allocations={allocations}",
            std::mem::size_of::<SpeedModel>(),
            std::mem::size_of::<FixedStr<48>>()
        );
    }
}
