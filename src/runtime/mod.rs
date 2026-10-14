mod acmd;
mod api_trace;
mod auto_driver;
mod css_capture;
mod dispatcher_backoff;
mod effect_context;
mod effects;
mod fighter_main;
mod fighter_probe;
mod fps_boost;
mod hang_trace;
mod hooks;
mod imgui;
mod imgui_stats;
mod input;
mod logging;
mod match_capture;
mod memory;
mod native_banner;
mod nvn;
mod online;
mod overlay_host;
mod peer_observer;
mod pia_rtt;
mod plugin_export;
mod presentation;
mod render_probe;
mod render_sequence;
mod render_ultra;
mod renderer;
mod resolution;
mod response_probe;
mod timing;
mod ui;
mod variant;

use crate::{
    config::{Config, FxMode, FxScope},
    diagnostics::{Code, Recorder},
};
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU64, AtomicU8, Ordering};

pub(super) static RECORDER: Recorder = Recorder::new();
static FX_MODE: AtomicU8 = AtomicU8::new(0);
static FX_SCOPE: AtomicU8 = AtomicU8::new(FxScope::Online as u8);
static RENDER_MODE: AtomicU8 = AtomicU8::new(0);
static AUTO_PROFILE: AtomicBool = AtomicBool::new(true);
static PLATFORM: AtomicU8 = AtomicU8::new(0);
static INPUT_BUFFER: AtomicI8 = AtomicI8::new(-1);
static PEER_OBSERVER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static STEALTH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static STREAMER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static FPS_BOOST: AtomicBool = AtomicBool::new(false);
static CONFIG_GENERATION: AtomicU64 = AtomicU64::new(0);
pub(super) const DATA_DIR: &str = "sd:/ultimate/ssbu_online_deluxe_rewrite";

pub(super) fn saved_render_mode() -> crate::config::RenderMode {
    crate::config::RenderMode::from_byte(RENDER_MODE.load(Ordering::Acquire))
}

pub(super) fn settings() -> Config {
    Config {
        effects: match FX_MODE.load(Ordering::Acquire) {
            1 => FxMode::Off,
            2 => FxMode::Reduced,
            _ => FxMode::On,
        },
        effects_scope: if FX_SCOPE.load(Ordering::Acquire) == FxScope::All as u8 {
            FxScope::All
        } else {
            FxScope::Online
        },
        input_buffer: INPUT_BUFFER.load(Ordering::Acquire),
        render_mode: saved_render_mode(),
        auto_profile: AUTO_PROFILE.load(Ordering::Acquire),
        platform: if PLATFORM.load(Ordering::Acquire) == crate::config::Platform::Emulator as u8 {
            crate::config::Platform::Emulator
        } else {
            crate::config::Platform::Console
        },
        peer_observer: PEER_OBSERVER.load(Ordering::Acquire),
        stealth: STEALTH.load(Ordering::Acquire),
        streamer: STREAMER.load(Ordering::Acquire),
        fps_boost: FPS_BOOST.load(Ordering::Acquire),
    }
}

/// Saved Stealth flag. It starts on, matching `Config::default`, until load.
pub(super) fn stealth() -> bool {
    STEALTH.load(Ordering::Acquire)
}

pub(super) fn publish(config: &Config) {
    let mode_changed = saved_render_mode() != config.render_mode;
    let auto_changed = AUTO_PROFILE.load(Ordering::Acquire) != config.auto_profile;
    let previous_fx = fx_mode();
    FX_MODE.store(config.effects as u8, Ordering::Release);
    crate::effects_reduced::observe_mode(previous_fx, config.effects);
    FX_SCOPE.store(config.effects_scope as u8, Ordering::Release);
    INPUT_BUFFER.store(config.input_buffer, Ordering::Release);
    RENDER_MODE.store(config.render_mode as u8, Ordering::Release);
    AUTO_PROFILE.store(config.auto_profile, Ordering::Release);
    PLATFORM.store(config.platform as u8, Ordering::Release);
    PEER_OBSERVER.store(config.peer_observer, Ordering::Release);
    STEALTH.store(config.stealth, Ordering::Release);
    STREAMER.store(config.streamer, Ordering::Release);
    FPS_BOOST.store(config.fps_boost, Ordering::Release);
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

pub(super) fn fx_mode() -> FxMode {
    match FX_MODE.load(Ordering::Relaxed) {
        1 => FxMode::Off,
        2 => FxMode::Reduced,
        _ => FxMode::On,
    }
}

pub(super) fn fx_scope_allows() -> bool {
    FX_SCOPE.load(Ordering::Relaxed) == FxScope::All as u8 || online::active()
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
    // Scene entries are observed only (Auto profile, banners, captures and
    // the exported online context); no scene mutation, global Layout::Draw,
    // fighter-frame or Training input-delay hook. Native titles are the CSS
    // function hook, not Layout::Draw.
    if legacy_host_present() {
        record(Code::Compatibility, 1, 0);
        skyline::println!("[Deluxe] Existing SsbuSync host detected; rewrite hooks skipped");
        return;
    }
    render_probe::dump_if_requested();
    presentation::install();
    for (name, install) in [
        ("RENDERER", renderer::install as fn()),
        ("MENU", ui::install as fn()),
        ("NATIVE", native_banner::install as fn()),
        // The scene tracker was installed from the FX family until 0.4.2; it
        // is its own family now so DISABLE_FX no longer blinds Auto.
        ("SCENE", online::install as fn()),
        ("FX", effects::install as fn()),
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
    // match flag is read first: it also wants the CSS capture hooks.
    match_capture::install();
    css_capture::install();
    resolution::install();
    fighter_probe::install();
    peer_observer::install();
    fps_boost::arm();
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
