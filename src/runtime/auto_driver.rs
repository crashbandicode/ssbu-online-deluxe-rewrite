//! Applies the pure Auto decision to the E2 request. Scene hooks and presents
//! call this; it does not write config.ini.

use super::{record, settings, Code};
use crate::auto_policy::{self, Connection, Platform, Scene, SceneStatus, Selection};
use crate::config::RenderMode;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};

static SCENE: AtomicU8 = AtomicU8::new(0);
static READY: AtomicBool = AtomicBool::new(false);
/// Latency-hook reachability for FX only. It does not classify a match.
static MATCH_MARKED: AtomicBool = AtomicBool::new(false);
static CLEANUP: AtomicU64 = AtomicU64::new(0);
static SELECTION_GENERATION: AtomicU64 = AtomicU64::new(0);
static LAST_CONTEXT: AtomicU64 = AtomicU64::new(u64::MAX);
static LAST_REQUEST: AtomicU64 = AtomicU64::new(u64::MAX);

const CLEANUP_ACTIVE: u64 = 1 << 63;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Snapshot {
    pub request: RenderMode,
    pub selected: RenderMode,
    pub auto_profile: bool,
    pub platform: Platform,
    pub ready: bool,
    pub connection: Connection,
    pub status: SceneStatus,
    pub cleanup_hold: bool,
    pub selection_generation: u64,
}

pub(crate) fn ready() -> bool {
    READY.load(Ordering::Acquire)
}

pub(crate) fn fx_online() -> bool {
    ready() && (connection().is_online() || MATCH_MARKED.load(Ordering::Acquire))
}

/// The online-latency hook reached a match. FX treats that as online, and
/// Auto treats a stage the hooks left Offline as an online match (the console
/// ran two Quickplay matches on 2026-10-14 with the connection still Offline).
/// The classified connection itself is kept; nothing assumes Quickplay for
/// the UI.
pub(crate) fn mark_match() {
    if !ready() {
        return;
    }
    let previous = MATCH_MARKED.swap(true, Ordering::AcqRel);
    if !previous && !connection().is_online() {
        record(Code::FxOnlineContext, 1, 0x16ccab8);
        let _ = current_request();
    }
}

fn scene_pair() -> (Connection, SceneStatus) {
    let packed = SCENE.load(Ordering::Acquire);
    (
        Connection::from_byte(packed & 0b11),
        SceneStatus::from_byte(packed >> 2),
    )
}

pub(crate) fn connection() -> Connection {
    scene_pair().0
}

fn store_scene(connection: Connection, status: SceneStatus) {
    let packed = (connection as u8) | (status as u8) << 2;
    SCENE.store(packed, Ordering::Release);
    super::hang_trace::note_scene(packed);
}

fn cleanup() -> (bool, u64) {
    let word = CLEANUP.load(Ordering::Acquire);
    (word & CLEANUP_ACTIVE != 0, word & !CLEANUP_ACTIVE)
}

fn store_cleanup(active: bool, generation: u64) {
    let mut word = generation & !CLEANUP_ACTIVE;
    if active {
        word |= CLEANUP_ACTIVE;
    }
    CLEANUP.store(word, Ordering::Release);
}

fn selection() -> Selection {
    let config = settings();
    let (cleanup_active, cleanup_generation) = cleanup();
    Selection {
        auto_profile: config.auto_profile,
        manual: config.render_mode,
        platform: match config.platform {
            crate::config::Platform::Console => Platform::Console,
            crate::config::Platform::Emulator => Platform::Emulator,
        },
        selection_generation: SELECTION_GENERATION.load(Ordering::Acquire),
        cleanup_active,
        cleanup_generation,
    }
}

pub(crate) fn snapshot() -> Snapshot {
    let (connection, status) = scene_pair();
    let scene = Scene {
        ready: ready(),
        connection,
        status,
        marked_online: MATCH_MARKED.load(Ordering::Acquire),
    };
    let selection = selection();
    let applied = auto_policy::decide(scene, selection);
    Snapshot {
        request: applied.request,
        selected: selection.manual,
        auto_profile: selection.auto_profile,
        platform: selection.platform,
        ready: scene.ready,
        connection: scene.connection,
        status: scene.status,
        cleanup_hold: applied.cleanup_hold,
        selection_generation: selection.selection_generation,
    }
}

/// Explicit menu choice. Does not write the INI. Always advances the selection
/// generation, including when the mode is unchanged, so a cleanup hold cannot
/// ignore the action. Auto stays as it is; the menu cycle turns it off first.
pub(crate) fn select_user_profile(mode: RenderMode) {
    super::RENDER_MODE.store(mode as u8, Ordering::Release);
    bump_selection();
    let _ = current_request();
}

pub(crate) fn set_auto_profile(enabled: bool) {
    super::AUTO_PROFILE.store(enabled, Ordering::Release);
    let _ = current_request();
}

pub(crate) fn note_published(mode_changed: bool, auto_changed: bool) {
    if mode_changed || auto_changed {
        bump_selection();
    }
    let _ = current_request();
}

fn bump_selection() {
    SELECTION_GENERATION.fetch_add(1, Ordering::AcqRel);
}

pub(crate) fn set_ready(ready: bool, source: u64) {
    let previous = READY.swap(ready, Ordering::AcqRel);
    if previous != ready {
        log_context(source);
        let _ = current_request();
    }
}

pub(crate) fn set_scene(
    next_connection: Option<Connection>,
    next_status: Option<SceneStatus>,
    source: u64,
) {
    // Every scene hook, changed or not, proves the game is past initialization.
    super::dispatcher_backoff::note_scene();
    let (previous_connection, previous_status) = scene_pair();
    let connection = next_connection.unwrap_or(previous_connection);
    let status = next_status.unwrap_or(previous_status);
    if next_connection == Some(Connection::Offline) {
        let was_marked = MATCH_MARKED.swap(false, Ordering::AcqRel);
        if was_marked && !previous_connection.is_online() {
            record(Code::FxOnlineContext, 0, source);
        }
    }
    if connection == previous_connection && status == previous_status {
        return;
    }
    if status != previous_status {
        let (active, generation) = cleanup();
        let (active, generation) = auto_policy::transition_cleanup(
            previous_status,
            status,
            SELECTION_GENERATION.load(Ordering::Acquire),
            active,
            generation,
        );
        store_cleanup(active, generation);
        super::match_capture::note_scene(previous_status, status);
    }
    store_scene(connection, status);
    if previous_connection.is_online() != connection.is_online() {
        record(
            Code::FxOnlineContext,
            u64::from(connection.is_online()),
            source,
        );
    }
    log_context(source);
    let _ = current_request();
    super::fighter_main::reset_reports();
}

pub(crate) fn current_request() -> RenderMode {
    let snap = snapshot();
    super::renderer::store_request(snap.request);
    let packed = request_bits(&snap);
    if LAST_REQUEST.swap(packed, Ordering::AcqRel) != packed {
        record(Code::AutoRequest, packed, snap.selection_generation);
    }
    snap.request
}

fn log_context(source: u64) {
    let (connection, status) = scene_pair();
    let packed =
        u64::from(connection as u8) | (u64::from(status as u8) << 8) | (u64::from(ready()) << 16);
    if LAST_CONTEXT.swap(packed, Ordering::AcqRel) != packed {
        record(Code::AutoContext, packed, source);
    }
}

fn request_bits(snap: &Snapshot) -> u64 {
    u64::from(snap.request as u8)
        | (u64::from(snap.selected as u8) << 8)
        | (u64::from(snap.auto_profile) << 16)
        | (u64::from(snap.platform as u8) << 17)
        | (u64::from(snap.cleanup_hold) << 18)
        | (u64::from(snap.ready) << 19)
}
