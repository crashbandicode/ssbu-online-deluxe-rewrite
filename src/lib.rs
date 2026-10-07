//! Portable settings, policy and diagnostics; Switch hooks are isolated.
pub mod acmd;
pub mod auto_policy;
pub mod config;
pub mod config_store;
pub mod diagnostics;
pub mod effect_context;
pub mod effects;
mod effects_catalog;
mod imgui_host;
pub mod menu;
pub mod native_banner;
pub mod overlay;
mod peer_observer;
mod peer_passive;
mod peer_transport;
mod pia_rtt;
pub mod policy;
pub mod presentation;
pub mod ui_safety;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/build_info.rs"));
}

pub mod fighter_probe;
pub mod perf_scaler;
pub mod render_policy;
pub mod render_trace;
pub mod resolution;
#[cfg(target_os = "switch")]
mod runtime;

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
pub mod timing;
