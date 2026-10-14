//! Opt-in match capture. Absent by default: with `CAPTURE_MATCH` in the data
//! directory at boot, the existing finite capture pair (`timing-<id>.csv`,
//! `api-<id>.csv`) is armed when the scene enters a stage (Singles, Doubles
//! or Training) and ends when it leaves it, with a ten-minute ceiling. Both
//! CSVs stream from the diagnostics thread while the match runs; no hook,
//! wait, priority, clock or task order changes. The flag also installs the
//! three read-only `CAPTURE_CSS` hooks so the trace carries dependency waits
//! and the UI job in a match.
use super::{record, Code};
use crate::auto_policy::SceneStatus;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const FLAG: &str = "CAPTURE_MATCH";

static FLAG_PRESENT: AtomicBool = AtomicBool::new(false);
static ARMED_CAPTURES: AtomicU32 = AtomicU32::new(0);
static ENDED_CAPTURES: AtomicU32 = AtomicU32::new(0);

/// Startup. Only reads the flag; the hooks it relies on are the scene hooks.
pub(super) fn install() {
    let present = std::path::Path::new(super::DATA_DIR).join(FLAG).is_file();
    FLAG_PRESENT.store(present, Ordering::Release);
    record(Code::MatchCapture, 0, u64::from(present));
    if present {
        skyline::println!("[Deluxe] match capture flag present; arming on stage entry");
    }
}

pub(super) fn flag_present() -> bool {
    FLAG_PRESENT.load(Ordering::Acquire)
}

fn on_stage(status: SceneStatus) -> bool {
    matches!(
        status,
        SceneStatus::Singles | SceneStatus::Doubles | SceneStatus::Training
    )
}

/// Capture label for a stage status. Only called for `on_stage` statuses.
pub(super) fn label(status: SceneStatus) -> &'static str {
    match status {
        SceneStatus::Singles => "match_singles",
        SceneStatus::Doubles => "match_doubles",
        SceneStatus::Training => "match_training",
        SceneStatus::Inactive => "match",
    }
}

/// Scene status changed (any scene hook). Entering a stage from Inactive arms
/// a capture; leaving a stage for Inactive ends the running one.
pub(super) fn note_scene(previous: SceneStatus, status: SceneStatus) {
    if !flag_present() || previous == status {
        return;
    }
    if previous == SceneStatus::Inactive && on_stage(status) {
        if super::timing::request_match(status) {
            ARMED_CAPTURES.fetch_add(1, Ordering::Relaxed);
        }
        record(Code::MatchCapture, 1, status as u64);
    } else if on_stage(previous) && status == SceneStatus::Inactive {
        super::timing::end_match();
        ENDED_CAPTURES.fetch_add(1, Ordering::Relaxed);
        record(Code::MatchCapture, 2, previous as u64);
    }
}

pub(super) fn status_label() -> String {
    if !flag_present() {
        return "Off (no CAPTURE_MATCH)".to_string();
    }
    format!(
        "Armed; {} capture(s) on stage entry, {} ended",
        ARMED_CAPTURES.load(Ordering::Relaxed),
        ENDED_CAPTURES.load(Ordering::Relaxed)
    )
}
