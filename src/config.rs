//! Versioned, human-readable settings. `version=1` is required; omitted settings
//! use defaults. Blank lines and `#` comments (including trailing comments) are
//! allowed. Each remaining line is exactly one `key=value` assignment.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum FxMode {
    #[default]
    On = 0,
    Off = 1,
    Reduced = 2,
}

impl FxMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::On => "On",
            Self::Off => "Off",
            Self::Reduced => "Reduced",
        }
    }

    /// Fail safely to normal visuals if an unsupported value reaches runtime.
    pub const fn effective(self) -> Self {
        match self {
            Self::Reduced => Self::On,
            supported => supported,
        }
    }

    pub const fn available(self) -> bool {
        !matches!(self, Self::Reduced)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub effects: FxMode,
    /// -1 means Auto; explicit values must be in 0..=25.
    pub input_buffer: i8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            effects: FxMode::On,
            input_buffer: -1,
        }
    }
}

pub fn parse(text: &str) -> Result<Config, String> {
    let mut config = Config::default();
    let mut seen = [false; 3];
    for (index, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let error = |message: &str| format!("line {}: {message}", index + 1);
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| error("expected key=value"))?;
        let key = key.trim();
        let value = value.trim();
        let slot = match key {
            "version" => 0,
            "effects" => 1,
            "input_buffer" => 2,
            _ => return Err(error(&format!("unknown key '{key}'"))),
        };
        if seen[slot] {
            return Err(error(&format!("duplicate key '{key}'")));
        }
        seen[slot] = true;
        match key {
            "version" if value != "1" => return Err(error("unsupported version; expected 1")),
            "version" => {}
            "effects" => {
                config.effects = match value {
                    "on" => FxMode::On,
                    "off" => FxMode::Off,
                    "reduced" => {
                        return Err(error("Reduced FX is unavailable; choose on or off"));
                    }
                    _ => return Err(error("effects must be on or off (Reduced is unavailable)")),
                };
            }
            "input_buffer" => {
                config.input_buffer = if value == "auto" {
                    -1
                } else {
                    // Decimal digits only: negative numbers and signs are not settings.
                    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                        return Err(error(
                            "input_buffer must be auto or an integer from 0 to 25",
                        ));
                    }
                    value
                        .parse::<i8>()
                        .ok()
                        .filter(|value| (0..=25).contains(value))
                        .ok_or_else(|| {
                            error("input_buffer must be auto or an integer from 0 to 25")
                        })?
                };
            }
            _ => unreachable!(),
        }
    }
    if !seen[0] {
        return Err("missing version=1".to_owned());
    }
    Ok(config)
}

pub fn serialize(config: &Config) -> String {
    let effects = match config.effects {
        FxMode::On => "on",
        FxMode::Off => "off",
        FxMode::Reduced => "reduced",
    };
    let input_buffer = if config.input_buffer == -1 {
        "auto".to_owned()
    } else {
        config.input_buffer.to_string()
    };
    format!("# Online Deluxe settings\nversion=1\neffects={effects}\ninput_buffer={input_buffer}\n")
}

impl Config {
    pub fn serialize(&self) -> String {
        serialize(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_versioned_file_and_comments_keep_safe_defaults() {
        assert_eq!(
            parse(" # settings\n version = 1 # format\n").unwrap(),
            Config::default()
        );
        assert_eq!(Config::default().effects, FxMode::On);
        assert_eq!(Config::default().input_buffer, -1);
    }

    #[test]
    fn saved_settings_round_trip_at_every_buffer_boundary() {
        for effects in [FxMode::On, FxMode::Off] {
            for input_buffer in [-1, 0, 25] {
                let config = Config {
                    effects,
                    input_buffer,
                };
                assert_eq!(parse(&config.serialize()).unwrap(), config);
            }
        }
    }

    #[test]
    fn malformed_or_ambiguous_files_are_rejected() {
        for text in [
            "effects=on",
            "version=2",
            "version=1\nversion=1",
            "version=1\neffects=on\neffects=off",
            "version=1\nunknown=1",
            "version=1\neffects=ON",
            "version=1\ninput_buffer=-1",
            "version=1\ninput_buffer=26",
            "version=1\ninput_buffer=+1",
            "version=1\ninput_buffer=999",
            "version=1\ninput_buffer=",
            "version=1\ninput_buffer=1=2",
            "version 1",
        ] {
            assert!(parse(text).is_err(), "accepted {text:?}");
        }
    }

    #[test]
    fn reduced_is_rejected_and_defensive_runtime_fallback_is_on() {
        assert!(parse("version=1\neffects=reduced")
            .unwrap_err()
            .contains("unavailable"));
        assert!(!FxMode::Reduced.available());
        assert_eq!(FxMode::Reduced.effective(), FxMode::On);
    }
}
