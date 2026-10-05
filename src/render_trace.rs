//! Bounded CPU-side API spans. These do not measure GPU execution or scanout.
use crate::timing::{Finish, DURATION_TICKS, TICK_HZ};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
};
pub const CAPACITY: usize = 4096;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Event {
    pub begin: u64,
    pub end: u64,
    /// 1=present, 2=acquire, 3=texture CPU wait, 4=render poll, 5=dispatch flush.
    pub kind: u8,
    /// Texture index for present/acquire; -1 for events without an index.
    pub index: i32,
    /// Native wait result; -1 when no trustworthy result exists.
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
                events: [Event {
                    begin: 0,
                    end: 0,
                    kind: 0,
                    index: -1,
                    result: -1,
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
        let trace = Trace::new();
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
        let trace = Trace::new();
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
}
