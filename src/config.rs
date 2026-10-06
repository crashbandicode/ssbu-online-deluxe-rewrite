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

/// Restricts both Off and future Reduced policies; it never overrides FX On.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum FxScope {
    #[default]
    Online = 0,
    All = 1,
}

impl FxScope {
    pub const fn allows(self, online: bool) -> bool {
        matches!(self, Self::All) || online
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Online => "Online",
            Self::All => "All",
        }
    }
}

/// Startup-only render policies. Core variants retain the old sequencing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum RenderMode {
    #[default]
    Vanilla = 0,
    LessLagCore = 1,
    DoublesCore = 2,
    LessLag = 3,
    LessLagUltra = 4,
}
impl RenderMode {
    pub const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::LessLagCore,
            2 => Self::DoublesCore,
            3 => Self::LessLag,
            4 => Self::LessLagUltra,
            _ => Self::Vanilla,
        }
    }
    pub const fn key(self) -> &'static str {
        match self {
            Self::Vanilla => "vanilla",
            Self::LessLagCore => "lesslag_core",
            Self::DoublesCore => "doubles_core",
            Self::LessLag => "lesslag",
            Self::LessLagUltra => "llultra",
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::LessLagCore => "LessLag core",
            Self::DoublesCore => "Doubles core",
            Self::LessLag => "LessLag",
            Self::LessLagUltra => "LLUltra",
        }
    }
    pub const fn immediate(self) -> bool {
        matches!(self, Self::LessLagUltra)
    }
    pub const fn render_opts(self) -> bool {
        matches!(self, Self::LessLag | Self::LessLagUltra)
    }
    pub const fn textures(self) -> i32 {
        match self {
            Self::LessLagCore | Self::LessLag | Self::LessLagUltra => 2,
            _ => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub effects: FxMode,
    pub effects_scope: FxScope,
    /// -1 means Auto; explicit values must be in 0..=25.
    pub input_buffer: i8,
    /// Read once at startup; changing this requires a full restart.
    pub render_mode: RenderMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            effects: FxMode::On,
            effects_scope: FxScope::Online,
            input_buffer: -1,
            render_mode: RenderMode::Vanilla,
        }
    }
}

pub fn parse(text: &str) -> Result<Config, String> {
    let mut config = Config::default();
    let mut seen = [false; 5];
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
            "effects_scope" => 3,
            "render_mode" => 4,
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
            "effects_scope" => {
                config.effects_scope = match value {
                    "online" => FxScope::Online,
                    "all" => FxScope::All,
                    _ => return Err(error("effects_scope must be online or all")),
                };
            }
            "render_mode" => {
                config.render_mode =
                    match value {
                        "vanilla" => RenderMode::Vanilla,
                        "lesslag_core" => RenderMode::LessLagCore,
                        "lesslag" => RenderMode::LessLag,
                    "llultra" => RenderMode::LessLagUltra,
                        "doubles_core" => RenderMode::DoublesCore,
                        _ => return Err(error(
                            "render_mode must be vanilla, lesslag, llultra, lesslag_core or doubles_core",
                        )),
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
    let effects_scope = match config.effects_scope {
        FxScope::Online => "online",
        FxScope::All => "all",
    };
    let input_buffer = if config.input_buffer == -1 {
        "auto".to_owned()
    } else {
        config.input_buffer.to_string()
    };
    format!("# Online Deluxe settings\nversion=1\neffects={effects}\neffects_scope={effects_scope}\ninput_buffer={input_buffer}\nrender_mode={}\n", config.render_mode.key())
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
                    effects_scope: FxScope::Online,
                    input_buffer,
                    render_mode: RenderMode::Vanilla,
                };
                assert_eq!(parse(&config.serialize()).unwrap(), config);
            }
        }
    }

    #[test]
    fn renderer_selection_is_explicit_and_round_trips() {
        for render_mode in [
            RenderMode::Vanilla,
            RenderMode::LessLagCore,
            RenderMode::DoublesCore,
            RenderMode::LessLag,
            RenderMode::LessLagUltra,
        ] {
            let config = Config {
                render_mode,
                ..Config::default()
            };
            assert_eq!(parse(&config.serialize()).unwrap(), config);
        }
        assert!(parse("version=1\nrender_mode=unrecognized").is_err());
        assert!(parse("version=1\nrender_mode=vanilla\nrender_mode=vanilla").is_err());
        assert_eq!(RenderMode::from_byte(255), RenderMode::Vanilla);
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
    #[test]
    fn scope_defaults_migrates_and_round_trips() {
        let old = parse("version=1\neffects=off\ninput_buffer=auto").unwrap();
        assert_eq!(old.effects_scope, FxScope::Online);
        for effects_scope in [FxScope::Online, FxScope::All] {
            let config = Config {
                effects_scope,
                ..old.clone()
            };
            assert_eq!(parse(&config.serialize()).unwrap(), config);
        }
        for text in [
            "version=1\neffects_scope=offline",
            "version=1\neffects_scope=all\neffects_scope=online",
        ] {
            assert!(parse(text).is_err());
        }
        assert!(!FxScope::Online.allows(false));
        assert!(FxScope::Online.allows(true));
        assert!(FxScope::All.allows(false));
        assert!(FxScope::All.allows(true));
    }
}
