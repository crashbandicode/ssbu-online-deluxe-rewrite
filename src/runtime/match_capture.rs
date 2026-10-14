//! Opt-in match capture. Absent by default: with `CAPTURE_MATCH` in the data
//! directory at boot, the existing finite capture pair (`timing-<id>.csv`,
//! `api-<id>.csv`) is armed when a stage loads (Singles, Doubles or
//! Training) and ends when the game leaves it, with a ten-minute ceiling.
//! Both CSVs stream from the diagnostics thread while the match runs; no
//! hook, wait, priority, clock or task order changes. The flag also installs
//! the three read-only `CAPTURE_CSS` hooks so the trace carries dependency
//! waits and the UI job in a match.
//!
//! End signals, either one: the scene status leaves a stage for Inactive
//! (results screen, main menu), or the character-select update runs (the
//! Training CSS keeps the Training status, so the status alone would not
//! end a Training capture).
use super::{record, Code};
use crate::auto_policy::SceneStatus;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const FLAG: &str = "CAPTURE_MATCH";
/// `b` of code 106 `a=2` when the character-select update ended the capture.
const END_BY_CSS: u64 = 0xC55;

static FLAG_PRESENT: AtomicBool = AtomicBool::new(false);
static ARMED_CAPTURES: AtomicU32 = AtomicU32::new(0);
static ENDED_CAPTURES: AtomicU32 = AtomicU32::new(0);
/// The current capture's end was already recorded; cleared by the next stage.
static END_RECORDED: AtomicBool = AtomicBool::new(false);

/// Startup. Only reads the flag; the triggers are the existing scene hooks.
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

/// Capture label for a stage status.
pub(super) fn label(status: SceneStatus) -> &'static str {
    match status {
        SceneStatus::Singles => "match_singles",
        SceneStatus::Doubles => "match_doubles",
        SceneStatus::Training => "match_training",
        SceneStatus::Inactive => "match",
    }
}

/// The stage-setup hook classified a loading stage (every stage load, also
/// Training re-entered from its own character select).
pub(super) fn note_stage(status: SceneStatus) {
    if !flag_present() || !on_stage(status) {
        return;
    }
    END_RECORDED.store(false, Ordering::Release);
    if super::timing::request_match(status) {
        ARMED_CAPTURES.fetch_add(1, Ordering::Relaxed);
    }
    record(Code::MatchCapture, 1, status as u64);
}

fn end(reason: u64) {
    super::timing::end_match();
    if !END_RECORDED.swap(true, Ordering::AcqRel) {
        ENDED_CAPTURES.fetch_add(1, Ordering::Relaxed);
        record(Code::MatchCapture, 2, reason);
    }
}

/// Scene status changed (any scene hook): leaving a stage for Inactive ends
/// the running capture.
pub(super) fn note_scene(previous: SceneStatus, status: SceneStatus) {
    if flag_present() && on_stage(previous) && status == SceneStatus::Inactive {
        end(previous as u64);
    }
}

/// The game's character-select update ran: the stage is gone whatever the
/// status says. One atomic load per CSS frame while no capture runs.
pub(super) fn note_css() {
    if flag_present() && super::timing::match_active() {
        end(END_BY_CSS);
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
