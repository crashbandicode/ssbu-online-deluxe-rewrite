//! Emulator FPS++ policy for the native-window swap interval.
//!
//! The canonical backend stores `10000` at `native_window + 0xf14` and puts
//! the owned original back when the request ends. Eden reads that word as
//! `QueueBufferInput.swap_interval` and, at `10000`, asks the compositor for
//! a 100x scheduling scale. In the measured Eden build this accelerates the
//! match timer, so it is gated to diagnostic opt-in. It does not set hardware
//! clocks or start a thread. The tracked NVN window lifecycle owns the write.
//!
//! A request applies only when the platform is the explicit emulator, the
//! saved flag is set, and the startup `ALLOW_FPS_BOOST` marker latched.
//! Without that marker an emulator request is Unsupported and does not write
//! `10000`. There is no scene gate: menus and matches use the same step.
//! The chain is `main + 0x6d43430`, then `+0x10`, then `+0xf14`.
//! The 2026-10-07 readwatch (`text 0x8036d000`, outer `0x1180470fa0`, window
//! `0x1184cfc010`) supplies the owner vtable and the `dnw_` header. The nnSdk
//! reader addresses the same word as `[x21 + 0xf24]` with
//! `x21 = native_window - 0x10`, so that header must be readable.
//!
//! Ownership starts only when the live word is the captured idle value `1`.
//! A window the chain no longer names is not written, including to restore it.
//! A mismatched readback is restored once. If that restore also fails to read
//! back, that window is latched until the request changes.

/// `*(main + OWNER_SLOT_OFFSET)` is the vi owner.
pub const OWNER_SLOT_OFFSET: u64 = 0x06d4_3430;
/// `*(owner + WINDOW_POINTER_OFFSET)` is the native window.
pub const WINDOW_POINTER_OFFSET: u64 = 0x10;
/// Swap interval. nnSdk forwards this word at queue-input `+0x2c`.
pub const INTERVAL_OFFSET: u64 = 0xf14;
/// Owner vtable from the readwatch: `0x855aef38 - 0x8036d000`.
pub const OWNER_VTABLE_OFFSET: u64 = 0x0524_1f38;
/// Captured window header `dnw_`.
pub const WINDOW_MAGIC: u32 = 0x5f77_6e64;
/// Captured second header word.
pub const WINDOW_HEADER: u32 = 0xc0;
/// Canonical enabled interval. Eden's scale is `value / 100`.
pub const ENABLED_INTERVAL: u32 = 10_000;
/// Captured idle interval. The only word this policy will replace.
pub const IDLE_INTERVAL: u32 = 1;
/// Bytes before the window pointer that the nnSdk reader treats as `x21`.
pub const SDK_HEADER_BACK: u64 = 0x10;

const MIN_ADDRESS: u64 = 0x1_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request {
    pub emulator: bool,
    pub configured: bool,
    /// Startup `ALLOW_FPS_BOOST` latch. The saved flag stays independent.
    pub diagnostic: bool,
}

impl Request {
    pub const fn wants_boost(self) -> bool {
        self.emulator && self.configured && self.diagnostic
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Phase {
    Off = 0,
    /// Stored phase 1 still decodes. The marker path now reports [`Self::Diagnostic`].
    Applied = 1,
    Unavailable = 2,
    Refused = 3,
    /// Saved emulator request with no startup marker. The interval is not written.
    Unsupported = 4,
    /// Marker path, and the enabled word read back.
    Diagnostic = 5,
}

impl Phase {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Applied => "Applied",
            Self::Unavailable => "Unavailable",
            Self::Refused => "Refused",
            Self::Unsupported => "Unsupported",
            Self::Diagnostic => "Diagnostic",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Note {
    Idle = 0,
    WroteEnabled = 1,
    Restored = 2,
    LeftUntouched = 3,
    StaleDropped = 4,
    ReadbackFailed = 5,
    ChainMissing = 6,
    BadFingerprint = 7,
    /// Ordinary emulator request. Distinct from an idle Off.
    Unsupported = 8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Outcome {
    pub phase: Phase,
    pub note: Note,
    pub applied: bool,
}

/// Saved On/Off stays in the value column. This is the separate status.
pub fn status_label(phase: Phase, request: Request) -> &'static str {
    match phase {
        Phase::Diagnostic => "Diagnostic",
        Phase::Applied if request.diagnostic => "Diagnostic",
        Phase::Applied => "Applied",
        Phase::Unavailable => "Unavailable",
        Phase::Refused => "Refused",
        Phase::Unsupported => "Unsupported",
        Phase::Off if request.configured && !request.emulator => "Console",
        Phase::Off if request.configured && request.emulator && !request.diagnostic => {
            "Unsupported"
        }
        Phase::Off => "Off",
    }
}

/// Settings value. Saved Off stays Off. Saved On keeps the gate word visible.
pub fn settings_value(saved: bool, status: &'static str) -> &'static str {
    if !saved {
        return "Off";
    }
    match status {
        "Diagnostic" => "On (Diagnostic)",
        "Applied" => "On (Applied)",
        "Unavailable" => "On (Unavailable)",
        "Refused" => "On (Refused)",
        "Unsupported" => "On (Unsupported)",
        "Console" => "On (Console)",
        _ => "On",
    }
}

/// Fork chord: the enabled bit is stored only on an emulator.
/// A console toggle clears a previously saved enabled bit.
pub fn saved_after_toggle(emulator: bool, saved: bool) -> bool {
    emulator && !saved
}

/// Fork profile suffix. Empty unless the native-window write read back.
pub fn append_applied_suffix(label: &str, applied: bool) -> String {
    if applied {
        format!("{label}++")
    } else {
        label.to_owned()
    }
}

pub trait GuestMemory {
    fn read_u32(&self, address: u64) -> Option<u32>;
    fn read_u64(&self, address: u64) -> Option<u64>;
    fn write_u32(&mut self, address: u64, value: u32) -> bool;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct State {
    owned: Option<Owned>,
    last_wants: Option<bool>,
}

impl State {
    pub const fn new() -> Self {
        Self {
            owned: None,
            last_wants: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Owned {
    window: u64,
    field: u64,
    original: u32,
    faulted: bool,
}

struct Located {
    window: u64,
    field: u64,
    value: u32,
}

enum Chain {
    Missing,
    Rejected,
    Window(u64),
}

#[derive(PartialEq, Eq)]
enum Commit {
    Held,
    Unchanged,
    Torn,
    Moved,
}

/// Half-second bound for ordinary presents. Window initialize and finalize
/// are not delayed. `19_200_000` is the same tick rate as [`crate::timing::TICK_HZ`].
pub const OWNER_INTERVAL_TICKS: u64 = crate::timing::TICK_HZ / 2;

/// Which tracked-window callback is asking to run [`step`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerEdge {
    Initialize,
    Present,
    Finalize,
}

/// Cadence for the single lifecycle owner. Present walks are skipped while the
/// request is unchanged and the previous step is still inside the interval.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Schedule {
    last_tick: Option<u64>,
    last_wants: Option<bool>,
}

impl Schedule {
    pub const fn new() -> Self {
        Self {
            last_tick: None,
            last_wants: None,
        }
    }

    pub fn due(self, edge: OwnerEdge, tick: u64, wants: bool) -> bool {
        if edge != OwnerEdge::Present {
            return true;
        }
        match (self.last_tick, self.last_wants) {
            (Some(last), Some(previous)) if previous == wants => {
                tick.wrapping_sub(last) >= OWNER_INTERVAL_TICKS
            }
            _ => true,
        }
    }

    pub fn note(&mut self, tick: u64, wants: bool) {
        self.last_tick = Some(tick);
        self.last_wants = Some(wants);
    }

    /// `u64::MAX` and `0` mean this schedule has not stepped.
    pub fn cache(self) -> (u64, u8) {
        let tick = self.last_tick.unwrap_or(u64::MAX);
        let code = match self.last_wants {
            None => 0,
            Some(false) => 1,
            Some(true) => 2,
        };
        (tick, code)
    }

    pub fn from_cache(tick: u64, code: u8) -> Self {
        Self {
            last_tick: (tick != u64::MAX).then_some(tick),
            last_wants: match code {
                1 => Some(false),
                2 => Some(true),
                _ => None,
            },
        }
    }
}

/// One owner step. The caller supplies the cadence and does not sleep or
/// spawn work. Passing the same request again does not reset a live boost.
pub fn step(state: &mut State, mem: &mut dyn GuestMemory, main: u64, request: Request) -> Outcome {
    gate_ordinary(request, decide(state, mem, main, request))
}

fn gate_ordinary(request: Request, outcome: Outcome) -> Outcome {
    if outcome.phase != Phase::Off || !request.emulator || !request.configured || request.diagnostic
    {
        return outcome;
    }
    Outcome {
        phase: Phase::Unsupported,
        note: if outcome.note == Note::Idle {
            Note::Unsupported
        } else {
            outcome.note
        },
        applied: false,
    }
}

fn decide(state: &mut State, mem: &mut dyn GuestMemory, main: u64, request: Request) -> Outcome {
    let wants = request.wants_boost();
    if state.last_wants != Some(wants) {
        if let Some(owned) = state.owned.as_mut() {
            owned.faulted = false;
        }
        state.last_wants = Some(wants);
    }

    let chain = read_chain(mem, main);
    let named = match chain {
        Chain::Window(window) => Some(window),
        Chain::Missing | Chain::Rejected => None,
    };
    let dropped = forget_if_unnamed(state, named);
    let window = match chain {
        Chain::Window(window) => window,
        Chain::Missing => return inactive(wants, dropped, Note::ChainMissing),
        Chain::Rejected => return inactive(wants, dropped, Note::BadFingerprint),
    };
    let loc = match inspect(mem, window) {
        Ok(loc) => loc,
        Err(()) if state.owned.is_some() => {
            return Outcome {
                phase: Phase::Unavailable,
                note: Note::BadFingerprint,
                applied: false,
            };
        }
        Err(()) => return inactive(wants, dropped, Note::BadFingerprint),
    };
    if state
        .owned
        .is_some_and(|owned| owned.faulted && owned.window == loc.window)
    {
        return Outcome {
            phase: Phase::Unavailable,
            note: Note::ReadbackFailed,
            applied: false,
        };
    }
    if let Some(owned) = state.owned {
        step_owned(state, mem, main, wants, owned, &loc)
    } else {
        step_unowned(state, mem, main, wants, &loc, dropped)
    }
}

fn forget_if_unnamed(state: &mut State, named: Option<u64>) -> bool {
    let Some(owned) = state.owned else {
        return false;
    };
    if named == Some(owned.window) {
        return false;
    }
    state.owned = None;
    true
}

fn inactive(wants: bool, dropped: bool, reason: Note) -> Outcome {
    let note = if dropped {
        Note::StaleDropped
    } else if wants {
        reason
    } else {
        Note::Idle
    };
    Outcome {
        phase: if wants {
            Phase::Unavailable
        } else {
            Phase::Off
        },
        note,
        applied: false,
    }
}

fn step_owned(
    state: &mut State,
    mem: &mut dyn GuestMemory,
    main: u64,
    wants: bool,
    owned: Owned,
    loc: &Located,
) -> Outcome {
    if wants {
        if loc.value == ENABLED_INTERVAL {
            return Outcome {
                phase: Phase::Diagnostic,
                note: Note::Idle,
                applied: true,
            };
        }
        if loc.value == owned.original {
            return write_enabled(state, mem, main, loc, owned.original);
        }
        return Outcome {
            phase: Phase::Refused,
            note: Note::LeftUntouched,
            applied: false,
        };
    }
    if loc.value == owned.original {
        state.owned = None;
        return Outcome {
            phase: Phase::Off,
            note: Note::Idle,
            applied: false,
        };
    }
    if loc.value != ENABLED_INTERVAL {
        return Outcome {
            phase: Phase::Refused,
            note: Note::LeftUntouched,
            applied: false,
        };
    }
    match store(mem, main, loc.field, owned.original) {
        Commit::Held => {
            state.owned = None;
            Outcome {
                phase: Phase::Off,
                note: Note::Restored,
                applied: false,
            }
        }
        Commit::Unchanged => Outcome {
            phase: Phase::Unavailable,
            note: Note::ReadbackFailed,
            applied: false,
        },
        Commit::Torn => {
            state.owned = Some(Owned {
                faulted: true,
                ..owned
            });
            Outcome {
                phase: Phase::Unavailable,
                note: Note::ReadbackFailed,
                applied: false,
            }
        }
        Commit::Moved => {
            state.owned = None;
            Outcome {
                phase: Phase::Unavailable,
                note: Note::StaleDropped,
                applied: false,
            }
        }
    }
}

fn step_unowned(
    state: &mut State,
    mem: &mut dyn GuestMemory,
    main: u64,
    wants: bool,
    loc: &Located,
    dropped: bool,
) -> Outcome {
    if !wants {
        return Outcome {
            phase: Phase::Off,
            note: if dropped {
                Note::StaleDropped
            } else {
                Note::Idle
            },
            applied: false,
        };
    }
    if loc.value != IDLE_INTERVAL {
        return Outcome {
            phase: Phase::Refused,
            note: Note::LeftUntouched,
            applied: false,
        };
    }
    write_enabled(state, mem, main, loc, IDLE_INTERVAL)
}

fn write_enabled(
    state: &mut State,
    mem: &mut dyn GuestMemory,
    main: u64,
    loc: &Located,
    original: u32,
) -> Outcome {
    match store(mem, main, loc.field, ENABLED_INTERVAL) {
        Commit::Held => {
            state.owned = Some(Owned {
                window: loc.window,
                field: loc.field,
                original,
                faulted: false,
            });
            Outcome {
                phase: Phase::Diagnostic,
                note: Note::WroteEnabled,
                applied: true,
            }
        }
        Commit::Unchanged => Outcome {
            phase: Phase::Unavailable,
            note: Note::ReadbackFailed,
            applied: false,
        },
        Commit::Moved => {
            state.owned = None;
            Outcome {
                phase: Phase::Unavailable,
                note: Note::StaleDropped,
                applied: false,
            }
        }
        Commit::Torn => {
            let restored = store(mem, main, loc.field, original) == Commit::Held;
            state.owned = if restored {
                None
            } else {
                Some(Owned {
                    window: loc.window,
                    field: loc.field,
                    original,
                    faulted: true,
                })
            };
            Outcome {
                phase: Phase::Unavailable,
                note: Note::ReadbackFailed,
                applied: false,
            }
        }
    }
}

fn store(mem: &mut dyn GuestMemory, main: u64, field: u64, value: u32) -> Commit {
    let Some(loc) = live(mem, main) else {
        return Commit::Moved;
    };
    if loc.field != field {
        return Commit::Moved;
    }
    if !mem.write_u32(field, value) {
        return Commit::Unchanged;
    }
    if mem.read_u32(field) != Some(value) {
        return if still_named(mem, main, field) {
            Commit::Torn
        } else {
            Commit::Moved
        };
    }
    if still_named(mem, main, field) {
        Commit::Held
    } else {
        Commit::Moved
    }
}

fn still_named(mem: &dyn GuestMemory, main: u64, field: u64) -> bool {
    live(mem, main).is_some_and(|loc| loc.field == field)
}

fn live(mem: &dyn GuestMemory, main: u64) -> Option<Located> {
    let Chain::Window(window) = read_chain(mem, main) else {
        return None;
    };
    inspect(mem, window).ok()
}

fn read_chain(mem: &dyn GuestMemory, main: u64) -> Chain {
    let Some(slot) = main.checked_add(OWNER_SLOT_OFFSET) else {
        return Chain::Missing;
    };
    let Some(owner) = mem.read_u64(slot) else {
        return Chain::Missing;
    };
    if !aligned_ptr(owner) {
        return Chain::Missing;
    }
    let Some(vtable) = mem.read_u64(owner) else {
        return Chain::Missing;
    };
    let Some(expected) = main.checked_add(OWNER_VTABLE_OFFSET) else {
        return Chain::Missing;
    };
    if vtable != expected {
        return Chain::Rejected;
    }
    let Some(window_at) = owner.checked_add(WINDOW_POINTER_OFFSET) else {
        return Chain::Missing;
    };
    let Some(window) = mem.read_u64(window_at) else {
        return Chain::Missing;
    };
    if aligned_ptr(window) {
        Chain::Window(window)
    } else {
        Chain::Missing
    }
}

fn inspect(mem: &dyn GuestMemory, window: u64) -> Result<Located, ()> {
    let header = window.checked_sub(SDK_HEADER_BACK).ok_or(())?;
    if mem.read_u64(header).is_none() {
        return Err(());
    }
    if mem.read_u32(window) != Some(WINDOW_MAGIC) || mem.read_u32(window + 4) != Some(WINDOW_HEADER)
    {
        return Err(());
    }
    let field = window.checked_add(INTERVAL_OFFSET).ok_or(())?;
    let value = mem.read_u32(field).ok_or(())?;
    Ok(Located {
        window,
        field,
        value,
    })
}

fn aligned_ptr(value: u64) -> bool {
    value >= MIN_ADDRESS && value.is_multiple_of(8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const MAIN: u64 = 0x8000_0000;
    const OWNER: u64 = 0x1_0000_1000;
    const WINDOW: u64 = 0x1_0000_2000;
    const WINDOW_B: u64 = 0x1_0000_4000;

    #[derive(Clone, Default)]
    struct MapMem {
        bytes: BTreeMap<u64, u8>,
        writes: u32,
    }

    impl MapMem {
        fn store_u32(&mut self, address: u64, value: u32) {
            for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.bytes.insert(address + index as u64, byte);
            }
        }

        fn store_u64(&mut self, address: u64, value: u64) {
            for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.bytes.insert(address + index as u64, byte);
            }
        }

        fn word(&self, address: u64) -> Option<u32> {
            let mut bytes = [0u8; 4];
            for (index, slot) in bytes.iter_mut().enumerate() {
                *slot = *self.bytes.get(&(address + index as u64))?;
            }
            Some(u32::from_le_bytes(bytes))
        }

        fn plant(&mut self, window: u64, interval: u32) {
            self.store_u64(window - SDK_HEADER_BACK, 0x1111_2222_3333_4444);
            self.store_u32(window, WINDOW_MAGIC);
            self.store_u32(window + 4, WINDOW_HEADER);
            self.store_u32(window + INTERVAL_OFFSET, interval);
        }
    }

    impl GuestMemory for MapMem {
        fn read_u32(&self, address: u64) -> Option<u32> {
            if !address.is_multiple_of(4) {
                return None;
            }
            self.word(address)
        }

        fn read_u64(&self, address: u64) -> Option<u64> {
            if !address.is_multiple_of(8) {
                return None;
            }
            let mut bytes = [0u8; 8];
            for (index, slot) in bytes.iter_mut().enumerate() {
                *slot = *self.bytes.get(&(address + index as u64))?;
            }
            Some(u64::from_le_bytes(bytes))
        }

        fn write_u32(&mut self, address: u64, value: u32) -> bool {
            if !address.is_multiple_of(4) || self.word(address).is_none() {
                return false;
            }
            self.writes += 1;
            self.store_u32(address, value);
            true
        }
    }

    struct Lie {
        inner: MapMem,
        field: u64,
        lie: bool,
    }

    impl GuestMemory for Lie {
        fn read_u32(&self, address: u64) -> Option<u32> {
            if self.lie && address == self.field {
                return Some(99);
            }
            self.inner.read_u32(address)
        }

        fn read_u64(&self, address: u64) -> Option<u64> {
            self.inner.read_u64(address)
        }

        fn write_u32(&mut self, address: u64, value: u32) -> bool {
            let wrote = self.inner.write_u32(address, value);
            if wrote && value == ENABLED_INTERVAL {
                self.lie = true;
            }
            wrote
        }
    }

    fn vanilla() -> MapMem {
        let mut mem = MapMem::default();
        mem.store_u64(MAIN + OWNER_SLOT_OFFSET, OWNER);
        mem.store_u64(OWNER, MAIN + OWNER_VTABLE_OFFSET);
        mem.store_u64(OWNER + WINDOW_POINTER_OFFSET, WINDOW);
        mem.plant(WINDOW, IDLE_INTERVAL);
        mem
    }

    fn emulator(configured: bool) -> Request {
        Request {
            emulator: true,
            configured,
            diagnostic: false,
        }
    }

    fn allowed(configured: bool) -> Request {
        Request {
            emulator: true,
            configured,
            diagnostic: true,
        }
    }

    fn field_of(window: u64) -> u64 {
        window + INTERVAL_OFFSET
    }

    #[test]
    fn off_and_console_leave_the_idle_word_untouched() {
        let mut mem = vanilla();
        let mut state = State::default();
        let off = step(&mut state, &mut mem, MAIN, emulator(false));
        assert_eq!(off.phase, Phase::Off);
        assert!(!off.applied);
        assert_eq!(mem.writes, 0);
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
        let console = step(
            &mut state,
            &mut mem,
            MAIN,
            Request {
                emulator: false,
                configured: true,
                diagnostic: false,
            },
        );
        assert_eq!(console.phase, Phase::Off);
        assert_eq!(
            status_label(
                console.phase,
                Request {
                    emulator: false,
                    configured: true,
                    diagnostic: false,
                }
            ),
            "Console"
        );
        assert_eq!(mem.writes, 0);
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
    }

    #[test]
    fn emulator_opt_in_applies_without_a_scene_and_holds_across_polls() {
        let mut mem = vanilla();
        let mut state = State::default();
        let first = step(&mut state, &mut mem, MAIN, allowed(true));
        assert_eq!(
            (first.phase, first.note, first.applied),
            (Phase::Diagnostic, Note::WroteEnabled, true)
        );
        assert_eq!(mem.word(field_of(WINDOW)), Some(ENABLED_INTERVAL));
        assert_eq!(mem.writes, 1);
        let second = step(&mut state, &mut mem, MAIN, allowed(true));
        assert_eq!(
            (second.phase, second.note, second.applied),
            (Phase::Diagnostic, Note::Idle, true)
        );
        assert_eq!(mem.writes, 1);
        let restored = step(&mut state, &mut mem, MAIN, emulator(false));
        assert_eq!(
            (restored.phase, restored.note, restored.applied),
            (Phase::Off, Note::Restored, false)
        );
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
        assert_eq!(mem.writes, 2);
        let again = step(&mut state, &mut mem, MAIN, allowed(true));
        assert!(again.applied);
        assert_eq!(mem.word(field_of(WINDOW)), Some(ENABLED_INTERVAL));
        assert_eq!(mem.writes, 3);
        let console = step(
            &mut state,
            &mut mem,
            MAIN,
            Request {
                emulator: false,
                configured: true,
                diagnostic: false,
            },
        );
        assert_eq!(
            (console.phase, console.note, console.applied),
            (Phase::Off, Note::Restored, false)
        );
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
        assert_eq!(mem.writes, 4);
    }

    #[test]
    fn unexpected_and_unowned_boost_words_are_not_replaced() {
        for initial in [0, 2, 7, ENABLED_INTERVAL] {
            let mut mem = vanilla();
            mem.store_u32(field_of(WINDOW), initial);
            let mut state = State::default();
            let outcome = step(&mut state, &mut mem, MAIN, allowed(true));
            assert_eq!(outcome.phase, Phase::Refused, "initial {initial}");
            assert!(!outcome.applied);
            assert_eq!(outcome.note, Note::LeftUntouched);
            assert_eq!(mem.writes, 0);
            assert_eq!(mem.word(field_of(WINDOW)), Some(initial));
        }
    }

    #[test]
    fn a_changed_owned_word_is_not_overwritten_on_disable() {
        let mut mem = vanilla();
        let mut state = State::default();
        assert!(step(&mut state, &mut mem, MAIN, allowed(true)).applied);
        mem.store_u32(field_of(WINDOW), 7);
        let writes = mem.writes;
        let outcome = step(&mut state, &mut mem, MAIN, emulator(false));
        assert_eq!(outcome.phase, Phase::Refused);
        assert!(!outcome.applied);
        assert_eq!(mem.writes, writes);
        assert_eq!(mem.word(field_of(WINDOW)), Some(7));
    }

    #[test]
    fn missing_chain_and_bad_fingerprint_do_not_write() {
        let mut missing = vanilla();
        missing.store_u64(MAIN + OWNER_SLOT_OFFSET, 0);
        let mut state = State::default();
        let outcome = step(&mut state, &mut missing, MAIN, allowed(true));
        assert_eq!(outcome.note, Note::ChainMissing);
        assert_eq!(outcome.phase, Phase::Unavailable);
        assert_eq!(missing.writes, 0);

        let mut magic = vanilla();
        magic.store_u32(WINDOW, 0);
        let outcome = step(&mut state, &mut magic, MAIN, allowed(true));
        assert_eq!(outcome.note, Note::BadFingerprint);
        assert_eq!(magic.writes, 0);
        assert_eq!(magic.word(field_of(WINDOW)), Some(IDLE_INTERVAL));

        let mut header = vanilla();
        for offset in 0..8u64 {
            let _ = header.bytes.remove(&(WINDOW - SDK_HEADER_BACK + offset));
        }
        let outcome = step(&mut state, &mut header, MAIN, allowed(true));
        assert_eq!(outcome.note, Note::BadFingerprint);
        assert_eq!(header.writes, 0);

        let mut vtable = vanilla();
        vtable.store_u64(OWNER, MAIN + 0x1000);
        let outcome = step(&mut state, &mut vtable, MAIN, allowed(true));
        assert_eq!(outcome.note, Note::BadFingerprint);
        assert_eq!(vtable.writes, 0);
    }

    #[test]
    fn bad_magic_defers_restore_until_the_header_matches() {
        let mut mem = vanilla();
        let mut state = State::default();
        assert!(step(&mut state, &mut mem, MAIN, allowed(true)).applied);
        mem.store_u32(WINDOW, 0);
        let blocked = step(&mut state, &mut mem, MAIN, emulator(false));
        assert_eq!(blocked.phase, Phase::Unavailable);
        assert_eq!(blocked.note, Note::BadFingerprint);
        assert_eq!(mem.writes, 1);
        assert_eq!(mem.word(field_of(WINDOW)), Some(ENABLED_INTERVAL));
        mem.store_u32(WINDOW, WINDOW_MAGIC);
        let restored = step(&mut state, &mut mem, MAIN, emulator(false));
        assert_eq!(restored.note, Note::Restored);
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
        assert_eq!(mem.writes, 2);
    }

    #[test]
    fn a_replaced_window_is_not_written_and_the_new_idle_window_can_be_owned() {
        let mut mem = vanilla();
        let mut state = State::default();
        assert!(step(&mut state, &mut mem, MAIN, allowed(true)).applied);
        mem.plant(WINDOW_B, IDLE_INTERVAL);
        mem.store_u64(OWNER + WINDOW_POINTER_OFFSET, WINDOW_B);
        let moved = step(&mut state, &mut mem, MAIN, allowed(true));
        assert!(moved.applied);
        assert_eq!(mem.word(field_of(WINDOW)), Some(ENABLED_INTERVAL));
        assert_eq!(mem.word(field_of(WINDOW_B)), Some(ENABLED_INTERVAL));
        assert_eq!(mem.writes, 2);
    }

    #[test]
    fn a_lost_chain_drops_ownership_without_another_write() {
        let mut mem = vanilla();
        let mut state = State::default();
        assert!(step(&mut state, &mut mem, MAIN, allowed(true)).applied);
        mem.store_u64(MAIN + OWNER_SLOT_OFFSET, 0);
        let lost = step(&mut state, &mut mem, MAIN, emulator(false));
        assert_eq!(lost.note, Note::StaleDropped);
        assert!(!lost.applied);
        assert_eq!(mem.writes, 1);
        assert_eq!(mem.word(field_of(WINDOW)), Some(ENABLED_INTERVAL));
    }

    #[test]
    fn a_failed_readback_restores_once_and_then_stops() {
        let inner = vanilla();
        let field = field_of(WINDOW);
        let mut mem = Lie {
            inner,
            field,
            lie: false,
        };
        let mut state = State::default();
        let failed = step(&mut state, &mut mem, MAIN, allowed(true));
        assert_eq!(failed.phase, Phase::Unavailable);
        assert_eq!(failed.note, Note::ReadbackFailed);
        assert!(!failed.applied);
        assert_eq!(mem.inner.word(field), Some(IDLE_INTERVAL));
        assert_eq!(mem.inner.writes, 2);
        let stopped = step(&mut state, &mut mem, MAIN, allowed(true));
        assert_eq!(stopped.note, Note::ReadbackFailed);
        assert_eq!(mem.inner.writes, 2);
    }

    #[test]
    fn suffix_and_saved_toggle_follow_the_fork() {
        assert_eq!(append_applied_suffix("LLUltra", false), "LLUltra");
        assert_eq!(append_applied_suffix("Vanilla", true), "Vanilla++");
        assert_eq!(append_applied_suffix("LessLag", false), "LessLag");
        assert!(saved_after_toggle(true, false));
        assert!(!saved_after_toggle(true, true));
        assert!(!saved_after_toggle(false, false));
        assert!(!saved_after_toggle(false, true));
        assert_eq!(status_label(Phase::Diagnostic, allowed(true)), "Diagnostic");
        assert_eq!(status_label(Phase::Applied, allowed(true)), "Diagnostic");
        assert_eq!(status_label(Phase::Applied, emulator(true)), "Applied");
        assert_eq!(
            status_label(Phase::Unsupported, emulator(true)),
            "Unsupported"
        );
        assert_eq!(status_label(Phase::Off, emulator(true)), "Unsupported");
        assert_eq!(status_label(Phase::Off, emulator(false)), "Off");
        assert_eq!(
            status_label(
                Phase::Off,
                Request {
                    emulator: false,
                    configured: true,
                    diagnostic: true,
                }
            ),
            "Console"
        );
        assert_eq!(
            status_label(Phase::Unavailable, allowed(true)),
            "Unavailable"
        );
        assert_eq!(status_label(Phase::Refused, allowed(true)), "Refused");
    }

    #[test]
    fn present_walks_are_bounded_and_window_edges_always_run() {
        let mut schedule = Schedule::new();
        assert!(schedule.due(OwnerEdge::Present, 1_000, true));
        schedule.note(1_000, true);
        let (tick, code) = schedule.cache();
        assert_eq!(Schedule::from_cache(tick, code), schedule);
        assert!(!schedule.due(OwnerEdge::Present, 1_001, true));
        assert!(!schedule.due(OwnerEdge::Present, 1_000 + OWNER_INTERVAL_TICKS - 1, true));
        assert!(schedule.due(OwnerEdge::Present, 1_000 + OWNER_INTERVAL_TICKS, true));
        assert!(schedule.due(OwnerEdge::Present, 1_050, false));
        assert!(schedule.due(OwnerEdge::Initialize, 1_050, true));
        assert!(schedule.due(OwnerEdge::Finalize, 1_050, true));
        schedule.note(5_000, true);
        assert!(schedule.due(OwnerEdge::Present, 4_000, true));
    }

    #[test]
    fn ordinary_emulator_request_never_writes_the_interval() {
        let mut mem = vanilla();
        let mut state = State::default();
        let request = emulator(true);
        let outcome = step(&mut state, &mut mem, MAIN, request);
        assert_eq!(
            (outcome.phase, outcome.note, outcome.applied),
            (Phase::Unsupported, Note::Unsupported, false)
        );
        assert_eq!(status_label(outcome.phase, request), "Unsupported");
        assert_eq!(mem.writes, 0);
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
        let again = step(&mut state, &mut mem, MAIN, request);
        assert_eq!(again.phase, Phase::Unsupported);
        assert!(!again.applied);
        assert_eq!(mem.writes, 0);
        mem.store_u32(field_of(WINDOW), ENABLED_INTERVAL);
        let foreign = step(&mut state, &mut mem, MAIN, request);
        assert_eq!(foreign.phase, Phase::Unsupported);
        assert!(!foreign.applied);
        assert_eq!(mem.writes, 0);
        assert_eq!(mem.word(field_of(WINDOW)), Some(ENABLED_INTERVAL));
    }

    #[test]
    fn diagnostic_opt_in_restores_on_off_and_ordinary_request_does_not_reapply() {
        let mut mem = vanilla();
        let mut state = State::default();
        let on = step(&mut state, &mut mem, MAIN, allowed(true));
        assert_eq!(
            (on.phase, on.note, on.applied),
            (Phase::Diagnostic, Note::WroteEnabled, true)
        );
        assert_eq!(status_label(on.phase, allowed(true)), "Diagnostic");
        assert_eq!(mem.word(field_of(WINDOW)), Some(ENABLED_INTERVAL));
        assert_eq!(mem.writes, 1);
        // Finalize passes configured=false and leaves the saved flag alone.
        let off = step(&mut state, &mut mem, MAIN, allowed(false));
        assert_eq!(
            (off.phase, off.note, off.applied),
            (Phase::Off, Note::Restored, false)
        );
        assert_eq!(status_label(off.phase, allowed(false)), "Off");
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
        let again = step(&mut state, &mut mem, MAIN, allowed(true));
        assert_eq!(again.phase, Phase::Diagnostic);
        assert!(again.applied);
        let blocked = step(&mut state, &mut mem, MAIN, emulator(true));
        assert_eq!(
            (blocked.phase, blocked.note, blocked.applied),
            (Phase::Unsupported, Note::Restored, false)
        );
        assert_eq!(status_label(blocked.phase, emulator(true)), "Unsupported");
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
        let held = step(&mut state, &mut mem, MAIN, emulator(true));
        assert_eq!(
            (held.phase, held.note),
            (Phase::Unsupported, Note::Unsupported)
        );
        assert!(!held.applied);
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
        assert_eq!(mem.writes, 4);
    }

    #[test]
    fn console_stays_console_when_the_marker_is_present() {
        let mut mem = vanilla();
        let mut state = State::default();
        let request = Request {
            emulator: false,
            configured: true,
            diagnostic: true,
        };
        let outcome = step(&mut state, &mut mem, MAIN, request);
        assert_eq!(outcome.phase, Phase::Off);
        assert!(!outcome.applied);
        assert_eq!(status_label(outcome.phase, request), "Console");
        assert_eq!(mem.writes, 0);
        assert_eq!(mem.word(field_of(WINDOW)), Some(IDLE_INTERVAL));
    }
}
