//! Read-only record of one `common` module load, plus the host-side Main guard.
//!
//! The Switch observer copies bytes into this struct. This module does not
//! install a hook. `call_line_status` at export `0x9ea0` is an identity check
//! only: its fourth word moves `x8` into `x20`, so it is not the logical Main.
//! Smashline's Main site is `common+0x3250`.

use crate::diagnostics::Code;

pub const WORD_COUNT: usize = 16;
pub const EVENT_CAP: usize = 2 + WORD_COUNT;
/// `lua2cpp::L2CFighterBase::call_line_status` offset inside `common`.
pub const EXPORT_OFFSET: u64 = 0x9ea0;
/// Smashline logical Main site inside `common`. Not the export above.
pub const SITE_OFFSET: u64 = 0x3250;
/// skyline-smash bed642e `L2CAgent`: battle object, then module accessor.
pub const AGENT_BATTLE_OBJECT: usize = 0x38;
pub const AGENT_MODULE_ACCESSOR: usize = 0x40;
/// `BattleObject` id word. Category is the top nibble, matching
/// `utility::get_category` and `BATTLE_OBJECT_CATEGORY_FIGHTER`.
pub const OBJECT_ID: usize = 0x8;
pub const OBJECT_KIND: usize = 0xc;
/// `BattleObject::module_accessor`. Must equal `L2CAgent+0x40` before any SDK call.
pub const OBJECT_MODULE_ACCESSOR: usize = 0x20;
pub const FIGHTER_CATEGORY: u32 = 0;
/// First eight words observed at `common+0x3250` on the 13.0.5 probe.
pub const SITE_GUARD: [u32; 8] = [
    0xd101_43ff,
    0xf900_13f5,
    0xa903_4ff4,
    0xa904_7bfd,
    0x9101_03fd,
    0xaa00_03f3,
    0xaa01_03e0,
    0xaa01_03f4,
];
/// First eight words observed at the `call_line_status` export.
pub const EXPORT_GUARD: [u32; 8] = [
    0xa9be_4ff4,
    0xa901_7bfd,
    0x9100_43fd,
    0xaa08_03f4,
    0xd000_4008,
    0xf942_bd08,
    0xb940_0d08,
    0xaa00_03f3,
];
/// Export of `lua2cpp::L2CFighterBase::call_line_status`. Lookup only.
pub const CALL_LINE_SYMBOL: &[u8] = b"_ZN7lua2cpp14L2CFighterBase16call_line_statusEv\0";
/// `nn::ro::LoadModuleInternal(Module*, void const*, void*, size_t, int, bool)`.
pub const LOAD_MODULE_INTERNAL: &[u8] = b"_ZN2nn2ro18LoadModuleInternalEPNS0_6ModuleEPKvPvmib\0";
/// Installed nnsdk 0.6.0 binding. The probe hooks this symbol, not an internal.
pub const UNLOAD_MODULE: &[u8] = b"_ZN2nn2ro12UnloadModuleEPNS0_6ModuleE\0";

const FLAG_LOOKUP: u64 = 1;
const FLAG_SYMBOL_WORDS: u64 = 2;
const FLAG_SITE_WORDS: u64 = 4;
const FLAG_OFFSET_MATCH: u64 = 8;
const FLAG_UNLOADED: u64 = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObserveState {
    Idle,
    Observed,
    LookupMiss,
    Unmapped,
    Unloaded,
}

impl ObserveState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Observed => "observed",
            Self::LookupMiss => "lookup_miss",
            Self::Unmapped => "unmapped",
            Self::Unloaded => "unloaded",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observation {
    pub generation: u32,
    pub module: u64,
    pub base: u64,
    pub symbol: u64,
    pub offset: u64,
    pub symbol_words: [u32; WORD_COUNT],
    pub site_words: [u32; WORD_COUNT],
    pub lookup_ok: bool,
    pub symbol_readable: bool,
    pub site_readable: bool,
    pub offset_matches_cross_check: bool,
    pub state: ObserveState,
}

impl Observation {
    pub const fn idle() -> Self {
        Self {
            generation: 0,
            module: 0,
            base: 0,
            symbol: 0,
            offset: 0,
            symbol_words: [0; WORD_COUNT],
            site_words: [0; WORD_COUNT],
            lookup_ok: false,
            symbol_readable: false,
            site_readable: false,
            offset_matches_cross_check: false,
            state: ObserveState::Idle,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProbeEvent {
    pub code: Code,
    pub a: u64,
    pub b: u64,
}

pub struct LoadView<'a> {
    pub module: u64,
    pub result: u32,
    pub name: &'a [u8],
    pub base: Option<u64>,
    pub symbol: Option<u64>,
    pub symbol_words: Option<&'a [u32]>,
    pub site_words: Option<&'a [u32]>,
}

/// Exact `common` name. Seven bytes, NUL included. No UTF-8 conversion.
pub fn name_is_common(name: &[u8]) -> bool {
    name.len() >= 7 && name[..7] == *b"common\0"
}

pub fn note_load(current: Observation, view: LoadView<'_>) -> Observation {
    if view.result != 0 || !name_is_common(view.name) {
        return current;
    }
    let mut next = Observation {
        generation: current.generation.wrapping_add(1),
        module: view.module,
        ..Observation::idle()
    };
    let Some(base) = view.base.filter(|base| *base != 0) else {
        next.state = ObserveState::Unmapped;
        return next;
    };
    next.base = base;
    if let Some(symbol) = view.symbol.filter(|symbol| *symbol != 0) {
        next.symbol = symbol;
        next.lookup_ok = true;
        if symbol >= base {
            next.offset = symbol - base;
            next.offset_matches_cross_check = next.offset == EXPORT_OFFSET;
        }
    }
    if let Some(words) = exact_words(view.symbol_words) {
        next.symbol_words = words;
        next.symbol_readable = true;
    }
    if let Some(words) = exact_words(view.site_words) {
        next.site_words = words;
        next.site_readable = true;
    }
    next.state = if !next.lookup_ok {
        ObserveState::LookupMiss
    } else if !next.symbol_readable || !next.site_readable {
        ObserveState::Unmapped
    } else {
        ObserveState::Observed
    };
    next
}

pub fn note_unload(current: Observation, module: u64, result: u32) -> Observation {
    if result != 0 || module == 0 || module != current.module {
        return current;
    }
    Observation {
        generation: current.generation.wrapping_add(1),
        state: ObserveState::Unloaded,
        ..Observation::idle()
    }
}

pub fn events(obs: &Observation) -> (usize, [ProbeEvent; EVENT_CAP]) {
    let blank = ProbeEvent {
        code: Code::FighterModule,
        a: 0,
        b: 0,
    };
    let mut items = [blank; EVENT_CAP];
    if obs.state == ObserveState::Idle {
        return (0, items);
    }
    let mut len = 0;
    items[len] = ProbeEvent {
        code: Code::FighterModule,
        a: obs.base,
        b: obs.symbol,
    };
    len += 1;
    let mut flags = (obs.generation as u64) << 32;
    if obs.lookup_ok {
        flags |= FLAG_LOOKUP;
    }
    if obs.symbol_readable {
        flags |= FLAG_SYMBOL_WORDS;
    }
    if obs.site_readable {
        flags |= FLAG_SITE_WORDS;
    }
    if obs.offset_matches_cross_check {
        flags |= FLAG_OFFSET_MATCH;
    }
    if obs.state == ObserveState::Unloaded {
        flags |= FLAG_UNLOADED;
    }
    items[len] = ProbeEvent {
        code: Code::FighterSymbol,
        a: obs.offset,
        b: flags,
    };
    len += 1;
    if obs.state != ObserveState::Unloaded {
        len = push_words(&mut items, len, 0, obs.symbol_readable, &obs.symbol_words);
        len = push_words(&mut items, len, 1, obs.site_readable, &obs.site_words);
    }
    (len, items)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HookView {
    pub callback_armed: bool,
    pub cut_in_installed: bool,
    pub count_only: bool,
}

impl HookView {
    pub const fn idle() -> Self {
        Self {
            callback_armed: false,
            cut_in_installed: false,
            count_only: false,
        }
    }
}

pub fn status_text(build_id: &str, obs: &Observation, view: HookView) -> String {
    let cross = if obs.offset_matches_cross_check {
        "match"
    } else {
        "differ"
    };
    let site_hook = if view.callback_armed {
        "installed"
    } else {
        "not_installed"
    };
    let count_only = if view.count_only { "true" } else { "false" };
    let cut_in = if view.cut_in_installed {
        "true"
    } else {
        "false"
    };
    let armed = if view.callback_armed { "true" } else { "false" };
    format!(
        "build={build_id}\nstate={state}\ngeneration={generation}\nmodule={module:#x}\nbase={base:#x}\nsymbol={symbol:#x}\noffset={offset:#x}\ncross_check={cross}\nlookup_ok={lookup}\nsymbol_readable={symbol_readable}\nsite_readable={site_readable}\ncallback_armed={armed}\ncut_in_installed={cut_in}\ncount_only={count_only}\ncall_line_hook=not_installed\nsite_3250_hook={site_hook}\nsymbol_words={symbol_words}\nsite_words={site_words}\n",
        state = obs.state.label(),
        generation = obs.generation,
        module = obs.module,
        base = obs.base,
        symbol = obs.symbol,
        offset = obs.offset,
        lookup = obs.lookup_ok,
        symbol_readable = obs.symbol_readable,
        site_readable = obs.site_readable,
        symbol_words = hex_words(&obs.symbol_words),
        site_words = hex_words(&obs.site_words),
    )
}

fn exact_words(words: Option<&[u32]>) -> Option<[u32; WORD_COUNT]> {
    let words = words?;
    if words.len() != WORD_COUNT {
        return None;
    }
    let mut out = [0; WORD_COUNT];
    out.copy_from_slice(words);
    Some(out)
}

fn push_words(
    items: &mut [ProbeEvent; EVENT_CAP],
    mut len: usize,
    site: u64,
    readable: bool,
    words: &[u32; WORD_COUNT],
) -> usize {
    if !readable {
        return len;
    }
    let mut index = 0;
    while index < WORD_COUNT {
        let pair = index as u64 / 2;
        items[len] = ProbeEvent {
            code: Code::FighterWords,
            a: (site << 32) | pair,
            b: u64::from(words[index]) | (u64::from(words[index + 1]) << 32),
        };
        len += 1;
        index += 2;
    }
    len
}

fn hex_words(words: &[u32; WORD_COUNT]) -> String {
    let mut line = String::new();
    for (index, word) in words.iter().enumerate() {
        if index != 0 {
            line.push(' ');
        }
        line.push_str(&format!("{word:08x}"));
    }
    line
}

pub const fn category_of(object_id: u32) -> u32 {
    object_id >> 28
}

pub const fn site_already_hooked(word: u32) -> bool {
    let opcode = word & 0xfc00_0000;
    opcode == 0x1400_0000 || opcode == 0x9400_0000 || word == 0x5800_0051
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArmPlan {
    Install,
    Reject(&'static str),
}

pub fn arm_plan(export_offset: u64, export_head: &[u32], site_head: &[u32]) -> ArmPlan {
    if export_head.len() < 8 || site_head.len() < 8 {
        return ArmPlan::Reject("short");
    }
    if site_already_hooked(site_head[0]) {
        return ArmPlan::Reject("already_hooked");
    }
    if export_offset != EXPORT_OFFSET {
        return ArmPlan::Reject("export_offset");
    }
    if export_head[..8] != EXPORT_GUARD {
        return ArmPlan::Reject("export_words");
    }
    if site_head[..8] != SITE_GUARD {
        return ArmPlan::Reject("site_words");
    }
    ArmPlan::Install
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntryTick {
    pub count: u32,
    pub kind: i32,
    pub first_status: i32,
    pub have_status: bool,
    pub slow: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickLog {
    pub entries: [EntryTick; 8],
    pub rejects: u32,
    pub last_reject: u8,
    pub ticks: u32,
    pub present: u64,
    pub count_only: bool,
    pub main_armed: bool,
    pub cut_in_installed: bool,
    pub generation: u32,
}

impl TickLog {
    pub const fn idle() -> Self {
        Self {
            entries: [EntryTick {
                count: 0,
                kind: 0,
                first_status: 0,
                have_status: false,
                slow: 0,
            }; 8],
            rejects: 0,
            last_reject: 0,
            ticks: 0,
            present: 0,
            count_only: false,
            main_armed: false,
            cut_in_installed: false,
            generation: 0,
        }
    }

    pub fn reject(&mut self, reason: u8, present: u64) {
        self.rejects = self.rejects.saturating_add(1);
        self.last_reject = reason;
        self.present = present;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn note(&mut self, entry: usize, kind: i32, status: i32, slow: bool, present: u64) {
        let Some(slot) = self.entries.get_mut(entry) else {
            self.reject(7, present);
            return;
        };
        slot.count = slot.count.saturating_add(1);
        if slot.count == 1 {
            slot.kind = kind;
        }
        if !slot.have_status {
            slot.first_status = status;
            slot.have_status = true;
        }
        if slow {
            slot.slow = slot.slow.saturating_add(1);
        }
        self.ticks = self.ticks.saturating_add(1);
        self.present = present;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn text(&self, build_id: &str) -> String {
        let mode = if self.count_only {
            "count-only"
        } else {
            "normal"
        };
        let triggers = if self.main_armed && !self.count_only {
            "available"
        } else {
            "unavailable"
        };
        let mut text = format!(
            "build={build_id}\nmode={mode}\nmain_armed={armed}\ncut_in_installed={cut_in}\ntriggers={triggers}\ntracked_presents={present}\nticks={ticks}\nread_rejects={rejects}\nlast_reject={last}\n",
            armed = flag(self.main_armed),
            cut_in = flag(self.cut_in_installed),
            present = self.present,
            ticks = self.ticks,
            rejects = self.rejects,
            last = self.last_reject,
        );
        for (index, entry) in self.entries.iter().enumerate() {
            let status = if entry.have_status {
                entry.first_status.to_string()
            } else {
                "none".to_owned()
            };
            let kind = if entry.count == 0 {
                "none".to_owned()
            } else {
                entry.kind.to_string()
            };
            text.push_str(&format!(
                "entry{index}_count={count}\nentry{index}_kind={kind}\nentry{index}_first_status={status}\nentry{index}_slow={slow}\n",
                count = entry.count,
                slow = entry.slow,
            ));
        }
        text
    }
}

fn flag(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(fill: u32) -> [u32; WORD_COUNT] {
        let mut out = [0; WORD_COUNT];
        for (index, word) in out.iter_mut().enumerate() {
            *word = fill.wrapping_add(index as u32);
        }
        out
    }

    #[test]
    fn common_name_is_bounded_and_exact() {
        assert!(name_is_common(b"common\0"));
        assert!(name_is_common(b"common\0extra"));
        assert!(!name_is_common(b"common"));
        assert!(!name_is_common(b"common2\0"));
        assert!(!name_is_common(b"Common\0"));
        assert!(!name_is_common(&[0xff, 0xfe, 0xfd, 0xfc, 0xfb, 0xfa, 0]));
        assert!(CALL_LINE_SYMBOL.ends_with(&[0]));
        assert!(LOAD_MODULE_INTERNAL.ends_with(&[0]));
        assert!(UNLOAD_MODULE.ends_with(&[0]));
        assert_eq!(Code::FighterModule as u16, 75);
        assert_eq!(Code::FighterSymbol as u16, 76);
        assert_eq!(Code::FighterWords as u16, 77);
        assert_eq!(Code::ResolutionReport as u16, 78);
        assert_eq!(Code::FighterTransition as u16, 82);
        assert_eq!(Code::CropObserve as u16, 83);
    }

    #[test]
    fn failed_load_and_other_names_do_not_record() {
        let current = Observation::idle();
        let same = note_load(
            current,
            LoadView {
                module: 9,
                result: 1,
                name: b"common\0",
                base: Some(0x1000),
                symbol: Some(0x4250),
                symbol_words: None,
                site_words: None,
            },
        );
        assert_eq!(same, current);
        let other = note_load(
            current,
            LoadView {
                module: 9,
                result: 0,
                name: b"fighter\0",
                base: Some(0x1000),
                symbol: Some(0x4250),
                symbol_words: None,
                site_words: None,
            },
        );
        assert_eq!(other, current);
    }

    #[test]
    fn observed_load_keeps_both_word_windows_without_arming() {
        let symbol_words = words(0x10);
        let site_words = words(0x20);
        let next = note_load(
            Observation::idle(),
            LoadView {
                module: 0x50,
                result: 0,
                name: b"common\0",
                base: Some(0x1000),
                symbol: Some(0xaea0),
                symbol_words: Some(&symbol_words),
                site_words: Some(&site_words),
            },
        );
        assert_eq!(next.state, ObserveState::Observed);
        assert_eq!(next.offset, EXPORT_OFFSET);
        assert!(next.offset_matches_cross_check);
        assert_ne!(next.offset, SITE_OFFSET);
        assert_eq!(next.symbol_words, symbol_words);
        assert_eq!(next.site_words, site_words);
        let text = status_text("test", &next, HookView::idle());
        assert!(text.contains("callback_armed=false\n"));
        assert!(text.contains("cut_in_installed=false\n"));
        assert!(text.contains("count_only=false\n"));
        assert!(text.contains("call_line_hook=not_installed\n"));
        assert!(text.contains("site_3250_hook=not_installed\n"));
        assert!(text.contains("cross_check=match\n"));
        let (len, events) = events(&next);
        assert_eq!(len, EVENT_CAP);
        assert_eq!(events[0].code, Code::FighterModule);
        assert_eq!(events[0].a, 0x1000);
        assert_eq!(events[0].b, 0xaea0);
        assert_eq!(events[1].a, EXPORT_OFFSET);
        assert_eq!(
            events[1].b & 0xffff,
            FLAG_LOOKUP | FLAG_SYMBOL_WORDS | FLAG_SITE_WORDS | FLAG_OFFSET_MATCH
        );
        assert_eq!(events[2].code, Code::FighterWords);
        assert_eq!(events[2].a, 0);
        assert_eq!(
            events[2].b,
            u64::from(symbol_words[0]) | (u64::from(symbol_words[1]) << 32)
        );
        assert_eq!(events[10].a, 1 << 32);
    }

    #[test]
    fn lookup_miss_unmapped_base_and_short_words_stay_disarmed() {
        let site = words(1);
        let miss = note_load(
            Observation::idle(),
            LoadView {
                module: 4,
                result: 0,
                name: b"common\0",
                base: Some(0x2000),
                symbol: None,
                symbol_words: None,
                site_words: Some(&site),
            },
        );
        assert_eq!(miss.state, ObserveState::LookupMiss);
        assert!(!miss.lookup_ok);
        assert!(miss.site_readable);
        assert!(!miss.offset_matches_cross_check);
        assert_eq!(miss.site_words, site);
        let short = [1u32, 2];
        let partial = note_load(
            Observation::idle(),
            LoadView {
                module: 4,
                result: 0,
                name: b"common\0",
                base: Some(0x2000),
                symbol: Some(0x2100),
                symbol_words: Some(&short),
                site_words: Some(&site),
            },
        );
        assert_eq!(partial.state, ObserveState::Unmapped);
        assert!(!partial.symbol_readable);
        assert_eq!(partial.offset, 0x100);
        let unmapped = note_load(
            Observation::idle(),
            LoadView {
                module: 4,
                result: 0,
                name: b"common\0",
                base: None,
                symbol: Some(0x2100),
                symbol_words: Some(&site),
                site_words: Some(&site),
            },
        );
        assert_eq!(unmapped.state, ObserveState::Unmapped);
        assert_eq!(unmapped.base, 0);
        assert_eq!(unmapped.module, 4);
    }

    #[test]
    fn unload_clears_only_the_tracked_module() {
        let symbol_words = words(3);
        let site_words = words(4);
        let loaded = note_load(
            Observation::idle(),
            LoadView {
                module: 8,
                result: 0,
                name: b"common\0",
                base: Some(0x3000),
                symbol: Some(0x3100),
                symbol_words: Some(&symbol_words),
                site_words: Some(&site_words),
            },
        );
        assert_eq!(note_unload(loaded, 9, 0), loaded);
        assert_eq!(note_unload(loaded, 8, 1), loaded);
        let cleared = note_unload(loaded, 8, 0);
        assert_eq!(cleared.state, ObserveState::Unloaded);
        assert_eq!(cleared.module, 0);
        assert_eq!(cleared.base, 0);
        assert_eq!(cleared.symbol_words, [0; WORD_COUNT]);
        assert_eq!(cleared.generation, loaded.generation + 1);
        let (len, _) = events(&cleared);
        assert_eq!(len, 2);
        let again = note_load(
            cleared,
            LoadView {
                module: 12,
                result: 0,
                name: b"common\0",
                base: Some(0x4000),
                symbol: Some(0xdea0),
                symbol_words: Some(&symbol_words),
                site_words: Some(&site_words),
            },
        );
        assert_eq!(again.state, ObserveState::Observed);
        assert_eq!(again.generation, cleared.generation + 1);
        assert!(again.offset_matches_cross_check);
    }

    #[test]
    fn main_guard_accepts_only_the_observed_pair() {
        assert_eq!(
            arm_plan(EXPORT_OFFSET, &EXPORT_GUARD, &SITE_GUARD),
            ArmPlan::Install
        );
        assert_eq!(
            arm_plan(SITE_OFFSET, &EXPORT_GUARD, &SITE_GUARD),
            ArmPlan::Reject("export_offset")
        );
        assert_eq!(
            arm_plan(EXPORT_OFFSET, &SITE_GUARD, &SITE_GUARD),
            ArmPlan::Reject("export_words")
        );
        let mut hooked = SITE_GUARD;
        hooked[0] = 0x1400_0001;
        assert_eq!(
            arm_plan(EXPORT_OFFSET, &EXPORT_GUARD, &hooked),
            ArmPlan::Reject("already_hooked")
        );
        assert!(site_already_hooked(0x5800_0051));
        assert!(!site_already_hooked(SITE_GUARD[0]));
        assert_ne!(SITE_GUARD, EXPORT_GUARD);
        assert_eq!(EXPORT_GUARD[3], 0xaa08_03f4);
        assert_eq!((EXPORT_GUARD[3] >> 16) & 0x1f, 8);
        assert_eq!(category_of(0), FIGHTER_CATEGORY);
        assert_eq!(category_of(0x1000_0000), 1);
        assert_ne!(category_of(0x1000_0000), FIGHTER_CATEGORY);
        assert_eq!(AGENT_BATTLE_OBJECT, 0x38);
        assert_eq!(AGENT_MODULE_ACCESSOR, 0x40);
        assert_eq!(OBJECT_MODULE_ACCESSOR, 0x20);
        assert_ne!(OBJECT_MODULE_ACCESSOR, AGENT_MODULE_ACCESSOR);
        assert_eq!(OBJECT_KIND, 0xc);
    }

    #[test]
    fn count_log_keeps_per_entry_aggregates() {
        let mut log = TickLog::idle();
        log.count_only = true;
        log.note(0, 0x21, 12, false, 4);
        log.note(0, 0x99, 40, true, 5);
        log.note(1, 0x59, 7, false, 5);
        log.reject(5, 6);
        assert_eq!(log.entries[0].count, 2);
        assert_eq!(log.entries[0].kind, 0x21);
        assert_eq!(log.entries[0].first_status, 12);
        assert_eq!(log.entries[0].slow, 1);
        assert_eq!(log.entries[1].count, 1);
        assert_eq!(log.ticks, 3);
        assert_eq!(log.rejects, 1);
        assert_eq!(log.present, 6);
        let text = log.text("build");
        assert!(text.contains("mode=count-only\n"));
        assert!(text.contains("triggers=unavailable\n"));
        assert!(text.contains("tracked_presents=6\n"));
        assert!(text.contains("entry0_count=2\n"));
        assert!(text.contains("entry0_kind=33\n"));
        assert!(text.contains("entry0_first_status=12\n"));
        assert!(text.contains("entry1_kind=89\n"));
        assert!(text.contains("read_rejects=1\n"));
    }
}
