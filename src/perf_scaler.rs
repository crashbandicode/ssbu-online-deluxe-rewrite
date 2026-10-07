//! Eight-entry cut-in and Sephiroth tick policy from the quickplay fork.
//!
//! Tokens are the resolution report tokens. This module does not read or write
//! the game. The runtime calls it only after logical Main is armed and
//! count-only mode is off.

use crate::resolution::{self, Level, PushResult, ReportToken};
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU32, AtomicU8, Ordering};

pub const ENTRIES: usize = 8;
/// Fork `CRITICAL_HIT_FINISH_COOLDOWN_FRAMES`.
pub const CUT_IN_COOLDOWN_FRAMES: i8 = 7;
/// 13.0.5 `FighterCutInStart`. The runtime installs it only after Main arms
/// and count-only mode is off.
pub const CUT_IN_OFFSET: usize = 0x06a_7100;
/// 13.0.5 entry word at [`CUT_IN_OFFSET`].
pub const CUT_IN_PROLOGUE: u32 = 0xd106_83ff;
/// skyline-smash bed642e `lua_const.rs`.
pub const EDGE_KIND: i32 = 0x59;
pub const EDGE_SPECIAL_N_SHOOT: i32 = 0x1e5;
pub const EDGE_CHARGE_KIND: i32 = 0x1100_0005;
pub const CHARGE_FULL: i32 = 3;
pub const ENTRY_ID_WORK: i32 = 0x1000_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Ignored,
    Idle,
    Held,
    Already,
    Started,
    NotStored,
    Cooling { left: i8 },
    Ended,
}

/// Fork `push_dynamic_res_report`: 576 and below go to 480, 720 goes to 576,
/// and 900/1080 go to 720. Discriminants match ultelier `ResolutionLevel`.
pub const fn intensive_level(base: Level) -> Level {
    if (base as u8) >= (Level::P576 as u8) {
        Level::P480
    } else if (base as u8) >= (Level::P720 as u8) {
        Level::P576
    } else {
        Level::P720
    }
}

/// Cut-in stays uninstalled until the fighter callback is armed and the live
/// word is still the 13.0.5 prologue. A far branch or any other word refuses.
pub const fn cut_in_install_allowed(callback_armed: bool, word: u32) -> bool {
    callback_armed && word == CUT_IN_PROLOGUE
}

struct CutSlot {
    active: AtomicBool,
    cooldown: AtomicI8,
    token: AtomicU32,
}

struct EdgeSlot {
    active: AtomicBool,
    token: AtomicU32,
}

impl CutSlot {
    const fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            cooldown: AtomicI8::new(CUT_IN_COOLDOWN_FRAMES),
            token: AtomicU32::new(0),
        }
    }
}

impl EdgeSlot {
    const fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            token: AtomicU32::new(0),
        }
    }
}

pub struct Scaler {
    base: AtomicU8,
    cut_in: [CutSlot; ENTRIES],
    edge: [EdgeSlot; ENTRIES],
}

impl Scaler {
    pub const fn new(base: Level) -> Self {
        Self {
            base: AtomicU8::new(base as u8),
            cut_in: [const { CutSlot::new() }; ENTRIES],
            edge: [const { EdgeSlot::new() }; ENTRIES],
        }
    }

    pub fn base_level(&self) -> Level {
        Level::from_u8(self.base.load(Ordering::Acquire)).unwrap_or(Level::P1080)
    }

    pub fn set_base(&self, level: Level) {
        self.base.store(level as u8, Ordering::Release);
    }

    pub fn cut_in_start(&self, entry_id: i32, entry_present: bool) -> Action {
        let Some(index) = entry_slot(entry_id, entry_present) else {
            return Action::Ignored;
        };
        let slot = &self.cut_in[index];
        if slot
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            slot.cooldown.store(CUT_IN_COOLDOWN_FRAMES, Ordering::Release);
            return Action::Already;
        }
        slot.cooldown
            .store(CUT_IN_COOLDOWN_FRAMES, Ordering::Release);
        match resolution::push_report(intensive_level(self.base_level())) {
            PushResult::Stored(token) => {
                slot.token.store(pack(token), Ordering::Release);
                Action::Started
            }
            PushResult::Ignored | PushResult::Full => {
                slot.active.store(false, Ordering::Release);
                Action::NotStored
            }
        }
    }

    pub fn cut_in_tick(&self, entry_id: i32, entry_present: bool, slow: bool) -> Action {
        let Some(index) = entry_slot(entry_id, entry_present) else {
            return Action::Ignored;
        };
        let slot = &self.cut_in[index];
        if !slot.active.load(Ordering::Acquire) {
            return Action::Idle;
        }
        if slot.token.load(Ordering::Acquire) == 0 {
            return Action::Held;
        }
        if slow {
            slot.cooldown
                .store(CUT_IN_COOLDOWN_FRAMES, Ordering::Release);
            return Action::Held;
        }
        let previous = slot.cooldown.fetch_sub(1, Ordering::AcqRel);
        let left = previous.wrapping_sub(1);
        if left <= 0 {
            pop_packed(&slot.token);
            slot.active.store(false, Ordering::Release);
            slot.cooldown
                .store(CUT_IN_COOLDOWN_FRAMES, Ordering::Release);
            return Action::Ended;
        }
        Action::Cooling { left }
    }

    pub fn sephiroth_tick(
        &self,
        entry_id: i32,
        entry_present: bool,
        status: i32,
        charge: i32,
    ) -> Action {
        let Some(index) = entry_slot(entry_id, entry_present) else {
            return Action::Ignored;
        };
        let slot = &self.edge[index];
        let shooting = status == EDGE_SPECIAL_N_SHOOT && charge >= CHARGE_FULL;
        if shooting {
            if slot
                .active
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                return Action::Held;
            }
            match resolution::push_report(intensive_level(self.base_level())) {
                PushResult::Stored(token) => {
                    slot.token.store(pack(token), Ordering::Release);
                    Action::Started
                }
                PushResult::Ignored | PushResult::Full => {
                    slot.active.store(false, Ordering::Release);
                    Action::NotStored
                }
            }
        } else if slot.active.load(Ordering::Acquire) {
            if slot.token.load(Ordering::Acquire) == 0 {
                return Action::Held;
            }
            pop_packed(&slot.token);
            slot.active.store(false, Ordering::Release);
            Action::Ended
        } else {
            Action::Idle
        }
    }

    /// Both fork callbacks for one Main tick. Sephiroth runs only for Edge.
    /// The cut-in cooldown runs for every present entry.
    pub fn on_main(
        &self,
        entry_id: i32,
        entry_present: bool,
        kind: i32,
        slow: bool,
        status: i32,
        charge: i32,
    ) -> (Action, Action) {
        let cut = self.cut_in_tick(entry_id, entry_present, slow);
        let edge = if kind == EDGE_KIND {
            self.sephiroth_tick(entry_id, entry_present, status, charge)
        } else {
            Action::Ignored
        };
        (cut, edge)
    }

    pub fn clear(&self) {
        for slot in &self.cut_in {
            slot.active.store(false, Ordering::Release);
            pop_packed(&slot.token);
            slot.cooldown
                .store(CUT_IN_COOLDOWN_FRAMES, Ordering::Release);
        }
        for slot in &self.edge {
            slot.active.store(false, Ordering::Release);
            pop_packed(&slot.token);
        }
    }
}

static GLOBAL: Scaler = Scaler::new(Level::P1080);

pub fn global() -> &'static Scaler {
    &GLOBAL
}

pub fn clear_global() {
    GLOBAL.clear();
}

fn entry_slot(entry_id: i32, entry_present: bool) -> Option<usize> {
    if !entry_present {
        return None;
    }
    usize::try_from(entry_id)
        .ok()
        .filter(|index| *index < ENTRIES)
}

fn pack(token: ReportToken) -> u32 {
    (u32::from(token.generation) << 8) | u32::from(token.index)
}

fn unpack(packed: u32) -> ReportToken {
    ReportToken {
        index: (packed & 0xff) as u8,
        generation: (packed >> 8) as u16,
    }
}

fn pop_packed(slot: &AtomicU32) {
    let packed = slot.swap(0, Ordering::AcqRel);
    if packed != 0 {
        let _ = resolution::pop_report(unpack(packed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolution::{self, Level};
    use std::sync::Mutex;

    static LOCK: Mutex<()> = Mutex::new(());

    fn with_reports(base: Level, test: impl FnOnce(&Scaler)) {
        let _guard = LOCK.lock().unwrap_or_else(|err| err.into_inner());
        resolution::clear_reports();
        resolution::update_settings(Level::P1080, true);
        let scaler = Scaler::new(base);
        test(&scaler);
        scaler.clear();
        resolution::clear_reports();
        resolution::update_settings(Level::P1080, false);
    }

    #[test]
    fn fork_levels_and_cut_in_guard() {
        assert_eq!(intensive_level(Level::P1080), Level::P720);
        assert_eq!(intensive_level(Level::P900), Level::P720);
        assert_eq!(intensive_level(Level::P720), Level::P576);
        assert_eq!(intensive_level(Level::P576), Level::P480);
        assert_eq!(intensive_level(Level::P480), Level::P480);
        assert!(!cut_in_install_allowed(false, CUT_IN_PROLOGUE));
        assert!(!cut_in_install_allowed(true, 0x5800_0051));
        assert!(!cut_in_install_allowed(true, 0));
        assert!(cut_in_install_allowed(true, CUT_IN_PROLOGUE));
        assert_eq!(EDGE_CHARGE_KIND, 0x1100_0005);
        assert_eq!(ENTRY_ID_WORK, 0x1000_0000);
    }

    #[test]
    fn cut_in_pops_on_the_seventh_clear_tick_and_resets_while_slow() {
        with_reports(Level::P1080, |scaler| {
            assert_eq!(scaler.cut_in_start(0, true), Action::Started);
            assert_eq!(resolution::status().reports, 1);
            assert_eq!(resolution::status().effective, Level::P720);
            assert_eq!(
                scaler.cut_in_tick(0, true, false),
                Action::Cooling { left: 6 }
            );
            assert_eq!(scaler.cut_in_tick(0, true, true), Action::Held);
            let mut lefts = Vec::new();
            for _ in 0..6 {
                match scaler.cut_in_tick(0, true, false) {
                    Action::Cooling { left } => lefts.push(left),
                    other => panic!("expected cooling, got {other:?}"),
                }
            }
            assert_eq!(lefts, vec![6, 5, 4, 3, 2, 1]);
            assert_eq!(scaler.cut_in_tick(0, true, false), Action::Ended);
            assert_eq!(resolution::status().reports, 0);
            assert_eq!(resolution::status().effective, Level::P1080);
            assert_eq!(scaler.cut_in_tick(0, true, false), Action::Idle);
        });
    }

    #[test]
    fn eight_entries_are_independent_and_out_of_range_is_ignored() {
        with_reports(Level::P1080, |scaler| {
            for entry in 0..ENTRIES as i32 {
                assert_eq!(scaler.cut_in_start(entry, true), Action::Started);
            }
            assert_eq!(resolution::status().reports, ENTRIES as u8);
            assert_eq!(scaler.cut_in_start(0, true), Action::Already);
            assert_eq!(resolution::status().reports, ENTRIES as u8);
            assert_eq!(scaler.cut_in_start(8, true), Action::Ignored);
            assert_eq!(scaler.cut_in_start(-1, true), Action::Ignored);
            assert_eq!(scaler.cut_in_start(0, false), Action::Ignored);
            for _ in 0..7 {
                assert!(matches!(
                    scaler.cut_in_tick(3, true, false),
                    Action::Cooling { .. } | Action::Ended
                ));
            }
            assert_eq!(resolution::status().reports, (ENTRIES as u8) - 1);
            assert_eq!(scaler.cut_in_tick(4, true, false), Action::Cooling { left: 6 });
        });
    }

    #[test]
    fn stale_token_does_not_pop_a_newer_report() {
        with_reports(Level::P1080, |scaler| {
            assert_eq!(scaler.cut_in_start(1, true), Action::Started);
            resolution::clear_reports();
            let kept = resolution::push_report(Level::P480);
            assert!(matches!(kept, PushResult::Stored(_)));
            for _ in 0..6 {
                assert!(matches!(
                    scaler.cut_in_tick(1, true, false),
                    Action::Cooling { .. }
                ));
            }
            assert_eq!(scaler.cut_in_tick(1, true, false), Action::Ended);
            assert_eq!(resolution::status().reports, 1);
            assert_eq!(resolution::status().effective, Level::P480);
            assert!(resolution::pop_report(match kept {
                PushResult::Stored(token) => token,
                other => panic!("expected stored, got {other:?}"),
            }));
        });
    }

    #[test]
    fn dynamic_off_and_full_multiset_do_not_stick_active() {
        with_reports(Level::P1080, |scaler| {
            resolution::update_settings(Level::P1080, false);
            assert_eq!(scaler.cut_in_start(0, true), Action::NotStored);
            assert_eq!(scaler.cut_in_tick(0, true, false), Action::Idle);
            assert_eq!(resolution::status().reports, 0);
            resolution::update_settings(Level::P1080, true);
            let mut held = Vec::new();
            for _ in 0..8 {
                held.push(match resolution::push_report(Level::P900) {
                    PushResult::Stored(token) => token,
                    other => panic!("expected stored, got {other:?}"),
                });
            }
            assert_eq!(scaler.cut_in_start(2, true), Action::NotStored);
            assert_eq!(resolution::status().reports, 8);
            for token in held {
                assert!(resolution::pop_report(token));
            }
        });
    }

    #[test]
    fn sephiroth_tracks_only_a_full_special_n_shoot() {
        with_reports(Level::P1080, |scaler| {
            assert_eq!(
                scaler.sephiroth_tick(2, true, EDGE_SPECIAL_N_SHOOT, 2),
                Action::Idle
            );
            assert_eq!(
                scaler.sephiroth_tick(2, true, EDGE_SPECIAL_N_SHOOT, 3),
                Action::Started
            );
            assert_eq!(
                scaler.sephiroth_tick(2, true, EDGE_SPECIAL_N_SHOOT, 3),
                Action::Held
            );
            assert_eq!(resolution::status().reports, 1);
            assert_eq!(resolution::status().effective, Level::P720);
            let (cut, edge) = scaler.on_main(2, true, EDGE_KIND, false, 0, 3);
            assert_eq!(cut, Action::Idle);
            assert_eq!(edge, Action::Ended);
            assert_eq!(resolution::status().reports, 0);
            let (_cut, ignored) = scaler.on_main(2, true, 0x1, false, EDGE_SPECIAL_N_SHOOT, 9);
            assert_eq!(ignored, Action::Ignored);
            assert_eq!(resolution::status().reports, 0);
        });
    }

    #[test]
    fn cut_in_and_sephiroth_keep_separate_tokens() {
        with_reports(Level::P720, |scaler| {
            assert_eq!(scaler.cut_in_start(5, true), Action::Started);
            assert_eq!(
                scaler.sephiroth_tick(5, true, EDGE_SPECIAL_N_SHOOT, 4),
                Action::Started
            );
            assert_eq!(resolution::status().reports, 2);
            assert_eq!(resolution::status().effective, Level::P576);
            assert_eq!(
                scaler.sephiroth_tick(5, true, EDGE_SPECIAL_N_SHOOT - 1, 4),
                Action::Ended
            );
            assert_eq!(resolution::status().reports, 1);
            assert_eq!(resolution::status().effective, Level::P576);
            scaler.clear();
            assert_eq!(resolution::status().reports, 0);
        });
    }
}
