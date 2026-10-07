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
    TimingStarted = 31,
    TimingFinished = 32,
    RenderOwner = 33,
    RenderCaller = 34,
    RenderFlushTarget = 35,
    PiaRtt = 36,
    PeerObserve = 37,
    PeerObserveName = 38,
    PeerObserveDrop = 39,
    PeerObserveMap = 40,
    /// Outgoing plaintext packet header at the encrypt site.
    SendPacket = 41,
    /// Mixed source id plus the flag, kind, length, and inheritance mask.
    SendIdentity = 42,
    /// Mixed destination id and the receive-map index, when the id is known.
    SendDestination = 43,
    /// Kind 0x14 subtype-2 spare slot, or kind 0x58 type and Data baseline.
    SendDetail = 44,
    /// Send-site ring or unreadable packet drops.
    SendDrop = 45,
    ProfileTransition = 46,
    ProfileRefused = 47,
    /// Station-index hook reachability. `a` is hits since install; `b` is flags.
    StationProbe = 64,
    /// Mixed source id from that sample. `a` is `mix_id` after a successful read.
    StationProbeId = 65,
    /// Mixed constant id learned from an inverse connection request.
    TransportHandshake = 66,
    /// Mixed constant id confirmed by a matching connection response.
    TransportConfirm = 67,
    /// Session table cleared. `a` is the cleanup count.
    TransportCleanup = 68,
    /// Beacon write or skip. `a` is writes, `b` is skips.
    TransportBeacon = 69,
    /// Modded-peer payload write or skip. `a` is writes, `b` is skips.
    TransportPayload = 70,
    /// Stealth suppressed a beacon or payload write. `a` is the count.
    TransportStealth = 71,
    /// Ambiguous mapping or a busy gate. `a` is ambiguous, `b` is contention.
    TransportSkip = 72,
    /// Connection in the low byte, scene status in the next, ready in bit 16.
    /// `b` is the hook offset. This is not the startup render mode.
    AutoContext = 73,
    /// Requested mode in the low byte, selected mode in the next, then auto,
    /// platform, cleanup hold, and scene-ready bits. `b` is the selection generation.
    AutoRequest = 74,
    /// common.nro load observation. `a` is the module base, `b` is the symbol.
    FighterModule = 75,
    /// `call_line_status` offset in `a`. `b` is flags plus generation.
    FighterSymbol = 76,
    /// Two instruction words from the opt-in module probe. Not a hook site.
    FighterWords = 77,
    /// Report push, pop, or clear. `a` is the kind, `b` is the level or count.
    ResolutionReport = 78,
    /// The original resolution call returned. `a` is the applied level.
    ResolutionApplied = 79,
    /// Accepted crop. `a` is width, `b` is height.
    ResolutionCrop = 80,
    /// `common` Main arm or rejection. `a` is the result, `b` is the detail.
    FighterMain = 81,
    /// Cut-in or Sephiroth start/end. Not a per-tick counter.
    FighterTransition = 82,
    /// Opt-in `PROBE_CROP` view of a game `nvnWindowSetCrop`. Not main's crop.
    ///
    /// `a`/`b` come from [`crop_rect_event`], [`crop_cap_event`], or
    /// [`crop_armed_event`]. Main's own crop stays [`Code::ResolutionCrop`].
    CropObserve = 83,
}

/// Game SetCrop observations recorded before further calls are forwarded only.
pub const CROP_OBSERVE_CAP: u32 = 32;
const CROP_PRESENT_MARK: u64 = 1 << 63;

/// `x,y,w,h` in the low 16 bits of each field. Values outside `0..=65535` truncate.
pub const fn crop_rect_word(x: i32, y: i32, w: i32, h: i32) -> u64 {
    (x as u16 as u64)
        | ((y as u16 as u64) << 16)
        | ((w as u16 as u64) << 32)
        | ((h as u16 as u64) << 48)
}

/// Rect observation. Bit 63 of `b` is clear. There is no caller word.
pub const fn crop_rect_event(present: u64, x: i32, y: i32, w: i32, h: i32) -> (u64, u64) {
    (crop_rect_word(x, y, w, h), present & !CROP_PRESENT_MARK)
}

/// The observe cap was hit. Later SetCrop calls are not recorded.
pub const fn crop_cap_event(present: u64) -> (u64, u64) {
    (CROP_PRESENT_MARK, present & !CROP_PRESENT_MARK)
}

/// The wrapper was handed to the game once. Not a crop rectangle.
pub const fn crop_armed_event() -> (u64, u64) {
    (CROP_PRESENT_MARK | 1, 0)
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
    fn crop_observe_words_separate_rect_and_cap() {
        assert_eq!(Code::CropObserve as u16, 83);
        assert_eq!(Code::ResolutionCrop as u16, 80);
        let rect = crop_rect_event(4, 0, 0, 1024, 576);
        assert_eq!(rect.0, crop_rect_word(0, 0, 1024, 576));
        assert_eq!(rect.1, 4);
        assert_eq!(
            crop_rect_word(0, 0, 1920, 1080) >> 32,
            u64::from(1920u16) | (u64::from(1080u16) << 16)
        );
        let cap = crop_cap_event(9);
        assert_eq!(cap.0, 1u64 << 63);
        assert_eq!(cap.1, 9);
        assert_ne!(cap.0, rect.0);
        assert_eq!(crop_armed_event(), ((1u64 << 63) | 1, 0));
        assert_ne!(crop_armed_event().0, cap.0);
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
