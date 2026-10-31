//! The refresh-clock plugin's `status.txt` (0.9.0): its 120 Hz counters,
//! parsed and packed into event code 120 so `events.log` and every stage log
//! carry them from boot, every 5 s, without a relaunch. Deluxe only reads the
//! file; the plugin rewrites it once a second while its `telemetry.flag`
//! exists (otherwise once, after startup, and the interval fields are absent).

/// The fields the 120 Hz checks use; missing ones stay 0 or false.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Sample {
    pub normalizing: bool,
    pub interval_valid: bool,
    pub observe_only: bool,
    /// VSync events in the last telemetry interval (about 120 a second at 120 Hz).
    pub physical: u64,
    /// Game ticks in that interval (about 60 a second while normalizing).
    pub logical: u64,
    /// The interval's length in 19.2 MHz system ticks.
    pub interval_ticks: u64,
    pub mode_transitions: u64,
    /// From the SaltyNX metadata: display refresh, swap interval, docked FPS target.
    pub refresh: u64,
    pub interval: u64,
    pub docked_target: u64,
}

fn number(value: &str) -> u64 {
    value.trim().parse().unwrap_or(0)
}

/// `None` for a file without the plugin's `normalizing=` line, such as a
/// partial read while the plugin rewrites it.
pub fn parse(text: &str) -> Option<Sample> {
    let mut sample = Sample::default();
    let mut seen = false;
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "normalizing" => {
                sample.normalizing = value.trim() == "true";
                seen = true;
            }
            "interval_valid" => sample.interval_valid = value.trim() == "true",
            "observe_only" => sample.observe_only = value.trim() == "true",
            "interval_physical_events" => sample.physical = number(value),
            "interval_logical_ticks" => sample.logical = number(value),
            "interval_system_ticks" => sample.interval_ticks = number(value),
            "mode_transitions" => sample.mode_transitions = number(value),
            "metadata" => {
                // `Sample { valid: true, global_refresh: 120, active: 1,
                // interval: 2, api: 1, refresh: 120, docked_target: 60 }`
                let inner = value
                    .split_once('{')
                    .and_then(|(_, rest)| rest.split_once('}'))
                    .map_or("", |(fields, _)| fields);
                for field in inner.split(',') {
                    if let Some((name, value)) = field.split_once(':') {
                        match name.trim() {
                            "refresh" => sample.refresh = number(value),
                            "interval" => sample.interval = number(value),
                            "docked_target" => sample.docked_target = number(value),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    seen.then_some(sample)
}

/// Code 120's arguments. `a`: physical events (bits 0–15), logical ticks
/// (16–31), refresh (32–39), swap interval (40–43), normalizing (44),
/// interval valid (45), observe-only (46), mode transitions (48–55), docked
/// target (56–63), each saturated to its width. `b`: the interval's system
/// ticks, so physical × 19,200,000 / b is the VSync rate.
pub fn pack(sample: Sample) -> (u64, u64) {
    let field = |value: u64, bits: u32| value.min((1 << bits) - 1);
    let a = field(sample.physical, 16)
        | field(sample.logical, 16) << 16
        | field(sample.refresh, 8) << 32
        | field(sample.interval, 4) << 40
        | u64::from(sample.normalizing) << 44
        | u64::from(sample.interval_valid) << 45
        | u64::from(sample.observe_only) << 46
        | field(sample.mode_transitions, 8) << 48
        | field(sample.docked_target, 8) << 56;
    (a, sample.interval_ticks)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `usb-logs-1028/read1/refresh-clock/status.txt`, 2026-10-28.
    const CARD: &str = "candidate=0.0.2-dev\nhardware_accepted=false\nguard_ok=true\ncontinuous_telemetry=true\nsnapshot_final=false\nobserve_only=false\nthread_entries=1\nnormalizing=true\nphysical_events=144555\nlogical_ticks=93471\nnormalized_events=102168\nmode_transitions=1\nelapsed_system_ticks=30305098822\ntick_hz=19200000\ninterval_valid=true\ninterval_system_ticks=19772558\ninterval_physical_events=124\ninterval_logical_ticks=62\nmetadata=Sample { valid: true, global_refresh: 120, active: 1, interval: 2, api: 1, refresh: 120, docked_target: 60 }\nshared_map_attempts=1\n";

    #[test]
    fn the_card_status_parses_and_packs() {
        let sample = parse(CARD).unwrap();
        assert_eq!(
            sample,
            Sample {
                normalizing: true,
                interval_valid: true,
                observe_only: false,
                physical: 124,
                logical: 62,
                interval_ticks: 19_772_558,
                mode_transitions: 1,
                refresh: 120,
                interval: 2,
                docked_target: 60,
            }
        );
        let (a, b) = pack(sample);
        assert_eq!(a & 0xffff, 124);
        assert_eq!(a >> 16 & 0xffff, 62);
        assert_eq!(a >> 32 & 0xff, 120);
        assert_eq!(a >> 40 & 0xf, 2);
        assert_eq!(a >> 44 & 0b111, 0b011);
        assert_eq!(a >> 48 & 0xff, 1);
        assert_eq!(a >> 56, 60);
        assert_eq!(b, 19_772_558);
        // 124 events over 1.03 s: about 120 per second.
        assert_eq!(124 * 19_200_000 / b, 120);
    }

    #[test]
    fn a_partial_or_foreign_file_is_skipped_and_wide_values_saturate() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("candidate=0.0.2-dev\nguard_ok=true\n"), None);
        let startup = parse("normalizing=false\nmetadata=garbage\n").unwrap();
        assert_eq!(startup, Sample::default());
        let (a, _) = pack(Sample {
            physical: 1 << 20,
            mode_transitions: 999,
            interval: 31,
            ..Sample::default()
        });
        assert_eq!(a & 0xffff, 0xffff);
        assert_eq!(a >> 48 & 0xff, 0xff);
        assert_eq!(a >> 40 & 0xf, 0xf);
    }
}
