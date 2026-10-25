mod adaptive_drs;
mod api_trace;
mod auto_driver;
mod capture;
mod cells;
mod clock_probe;
mod connection_transport;
#[cfg(feature = "coverage")]
mod coverage;
mod css_capture;
mod dispatcher_backoff;
mod fighter_main;
mod fighter_probe;
mod hang_trace;
mod hooks;
mod imgui;
mod imgui_stats;
mod input;
mod logging;
mod memory;
mod native_banner;
mod nvn;
mod online;
mod overlay_host;
mod pia_rtt;
mod plugin_export;
mod render_sequence;
mod render_ultra;
mod renderer;
mod resolution;
mod response_probe;
mod ui;

use crate::{
    config::Config,
    diagnostics::{Code, Recorder},
};
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU64, AtomicU8, Ordering};

pub(super) static RECORDER: Recorder = Recorder::new();
static RENDER_MODE: AtomicU8 = AtomicU8::new(0);
static AUTO_PROFILE: AtomicBool = AtomicBool::new(true);
static PLATFORM: AtomicU8 = AtomicU8::new(0);
static INPUT_BUFFER: AtomicI8 = AtomicI8::new(-1);
/// A match-scoped observation of the buffer the network update actually reads.
/// No game pointer survives an observation callback.
static AUTO_BUFFER: crate::auto_buffer::AutoBufferState =
    crate::auto_buffer::AutoBufferState::new();
static STEALTH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static STREAMER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// `ultra_resolution` (0.5.3): `UltraResolution::to_byte` (0 = 576p, 1 = 720p,
/// 2 = 1080p since 0.5.6).
static ULTRA_RES: AtomicU8 = AtomicU8::new(0);
/// `hide_pages` (0.5.3) as bits: 1 plugins, 2 own pages, 4 performance.
static HIDE_PAGES: AtomicU8 = AtomicU8::new(0);
/// `perf_view` (0.5.3): true = number, false = graph.
static PERF_NUMBER: AtomicBool = AtomicBool::new(false);
/// `early_present` (0.5.3): present at the flush in the render-opts profiles.
static EARLY_PRESENT: AtomicBool = AtomicBool::new(true);
/// `dispatcher_backoff` (0.5.3): let the installed backoff hook sleep.
static BACKOFF_ENABLED: AtomicBool = AtomicBool::new(true);
/// `vanilla_plus` (0.5.6): present at the flush in Vanilla too. Live; the
/// L+R+X chord toggles it.
static VANILLA_PLUS: AtomicBool = AtomicBool::new(true);
/// `hints` (0.6.0): the chord hint line under the Settings and Window tables.
static HINTS: AtomicBool = AtomicBool::new(true);
static DRS_MOVES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static DRS_ADAPTIVE: AtomicBool = AtomicBool::new(false);
static DRS_THRESHOLD: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(1750);
static DRS_TRIGGER_FRAMES: AtomicU8 = AtomicU8::new(6);
static DRS_RELEASE_FRAMES: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(90);
static CONFIG_GENERATION: AtomicU64 = AtomicU64::new(0);
pub(super) const DATA_DIR: &str = "sd:/ultimate/ssbu_online_deluxe_rewrite";

pub(super) fn begin_auto_buffer_match() {
    AUTO_BUFFER.begin_match(INPUT_BUFFER.load(Ordering::Acquire));
}

pub(super) fn end_auto_buffer_match() {
    AUTO_BUFFER.end_match();
}

pub(super) fn clear_auto_buffer() {
    AUTO_BUFFER.clear();
}

pub(super) fn invalidate_manual_auto_buffer() {
    AUTO_BUFFER.invalidate_manual();
}

pub(super) fn auto_buffer_observation() -> Option<crate::auto_buffer::ObservationToken> {
    AUTO_BUFFER.observation_token()
}

pub(super) fn observe_auto_buffer(token: crate::auto_buffer::ObservationToken, frames: u8) {
    if AUTO_BUFFER.observe(token, frames) {
        record(Code::AutoBufferObserved, u64::from(frames), 0x16d81a8);
    }
}

/// Current-match count, or its result until the next stage starts. Unknown
/// during a new match's setup or after a manual selection.
pub(super) fn auto_buffer() -> Option<u8> {
    AUTO_BUFFER.value()
}

pub(super) fn saved_render_mode() -> crate::config::RenderMode {
    crate::config::RenderMode::from_byte(RENDER_MODE.load(Ordering::Acquire))
}

pub(super) fn settings() -> Config {
    Config {
        input_buffer: INPUT_BUFFER.load(Ordering::Acquire),
        render_mode: saved_render_mode(),
        auto_profile: AUTO_PROFILE.load(Ordering::Acquire),
        platform: if PLATFORM.load(Ordering::Acquire) == crate::config::Platform::Emulator as u8 {
            crate::config::Platform::Emulator
        } else {
            crate::config::Platform::Console
        },
        stealth: STEALTH.load(Ordering::Acquire),
        streamer: STREAMER.load(Ordering::Acquire),
        drs_moves: drs_moves(),
        drs_adaptive: adaptive_settings(),
        ultra_resolution: crate::config::UltraResolution::from_byte(
            ULTRA_RES.load(Ordering::Acquire),
        ),
        hide_pages: {
            let bits = HIDE_PAGES.load(Ordering::Acquire);
            crate::config::HiddenPages {
                plugins: bits & 1 != 0,
                own_pages: bits & 2 != 0,
                performance: bits & 4 != 0,
            }
        },
        perf_view: if PERF_NUMBER.load(Ordering::Acquire) {
            crate::config::PerfView::Number
        } else {
            crate::config::PerfView::Graph
        },
        early_present: EARLY_PRESENT.load(Ordering::Acquire),
        dispatcher_backoff: BACKOFF_ENABLED.load(Ordering::Acquire),
        vanilla_plus: VANILLA_PLUS.load(Ordering::Acquire),
        hints: HINTS.load(Ordering::Acquire),
    }
}

/// `early_present` (0.5.3). Read by the flush wrapper every frame.
pub(super) fn early_present_enabled() -> bool {
    EARLY_PRESENT.load(Ordering::Relaxed)
}

/// `vanilla_plus` (0.5.6). Read by the flush wrapper every frame; changes
/// live with the L+R+X chord.
pub(super) fn vanilla_plus_enabled() -> bool {
    VANILLA_PLUS.load(Ordering::Relaxed)
}

/// `dispatcher_backoff` (0.5.3). Read by the backoff hook on every idle pass.
pub(super) fn backoff_enabled() -> bool {
    BACKOFF_ENABLED.load(Ordering::Relaxed)
}

/// Enabled per-move resolution triggers (0.5.2 experiment). Atomic read; the
/// fighter Main hook calls this every tick.
pub(super) fn drs_moves() -> crate::drs_triggers::MoveSet {
    crate::drs_triggers::MoveSet(DRS_MOVES.load(Ordering::Acquire))
}

/// Adaptive trigger settings (0.5.2 experiment). Atomic reads; the present
/// hook calls this every frame.
pub(super) fn adaptive_settings() -> crate::drs_triggers::AdaptiveSettings {
    crate::drs_triggers::AdaptiveSettings {
        enabled: DRS_ADAPTIVE.load(Ordering::Acquire),
        threshold_hundredths_ms: DRS_THRESHOLD.load(Ordering::Acquire),
        trigger_frames: DRS_TRIGGER_FRAMES.load(Ordering::Acquire),
        release_frames: DRS_RELEASE_FRAMES.load(Ordering::Acquire),
    }
}

/// Saved Stealth flag. It starts on, matching `Config::default`, until load.
pub(super) fn stealth() -> bool {
    STEALTH.load(Ordering::Acquire)
}

pub(super) fn publish(config: &Config) {
    let mode_changed = saved_render_mode() != config.render_mode;
    let auto_changed = AUTO_PROFILE.load(Ordering::Acquire) != config.auto_profile;
    let previous_buffer = INPUT_BUFFER.swap(config.input_buffer, Ordering::AcqRel);
    if previous_buffer != config.input_buffer && (previous_buffer >= 0 || config.input_buffer >= 0)
    {
        // A manual choice can already have overwritten the game byte. Returning
        // to Auto must not present that manual byte as a fresh automatic choice.
        AUTO_BUFFER.invalidate_manual();
    }
    RENDER_MODE.store(config.render_mode as u8, Ordering::Release);
    AUTO_PROFILE.store(config.auto_profile, Ordering::Release);
    PLATFORM.store(config.platform as u8, Ordering::Release);
    STEALTH.store(config.stealth, Ordering::Release);
    STREAMER.store(config.streamer, Ordering::Release);
    ULTRA_RES.store(config.ultra_resolution.to_byte(), Ordering::Release);
    HIDE_PAGES.store(
        u8::from(config.hide_pages.plugins)
            | u8::from(config.hide_pages.own_pages) << 1
            | u8::from(config.hide_pages.performance) << 2,
        Ordering::Release,
    );
    PERF_NUMBER.store(
        config.perf_view == crate::config::PerfView::Number,
        Ordering::Release,
    );
    EARLY_PRESENT.store(config.early_present, Ordering::Release);
    BACKOFF_ENABLED.store(config.dispatcher_backoff, Ordering::Release);
    VANILLA_PLUS.store(config.vanilla_plus, Ordering::Release);
    HINTS.store(config.hints, Ordering::Release);
    DRS_MOVES.store(config.drs_moves.0, Ordering::Release);
    DRS_ADAPTIVE.store(config.drs_adaptive.enabled, Ordering::Release);
    DRS_THRESHOLD.store(
        config.drs_adaptive.threshold_hundredths_ms,
        Ordering::Release,
    );
    DRS_TRIGGER_FRAMES.store(config.drs_adaptive.trigger_frames, Ordering::Release);
    DRS_RELEASE_FRAMES.store(config.drs_adaptive.release_frames, Ordering::Release);
    CONFIG_GENERATION.fetch_add(1, Ordering::Release);
    // Same saved mode and Auto flag do not clear a post-match Vanilla hold.
    auto_driver::note_published(mode_changed, auto_changed);
}

pub(crate) fn select_user_profile(mode: crate::config::RenderMode) {
    auto_driver::select_user_profile(mode);
}

pub(crate) fn set_auto_profile(enabled: bool) {
    auto_driver::set_auto_profile(enabled);
}

pub(super) fn record(code: Code, a: u64, b: u64) {
    let tick = unsafe { skyline::nn::os::GetSystemTick() as u64 };
    RECORDER.record(tick, code, a, b);
    // Startup-only checkpoints reach disk before/after each patch, so an install
    // crash is diagnosable even before the background sink first wakes.
    if matches!(
        code,
        Code::HookInstalling | Code::HookInstalled | Code::HookRejected
    ) {
        logging::install_checkpoint(code, a, b);
    }
}

fn display_version() -> String {
    let mut version = skyline::nn::oe::DisplayVersion { name: [0; 16] };
    unsafe {
        skyline::nn::oe::GetDisplayVersion(&mut version);
    }
    let end = version
        .name
        .iter()
        .position(|v| *v == 0)
        .unwrap_or(version.name.len());
    String::from_utf8_lossy(&version.name[..end]).into_owned()
}

#[skyline::main(name = "ssbu-online-deluxe-rewrite")]
pub fn main() {
    logging::prepare();
    logging::startup_phase("version_query_begin");
    let version = display_version();
    logging::startup_phase("version_query_complete");
    logging::start(&version);
    if version != "13.0.5" {
        record(Code::UnsupportedVersion, 0, 0);
        skyline::println!(
            "[Deluxe] Unsupported Smash version {}; hooks skipped",
            version
        );
        return;
    }
    // Liveness log first: a hang during hook installation still leaves it.
    hang_trace::install();
    // Clock state for the ledger (Horizon-OC files, CPU calibration); no hooks.
    clock_probe::install();
    // Scene entries are observed only (Auto profile, banners, captures and
    // the exported online context); no scene mutation, global Layout::Draw,
    // fighter-frame or Training input-delay hook. Native titles are the CSS
    // function hook, not Layout::Draw.
    if legacy_host_present() {
        record(Code::Compatibility, 1, 0);
        skyline::println!("[Deluxe] Existing SsbuSync host detected; rewrite hooks skipped");
        return;
    }
    for (name, install) in [
        ("RENDERER", renderer::install as fn()),
        ("MENU", ui::install as fn()),
        ("NATIVE", native_banner::install as fn()),
        // Character FX and its hook family moved to ssbu-fx-control in 0.5.0.
        ("SCENE", online::install as fn()),
        ("BUFFER", hooks::install_latency as fn()),
        ("PIA", pia_rtt::install as fn()),
        ("BACKOFF", dispatcher_backoff::install as fn()),
    ] {
        if std::path::Path::new(DATA_DIR)
            .join(format!("DISABLE_{name}"))
            .exists()
        {
            record(Code::Compatibility, 2, name.as_bytes()[0] as u64);
            skyline::println!("[Deluxe] {} hooks disabled by diagnostic flag", name);
            if name == "BACKOFF" {
                dispatcher_backoff::note_disabled();
            }
        } else {
            install();
        }
    }
    // After the renderer family verified the dispatcher sites it shares. The
    // capture flags are read first: `CAPTURE_MATCH` also wants the CSS hooks.
    capture::install();
    css_capture::install();
    resolution::install();
    fighter_probe::install();
    connection_transport::install();
}

fn legacy_host_present() -> bool {
    for name in [
        b"ssbusync_status\0".as_slice(),
        b"ssbusync_restrict_mark_arena_mode\0".as_slice(),
    ] {
        let mut address = 0usize;
        unsafe {
            skyline::nn::ro::LookupSymbol(&mut address, name.as_ptr());
        }
        if address != 0 {
            return true;
        }
    }
    false
}
