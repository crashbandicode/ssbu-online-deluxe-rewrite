//! Bounded CPU-side API spans. These do not measure GPU execution or scanout.
use crate::timing::{Finish, DURATION_TICKS, TICK_HZ};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
};
/// 65536 × 32 B. The initial array is all zero bytes so it lives in `.bss`,
/// not in the NRO file. At about 18 rows per frame this holds a 30 s capture.
pub const CAPACITY: usize = 65536;
// 30 s at 60 frames and about 18 rows per frame must fit without a capacity finish.
const _: () = assert!(CAPACITY >= 1800 * 18);
pub const KIND_INDEX: u8 = 12;
pub const KIND_PRESENT_ENTRY: u8 = 13;
pub const KIND_LATE_FLUSH: u8 = 14;
pub const KIND_RECORD_FIGHTERS: u8 = 15;
pub const KIND_POST_SCENE: u8 = 16;
pub const KIND_OUTSTANDING: u8 = 17;
/// `nvnSyncWait` with an infinite timeout on a sync that is not the texture sync.
/// `index` stays -1. `result` is `NVNsyncWaitResult`. Duration is `end - begin`.
/// This is not kind 4 (the timeout-0 render poll).
pub const KIND_SYNC_INFINITE_OTHER: u8 = 27;
/// One embedded backend log containing `Resizing`. `index` is the occurrence
/// count. Both tick columns are the sample tick.
pub const KIND_MENU_RESIZE: u8 = 28;
/// LessLag flush plan while a finite API trace is active. `index` is `calls`.
/// `result` bits: 0 store executed (`+0x1d18` was non-zero before the write),
/// 1 `arm_drain`, 8..15 generation. Both ticks are the sample tick.
/// Emitted only when `lesslag_fields` is set and the flush object is non-null.
pub const KIND_FLUSH_PLAN: u8 = 29;
/// Pointer halves for that flush object. Not a guessed global.
pub const KIND_FLUSH_OBJECT: u8 = 30;
/// Four raw bytes in `index`, low to high: pre `+0x1d18` u8, post `+0x1d18` u8,
/// pre `+0x1d28` u8, post `+0x1d28` u8. `result` stays -1.
pub const KIND_FLUSH_NARROW: u8 = 31;
/// `index` is pre `+0x1d20` u32 bits. `result` is post `+0x1d20` u32 bits.
pub const KIND_FLUSH_WORD: u8 = 32;
/// Pointer halves for pre `+0x1d30` u64.
pub const KIND_FLUSH_WIDE_PRE: u8 = 33;
/// Pointer halves for post `+0x1d30` u64.
pub const KIND_FLUSH_WIDE_POST: u8 = 34;
/// Dispatcher backoff sleeps since the previous tracked present. Point sample
/// at the present call: `index` is the sleep count, `result` the summed actual
/// sleep time in microseconds (tick before and after `SleepThread`).
pub const KIND_BACKOFF_FRAME_SLEEP: u8 = 60;
/// Same frame: `index` is the longest single actual sleep in microseconds,
/// `result` the number of idle streaks that started in that frame.
pub const KIND_BACKOFF_FRAME_PEAK: u8 = 61;
/// One SDK `ThreadType` pointer (halves) seen by a CSS capture hook, once per
/// thread per capture. Hash it with [`thread_label`] to match the other rows.
pub const KIND_THREAD_SETUP_POINTER: u8 = 62;
/// Same thread, read-only: `index` is `nn::os::GetThreadPriority`, `result`
/// is [`core_info`] (ideal core and the low 24 affinity bits). Nothing is set.
pub const KIND_THREAD_SETUP_INFO: u8 = 63;
/// The UI aggregate's child job was registered with the game dispatcher.
/// `index` is [`dispatch_info`] (worker and queue priority), `result` the
/// hashed label of the registering thread. Point sample.
pub const KIND_UI_DISPATCH: u8 = 64;
/// Dependency wait `main+0x354ccd0`, any thread: `begin`/`end` span, `index`
/// the awaited counter's offset from the game text base (-1 when outside
/// the image), `result` the hashed thread label. MainThread's wait on
/// `0x5333160` is the UI barrier; the same label as kind 77 is a nested wait
/// inside the UI job.
pub const KIND_DEPENDENCY_WAIT: u8 = 66;
/// The UI job `main+0x375e260` from entry to return on its worker: span,
/// `index` -1, `result` the hashed thread label. Wall time, not CPU time.
pub const KIND_UI_JOB: u8 = 77;

/// Hashed thread label shared by kinds 12–17 and 62–77. Pure so a reader can
/// recompute it from the kind 62 pointer.
pub const fn thread_label(thread: u64) -> i32 {
    let mixed = thread
        .wrapping_add(0x9E37_79B1_85EB_CA87)
        .wrapping_mul(0x9E37_79B1_85EB_CA87)
        .rotate_left(27);
    (mixed as u32 & 0x7fff_ffff) as i32
}

/// Ideal core in the low byte, affinity mask bits 0..23 above it. A negative
/// or oversized ideal core is clamped to 255 so the mask stays readable.
pub const fn core_info(ideal_core: i32, affinity_mask: u64) -> i32 {
    let core = if ideal_core < 0 || ideal_core > 254 {
        255
    } else {
        ideal_core as u32
    };
    bits_u32(core | ((affinity_mask as u32 & 0x00ff_ffff) << 8))
}

pub const fn core_info_parts(info: i32) -> (u8, u32) {
    let word = u32_bits(info);
    (word as u8, word >> 8)
}

/// Dispatcher worker in the low byte, queue priority in the next.
pub const fn dispatch_info(worker: u32, priority: u32) -> i32 {
    bits_u32((worker & 0xff) | ((priority & 0xff) << 8))
}

/// Ticks to whole microseconds, saturating at `i32::MAX`.
pub const fn ticks_to_us(ticks: u64) -> i32 {
    // 19.2 MHz is not a whole number of ticks per microsecond; widen instead.
    let us = (ticks as u128 * 1_000_000) / TICK_HZ as u128;
    if us > i32::MAX as u128 {
        i32::MAX
    } else {
        us as i32
    }
}

/// Counter offset from the game text base, or -1 when it is not inside the
/// first 2 GiB of the image.
pub const fn image_offset(address: u64, text_base: u64) -> i32 {
    let offset = address.wrapping_sub(text_base);
    if offset < 0x8000_0000 {
        offset as i32
    } else {
        -1
    }
}

/// The two per-frame backoff rows for one tracked present.
pub const fn backoff_frame_events(
    tick: u64,
    sleeps: u64,
    sleep_ticks: u64,
    max_sleep_ticks: u64,
    streaks: u64,
) -> [Event; 2] {
    [
        Event {
            begin: tick,
            end: tick,
            kind: KIND_BACKOFF_FRAME_SLEEP,
            index: if sleeps > i32::MAX as u64 {
                i32::MAX
            } else {
                sleeps as i32
            },
            result: ticks_to_us(sleep_ticks),
        },
        Event {
            begin: tick,
            end: tick,
            kind: KIND_BACKOFF_FRAME_PEAK,
            index: ticks_to_us(max_sleep_ticks),
            result: if streaks > i32::MAX as u64 {
                i32::MAX
            } else {
                streaks as i32
            },
        },
    ]
}

/// Count still held after one tracked present returns.
/// A real zero stays zero. An unmatched present becomes negative.
#[cfg_attr(not(any(test, target_os = "switch")), allow(dead_code))]
pub const fn outstanding_after_present(before: i64) -> i64 {
    before.wrapping_sub(1)
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Event {
    pub begin: u64,
    pub end: u64,
    /// 1=present, 2=acquire, 3=texture CPU wait, 4=render poll, 5=dispatch flush.
    /// 6=HID poll/marker (index=24-bit serial, result=left Y),
    /// 7=same poll sampling counter (index=serial, result=counter low32).
    /// 12=index selection, 13=present entry, 14=late flush, 15=record fighters,
    /// 16=post-scene, 17=tracked-window outstanding after present return.
    /// Kinds 18–26 and 35–38 are retired menu-probe ids; archived traces still decode them.
    /// 27=infinite wait on a non-texture sync. 28=embedded geometry resize.
    /// 29–34=LessLag flush field pre/post bytes.
    /// Kinds 12–17 reuse these columns: `index` is the phase value and `result`
    /// is the hashed thread label. Kinds 6 and 7 stay the HID records.
    /// 60–64, 66 and 77 are the CSS capture rows (see the constants above).
    pub kind: u8,
    /// Texture index for present/acquire; phase value for kinds 12–17;
    /// -1 for events without an index.
    pub index: i32,
    /// Native wait result, or the hashed thread label for kinds 12–17.
    /// -1 when no trustworthy result exists.
    pub result: i32,
}
struct State {
    id: u64,
    first: u64,
    len: usize,
    phase: u8,
    reason: Finish,
    events: [Event; CAPACITY],
}
pub struct Trace {
    state: Mutex<State>,
    active: AtomicBool,
    epoch: AtomicU64,
    skipped: AtomicU64,
}
pub struct Snapshot {
    pub id: u64,
    pub reason: Finish,
    pub skipped: u64,
    pub events: Vec<Event>,
}
impl Default for Trace {
    fn default() -> Self {
        Self::new()
    }
}
impl Trace {
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(State {
                id: 0,
                first: 0,
                len: 0,
                phase: 0,
                reason: Finish::Aborted,
                // All-zero so the static stays in .bss; only `events[..len]` is read.
                events: [Event {
                    begin: 0,
                    end: 0,
                    kind: 0,
                    index: 0,
                    result: 0,
                }; CAPACITY],
            }),
            active: AtomicBool::new(false),
            epoch: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
        }
    }
    pub fn active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
    pub fn begin(&self, id: u64) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.phase != 0 {
            return false;
        }
        state.id = id;
        state.first = 0;
        state.len = 0;
        state.phase = 1;
        self.skipped.store(0, Ordering::Relaxed);
        self.epoch.fetch_add(1, Ordering::Release);
        drop(state);
        self.active.store(true, Ordering::Release);
        true
    }
    /// No allocation, I/O or blocking in a hooked call.
    pub fn observe(&self, event: Event) {
        let epoch = self.epoch.load(Ordering::Acquire);
        if !self.active() {
            return;
        }
        let Ok(mut state) = self.state.try_lock() else {
            self.skipped.fetch_add(1, Ordering::Relaxed);
            return;
        };
        if state.phase != 1 || event.begin < state.id || self.epoch.load(Ordering::Acquire) != epoch
        {
            return;
        }
        if state.len == 0 {
            state.first = event.begin;
        }
        let n = state.len;
        state.events[n] = event;
        state.len += 1;
        let reason = if event.end < event.begin {
            Some(Finish::ClockInvalid)
        } else if event.end.saturating_sub(state.first) >= DURATION_TICKS {
            Some(Finish::Elapsed)
        } else if state.len == CAPACITY {
            Some(Finish::Capacity)
        } else {
            None
        };
        if let Some(reason) = reason {
            state.reason = reason;
            state.phase = 2;
            self.active.store(false, Ordering::Release);
        }
    }
    pub fn finish(&self) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state.phase == 1 {
            state.reason = Finish::Aborted;
            state.phase = 2;
            self.active.store(false, Ordering::Release);
        }
    }
    pub fn take_ready(&self) -> Option<Snapshot> {
        if self.active() {
            return None;
        }
        let mut state = self.state.lock().ok()?;
        if state.phase != 2 {
            return None;
        }
        let snapshot = Snapshot {
            id: state.id,
            reason: state.reason,
            skipped: self.skipped.load(Ordering::Relaxed),
            events: state.events[..state.len].to_vec(),
        };
        state.phase = 0;
        Some(snapshot)
    }
}
pub const fn pointer_halves(pointer: u64) -> (i32, i32) {
    (pointer as u32 as i32, (pointer >> 32) as u32 as i32)
}

pub const fn pointer_from_halves(low: i32, high: i32) -> u64 {
    (low as u32 as u64) | ((high as u32 as u64) << 32)
}

/// Bit pattern of a raw `u32`. The CSV column is signed; decode with [`u32_bits`].
pub const fn bits_u32(value: u32) -> i32 {
    value as i32
}

pub const fn u32_bits(value: i32) -> u32 {
    value as u32
}

/// One read of the four LessLag flush slots. Widths are `u8`, `u32`, `u8`, `u64`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlushFields {
    pub slot_18: u8,
    pub slot_20: u32,
    pub slot_28: u8,
    pub slot_30: u64,
}

/// Plan bits recorded with one flush sample. `store_ran` is the pre `slot_18` gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlushPlanDiagnostic {
    pub calls: i32,
    pub store_ran: bool,
    pub arm_drain: bool,
    pub generation: u8,
    pub object: u64,
}

/// Low to high: pre `+0x1d18`, post `+0x1d18`, pre `+0x1d28`, post `+0x1d28`.
pub const fn flush_narrow_word(pre: FlushFields, post: FlushFields) -> u32 {
    (pre.slot_18 as u32)
        | ((post.slot_18 as u32) << 8)
        | ((pre.slot_28 as u32) << 16)
        | ((post.slot_28 as u32) << 24)
}

pub const fn flush_narrow_parts(word: u32) -> (u8, u8, u8, u8) {
    (
        word as u8,
        (word >> 8) as u8,
        (word >> 16) as u8,
        (word >> 24) as u8,
    )
}

/// Bit 0 store executed, bit 1 `arm_drain`, bits 8..15 generation.
pub const fn flush_plan_result(plan: FlushPlanDiagnostic) -> i32 {
    let store = plan.store_ran as u32;
    let drain = plan.arm_drain as u32;
    bits_u32(store | (drain << 1) | ((plan.generation as u32) << 8))
}

pub const fn flush_plan_parts(result: i32) -> (bool, bool, u8) {
    let word = u32_bits(result);
    (word & 1 != 0, (word >> 1) & 1 != 0, (word >> 8) as u8)
}

/// Point samples for one LessLag field flush. Widths stay in the documented columns.
pub const fn flush_field_events(
    tick: u64,
    plan: FlushPlanDiagnostic,
    pre: FlushFields,
    post: FlushFields,
) -> [Event; 6] {
    let (object_low, object_high) = pointer_halves(plan.object);
    let (pre_low, pre_high) = pointer_halves(pre.slot_30);
    let (post_low, post_high) = pointer_halves(post.slot_30);
    [
        Event {
            begin: tick,
            end: tick,
            kind: KIND_FLUSH_PLAN,
            index: plan.calls,
            result: flush_plan_result(plan),
        },
        Event {
            begin: tick,
            end: tick,
            kind: KIND_FLUSH_OBJECT,
            index: object_low,
            result: object_high,
        },
        Event {
            begin: tick,
            end: tick,
            kind: KIND_FLUSH_NARROW,
            index: bits_u32(flush_narrow_word(pre, post)),
            result: -1,
        },
        Event {
            begin: tick,
            end: tick,
            kind: KIND_FLUSH_WORD,
            index: bits_u32(pre.slot_20),
            result: bits_u32(post.slot_20),
        },
        Event {
            begin: tick,
            end: tick,
            kind: KIND_FLUSH_WIDE_PRE,
            index: pre_low,
            result: pre_high,
        },
        Event {
            begin: tick,
            end: tick,
            kind: KIND_FLUSH_WIDE_POST,
            index: post_low,
            result: post_high,
        },
    ]
}

pub fn contains_resizing(bytes: &[u8]) -> bool {
    bytes.windows(8).any(|window| window == b"Resizing")
}

pub const fn resize_event(tick: u64, count: u64) -> Event {
    Event {
        begin: tick,
        end: tick,
        kind: KIND_MENU_RESIZE,
        index: if count > i32::MAX as u64 {
            -1
        } else {
            count as i32
        },
        result: -1,
    }
}

pub fn write_csv(
    mut out: impl Write,
    data: &Snapshot,
    build: &str,
    label: &str,
    engine: &str,
) -> io::Result<()> {
    if [build, label, engine]
        .iter()
        .any(|v| v.contains(['\r', '\n']))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "multiline API trace metadata",
        ));
    }
    writeln!(out, "# schema=1\n# metric=cpu_api_spans\n# tick_hz={TICK_HZ}\n# id={}\n# build={build}\n# label={label}\n# engine={engine}\n# reason={}\n# skipped={}\n# event_count={}",
        data.id,data.reason.label(),data.skipped,data.events.len())?;
    writeln!(out, "event,kind,begin_tick,end_tick,index,result")?;
    for (n, event) in data.events.iter().enumerate() {
        writeln!(
            out,
            "{n},{},{},{},{},{}",
            event.kind, event.begin, event.end, event.index, event.result
        )?;
    }
    writeln!(out, "# complete=1")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_spans_remain_valid_and_finish_at_duration() {
        // 2 MiB of events: keep the test trace off the test thread's stack.
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        assert!(trace.begin(1));
        assert!(!trace.begin(12));
        trace.observe(Event {
            begin: 20,
            end: 30,
            kind: 1,
            ..Event::default()
        });
        trace.observe(Event {
            begin: 10,
            end: 40,
            kind: 5,
            ..Event::default()
        });
        trace.observe(Event {
            begin: DURATION_TICKS,
            end: DURATION_TICKS + 20,
            kind: 1,
            ..Event::default()
        });
        let data = trace.take_ready().unwrap();
        assert_eq!(
            (data.id, data.reason, data.events.len()),
            (1, Finish::Elapsed, 3)
        );
        assert!(trace.begin(12));
    }
    #[test]
    fn capacity_abort_contention_and_invalid_time_are_explicit() {
        // 2 MiB of events: keep the test trace off the test thread's stack.
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        trace.begin(0);
        let guard = trace.state.lock().unwrap();
        trace.observe(Event::default());
        drop(guard);
        for _ in 0..CAPACITY + 1 {
            trace.observe(Event::default());
        }
        let data = trace.take_ready().unwrap();
        assert_eq!(
            (data.reason, data.events.len(), data.skipped),
            (Finish::Capacity, CAPACITY, 1)
        );
        trace.begin(2);
        trace.observe(Event {
            begin: 2,
            end: 1,
            ..Event::default()
        });
        assert_eq!(trace.take_ready().unwrap().reason, Finish::ClockInvalid);
        trace.begin(3);
        trace.finish();
        assert_eq!(trace.take_ready().unwrap().reason, Finish::Aborted);
    }
    #[test]
    fn present_return_keeps_a_real_zero_and_an_unmatched_present() {
        assert_eq!(outstanding_after_present(1), 0);
        assert_eq!(outstanding_after_present(0), -1);
        assert_eq!(outstanding_after_present(2), 1);
    }
    #[test]
    fn pointer_halves_round_trip_without_ending_the_capture() {
        let pointer = 0x0000_0001_8000_0000;
        let (low, high) = pointer_halves(pointer);
        assert_eq!(pointer_from_halves(low, high), pointer);
        assert_eq!(pointer_from_halves(-1, -1), u64::MAX);
        // 2 MiB of events: keep the test trace off the test thread's stack.
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        assert!(trace.begin(7));
        trace.observe(Event {
            begin: 1_000,
            end: 1_000,
            kind: KIND_FLUSH_OBJECT,
            index: low,
            result: high,
        });
        assert!(trace.take_ready().is_none());
        trace.finish();
        let data = trace.take_ready().unwrap();
        assert_eq!((data.reason, data.events.len()), (Finish::Aborted, 1));
        assert_eq!(
            pointer_from_halves(data.events[0].index, data.events[0].result),
            pointer
        );
    }
    #[test]
    fn resize_stays_its_own_point_sample() {
        assert!(contains_resizing(b"Resizing Vertex Buffer to Size: 12\n"));
        assert!(!contains_resizing(
            b"Initializing Vertex Buffer to Size: 12\n"
        ));
        let resize = resize_event(9, 2);
        assert_eq!(
            (resize.kind, resize.index, resize.begin, resize.end),
            (KIND_MENU_RESIZE, 2, 9, 9)
        );
        let wait = Event {
            begin: 10,
            end: 50,
            kind: KIND_SYNC_INFINITE_OTHER,
            index: -1,
            result: 1,
        };
        assert_eq!(wait.end - wait.begin, 40);
        let kinds = [KIND_SYNC_INFINITE_OTHER, KIND_MENU_RESIZE];
        let older = [1u8, 2, 3, 4, 5, 6, 7, 12, 13, 14, 15, 16, 17];
        for kind in kinds {
            assert!(!older.contains(&kind));
        }
        // 2 MiB of events: keep the test trace off the test thread's stack.
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        assert!(trace.begin(8));
        trace.observe(resize);
        assert!(trace.take_ready().is_none());
        trace.finish();
        assert_eq!(trace.take_ready().unwrap().events.len(), 1);
    }

    #[test]
    fn css_capture_rows_pack_and_unpack() {
        // The label is the same function the runtime used for kinds 12–17.
        assert_eq!(thread_label(0), thread_label(0));
        assert_ne!(thread_label(0x1000), thread_label(0x1010));
        assert!(thread_label(u64::MAX) >= 0);
        assert_eq!(core_info_parts(core_info(1, 0x2)), (1, 0x2));
        assert_eq!(
            core_info_parts(core_info(-1, 0xffff_ffff)),
            (255, 0x00ff_ffff)
        );
        assert_eq!(dispatch_info(1, 1), 0x101);
        assert_eq!(dispatch_info(0x1ff, 0x2ff), 0xffff);
        assert_eq!(ticks_to_us(TICK_HZ), 1_000_000);
        assert_eq!(ticks_to_us(19), 0);
        assert_eq!(ticks_to_us(20), 1);
        assert_eq!(ticks_to_us(u64::MAX), i32::MAX);
        let text_base = 0x0800_0000_0000u64;
        assert_eq!(
            image_offset(text_base + 0x0533_3160, text_base),
            0x0533_3160
        );
        assert_eq!(image_offset(0x10, text_base), -1);
        assert_eq!(image_offset(text_base + 0x8000_0000, text_base), -1);
        let rows = backoff_frame_events(9_000, 80, 4 * TICK_HZ / 1000, 960, u64::MAX);
        assert_eq!(rows[0].kind, KIND_BACKOFF_FRAME_SLEEP);
        assert_eq!((rows[0].index, rows[0].result), (80, 4_000));
        assert_eq!(rows[1].kind, KIND_BACKOFF_FRAME_PEAK);
        assert_eq!((rows[1].index, rows[1].result), (50, i32::MAX));
        assert_eq!((rows[0].begin, rows[1].end), (9_000, 9_000));
        let new_kinds = [
            KIND_BACKOFF_FRAME_SLEEP,
            KIND_BACKOFF_FRAME_PEAK,
            KIND_THREAD_SETUP_POINTER,
            KIND_THREAD_SETUP_INFO,
            KIND_UI_DISPATCH,
            KIND_DEPENDENCY_WAIT,
            KIND_UI_JOB,
        ];
        let older = [
            1u8, 2, 3, 4, 5, 6, 7, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
            28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38,
        ];
        for kind in new_kinds {
            assert!(!older.contains(&kind));
        }
    }

    #[test]
    fn flush_field_words_keep_width() {
        let pre = FlushFields {
            slot_18: 0x7f,
            slot_20: 0x8000_0001,
            slot_28: 0x01,
            slot_30: 0xFFFF_0000_0000_0040,
        };
        let post = FlushFields {
            slot_18: 0x80,
            slot_20: 0x0000_0001,
            slot_28: 0xff,
            slot_30: 0x0000_0001_8000_0001,
        };
        let plan = FlushPlanDiagnostic {
            calls: 1,
            store_ran: true,
            arm_drain: false,
            generation: 7,
            object: 0x0000_0011_c9ca_ec80,
        };
        let events = flush_field_events(9_000, plan, pre, post);
        assert_eq!(events[0].kind, KIND_FLUSH_PLAN);
        assert_eq!((events[0].index, events[0].begin), (1, 9_000));
        assert_eq!(flush_plan_parts(events[0].result), (true, false, 7));
        assert_eq!(
            pointer_from_halves(events[1].index, events[1].result),
            plan.object
        );
        assert_eq!(
            flush_narrow_parts(u32_bits(events[2].index)),
            (0x7f, 0x80, 0x01, 0xff)
        );
        assert!(events[2].index < 0);
        assert_eq!(
            (u32_bits(events[3].index), u32_bits(events[3].result)),
            (pre.slot_20, post.slot_20)
        );
        assert!(events[3].index < 0);
        assert_eq!(
            pointer_from_halves(events[4].index, events[4].result),
            pre.slot_30
        );
        assert_eq!(
            pointer_from_halves(events[5].index, events[5].result),
            post.slot_30
        );
        assert_eq!(
            flush_plan_parts(flush_plan_result(FlushPlanDiagnostic {
                calls: 3,
                store_ran: false,
                arm_drain: true,
                generation: 0,
                object: 0,
            })),
            (false, true, 0)
        );
    }
}
