//! Bounded CPU-side API spans. These do not measure GPU execution or scanout.
use crate::timing::{Finish, TICK_HZ};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
};
/// Rows per drain chunk. Two chunks are the whole resident cost of the trace:
/// 2 × 2048 × 20 B = 80 KiB of `.bss`. At the ~1,100 rows/s measured on the
/// hardware CSS capture a chunk holds about 1.9 s; the diagnostics thread drains
/// every 500 ms, so the capture length is bounded by `DURATION_TICKS`, not by
/// memory (0.3.1–0.3.4 held all 65536 × 32 B rows until the end; 0.3.5 and
/// 0.3.6 used 4096-row chunks).
pub const CHUNK: usize = 2048;
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
/// Stored row: ticks relative to the chunk's base and the span length, both
/// 32-bit, so a row is 20 B instead of 32. The base is set `CHUNK_HALF_RANGE`
/// before the chunk's first row, so a span that began up to 111 s before it
/// (or a row up to 111 s after it) still packs; a match capture of many
/// minutes therefore needs no 64-bit column (before the match capture the
/// base was the capture start, which limited a capture to 223 s).
#[derive(Clone, Copy)]
#[repr(C)]
struct Packed {
    begin: u32,
    span: u32,
    index: i32,
    result: i32,
    kind: u8,
}
const PACKED_ZERO: Packed = Packed {
    begin: 0,
    span: 0,
    index: 0,
    result: 0,
    kind: 0,
};
const CHUNK_HALF_RANGE: u64 = 1 << 31;
struct Chunk {
    len: usize,
    /// Tick the rows' `begin` is relative to; set with the first row.
    base: u64,
    rows: [Packed; CHUNK],
}
struct State {
    id: u64,
    duration: u64,
    first: u64,
    /// Rows accepted since `begin`, drained or not.
    total: usize,
    phase: u8,
    reason: Finish,
    /// Index of the chunk hooks write into; the other one is being drained.
    active: usize,
    chunks: [Chunk; 2],
}
pub struct Trace {
    state: Mutex<State>,
    active: AtomicBool,
    epoch: AtomicU64,
    skipped: AtomicU64,
}
/// Identity and counts of a finished capture. Its rows were handed out by
/// `drain` while it ran; nothing is copied at the end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Summary {
    pub id: u64,
    pub reason: Finish,
    pub skipped: u64,
    pub event_count: usize,
}
/// A whole capture in memory: only tests and the single-shot `write_csv` use it.
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
                duration: 0,
                first: 0,
                total: 0,
                phase: 0,
                reason: Finish::Aborted,
                active: 0,
                // All zero so the static lives in `.bss`.
                chunks: [
                    Chunk {
                        len: 0,
                        base: 0,
                        rows: [PACKED_ZERO; CHUNK],
                    },
                    Chunk {
                        len: 0,
                        base: 0,
                        rows: [PACKED_ZERO; CHUNK],
                    },
                ],
            }),
            active: AtomicBool::new(false),
            epoch: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
        }
    }
    pub fn active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
    /// The capture ends `duration_ticks` after its first row.
    pub fn begin(&self, id: u64, duration_ticks: u64) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.phase != 0 {
            return false;
        }
        state.id = id;
        state.duration = duration_ticks;
        state.first = 0;
        state.total = 0;
        state.active = 0;
        state.chunks[0].len = 0;
        state.chunks[1].len = 0;
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
        if state.total == 0 {
            state.first = event.begin;
        }
        // A full chunk, or a row whose `begin` is outside the chunk's 32-bit
        // range (the game stalled for minutes), hands over to the other chunk
        // only once that has been drained; until then the row is counted as
        // skipped, never blocked on.
        let mut active = state.active;
        let current = &state.chunks[active];
        if current.len == CHUNK
            || (current.len > 0 && event.begin.wrapping_sub(current.base) > u64::from(u32::MAX))
        {
            let other = active ^ 1;
            if state.chunks[other].len != 0 {
                self.skipped.fetch_add(1, Ordering::Relaxed);
                return;
            }
            state.active = other;
            active = other;
        }
        if state.chunks[active].len == 0 {
            state.chunks[active].base = event.begin.wrapping_sub(CHUNK_HALF_RANGE);
        }
        let rel_begin = event.begin.wrapping_sub(state.chunks[active].base);
        let span = event.end.wrapping_sub(event.begin);
        let clock_invalid = event.end < event.begin || span > u64::from(u32::MAX);
        let packed = Packed {
            begin: u32::try_from(rel_begin).unwrap_or(u32::MAX),
            span: u32::try_from(span).unwrap_or(u32::MAX),
            index: event.index,
            result: event.result,
            kind: event.kind,
        };
        let n = state.chunks[active].len;
        state.chunks[active].rows[n] = packed;
        state.chunks[active].len = n + 1;
        state.total += 1;
        let reason = if clock_invalid {
            Some(Finish::ClockInvalid)
        } else if event.end.saturating_sub(state.first) >= state.duration {
            Some(Finish::Elapsed)
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
        self.finish_with(Finish::Aborted);
    }
    /// Background end with a stated reason (scene end, abort).
    pub fn finish_with(&self, reason: Finish) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state.phase == 1 {
            state.reason = reason;
            state.phase = 2;
            self.active.store(false, Ordering::Release);
        }
    }
    /// Diagnostics thread only. Appends finished rows to `out` in capture
    /// order: the chunk hooks are not writing, then, once the capture has
    /// finished, the active chunk as well. The lock is held for the copy only;
    /// a hook that collides with it counts as skipped instead of waiting.
    /// Returns the number of rows moved.
    pub fn drain(&self, out: &mut Vec<Event>) -> usize {
        let Ok(mut state) = self.state.lock() else {
            return 0;
        };
        if state.phase == 0 {
            return 0;
        }
        let inactive = state.active ^ 1;
        let mut moved = take_chunk(&mut state.chunks[inactive], out);
        if state.phase == 2 {
            let active = state.active;
            moved += take_chunk(&mut state.chunks[active], out);
        }
        moved
    }
    /// The finished capture's identity and counts. Call `drain` first: rows
    /// still in the chunks are discarded here.
    pub fn take_ready(&self) -> Option<Summary> {
        if self.active() {
            return None;
        }
        let mut state = self.state.lock().ok()?;
        if state.phase != 2 {
            return None;
        }
        let summary = Summary {
            id: state.id,
            reason: state.reason,
            skipped: self.skipped.load(Ordering::Relaxed),
            event_count: state.total,
        };
        state.chunks[0].len = 0;
        state.chunks[1].len = 0;
        state.active = 0;
        state.total = 0;
        state.phase = 0;
        Some(summary)
    }
}
fn take_chunk(chunk: &mut Chunk, out: &mut Vec<Event>) -> usize {
    let len = chunk.len;
    let base = chunk.base;
    out.extend(chunk.rows[..len].iter().map(|row| {
        let begin = base.wrapping_add(u64::from(row.begin));
        Event {
            begin,
            end: begin + u64::from(row.span),
            kind: row.kind,
            index: row.index,
            result: row.result,
        }
    }));
    chunk.len = 0;
    len
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

fn single_line(values: &[&str]) -> io::Result<()> {
    if values.iter().any(|v| v.contains(['\r', '\n'])) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "multiline API trace metadata",
        ));
    }
    Ok(())
}
/// The CSV is streamed: header at arm time, rows as chunks drain, footer when
/// the capture finishes. `tools/analyze_render_trace.py` reads `# key=value`
/// lines anywhere and requires `# complete=1` last, so engine, reason, skipped
/// and event_count move to the footer where they are known.
pub fn write_csv_header(mut out: impl Write, id: u64, build: &str, label: &str) -> io::Result<()> {
    single_line(&[build, label])?;
    writeln!(
        out,
        "# schema=1\n# metric=cpu_api_spans\n# tick_hz={TICK_HZ}\n# id={id}\n# build={build}\n# label={label}"
    )?;
    writeln!(out, "event,kind,begin_tick,end_tick,index,result")
}
/// `next` is the running row number across chunks.
pub fn write_csv_rows(mut out: impl Write, next: &mut usize, events: &[Event]) -> io::Result<()> {
    for event in events {
        writeln!(
            out,
            "{},{},{},{},{},{}",
            *next, event.kind, event.begin, event.end, event.index, event.result
        )?;
        *next += 1;
    }
    Ok(())
}
pub fn write_csv_footer(mut out: impl Write, engine: &str, summary: &Summary) -> io::Result<()> {
    single_line(&[engine])?;
    writeln!(
        out,
        "# engine={engine}\n# reason={}\n# skipped={}\n# event_count={}\n# complete=1",
        summary.reason.label(),
        summary.skipped,
        summary.event_count
    )
}
/// Whole capture in one call (tests and tools); same bytes as the streamed form.
pub fn write_csv(
    mut out: impl Write,
    data: &Snapshot,
    build: &str,
    label: &str,
    engine: &str,
) -> io::Result<()> {
    write_csv_header(&mut out, data.id, build, label)?;
    let mut next = 0;
    write_csv_rows(&mut out, &mut next, &data.events)?;
    write_csv_footer(
        &mut out,
        engine,
        &Summary {
            id: data.id,
            reason: data.reason,
            skipped: data.skipped,
            event_count: data.events.len(),
        },
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing::DURATION_TICKS;
    fn drained(trace: &Trace) -> Vec<Event> {
        let mut out = Vec::new();
        trace.drain(&mut out);
        out
    }
    #[test]
    fn nested_spans_remain_valid_and_finish_at_duration() {
        // 160 KiB of rows: keep the test trace off the test thread's stack.
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        assert!(trace.begin(1, DURATION_TICKS));
        assert!(!trace.begin(12, DURATION_TICKS));
        let rows = [
            Event {
                begin: 20,
                end: 30,
                kind: 1,
                ..Event::default()
            },
            Event {
                begin: 10,
                end: 40,
                kind: 5,
                index: -7,
                result: 3,
            },
            Event {
                begin: DURATION_TICKS,
                end: DURATION_TICKS + 20,
                kind: 1,
                ..Event::default()
            },
        ];
        for row in rows {
            trace.observe(row);
        }
        // Rows come back exactly, in order, once the capture has finished.
        assert_eq!(drained(trace), rows.to_vec());
        let data = trace.take_ready().unwrap();
        assert_eq!(
            data,
            Summary {
                id: 1,
                reason: Finish::Elapsed,
                skipped: 0,
                event_count: 3
            }
        );
        assert!(trace.begin(12, DURATION_TICKS));
    }
    #[test]
    fn chunks_hand_over_only_after_a_drain_and_count_the_overflow() {
        // 160 KiB of rows: keep the test trace off the test thread's stack.
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        trace.begin(0, DURATION_TICKS);
        let guard = trace.state.lock().unwrap();
        trace.observe(Event::default());
        drop(guard);
        for n in 0..2 * CHUNK + 1 {
            trace.observe(Event {
                begin: n as u64,
                end: n as u64,
                ..Event::default()
            });
        }
        // Both chunks full and nothing drained: one row lost, counted.
        let mut out = Vec::new();
        assert_eq!(trace.drain(&mut out), CHUNK);
        assert_eq!(out.len(), CHUNK);
        assert_eq!((out[0].begin, out[CHUNK - 1].begin), (0, CHUNK as u64 - 1));
        // The drained chunk is free again, so the next row hands over to it.
        trace.observe(Event {
            begin: 2 * CHUNK as u64 + 5,
            end: 2 * CHUNK as u64 + 5,
            ..Event::default()
        });
        trace.finish();
        out.clear();
        assert_eq!(trace.drain(&mut out), CHUNK + 1);
        assert_eq!(out[0].begin, CHUNK as u64);
        assert_eq!(out[CHUNK].begin, 2 * CHUNK as u64 + 5);
        let data = trace.take_ready().unwrap();
        assert_eq!(
            (data.reason, data.event_count, data.skipped),
            (Finish::Aborted, 2 * CHUNK + 1, 2)
        );
        trace.begin(2, DURATION_TICKS);
        trace.observe(Event {
            begin: 2,
            end: 1,
            ..Event::default()
        });
        assert_eq!(trace.take_ready().unwrap().reason, Finish::ClockInvalid);
        trace.begin(3, DURATION_TICKS);
        trace.finish();
        assert_eq!(trace.take_ready().unwrap().reason, Finish::Aborted);
    }
    #[test]
    fn packed_rows_keep_absolute_ticks_and_reject_impossible_spans() {
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        let id = 0x0629_a32f_64f8u64;
        assert!(trace.begin(id, DURATION_TICKS));
        let late = Event {
            begin: id + 29 * TICK_HZ,
            end: id + 29 * TICK_HZ + 123_456,
            kind: 3,
            index: i32::MIN,
            result: i32::MAX,
        };
        trace.observe(late);
        trace.finish();
        assert_eq!(drained(trace), vec![late]);
        trace.take_ready().unwrap();
        assert!(trace.begin(id, DURATION_TICKS));
        trace.observe(Event {
            begin: id + 1,
            end: id + 1 + u64::from(u32::MAX) + 1,
            ..Event::default()
        });
        assert_eq!(trace.take_ready().unwrap().reason, Finish::ClockInvalid);
    }
    #[test]
    fn long_captures_keep_exact_ticks_across_chunk_bases_and_end_by_scene() {
        use crate::timing::MATCH_DURATION_TICKS;
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        let id = 0x0629_a32f_64f8u64;
        assert!(trace.begin(id, MATCH_DURATION_TICKS));
        // Well past the 223 s a capture-relative u32 could hold.
        let minute_five = id + 300 * TICK_HZ;
        let first = Event {
            begin: minute_five,
            end: minute_five + 10,
            kind: 1,
            index: 0,
            result: 0,
        };
        // A span that began 100 s before the chunk's first row still packs.
        let earlier = Event {
            begin: minute_five - 100 * TICK_HZ,
            end: minute_five + 20,
            kind: 27,
            index: -1,
            result: 1,
        };
        let later = Event {
            begin: minute_five + 110 * TICK_HZ,
            end: minute_five + 110 * TICK_HZ + 5,
            kind: 2,
            index: 1,
            result: 0,
        };
        for row in [first, earlier, later] {
            trace.observe(row);
        }
        // Rows stay in the active chunk until it is full or the capture ends.
        assert!(trace.take_ready().is_none());
        assert!(drained(trace).is_empty());
        // A row outside the chunk's range (a long stall) starts the other
        // chunk with its own base instead of ending the capture.
        let near_end = id + MATCH_DURATION_TICKS - TICK_HZ;
        let late = Event {
            begin: near_end,
            end: near_end + 1,
            kind: 1,
            index: 0,
            result: 2,
        };
        trace.observe(late);
        trace.finish_with(Finish::SceneEnd);
        assert_eq!(drained(trace), vec![first, earlier, later, late]);
        let data = trace.take_ready().unwrap();
        assert_eq!(
            (data.reason, data.event_count, data.skipped),
            (Finish::SceneEnd, 4, 0)
        );
        assert_eq!(data.reason.label(), "scene_end");
        // The ceiling is measured from the first row, not from the id.
        assert!(trace.begin(id, MATCH_DURATION_TICKS));
        trace.observe(first);
        trace.observe(Event {
            begin: minute_five + MATCH_DURATION_TICKS - 1,
            end: minute_five + MATCH_DURATION_TICKS - 1,
            ..Event::default()
        });
        assert!(trace.take_ready().is_none());
        trace.observe(Event {
            begin: minute_five + MATCH_DURATION_TICKS,
            end: minute_five + MATCH_DURATION_TICKS,
            ..Event::default()
        });
        assert_eq!(drained(trace).len(), 3);
        assert_eq!(trace.take_ready().unwrap().reason, Finish::Elapsed);
    }
    #[test]
    fn streamed_csv_matches_the_single_shot_form() {
        let events = vec![
            Event {
                begin: 100,
                end: 150,
                kind: 1,
                index: 2,
                result: 0,
            },
            Event {
                begin: 160,
                end: 160,
                kind: 60,
                index: -1,
                result: 7,
            },
        ];
        let snapshot = Snapshot {
            id: 99,
            reason: Finish::Elapsed,
            skipped: 1,
            events: events.clone(),
        };
        let mut single = Vec::new();
        write_csv(&mut single, &snapshot, "b1", "css", "LessLag").unwrap();
        let mut streamed = Vec::new();
        write_csv_header(&mut streamed, 99, "b1", "css").unwrap();
        let mut next = 0;
        write_csv_rows(&mut streamed, &mut next, &events[..1]).unwrap();
        write_csv_rows(&mut streamed, &mut next, &events[1..]).unwrap();
        write_csv_footer(
            &mut streamed,
            "LessLag",
            &Summary {
                id: 99,
                reason: Finish::Elapsed,
                skipped: 1,
                event_count: 2,
            },
        )
        .unwrap();
        assert_eq!(single, streamed);
        let text = String::from_utf8(single).unwrap();
        assert!(text.starts_with("# schema=1\n# metric=cpu_api_spans\n# tick_hz="));
        assert!(text.contains("\nevent,kind,begin_tick,end_tick,index,result\n0,1,100,150,2,0\n1,60,160,160,-1,7\n# engine=LessLag\n# reason=elapsed\n# skipped=1\n# event_count=2\n"));
        assert!(text.ends_with("# complete=1\n"));
        assert!(write_csv_header(&mut Vec::new(), 1, "b\n1", "css").is_err());
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
        // 160 KiB of rows: keep the test trace off the test thread's stack.
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        assert!(trace.begin(7, DURATION_TICKS));
        trace.observe(Event {
            begin: 1_000,
            end: 1_000,
            kind: KIND_FLUSH_OBJECT,
            index: low,
            result: high,
        });
        assert!(trace.take_ready().is_none());
        trace.finish();
        let rows = drained(trace);
        let data = trace.take_ready().unwrap();
        assert_eq!((data.reason, rows.len()), (Finish::Aborted, 1));
        assert_eq!(pointer_from_halves(rows[0].index, rows[0].result), pointer);
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
        // 160 KiB of rows: keep the test trace off the test thread's stack.
        static TRACE: Trace = Trace::new();
        let trace = &TRACE;
        assert!(trace.begin(8, DURATION_TICKS));
        trace.observe(resize);
        assert!(trace.take_ready().is_none());
        trace.finish();
        assert_eq!(drained(trace), vec![resize]);
        assert_eq!(trace.take_ready().unwrap().event_count, 1);
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
