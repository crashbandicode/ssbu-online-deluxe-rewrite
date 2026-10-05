//! Bounded, nonblocking event capture. File I/O belongs to the background sink.
//! Contended events are counted as dropped; a game hook never waits for a logger.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

pub const CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Code {
    Boot = 1,
    UnsupportedVersion = 2,
    HookInstalling = 3,
    HookInstalled = 4,
    HookRejected = 5,
    ConfigLoaded = 6,
    ConfigRejected = 7,
    MenuChanged = 8,
    LatencyApplied = 9,
    NullLatencyPointer = 10,
    FxSuppressed = 11,
    CssEntered = 12,
    CssExited = 13,
    Panic = 14,
    Heartbeat = 15,
    Compatibility = 16,
    HookEntryWord = 17,
    PaneAccepted = 18,
    PaneRejected = 19,
    PaneField = 20,
    OverlayEntered = 21,
    OverlayExited = 22,
    OverlayView = 23,
    ConfigSaved = 24,
    OverlayHealth = 25,
    OverlayBackendReady = 26,
    AcmdEffectSeen = 27,
    AcmdEffectSuppressed = 28,
    AcmdStackRejected = 29,
    FxOnlineContext = 30,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Event {
    pub sequence: u64,
    pub tick: u64,
    pub code: u16,
    pub a: u64,
    pub b: u64,
}

struct State {
    events: [Event; CAPACITY],
    sequence: u64,
}

pub struct Recorder {
    state: Mutex<State>,
    dropped: AtomicU64,
}

#[derive(Clone)]
pub struct Snapshot {
    pub events: [Event; CAPACITY],
    pub newest: u64,
    pub dropped: u64,
}

impl Snapshot {
    /// Returns retained events oldest-first, excluding events already persisted.
    pub fn after(&self, sequence: u64) -> impl Iterator<Item = &Event> {
        let start = self.newest.saturating_sub(CAPACITY as u64).max(sequence);
        ((start + 1)..=self.newest).map(|n| &self.events[((n - 1) % CAPACITY as u64) as usize])
    }
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

impl Recorder {
    pub const fn new() -> Self {
        const EMPTY: Event = Event {
            sequence: 0,
            tick: 0,
            code: 0,
            a: 0,
            b: 0,
        };
        Self {
            state: Mutex::new(State {
                events: [EMPTY; CAPACITY],
                sequence: 0,
            }),
            dropped: AtomicU64::new(0),
        }
    }

    pub fn record(&self, tick: u64, code: Code, a: u64, b: u64) {
        let Ok(mut state) = self.state.try_lock() else {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        };
        state.sequence += 1;
        let sequence = state.sequence;
        state.events[((sequence - 1) % CAPACITY as u64) as usize] = Event {
            sequence,
            tick,
            code: code as u16,
            a,
            b,
        };
    }

    pub fn snapshot(&self) -> Option<Snapshot> {
        let state = self.state.try_lock().ok()?;
        Some(Snapshot {
            events: state.events,
            newest: state.sequence,
            dropped: self.dropped.load(Ordering::Relaxed),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrap_retains_order_and_exposes_lost_range() {
        let recorder = Recorder::new();
        for n in 1..=150 {
            recorder.record(n, Code::Heartbeat, n * 2, 0);
        }
        let snapshot = recorder.snapshot().unwrap();
        let events: Vec<_> = snapshot.after(0).collect();
        assert_eq!(events.len(), CAPACITY);
        assert_eq!(events.first().unwrap().sequence, 23);
        assert_eq!(events.last().unwrap().a, 300);
        assert!(events
            .windows(2)
            .all(|pair| pair[1].sequence == pair[0].sequence + 1));
        assert_eq!(snapshot.after(148).count(), 2);
        assert_eq!(snapshot.after(150).count(), 0);
    }
    #[test]
    fn recursive_or_contended_logging_drops_instead_of_deadlocking() {
        let recorder = Recorder::new();
        let lock = recorder.state.lock().unwrap();
        recorder.record(0, Code::Panic, 0, 0);
        assert!(recorder.snapshot().is_none());
        drop(lock);
        assert_eq!(recorder.snapshot().unwrap().dropped, 1);
    }
    #[test]
    fn concurrent_hooks_keep_records_coherent() {
        let recorder = std::sync::Arc::new(Recorder::new());
        let workers: Vec<_> = (0..4)
            .map(|i| {
                let r = recorder.clone();
                std::thread::spawn(move || {
                    for n in 0..1000 {
                        r.record(n, Code::FxSuppressed, i, !i);
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
        let snapshot = recorder.snapshot().unwrap();
        assert_eq!(snapshot.newest + snapshot.dropped, 4000);
        assert!(snapshot.after(0).all(|e| e.b == !e.a));
    }
}
