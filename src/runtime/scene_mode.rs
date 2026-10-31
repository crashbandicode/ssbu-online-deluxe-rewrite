//! Read-only waiting-room observation (0.9.0). Lobby practice in the
//! local-wireless and online waiting rooms can load any stage with extra
//! fighters, so stage setup also reads the scene-mode values that the
//! operator's Training Modpack fork and smash-gamepad-overlay v0.1.3 use to
//! recognise it (`auto_policy::is_waiting_practice`). No hook, no writes.
use super::memory;
use skyline::hooks::{getRegionAddress, Region};
use std::sync::atomic::{AtomicUsize, Ordering};

/// `app::smashball::is_training_mode` in 13.0.5 main
/// 21450C647B8C5940ED5B68218EA128FD3C5D597D. Its first instructions address
/// the same scene data, so a match pins the three offsets below to this build.
const PREDICATE: usize = 0x1671650;
const PREDICATE_CODE: [u8; 32] = [
    0xfd, 0x7b, 0xbf, 0xa9, 0xfd, 0x03, 0x00, 0x91, 0x08, 0xe5, 0x01, 0xf0, 0x08, 0x61, 0x13, 0x91,
    0x08, 0xfd, 0xdf, 0x08, 0x48, 0x01, 0x00, 0x36, 0xa8, 0xe2, 0x01, 0x90, 0x08, 0xc1, 0x46, 0x39,
];
const INITIALIZED: usize = 0x53144d8;
const MELEE: usize = 0x52c51b0;
const MODE: usize = 0x53050f0;

/// The text base once the predicate matched; 0 leaves the reads disarmed.
static BASE: AtomicUsize = AtomicUsize::new(0);

/// Arms the reads when the predicate matches 13.0.5; returns whether it did.
pub(super) fn init() -> bool {
    let mut predicate = 0usize;
    let symbol = b"_ZN3app9smashball16is_training_modeEv\0";
    unsafe {
        let base = getRegionAddress(Region::Text) as usize;
        let matched = skyline::nn::ro::LookupSymbol(&mut predicate, symbol.as_ptr()) == 0
            && predicate == base + PREDICATE
            && memory::accessible(predicate, PREDICATE_CODE.len(), false)
            && std::slice::from_raw_parts(predicate as *const u8, PREDICATE_CODE.len())
                == PREDICATE_CODE
            && [INITIALIZED, MELEE, MODE]
                .iter()
                .all(|offset| memory::accessible(base + offset, 4, false));
        if matched {
            BASE.store(base, Ordering::Release);
        }
        matched
    }
}

/// `(initialized, melee, mode)` while armed.
pub(super) fn current() -> Option<(bool, u8, u32)> {
    let base = BASE.load(Ordering::Acquire);
    if base == 0 {
        return None;
    }
    unsafe {
        Some((
            std::ptr::read_volatile((base + INITIALIZED) as *const u8) & 1 != 0,
            std::ptr::read_volatile((base + MELEE) as *const u8),
            std::ptr::read_volatile((base + MODE) as *const u32),
        ))
    }
}
