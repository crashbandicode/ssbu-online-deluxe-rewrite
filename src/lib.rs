//! Portable settings, policy and diagnostics; Switch hooks are isolated.
pub mod config;
pub mod diagnostics;
pub mod effects;
pub mod menu;
pub mod policy;
pub mod ui_safety;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/build_info.rs"));
}

#[cfg(target_os = "switch")]
mod runtime;
