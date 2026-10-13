//! Hang-trace formatting (0.3.2). Pure and host-tested; the device side in
//! `runtime::hang_trace` only owns the atomics and the file.
//!
//! Purpose: when the game stops on a loading screen, say which of this
//! plugin's hooks were mid-call and which of its liveness signals stopped
//! advancing, in a text file another agent can align with its own log by
//! system tick. Nothing here changes game behaviour.

use crate::auto_policy::{Connection, SceneStatus};

pub const TICK_HZ: u64 = 19_200_000;

/// Tracked hooks, in the order they appear on a heartbeat line.
pub const HOOK_NAMES: [&str; HOOK_COUNT] = [
    "present",
    "backoff_pass",
    "backoff_sleep",
    "css_init",
    "css_wait",
    "css_ui_job",
    "fighter_load",
    "fighter_unload",
    "fighter_main",
    "cut_in",
];
pub const HOOK_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HookSample {
    pub calls: u64,
    /// Entries without a matching exit. Count-only hooks stay at zero.
    pub in_flight: u32,
    /// Tick of the most recent entry; meaningful while `in_flight > 0`.
    pub entry_tick: u64,
    /// First call since the current scene began, zero until it happens.
    pub first_tick: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Liveness {
    pub scene: u8,
    pub ready: bool,
    pub in_css: bool,
    pub trace_active: bool,
}

/// Seconds since `boot` with millisecond precision; ticks before boot read 0.
pub fn seconds(tick: u64, boot: u64) -> String {
    let ticks = tick.saturating_sub(boot);
    let ms = ticks / (TICK_HZ / 1000);
    format!("{}.{:03}", ms / 1000, ms % 1000)
}

pub fn age_ms(now: u64, entry: u64) -> u64 {
    now.saturating_sub(entry) / (TICK_HZ / 1000)
}

/// `connection/status` from the packed scene byte used by `auto_driver`.
pub fn scene_label(packed: u8) -> String {
    format!(
        "{}/{}",
        Connection::from_byte(packed & 0b11).key(),
        SceneStatus::from_byte(packed >> 2).key()
    )
}

pub fn heartbeat_line(
    now: u64,
    boot: u64,
    live: Liveness,
    samples: &[HookSample; HOOK_COUNT],
) -> String {
    let mut line = format!(
        "[Heartbeat] t={} tick={:#x} scene={} ready={} css={} trace={}",
        seconds(now, boot),
        now,
        scene_label(live.scene),
        u8::from(live.ready),
        u8::from(live.in_css),
        u8::from(live.trace_active),
    );
    for (name, sample) in HOOK_NAMES.iter().zip(samples) {
        line.push_str(&format!(" {name}={}", sample.calls));
        if sample.in_flight > 0 {
            line.push_str(&format!(
                "[ACTIVE x{} age_ms={}]",
                sample.in_flight,
                age_ms(now, sample.entry_tick)
            ));
        }
    }
    line
}

pub fn first_line(hook: usize, first_tick: u64, boot: u64, scene: u8) -> String {
    format!(
        "[HangTrace] first {} t={} tick={:#x} scene={}",
        HOOK_NAMES.get(hook).copied().unwrap_or("?"),
        seconds(first_tick, boot),
        first_tick,
        scene_label(scene)
    )
}

pub fn scene_line(previous: u8, next: u8, now: u64, boot: u64) -> String {
    format!(
        "[HangTrace] scene {} -> {} t={} tick={:#x}",
        scene_label(previous),
        scene_label(next),
        seconds(now, boot),
        now
    )
}

pub fn header(build: &str, boot: u64) -> String {
    format!(
        "build={build}\nboot_tick={boot:#x} tick_hz={TICK_HZ}\n# [Heartbeat] every 5 s: t = seconds since boot_tick; each hook shows calls since boot and, while a call has entered and not returned, [ACTIVE xN age_ms]\n# [HangTrace] first <hook>: first call of that hook since the current scene began\n# present/backoff_pass/fighter_main are count-only liveness signals\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_and_ages_are_millisecond_precise_and_saturating() {
        assert_eq!(seconds(TICK_HZ * 12 + TICK_HZ / 2, 0), "12.500");
        assert_eq!(seconds(100, 200), "0.000");
        assert_eq!(age_ms(TICK_HZ / 1000 * 73 + 5, 0), 73);
        assert_eq!(age_ms(5, 9), 0);
    }

    #[test]
    fn scene_label_decodes_the_packed_byte() {
        assert_eq!(scene_label(0), "offline/inactive");
        assert_eq!(scene_label(1 | (1 << 2)), "local/training");
        assert_eq!(scene_label(3 | (2 << 2)), "quickplay/singles");
    }

    #[test]
    fn heartbeat_marks_only_in_flight_hooks_active() {
        let mut samples = [HookSample::default(); HOOK_COUNT];
        samples[0].calls = 1200;
        samples[4] = HookSample {
            calls: 9,
            in_flight: 2,
            entry_tick: TICK_HZ,
            first_tick: TICK_HZ,
        };
        let line = heartbeat_line(
            TICK_HZ * 3,
            TICK_HZ,
            Liveness {
                scene: 1,
                ready: true,
                in_css: false,
                trace_active: false,
            },
            &samples,
        );
        assert!(line.starts_with("[Heartbeat] t=2.000 tick=0x"));
        assert!(line.contains(" scene=local/inactive ready=1 css=0 trace=0 present=1200 "));
        assert!(line.contains(" css_wait=9[ACTIVE x2 age_ms=2000] "));
        assert!(!line.contains("present=1200["));
        assert_eq!(line.matches("ACTIVE").count(), 1);
    }

    #[test]
    fn first_and_scene_lines_name_hooks_and_scenes() {
        assert_eq!(
            first_line(6, TICK_HZ * 4, TICK_HZ, 4),
            format!(
                "[HangTrace] first fighter_load t=3.000 tick={:#x} scene=offline/training",
                TICK_HZ * 4
            )
        );
        assert!(first_line(99, 0, 0, 0).contains("first ? "));
        assert!(scene_line(0, 5, TICK_HZ, 0)
            .starts_with("[HangTrace] scene offline/inactive -> local/training t=1.000"));
        assert!(header("b", 7).starts_with("build=b\nboot_tick=0x7 tick_hz=19200000\n"));
    }
}
