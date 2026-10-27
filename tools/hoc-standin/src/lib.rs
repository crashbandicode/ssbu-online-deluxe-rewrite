//! Test-only stand-in for the game-aware hoc-clk fork (protocol v1) inside the
//! Smash process, for Eden integration runs and documentation captures. Eden
//! cannot run Horizon-OC; this plugin registers `hoc:clk` with Eden's sm and
//! answers commands 0 and 100-103 with the fork's arm/lease semantics, so
//! Deluxe's real client, gate and indicator run end to end. It changes no
//! clock. Never install it on a console or ship it in a Deluxe release.

pub mod protocol;

#[cfg(target_os = "switch")]
mod runtime;

#[cfg(target_os = "switch")]
#[skyline::main(name = "hoc-standin")]
pub fn main() {
    runtime::start();
}
