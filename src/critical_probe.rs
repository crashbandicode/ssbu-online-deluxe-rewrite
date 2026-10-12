//! Diagnostic allowlist for Special Zoom background requests.
//!
//! `PROBE_CRITICAL_SCREEN` is a marker file latched once when effect hooks
//! install. It is not a config or menu setting. Suppressing the names below
//! is a capture aid only: it is not a performance measurement and it does not
//! prove online parity. Camera, hitstop, slowdown, hitboxes, articles, and
//! RNG are outside this module.

use crate::config::FxMode;
use crate::effects::sorted_hashes;

/// Marker file under the plugin data directory. Absent means the probe is off.
pub const MARKER_FILE: &str = "PROBE_CRITICAL_SCREEN";

/// Startup observations kept before a finite API trace is active.
pub const STARTUP_LOG_LIMIT: u64 = 32;

/// `req_screen` boundary.
pub const PATH_SCREEN: u64 = 1;
/// `req_screen_system` boundary.
pub const PATH_SYSTEM: u64 = 2;
/// Ordinary effect `policy_hash`, including the On early return.
pub const PATH_ORDINARY: u64 = 3;

const PATH_MASK: u64 = 0xff;
const SUPPRESSED_BIT: u64 = 1 << 8;

/// Exact peer visual names. No prefix or wildcard matching.
const NAMES: &[&str] = &[
    "bg_criticalhit",
    "bg_criticalhit_r",
    "bg_criticalhit_b",
    "bg_criticalhit_y",
    "bg_criticalhit_g",
    "bg_criticalhit_o",
    "bg_criticalhit_c",
    "bg_criticalhit_m",
    "bg_criticalhit_p",
    "sys_bg_criticalhit",
    "sys_bg_criticalhit_r",
    "sys_bg_criticalhit_b",
    "sys_bg_criticalhit_y",
    "sys_bg_criticalhit_g",
    "sys_bg_criticalhit_o",
    "sys_bg_criticalhit_c",
    "sys_bg_criticalhit_m",
    "sys_bg_criticalhit_p",
];

const HASHES: [u64; NAMES.len()] = sorted_hashes(NAMES);

pub fn is_listed(hash: u64) -> bool {
    HASHES.binary_search(&hash).is_ok()
}

/// Marker plus FX Off plus an allowing scope. On, Reduced, and a blocked
/// scope stay on the caller's existing path.
pub fn suppress(marker: bool, mode: FxMode, scope_allows: bool, hash: u64) -> bool {
    marker && mode == FxMode::Off && scope_allows && is_listed(hash)
}

/// `b` for [`crate::diagnostics::Code::FxCriticalRequest`].
/// Bits 0..7 are the path. Bit 8 is set when this probe suppressed the request.
pub const fn detail(path: u64, suppressed: bool) -> u64 {
    (path & PATH_MASK) | if suppressed { SUPPRESSED_BIT } else { 0 }
}

/// First [`STARTUP_LOG_LIMIT`] matches, then only while a finite capture is active.
pub const fn should_log(ordinal: u64, capture_active: bool) -> bool {
    ordinal > 0 && (ordinal <= STARTUP_LOG_LIMIT || capture_active)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects;

    const COLORS: [&str; 8] = ["r", "b", "y", "g", "o", "c", "m", "p"];

    #[test]
    fn allowlist_is_nine_names_on_both_routes_and_nothing_nearby() {
        assert_eq!(NAMES.len(), 18);
        assert!(is_listed(effects::hash40("bg_criticalhit")));
        assert!(is_listed(effects::hash40("sys_bg_criticalhit")));
        for color in COLORS {
            assert!(is_listed(effects::hash40(&format!(
                "bg_criticalhit_{color}"
            ))));
            assert!(is_listed(effects::hash40(&format!(
                "sys_bg_criticalhit_{color}"
            ))));
        }
        for name in [
            "bg_finishhit",
            "bg_final",
            "edge_fire3_screen2",
            "sys_bg_black",
            "sys_hit_normal",
            "bg_criticalhit_rb",
            "bg_critical",
            "sys_bg_criticalhit_extra",
            "BG_CRITICALHIT",
            "null",
        ] {
            assert!(!is_listed(effects::hash40(name)), "{name}");
        }
        assert!(!is_listed(0));
        for index in 1..HASHES.len() {
            assert!(HASHES[index] > HASHES[index - 1]);
        }
    }

    #[test]
    fn only_off_scope_and_marker_suppress_listed_names() {
        let listed = effects::hash40("bg_criticalhit_b");
        let system = effects::hash40("sys_bg_criticalhit_p");
        let finish = effects::hash40("bg_finishhit");
        let black = effects::hash40("sys_bg_black");
        let screen = effects::hash40("edge_fire3_screen2");
        let hit = effects::hash40("sys_hit_normal");
        let final_bg = effects::hash40("bg_final");
        for hash in [listed, system] {
            assert!(suppress(true, FxMode::Off, true, hash));
            assert!(!suppress(false, FxMode::Off, true, hash));
            assert!(!suppress(true, FxMode::On, true, hash));
            assert!(!suppress(true, FxMode::Reduced, true, hash));
            assert!(!suppress(true, FxMode::Off, false, hash));
        }
        for hash in [
            finish,
            black,
            screen,
            hit,
            final_bg,
            0,
            effects::NULL_EFFECT,
        ] {
            assert!(!suppress(true, FxMode::Off, true, hash));
            assert!(!suppress(false, FxMode::On, false, hash));
        }
    }

    #[test]
    fn detail_and_startup_sample_keep_path_and_capture_anchor() {
        assert_eq!(detail(PATH_SCREEN, false), 1);
        assert_eq!(detail(PATH_SYSTEM, true), 2 | SUPPRESSED_BIT);
        assert_eq!(detail(PATH_ORDINARY, false), 3);
        assert_eq!(detail(PATH_ORDINARY, true) & PATH_MASK, PATH_ORDINARY);
        assert!(should_log(1, false));
        assert!(should_log(STARTUP_LOG_LIMIT, false));
        assert!(!should_log(STARTUP_LOG_LIMIT + 1, false));
        assert!(should_log(STARTUP_LOG_LIMIT + 1, true));
        assert!(!should_log(0, true));
    }
}
