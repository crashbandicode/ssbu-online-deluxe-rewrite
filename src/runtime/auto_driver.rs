//! Applies the pure Auto decision to the E2 request. Scene hooks and presents
//! call this; it does not write config.ini.

use super::{record, settings, Code};
use crate::auto_policy::{self, Connection, Platform, Scene, SceneStatus, Selection};
use crate::config::RenderMode;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};

static SCENE: AtomicU8 = AtomicU8::new(0);
static READY: AtomicBool = AtomicBool::new(false);
/// Latency-hook reachability for the online context only. It does not
/// classify a match.
static MATCH_MARKED: AtomicBool = AtomicBool::new(false);
static CLEANUP: AtomicU64 = AtomicU64::new(0);
/// Tick of the Singles/Doubles stage load under a menu-set Quickplay; 0 idle.
static OFFLINE_CHECK: AtomicU64 = AtomicU64::new(0);
/// Pseudo-source of the demotion in the context log (not a hook address).
const OFFLINE_CHECK_SOURCE: u64 = 0x0ff_11e;
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

/// The exported online bit: an online connection, or a match the online-latency
/// hook reached. False until the scene tracker is ready.
pub(crate) fn context_online() -> bool {
    ready() && (connection().is_online() || MATCH_MARKED.load(Ordering::Acquire))
}

/// The online-latency hook reached a match. The exported online context
/// treats that as online, and Auto treats a stage the hooks left Offline as an
/// online match (the console ran two Quickplay matches on 2026-10-14 with the
/// connection still Offline).
/// The classified connection itself is kept; nothing assumes Quickplay for
/// the UI.
pub(crate) fn mark_match() {
    if !ready() {
        return;
    }
    let previous = MATCH_MARKED.swap(true, Ordering::AcqRel);
    // The match is online: the stale-Quickplay check has its answer.
    OFFLINE_CHECK.store(0, Ordering::Release);
    if !previous && !connection().is_online() {
        record(Code::OnlineContext, 1, 0x16ccab8);
        let _ = current_request();
    }
}

/// Main-menu entry (`0x235aaa0`). Quickplay survives it: background
/// matchmaking passes through the main menu and starts the match from there.
/// Any other connection resets to Offline. Code 107 `a=1` records a kept
/// Quickplay.
pub(crate) fn note_main_menu(source: u64) {
    let (connection, _) = scene_pair();
    let next = auto_policy::main_menu_connection(connection);
    if next == connection && connection == Connection::Quickplay {
        record(Code::SceneHold, 1, source);
        set_scene(None, Some(SceneStatus::Inactive), source);
    } else {
        set_scene(Some(next), Some(SceneStatus::Inactive), source);
    }
}

/// Diagnostics thread, every 500 ms: a Singles/Doubles stage that has run
/// five seconds under a menu-set Quickplay without the online-latency hook is
/// an offline match; demote the connection so Auto stops treating it as
/// online. Code 107 `a=2`, context source `0x0ff11e`.
pub(crate) fn poll() {
    let armed = OFFLINE_CHECK.load(Ordering::Acquire);
    if armed == 0 {
        return;
    }
    let (connection, status) = scene_pair();
    let now = unsafe { skyline::nn::os::GetSystemTick() as u64 };
    if !auto_policy::offline_match_due(
        connection,
        status,
        MATCH_MARKED.load(Ordering::Acquire),
        armed,
        now,
    ) {
        if !matches!(status, SceneStatus::Singles | SceneStatus::Doubles)
            || connection != Connection::Quickplay
        {
            OFFLINE_CHECK.store(0, Ordering::Release);
        }
        return;
    }
    OFFLINE_CHECK.store(0, Ordering::Release);
    record(Code::SceneHold, 2, now.wrapping_sub(armed));
    set_scene(Some(Connection::Offline), None, OFFLINE_CHECK_SOURCE);
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
    if status == SceneStatus::Training
        || connection != previous_connection
        || next_connection == Some(Connection::Offline)
    {
        // A retained result belongs to its connection; do not carry it into
        // another online mode's lobby or an offline/Training scene.
        super::clear_auto_buffer();
    }
    if next_connection == Some(Connection::Offline) {
        let was_marked = MATCH_MARKED.swap(false, Ordering::AcqRel);
        if was_marked && !previous_connection.is_online() {
            record(Code::OnlineContext, 0, source);
        }
    } else if matches!(previous_status, SceneStatus::Singles | SceneStatus::Doubles)
        && !matches!(status, SceneStatus::Singles | SceneStatus::Doubles)
    {
        // Keep the result for CSS/results, but reject late network callbacks.
        super::end_auto_buffer_match();
    }
    if connection == previous_connection && status == previous_status {
        return;
    }
    // 0.9.0: an earlier match's mark expires with the shared online session,
    // so a stage entered after that session ended proves itself again.
    // Code 107 `a=3`.
    let entering_match = status != previous_status
        && matches!(status, SceneStatus::Singles | SceneStatus::Doubles);
    if auto_policy::carried_mark_expired(
        connection,
        entering_match,
        MATCH_MARKED.load(Ordering::Acquire),
        super::online_session::live_at(super::online_session::tick()),
    ) && MATCH_MARKED.swap(false, Ordering::AcqRel)
    {
        record(Code::SceneHold, 3, source);
    }
    // A match stage under a menu-set Quickplay starts the stale check; any
    // other status ends it.
    if matches!(status, SceneStatus::Singles | SceneStatus::Doubles)
        && connection == Connection::Quickplay
        && !MATCH_MARKED.load(Ordering::Acquire)
    {
        if status != previous_status {
            OFFLINE_CHECK.store(
                unsafe { skyline::nn::os::GetSystemTick() as u64 }.max(1),
                Ordering::Release,
            );
        }
    } else {
        OFFLINE_CHECK.store(0, Ordering::Release);
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
        super::capture::note_scene(previous_status, status);
        #[cfg(feature = "coverage")]
        super::coverage::note_scene(previous_status as u8, status as u8);
    }
    store_scene(connection, status);
    if previous_connection.is_online() != connection.is_online() {
        record(
            Code::OnlineContext,
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
