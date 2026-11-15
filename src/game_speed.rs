//! Native elapsed-match time compared with wall time, independent of presentation.
//!
//! Input frames are the verified native elapsed clock in 60-Hz units, not an
//! unconditional engine-tick serial. The owner can apply native rate/pause state.
//! Callers must supply current-stage eligibility and a fresh nonzero generation.
//! All hint thresholds below are PROVISIONAL pending paired calibration. In
//! particular, integer clock quantization can classify steady 80% as a spike.

use crate::fixed::FixedStr;
use std::sync::atomic::{AtomicU64, Ordering};

pub const SPEED_TICKS_PER_SECOND: u64 = 19_200_000;
pub const SPEED_NATIVE_SATURATION: u32 = 21_599_999;
const SPEED_BIN: u64 = SPEED_TICKS_PER_SECOND / 10;
const SPEED_WINDOW: u64 = SPEED_TICKS_PER_SECOND;
// Provisional approved design values; paired measurements must precede changes.
const SPEED_FAST_WINDOW: u64 = SPEED_TICKS_PER_SECOND / 4;
const SPEED_NO_ADVANCE: u64 = SPEED_TICKS_PER_SECOND / 10;
const SPEED_FAST_PERCENT: u128 = 80;
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
        if let Some(reference) = self.speed_reference(tick, SPEED_FAST_WINDOW) {
            if Self::speed_below(point, reference, SPEED_FAST_PERCENT) {
                self.spike_until = tick.saturating_add(SPEED_SPIKE_HOLD);
            }
        }
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
                    let opp = crate::menu::opp_line(
                        &name, "LessLag", "25f", "999 ms", "Ethernet",
                    );
                    let opp: crate::menu::StripText = crate::fixed!("Opp: {}", opp);
                    let facts = [opp.as_str(), "* Stealth: reconnect", timing.as_str()];
                    let second: FixedStr<512> = crate::fixed::join(&facts, "   ");
                    assert!(second.ends_with(label.as_str()));
                    assert!(second.len() < second.capacity());
                    for scale in [0.5, 1.0] {
                        let size = crate::menu::compact_font_size(
                            "You: LessLag | 25f | Ethernet", &second,
                            1280.0 * scale, scale, scale,
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
            model.observe_speed(
                epoch.speed_stage_token().unwrap(),
                speed_tick_ms(ms),
                Some((ms * 552 / 10_000) as u32),
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
