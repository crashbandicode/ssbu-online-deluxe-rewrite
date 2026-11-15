//! Read-only native elapsed match time (13.0.5), in 60-Hz units.
//! Counter owner1572540 accumulates native rate/pause state; this is not an
//! unconditional engine-tick serial. Copy157d738 publishes object+3d8 here.
//! RE, 64-byte windows and normal/hitlag/peer-wait evidence: docs/GAME_SPEED.md.
use crate::auto_policy::SceneStatus;
use crate::game_speed::{speed_stage_eligible, StageEpoch};
use skyline::hooks::{getRegionAddress, Region};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

const ELAPSED: usize = 0x52e8b44;
const READY_SLOT: usize = 0x52b94f8;
const READY_GETTER: usize = 0x227ee10;

// Each native window was matched to build21450c647b8c5940ed5b68218ea128fd3c5d597d.
// New read-only pins; existing hook guards and their failure behavior are unchanged.
const OWNER_CODE: [u8; 64] = [
    0x0a, 0xac, 0x40, 0x29, 0x4a, 0x01, 0x08, 0x6b, 0x4a, 0xc1, 0x9f, 0x1a, 0xe9, 0xdf, 0x92, 0x52,
    0x29, 0x29, 0xa0, 0x72, 0x0a, 0x04, 0x00, 0xb9, 0x6a, 0x01, 0x08, 0x0b, 0x5f, 0x01, 0x09, 0x6b,
    0x4a, 0xb1, 0x89, 0x1a, 0x0a, 0x08, 0x00, 0xb9, 0x0a, 0x04, 0x40, 0x39, 0xca, 0x01, 0x00, 0x34,
    0x0a, 0x0c, 0x40, 0xb9, 0x48, 0x01, 0x08, 0x0b, 0x1f, 0x01, 0x09, 0x6b, 0x08, 0xb1, 0x89, 0x1a,
];
const COPY_CODE: [u8; 64] = [
    0xfd, 0x43, 0x02, 0x91, 0x08, 0x10, 0x40, 0xf9, 0x0c, 0xd8, 0x43, 0xb9, 0x89, 0xad, 0x98, 0x52,
    0x29, 0x00, 0xa0, 0x72, 0xed, 0x03, 0x1f, 0x32, 0xea, 0x03, 0x1d, 0x32, 0x4b, 0x18, 0x87, 0x52,
    0xe8, 0x03, 0x27, 0x1e, 0xfb, 0x03, 0x00, 0xaa, 0x0c, 0x69, 0x29, 0xb8, 0xc9, 0x1d, 0x87, 0x52,
    0x0d, 0x89, 0x01, 0x39, 0x0a, 0x39, 0x02, 0x39, 0xf3, 0x03, 0x1f, 0xaa, 0x0d, 0x69, 0x2b, 0x38,
];
const READY_CODE: [u8; 64] = [
    0xc8, 0x81, 0x01, 0xf0, 0x08, 0x7d, 0x42, 0xf9, 0x08, 0x01, 0x40, 0xf9, 0x00, 0x49, 0x43, 0x39,
    0xc0, 0x03, 0x5f, 0xd6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xfd, 0x7b, 0xbf, 0xa9, 0xfd, 0x03, 0x00, 0x91, 0x08, 0x80, 0x5f, 0xf8, 0x08, 0xd1, 0x40, 0xf9,
    0x00, 0x29, 0x40, 0xf9, 0x08, 0x00, 0x40, 0xf9, 0x08, 0x4d, 0x40, 0xf9, 0xe1, 0x03, 0x04, 0x32,
];

static ELAPSED_BASE: AtomicUsize = AtomicUsize::new(0);
static CSS_LIFECYCLE: AtomicBool = AtomicBool::new(false);
static ELAPSED_STAGE: StageEpoch = StageEpoch::new();

/// Called only after the existing guarded CSS hook has actually installed.
/// Without that invalidation path, a retained native battle tuple is insufficient.
pub(super) fn note_elapsed_css_installed() {
    CSS_LIFECYCLE.store(true, Ordering::Release);
}

pub(super) fn begin_elapsed_stage() -> u64 {
    ELAPSED_STAGE.speed_begin_stage()
}

pub(super) fn invalidate_elapsed_stage() {
    ELAPSED_STAGE.speed_invalidate_stage();
}

/// Arm only the setup attempt whose ticket still belongs to the current stage.
pub(super) fn arm_elapsed_stage(ticket: u64, mode: Option<(bool, u8, u32)>, status: SceneStatus) {
    let match_stage = matches!(status, SceneStatus::Singles | SceneStatus::Doubles);
    if mode.is_some_and(|(initialized, melee, word)| {
        // GO is tested on every draw. Here only the new stage's identity is offered.
        speed_stage_eligible(initialized, melee, word, match_stage, true)
    }) {
        ELAPSED_STAGE.speed_arm_stage(ticket);
    }
}

unsafe fn elapsed_code_matches(base: usize, offset: usize, code: &[u8; 64]) -> bool {
    super::memory::accessible(base + offset, code.len(), false)
        && std::slice::from_raw_parts((base + offset) as *const u8, code.len()) == code
}

/// No hook or game write. Called with scene-mode installation, before stage entry.
pub(super) fn init_elapsed_reader() {
    let mut getter = 0usize;
    let symbol = c"_ZN3app14sv_information11is_ready_goEv";
    let base = unsafe { getRegionAddress(Region::Text) as usize };
    let matched = unsafe {
        skyline::nn::ro::LookupSymbol(&mut getter, symbol.as_ptr().cast()) == 0
            && getter == base + READY_GETTER
            && elapsed_code_matches(base, 0x15725a0, &OWNER_CODE)
            && elapsed_code_matches(base, 0x157d710, &COPY_CODE)
            && elapsed_code_matches(base, READY_GETTER, &READY_CODE)
            && super::memory::accessible(base + ELAPSED, 4, false)
            && super::memory::accessible(base + READY_SLOT, 8, false)
    };
    if matched {
        ELAPSED_BASE.store(base, Ordering::Release);
    } else {
        super::record(super::Code::Compatibility, 4, ELAPSED as u64);
    }
}

/// The exported native getter's two read-only indirections, never a saved heap object.
unsafe fn elapsed_ready_go(base: usize) -> bool {
    let first = std::ptr::read_volatile((base + READY_SLOT) as *const usize);
    if !first.is_multiple_of(8) || !super::memory::accessible(first, 8, false) {
        return false;
    }
    let state = std::ptr::read_volatile(first as *const usize);
    let Some(flag) = state.checked_add(0xd2) else {
        return false;
    };
    state.is_multiple_of(8)
        && super::memory::accessible(flag, 1, false)
        && std::ptr::read_volatile(flag as *const u8) != 0
}

pub(super) fn elapsed_token_visible(token: u64) -> bool {
    token != 0 && ELAPSED_STAGE.speed_stage_token() == Some(token)
}

/// Invoked once per existing ImGui draw under UI.try_lock, including Hidden.
/// Failed proof clears the model's warmup; neither stored connection nor presents
/// can supply eligibility. Recheck the generation after the native memory reads.
pub(super) fn elapsed_observation() -> (u64, Option<u32>) {
    let Some(token) = ELAPSED_STAGE.speed_stage_token() else {
        return (0, None);
    };
    let base = ELAPSED_BASE.load(Ordering::Acquire);
    if base == 0 || !CSS_LIFECYCLE.load(Ordering::Acquire) {
        return (token, None);
    }
    let snapshot = super::auto_driver::snapshot();
    let match_stage =
        snapshot.ready && matches!(snapshot.status, SceneStatus::Singles | SceneStatus::Doubles);
    let eligible = super::scene_mode::current().is_some_and(|(initialized, melee, mode)| {
        // Current native GO is checked separately below. Snapshot.ready is hook readiness.
        speed_stage_eligible(initialized, melee, mode, match_stage, true)
    });
    if !eligible || !unsafe { elapsed_ready_go(base) } {
        return (token, None);
    }
    let elapsed = unsafe { std::ptr::read_volatile((base + ELAPSED) as *const u32) };
    if ELAPSED_STAGE.speed_stage_token() != Some(token) {
        return (0, None);
    }
    (token, Some(elapsed))
}
