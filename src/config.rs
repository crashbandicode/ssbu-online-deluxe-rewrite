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

/// Live profiles plus the startup-only core experiments.
/// `LessLagDoubles` is the 3-texture OneBehind profile. `DoublesCore` is not.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum RenderMode {
    #[default]
    Vanilla = 0,
    LessLagCore = 1,
    DoublesCore = 2,
    LessLag = 3,
    LessLagUltra = 4,
    LessLagDoubles = 5,
}
impl RenderMode {
    pub const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::LessLagCore,
            2 => Self::DoublesCore,
            3 => Self::LessLag,
            4 => Self::LessLagUltra,
            5 => Self::LessLagDoubles,
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
            Self::LessLagDoubles => "lldoubles",
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::LessLagCore => "LessLag core",
            Self::DoublesCore => "Doubles core",
            Self::LessLag => "LessLag",
            Self::LessLagUltra => "LLUltra",
            Self::LessLagDoubles => "LLDoubles",
        }
    }
    pub const fn immediate(self) -> bool {
        matches!(self, Self::LessLagUltra)
    }
    pub const fn render_opts(self) -> bool {
        matches!(
            self,
            Self::LessLag | Self::LessLagUltra | Self::LessLagDoubles
        )
    }
    pub const fn textures(self) -> i32 {
        match self {
            Self::LessLagCore | Self::LessLag | Self::LessLagUltra => 2,
            Self::Vanilla | Self::DoublesCore | Self::LessLagDoubles => 3,
        }
    }
}

/// Explicit platform override. This is not emulator detection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum Platform {
    #[default]
    Console = 0,
    Emulator = 1,
}

impl Platform {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Console => "console",
            Self::Emulator => "emulator",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub effects: FxMode,
    pub effects_scope: FxScope,
    /// -1 means Auto; explicit values must be in 0..=25.
    pub input_buffer: i8,
    /// Saved manual profile. Auto does not overwrite this when the scene changes.
    /// Vanilla, LessLag, LLUltra, and LLDoubles may change live after the window
    /// count is known. Legacy core modes still apply on the next start.
    pub render_mode: RenderMode,
    /// Fork Auto. Absent from an older file that already saved `render_mode`,
    /// so that explicit choice stays manual. A file with neither key defaults on.
    pub auto_profile: bool,
    /// Honest console/emulator override. Default console. No text-base heuristic.
    pub platform: Platform,
    /// Diagnostic-only inbound observer. Omitted keys stay false.
    pub peer_observer: bool,
    /// Saved next-connection preference. The default and a missing key are on.
    /// An explicit true or false is kept. A live session latches the value at
    /// its first inverse request until session cleanup.
    pub stealth: bool,
    /// Display privacy. Default and a missing key are off. It does not change packets.
    pub streamer: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            effects: FxMode::On,
            effects_scope: FxScope::Online,
            input_buffer: -1,
            render_mode: RenderMode::Vanilla,
            auto_profile: true,
            platform: Platform::Console,
            peer_observer: false,
            stealth: true,
            streamer: false,
        }
    }
}

pub fn parse(text: &str) -> Result<Config, String> {
    let mut config = Config::default();
    let mut seen = [false; 10];
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
            "auto_profile" => 5,
            "platform" => 6,
            "peer_observer" => 7,
            "stealth" => 8,
            "streamer" => 9,
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
                config.render_mode = match value {
                    "vanilla" => RenderMode::Vanilla,
                    "lesslag_core" => RenderMode::LessLagCore,
                    "lesslag" => RenderMode::LessLag,
                    "llultra" => RenderMode::LessLagUltra,
                    "lldoubles" => RenderMode::LessLagDoubles,
                    "doubles_core" => RenderMode::DoublesCore,
                    _ => {
                        return Err(error(
                            "render_mode must be vanilla, lesslag, llultra, lldoubles, lesslag_core or doubles_core",
                        ))
                    }
                };
            }
            "auto_profile" => {
                config.auto_profile = match value {
                    "true" => true,
                    "false" => false,
                    _ => return Err(error("auto_profile must be true or false")),
                };
            }
            "platform" => {
                config.platform = match value {
                    "console" => Platform::Console,
                    "emulator" => Platform::Emulator,
                    _ => return Err(error("platform must be console or emulator")),
                };
            }
            "peer_observer" => {
                config.peer_observer = match value {
                    "true" => true,
                    "false" => false,
                    _ => return Err(error("peer_observer must be true or false")),
                };
            }
            "stealth" => {
                config.stealth = match value {
                    "true" => true,
                    "false" => false,
                    _ => return Err(error("stealth must be true or false")),
                };
            }
            "streamer" => {
                config.streamer = match value {
                    "true" => true,
                    "false" => false,
                    _ => return Err(error("streamer must be true or false")),
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
    // Older files always wrote render_mode and had no Auto key. Keep that
    // explicit profile manual. A file with neither key gets fork Auto.
    if !seen[5] {
        config.auto_profile = !seen[4];
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
    let peer_observer = if config.peer_observer { "true" } else { "false" };
    let stealth = if config.stealth { "true" } else { "false" };
    let streamer = if config.streamer { "true" } else { "false" };
    let auto_profile = if config.auto_profile { "true" } else { "false" };
    format!(
        "# Online Deluxe settings\nversion=1\neffects={effects}\neffects_scope={effects_scope}\ninput_buffer={input_buffer}\nrender_mode={}\nauto_profile={auto_profile}\nplatform={}\npeer_observer={peer_observer}\nstealth={stealth}\nstreamer={streamer}\n",
        config.render_mode.key(),
        config.platform.key(),
    )
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
        assert!(Config::default().stealth);
        assert!(parse("version=1").unwrap().stealth);
        assert!(!parse("version=1\nstealth=false").unwrap().stealth);
        assert!(parse("version=1\nstealth=true").unwrap().stealth);
        assert!(!parse("version=1").unwrap().streamer);
        assert!(parse("version=1\nstreamer=true").unwrap().streamer);
        assert!(!parse("version=1\nstreamer=false").unwrap().streamer);
        let enabled = Config {
            streamer: true,
            ..Config::default()
        };
        assert_eq!(parse(&enabled.serialize()).unwrap(), enabled);
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
                    auto_profile: true,
                    platform: Platform::Console,
                    peer_observer: false,
                    stealth: false,
                    streamer: false,
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
            RenderMode::LessLagDoubles,
        ] {
            let config = Config {
                render_mode,
                ..Config::default()
            };
            assert_eq!(parse(&config.serialize()).unwrap(), config);
        }
        assert!(parse("version=1\nrender_mode=unrecognized").is_err());
        assert!(parse("version=1\nrender_mode=lesslag+").is_err());
        assert!(parse("version=1\nrender_mode=lldoubles+").is_err());
        assert!(parse("version=1\nrender_mode=vanilla\nrender_mode=vanilla").is_err());
        assert_eq!(RenderMode::from_byte(5), RenderMode::LessLagDoubles);
        assert_eq!(RenderMode::from_byte(255), RenderMode::Vanilla);
        assert!(RenderMode::LessLagDoubles.render_opts());
        assert!(!RenderMode::LessLagDoubles.immediate());
        assert_eq!(RenderMode::LessLagDoubles.textures(), 3);
        assert!(!RenderMode::DoublesCore.render_opts());
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
        assert!(!parse("version=1").unwrap().peer_observer);
        assert!(parse("version=1\npeer_observer=true").unwrap().peer_observer);
        assert!(!parse("version=1\npeer_observer=false").unwrap().peer_observer);
        let enabled = Config {
            peer_observer: true,
            ..Config::default()
        };
        assert!(parse(&enabled.serialize()).unwrap().peer_observer);
        for text in [
            "version=1\npeer_observer=yes",
            "version=1\npeer_observer=TRUE",
            "version=1\npeer_observer=true\npeer_observer=false",
        ] {
            assert!(parse(text).is_err(), "accepted {text:?}");
        }
        assert!(!FxScope::Online.allows(false));
        assert!(FxScope::Online.allows(true));
        assert!(FxScope::All.allows(false));
        assert!(FxScope::All.allows(true));
    }

    #[test]
    fn profile_and_stealth_survive_save_recovery_and_reload() {
        let saved = Config {
            render_mode: RenderMode::LessLagUltra,
            stealth: true,
            ..Config::default()
        };
        assert_eq!(parse(&saved.serialize()).unwrap(), saved);
        let old = parse("version=1\neffects=off\ninput_buffer=4").unwrap();
        assert!(old.stealth);
        let explicit_off = Config {
            stealth: false,
            ..Config::default()
        };
        assert_eq!(parse(&explicit_off.serialize()).unwrap(), explicit_off);
        assert_eq!(old.render_mode, RenderMode::Vanilla);
        for text in [
            "version=1\nstealth=yes",
            "version=1\nstealth=TRUE",
            "version=1\nstealth=",
            "version=1\nstealth=true\nstealth=false",
            "version=1\nstreamer=yes",
            "version=1\nstreamer=true\nstreamer=false",
            "version=1\nrender_mode=auto",
            "version=1\nauto_profile=yes",
            "version=1\nplatform=eden",
            "version=1\nauto_profile=true\nauto_profile=false",
            "version=1\nplatform=console\nplatform=emulator",
        ] {
            assert!(parse(text).is_err(), "accepted {text:?}");
        }
        let dir = std::env::temp_dir().join(format!(
            "deluxe-profile-menu-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.ini");
        let original = Config::default();
        crate::config_store::save(&path, &original.serialize()).unwrap();
        crate::config_store::save(&path, &saved.serialize()).unwrap();
        assert_eq!(
            parse(&std::fs::read_to_string(&path).unwrap()).unwrap(),
            saved
        );
        std::fs::remove_file(&path).unwrap();
        crate::config_store::recover(&path).unwrap();
        assert_eq!(
            parse(&std::fs::read_to_string(&path).unwrap()).unwrap(),
            original
        );
        crate::config_store::save(&path, &saved.serialize()).unwrap();
        crate::config_store::recover(&path).unwrap();
        assert_eq!(
            parse(&std::fs::read_to_string(&path).unwrap()).unwrap(),
            saved
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn old_explicit_render_mode_stays_manual_and_new_files_default_auto() {
        let legacy = parse("version=1\nrender_mode=lesslag").unwrap();
        assert_eq!(legacy.render_mode, RenderMode::LessLag);
        assert!(!legacy.auto_profile);
        assert_eq!(legacy.platform, Platform::Console);
        let untouched = parse("version=1").unwrap();
        assert!(untouched.auto_profile);
        assert_eq!(untouched.render_mode, RenderMode::Vanilla);
        assert_eq!(untouched.platform, Platform::Console);
        let opted_in = parse("version=1\nrender_mode=lesslag\nauto_profile=true\nplatform=emulator").unwrap();
        assert!(opted_in.auto_profile);
        assert_eq!(opted_in.render_mode, RenderMode::LessLag);
        assert_eq!(opted_in.platform, Platform::Emulator);
        let doubles = Config {
            render_mode: RenderMode::LessLagDoubles,
            auto_profile: true,
            platform: Platform::Emulator,
            ..Config::default()
        };
        assert_eq!(parse(&doubles.serialize()).unwrap(), doubles);
        assert!(doubles.serialize().contains("render_mode=lldoubles"));
        assert!(doubles.serialize().contains("auto_profile=true"));
        assert!(doubles.serialize().contains("platform=emulator"));
    }
}
