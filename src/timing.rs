//! Finite, opt-in render-callback capture. No inference about displayed frames.
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
pub const DURATION_SECONDS: u64 = DURATION_TICKS / TICK_HZ;
/// Ceiling for a match capture (`CAPTURE_MATCH`): it normally ends when the
/// scene leaves the stage, this bounds a match that never does.
pub const MATCH_DURATION_TICKS: u64 = 10 * 60 * TICK_HZ;
pub const MATCH_DURATION_SECONDS: u64 = MATCH_DURATION_TICKS / TICK_HZ;
/// Samples held between two drains of the diagnostics thread (every 500 ms,
/// 250 ms while a capture runs): 60 callbacks per second with headroom for a
/// stalled drain. Since the match capture the buffer is drained and streamed,
/// so it bounds nothing but a drain that stops.
pub const CAPACITY: usize = 2048;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sample {
    pub callback: u64,
    pub tick: u64,
    pub buttons: u64,
    pub fx_mode: u8,
    pub fx_scope: u8,
    pub fx_effective_off: bool,
    pub overlay_view: u8,
}

/// `Aborted` is the first variant on purpose: it is the initial `reason` of
/// the static capture state, and an all-zero initializer keeps it in `.bss`
/// instead of the NRO file. `Capacity` ends the callback capture when its
/// buffer is full; the API trace no longer produces it (it streams since
/// 0.3.5) but the label stays for decoding older CSVs. `SceneEnd` is the
/// normal end of a match capture: the scene left the stage before the ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finish {
    Aborted,
    Elapsed,
    Capacity,
    ClockInvalid,
    SceneEnd,
}

impl Finish {
    pub fn label(self) -> &'static str {
        match self {
            Self::Elapsed => "elapsed",
            Self::Capacity => "capacity",
            Self::Aborted => "aborted",
            Self::ClockInvalid => "clock_invalid",
            Self::SceneEnd => "scene_end",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Recording,
    Ready,
}

/// `samples` is empty between captures. `begin` reserves `CAPACITY` on the
/// background thread (80 KiB; since 0.3.7 the buffer is not resident while
/// idle); `drain` copies and clears it under the lock every poll, so the
/// render path never allocates or frees and the capture length is bounded by
/// its duration, not by memory.
struct State {
    phase: Phase,
    id: u64,
    duration: u64,
    first: u64,
    last_tick: u64,
    last_callback: u64,
    /// Samples accepted since `begin`, drained or not.
    total: usize,
    samples: Vec<Sample>,
    reason: Finish,
}

pub struct Capture {
    state: Mutex<State>,
    active: AtomicBool,
    epoch: AtomicU64,
    skipped: AtomicU64,
}

/// Identity and counts of a finished capture. Its samples were handed out by
/// `drain` while it ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Summary {
    pub id: u64,
    pub reason: Finish,
    pub skipped: u64,
    pub sample_count: usize,
}

/// A whole capture in memory: only tests and the single-shot `write_csv` use it.
pub struct Snapshot {
    pub id: u64,
    pub reason: Finish,
    pub samples: Vec<Sample>,
    pub skipped: u64,
}

impl Default for Capture {
    fn default() -> Self {
        Self::new()
    }
}

impl Capture {
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(State {
                phase: Phase::Idle,
                id: 0,
                duration: 0,
                first: 0,
                last_tick: 0,
                last_callback: 0,
                total: 0,
                samples: Vec::new(),
                reason: Finish::Aborted,
            }),
            active: AtomicBool::new(false),
            epoch: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
        }
    }

    pub fn active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Background thread only. Refuses to overwrite an unexported capture, and
    /// refuses to arm when the sample buffer cannot be allocated. The capture
    /// ends `duration_ticks` after its first sample.
    pub fn begin(&self, id: u64, duration_ticks: u64) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.phase != Phase::Idle {
            return false;
        }
        state.samples.clear();
        if state.samples.try_reserve_exact(CAPACITY).is_err() {
            return false;
        }
        state.phase = Phase::Recording;
        state.id = id;
        state.duration = duration_ticks;
        state.first = 0;
        state.last_tick = 0;
        state.last_callback = 0;
        state.total = 0;
        self.skipped.store(0, Ordering::Relaxed);
        self.epoch.fetch_add(1, Ordering::Release);
        drop(state);
        self.active.store(true, Ordering::Release);
        true
    }

    /// Render path: bounded work, no allocation, blocking, or I/O.
    /// Mutex protection also covers unexpected calls from multiple threads.
    pub fn observe(&self, sample: Sample) {
        let epoch = self.epoch.load(Ordering::Acquire);
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        let Ok(mut state) = self.state.try_lock() else {
            self.skipped.fetch_add(1, Ordering::Relaxed);
            return;
        };
        if state.phase != Phase::Recording || self.epoch.load(Ordering::Acquire) != epoch {
            return;
        }
        let invalid = state.total > 0
            && (sample.tick <= state.last_tick || sample.callback <= state.last_callback);
        if state.total == 0 {
            state.first = sample.tick;
        }
        state.last_tick = sample.tick;
        state.last_callback = sample.callback;
        // `begin` reserved CAPACITY, so this push never reallocates. A buffer
        // that is somehow full ends the capture instead of growing it here.
        if state.samples.len() < state.samples.capacity() {
            state.samples.push(sample);
            state.total += 1;
        }
        let reason = if invalid {
            Some(Finish::ClockInvalid)
        } else if sample.tick.saturating_sub(state.first) >= state.duration {
            Some(Finish::Elapsed)
        } else if state.samples.len() >= CAPACITY {
            Some(Finish::Capacity)
        } else {
            None
        };
        if let Some(reason) = reason {
            state.reason = reason;
            state.phase = Phase::Ready;
            self.active.store(false, Ordering::Release);
        }
    }

    /// Background timeout/abort. An unfinished trace stays explicitly incomplete.
    pub fn finish(&self) {
        self.finish_with(Finish::Aborted);
    }

    /// Background end with a stated reason (scene end, abort).
    pub fn finish_with(&self, reason: Finish) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state.phase == Phase::Recording {
            state.reason = reason;
            state.phase = Phase::Ready;
            self.active.store(false, Ordering::Release);
        }
    }

    /// Diagnostics thread only. Moves the samples recorded since the previous
    /// drain to `out`, in order. The lock is held for the copy only; a callback
    /// that collides with it counts as skipped instead of waiting.
    pub fn drain(&self, out: &mut Vec<Sample>) -> usize {
        let Ok(mut state) = self.state.lock() else {
            return 0;
        };
        if state.phase == Phase::Idle {
            return 0;
        }
        let moved = state.samples.len();
        out.extend_from_slice(&state.samples);
        state.samples.clear();
        moved
    }

    /// Background thread only. Call `drain` first: samples still in the
    /// buffer are discarded here, and the buffer is released.
    pub fn take_ready(&self) -> Option<Summary> {
        // Do not compete with recording just to ask whether it has finished.
        if self.active.load(Ordering::Acquire) {
            return None;
        }
        let mut state = self.state.lock().ok()?;
        if state.phase != Phase::Ready {
            return None;
        }
        // The idle state keeps an empty, unreserved vector.
        state.samples = Vec::new();
        let summary = Summary {
            id: state.id,
            reason: state.reason,
            skipped: self.skipped.load(Ordering::Relaxed),
            sample_count: state.total,
        };
        state.phase = Phase::Idle;
        Some(summary)
    }
}

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

fn single_line(values: &[&str]) -> io::Result<()> {
    if values.iter().any(|v| v.contains(['\n', '\r'])) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "multiline timing metadata",
        ));
    }
    Ok(())
}

/// The CSV is streamed like the API trace: header at arm time, rows as the
/// buffer drains, footer when the capture finishes. `tools/analyze_timing.py`
/// reads `# key=value` lines anywhere and requires `# complete=1` last, so
/// engine, reason, skipped and sample_count move to the footer.
pub fn write_csv_header(
    mut out: impl Write,
    id: u64,
    duration_ticks: u64,
    build: &str,
    label: &str,
) -> io::Result<()> {
    single_line(&[build, label])?;
    writeln!(
        out,
        "# schema=1\n# metric=render_callback\n# tick_hz={TICK_HZ}\n# duration_ticks={duration_ticks}\n# id={id}\n# build={build}\n# label={label}"
    )?;
    writeln!(
        out,
        "sample,callback,tick,buttons,fx_mode,fx_scope,fx_effective_off,overlay_view"
    )
}

/// `next` is the running sample number across drains.
pub fn write_csv_rows(mut out: impl Write, next: &mut usize, samples: &[Sample]) -> io::Result<()> {
    for sample in samples {
        writeln!(
            out,
            "{},{},{},{},{},{},{},{}",
            *next,
            sample.callback,
            sample.tick,
            sample.buttons,
            sample.fx_mode,
            sample.fx_scope,
            u8::from(sample.fx_effective_off),
            sample.overlay_view
        )?;
        *next += 1;
    }
    Ok(())
}

pub fn write_csv_footer(mut out: impl Write, engine: &str, summary: &Summary) -> io::Result<()> {
    single_line(&[engine])?;
    writeln!(
        out,
        "# engine={engine}\n# reason={}\n# skipped={}\n# sample_count={}\n# complete=1",
        summary.reason.label(),
        summary.skipped,
        summary.sample_count
    )
}

/// Whole capture in one call (tests and tools); same bytes as the streamed form.
pub fn write_csv(
    mut out: impl Write,
    data: &Snapshot,
    duration_ticks: u64,
    build: &str,
    label: &str,
    engine: &str,
) -> io::Result<()> {
    write_csv_header(&mut out, data.id, duration_ticks, build, label)?;
    let mut next = 0;
    write_csv_rows(&mut out, &mut next, &data.samples)?;
    write_csv_footer(
        &mut out,
        engine,
        &Summary {
            id: data.id,
            reason: data.reason,
            skipped: data.skipped,
            sample_count: data.samples.len(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn s(callback: u64, tick: u64) -> Sample {
        Sample {
            callback,
            tick,
            ..Sample::default()
        }
    }
    fn drained(capture: &Capture) -> Vec<Sample> {
        let mut out = Vec::new();
        capture.drain(&mut out);
        out
    }

    #[test]
    fn finite_capture_is_armed_on_first_sample_and_not_overwritten() {
        let capture = Capture::new();
        assert!(capture.begin(7, DURATION_TICKS));
        assert!(!capture.begin(8, DURATION_TICKS));
        capture.observe(s(100, 500));
        capture.observe(s(101, 500 + DURATION_TICKS - 1));
        assert!(capture.take_ready().is_none());
        capture.observe(s(102, 500 + DURATION_TICKS));
        assert!(!capture.begin(9, DURATION_TICKS));
        capture.observe(s(103, 999_999_999));
        assert_eq!(drained(&capture).len(), 3);
        let summary = capture.take_ready().unwrap();
        assert_eq!(
            summary,
            Summary {
                id: 7,
                reason: Finish::Elapsed,
                skipped: 0,
                sample_count: 3
            }
        );
        assert!(capture.begin(9, DURATION_TICKS));
    }

    #[test]
    fn drains_stream_in_order_and_a_long_capture_ends_at_its_own_duration() {
        let capture = Capture::new();
        assert!(capture.begin(1, MATCH_DURATION_TICKS));
        for n in 0..3 * CAPACITY as u64 {
            capture.observe(s(n, 1_000 + n * TICK_HZ / 60));
            // A drain every few samples keeps the buffer far from CAPACITY.
            if n % 500 == 499 {
                let rows = drained(&capture);
                assert_eq!(rows.len(), 500);
                assert_eq!(rows[0].callback, n - 499);
            }
        }
        assert!(capture.take_ready().is_none());
        let late = 1_000 + 3 * CAPACITY as u64 * TICK_HZ / 60;
        assert!(late < MATCH_DURATION_TICKS);
        capture.observe(s(u64::MAX / 2, 1_000 + MATCH_DURATION_TICKS));
        let rest = drained(&capture);
        assert_eq!(rest.len(), 3 * CAPACITY - 6000 + 1);
        let summary = capture.take_ready().unwrap();
        assert_eq!(summary.reason, Finish::Elapsed);
        assert_eq!(summary.sample_count, 3 * CAPACITY + 1);
        assert_eq!(MATCH_DURATION_SECONDS, 600);
    }

    #[test]
    fn bounds_and_bad_clocks_produce_explicit_incomplete_results() {
        assert_eq!(DURATION_SECONDS, 30);
        assert!(CAPACITY as u64 > DURATION_SECONDS * 60);
        let capture = Capture::new();
        capture.begin(1, DURATION_TICKS);
        for n in 0..(2 * CAPACITY as u64) {
            capture.observe(s(n, n));
        }
        assert_eq!(drained(&capture).len(), CAPACITY);
        let summary = capture.take_ready().unwrap();
        assert_eq!(
            (summary.reason, summary.sample_count),
            (Finish::Capacity, CAPACITY)
        );
        capture.begin(2, DURATION_TICKS);
        capture.observe(s(1, 10));
        capture.observe(s(2, 10));
        assert_eq!(capture.take_ready().unwrap().reason, Finish::ClockInvalid);
        capture.begin(3, DURATION_TICKS);
        capture.finish();
        assert_eq!(capture.take_ready().unwrap().reason, Finish::Aborted);
        capture.begin(4, DURATION_TICKS);
        capture.observe(s(1, 10));
        capture.finish_with(Finish::SceneEnd);
        assert!(!capture.active());
        let summary = capture.take_ready().unwrap();
        assert_eq!(
            (summary.reason, summary.sample_count, summary.reason.label()),
            (Finish::SceneEnd, 1, "scene_end")
        );
        // Undrained samples are dropped with the buffer, not kept for the next capture.
        assert!(capture.begin(5, DURATION_TICKS));
        assert!(drained(&capture).is_empty());
    }

    #[test]
    fn contention_skips_instead_of_blocking_and_is_reported() {
        let capture = Capture::new();
        capture.begin(1, DURATION_TICKS);
        let guard = capture.state.lock().unwrap();
        capture.observe(s(1, 1));
        drop(guard);
        capture.observe(s(2, 2));
        capture.finish();
        assert_eq!(capture.take_ready().unwrap().skipped, 1);
    }

    #[test]
    fn csv_preserves_raw_order_and_has_a_completion_marker() {
        let snapshot = Snapshot {
            id: 4,
            reason: Finish::Elapsed,
            skipped: 0,
            samples: vec![s(2, 100), s(3, 100 + DURATION_TICKS)],
        };
        let mut bytes = Vec::new();
        write_csv(
            &mut bytes,
            &snapshot,
            DURATION_TICKS,
            "build",
            "test",
            "Vanilla",
        )
        .unwrap();
        let csv = String::from_utf8(bytes.clone()).unwrap();
        assert!(csv.starts_with("# schema=1\n# metric=render_callback\n# tick_hz=19200000\n# duration_ticks=576000000\n# id=4\n"));
        assert!(csv.contains("0,2,100,0,0,0,0,0\n1,3,576000100,0,0,0,0,0\n"));
        assert!(csv.contains("# engine=Vanilla\n# reason=elapsed\n# skipped=0\n# sample_count=2\n"));
        assert!(csv.ends_with("# complete=1\n"));
        assert!(write_csv(
            Vec::new(),
            &snapshot,
            DURATION_TICKS,
            "bad\nbuild",
            "test",
            "Vanilla"
        )
        .is_err());
        // The streamed form writes the same bytes.
        let mut streamed = Vec::new();
        write_csv_header(&mut streamed, 4, DURATION_TICKS, "build", "test").unwrap();
        let mut next = 0;
        write_csv_rows(&mut streamed, &mut next, &snapshot.samples[..1]).unwrap();
        write_csv_rows(&mut streamed, &mut next, &snapshot.samples[1..]).unwrap();
        write_csv_footer(
            &mut streamed,
            "Vanilla",
            &Summary {
                id: 4,
                reason: Finish::Elapsed,
                skipped: 0,
                sample_count: 2,
            },
        )
        .unwrap();
        assert_eq!(streamed, bytes);
        assert!(write_csv_footer(
            Vec::new(),
            "Less\nLag",
            &Summary {
                id: 4,
                reason: Finish::Elapsed,
                skipped: 0,
                sample_count: 2
            }
        )
        .is_err());
    }

    #[test]
    fn request_labels_reject_invalid_encodings_and_oversized_input() {
        assert_eq!(request_label(b"  on_warm-1\n"), Ok("on_warm-1"));
        assert_eq!(request_label(b""), Ok("manual"));
        for bytes in [
            b"bad/name".as_slice(),
            b"\xff\xfea\0",
            &[b'a'; 65],
            b"first\nsecond",
        ] {
            assert!(request_label(bytes).is_err());
        }
    }
}
