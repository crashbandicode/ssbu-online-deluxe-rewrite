//! Per-move and adaptive dynamic-resolution triggers (0.5.2 experiment).
//!
//! Pure policy, no game reads. `perf_scaler` feeds fighter kind and status
//! ticks to the move table; the present hook feeds frame intervals to the
//! adaptive state machine. Both are off unless `config.ini` enables them:
//! `drs_moves=` names moves from [`MOVES`] (or `all`), `drs_adaptive=true`
//! turns on the interval trigger with its own threshold and frame counts.
//!
//! Constants are skyline-smash bed642e `lua_const.rs` values for 13.0.5.

/// One named move: a fighter kind and the status kinds that count as the move.
#[derive(Clone, Copy, Debug)]
pub struct MoveTrigger {
    pub name: &'static str,
    pub kind: i32,
    pub statuses: &'static [i32],
}

pub const KIND_KIRBY: i32 = 0x6;
pub const KIND_NESS: i32 = 0xA;
pub const KIND_CAPTAIN: i32 = 0xB;
pub const KIND_LUCAS: i32 = 0x28;
pub const KIND_PALUTENA: i32 = 0x36;
pub const KIND_INKLING: i32 = 0x41;
pub const KIND_MASTER: i32 = 0x56;
pub const KIND_EFLAME: i32 = 0x5A;
pub const KIND_ELIGHT: i32 = 0x5B;

/// Common fighter statuses shared by every kind.
pub const SPECIAL_N: i32 = 0x1DC;
pub const SPECIAL_S: i32 = 0x1DD;
pub const SPECIAL_HI: i32 = 0x1DE;
pub const SPECIAL_LW: i32 = 0x1DF;

/// Catalog order is the bit order of [`MoveSet`]; append only.
pub const MOVES: &[MoveTrigger] = &[
    MoveTrigger {
        name: "falcon_punch",
        kind: KIND_CAPTAIN,
        statuses: &[SPECIAL_N, 0x1E1],
    },
    MoveTrigger {
        name: "pyra_side_b",
        kind: KIND_EFLAME,
        statuses: &[SPECIAL_S, 0x1EA],
    },
    MoveTrigger {
        name: "pyra_neutral_b",
        kind: KIND_EFLAME,
        statuses: &[SPECIAL_N, 0x1E7, 0x1E8, 0x1E9],
    },
    MoveTrigger {
        name: "pyra_up_b",
        kind: KIND_EFLAME,
        statuses: &[SPECIAL_HI, 0x1EB, 0x1EC, 0x1ED],
    },
    MoveTrigger {
        name: "mythra_specials",
        kind: KIND_ELIGHT,
        statuses: &[
            SPECIAL_N, 0x1E7, 0x1E8, SPECIAL_S, 0x1E9, 0x1EA, SPECIAL_HI, 0x1EB, 0x1EC, 0x1ED,
            0x1EE,
        ],
    },
    MoveTrigger {
        name: "palutena_explosive_flame",
        kind: KIND_PALUTENA,
        statuses: &[SPECIAL_S],
    },
    MoveTrigger {
        name: "ness_pk_thunder",
        kind: KIND_NESS,
        statuses: &[SPECIAL_HI, 0x1E4, 0x1E5, 0x1E6, 0x1E7, 0x1E8],
    },
    MoveTrigger {
        name: "lucas_pk_thunder",
        kind: KIND_LUCAS,
        statuses: &[SPECIAL_HI, 0x1E4, 0x1E5, 0x1E6, 0x1E7, 0x1E8],
    },
    MoveTrigger {
        name: "kirby_hammer",
        kind: KIND_KIRBY,
        statuses: &[
            SPECIAL_S, 0x1F3, 0x1F4, 0x1F5, 0x1F6, 0x1F7, 0x1F8, 0x1F9, 0x1FA, 0x1FB,
        ],
    },
    MoveTrigger {
        name: "byleth_specials",
        kind: KIND_MASTER,
        statuses: &[
            SPECIAL_N, 0x1E1, 0x1E2, 0x1E3, 0x1E4, 0x1E5, 0x1E6, SPECIAL_S, 0x1E7, 0x1E8, 0x1E9,
            SPECIAL_HI, 0x1EA, 0x1EB, 0x1EC, 0x1ED, SPECIAL_LW, 0x1EE, 0x1EF, 0x1F0, 0x1F1, 0x1F2,
        ],
    },
    MoveTrigger {
        name: "inkling_roller",
        kind: KIND_INKLING,
        statuses: &[
            SPECIAL_S, 0x1E3, 0x1E4, 0x1E5, 0x1E6, 0x1E7, 0x1E8, 0x1E9, 0x1EA,
        ],
    },
];

/// Enabled moves as a bit per catalog entry.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MoveSet(pub u32);

impl MoveSet {
    pub const NONE: Self = Self(0);

    pub const fn all() -> Self {
        Self((1u32 << MOVES.len()) - 1)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, index: usize) -> bool {
        index < MOVES.len() && self.0 & (1 << index) != 0
    }

    /// `none`, `all`, or comma-separated catalog names; blanks are ignored.
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        if text.is_empty() || text == "none" {
            return Ok(Self::NONE);
        }
        if text == "all" {
            return Ok(Self::all());
        }
        let mut set = 0u32;
        for name in text.split(',') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let Some(index) = MOVES.iter().position(|entry| entry.name == name) else {
                return Err(format!("unknown drs move '{name}'"));
            };
            set |= 1 << index;
        }
        Ok(Self(set))
    }

    pub fn serialize(self) -> String {
        if self.is_empty() {
            return "none".to_owned();
        }
        if self == Self::all() {
            return "all".to_owned();
        }
        MOVES
            .iter()
            .enumerate()
            .filter(|(index, _)| self.contains(*index))
            .map(|(_, entry)| entry.name)
            .collect::<Vec<_>>()
            .join(",")
    }

    /// The enabled catalog entry that owns this fighter kind and status.
    pub fn matching(self, kind: i32, status: i32) -> Option<u8> {
        MOVES.iter().enumerate().find_map(|(index, entry)| {
            (self.contains(index) && entry.kind == kind && entry.statuses.contains(&status))
                .then_some(index as u8)
        })
    }
}

/// Adaptive trigger settings. The threshold is stored in hundredths of a
/// millisecond so the INI round-trips exactly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdaptiveSettings {
    pub enabled: bool,
    pub threshold_hundredths_ms: u16,
    pub trigger_frames: u8,
    pub release_frames: u16,
}

impl Default for AdaptiveSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_hundredths_ms: 1750,
            trigger_frames: 6,
            release_frames: 90,
        }
    }
}

pub const THRESHOLD_MIN: u16 = 1000;
pub const THRESHOLD_MAX: u16 = 4000;
pub const TRIGGER_FRAMES_MAX: u8 = 60;
pub const RELEASE_FRAMES_MAX: u16 = 600;
/// Intervals at or above this are scene loads or hitches, not frame pacing;
/// they reset the counters without counting either way.
pub const HITCH_HUNDREDTHS_MS: u32 = 20_000;
/// 19.2 MHz system tick: 192 ticks per hundredth of a millisecond.
pub const TICKS_PER_HUNDREDTH_MS: u64 = 192;

/// `17.5`, `17`, `17.50` → 1750. Up to two decimals, 10.00–40.00 ms.
pub fn parse_threshold(text: &str) -> Result<u16, String> {
    let text = text.trim();
    let (whole, fraction) = match text.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (text, ""),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    // A trailing dot ("17.") is a typo, not a value.
    let dangling_dot = text.ends_with('.');
    if dangling_dot
        || !digits(whole)
        || (!fraction.is_empty() && !digits(fraction))
        || fraction.len() > 2
    {
        return Err("drs_adaptive_threshold_ms must be milliseconds with up to two decimals, 10.00 to 40.00".to_owned());
    }
    let whole: u32 = whole
        .parse()
        .map_err(|_| "drs_adaptive_threshold_ms is too large".to_owned())?;
    let fraction: u32 = if fraction.is_empty() {
        0
    } else {
        let padded = format!("{fraction:0<2}");
        padded.parse().unwrap_or(0)
    };
    let hundredths = whole * 100 + fraction;
    if !(u32::from(THRESHOLD_MIN)..=u32::from(THRESHOLD_MAX)).contains(&hundredths) {
        return Err("drs_adaptive_threshold_ms must be between 10.00 and 40.00".to_owned());
    }
    Ok(hundredths as u16)
}

pub fn format_threshold(hundredths: u16) -> String {
    let whole = hundredths / 100;
    let rest = hundredths % 100;
    if rest == 0 {
        format!("{whole}")
    } else if rest.is_multiple_of(10) {
        format!("{whole}.{}", rest / 10)
    } else {
        format!("{whole}.{rest:02}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdaptiveEvent {
    Push,
    Pop,
}

/// Counts consecutive over- and under-budget frames. `Push` after
/// `trigger_frames` late frames in a row, `Pop` after `release_frames` frames
/// in a row within budget. Disabling while active pops at once.
#[derive(Clone, Copy, Debug, Default)]
pub struct Adaptive {
    over: u16,
    under: u16,
    active: bool,
}

impl Adaptive {
    pub const fn new() -> Self {
        Self {
            over: 0,
            under: 0,
            active: false,
        }
    }

    pub const fn active(&self) -> bool {
        self.active
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn observe(
        &mut self,
        interval_hundredths_ms: u32,
        settings: &AdaptiveSettings,
    ) -> Option<AdaptiveEvent> {
        if !settings.enabled {
            let was_active = self.active;
            self.reset();
            return was_active.then_some(AdaptiveEvent::Pop);
        }
        if interval_hundredths_ms >= HITCH_HUNDREDTHS_MS {
            self.over = 0;
            self.under = 0;
            return None;
        }
        if interval_hundredths_ms > u32::from(settings.threshold_hundredths_ms) {
            self.under = 0;
            self.over = self.over.saturating_add(1);
            if !self.active && self.over >= u16::from(settings.trigger_frames.max(1)) {
                self.active = true;
                self.over = 0;
                return Some(AdaptiveEvent::Push);
            }
        } else {
            self.over = 0;
            self.under = self.under.saturating_add(1);
            if self.active && self.under >= settings.release_frames.max(1) {
                self.active = false;
                self.under = 0;
                return Some(AdaptiveEvent::Pop);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_fits_the_mask_and_has_unique_names() {
        assert!(MOVES.len() <= 32);
        for (i, a) in MOVES.iter().enumerate() {
            assert!(!a.statuses.is_empty(), "{}", a.name);
            for b in &MOVES[i + 1..] {
                assert_ne!(a.name, b.name);
            }
        }
    }

    #[test]
    fn move_set_parses_and_round_trips() {
        assert_eq!(MoveSet::parse("").unwrap(), MoveSet::NONE);
        assert_eq!(MoveSet::parse("none").unwrap(), MoveSet::NONE);
        assert_eq!(MoveSet::parse("all").unwrap(), MoveSet::all());
        let two = MoveSet::parse(" pyra_side_b , ness_pk_thunder ").unwrap();
        assert!(two.contains(1) && two.contains(6));
        assert_eq!(two.serialize(), "pyra_side_b,ness_pk_thunder");
        assert_eq!(MoveSet::parse(&two.serialize()).unwrap(), two);
        assert_eq!(MoveSet::all().serialize(), "all");
        assert_eq!(MoveSet::NONE.serialize(), "none");
        assert!(MoveSet::parse("pyra_side_b,nope").is_err());
        assert!(!MoveSet::NONE.contains(0));
        assert!(!MoveSet::all().contains(MOVES.len()));
    }

    #[test]
    fn matching_needs_both_kind_and_status_and_the_bit() {
        let pyra = MoveSet::parse("pyra_side_b").unwrap();
        assert_eq!(pyra.matching(KIND_EFLAME, SPECIAL_S), Some(1));
        assert_eq!(pyra.matching(KIND_EFLAME, 0x1EA), Some(1));
        assert_eq!(pyra.matching(KIND_EFLAME, SPECIAL_N), None);
        assert_eq!(pyra.matching(KIND_ELIGHT, SPECIAL_S), None);
        assert_eq!(MoveSet::all().matching(KIND_KIRBY, 0x1F3), Some(8));
        assert_eq!(MoveSet::all().matching(KIND_KIRBY, 0x100), None);
        assert_eq!(MoveSet::NONE.matching(KIND_CAPTAIN, SPECIAL_N), None);
    }

    #[test]
    fn threshold_parses_two_decimals_and_formats_back() {
        assert_eq!(parse_threshold("17.5").unwrap(), 1750);
        assert_eq!(parse_threshold("17").unwrap(), 1700);
        assert_eq!(parse_threshold("17.50").unwrap(), 1750);
        assert_eq!(parse_threshold("16.67").unwrap(), 1667);
        assert_eq!(format_threshold(1750), "17.5");
        assert_eq!(format_threshold(1700), "17");
        assert_eq!(format_threshold(1667), "16.67");
        for bad in ["", "9.99", "40.01", "17.555", "abc", "-17", "17.", ".5"] {
            assert!(parse_threshold(bad).is_err(), "accepted {bad:?}");
        }
        for value in [1000u16, 1667, 1750, 2000, 4000] {
            assert_eq!(parse_threshold(&format_threshold(value)).unwrap(), value);
        }
    }

    #[test]
    fn adaptive_pushes_after_trigger_frames_and_pops_after_release_frames() {
        let settings = AdaptiveSettings {
            enabled: true,
            threshold_hundredths_ms: 1750,
            trigger_frames: 3,
            release_frames: 4,
        };
        let mut a = Adaptive::new();
        assert_eq!(a.observe(1800, &settings), None);
        assert_eq!(a.observe(1667, &settings), None); // resets the run
        assert_eq!(a.observe(1800, &settings), None);
        assert_eq!(a.observe(1900, &settings), None);
        assert_eq!(a.observe(2500, &settings), Some(AdaptiveEvent::Push));
        assert!(a.active());
        assert_eq!(a.observe(2500, &settings), None);
        for _ in 0..3 {
            assert_eq!(a.observe(1667, &settings), None);
        }
        assert_eq!(a.observe(1800, &settings), None); // late frame breaks the release run
        for _ in 0..3 {
            assert_eq!(a.observe(1667, &settings), None);
        }
        assert_eq!(a.observe(1667, &settings), Some(AdaptiveEvent::Pop));
        assert!(!a.active());
    }

    #[test]
    fn hitches_do_not_count_and_disabling_pops() {
        let settings = AdaptiveSettings {
            enabled: true,
            trigger_frames: 2,
            ..AdaptiveSettings::default()
        };
        let mut a = Adaptive::new();
        assert_eq!(a.observe(1800, &settings), None);
        assert_eq!(a.observe(HITCH_HUNDREDTHS_MS, &settings), None);
        assert_eq!(a.observe(1800, &settings), None); // run restarted after the hitch
        assert_eq!(a.observe(1800, &settings), Some(AdaptiveEvent::Push));
        let off = AdaptiveSettings::default();
        assert_eq!(a.observe(1667, &off), Some(AdaptiveEvent::Pop));
        assert_eq!(a.observe(1667, &off), None);
        assert_eq!(TICKS_PER_HUNDREDTH_MS * 100 * 1000, 19_200_000);
    }
}
