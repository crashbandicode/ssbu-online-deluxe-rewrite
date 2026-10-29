//! Portable settings, policy and diagnostics; Switch hooks are isolated.
pub mod auto_buffer;
pub mod auto_policy;
pub mod capture;
pub mod clock_probe;
pub mod config;
pub mod config_store;
mod connection_transport;
pub mod coverage_profile;
pub mod diagnostics;
pub mod dispatcher_backoff;
pub mod hoc_game_aware;
pub mod hook_chain;
pub mod hud_place;
mod imgui_host;
pub mod match_log;
pub mod match_state;
pub mod menu;
pub mod native_banner;
pub mod net_watch;
pub mod online_session;
pub mod overlay;
mod peer_passive;
mod peer_transport;
mod pia_rtt;
pub mod plugin_api;
pub mod policy;
pub mod present_pace;
pub mod refresh_clock_status;
pub mod ui_safety;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/build_info.rs"));
}

pub mod drs_triggers;
pub mod fighter_probe;
pub mod hang_trace;
pub mod perf_scaler;
pub mod render_policy;
pub mod resolution;
#[cfg(target_os = "switch")]
mod runtime;
pub mod teardown_guard;

/// Records an explicit manual profile choice for the later menu.
/// Does not write `config.ini`. Pair with [`set_auto_profile`] when leaving Auto.
#[cfg(target_os = "switch")]
pub fn select_user_profile(mode: config::RenderMode) {
    runtime::select_user_profile(mode);
}

/// Toggles Auto without changing the saved manual profile and without writing the INI.
#[cfg(target_os = "switch")]
pub fn set_auto_profile(enabled: bool) {
    runtime::set_auto_profile(enabled);
}
