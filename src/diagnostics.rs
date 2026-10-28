//! Bounded, nonblocking event capture. File I/O belongs to the background sink.
//! Contended events are counted as dropped; a game hook never waits for a logger.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};

pub const CAPACITY: usize = 128;

/// Numbers are stable. Codes 11, 27 to 29, 84 to 87 and 91 to 97 were the
/// character-FX records; they moved to the `ssbu-fx-control` plugin in 0.5.0
/// and are not reused. Codes 75 to 77 and 83 were opt-in diagnostics retired
/// on 2026-10-16. Codes 37 to 45, 64 and 65 were the opt-in peer observer
/// (parsed observation, name sample, drops, station map, the 41 to 45 send
/// probe, the 64/65 station-map reachability sample) and code 88 was FPS++;
/// both were removed on 2026-10-17. Code 89 (`MenuOrder`, one row per capture
/// arm) stopped being emitted the same day when the captures were
/// consolidated; the variant stays so the number remains reserved. None of
/// these numbers is reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Code {
    Boot = 1,
    UnsupportedVersion = 2,
    HookInstalling = 3,
    HookInstalled = 4,
    HookRejected = 5,
    /// `a` is 0 (the FX mode until 0.5.0). `b` is the input buffer, -1 Auto.
    ConfigLoaded = 6,
    ConfigRejected = 7,
    /// `a` is 0 (the FX mode until 0.5.0). `b` is the input buffer, -1 Auto.
    MenuChanged = 8,
    LatencyApplied = 9,
    NullLatencyPointer = 10,
    CssEntered = 12,
    CssExited = 13,
    Panic = 14,
    /// `a` is 0 (the FX mode until 0.5.0). `b` is the diagnostics iteration.
    Heartbeat = 15,
    Compatibility = 16,
    HookEntryWord = 17,
    PaneAccepted = 18,
    PaneRejected = 19,
    PaneField = 20,
    OverlayEntered = 21,
    OverlayExited = 22,
    OverlayView = 23,
    /// `a` is the config generation. `b` is 0 (the FX mode until 0.5.0).
    ConfigSaved = 24,
    OverlayHealth = 25,
    OverlayBackendReady = 26,
    /// The scene tracker's online mark changed. `a` is 1 online, 0 offline;
    /// `b` is the hook address or source. Named `FxOnlineContext` before 0.5.0.
    OnlineContext = 30,
    TimingStarted = 31,
    TimingFinished = 32,
    RenderOwner = 33,
    RenderCaller = 34,
    RenderFlushTarget = 35,
    PiaRtt = 36,
    ProfileTransition = 46,
    ProfileRefused = 47,
    /// Render-opts teardown guard: a module unloaded between this frame's
    /// fighter recording and its post-scene hook, so the relocated model
    /// update and fighter worker were skipped and the game's own ordering
    /// runs for the quiet window. `a` is the unload epoch, `b` the frame.
    RenderTeardownGuard = 48,
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
    /// Report push, pop, or clear. `a` is the kind, `b` is the level or count.
    ResolutionReport = 78,
    /// The original resolution call returned. `a` is the applied level.
    ResolutionApplied = 79,
    /// Accepted crop. `a` is width, `b` is height.
    ResolutionCrop = 80,
    /// `common` Main arm or rejection. `a` is the result, `b` is the detail.
    FighterMain = 81,
    /// Cut-in start/end. `a` is the entry, the action (1 start, 2 end) in bits
    /// 8..15 and the source in bits 16..23: 1 cut-in, 3 a 0.5.2 move-table
    /// edge. Source 2 was the Sephiroth Gigaflare trigger, removed 2026-10-17
    /// and not reused. Not a per-tick counter.
    FighterTransition = 82,
    /// Reserved; no longer emitted (2026-10-17). Was one row per capture arm:
    /// `a` the capture id, `b` 1 when the embedded backend was the sampled one.
    MenuOrder = 89,
    /// Startup-only PROBE_RENDER_POLL discriminator; not a shipping fix.
    /// a=1: execute the original zero-timeout render poll without changing profile.
    RenderPollProbe = 90,
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
    /// CSS capture startup. Since 2026-10-17 `a` is the installed hook mask:
    /// bit 0 dependency wait, bit 1 UI job (3 = both, arming possible); `b`
    /// bit 0 is the `CAPTURE_CSS` flag. Both zero: no flag, no hooks. Before
    /// that, `a` bit 0 was the dispatcher-registration hook (7 = all three)
    /// and `b` bits 1–2 said whether the SDK thread-info symbols resolved.
    CssCapture = 104,
    /// Two-image LessLag presents at the flush. Only mismatches are recorded:
    /// `a` = 2 when the game then presented a different index than the one
    /// presented early (that call went through), `b` is the game's index.
    EarlyPresent = 105,
    /// `CAPTURE_MATCH` capture. `a` = 0 startup (`b` = flag present), 1 stage
    /// entered and a capture requested (`b` = scene status), 2 stage left and
    /// the capture ended (`b` = previous status), 3 a running 30 s capture was
    /// ended early for a starting match (`b` = its id). `TimingStarted` `b` = 1
    /// marks the match capture itself.
    MatchCapture = 106,
    /// Scene connection hold. `a=1`: main-menu entry kept a stored Quickplay
    /// (`b` = hook address). `a=2`: a Singles/Doubles stage ran the deadline
    /// under Quickplay without the online-latency hook and was demoted to
    /// Offline (`b` = ticks since the stage load); the matching `AutoContext`
    /// row carries source `0x0ff11e`.
    SceneHold = 107,
    /// 0.5.2 dynamic-resolution experiment trigger edge. `a`: source in bits
    /// 0..7 (1 = move table, 2 = adaptive), catalog move index in bits 8..15
    /// (moves only), fighter entry in bits 16..23 (moves only). `b`: 1 push or
    /// 2 pop in bits 0..7, the pushed level in bits 8..15, and for the adaptive
    /// source the triggering present interval in hundredths of a millisecond in
    /// bits 16..47. Edges only, never per frame.
    DrsTrigger = 108,
    /// Coverage build only (feature `coverage`): a raw LLVM profile was written
    /// under `coverage/`. `a` bits 0..7: 1 periodic `live.profraw`, 2 scene
    /// status change (bits 8..15 previous status, bits 16..23 new status),
    /// 3 manual (Debug view L+R+Left). `b`: bytes written, 0 when the write
    /// failed (the first failure is also printed to the Skyline log).
    CoverageDump = 109,
    /// A stage finished loading (`deluxe_rewrite_stage_setup`): `a` = the
    /// game's stage id, `b` = the scene status it was classified as
    /// (`SceneStatus` as u8). One edge per stage load, so a capture can be
    /// attributed to its stage.
    StageLoaded = 110,
    /// Clock probe (0.5.3): `a` = CPU MHz from the dependent-add calibration
    /// (0 unknown; 1020 stock, 1785 the game's loading boost or Horizon-OC),
    /// `b` = Horizon-OC file bits (1 hoc.kip, 2 hoc-clk autostart, 4 a Smash
    /// profile, 8 uncapped_clocks). At boot, on every stock/OC change and
    /// once a minute. See `clock_probe`.
    ClockSample = 111,
    /// Live-peer rule (0.5.5): the selected opponent station is still bound
    /// but has produced no RTT sample for `pia_rtt::LIVE_MS`, so the opponent
    /// column, strip line and CSS line were hidden. `a` = station index,
    /// `b` = the silence in ms. One row per transition, not per frame. Seen
    /// after a declined rematch whose leave never reached the station hooks.
    PeerStale = 112,
    /// A current-match Auto observation changed (0.8.0). `a` is the frame
    /// count; `b` is the verified input-consumption hook. Not a per-frame log.
    AutoBufferObserved = 113,
    /// Horizon-OC game-aware handshake (0.8.1): `a` = the
    /// `hoc_game_aware::Capability` byte (1 no `hoc:clk`, 2 stock, 3
    /// incompatible, 4 fork present, 5 fork stopped answering), `b` = the
    /// failing result or reply detail (cmd 100 result, or `caps << 16 |
    /// version` when present). One row per capability change.
    HocCapability = 114,
    /// Game-aware gate edge (0.8.1): `a` = applying (bit 0), armed (bit 1),
    /// `hoc_oc` online (bit 2), online session live (bit 3); `b` = the reply
    /// generation, or the failing result with bit 32 set. One row per change
    /// of `a`, not per heartbeat.
    HocGate = 115,
    /// Waiting-room observation at a stage load (0.8.2), right after its
    /// `StageLoaded` row: `a` = melee mode (bits 0–7), scene initialized (bit
    /// 8), reads armed by the 13.0.5 guard (bit 9), classified as lobby
    /// practice (bit 10); `b` = the game's mode word (0 while disarmed).
    SceneMode = 116,
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
    fn event_codes_keep_their_numbers() {
        assert_eq!(Code::PiaRtt as u16, 36);
        assert_eq!(Code::ProfileTransition as u16, 46);
        assert_eq!(Code::RenderTeardownGuard as u16, 48);
        assert_eq!(Code::TransportHandshake as u16, 66);
        assert_eq!(Code::FighterTransition as u16, 82);
        assert_eq!(Code::MenuOrder as u16, 89);
        assert_eq!(Code::RenderPollProbe as u16, 90);
        assert_eq!(Code::NullLatencyPointer as u16, 10);
        assert_eq!(Code::CssEntered as u16, 12);
        assert_eq!(Code::OverlayBackendReady as u16, 26);
        assert_eq!(Code::OnlineContext as u16, 30);
        assert_eq!(Code::ResolutionCrop as u16, 80);
        assert_eq!(Code::OverlayCommand as u16, 98);
        assert_eq!(Code::PluginRegistered as u16, 99);
        assert_eq!(Code::PluginRequest as u16, 100);
        assert_eq!(Code::PluginPersisted as u16, 101);
        assert_eq!(Code::DispatcherBackoff as u16, 102);
        assert_eq!(Code::DispatcherBackoffStats as u16, 103);
        assert_eq!(Code::CssCapture as u16, 104);
        assert_eq!(Code::EarlyPresent as u16, 105);
        assert_eq!(Code::MatchCapture as u16, 106);
        assert_eq!(Code::SceneHold as u16, 107);
        assert_eq!(Code::DrsTrigger as u16, 108);
        assert_eq!(Code::CoverageDump as u16, 109);
        assert_eq!(Code::StageLoaded as u16, 110);
        assert_eq!(Code::ClockSample as u16, 111);
        assert_eq!(Code::AutoBufferObserved as u16, 113);
        assert_eq!(Code::HocCapability as u16, 114);
        assert_eq!(Code::HocGate as u16, 115);
        assert_eq!(Code::SceneMode as u16, 116);
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
                        r.record(n, Code::Heartbeat, i, !i);
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
