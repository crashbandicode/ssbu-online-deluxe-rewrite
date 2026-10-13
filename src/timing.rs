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
/// 60 render callbacks per second for the whole duration, with headroom.
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
/// the static capture buffers, and an all-zero initializer keeps the 2 MiB API
/// trace in `.bss` instead of the NRO file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finish {
    Aborted,
    Elapsed,
    Capacity,
    ClockInvalid,
}

impl Finish {
    pub fn label(self) -> &'static str {
        match self {
            Self::Elapsed => "elapsed",
            Self::Capacity => "capacity",
            Self::Aborted => "aborted",
            Self::ClockInvalid => "clock_invalid",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Recording,
    Ready,
}

struct State {
    phase: Phase,
    id: u64,
    samples: [Sample; CAPACITY],
    len: usize,
    reason: Finish,
}

pub struct Capture {
    state: Mutex<State>,
    active: AtomicBool,
    epoch: AtomicU64,
    skipped: AtomicU64,
}

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
        const EMPTY: Sample = Sample {
            callback: 0,
            tick: 0,
            buttons: 0,
            fx_mode: 0,
            fx_scope: 0,
            fx_effective_off: false,
            overlay_view: 0,
        };
        Self {
            state: Mutex::new(State {
                phase: Phase::Idle,
                id: 0,
                samples: [EMPTY; CAPACITY],
                len: 0,
                reason: Finish::Aborted,
            }),
            active: AtomicBool::new(false),
            epoch: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
        }
    }

    /// Background thread only. Refuses to overwrite an unexported capture.
    pub fn begin(&self, id: u64) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if state.phase != Phase::Idle {
            return false;
        }
        state.phase = Phase::Recording;
        state.id = id;
        state.len = 0;
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
        let n = state.len;
        let invalid = n > 0
            && (sample.tick <= state.samples[n - 1].tick
                || sample.callback <= state.samples[n - 1].callback);
        state.samples[n] = sample;
        state.len += 1;
        let reason = if invalid {
            Some(Finish::ClockInvalid)
        } else if sample.tick.saturating_sub(state.samples[0].tick) >= DURATION_TICKS {
            Some(Finish::Elapsed)
        } else if state.len == CAPACITY {
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
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state.phase == Phase::Recording {
            state.reason = Finish::Aborted;
            state.phase = Phase::Ready;
            self.active.store(false, Ordering::Release);
        }
    }

    /// Background thread only. Allocates after recording has stopped.
    pub fn take_ready(&self) -> Option<Snapshot> {
        // Do not compete with recording just to ask whether it has finished.
        if self.active.load(Ordering::Acquire) {
            return None;
        }
        let mut state = self.state.lock().ok()?;
        if state.phase != Phase::Ready {
            return None;
        }
        let snapshot = Snapshot {
            id: state.id,
            reason: state.reason,
            samples: state.samples[..state.len].to_vec(),
            skipped: self.skipped.load(Ordering::Relaxed),
        };
        state.phase = Phase::Idle;
        Some(snapshot)
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

pub fn write_csv(
    mut out: impl Write,
    data: &Snapshot,
    build: &str,
    label: &str,
    engine: &str,
) -> io::Result<()> {
    if [build, label, engine]
        .iter()
        .any(|v| v.contains(['\n', '\r']))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "multiline timing metadata",
        ));
    }
    writeln!(out, "# schema=1\n# metric=render_callback\n# engine={engine}\n# tick_hz={TICK_HZ}\n# duration_ticks={DURATION_TICKS}")?;
    writeln!(
        out,
        "# id={}\n# build={build}\n# label={label}\n# reason={}\n# skipped={}\n# sample_count={}",
        data.id,
        data.reason.label(),
        data.skipped,
        data.samples.len()
    )?;
    writeln!(
        out,
        "sample,callback,tick,buttons,fx_mode,fx_scope,fx_effective_off,overlay_view"
    )?;
    for (n, sample) in data.samples.iter().enumerate() {
        writeln!(
            out,
            "{n},{},{},{},{},{},{},{}",
            sample.callback,
            sample.tick,
            sample.buttons,
            sample.fx_mode,
            sample.fx_scope,
            u8::from(sample.fx_effective_off),
            sample.overlay_view
        )?;
    }
    writeln!(out, "# complete=1")
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

    #[test]
    fn finite_capture_is_armed_on_first_sample_and_not_overwritten() {
        let capture = Capture::new();
        assert!(capture.begin(7));
        assert!(!capture.begin(8));
        capture.observe(s(100, 500));
        capture.observe(s(101, 500 + DURATION_TICKS - 1));
        assert!(capture.take_ready().is_none());
        capture.observe(s(102, 500 + DURATION_TICKS));
        assert!(!capture.begin(9));
        capture.observe(s(103, 999_999_999));
        let snapshot = capture.take_ready().unwrap();
        assert_eq!(
            (snapshot.id, snapshot.reason, snapshot.samples.len()),
            (7, Finish::Elapsed, 3)
        );
        assert!(capture.begin(9));
    }

    #[test]
    fn bounds_and_bad_clocks_produce_explicit_incomplete_results() {
        assert_eq!(DURATION_SECONDS, 30);
        assert!(CAPACITY as u64 > DURATION_SECONDS * 60);
        let capture = Capture::new();
        capture.begin(1);
        for n in 0..(2 * CAPACITY as u64) {
            capture.observe(s(n, n));
        }
        let snapshot = capture.take_ready().unwrap();
        assert_eq!(
            (snapshot.reason, snapshot.samples.len()),
            (Finish::Capacity, CAPACITY)
        );
        capture.begin(2);
        capture.observe(s(1, 10));
        capture.observe(s(2, 10));
        assert_eq!(capture.take_ready().unwrap().reason, Finish::ClockInvalid);
        capture.begin(3);
        capture.finish();
        assert_eq!(capture.take_ready().unwrap().reason, Finish::Aborted);
    }

    #[test]
    fn contention_skips_instead_of_blocking_and_is_reported() {
        let capture = Capture::new();
        capture.begin(1);
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
        write_csv(&mut bytes, &snapshot, "build", "test", "Vanilla").unwrap();
        let csv = String::from_utf8(bytes).unwrap();
        assert!(csv.contains("# metric=render_callback\n"));
        assert!(csv.contains("0,2,100,0,0,0,0,0\n1,3,576000100,0,0,0,0,0\n"));
        assert!(csv.ends_with("# complete=1\n"));
        assert!(write_csv(Vec::new(), &snapshot, "bad\nbuild", "test", "Vanilla").is_err());
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
