//! Palutena up air, Sephiroth Shadow Flare, and Inkling roller splash.
//!
//! On keeps the original hash. Off keeps the existing catalog. Reduced maps a
//! parent to its sibling only when scope allows it and that file's own
//! callback has already succeeded. One file's failure does not clear another.

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

use crate::config::FxMode;
use crate::eff_sibling::SiblingRequest;
use crate::effect_context::{self, Context, Profile};
use crate::effects::hash40;

pub const SOURCE: u64 = hash40("palutena_backlight2");
pub const REDUCED: u64 = hash40("palutena_backlight2_reduced");
pub const PATH_HASH: u64 = hash40("effect/fighter/palutena/ef_palutena.eff");
/// Maximum size of the ARCropolis-provided load buffer.
pub const MAX_BYTES: usize = 13 * 1024 * 1024;

pub const REASON_API: u64 = 1;
pub const REASON_TRANSFORM: u64 = 2;
pub const REASON_SIZE: u64 = 3;

pub const EDGE_PATH_HASH: u64 = hash40("effect/fighter/edge/ef_edge.eff");
/// Stock `ef_edge.eff` is 17,721,264 bytes. Eight siblings stay under this cap.
pub const EDGE_MAX_BYTES: usize = 21 * 1024 * 1024;

/// Multi-emitter Shadow Flare parents. Aura is absent: it has one emitter.
pub const EDGE_REQUESTS: &[SiblingRequest<'static>] = &[
    SiblingRequest {
        source_handle: "EDGE_SHADOWFLARE_START",
        source_set: "P_EdgeShadowFlareStart",
        new_handle: "EDGE_SHADOWFLARE_START_REDUCED",
        new_set: "P_EdgeShadowFlareStartReduced",
        keep_emitters: &["light1", "Flare0"],
    },
    SiblingRequest {
        source_handle: "EDGE_SHADOWFLARE_HOLD",
        source_set: "P_EdgeShadowFlareHold",
        new_handle: "EDGE_SHADOWFLARE_HOLD_REDUCED",
        new_set: "P_EdgeShadowFlareHoldReduced",
        keep_emitters: &["hold_line1", "flare3"],
    },
    SiblingRequest {
        source_handle: "EDGE_SHADOWFLARE_BEAM",
        source_set: "P_EdgeShadowFlareBeam",
        new_handle: "EDGE_SHADOWFLARE_BEAM_REDUCED",
        new_set: "P_EdgeShadowFlareBeamReduced",
        keep_emitters: &["lightline1"],
    },
    SiblingRequest {
        source_handle: "EDGE_SHADOWFLARE_BEAM_HIT",
        source_set: "P_EdgeShadowFlareBeamHit",
        new_handle: "EDGE_SHADOWFLARE_BEAM_HIT_REDUCED",
        new_set: "P_EdgeShadowFlareBeamHitReduced",
        keep_emitters: &["impact1", "flash2"],
    },
    SiblingRequest {
        source_handle: "EDGE_SHADOWFLARE_BULLET",
        source_set: "P_EdgeShadowFlareBullet",
        new_handle: "EDGE_SHADOWFLARE_BULLET_REDUCED",
        new_set: "P_EdgeShadowFlareBulletReduced",
        keep_emitters: &["sphereBase", "flare1"],
    },
    SiblingRequest {
        source_handle: "EDGE_SHADOWFLARE_BULLET_END",
        source_set: "P_EdgeShadowFlareBulletEnd",
        new_handle: "EDGE_SHADOWFLARE_BULLET_END_REDUCED",
        new_set: "P_EdgeShadowFlareBulletEndReduced",
        keep_emitters: &["aura1", "flare4"],
    },
    SiblingRequest {
        source_handle: "EDGE_SHADOWFLARE_BULLET_ATTACK",
        source_set: "P_EdgeShadowFlareBulletAttack",
        new_handle: "EDGE_SHADOWFLARE_BULLET_ATTACK_REDUCED",
        new_set: "P_EdgeShadowFlareBulletAttackReduced",
        keep_emitters: &["Base1", "flare1"],
    },
    SiblingRequest {
        source_handle: "EDGE_SHADOWFLARE_BULLET_HIT",
        source_set: "P_EdgeShadowFlareBulletHit",
        new_handle: "EDGE_SHADOWFLARE_BULLET_HIT_REDUCED",
        new_set: "P_EdgeShadowFlareBulletHitReduced",
        keep_emitters: &["impact1", "sphere1"],
    },
];

const EDGE_CUES: &[(u64, u64)] = &[
    (
        hash40("edge_shadowflare_start"),
        hash40("edge_shadowflare_start_reduced"),
    ),
    (
        hash40("edge_shadowflare_hold"),
        hash40("edge_shadowflare_hold_reduced"),
    ),
    (
        hash40("edge_shadowflare_beam"),
        hash40("edge_shadowflare_beam_reduced"),
    ),
    (
        hash40("edge_shadowflare_beam_hit"),
        hash40("edge_shadowflare_beam_hit_reduced"),
    ),
    (
        hash40("edge_shadowflare_bullet"),
        hash40("edge_shadowflare_bullet_reduced"),
    ),
    (
        hash40("edge_shadowflare_bullet_end"),
        hash40("edge_shadowflare_bullet_end_reduced"),
    ),
    (
        hash40("edge_shadowflare_bullet_attack"),
        hash40("edge_shadowflare_bullet_attack_reduced"),
    ),
    (
        hash40("edge_shadowflare_bullet_hit"),
        hash40("edge_shadowflare_bullet_hit_reduced"),
    ),
];

const PENDING: u8 = 0;
const READY: u8 = 1;
const UNAVAILABLE: u8 = 2;

static STATE: AtomicU8 = AtomicU8::new(PENDING);
static REASON: AtomicU64 = AtomicU64::new(0);
static LOADS: AtomicU64 = AtomicU64::new(0);
static SWAPS: AtomicU64 = AtomicU64::new(0);
static EDGE_STATE: AtomicU8 = AtomicU8::new(PENDING);
static EDGE_REASON: AtomicU64 = AtomicU64::new(0);
static EDGE_LOADS: AtomicU64 = AtomicU64::new(0);
static EDGE_SWAPS: AtomicU64 = AtomicU64::new(0);
static INK_STATE: AtomicU8 = AtomicU8::new(PENDING);
static INK_REASON: AtomicU64 = AtomicU64::new(0);
static INK_LOADS: AtomicU64 = AtomicU64::new(0);
static INK_SWAPS: AtomicU64 = AtomicU64::new(0);
static CHROM_STATE: AtomicU8 = AtomicU8::new(PENDING);
static CHROM_REASON: AtomicU64 = AtomicU64::new(0);
static CHROM_LOADS: AtomicU64 = AtomicU64::new(0);
static CHROM_SWAPS: AtomicU64 = AtomicU64::new(0);

pub const INK_PATH_HASH: u64 = hash40("effect/fighter/inkling/ef_inkling.eff");
/// Stock `ef_inkling.eff` is 5,059,072 bytes.
pub const INK_MAX_BYTES: usize = 8 * 1024 * 1024;

/// Roller splash parents. Each script requests one of these. `Splash` stays.
pub const INK_REQUESTS: &[SiblingRequest<'static>] = &[
    SiblingRequest {
        source_handle: "INKLING_ROLLER_SPLASH_WALK",
        source_set: "P_InklingRollerSplashWalk",
        new_handle: "INKLING_ROLLER_SPLASH_WALK_REDUCED",
        new_set: "P_InklingRollerSplashWalkReduced",
        keep_emitters: &["Splash"],
    },
    SiblingRequest {
        source_handle: "INKLING_ROLLER_SPLASH_RUN",
        source_set: "P_InklingRollerSplashRun",
        new_handle: "INKLING_ROLLER_SPLASH_RUN_REDUCED",
        new_set: "P_InklingRollerSplashRunReduced",
        keep_emitters: &["Splash"],
    },
    SiblingRequest {
        source_handle: "INKLING_ROLLER_SPLASH_AIR",
        source_set: "P_InklingRollerSplashAir",
        new_handle: "INKLING_ROLLER_SPLASH_AIR_REDUCED",
        new_set: "P_InklingRollerSplashAirReduced",
        keep_emitters: &["Splash"],
    },
];

pub const CHROM_PATH_HASH: u64 = hash40("effect/fighter/chrom/ef_chrom.eff");
/// Stock `ef_chrom.eff` is 10,359,296 bytes. One copied emitter must stay
/// under this cap or the callback refuses and the original cue remains.
pub const CHROM_MAX_BYTES: usize = 16 * 1024 * 1024;

/// First trail only: `P_ChromMC1` / `chrom_mc_1`. Root's census also names
/// other `chrom_mc_*` trails with life 3, 4, or 6. Those stay original.
/// `trace1` is kept and its copied life goes 3 to 2.
pub const CHROM_REQUESTS: &[SiblingRequest<'static>] = &[SiblingRequest {
    source_handle: "CHROM_MC_1",
    source_set: "P_ChromMC1",
    new_handle: "CHROM_MC_1_REDUCED",
    new_set: "P_ChromMC1Reduced",
    keep_emitters: &["trace1"],
}];

const CHROM_CUES: &[(u64, u64)] = &[(hash40("chrom_mc_1"), hash40("chrom_mc_1_reduced"))];

const INK_CUES: &[(u64, u64)] = &[
    (
        hash40("inkling_roller_splash_walk"),
        hash40("inkling_roller_splash_walk_reduced"),
    ),
    (
        hash40("inkling_roller_splash_run"),
        hash40("inkling_roller_splash_run_reduced"),
    ),
    (
        hash40("inkling_roller_splash_air"),
        hash40("inkling_roller_splash_air_reduced"),
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariantAvailability {
    Pending,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadSample {
    pub count: u64,
    pub log: bool,
}

pub fn availability() -> VariantAvailability {
    match STATE.load(Ordering::Acquire) {
        READY => VariantAvailability::Ready,
        UNAVAILABLE => VariantAvailability::Unavailable,
        _ => VariantAvailability::Pending,
    }
}

pub fn reason() -> u64 {
    REASON.load(Ordering::Acquire)
}

/// Up air and the Shadow Flare parents are not cull-table names. Context is
/// still required so a sibling is not applied to another move.
pub fn wants_context(hash: u64) -> bool {
    hash == SOURCE || edge_reduced(hash).is_some()
}

fn cue_reduced(table: &[(u64, u64)], hash: u64) -> Option<u64> {
    table
        .iter()
        .find(|(source, _)| *source == hash)
        .map(|(_, reduced)| *reduced)
}

fn edge_reduced(hash: u64) -> Option<u64> {
    cue_reduced(EDGE_CUES, hash)
}

fn ink_reduced(hash: u64) -> Option<u64> {
    cue_reduced(INK_CUES, hash)
}

fn chrom_reduced(hash: u64) -> Option<u64> {
    cue_reduced(CHROM_CUES, hash)
}

/// Clone hash for a parent that has a real sibling. Independent of FX mode,
/// scope, move, and file readiness: the clone may already exist. `null`,
/// zero, `sys_*`, and the clone hashes themselves return none.
pub fn cleanup_alias(hash: u64) -> Option<u64> {
    if hash == 0 || hash == crate::effects::NULL_EFFECT {
        return None;
    }
    let alias = if hash == SOURCE {
        Some(REDUCED)
    } else {
        edge_reduced(hash)
            .or_else(|| ink_reduced(hash))
            .or_else(|| chrom_reduced(hash))
    }?;
    if alias == 0 || alias == hash || alias == crate::effects::NULL_EFFECT {
        None
    } else {
        Some(alias)
    }
}

/// EFF path hash for a Reduced sibling parent. Zero for every other hash,
/// including Shadow Flare aura and Chrom side B.
pub fn source_file(hash: u64) -> u64 {
    if hash == SOURCE {
        PATH_HASH
    } else if edge_reduced(hash).is_some() {
        EDGE_PATH_HASH
    } else if ink_reduced(hash).is_some() {
        INK_PATH_HASH
    } else if chrom_reduced(hash).is_some() {
        CHROM_PATH_HASH
    } else {
        0
    }
}

pub fn edge_availability() -> VariantAvailability {
    match EDGE_STATE.load(Ordering::Acquire) {
        READY => VariantAvailability::Ready,
        UNAVAILABLE => VariantAvailability::Unavailable,
        _ => VariantAvailability::Pending,
    }
}

/// First transition away from pending. Later calls do not log again.
pub fn note_missing_api() -> bool {
    REASON.store(REASON_API, Ordering::Release);
    STATE
        .compare_exchange(PENDING, UNAVAILABLE, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

pub fn note_load_ok() -> LoadSample {
    let previous = STATE.swap(READY, Ordering::AcqRel);
    REASON.store(0, Ordering::Release);
    let count = LOADS.fetch_add(1, Ordering::Relaxed) + 1;
    LoadSample {
        count,
        log: count == 1 || previous != READY,
    }
}

pub fn note_load_failed(reason: u64) -> LoadSample {
    let previous = STATE.swap(UNAVAILABLE, Ordering::AcqRel);
    REASON.store(reason, Ordering::Release);
    let count = LOADS.fetch_add(1, Ordering::Relaxed) + 1;
    LoadSample {
        count,
        log: count == 1 || previous != UNAVAILABLE,
    }
}

/// Sampled by the request path. Palutena and Shadow Flare do not share a counter.
pub fn observe_swap(hash: u64) -> u64 {
    let slot = if edge_reduced(hash).is_some() {
        &EDGE_SWAPS
    } else if ink_reduced(hash).is_some() {
        &INK_SWAPS
    } else if chrom_reduced(hash).is_some() {
        &CHROM_SWAPS
    } else {
        &SWAPS
    };
    slot.fetch_add(1, Ordering::Relaxed) + 1
}

pub fn note_edge_missing_api() -> bool {
    EDGE_REASON.store(REASON_API, Ordering::Release);
    EDGE_STATE
        .compare_exchange(PENDING, UNAVAILABLE, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

pub fn note_edge_load_ok() -> LoadSample {
    let previous = EDGE_STATE.swap(READY, Ordering::AcqRel);
    EDGE_REASON.store(0, Ordering::Release);
    let count = EDGE_LOADS.fetch_add(1, Ordering::Relaxed) + 1;
    LoadSample {
        count,
        log: count == 1 || previous != READY,
    }
}

pub fn note_edge_load_failed(reason: u64) -> LoadSample {
    let previous = EDGE_STATE.swap(UNAVAILABLE, Ordering::AcqRel);
    EDGE_REASON.store(reason, Ordering::Release);
    let count = EDGE_LOADS.fetch_add(1, Ordering::Relaxed) + 1;
    LoadSample {
        count,
        log: count == 1 || previous != UNAVAILABLE,
    }
}

pub fn forward(mode: FxMode, scope: bool, hash: u64, context: Option<Context>) -> u64 {
    if mode != FxMode::Reduced || !scope {
        return hash;
    }
    if let Some(reduced) = ink_reduced(hash) {
        return if ink_availability() == VariantAvailability::Ready {
            reduced
        } else {
            hash
        };
    }
    if let Some(reduced) = chrom_reduced(hash) {
        return if chrom_availability() == VariantAvailability::Ready {
            reduced
        } else {
            hash
        };
    }
    if hash == SOURCE {
        if availability() != VariantAvailability::Ready {
            return hash;
        }
        return match context.and_then(effect_context::profile) {
            Some(Profile::PalutenaUpAir) => REDUCED,
            _ => hash,
        };
    }
    let Some(reduced) = edge_reduced(hash) else {
        return hash;
    };
    if edge_availability() != VariantAvailability::Ready {
        return hash;
    }
    match context.and_then(effect_context::profile) {
        Some(Profile::EdgeSideB) => reduced,
        _ => hash,
    }
}

pub fn ink_availability() -> VariantAvailability {
    match INK_STATE.load(Ordering::Acquire) {
        READY => VariantAvailability::Ready,
        UNAVAILABLE => VariantAvailability::Unavailable,
        _ => VariantAvailability::Pending,
    }
}

pub fn note_ink_missing_api() -> bool {
    INK_REASON.store(REASON_API, Ordering::Release);
    INK_STATE
        .compare_exchange(PENDING, UNAVAILABLE, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

pub fn note_ink_load_ok() -> LoadSample {
    let previous = INK_STATE.swap(READY, Ordering::AcqRel);
    INK_REASON.store(0, Ordering::Release);
    let count = INK_LOADS.fetch_add(1, Ordering::Relaxed) + 1;
    LoadSample {
        count,
        log: count == 1 || previous != READY,
    }
}

pub fn chrom_availability() -> VariantAvailability {
    match CHROM_STATE.load(Ordering::Acquire) {
        READY => VariantAvailability::Ready,
        UNAVAILABLE => VariantAvailability::Unavailable,
        _ => VariantAvailability::Pending,
    }
}

pub fn note_chrom_missing_api() -> bool {
    CHROM_REASON.store(REASON_API, Ordering::Release);
    CHROM_STATE
        .compare_exchange(PENDING, UNAVAILABLE, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

pub fn note_chrom_load_ok() -> LoadSample {
    let previous = CHROM_STATE.swap(READY, Ordering::AcqRel);
    CHROM_REASON.store(0, Ordering::Release);
    let count = CHROM_LOADS.fetch_add(1, Ordering::Relaxed) + 1;
    LoadSample {
        count,
        log: count == 1 || previous != READY,
    }
}

pub fn note_chrom_load_failed(reason: u64) -> LoadSample {
    let previous = CHROM_STATE.swap(UNAVAILABLE, Ordering::AcqRel);
    CHROM_REASON.store(reason, Ordering::Release);
    let count = CHROM_LOADS.fetch_add(1, Ordering::Relaxed) + 1;
    LoadSample {
        count,
        log: count == 1 || previous != UNAVAILABLE,
    }
}

pub fn note_ink_load_failed(reason: u64) -> LoadSample {
    let previous = INK_STATE.swap(UNAVAILABLE, Ordering::AcqRel);
    INK_REASON.store(reason, Ordering::Release);
    let count = INK_LOADS.fetch_add(1, Ordering::Relaxed) + 1;
    LoadSample {
        count,
        log: count == 1 || previous != UNAVAILABLE,
    }
}

#[cfg(any(test, feature = "variant-test"))]
pub fn set_availability_for_test(state: VariantAvailability) {
    let raw = match state {
        VariantAvailability::Pending => PENDING,
        VariantAvailability::Ready => READY,
        VariantAvailability::Unavailable => UNAVAILABLE,
    };
    STATE.store(raw, Ordering::Release);
}

#[cfg(any(test, feature = "variant-test"))]
pub fn set_edge_availability_for_test(state: VariantAvailability) {
    let raw = match state {
        VariantAvailability::Pending => PENDING,
        VariantAvailability::Ready => READY,
        VariantAvailability::Unavailable => UNAVAILABLE,
    };
    EDGE_STATE.store(raw, Ordering::Release);
}

#[cfg(any(test, feature = "variant-test"))]
pub fn reset_for_test() {
    STATE.store(PENDING, Ordering::Release);
    REASON.store(0, Ordering::Release);
    LOADS.store(0, Ordering::Release);
    SWAPS.store(0, Ordering::Release);
    EDGE_STATE.store(PENDING, Ordering::Release);
    EDGE_REASON.store(0, Ordering::Release);
    EDGE_LOADS.store(0, Ordering::Release);
    EDGE_SWAPS.store(0, Ordering::Release);
    INK_STATE.store(PENDING, Ordering::Release);
    INK_REASON.store(0, Ordering::Release);
    INK_LOADS.store(0, Ordering::Release);
    INK_SWAPS.store(0, Ordering::Release);
    CHROM_STATE.store(PENDING, Ordering::Release);
    CHROM_REASON.store(0, Ordering::Release);
    CHROM_LOADS.store(0, Ordering::Release);
    CHROM_SWAPS.store(0, Ordering::Release);
}

#[cfg(any(test, feature = "variant-test"))]
pub fn set_ink_availability_for_test(state: VariantAvailability) {
    let raw = match state {
        VariantAvailability::Pending => PENDING,
        VariantAvailability::Ready => READY,
        VariantAvailability::Unavailable => UNAVAILABLE,
    };
    INK_STATE.store(raw, Ordering::Release);
}

#[cfg(any(test, feature = "variant-test"))]
pub fn set_chrom_availability_for_test(state: VariantAvailability) {
    let raw = match state {
        VariantAvailability::Pending => PENDING,
        VariantAvailability::Ready => READY,
        VariantAvailability::Unavailable => UNAVAILABLE,
    };
    CHROM_STATE.store(raw, Ordering::Release);
}

#[cfg(test)]
mod tests {
    use super::{
        source_file, CHROM_PATH_HASH, EDGE_PATH_HASH, INK_PATH_HASH, PATH_HASH, REDUCED, SOURCE,
    };
    use crate::effects::hash40;

    #[test]
    fn source_file_names_the_eff_and_leaves_sole_emitters_unmapped() {
        assert_eq!(source_file(SOURCE), PATH_HASH);
        assert_eq!(
            source_file(hash40("edge_shadowflare_bullet")),
            EDGE_PATH_HASH
        );
        assert_eq!(
            source_file(hash40("inkling_roller_splash_walk")),
            INK_PATH_HASH
        );
        assert_eq!(source_file(hash40("edge_shadowflare_aura")), 0);
        assert_eq!(source_file(hash40("chrom_mc_1")), CHROM_PATH_HASH);
        assert_eq!(source_file(hash40("chrom_mc_2hi")), 0);
    }

    #[test]
    fn cleanup_alias_follows_real_siblings_after_mode_changes() {
        use super::cleanup_alias;
        let start = hash40("edge_shadowflare_start");
        let hold = hash40("edge_shadowflare_hold");
        assert_eq!(
            cleanup_alias(start),
            Some(hash40("edge_shadowflare_start_reduced"))
        );
        assert_eq!(
            cleanup_alias(hold),
            Some(hash40("edge_shadowflare_hold_reduced"))
        );
        assert_eq!(
            cleanup_alias(hash40("edge_shadowflare_bullet")),
            Some(hash40("edge_shadowflare_bullet_reduced"))
        );
        assert_eq!(cleanup_alias(SOURCE), Some(REDUCED));
        assert_eq!(
            cleanup_alias(hash40("inkling_roller_splash_run")),
            Some(hash40("inkling_roller_splash_run_reduced"))
        );
        assert_eq!(
            cleanup_alias(hash40("chrom_mc_1")),
            Some(hash40("chrom_mc_1_reduced"))
        );
        assert_eq!(cleanup_alias(hash40("edge_shadowflare_aura")), None);
        assert_eq!(cleanup_alias(REDUCED), None);
        assert_eq!(
            cleanup_alias(hash40("edge_shadowflare_start_reduced")),
            None
        );
        assert_eq!(cleanup_alias(crate::effects::NULL_EFFECT), None);
        assert_eq!(cleanup_alias(0), None);
        assert_eq!(cleanup_alias(hash40("sys_dash_smoke")), None);
    }
}
