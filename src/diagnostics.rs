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
    /// Native-window FPS++ phase in the low byte of `a` and the note in the
    /// next byte. `b` is 1 when the interval read back as applied. The
    /// tracked NVN window thread produces the pair. The diagnostics poll only
    /// publishes it, and only when it changes.
    FpsBoost = 88,
    /// One capture arm. `a` is the API-trace id. `b` is 1 when the embedded
    /// backend is the one that will be sampled. Not a per-frame record.
    MenuOrder = 89,
    /// Startup-only PROBE_RENDER_POLL discriminator; not a shipping fix.
    /// a=1: execute the original zero-timeout render poll without changing profile.
    RenderPollProbe = 90,
    /// Reduced substituted `null` for one decorative layer. `a` is the original hash.
    FxReducedCull = 84,
    /// Reduced kept a core effect request. `a` is the hash.
    FxReducedKeep = 85,
    /// Reduced was selected but scope blocked that matched request. `a` is the hash.
    FxReducedRestore = 86,
    /// A reduced-table name was seen on a different move, so it stayed On.
    FxReducedPreserve = 87,
    /// A sibling file was not registered or its load was rejected.
    /// `a` is the reason: 1 missing API, 2 transform, 3 size.
    /// `b` is that file's path hash. The original request stays. This is not Off.
    FxVariantUnavailable = 91,
    /// Sibling callback armed, or one successful load of that whole file.
    /// Armed: `a` is 0 and `b` is the path hash.
    /// Loaded: `a` is the path hash and `b` is the output length in bytes.
    /// The load counter only decides whether this record is written.
    FxVariantReady = 92,
    /// Reduced forwarded one parent to its sibling. Sampled on the first swap
    /// and every 128th. `a` is the EFF path hash. `b` is the original effect
    /// hash. The swap counter is not stored.
    FxVariantUse = 93,
    /// A kill or detach of an original sibling parent also named its clone.
    /// Sampled on the first event and every 128th. `a` is the original hash.
    /// `b` is the alias hash. This is not a per-frame record.
    FxVariantCleanup = 94,
    /// Which cleanup boundary ran. Each layer and nested bit keeps its own
    /// first event and every 128th, so one route cannot hide another.
    /// `a` is 1 `kill_kind`, 2 `detach_kind`, 3 ACMD `EFFECT_OFF_KIND`, or 4
    /// ACMD `EFFECT_DETACH_KIND`. `b` is 1 when that exact accessor, original
    /// hash, and kill-or-detach was already delivered on this thread.
    /// It is not a per-frame record.
    FxCleanupRoute = 95,
    /// Special Zoom background request. Diagnostic capture only: not a
    /// performance measurement and not online proof.
    ///
    /// `a` is the original hash. `b` bits 0..7 are the path from
    /// [`crate::critical_probe`]: 1 `req_screen`, 2 `req_screen_system`,
    /// 3 ordinary `policy_hash`. Bit 8 is set when `PROBE_CRITICAL_SCREEN`
    /// suppressed that request. The first 32 matches are kept, and every
    /// later match while a finite API trace is active. On is included.
    FxCriticalRequest = 96,
    /// Latched once from `PROBE_CRITICAL_SCREEN` when effect hooks install.
    /// `a` is 1 when that marker file was present, otherwise 0. Absent stays
    /// off. This is not a config or menu setting, and it is not performance
    /// or online proof.
    CriticalProbe = 97,
    /// A changed overlay command chord, not a per-frame input log. `a` is the
    /// sampled HID button mask. `b`: previous view in bits 0..7, resulting view
    /// 8..15, Npad id 16..23, style flags 24..55, command-edge flags 56..63
    /// (bit 0 Down pressed, bit 1 Plus/Minus pressed). Style 0 means no device.
    OverlayCommand = 98,
    /// A separate NRO called the plugin registration export. `a` is the
    /// returned handle, or the negative `REGISTER_ERR_*` code as a two's
    /// complement word. `b` is the option count that was accepted (0 on error).
    PluginRegistered = 99,
    /// One accepted Left/Right edge on the Plugins page. `a` is plugin index
    /// in bits 0..15 and option index in bits 16..31. `b` is the requested
    /// choice in bits 0..15 and the plugin's `REQUEST_*` answer in bits 16..23.
    PluginRequest = 100,
    /// Host persistence wrote (or failed to write) a plugin's own INI after a
    /// menu edit. `a` is plugin index in bits 0..15 and option index in bits
    /// 16..31. `b` is 1 on success, 0 on an I/O error.
    PluginPersisted = 101,
    /// Dispatcher backoff install outcome. `a` is the result: 1 installed,
    /// 2 another owner already patched the site, 3 site words differ from
    /// 13.0.5, 4 disabled by flag. `b` packs the policy: `spin_passes` in
    /// bits 0..15, `base_ns / 1000` in bits 16..31, `max_ns / 1000` in 32..47.
    /// `a` = 5 is written once when the eligibility gate first passed (a scene
    /// hook has fired and the tracked window is presenting); `b` is that tick.
    DispatcherBackoff = 102,
    /// Periodic backoff counters from the diagnostics poll, written only when
    /// they change. `a` is total idle passes seen at the hook site. `b` is
    /// total sleeps in bits 0..47 and the last sleep length in microseconds
    /// in bits 48..63. Not a per-frame record.
    DispatcherBackoffStats = 103,
    /// CSS capture startup. `a` is the installed hook mask: bit 0 dispatcher
    /// registration, bit 1 dependency wait, bit 2 UI job (7 = arming). `b`
    /// bit 0 is the `CAPTURE_CSS` flag, bit 1 `GetThreadPriority` resolved,
    /// bit 2 `GetThreadCoreMask` resolved. Both zero: flag absent, no hooks.
    CssCapture = 104,
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
        assert_eq!(Code::FpsBoost as u16, 88);
        assert_eq!(Code::MenuOrder as u16, 89);
        assert_eq!(Code::RenderPollProbe as u16, 90);
        assert_eq!(Code::FxReducedCull as u16, 84);
        assert_eq!(Code::FxReducedKeep as u16, 85);
        assert_eq!(Code::FxReducedRestore as u16, 86);
        assert_eq!(Code::FxReducedPreserve as u16, 87);
        assert_eq!(Code::FxVariantUnavailable as u16, 91);
        assert_eq!(Code::FxVariantReady as u16, 92);
        assert_eq!(Code::FxVariantUse as u16, 93);
        assert_eq!(Code::FxVariantCleanup as u16, 94);
        assert_eq!(Code::FxCleanupRoute as u16, 95);
        assert_eq!(Code::FxCriticalRequest as u16, 96);
        assert_eq!(Code::CriticalProbe as u16, 97);
        assert_eq!(Code::ResolutionCrop as u16, 80);
        assert_eq!(Code::OverlayCommand as u16, 98);
        assert_eq!(Code::PluginRegistered as u16, 99);
        assert_eq!(Code::PluginRequest as u16, 100);
        assert_eq!(Code::PluginPersisted as u16, 101);
        assert_eq!(Code::DispatcherBackoff as u16, 102);
        assert_eq!(Code::DispatcherBackoffStats as u16, 103);
        assert_eq!(Code::CssCapture as u16, 104);
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
