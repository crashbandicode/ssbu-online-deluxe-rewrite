//! Resolution level, dynamic-resolution reports, and crop delay.
//!
//! The Switch hooks live in `runtime::resolution`. This module is the
//! allocation-free policy those hooks share with host tests. It does not write
//! the INI, and it does not treat a requested level as applied until
//! [`commit_original`] runs from the game function hook.

use crate::{config::Platform, render_policy::Profile};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// Canonical level order. A higher discriminant is a lower resolution.
/// The dynamic-resolution override is the maximum discriminant in the multiset.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Level {
    P1080 = 0,
    P900 = 1,
    P720 = 2,
    P576 = 3,
    P480 = 4,
}

impl Level {
    pub const fn from_u8(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::P1080,
            1 => Self::P900,
            2 => Self::P720,
            3 => Self::P576,
            4 => Self::P480,
            _ => return None,
        })
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::P1080 => "1080",
            Self::P900 => "900",
            Self::P720 => "720",
            Self::P576 => "576",
            Self::P480 => "480",
        }
    }

    pub const fn size(self) -> (u32, u32) {
        match self {
            Self::P1080 => (1920, 1080),
            Self::P900 => (1600, 900),
            Self::P720 => (1280, 720),
            Self::P576 => (1024, 576),
            Self::P480 => (854, 480),
        }
    }
}

/// Presents the canonical crop counter must already hold before `nvnWindowSetCrop`.
///
/// `present_texture_wrapper` compares the counter with 1, 2, or 3 and then
/// increments it. The resolution hook zeroes the counter. Immediate therefore
/// crops on the present that observes 1, which is the second tracked present
/// after the hook.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum IndexDelay {
    Immediate = 1,
    OneBehind = 2,
    TwoBehind = 3,
}

impl IndexDelay {
    pub const fn presents(self) -> u32 {
        self as u32
    }
}

/// One observed `nvnWindowSetCrop`. The policy does not call the driver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservedCrop {
    pub tracked: bool,
    pub hooks_ready: bool,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

const fn is_fullscreen_reset(x: i32, y: i32, width: i32, height: i32) -> bool {
    let (full_width, full_height) = Level::P1080.size();
    x == 0 && y == 0 && width == full_width as i32 && height == full_height as i32
}

/// Crop delay follows the committed render-policy profile, not a startup guess.
pub const fn delay_for_profile(profile: Profile) -> IndexDelay {
    match profile {
        Profile::Ultra => IndexDelay::Immediate,
        Profile::LessLag | Profile::Doubles => IndexDelay::OneBehind,
        Profile::Vanilla => IndexDelay::TwoBehind,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Setting {
    pub level: Level,
    pub dynamic: bool,
}

/// Committed profile plus the explicit platform. Legacy core modes are not
/// profiles: a machine that has not committed stays on [`active_setting`]'s
/// 1080p, dynamic-resolution-off result.
///
/// Vanilla is 1080p with dynamic resolution off. Console LessLag is 1080p on.
/// Console Ultra is `ultra` on (the INI's `ultra_resolution`, 576p by
/// default). Emulator LessLag and Ultra stay 1080p off. LLDoubles is 1080p on
/// for both platforms.
pub const fn defaults_for(profile: Profile, platform: Platform, ultra: Level) -> Setting {
    match (profile, platform) {
        (Profile::Vanilla, _) => Setting {
            level: Level::P1080,
            dynamic: false,
        },
        (Profile::LessLag, Platform::Console) => Setting {
            level: Level::P1080,
            dynamic: true,
        },
        (Profile::Ultra, Platform::Console) => Setting {
            level: ultra,
            dynamic: true,
        },
        (Profile::LessLag | Profile::Ultra, Platform::Emulator) => Setting {
            level: Level::P1080,
            dynamic: false,
        },
        (Profile::Doubles, _) => Setting {
            level: Level::P1080,
            dynamic: true,
        },
    }
}

/// Live committed profile only. A passthrough machine and a rejected
/// resolution guard both stay 1080p with dynamic resolution off, so a
/// requested profile cannot advertise or apply ahead of commit.
pub const fn active_setting(
    live: bool,
    hooks_ready: bool,
    committed: Profile,
    platform: Platform,
    ultra: Level,
) -> Setting {
    if live && hooks_ready {
        defaults_for(committed, platform, ultra)
    } else {
        Setting {
            level: Level::P1080,
            dynamic: false,
        }
    }
}

static HOOKS_READY: AtomicBool = AtomicBool::new(false);

pub fn set_hooks_ready(ready: bool) {
    HOOKS_READY.store(ready, Ordering::Release);
}

pub fn hooks_ready() -> bool {
    HOOKS_READY.load(Ordering::Acquire)
}

/// Far function hooks are `ldr x17, [pc, #8]; br x17` (`0x58000051`) when the
/// NRO is outside ±128 MB. Inline sites stay on the `0x14` branch form.
pub const fn installed_hook_word(offset: usize, word: u32) -> bool {
    let branch = word & 0xfc00_0000 == 0x1400_0000;
    let far_entry = offset == FUNCTION_OFFSET && word == 0x5800_0051;
    branch || far_entry
}

/// `(width << 32) | height`, the canonical packed word.
pub const fn canonical_pack(level: Level) -> u64 {
    let (width, height) = level.size();
    ((width as u64) << 32) | (height as u64)
}

/// Saved `x8` at `0x37cc17c`: `(height << 32) | width`.
pub const fn object_x8(pack: u64) -> u64 {
    pack.rotate_left(32)
}

/// Saved `x21` at `0x37cc1b0`: `(width << 32) | 0xe`.
pub const fn callback_x21(pack: u64) -> u64 {
    (pack & 0xffff_ffff_0000_0000) | 0xe
}

/// Saved `x22` at `0x37cc1b0`: height in the low 32 bits.
pub const fn callback_x22(pack: u64) -> u64 {
    pack & 0xffff_ffff
}

pub const NOP: u32 = 0xd503_201f;
pub const REPORT_CAP: usize = 8;
pub const FUNCTION_OFFSET: usize = 0x37cc130;
pub const OBJECT_INLINE: usize = 0x37cc17c;
pub const CALLBACK_INLINE: usize = 0x37cc1b0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SiteKind {
    /// Function hook. The word is not patched before `A64HookFunction`.
    Entry,
    /// Branch replaced with [`NOP`] before either inline hook is installed.
    Gate,
    /// Inline hook. The original instruction stays in the trampoline.
    Inline,
    /// Identity check only. Install writes nothing at this address.
    Layout,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Site {
    pub offset: usize,
    pub word: u32,
    pub kind: SiteKind,
}

/// 13.0.5 text words. Install reads every row before the first store.
pub const SITES: &[Site] = &[
    Site {
        offset: FUNCTION_OFFSET,
        word: 0xd101_83ff,
        kind: SiteKind::Entry,
    },
    Site {
        offset: 0x37cc154,
        word: 0x3600_07e0,
        kind: SiteKind::Gate,
    },
    Site {
        offset: 0x37cc160,
        word: 0x5400_0781,
        kind: SiteKind::Gate,
    },
    Site {
        offset: 0x37cc168,
        word: 0x3400_03e0,
        kind: SiteKind::Gate,
    },
    Site {
        offset: 0x37cc170,
        word: 0x5400_0701,
        kind: SiteKind::Gate,
    },
    Site {
        offset: 0x37cc174,
        word: 0xd280_f008,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x37cc178,
        word: 0xf2c0_8708,
        kind: SiteKind::Layout,
    },
    Site {
        offset: OBJECT_INLINE,
        word: 0xf900_1a68,
        kind: SiteKind::Inline,
    },
    Site {
        offset: 0x37cc1a0,
        word: 0xd280_01d5,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x37cc1a4,
        word: 0xaa00_03f3,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x37cc1a8,
        word: 0x5280_8716,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x37cc1ac,
        word: 0xf2c0_f015,
        kind: SiteKind::Layout,
    },
    Site {
        offset: CALLBACK_INLINE,
        word: 0xb000_d3b7,
        kind: SiteKind::Inline,
    },
    Site {
        offset: 0x37cc1b4,
        word: 0x913d_e2f7,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x37cc1e4,
        word: 0xd280_a008,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x37cc1e8,
        word: 0xf2c0_5a08,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x37cc1ec,
        word: 0xf900_1a68,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x37cc220,
        word: 0xb000_d3b7,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x37cc264,
        word: 0xd65f_03c0,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x0374_78c0,
        word: 0x9402_121c,
        kind: SiteKind::Layout,
    },
    Site {
        offset: 0x0374_7ac0,
        word: 0x9402_119c,
        kind: SiteKind::Layout,
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReportToken {
    pub index: u8,
    pub generation: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PushResult {
    Stored(ReportToken),
    /// Dynamic resolution is off, so the report is not stored.
    Ignored,
    /// All [`REPORT_CAP`] slots are live.
    Full,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Status {
    pub requested: Level,
    pub dynamic: bool,
    pub effective: Level,
    pub applied: Option<Level>,
    pub dirty: bool,
    pub reports: u8,
    pub crop_counter: u32,
    pub cropped: bool,
    pub crop: Option<(u32, u32)>,
    pub original_calls: u32,
}

pub const TRANSITION_CAP: usize = 12;
pub const TRANSITION_PUSH: u8 = 1;
pub const TRANSITION_POP: u8 = 2;
pub const TRANSITION_APPLIED: u8 = 3;
pub const TRANSITION_CROP: u8 = 4;
pub const TRANSITION_CLEARED: u8 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transition {
    pub kind: u8,
    pub a: u32,
    pub b: u32,
}

const NO_TRANSITION: Transition = Transition {
    kind: 0,
    a: 0,
    b: 0,
};

#[derive(Clone, Copy)]
struct Slot {
    level: Level,
    generation: u16,
    live: bool,
}

const EMPTY_SLOT: Slot = Slot {
    level: Level::P1080,
    generation: 0,
    live: false,
};

#[derive(Clone, Copy)]
pub struct Policy {
    requested: Level,
    dynamic: bool,
    slots: [Slot; REPORT_CAP],
    next_generation: u16,
    dirty: bool,
    applied: Option<Level>,
    crop_counter: u32,
    cropped: bool,
    crop: Option<(u32, u32)>,
    original_calls: u32,
    transitions: [Transition; TRANSITION_CAP],
    transition_len: u8,
    transition_dropped: u8,
}

impl Policy {
    pub const fn initial() -> Self {
        Self {
            requested: Level::P1080,
            dynamic: false,
            slots: [EMPTY_SLOT; REPORT_CAP],
            next_generation: 1,
            dirty: true,
            applied: None,
            crop_counter: 0,
            cropped: false,
            crop: None,
            original_calls: 0,
            transitions: [NO_TRANSITION; TRANSITION_CAP],
            transition_len: 0,
            transition_dropped: 0,
        }
    }

    pub fn dropped_transitions(&self) -> u8 {
        self.transition_dropped
    }

    pub fn transitions(&self) -> &[Transition] {
        &self.transitions[..self.transition_len as usize]
    }

    pub const fn effective(self) -> Level {
        if !self.dynamic {
            return self.requested;
        }
        let mut index = 0;
        let mut best: Option<Level> = None;
        while index < REPORT_CAP {
            let slot = self.slots[index];
            if slot.live {
                best = Some(match best {
                    Some(current) if (current as u8) >= (slot.level as u8) => current,
                    _ => slot.level,
                });
            }
            index += 1;
        }
        match best {
            Some(level) => level,
            None => self.requested,
        }
    }

    pub const fn status(self) -> Status {
        Status {
            requested: self.requested,
            dynamic: self.dynamic,
            effective: self.effective(),
            applied: self.applied,
            dirty: self.dirty,
            reports: self.live_reports(),
            crop_counter: self.crop_counter,
            cropped: self.cropped,
            crop: self.crop,
            original_calls: self.original_calls,
        }
    }

    const fn live_reports(self) -> u8 {
        let mut index = 0;
        let mut count = 0u8;
        while index < REPORT_CAP {
            if self.slots[index].live {
                count += 1;
            }
            index += 1;
        }
        count
    }

    pub fn update_settings(&mut self, level: Level, dynamic: bool) {
        let before = self.effective();
        let live = self.live_reports();
        self.requested = level;
        self.dynamic = dynamic;
        if !dynamic {
            self.clear_slots();
            if live != 0 {
                self.note(TRANSITION_CLEARED, u32::from(live), 0);
            }
        }
        if self.applied.is_none() || self.effective() != before {
            self.dirty = true;
        }
    }

    /// Changes the configured base or dynamic-resolution flag and drops every
    /// live report. The register rewrite and the dirty/original call shape stay
    /// as they are: nothing here claims the new level is applied.
    pub fn retarget(&mut self, level: Level, dynamic: bool) -> bool {
        if self.requested == level && self.dynamic == dynamic {
            return false;
        }
        let live = self.live_reports();
        self.clear_slots();
        self.requested = level;
        self.dynamic = dynamic;
        self.dirty = true;
        if live != 0 {
            self.note(TRANSITION_CLEARED, u32::from(live), 0);
        }
        true
    }

    pub fn push_report(&mut self, level: Level) -> PushResult {
        if !self.dynamic {
            return PushResult::Ignored;
        }
        let before = self.effective();
        let Some(index) = self.slots.iter().position(|slot| !slot.live) else {
            return PushResult::Full;
        };
        let generation = self.bump_generation();
        self.slots[index] = Slot {
            level,
            generation,
            live: true,
        };
        if self.effective() != before {
            self.dirty = true;
        }
        self.note(
            TRANSITION_PUSH,
            level as u32,
            u32::from(self.live_reports()),
        );
        PushResult::Stored(ReportToken {
            index: index as u8,
            generation,
        })
    }

    pub fn pop_report(&mut self, token: ReportToken) -> bool {
        let Some(slot) = self.slots.get(token.index as usize).copied() else {
            return false;
        };
        if !slot.live || slot.generation != token.generation {
            return false;
        }
        let before = self.effective();
        self.slots[token.index as usize].live = false;
        if self.effective() != before {
            self.dirty = true;
        }
        self.note(
            TRANSITION_POP,
            slot.level as u32,
            u32::from(self.live_reports()),
        );
        true
    }

    pub fn clear_reports(&mut self) {
        let before = self.effective();
        let live = self.live_reports();
        self.clear_slots();
        if self.effective() != before {
            self.dirty = true;
        }
        if live != 0 {
            self.note(TRANSITION_CLEARED, u32::from(live), 0);
        }
    }

    /// Drops dynamic-resolution reports and asks the next game call to run the
    /// original once. The last applied level stays visible until that call.
    pub fn reset_scene(&mut self) {
        let live = self.live_reports();
        self.clear_slots();
        self.dirty = true;
        if live != 0 {
            self.note(TRANSITION_CLEARED, u32::from(live), 0);
        }
    }

    pub fn begin_original(&mut self) -> Option<Level> {
        if !self.dirty {
            return None;
        }
        self.dirty = false;
        Some(self.effective())
    }

    pub fn commit_original(&mut self, level: Level) {
        self.applied = Some(level);
        self.crop_counter = 0;
        self.cropped = false;
        self.crop = None;
        self.original_calls = self.original_calls.saturating_add(1);
        self.note(TRANSITION_APPLIED, level as u32, self.original_calls);
    }

    /// `Some` once the counter has reached the delay. A shorter committed delay
    /// still matches a counter that already passed the old threshold. The
    /// counter does not advance while due, so a missing crop pointer retries,
    /// and it does not climb forever. [`note_crop`] stays single-shot.
    pub fn due_crop(&mut self, delay: IndexDelay) -> Option<(u32, u32)> {
        if self.applied.is_none() || self.cropped {
            return None;
        }
        if self.crop_counter >= delay.presents() {
            return self.applied.map(Level::size);
        }
        self.crop_counter = self.crop_counter.saturating_add(1);
        None
    }

    pub fn note_crop(&mut self, width: u32, height: u32) {
        if self.cropped {
            return;
        }
        self.cropped = true;
        self.crop = Some((width, height));
        self.crop_counter = self.crop_counter.saturating_add(1);
        self.note(TRANSITION_CROP, width, height);
    }

    /// Invalidate an accepted crop after the game writes exactly
    /// `0,0,1920,1080` on the tracked window. The counter becomes
    /// [`IndexDelay::presents`], so the next [`due_crop`] returns the applied
    /// size and retries until [`note_crop`].
    ///
    /// A not-yet-cropped apply is unchanged, including a fullscreen write
    /// inside the original resolution call before [`commit_original`] zeroes
    /// the counter. This does not change the applied level, the dirty bit, or
    /// dynamic-resolution reports, and it does not call the resolution hook.
    pub fn rearm_fullscreen_reset(&mut self, delay: IndexDelay, observed: ObservedCrop) -> bool {
        if !self.accepts_fullscreen_reset(observed) {
            return false;
        }
        self.cropped = false;
        self.crop = None;
        self.crop_counter = delay.presents();
        true
    }

    fn accepts_fullscreen_reset(&self, observed: ObservedCrop) -> bool {
        if !observed.tracked || !observed.hooks_ready || !self.cropped {
            return false;
        }
        let Some(applied) = self.applied else {
            return false;
        };
        if applied == Level::P1080 {
            return false;
        }
        let (width, height) = applied.size();
        let (full_width, full_height) = Level::P1080.size();
        if width == full_width && height == full_height {
            return false;
        }
        is_fullscreen_reset(observed.x, observed.y, observed.width, observed.height)
    }

    fn note(&mut self, kind: u8, a: u32, b: u32) {
        if (self.transition_len as usize) < TRANSITION_CAP {
            let index = self.transition_len as usize;
            self.transitions[index] = Transition { kind, a, b };
            self.transition_len += 1;
        } else {
            self.transition_dropped = self.transition_dropped.saturating_add(1);
        }
        emit_transition(kind, a, b);
    }

    fn clear_slots(&mut self) {
        self.slots = [EMPTY_SLOT; REPORT_CAP];
    }

    fn bump_generation(&mut self) -> u16 {
        let generation = self.next_generation;
        self.next_generation = self.next_generation.wrapping_add(1);
        if self.next_generation == 0 {
            self.next_generation = 1;
        }
        generation
    }
}

fn emit_transition(kind: u8, a: u32, b: u32) {
    #[cfg(target_os = "switch")]
    {
        let (code, event_a, event_b) = match kind {
            TRANSITION_APPLIED => (
                crate::diagnostics::Code::ResolutionApplied,
                u64::from(a),
                u64::from(b),
            ),
            TRANSITION_CROP => (
                crate::diagnostics::Code::ResolutionCrop,
                u64::from(a),
                u64::from(b),
            ),
            _ => (
                crate::diagnostics::Code::ResolutionReport,
                u64::from(kind),
                u64::from(a) | (u64::from(b) << 32),
            ),
        };
        crate::runtime::record(code, event_a, event_b);
    }
    #[cfg(not(target_os = "switch"))]
    {
        let _ = (kind, a, b);
    }
}

static POLICY: Mutex<Policy> = Mutex::new(Policy::initial());

fn with_policy<T>(body: impl FnOnce(&mut Policy) -> T) -> T {
    let mut policy = POLICY.lock().unwrap_or_else(|err| err.into_inner());
    body(&mut policy)
}

pub fn update_settings(level: Level, dynamic: bool) {
    with_policy(|policy| policy.update_settings(level, dynamic));
}

pub fn push_report(level: Level) -> PushResult {
    with_policy(|policy| policy.push_report(level))
}

pub fn pop_report(token: ReportToken) -> bool {
    with_policy(|policy| policy.pop_report(token))
}

pub fn clear_reports() {
    with_policy(|policy| policy.clear_reports());
}

pub fn reset_scene() {
    with_policy(|policy| policy.reset_scene());
}

pub fn needs_retarget(level: Level, dynamic: bool) -> bool {
    let view = status();
    view.requested != level || view.dynamic != dynamic
}

pub fn retarget(level: Level, dynamic: bool) -> bool {
    with_policy(|policy| policy.retarget(level, dynamic))
}

pub fn applied_label(applied: Option<Level>) -> &'static str {
    match applied {
        Some(level) => level.label(),
        None => "Unknown",
    }
}

/// Configured dynamic resolution is [`Status::dynamic`]. Triggers exist only
/// when logical Main is armed and the count-only marker is absent.
pub const fn triggers_available(main_armed: bool, count_only: bool) -> bool {
    main_armed && !count_only
}

pub fn format_status(ready: bool, triggers_available: bool, status: Status) -> String {
    let applied = match status.applied {
        Some(level) => level.label(),
        None => "unknown",
    };
    let crop = match status.crop {
        Some((width, height)) => format!("{width}x{height}"),
        None => "none".to_owned(),
    };
    let triggers = if triggers_available {
        "available"
    } else {
        "unavailable"
    };
    format!(
        "resolution_ready={ready}\nresolution_dirty={dirty}\nresolution_requested={requested}\nresolution_effective={effective}\nresolution_applied={applied}\nresolution_dynamic={dynamic}\nresolution_triggers={triggers}\nresolution_crop={crop}\nresolution_reports={reports}\n",
        ready = if ready { "true" } else { "false" },
        dirty = if status.dirty { "true" } else { "false" },
        requested = status.requested.label(),
        effective = status.effective.label(),
        dynamic = if status.dynamic { "true" } else { "false" },
        reports = status.reports,
    )
}

pub fn status() -> Status {
    with_policy(|policy| policy.status())
}

pub fn begin_original() -> Option<Level> {
    with_policy(|policy| policy.begin_original())
}

pub fn commit_original(level: Level) {
    with_policy(|policy| policy.commit_original(level));
}

pub fn due_crop(delay: IndexDelay) -> Option<(u32, u32)> {
    with_policy(|policy| policy.due_crop(delay))
}

pub fn note_crop(width: u32, height: u32) {
    with_policy(|policy| policy.note_crop(width, height));
}

pub fn rearm_fullscreen_reset(delay: IndexDelay, observed: ObservedCrop) -> bool {
    with_policy(|policy| policy.rearm_fullscreen_reset(delay, observed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(result: PushResult) -> ReportToken {
        match result {
            PushResult::Stored(token) => token,
            other => panic!("expected a stored report, got {other:?}"),
        }
    }

    #[test]
    fn multiset_keeps_the_lowest_resolution() {
        let mut policy = Policy::initial();
        policy.update_settings(Level::P1080, true);
        let _ = policy.begin_original();
        policy.commit_original(Level::P1080);
        assert_eq!(token_level(&mut policy, Level::P720), Level::P720);
        assert_eq!(token_level(&mut policy, Level::P900), Level::P720);
        assert_eq!(token_level(&mut policy, Level::P480), Level::P480);
        assert_eq!(policy.status().requested, Level::P1080);
        assert_eq!(policy.effective(), Level::P480);
    }

    fn token_level(policy: &mut Policy, level: Level) -> Level {
        assert!(matches!(policy.push_report(level), PushResult::Stored(_)));
        policy.effective()
    }

    #[test]
    fn disabling_dynamic_resolution_clears_reports() {
        let mut policy = Policy::initial();
        policy.update_settings(Level::P1080, true);
        let stored = token(policy.push_report(Level::P480));
        assert_eq!(policy.effective(), Level::P480);
        policy.update_settings(Level::P720, false);
        assert_eq!(policy.status().reports, 0);
        assert_eq!(policy.effective(), Level::P720);
        assert!(!policy.pop_report(stored));
        assert_eq!(policy.push_report(Level::P576), PushResult::Ignored);
    }

    #[test]
    fn clear_reports_returns_to_the_requested_level() {
        let mut policy = Policy::initial();
        policy.update_settings(Level::P1080, true);
        let _ = policy.push_report(Level::P576);
        policy.clear_reports();
        assert_eq!(policy.status().reports, 0);
        assert_eq!(policy.effective(), Level::P1080);
        assert!(policy.status().dirty);
    }

    #[test]
    fn defaults_follow_the_committed_profile_and_platform() {
        let off = Setting {
            level: Level::P1080,
            dynamic: false,
        };
        let on_1080 = Setting {
            level: Level::P1080,
            dynamic: true,
        };
        let ultra = Setting {
            level: Level::P576,
            dynamic: true,
        };
        let base = Level::P576;
        assert_eq!(defaults_for(Profile::Vanilla, Platform::Console, base), off);
        assert_eq!(
            defaults_for(Profile::Vanilla, Platform::Emulator, base),
            off
        );
        assert_eq!(
            defaults_for(Profile::LessLag, Platform::Console, base),
            on_1080
        );
        assert_eq!(defaults_for(Profile::Ultra, Platform::Console, base), ultra);
        assert_eq!(
            defaults_for(Profile::LessLag, Platform::Emulator, base),
            off
        );
        assert_eq!(defaults_for(Profile::Ultra, Platform::Emulator, base), off);
        assert_eq!(
            defaults_for(Profile::Doubles, Platform::Console, base),
            on_1080
        );
        assert_eq!(
            defaults_for(Profile::Doubles, Platform::Emulator, base),
            on_1080
        );
        // `ultra_resolution` moves console Ultra alone; emulator Ultra and the
        // other profiles ignore it. The INI offers 720; the policy itself
        // takes any level.
        for level in [Level::P720, Level::P1080] {
            assert_eq!(
                defaults_for(Profile::Ultra, Platform::Console, level),
                Setting {
                    level,
                    dynamic: true
                }
            );
            assert_eq!(defaults_for(Profile::Ultra, Platform::Emulator, level), off);
            assert_eq!(
                defaults_for(Profile::LessLag, Platform::Console, level),
                on_1080
            );
        }
        assert_eq!(
            active_setting(false, true, Profile::Ultra, Platform::Console, base),
            off
        );
        assert_eq!(
            active_setting(true, false, Profile::Doubles, Platform::Console, base),
            off
        );
        assert_eq!(
            active_setting(true, true, Profile::LessLag, Platform::Console, base),
            on_1080
        );
        assert_eq!(
            active_setting(true, true, Profile::Ultra, Platform::Emulator, base),
            off
        );
        assert_eq!(
            active_setting(true, true, Profile::Ultra, Platform::Console, Level::P720),
            Setting {
                level: Level::P720,
                dynamic: true
            }
        );
        assert!(installed_hook_word(FUNCTION_OFFSET, 0x5800_0051));
        assert!(installed_hook_word(FUNCTION_OFFSET, 0x1400_0001));
        assert!(!installed_hook_word(OBJECT_INLINE, 0x5800_0051));
        assert!(!installed_hook_word(CALLBACK_INLINE, 0x5800_0051));
        assert!(installed_hook_word(OBJECT_INLINE, 0x1400_0001));
        assert!(!installed_hook_word(FUNCTION_OFFSET, 0x5800_0050));
        assert!(!installed_hook_word(FUNCTION_OFFSET, 0xd106_83ff));
        assert_eq!(
            delay_for_profile(Profile::Ultra).presents(),
            IndexDelay::Immediate.presents()
        );
        assert_eq!(delay_for_profile(Profile::LessLag), IndexDelay::OneBehind);
        assert_eq!(delay_for_profile(Profile::Doubles), IndexDelay::OneBehind);
        assert_eq!(delay_for_profile(Profile::Vanilla), IndexDelay::TwoBehind);
    }

    #[test]
    fn retarget_clears_reports_without_claiming_the_new_level() {
        let mut policy = Policy::initial();
        policy.update_settings(Level::P1080, true);
        let level = policy.begin_original().expect("first");
        policy.commit_original(level);
        let _ = policy.push_report(Level::P720);
        assert!(!policy.retarget(Level::P1080, true));
        assert_eq!(policy.status().reports, 1);
        assert!(policy.retarget(Level::P576, true));
        let view = policy.status();
        assert_eq!(view.reports, 0);
        assert_eq!(view.requested, Level::P576);
        assert_eq!(view.effective, Level::P576);
        assert_eq!(view.applied, Some(Level::P1080));
        assert!(view.dirty);
        assert!(policy.begin_original().is_some());
        assert_eq!(policy.status().applied, Some(Level::P1080));
        let text = format_status(true, false, policy.status());
        assert!(text.contains("resolution_ready=true\n"));
        assert!(text.contains("resolution_dirty=false\n"));
        assert!(text.contains("resolution_requested=576\n"));
        assert!(text.contains("resolution_applied=1080\n"));
        assert!(text.contains("resolution_dynamic=true\n"));
        assert!(text.contains("resolution_triggers=unavailable\n"));
        assert!(text.contains("resolution_reports=0\n"));
        assert!(!triggers_available(false, false));
        assert!(!triggers_available(true, true));
        assert!(triggers_available(true, false));
        let unknown = format_status(
            false,
            true,
            Status {
                applied: None,
                ..policy.status()
            },
        );
        assert!(unknown.contains("resolution_ready=false\n"));
        assert!(unknown.contains("resolution_applied=unknown\n"));
        assert!(unknown.contains("resolution_dynamic=true\n"));
        assert!(unknown.contains("resolution_triggers=available\n"));
        assert_eq!(applied_label(None), "Unknown");
        assert_eq!(applied_label(Some(Level::P576)), "576");
    }

    #[test]
    fn push_pop_apply_and_crop_are_logged_in_order() {
        let mut policy = Policy::initial();
        policy.update_settings(Level::P1080, true);
        let stored = match policy.push_report(Level::P720) {
            PushResult::Stored(token) => token,
            other => panic!("expected stored, got {other:?}"),
        };
        assert!(policy.pop_report(stored));
        let level = policy.begin_original().expect("apply");
        policy.commit_original(level);
        let size = loop {
            if let Some(size) = policy.due_crop(IndexDelay::Immediate) {
                break size;
            }
        };
        policy.note_crop(size.0, size.1);
        let kinds: Vec<u8> = policy.transitions().iter().map(|item| item.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TRANSITION_PUSH,
                TRANSITION_POP,
                TRANSITION_APPLIED,
                TRANSITION_CROP
            ]
        );
        assert_eq!(policy.transitions()[3].a, 1920);
        assert_eq!(policy.transitions()[3].b, 1080);
        assert_eq!(policy.dropped_transitions(), 0);
    }

    #[test]
    fn pop_underflow_does_not_wrap() {
        let mut policy = Policy::initial();
        policy.update_settings(Level::P1080, true);
        assert!(!policy.pop_report(ReportToken {
            index: 0,
            generation: 1
        }));
        assert_eq!(policy.status().reports, 0);
        let stored = token(policy.push_report(Level::P720));
        assert!(policy.pop_report(stored));
        assert!(!policy.pop_report(stored));
        assert_eq!(policy.status().reports, 0);
        assert!(!policy.pop_report(ReportToken {
            index: 99,
            generation: 1
        }));
    }

    #[test]
    fn push_stops_at_the_report_bound() {
        let mut policy = Policy::initial();
        policy.update_settings(Level::P1080, true);
        let mut tokens = [ReportToken {
            index: 0,
            generation: 0,
        }; REPORT_CAP];
        for slot in &mut tokens {
            *slot = token(policy.push_report(Level::P900));
        }
        assert_eq!(policy.push_report(Level::P480), PushResult::Full);
        assert_eq!(policy.effective(), Level::P900);
        assert_eq!(policy.status().reports, REPORT_CAP as u8);
        assert!(policy.pop_report(tokens[3]));
        let replaced = token(policy.push_report(Level::P480));
        assert_eq!(replaced.index, tokens[3].index);
        assert_ne!(replaced.generation, tokens[3].generation);
        assert!(!policy.pop_report(tokens[3]));
        assert_eq!(policy.effective(), Level::P480);
    }

    #[test]
    fn crop_waits_until_the_counter_equals_the_delay() {
        for (delay, idle_presents) in [
            (IndexDelay::Immediate, 1u32),
            (IndexDelay::OneBehind, 2),
            (IndexDelay::TwoBehind, 3),
        ] {
            let mut policy = Policy::initial();
            assert!(policy.due_crop(delay).is_none());
            assert_eq!(policy.status().crop_counter, 0);
            let level = policy.begin_original().expect("initial apply");
            assert!(policy.begin_original().is_none());
            policy.commit_original(level);
            for _ in 0..idle_presents {
                assert!(policy.due_crop(delay).is_none());
            }
            let size = policy.due_crop(delay).expect("crop due");
            assert_eq!(size, level.size());
            policy.note_crop(size.0, size.1);
            assert_eq!(policy.status().crop, Some(size));
            assert!(policy.due_crop(delay).is_none());
        }
    }

    #[test]
    fn a_shorter_delay_still_crops_once() {
        let mut policy = Policy::initial();
        let level = policy.begin_original().expect("apply");
        policy.commit_original(level);
        assert!(policy.due_crop(IndexDelay::TwoBehind).is_none());
        assert!(policy.due_crop(IndexDelay::TwoBehind).is_none());
        let size = policy
            .due_crop(IndexDelay::Immediate)
            .expect("shrunk delay is due");
        assert_eq!(size, level.size());
        assert_eq!(policy.due_crop(IndexDelay::Immediate), Some(size));
        policy.note_crop(size.0, size.1);
        assert!(policy.due_crop(IndexDelay::Immediate).is_none());
        assert!(policy.due_crop(IndexDelay::OneBehind).is_none());
        assert!(policy.due_crop(IndexDelay::TwoBehind).is_none());
    }

    #[test]
    fn requested_stays_distinct_from_applied_until_the_hook() {
        let mut policy = Policy::initial();
        policy.update_settings(Level::P576, true);
        let stored = token(policy.push_report(Level::P480));
        let view = policy.status();
        assert_eq!(view.requested, Level::P576);
        assert_eq!(view.effective, Level::P480);
        assert_eq!(view.applied, None);
        assert!(view.dirty);
        assert_eq!(view.original_calls, 0);
        let applied = policy.begin_original().expect("dirty call");
        assert_eq!(applied, Level::P480);
        assert!(policy.begin_original().is_none());
        assert_eq!(policy.status().applied, None);
        policy.commit_original(applied);
        let committed = policy.status();
        assert_eq!(committed.applied, Some(Level::P480));
        assert_eq!(committed.original_calls, 1);
        assert!(!committed.dirty);
        assert!(policy.begin_original().is_none());
        policy.reset_scene();
        assert_eq!(policy.status().reports, 0);
        assert_eq!(policy.status().applied, Some(Level::P480));
        assert!(policy.status().dirty);
        assert!(!policy.pop_report(stored));
        let next = policy.begin_original().expect("scene reapply");
        assert_eq!(next, Level::P576);
        policy.commit_original(next);
        assert_eq!(policy.status().applied, Some(Level::P576));
        assert_eq!(policy.status().crop_counter, 0);
        assert!(!policy.status().cropped);
    }

    #[test]
    fn inline_registers_match_the_1080_and_576_pack() {
        let full = canonical_pack(Level::P1080);
        assert_eq!(object_x8(full), (1080u64 << 32) | 1920);
        assert_eq!(callback_x21(full), (1920u64 << 32) | 0xe);
        assert_eq!(callback_x22(full), 1080);
        let console = canonical_pack(Level::P576);
        assert_eq!(object_x8(console), (576u64 << 32) | 1024);
        assert_eq!(callback_x21(console), (1024u64 << 32) | 0xe);
        assert_eq!(callback_x22(console), 576);
        for level in [
            Level::P1080,
            Level::P900,
            Level::P720,
            Level::P576,
            Level::P480,
        ] {
            let (width, height) = level.size();
            let pack = canonical_pack(level);
            assert_eq!(pack, (u64::from(width) << 32) | u64::from(height));
            assert_ne!(pack, 0);
        }
    }

    #[test]
    fn gate_words_are_the_13_0_5_branches_and_not_nops() {
        let gates: Vec<usize> = SITES
            .iter()
            .filter(|site| site.kind == SiteKind::Gate)
            .map(|site| site.offset)
            .collect();
        assert_eq!(gates, vec![0x37cc154, 0x37cc160, 0x37cc168, 0x37cc170]);
        for site in SITES {
            if site.kind == SiteKind::Gate {
                assert_ne!(site.word, NOP);
            }
        }
        assert_eq!(NOP, 0xd503_201f);
        assert!(SITES.iter().any(|site| {
            site.offset == 0x37cc17c && site.kind == SiteKind::Inline && site.word == 0xf900_1a68
        }));
        assert!(SITES.iter().any(|site| {
            site.offset == 0x37cc1b0 && site.kind == SiteKind::Inline && site.word == 0xb000_d3b7
        }));
        assert!(SITES
            .iter()
            .any(|site| site.offset == 0x0374_7ac0 && site.kind == SiteKind::Layout));
    }

    fn fullscreen(tracked: bool, hooks_ready: bool) -> ObservedCrop {
        let (width, height) = Level::P1080.size();
        ObservedCrop {
            tracked,
            hooks_ready,
            x: 0,
            y: 0,
            width: width as i32,
            height: height as i32,
        }
    }

    fn apply_level(policy: &mut Policy, level: Level, dynamic: bool) {
        policy.update_settings(level, dynamic);
        let applied = policy.begin_original().expect("apply");
        assert_eq!(applied, level);
        policy.commit_original(applied);
    }

    fn accept_crop(policy: &mut Policy, delay: IndexDelay) -> (u32, u32) {
        for _ in 0..delay.presents() {
            assert!(policy.due_crop(delay).is_none());
        }
        let size = policy.due_crop(delay).expect("crop due");
        policy.note_crop(size.0, size.1);
        size
    }

    #[test]
    fn an_accepted_crop_rearms_on_the_next_present_once() {
        for delay in [
            IndexDelay::Immediate,
            IndexDelay::OneBehind,
            IndexDelay::TwoBehind,
        ] {
            let mut policy = Policy::initial();
            apply_level(&mut policy, Level::P576, true);
            let size = accept_crop(&mut policy, delay);
            assert_eq!(size, Level::P576.size());
            let before = policy.status();
            let transitions = policy.transitions().len();
            assert!(policy.rearm_fullscreen_reset(delay, fullscreen(true, true)));
            let rearmed = policy.status();
            assert_eq!(rearmed.applied, Some(Level::P576));
            assert_eq!(rearmed.requested, before.requested);
            assert_eq!(rearmed.effective, before.effective);
            assert_eq!(rearmed.dynamic, before.dynamic);
            assert_eq!(rearmed.dirty, before.dirty);
            assert_eq!(rearmed.reports, before.reports);
            assert_eq!(rearmed.original_calls, before.original_calls);
            assert!(!rearmed.cropped);
            assert_eq!(rearmed.crop, None);
            assert_eq!(rearmed.crop_counter, delay.presents());
            assert_eq!(policy.transitions().len(), transitions);
            assert_eq!(policy.due_crop(delay), Some(size));
            assert_eq!(policy.status().crop_counter, delay.presents());
            assert_eq!(policy.due_crop(delay), Some(size));
            policy.note_crop(size.0, size.1);
            assert_eq!(policy.status().crop, Some(size));
            assert!(policy.due_crop(delay).is_none());
            policy.note_crop(size.0, size.1);
            assert!(policy.due_crop(delay).is_none());
        }
    }

    #[test]
    fn a_matching_or_other_rectangle_does_not_rearm() {
        let mut full = Policy::initial();
        apply_level(&mut full, Level::P1080, false);
        let size = accept_crop(&mut full, IndexDelay::TwoBehind);
        assert_eq!(size, (1920, 1080));
        let before = full.status();
        assert!(!full.rearm_fullscreen_reset(IndexDelay::TwoBehind, fullscreen(true, true)));
        assert_eq!(full.status(), before);

        let mut policy = Policy::initial();
        apply_level(&mut policy, Level::P576, true);
        let size = accept_crop(&mut policy, IndexDelay::Immediate);
        let before = policy.status();
        let applied = ObservedCrop {
            width: size.0 as i32,
            height: size.1 as i32,
            ..fullscreen(true, true)
        };
        assert!(!policy.rearm_fullscreen_reset(IndexDelay::Immediate, applied));
        let other = ObservedCrop {
            width: 1280,
            height: 720,
            ..fullscreen(true, true)
        };
        assert!(!policy.rearm_fullscreen_reset(IndexDelay::Immediate, other));
        assert!(!policy.rearm_fullscreen_reset(IndexDelay::Immediate, fullscreen(false, true)));
        assert!(!policy.rearm_fullscreen_reset(IndexDelay::Immediate, fullscreen(true, false)));
        assert_eq!(policy.status(), before);
    }

    #[test]
    fn an_unapplied_or_pending_crop_keeps_its_delay() {
        let mut policy = Policy::initial();
        policy.update_settings(Level::P576, true);
        assert!(!policy.rearm_fullscreen_reset(IndexDelay::Immediate, fullscreen(true, true)));
        assert!(policy.status().applied.is_none());
        assert_eq!(policy.status().crop_counter, 0);
        let level = policy.begin_original().expect("during original");
        assert!(!policy.rearm_fullscreen_reset(IndexDelay::TwoBehind, fullscreen(true, true)));
        assert_eq!(policy.status().crop_counter, 0);
        policy.commit_original(level);
        assert!(!policy.status().cropped);
        assert!(!policy.rearm_fullscreen_reset(IndexDelay::Immediate, fullscreen(true, true)));
        assert_eq!(policy.status().crop_counter, 0);
        assert!(policy.due_crop(IndexDelay::Immediate).is_none());
        assert_eq!(policy.status().crop_counter, 1);
        assert!(!policy.rearm_fullscreen_reset(IndexDelay::Immediate, fullscreen(true, true)));
        assert_eq!(policy.status().crop_counter, 1);
        assert!(!policy.status().cropped);
        assert_eq!(
            policy.due_crop(IndexDelay::Immediate),
            Some(Level::P576.size())
        );
    }

    #[test]
    fn a_reset_before_commit_does_not_keep_the_rearm() {
        let mut policy = Policy::initial();
        apply_level(&mut policy, Level::P576, true);
        let delay = IndexDelay::TwoBehind;
        let _ = accept_crop(&mut policy, delay);
        policy.reset_scene();
        let next = policy.begin_original().expect("reapply");
        assert!(policy.rearm_fullscreen_reset(delay, fullscreen(true, true)));
        assert_eq!(policy.status().crop_counter, delay.presents());
        policy.commit_original(next);
        assert_eq!(policy.status().applied, Some(Level::P576));
        assert_eq!(policy.status().crop_counter, 0);
        assert!(!policy.status().cropped);
        assert_eq!(policy.status().crop, None);
        for _ in 0..delay.presents() {
            assert!(policy.due_crop(delay).is_none());
        }
        assert_eq!(policy.due_crop(delay), Some(Level::P576.size()));
    }

    #[test]
    fn rearm_preserves_dynamic_reports_and_the_dirty_bit() {
        let mut policy = Policy::initial();
        apply_level(&mut policy, Level::P576, true);
        let _ = accept_crop(&mut policy, IndexDelay::OneBehind);
        let stored = token(policy.push_report(Level::P480));
        assert_eq!(policy.effective(), Level::P480);
        assert!(policy.status().dirty);
        assert_eq!(policy.status().applied, Some(Level::P576));
        assert_eq!(policy.status().reports, 1);
        assert!(policy.rearm_fullscreen_reset(IndexDelay::OneBehind, fullscreen(true, true)));
        let view = policy.status();
        assert_eq!(view.applied, Some(Level::P576));
        assert_eq!(view.effective, Level::P480);
        assert!(view.dirty);
        assert!(view.dynamic);
        assert_eq!(view.reports, 1);
        assert_eq!(view.crop_counter, IndexDelay::OneBehind.presents());
        assert!(policy.pop_report(stored));
        assert_eq!(policy.status().reports, 0);
    }
}
