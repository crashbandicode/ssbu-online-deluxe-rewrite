//! Finite, opt-in capture pair (0.5.3): render-callback samples and CPU-side
//! API spans, each in a ring that never blocks a hook and that the
//! diagnostics thread swap-drains into two streamed CSVs. Pure and
//! host-tested; the device side is `runtime::capture`, `runtime::api_trace`
//! and `runtime::css_capture`. Nothing here measures GPU execution, scanout
//! or displayed frames.
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
};

pub const TICK_HZ: u64 = 19_200_000;
/// 30 s: ten seconds idle, then twenty of movement, for the CSS capture.
pub const DURATION_TICKS: u64 = 30 * TICK_HZ;
/// Ceiling for a match capture (`CAPTURE_MATCH`): it normally ends when the
/// scene leaves the stage; this bounds a match that never does.
pub const MATCH_DURATION_TICKS: u64 = 10 * 60 * TICK_HZ;
/// Wall-clock abort margin after the guest-clock duration.
pub const ABORT_MARGIN_SECONDS: u64 = 10;
/// Rows held between two drains (every 250 ms while a capture runs): 60
/// callbacks per second with headroom for a stalled drain.
pub const CALLBACK_ROWS: usize = 256;
/// About 2.5 s at the 1,660 rows/s of a match capture's stage-load burst.
pub const API_ROWS: usize = 4096;
pub const EXPORT_BUFFER_BYTES: usize = 64 * 1024;

/// Phase markers, point rows: `index` is the phase value, `result` the hashed
/// thread label. Kind 12 is the selected image index before the game's
/// modulo, 17 the tracked-window acquisitions still outstanding after a
/// present returned (zero is meaningful).
pub const KIND_INDEX: u8 = 12;
pub const KIND_PRESENT_ENTRY: u8 = 13;
pub const KIND_LATE_FLUSH: u8 = 14;
pub const KIND_RECORD_FIGHTERS: u8 = 15;
pub const KIND_POST_SCENE: u8 = 16;
pub const KIND_OUTSTANDING: u8 = 17;
/// `nvnSyncWait` with an infinite timeout on a sync that is not the texture
/// sync: `index` -1, `result` the wait result. Not kind 4 (the timeout-0 poll).
pub const KIND_SYNC_INFINITE_OTHER: u8 = 27;
/// Dependency wait `main+0x354ccd0`, any thread: span, `index` the awaited
/// counter's offset from the game text base (-1 outside the image), `result`
/// the hashed thread label. MainThread's wait on `0x5333160` is the UI
/// barrier; the same label as kind 77 is a nested wait inside the UI job.
pub const KIND_DEPENDENCY_WAIT: u8 = 66;
/// The UI job `main+0x375e260` from entry to return on its worker: span,
/// `index` -1, `result` the hashed thread label. Wall time, not CPU time.
pub const KIND_UI_JOB: u8 = 77;
/// Opt-in input trace: kinds 80–83 use `index=-1`, `result=thread label`.
pub const KIND_HID_BEFORE: u8 = 80;
pub const KIND_HID_AFTER: u8 = 81;
pub const KIND_SCENE_BEFORE: u8 = 82;
pub const KIND_SCENE_AFTER: u8 = 83;
/// Input cache snapshot at scene-before: `index` signed mode, `result` bit 0–7
/// for nonzero override bytes at owner+0x580..0x587. No row if unreadable.
pub const KIND_INPUT_CACHE_STATE: u8 = 84;

/// Hashed thread label shared by kinds 12–17, 66, 77 and 80–83. Pure, so
/// `tools/analyze_render_trace.py` recomputes it with the same formula.
pub const fn thread_label(thread: u64) -> i32 {
    let mixed = thread
        .wrapping_add(0x9E37_79B1_85EB_CA87)
        .wrapping_mul(0x9E37_79B1_85EB_CA87)
        .rotate_left(27);
    (mixed as u32 & 0x7fff_ffff) as i32
}

/// Offset from the game text base, or -1 when it is not inside the first
/// 2 GiB of the image.
pub const fn image_offset(address: u64, text_base: u64) -> i32 {
    let offset = address.wrapping_sub(text_base);
    if offset < 0x8000_0000 {
        offset as i32
    } else {
        -1
    }
}

/// The `CAPTURE_TIMING` file content as a `# label=`; empty means `manual`.
pub fn request_label(bytes: &[u8]) -> Result<&str, &'static str> {
    if bytes.len() > 64 {
        return Err("timing request exceeds 64 bytes");
    }
    let label = std::str::from_utf8(bytes)
        .map_err(|_| "timing label must be ASCII")?
        .trim();
    if label.len() > 48
        || !label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err("invalid timing label");
    }
    Ok(if label.is_empty() { "manual" } else { label })
}

/// `Aborted` is first on purpose: it is the zero initializer of the static
/// rings, which keeps them in `.bss`. `SceneEnd` is the normal end of a match
/// capture. A full ring counts rows as skipped and never ends a capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finish {
    Aborted,
    Elapsed,
    ClockInvalid,
    SceneEnd,
}

impl Finish {
    pub fn label(self) -> &'static str {
        match self {
            Self::Aborted => "aborted",
            Self::Elapsed => "elapsed",
            Self::ClockInvalid => "clock_invalid",
            Self::SceneEnd => "scene_end",
        }
    }
}

/// One ring row. `begin`/`end` drive the elapsed check, `clock_valid` is the
/// rule whose violation ends the capture `clock_invalid`, and `csv` writes the
/// row's line in its stream.
pub trait Entry: Copy {
    fn begin(&self) -> u64;
    fn end(&self) -> u64;
    fn clock_valid(&self, previous: Option<&Self>) -> bool;
    fn csv(&self, out: &mut dyn Write, n: usize) -> io::Result<()>;
}

/// One render callback. Schema 2 (0.5.0) dropped the three FX columns of
/// schema 1; the character FX live in `ssbu-fx-control`, which Deluxe does
/// not see. 32 bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sample {
    pub callback: u64,
    pub tick: u64,
    pub buttons: u64,
    pub overlay_view: u8,
}

impl Entry for Sample {
    fn begin(&self) -> u64 {
        self.tick
    }
    fn end(&self) -> u64 {
        self.tick
    }
    /// Tick and callback counter both strictly increase.
    fn clock_valid(&self, previous: Option<&Self>) -> bool {
        match previous {
            Some(p) => self.tick > p.tick && self.callback > p.callback,
            None => true,
        }
    }
    fn csv(&self, out: &mut dyn Write, n: usize) -> io::Result<()> {
        writeln!(
            out,
            "{n},{},{},{},{}",
            self.callback, self.tick, self.buttons, self.overlay_view
        )
    }
}

/// One CPU-side span or point. Kinds: 1 present, 2 acquire, 3 texture CPU
/// wait, 4 render poll, 5 dispatch flush, 6 HID poll marker (`index` 24-bit
/// serial, `result` left Y), 7 the same poll's sampling counter, then 12–17,
/// 27, 66, 77 and 80–84 above. Kinds 80–83 store a thread label in `result`;
/// kind 84 stores its override-byte mask there. Kinds 18–26, 28–38 and 60–64
/// are retired; archived traces still decode them. 32 bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Event {
    pub begin: u64,
    pub end: u64,
    pub kind: u8,
    /// Texture index for present/acquire; -1 for events without an index.
    pub index: i32,
    /// Native wait result or the hashed thread label; -1 when none exists.
    pub result: i32,
}

impl Entry for Event {
    fn begin(&self) -> u64 {
        self.begin
    }
    fn end(&self) -> u64 {
        self.end
    }
    /// Nested and overlapping spans are fine; one that ends before it began is not.
    fn clock_valid(&self, _: Option<&Self>) -> bool {
        self.end >= self.begin
    }
    fn csv(&self, out: &mut dyn Write, n: usize) -> io::Result<()> {
        writeln!(
            out,
            "{n},{},{},{},{},{}",
            self.kind, self.begin, self.end, self.index, self.result
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Recording,
    Ready,
}

struct State<T> {
    phase: Phase,
    id: u64,
    duration: u64,
    first: u64,
    last: Option<T>,
    /// Rows accepted since `begin`, handed out or not.
    total: usize,
    /// Empty and unreserved while idle; `begin` reserves the ring's capacity.
    rows: Vec<T>,
    reason: Finish,
}

/// One capture ring: hooks `observe`, the diagnostics thread arms, drains and
/// finishes. The callback and API rings are separate statics with separate
/// locks, so callbacks never lose rows to API contention or overflow.
pub struct Ring<T> {
    capacity: usize,
    state: Mutex<State<T>>,
    active: AtomicBool,
    epoch: AtomicU64,
    skipped: AtomicU64,
}

/// Identity and counts of a finished capture. `count` equals the rows handed
/// out by `drain` and `take_ready` together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Summary {
    pub id: u64,
    pub reason: Finish,
    pub skipped: u64,
    pub count: usize,
}

impl<T: Entry> Ring<T> {
    pub const fn new(capacity: usize) -> Self {
        Self {
            capacity,
            state: Mutex::new(State {
                phase: Phase::Idle,
                id: 0,
                duration: 0,
                first: 0,
                last: None,
                total: 0,
                rows: Vec::new(),
                reason: Finish::Aborted,
            }),
            active: AtomicBool::new(false),
            epoch: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
        }
    }

    /// One atomic load; producers ask this before reading the clock.
    pub fn active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Diagnostics thread only. Refuses to arm over an unexported capture and
    /// when the buffer cannot be allocated. The capture ends `duration` ticks
    /// after its first row.
    pub fn begin(&self, id: u64, duration: u64) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.phase != Phase::Idle {
            return false;
        }
        state.rows.clear();
        if state.rows.try_reserve_exact(self.capacity).is_err() {
            return false;
        }
        state.phase = Phase::Recording;
        state.id = id;
        state.duration = duration;
        state.first = 0;
        state.last = None;
        state.total = 0;
        self.skipped.store(0, Ordering::Relaxed);
        self.epoch.fetch_add(1, Ordering::Release);
        drop(state);
        self.active.store(true, Ordering::Release);
        true
    }

    /// Hook path: bounded work, no allocation, blocking or I/O. A lock
    /// collision and a full buffer both count as skipped. Rows that began
    /// before the arm tick are dropped. The row is kept before the capture
    /// finishes on it, so an `elapsed` capture spans at least its duration.
    pub fn observe(&self, row: T) {
        let epoch = self.epoch.load(Ordering::Acquire);
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        let Ok(mut state) = self.state.try_lock() else {
            self.skipped.fetch_add(1, Ordering::Relaxed);
            return;
        };
        if state.phase != Phase::Recording
            || self.epoch.load(Ordering::Acquire) != epoch
            || row.begin() < state.id
        {
            return;
        }
        let invalid = !row.clock_valid(state.last.as_ref());
        if state.total == 0 {
            state.first = row.begin();
        }
        state.last = Some(row);
        // `begin` reserved the capacity, so this push never reallocates.
        if state.rows.len() < state.rows.capacity() {
            state.rows.push(row);
            state.total += 1;
        } else {
            self.skipped.fetch_add(1, Ordering::Relaxed);
        }
        let reason = if invalid {
            Some(Finish::ClockInvalid)
        } else if row.end().saturating_sub(state.first) >= state.duration {
            Some(Finish::Elapsed)
        } else {
            None
        };
        if let Some(reason) = reason {
            state.reason = reason;
            state.phase = Phase::Ready;
            self.active.store(false, Ordering::Release);
        }
    }

    /// Diagnostics thread: abort, scene end or preemption. Only a recording
    /// capture changes.
    pub fn finish(&self, reason: Finish) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state.phase == Phase::Recording {
            state.reason = reason;
            state.phase = Phase::Ready;
            self.active.store(false, Ordering::Release);
        }
    }

    /// Diagnostics thread only. Moves the rows recorded since the previous
    /// drain into `out`, in order, by swapping buffers: O(1) under the lock,
    /// so a colliding hook waits for no copy. `out` should hold the ring's
    /// capacity (the service reserves it at arm); a smaller buffer is copied
    /// into instead, so the ring never shrinks.
    pub fn drain(&self, out: &mut Vec<T>) {
        out.clear();
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state.phase != Phase::Idle {
            self.hand_over(&mut state, out);
        }
    }

    /// Once the capture has finished: its last rows, under the same lock that
    /// counted them (a hook that finished the capture between a `drain` and
    /// this call loses nothing), then the summary; the storage is released.
    pub fn take_ready(&self, out: &mut Vec<T>) -> Option<Summary> {
        if self.active.load(Ordering::Acquire) {
            return None;
        }
        let mut state = self.state.lock().ok()?;
        if state.phase != Phase::Ready {
            return None;
        }
        out.clear();
        self.hand_over(&mut state, out);
        // The idle state keeps an empty, unreserved vector.
        state.rows = Vec::new();
        state.phase = Phase::Idle;
        Some(Summary {
            id: state.id,
            reason: state.reason,
            skipped: self.skipped.load(Ordering::Relaxed),
            count: state.total,
        })
    }

    fn hand_over(&self, state: &mut State<T>, out: &mut Vec<T>) {
        if out.capacity() >= self.capacity {
            std::mem::swap(&mut state.rows, out);
        } else {
            out.extend_from_slice(&state.rows);
            state.rows.clear();
        }
    }
}

/// Which CSV of the pair: `timing-<id>.csv` (schema 2) or `api-<id>.csv`
/// (schema 1). Both readers take `# key=value` lines anywhere and require
/// `# complete=1` last, so engine, reason and the counts live in the footer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Timing,
    Api,
}

fn single_line(values: &[&str]) -> io::Result<()> {
    if values.iter().any(|v| v.contains(['\n', '\r'])) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "multiline capture metadata",
        ));
    }
    Ok(())
}

/// Written at arm time. `clock` is `clock_probe::header_lines`: three
/// `# key=value` lines that follow the column line, as since 0.5.3.
pub fn header(
    mut out: impl Write,
    stream: Stream,
    id: u64,
    duration: u64,
    build: &str,
    label: &str,
    clock: &str,
) -> io::Result<()> {
    single_line(&[build, label])?;
    match stream {
        Stream::Timing => writeln!(
            out,
            "# schema=2\n# metric=render_callback\n# tick_hz={TICK_HZ}\n# duration_ticks={duration}\n# id={id}\n# build={build}\n# label={label}\nsample,callback,tick,buttons,overlay_view"
        )?,
        Stream::Api => writeln!(
            out,
            "# schema=1\n# metric=cpu_api_spans\n# tick_hz={TICK_HZ}\n# id={id}\n# build={build}\n# label={label}\nevent,kind,begin_tick,end_tick,index,result"
        )?,
    }
    writeln!(out, "{clock}")
}

/// `next` is the running row number across drains.
pub fn rows<T: Entry>(mut out: impl Write, next: &mut usize, rows: &[T]) -> io::Result<()> {
    for row in rows {
        row.csv(&mut out, *next)?;
        *next += 1;
    }
    Ok(())
}

/// Written when the capture has finished; `# complete=1` is the last line.
pub fn footer(
    mut out: impl Write,
    stream: Stream,
    engine: &str,
    summary: &Summary,
) -> io::Result<()> {
    single_line(&[engine])?;
    let count = match stream {
        Stream::Timing => "sample_count",
        Stream::Api => "event_count",
    };
    writeln!(
        out,
        "# engine={engine}\n# reason={}\n# skipped={}\n# {count}={}\n# complete=1",
        summary.reason.label(),
        summary.skipped,
        summary.count
    )
}

pub const PHASE_IDLE: u8 = 0;
pub const PHASE_CAPTURING: u8 = 1;
pub const PHASE_SAVED: u8 = 2;
pub const PHASE_EXPORT_ERROR: u8 = 3;
pub const PHASE_EXPORTING: u8 = 4;

/// The Debug view's `Timing` row.
pub fn status_label(phase: u8) -> &'static str {
    match phase {
        PHASE_CAPTURING => "Capturing",
        PHASE_SAVED => "CSV saved",
        PHASE_EXPORT_ERROR => "Export error",
        PHASE_EXPORTING => "Exporting CSV",
        _ => "Idle",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOCK: &str = "# clock_mhz=1020\n# clock=stock\n# hoc=none";

    fn s(callback: u64, tick: u64) -> Sample {
        Sample {
            callback,
            tick,
            ..Sample::default()
        }
    }

    fn e(begin: u64, end: u64, kind: u8) -> Event {
        Event {
            begin,
            end,
            kind,
            index: -1,
            result: 0,
        }
    }

    fn buffer<T>(capacity: usize) -> Vec<T> {
        Vec::with_capacity(capacity)
    }

    fn drained<T: Entry>(ring: &Ring<T>) -> Vec<T> {
        let mut out = buffer(ring.capacity);
        ring.drain(&mut out);
        out
    }

    fn ready<T: Entry>(ring: &Ring<T>) -> Option<(Summary, Vec<T>)> {
        let mut out = buffer(ring.capacity);
        ring.take_ready(&mut out).map(|summary| (summary, out))
    }

    #[test]
    fn arming_is_refused_while_busy_and_the_ring_finishes_on_elapsed() {
        let ring: Ring<Sample> = Ring::new(CALLBACK_ROWS);
        assert!(!ring.active());
        assert!(ring.begin(7, DURATION_TICKS));
        assert!(ring.active());
        assert!(!ring.begin(8, DURATION_TICKS));
        // A row that began before the arm tick is dropped, not counted.
        ring.observe(s(99, 6));
        ring.observe(s(100, 500));
        ring.observe(s(101, 500 + DURATION_TICKS - 1));
        assert!(ready(&ring).is_none());
        // The crossing row is kept, so the file spans the whole duration.
        ring.observe(s(102, 500 + DURATION_TICKS));
        assert!(!ring.active());
        assert!(!ring.begin(9, DURATION_TICKS));
        ring.observe(s(103, 999_999_999));
        assert_eq!(drained(&ring).len(), 3);
        let (summary, rest) = ready(&ring).unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            summary,
            Summary {
                id: 7,
                reason: Finish::Elapsed,
                skipped: 0,
                count: 3
            }
        );
        assert!(ring.begin(9, DURATION_TICKS));
        assert!(drained(&ring).is_empty());
    }

    #[test]
    fn callback_clock_is_strict_and_api_spans_may_nest() {
        let ring: Ring<Sample> = Ring::new(CALLBACK_ROWS);
        ring.begin(1, DURATION_TICKS);
        ring.observe(s(1, 10));
        ring.observe(s(2, 10));
        assert_eq!(ready(&ring).unwrap().0.reason, Finish::ClockInvalid);
        ring.begin(2, DURATION_TICKS);
        ring.observe(s(5, 10));
        ring.observe(s(5, 11));
        let (summary, rows) = ready(&ring).unwrap();
        assert_eq!(
            (summary.reason, summary.count, rows.len()),
            (Finish::ClockInvalid, 2, 2)
        );
        ring.begin(3, DURATION_TICKS);
        ring.finish(Finish::Aborted);
        assert_eq!(ready(&ring).unwrap().0.reason.label(), "aborted");
        ring.begin(4, DURATION_TICKS);
        ring.observe(s(1, 10));
        ring.finish(Finish::SceneEnd);
        assert!(!ring.active());
        let (summary, rows) = ready(&ring).unwrap();
        assert_eq!(
            (summary.reason.label(), summary.count, rows.len()),
            ("scene_end", 1, 1)
        );

        let api: Ring<Event> = Ring::new(API_ROWS);
        assert!(api.begin(1, DURATION_TICKS));
        let rows = [
            e(20, 30, 1),
            Event {
                begin: 10,
                end: 40,
                kind: 5,
                index: -7,
                result: 3,
            },
            e(DURATION_TICKS, DURATION_TICKS + 20, 1),
        ];
        for row in rows {
            api.observe(row);
        }
        assert_eq!(drained(&api), rows.to_vec());
        assert_eq!(ready(&api).unwrap().0.reason, Finish::Elapsed);
        api.begin(2, DURATION_TICKS);
        api.observe(e(2, 1, 1));
        assert_eq!(ready(&api).unwrap().0.reason, Finish::ClockInvalid);
    }

    #[test]
    fn contention_counts_as_skipped_instead_of_blocking() {
        let ring: Ring<Event> = Ring::new(API_ROWS);
        ring.begin(1, DURATION_TICKS);
        let guard = ring.state.lock().unwrap();
        ring.observe(e(1, 1, 1));
        drop(guard);
        ring.observe(e(2, 2, 1));
        ring.finish(Finish::Aborted);
        let (summary, rows) = ready(&ring).unwrap();
        assert_eq!((summary.skipped, summary.count, rows.len()), (1, 1, 1));
    }

    #[test]
    fn a_full_ring_skips_without_finishing_or_growing() {
        let ring: Ring<Event> = Ring::new(4);
        ring.begin(1, DURATION_TICKS);
        for n in 1..=6 {
            ring.observe(e(n, n, 1));
        }
        assert!(ring.active());
        assert_eq!(ring.state.lock().unwrap().rows.capacity(), 4);
        let rows = drained(&ring);
        assert_eq!(rows.len(), 4);
        assert_eq!((rows[0].begin, rows[3].begin), (1, 4));
        // The drained ring accepts rows again.
        ring.observe(e(7, 7, 1));
        ring.finish(Finish::Aborted);
        let (summary, rest) = ready(&ring).unwrap();
        assert_eq!(rest, vec![e(7, 7, 1)]);
        assert_eq!((summary.skipped, summary.count), (2, 5));
    }

    #[test]
    fn swap_drains_keep_order_across_a_long_capture() {
        let ring: Ring<Sample> = Ring::new(CALLBACK_ROWS);
        assert!(ring.begin(1, MATCH_DURATION_TICKS));
        let mut out = buffer(CALLBACK_ROWS);
        let mut seen = 0;
        for n in 0..3 * CALLBACK_ROWS as u64 {
            ring.observe(s(n + 1, 1_000 + n * TICK_HZ / 60));
            if n % 100 == 99 {
                ring.drain(&mut out);
                assert_eq!(out.len(), 100);
                assert_eq!(out[0].callback, n - 98);
                // Both buffers keep the ring's capacity, so no hook ever skips.
                assert!(out.capacity() >= CALLBACK_ROWS);
                seen += out.len();
            }
        }
        assert!(ready(&ring).is_none());
        let late = 1_000 + 3 * CALLBACK_ROWS as u64 * TICK_HZ / 60;
        assert!(late < MATCH_DURATION_TICKS);
        ring.observe(s(u64::MAX / 2, 1_000 + MATCH_DURATION_TICKS));
        let (summary, rest) = ready(&ring).unwrap();
        assert_eq!(rest.len(), 3 * CALLBACK_ROWS - seen + 1);
        assert_eq!(rest.last().unwrap().callback, u64::MAX / 2);
        assert_eq!(summary.reason, Finish::Elapsed);
        assert_eq!(summary.count, 3 * CALLBACK_ROWS + 1);
        assert_eq!(MATCH_DURATION_TICKS / TICK_HZ, 600);
        assert_eq!(DURATION_TICKS / TICK_HZ, 30);
        // An undersized buffer is copied into, and the ring keeps its capacity.
        ring.begin(2, DURATION_TICKS);
        ring.observe(s(1, 10));
        let mut small = Vec::new();
        ring.drain(&mut small);
        assert_eq!(small, vec![s(1, 10)]);
        assert_eq!(ring.state.lock().unwrap().rows.capacity(), CALLBACK_ROWS);
        ring.finish(Finish::Aborted);
        assert_eq!(ready(&ring).unwrap().0.count, 1);
    }

    #[test]
    fn take_ready_hands_over_the_final_rows_so_the_count_matches() {
        let ring: Ring<Event> = Ring::new(API_ROWS);
        ring.begin(1, DURATION_TICKS);
        ring.observe(e(10, 20, 1));
        ring.observe(e(30, 40, 2));
        let first = drained(&ring);
        assert_eq!(first.len(), 2);
        // A hook finishes the capture after the drain and before take_ready:
        // the finishing row is handed over with the summary, never dropped.
        ring.observe(e(50, 60, 1));
        ring.observe(e(70, 70 + DURATION_TICKS, 27));
        let (summary, rest) = ready(&ring).unwrap();
        assert_eq!(rest.len(), 2);
        assert_eq!(rest[1].kind, 27);
        assert_eq!(summary.count, first.len() + rest.len());
        assert_eq!(summary.reason, Finish::Elapsed);
        // Idle afterwards: nothing to drain, nothing ready, storage released.
        assert!(drained(&ring).is_empty());
        assert!(ready(&ring).is_none());
        assert_eq!(ring.state.lock().unwrap().rows.capacity(), 0);
    }

    #[test]
    fn golden_csv_bytes_for_both_streams() {
        let samples = [
            s(2, 100),
            Sample {
                callback: 3,
                tick: 100 + DURATION_TICKS,
                buttons: u64::MAX,
                overlay_view: 5,
            },
        ];
        let mut timing = Vec::new();
        header(
            &mut timing,
            Stream::Timing,
            4,
            DURATION_TICKS,
            "build",
            "test",
            CLOCK,
        )
        .unwrap();
        let mut next = 0;
        rows(&mut timing, &mut next, &samples[..1]).unwrap();
        rows(&mut timing, &mut next, &samples[1..]).unwrap();
        footer(
            &mut timing,
            Stream::Timing,
            "Vanilla",
            &Summary {
                id: 4,
                reason: Finish::Elapsed,
                skipped: 0,
                count: 2,
            },
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(timing).unwrap(),
            "# schema=2\n# metric=render_callback\n# tick_hz=19200000\n# duration_ticks=576000000\n# id=4\n# build=build\n# label=test\nsample,callback,tick,buttons,overlay_view\n# clock_mhz=1020\n# clock=stock\n# hoc=none\n0,2,100,0,0\n1,3,576000100,18446744073709551615,5\n# engine=Vanilla\n# reason=elapsed\n# skipped=0\n# sample_count=2\n# complete=1\n"
        );

        let events = [
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
                kind: 66,
                index: -1,
                result: 7,
            },
        ];
        let mut api = Vec::new();
        header(
            &mut api,
            Stream::Api,
            99,
            DURATION_TICKS,
            "b1",
            "css",
            CLOCK,
        )
        .unwrap();
        let mut next = 0;
        rows(&mut api, &mut next, &events).unwrap();
        footer(
            &mut api,
            Stream::Api,
            "LessLag",
            &Summary {
                id: 99,
                reason: Finish::SceneEnd,
                skipped: 1,
                count: 2,
            },
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(api).unwrap(),
            "# schema=1\n# metric=cpu_api_spans\n# tick_hz=19200000\n# id=99\n# build=b1\n# label=css\nevent,kind,begin_tick,end_tick,index,result\n# clock_mhz=1020\n# clock=stock\n# hoc=none\n0,1,100,150,2,0\n1,66,160,160,-1,7\n# engine=LessLag\n# reason=scene_end\n# skipped=1\n# event_count=2\n# complete=1\n"
        );

        let summary = Summary {
            id: 1,
            reason: Finish::Aborted,
            skipped: 0,
            count: 0,
        };
        assert!(header(Vec::new(), Stream::Api, 1, 0, "b\n1", "css", CLOCK).is_err());
        assert!(header(Vec::new(), Stream::Timing, 1, 0, "b", "la\rbel", CLOCK).is_err());
        assert!(footer(Vec::new(), Stream::Timing, "Less\nLag", &summary).is_err());
        let mut aborted = Vec::new();
        footer(&mut aborted, Stream::Api, "Mixed", &summary).unwrap();
        assert_eq!(
            aborted,
            b"# engine=Mixed\n# reason=aborted\n# skipped=0\n# event_count=0\n# complete=1\n"
        );
    }

    #[test]
    fn request_labels_reject_invalid_encodings_and_oversized_input() {
        assert_eq!(request_label(b"  on_warm-1\n"), Ok("on_warm-1"));
        assert_eq!(request_label(b""), Ok("manual"));
        for bytes in [
            b"bad/name".as_slice(),
            b"\xff\xfea\0",
            &[b'a'; 65],
            &[b'a'; 49],
            b"first\nsecond",
        ] {
            assert!(request_label(bytes).is_err());
        }
    }

    #[test]
    fn thread_label_matches_the_python_formula_and_offsets_stay_in_image() {
        // Pinned in tools/test_analyze_render_trace.py (test_thread_label_matches_the_formula).
        assert_eq!(thread_label(0), 137_139_250);
        assert_eq!(thread_label(0x1000), 199_847_157);
        assert_ne!(thread_label(0x1000), thread_label(0x1010));
        assert!(thread_label(u64::MAX) >= 0);
        let text_base = 0x0800_0000_0000u64;
        assert_eq!(
            image_offset(text_base + 0x0533_3160, text_base),
            0x0533_3160
        );
        assert_eq!(image_offset(0x10, text_base), -1);
        assert_eq!(image_offset(text_base + 0x8000_0000, text_base), -1);
        let kept = [
            KIND_INDEX,
            KIND_PRESENT_ENTRY,
            KIND_LATE_FLUSH,
            KIND_RECORD_FIGHTERS,
            KIND_POST_SCENE,
            KIND_OUTSTANDING,
            KIND_SYNC_INFINITE_OTHER,
            KIND_DEPENDENCY_WAIT,
            KIND_UI_JOB,
            KIND_HID_BEFORE,
            KIND_HID_AFTER,
            KIND_SCENE_BEFORE,
            KIND_SCENE_AFTER,
            KIND_INPUT_CACHE_STATE,
        ];
        assert_eq!(
            kept,
            [12, 13, 14, 15, 16, 17, 27, 66, 77, 80, 81, 82, 83, 84]
        );
        assert_eq!(std::mem::size_of::<Sample>(), 32);
        assert_eq!(std::mem::size_of::<Event>(), 32);
    }

    #[test]
    fn status_labels_name_the_five_phases() {
        assert_eq!(status_label(PHASE_IDLE), "Idle");
        assert_eq!(status_label(PHASE_CAPTURING), "Capturing");
        assert_eq!(status_label(PHASE_SAVED), "CSV saved");
        assert_eq!(status_label(PHASE_EXPORT_ERROR), "Export error");
        assert_eq!(status_label(PHASE_EXPORTING), "Exporting CSV");
        assert_eq!(status_label(200), "Idle");
    }
}
