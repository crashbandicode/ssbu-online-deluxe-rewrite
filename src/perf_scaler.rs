//! Eight-entry cut-in tick policy from the quickplay fork, plus the 0.5.2
//! move-table and adaptive triggers.
//!
//! Tokens are the resolution report tokens. This module does not read or write
//! the game. The runtime calls it only after logical Main is armed and
//! count-only mode is off. The fork's second trigger, Sephiroth's full-charge
//! Gigaflare, was removed on 2026-10-17: the console measurement did not
//! justify it.

use crate::drs_triggers::MoveSet;
use crate::resolution::{self, Level, PushResult, ReportToken};
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU32, AtomicU64, AtomicU8, Ordering};

pub const ENTRIES: usize = 8;
/// Fork `CRITICAL_HIT_FINISH_COOLDOWN_FRAMES`.
pub const CUT_IN_COOLDOWN_FRAMES: i8 = 7;
/// 13.0.5 `FighterCutInStart`. The runtime installs it only after Main arms
/// and count-only mode is off.
pub const CUT_IN_OFFSET: usize = 0x06a_7100;
/// 13.0.5 entry word at [`CUT_IN_OFFSET`].
pub const CUT_IN_PROLOGUE: u32 = 0xd106_83ff;
/// skyline-smash bed642e `lua_const.rs`.
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

/// One report with no cooldown: the adaptive trigger.
struct ReportSlot {
    active: AtomicBool,
    token: AtomicU32,
}

/// One fighter Main tick as the runtime samples it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MainTick {
    pub entry_id: i32,
    pub entry_present: bool,
    pub kind: i32,
    pub slow: bool,
    pub status: i32,
}

impl MainTick {
    /// A present entry with no slow-motion flag; tests and the runtime fill the rest.
    pub const fn new(entry_id: i32, kind: i32, status: i32) -> Self {
        Self {
            entry_id,
            entry_present: true,
            kind,
            slow: false,
            status,
        }
    }
}

/// 0.5.2 experiment: one named move active per fighter entry.
struct MoveSlot {
    active: AtomicBool,
    token: AtomicU32,
    which: AtomicU8,
}

impl MoveSlot {
    const fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            token: AtomicU32::new(0),
            which: AtomicU8::new(u8::MAX),
        }
    }
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

impl ReportSlot {
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
    moves: [MoveSlot; ENTRIES],
    /// 0.5.2 experiment: the adaptive (frame-interval) report, one for the scene.
    adaptive: ReportSlot,
    move_pushes: AtomicU64,
    move_pops: AtomicU64,
    adaptive_pushes: AtomicU64,
    adaptive_pops: AtomicU64,
}

/// Status-file and Debug-view counters for the 0.5.2 triggers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TriggerCounters {
    pub move_pushes: u64,
    pub move_pops: u64,
    pub adaptive_pushes: u64,
    pub adaptive_pops: u64,
    pub adaptive_active: bool,
    /// Catalog index of the move currently holding a report, lowest entry first.
    pub move_active: Option<u8>,
}

impl Scaler {
    pub const fn new(base: Level) -> Self {
        Self {
            base: AtomicU8::new(base as u8),
            cut_in: [const { CutSlot::new() }; ENTRIES],
            moves: [const { MoveSlot::new() }; ENTRIES],
            adaptive: ReportSlot::new(),
            move_pushes: AtomicU64::new(0),
            move_pops: AtomicU64::new(0),
            adaptive_pushes: AtomicU64::new(0),
            adaptive_pops: AtomicU64::new(0),
        }
    }

    pub fn counters(&self) -> TriggerCounters {
        TriggerCounters {
            move_pushes: self.move_pushes.load(Ordering::Relaxed),
            move_pops: self.move_pops.load(Ordering::Relaxed),
            adaptive_pushes: self.adaptive_pushes.load(Ordering::Relaxed),
            adaptive_pops: self.adaptive_pops.load(Ordering::Relaxed),
            adaptive_active: self.adaptive.active.load(Ordering::Acquire),
            move_active: self.moves.iter().find_map(|slot| {
                slot.active
                    .load(Ordering::Acquire)
                    .then(|| slot.which.load(Ordering::Acquire))
            }),
        }
    }

    /// Per-move trigger for one Main tick. `set` is the enabled catalog mask;
    /// a matching (kind, status) pushes one report per entry and holds it
    /// until the status leaves every enabled move of that entry.
    pub fn move_tick(
        &self,
        entry_id: i32,
        entry_present: bool,
        kind: i32,
        status: i32,
        set: MoveSet,
    ) -> (Action, Option<u8>) {
        let Some(index) = entry_slot(entry_id, entry_present) else {
            return (Action::Ignored, None);
        };
        let slot = &self.moves[index];
        match set.matching(kind, status) {
            Some(which) => {
                if slot
                    .active
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_err()
                {
                    slot.which.store(which, Ordering::Release);
                    return (Action::Held, Some(which));
                }
                match resolution::push_report(intensive_level(self.base_level())) {
                    PushResult::Stored(token) => {
                        slot.token.store(pack(token), Ordering::Release);
                        slot.which.store(which, Ordering::Release);
                        self.move_pushes.fetch_add(1, Ordering::Relaxed);
                        (Action::Started, Some(which))
                    }
                    PushResult::Ignored | PushResult::Full => {
                        slot.active.store(false, Ordering::Release);
                        (Action::NotStored, Some(which))
                    }
                }
            }
            None => {
                if !slot.active.load(Ordering::Acquire) {
                    return (Action::Idle, None);
                }
                let which = slot.which.load(Ordering::Acquire);
                if slot.token.load(Ordering::Acquire) == 0 {
                    return (Action::Held, Some(which));
                }
                pop_packed(&slot.token);
                slot.active.store(false, Ordering::Release);
                slot.which.store(u8::MAX, Ordering::Release);
                self.move_pops.fetch_add(1, Ordering::Relaxed);
                (Action::Ended, Some(which))
            }
        }
    }

    /// Adaptive trigger: one report for the whole scene, pushed by the present
    /// hook when frame intervals stay over budget.
    pub fn adaptive_push(&self) -> Action {
        if self
            .adaptive
            .active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Action::Already;
        }
        match resolution::push_report(intensive_level(self.base_level())) {
            PushResult::Stored(token) => {
                self.adaptive.token.store(pack(token), Ordering::Release);
                self.adaptive_pushes.fetch_add(1, Ordering::Relaxed);
                Action::Started
            }
            PushResult::Ignored | PushResult::Full => {
                self.adaptive.active.store(false, Ordering::Release);
                Action::NotStored
            }
        }
    }

    pub fn adaptive_pop(&self) -> Action {
        if !self.adaptive.active.load(Ordering::Acquire) {
            return Action::Idle;
        }
        if self.adaptive.token.load(Ordering::Acquire) == 0 {
            return Action::Held;
        }
        pop_packed(&self.adaptive.token);
        self.adaptive.active.store(false, Ordering::Release);
        self.adaptive_pops.fetch_add(1, Ordering::Relaxed);
        Action::Ended
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
            slot.cooldown
                .store(CUT_IN_COOLDOWN_FRAMES, Ordering::Release);
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

    /// The cut-in cooldown for one Main tick, plus the 0.5.2 move table. The
    /// cooldown runs for every present entry. `moves` is the enabled catalog
    /// mask (none = off).
    pub fn on_main(&self, tick: MainTick, moves: MoveSet) -> (Action, (Action, Option<u8>)) {
        let MainTick {
            entry_id,
            entry_present,
            kind,
            slow,
            status,
        } = tick;
        let cut = self.cut_in_tick(entry_id, entry_present, slow);
        let moved = if moves.is_empty() && !self.moves_hold(entry_id, entry_present) {
            (Action::Ignored, None)
        } else {
            self.move_tick(entry_id, entry_present, kind, status, moves)
        };
        (cut, moved)
    }

    fn moves_hold(&self, entry_id: i32, entry_present: bool) -> bool {
        entry_slot(entry_id, entry_present)
            .is_some_and(|index| self.moves[index].active.load(Ordering::Acquire))
    }

    pub fn clear(&self) {
        for slot in &self.cut_in {
            slot.active.store(false, Ordering::Release);
            pop_packed(&slot.token);
            slot.cooldown
                .store(CUT_IN_COOLDOWN_FRAMES, Ordering::Release);
        }
        for slot in &self.moves {
            slot.active.store(false, Ordering::Release);
            slot.which.store(u8::MAX, Ordering::Release);
            pop_packed(&slot.token);
        }
        self.adaptive.active.store(false, Ordering::Release);
        pop_packed(&self.adaptive.token);
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
        // Production paths only: a scene reset drops live reports, a retarget
        // sets the base level and the dynamic flag.
        resolution::reset_scene();
        resolution::retarget(Level::P1080, true);
        let scaler = Scaler::new(base);
        test(&scaler);
        scaler.clear();
        resolution::reset_scene();
        resolution::retarget(Level::P1080, false);
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
            assert_eq!(
                scaler.cut_in_tick(4, true, false),
                Action::Cooling { left: 6 }
            );
        });
    }

    #[test]
    fn stale_token_does_not_pop_a_newer_report() {
        with_reports(Level::P1080, |scaler| {
            assert_eq!(scaler.cut_in_start(1, true), Action::Started);
            resolution::reset_scene();
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
            resolution::retarget(Level::P1080, false);
            assert_eq!(scaler.cut_in_start(0, true), Action::NotStored);
            assert_eq!(scaler.cut_in_tick(0, true, false), Action::Idle);
            assert_eq!(resolution::status().reports, 0);
            resolution::retarget(Level::P1080, true);
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
    fn main_tick_runs_only_the_cut_in_cooldown_and_the_move_table() {
        with_reports(Level::P1080, |scaler| {
            // Sephiroth (kind 0x59) in the old Gigaflare shoot status is an
            // ordinary tick since 2026-10-17: no report, nothing to end.
            let (cut, moved) = scaler.on_main(MainTick::new(2, 0x59, 0x1e5), MoveSet::NONE);
            assert_eq!(cut, Action::Idle);
            assert_eq!(moved, (Action::Ignored, None));
            assert_eq!(resolution::status().reports, 0);
            assert_eq!(scaler.cut_in_start(2, true), Action::Started);
            let (cooling, moved) = scaler.on_main(MainTick::new(2, 0x59, 0x1e5), MoveSet::NONE);
            assert_eq!(cooling, Action::Cooling { left: 6 });
            assert_eq!(moved, (Action::Ignored, None));
            assert_eq!(resolution::status().reports, 1);
            scaler.clear();
            assert_eq!(resolution::status().reports, 0);
        });
    }

    #[test]
    fn move_table_pushes_while_the_status_matches_and_pops_when_it_leaves() {
        use crate::drs_triggers::{KIND_EFLAME, KIND_KIRBY, SPECIAL_N, SPECIAL_S};
        with_reports(Level::P1080, |scaler| {
            let set = MoveSet::parse("pyra_side_b,kirby_hammer").unwrap();
            // Off: nothing happens even on a matching status.
            let (_, off) = scaler.on_main(MainTick::new(0, KIND_EFLAME, SPECIAL_S), MoveSet::NONE);
            assert_eq!(off, (Action::Ignored, None));
            assert_eq!(resolution::status().reports, 0);
            // Pyra side special on entry 0, Kirby hammer on entry 1.
            assert_eq!(
                scaler.move_tick(0, true, KIND_EFLAME, SPECIAL_S, set),
                (Action::Started, Some(1))
            );
            assert_eq!(
                scaler.move_tick(0, true, KIND_EFLAME, 0x1EA, set),
                (Action::Held, Some(1))
            );
            assert_eq!(
                scaler.move_tick(1, true, KIND_KIRBY, 0x1F3, set),
                (Action::Started, Some(8))
            );
            assert_eq!(resolution::status().reports, 2);
            assert_eq!(resolution::status().effective, Level::P720);
            assert_eq!(scaler.counters().move_pushes, 2);
            assert_eq!(scaler.counters().move_active, Some(1));
            // Leaving the status pops; an unrelated kind never pushes.
            assert_eq!(
                scaler.move_tick(0, true, KIND_EFLAME, SPECIAL_N, set),
                (Action::Ended, Some(1))
            );
            assert_eq!(
                scaler.move_tick(0, true, KIND_EFLAME, SPECIAL_N, set),
                (Action::Idle, None)
            );
            assert_eq!(
                scaler.move_tick(2, true, 0x1, SPECIAL_S, set),
                (Action::Idle, None)
            );
            assert_eq!(resolution::status().reports, 1);
            // The mask going to none while a report is held still pops it.
            let (_, released) = scaler.on_main(MainTick::new(1, KIND_KIRBY, 0x1F3), MoveSet::NONE);
            assert_eq!(released, (Action::Ended, Some(8)));
            assert_eq!(resolution::status().reports, 0);
            assert_eq!(scaler.counters().move_pops, 2);
            assert_eq!(
                scaler.move_tick(8, true, KIND_KIRBY, 0x1F3, set),
                (Action::Ignored, None)
            );
        });
    }

    #[test]
    fn adaptive_report_is_single_and_clears_with_the_scene() {
        with_reports(Level::P1080, |scaler| {
            assert_eq!(scaler.adaptive_pop(), Action::Idle);
            assert_eq!(scaler.adaptive_push(), Action::Started);
            assert_eq!(scaler.adaptive_push(), Action::Already);
            assert_eq!(resolution::status().reports, 1);
            assert_eq!(resolution::status().effective, Level::P720);
            assert!(scaler.counters().adaptive_active);
            assert_eq!(scaler.adaptive_pop(), Action::Ended);
            assert_eq!(scaler.adaptive_pop(), Action::Idle);
            assert_eq!(resolution::status().reports, 0);
            assert_eq!(scaler.counters().adaptive_pushes, 1);
            assert_eq!(scaler.counters().adaptive_pops, 1);
            assert_eq!(scaler.adaptive_push(), Action::Started);
            scaler.clear();
            assert!(!scaler.counters().adaptive_active);
            assert_eq!(resolution::status().reports, 0);
            resolution::retarget(Level::P1080, false);
            assert_eq!(scaler.adaptive_push(), Action::NotStored);
            assert!(!scaler.counters().adaptive_active);
        });
    }

    #[test]
    fn cut_in_and_adaptive_keep_separate_tokens() {
        with_reports(Level::P720, |scaler| {
            assert_eq!(scaler.cut_in_start(5, true), Action::Started);
            assert_eq!(scaler.adaptive_push(), Action::Started);
            assert_eq!(resolution::status().reports, 2);
            assert_eq!(resolution::status().effective, Level::P576);
            assert_eq!(scaler.adaptive_pop(), Action::Ended);
            assert_eq!(resolution::status().reports, 1);
            assert_eq!(resolution::status().effective, Level::P576);
            scaler.clear();
            assert_eq!(resolution::status().reports, 0);
        });
    }
}
