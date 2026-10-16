//! Clock state for the measurement ledger (0.5.3): is Horizon-OC configured
//! for this title, and what CPU clock is the console actually running.
//!
//! Nothing inside the game process can read the clock registers, so the
//! runtime times a chain of dependent integer adds against the 19.2 MHz
//! system tick. A dependent `add` retires once per cycle on the Cortex-A57,
//! so `cycles / elapsed` is the core clock within a few percent; the official
//! clock is 1020 MHz, the game's own loading boost and Horizon-OC's usual
//! choice are 1785 MHz. GPU and EMC are not readable from here, but a HOC
//! profile moves all three together. The Horizon-OC files on the SD say what
//! is configured; the sample says what is running.
//!
//! Captures carry the result as `# clock_mhz=` / `# clock=` / `# hoc=`
//! header lines, `renderer-status.txt` as `clock_mhz=` / `clock=` / `hoc=`,
//! the Performance view as the `Clock` row, and the event log as code 111
//! (`a` = MHz, `b` = the HOC bits) at boot, on every class change and once a
//! minute. The analysis scripts refuse an `oc` capture for the ledger.

/// Official CPU clock in game.
pub const STOCK_MHZ: u32 = 1020;
/// A sample above this is not the stock clock (stock reads 980–1060 with
/// the loop's overhead; the next clock Horizon-OC offers is 1224).
pub const STOCK_LIMIT_MHZ: u32 = 1120;
/// Dependent adds per loop iteration in the runtime's calibration loop.
pub const ADDS_PER_ITERATION: u64 = 32;
/// Loop iterations per sample: 2^20 cycles, about 1 ms at stock.
pub const ITERATIONS_PER_SAMPLE: u64 = 1 << 15;

/// What the Horizon-OC files on the SD say.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HocState {
    /// `atmosphere/kips/hoc.kip` exists (the loader replacement is installed).
    pub kip: bool,
    /// hoc-clk's `boot2.flag` exists (the sysmodule starts with the console).
    pub autostart: bool,
    /// `config/horizon-oc/config.ini` has a `[01006A800016E000]` section.
    pub title_profile: bool,
    /// `uncapped_clocks=1` in that file.
    pub uncapped: bool,
}

impl HocState {
    pub const fn bits(self) -> u64 {
        (self.kip as u64) | (self.autostart as u64) << 1 | (self.title_profile as u64) << 2 | (self.uncapped as u64) << 3
    }

    pub const fn from_bits(bits: u64) -> Self {
        Self {
            kip: bits & 1 != 0,
            autostart: bits & 2 != 0,
            title_profile: bits & 4 != 0,
            uncapped: bits & 8 != 0,
        }
    }

    /// `none`, or the installed parts joined with `+`: `kip`, `clk`,
    /// `smash` (a profile for this title), `uncapped`.
    pub fn label(self) -> String {
        let parts: Vec<&str> = [
            (self.kip, "kip"),
            (self.autostart, "clk"),
            (self.title_profile, "smash"),
            (self.uncapped, "uncapped"),
        ]
        .into_iter()
        .filter_map(|(on, name)| on.then_some(name))
        .collect();
        if parts.is_empty() {
            "none".to_owned()
        } else {
            parts.join("+")
        }
    }
}

/// Reads the two facts the ledger needs from `config/horizon-oc/config.ini`.
pub fn parse_hoc_config(text: &str) -> (bool, bool) {
    let mut title_profile = false;
    let mut uncapped = false;
    let mut in_values = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_values = line.eq_ignore_ascii_case("[values]");
            if line.eq_ignore_ascii_case("[01006A800016E000]") {
                title_profile = true;
            }
            continue;
        }
        if in_values {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim() == "uncapped_clocks" {
                    uncapped = value.trim() == "1";
                }
            }
        }
    }
    (title_profile, uncapped)
}

/// MHz from a calibration sample: `cycles` dependent adds took `elapsed`
/// ticks of a `tick_hz` counter. Zero elapsed reads as 0 (unknown).
pub fn mhz_from_sample(cycles: u64, elapsed_ticks: u64, tick_hz: u64) -> u32 {
    if elapsed_ticks == 0 {
        return 0;
    }
    let hz = cycles as u128 * tick_hz as u128 / elapsed_ticks as u128;
    (hz / 1_000_000).min(u32::MAX as u128) as u32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Unknown,
    Stock,
    Oc,
}

impl Class {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Stock => "stock",
            Self::Oc => "oc",
        }
    }
}

pub const fn classify(mhz: u32) -> Class {
    if mhz == 0 {
        Class::Unknown
    } else if mhz <= STOCK_LIMIT_MHZ {
        Class::Stock
    } else {
        Class::Oc
    }
}

/// Performance-row text: `1021 MHz stock, HOC none` / `1783 MHz OC, HOC kip+clk+smash`.
pub fn status_label(mhz: u32, hoc: HocState) -> String {
    let clock = match classify(mhz) {
        Class::Unknown => "--- MHz".to_owned(),
        Class::Stock => format!("{mhz} MHz stock"),
        Class::Oc => format!("{mhz} MHz OC"),
    };
    format!("{clock}, HOC {}", hoc.label())
}

/// `# key=value` lines for the capture headers (no trailing newline).
pub fn header_lines(mhz: u32, hoc: HocState) -> String {
    format!(
        "# clock_mhz={mhz}\n# clock={}\n# hoc={}",
        classify(mhz).key(),
        hoc.label()
    )
}

/// `key=value` lines for `renderer-status.txt` (no trailing newline).
pub fn status_lines(mhz: u32, hoc: HocState) -> String {
    format!(
        "clock_mhz={mhz}\nclock={}\nhoc={}",
        classify(mhz).key(),
        hoc.label()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hoc_bits_round_trip_and_label() {
        for bits in 0..16u64 {
            let state = HocState::from_bits(bits);
            assert_eq!(state.bits(), bits);
        }
        assert_eq!(HocState::default().label(), "none");
        assert_eq!(
            HocState {
                kip: true,
                autostart: true,
                title_profile: true,
                uncapped: true
            }
            .label(),
            "kip+clk+smash+uncapped"
        );
        assert_eq!(
            HocState {
                kip: true,
                ..HocState::default()
            }
            .label(),
            "kip"
        );
    }

    #[test]
    fn hoc_config_parse_finds_the_title_section_and_uncapped_flag() {
        let card = "[values]\nuncapped_clocks=1\nis_first_load=0\nkip_version=251\n\n[01006A800016E000]\nhandheld_cpu=1785\nhandheld_gpu=460\n";
        assert_eq!(parse_hoc_config(card), (true, true));
        let capped = "[values]\nuncapped_clocks=0\n[0100000000010000]\nhandheld_cpu=1224\n";
        assert_eq!(parse_hoc_config(capped), (false, false));
        assert_eq!(parse_hoc_config(""), (false, false));
        // uncapped_clocks only counts in [values].
        assert_eq!(parse_hoc_config("[01006A800016E000]\nuncapped_clocks=1\n"), (true, false));
    }

    #[test]
    fn mhz_estimate_and_classification() {
        let cycles = ADDS_PER_ITERATION * ITERATIONS_PER_SAMPLE;
        assert_eq!(cycles, 1 << 20);
        // 2^20 cycles at 1020 MHz take 1.028 ms = 19 739 ticks of 19.2 MHz.
        let stock_ticks = (cycles as u128 * 19_200_000 / 1_020_000_000) as u64;
        let mhz = mhz_from_sample(cycles, stock_ticks, 19_200_000);
        assert!((1015..=1025).contains(&mhz), "{mhz}");
        assert_eq!(classify(mhz), Class::Stock);
        let boost_ticks = (cycles as u128 * 19_200_000 / 1_785_000_000) as u64;
        let boost = mhz_from_sample(cycles, boost_ticks, 19_200_000);
        assert!((1780..=1790).contains(&boost), "{boost}");
        assert_eq!(classify(boost), Class::Oc);
        // The next HOC step above stock is well past the limit; a slow sample
        // (preempted) reads lower, never higher, so it cannot fake OC.
        assert_eq!(classify(1224), Class::Oc);
        assert_eq!(classify(STOCK_LIMIT_MHZ), Class::Stock);
        assert_eq!(classify(0), Class::Unknown);
        assert_eq!(mhz_from_sample(cycles, 0, 19_200_000), 0);
    }

    #[test]
    fn labels_and_header_lines() {
        let hoc = HocState {
            kip: true,
            autostart: true,
            title_profile: true,
            uncapped: false,
        };
        assert_eq!(status_label(1021, hoc), "1021 MHz stock, HOC kip+clk+smash");
        assert_eq!(status_label(1783, HocState::default()), "1783 MHz OC, HOC none");
        assert_eq!(status_label(0, HocState::default()), "--- MHz, HOC none");
        assert_eq!(
            header_lines(1021, hoc),
            "# clock_mhz=1021\n# clock=stock\n# hoc=kip+clk+smash"
        );
        assert_eq!(status_lines(0, HocState::default()), "clock_mhz=0\nclock=unknown\nhoc=none");
    }
}
