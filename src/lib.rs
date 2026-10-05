//! Portable settings, policy and diagnostics; Switch hooks are isolated.
pub mod acmd;
pub mod config;
pub mod config_store;
pub mod diagnostics;
pub mod effect_context;
pub mod effects;
mod effects_catalog;
pub mod menu;
pub mod overlay;
pub mod policy;
pub mod presentation;
pub mod ui_safety;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/build_info.rs"));
}

pub mod render_trace;
#[cfg(target_os = "switch")]
mod runtime;
pub mod timing;
