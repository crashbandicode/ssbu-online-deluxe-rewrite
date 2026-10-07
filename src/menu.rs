//! Portable menu state. Runtime owns controller edges and rendering.

use crate::auto_policy::{cycle_profile_choice, ProfileChoice};
use crate::config::{Config, FxMode, Platform, RenderMode};

/// Editable Player Info rows, top to bottom.
pub const ROW_PEER: usize = 0;
pub const ROW_NET_LATENCY: usize = 1;
pub const ROW_NET_PROFILE: usize = 2;
pub const ROW_FX: usize = 3;
pub const ROW_STEALTH: usize = 4;
pub const ROW_STREAMER: usize = 5;
pub const ROW_COUNT: usize = 6;

/// What an explicit profile edit must do before `publish`.
///
/// `EnableAuto` keeps the saved manual mode and lets the current scene choose
/// the request. `SelectManual` includes a press that lands on the mode already
/// saved, so the cleanup hold can clear. Other rows are `Unchanged`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileAction {
    Unchanged,
    EnableAuto,
    SelectManual(RenderMode),
}

pub fn profile_choice(config: &Config) -> ProfileChoice {
    if config.auto_profile {
        ProfileChoice::Auto
    } else {
        ProfileChoice::Manual(config.render_mode)
    }
}

pub fn profile_action(before: &Config, after: &Config) -> ProfileAction {
    if before.auto_profile == after.auto_profile && before.render_mode == after.render_mode {
        return ProfileAction::Unchanged;
    }
    if after.auto_profile {
        ProfileAction::EnableAuto
    } else {
        ProfileAction::SelectManual(after.render_mode)
    }
}

/// Configured platform. This is the INI value, not a text-base guess.
pub fn platform_label(platform: Platform) -> &'static str {
    match platform {
        Platform::Console => "Console",
        Platform::Emulator => "Emulator",
    }
}

/// Saved choice versus the live policy word.
/// `applied` is false while the window is still awaiting the startup mode.
pub fn net_profile_value(
    saved: RenderMode,
    armed: RenderMode,
    textures: i32,
    machine: crate::render_policy::Machine,
    applied: bool,
) -> String {
    crate::render_policy::profile_text(saved, armed, textures, machine, applied)
}

/// Short third-column status. The profile cell itself stays the mode name.
pub fn net_profile_status(
    saved: RenderMode,
    armed: RenderMode,
    textures: i32,
    machine: crate::render_policy::Machine,
    applied: bool,
) -> &'static str {
    crate::render_policy::profile_status(saved, armed, textures, machine, applied)
}

/// Saved preference, and the latched session when one is open.
pub fn stealth_value(saved: bool, latched: Option<bool>) -> &'static str {
    crate::peer_transport::stealth_label(saved, latched)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Toggle,
    Up,
    Down,
    Left,
    Right,
    Close,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Menu {
    pub open: bool,
    pub selected: usize,
}

impl Menu {
    /// Returns true only when a setting actually changes.
    pub fn handle(&mut self, action: Action, config: &mut Config) -> bool {
        match action {
            Action::Toggle => {
                self.open = !self.open;
                if self.open {
                    self.selected = 0;
                }
                return false;
            }
            Action::Close => {
                self.open = false;
                return false;
            }
            _ if !self.open => return false,
            Action::Up => {
                self.selected = (self.selected + ROW_COUNT - 1) % ROW_COUNT;
                return false;
            }
            Action::Down => {
                self.selected = (self.selected + 1) % ROW_COUNT;
                return false;
            }
            Action::Left | Action::Right => {}
        }
        let direction: i8 = if action == Action::Left { -1 } else { 1 };
        match self.selected {
            ROW_PEER => {
                crate::peer_transport::cycle_peer(direction);
                false
            }
            ROW_NET_LATENCY => {
                let next = (i16::from(config.input_buffer) + i16::from(direction)).clamp(-1, 25) as i8;
                if next == config.input_buffer {
                    return false;
                }
                config.input_buffer = next;
                true
            }
            ROW_NET_PROFILE => {
                let next = cycle_profile_choice(profile_choice(config), direction);
                match next {
                    ProfileChoice::Auto => {
                        if config.auto_profile {
                            return false;
                        }
                        config.auto_profile = true;
                        true
                    }
                    ProfileChoice::Manual(mode) => {
                        if !config.auto_profile && config.render_mode == mode {
                            return false;
                        }
                        config.auto_profile = false;
                        config.render_mode = mode;
                        true
                    }
                }
            }
            ROW_FX => {
                config.effects = match config.effects {
                    FxMode::On => FxMode::Off,
                    FxMode::Off | FxMode::Reduced => FxMode::On,
                };
                true
            }
            ROW_STEALTH => {
                config.stealth = !config.stealth;
                true
            }
            ROW_STREAMER => {
                config.streamer = !config.streamer;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_focuses_the_first_row_and_closed_input_cannot_edit_settings() {
        let mut menu = Menu {
            open: false,
            selected: 1,
        };
        let mut config = Config::default();
        assert!(!menu.handle(Action::Right, &mut config));
        assert_eq!(config, Config::default());
        assert!(!menu.handle(Action::Toggle, &mut config));
        assert_eq!(menu.selected, ROW_PEER);
        assert!(!menu.handle(Action::Right, &mut config));
        assert_eq!(config.input_buffer, -1);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.input_buffer, 0);
        assert_eq!(config.effects, FxMode::On);
        menu.handle(Action::Close, &mut config);
        assert!(!menu.open);
    }

    #[test]
    fn fx_cycles_only_available_modes_even_after_unsupported_state() {
        let mut menu = Menu {
            open: true,
            selected: ROW_FX,
        };
        let mut config = Config {
            effects: FxMode::Reduced,
            ..Config::default()
        };
        for expected in [FxMode::On, FxMode::Off, FxMode::On, FxMode::Off] {
            assert!(menu.handle(Action::Left, &mut config));
            assert_eq!(config.effects, expected);
        }
    }

    #[test]
    fn navigation_wraps_and_buffer_saturates_without_false_change_reports() {
        let mut menu = Menu {
            open: true,
            selected: 0,
        };
        let mut config = Config::default();
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_NET_PROFILE);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_FX);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_STEALTH);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_STREAMER);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PEER);
        menu.handle(Action::Up, &mut config);
        assert_eq!(menu.selected, ROW_STREAMER);
        menu.selected = ROW_NET_LATENCY;
        assert!(!menu.handle(Action::Left, &mut config));
        for _ in 0..26 {
            assert!(menu.handle(Action::Right, &mut config));
        }
        assert_eq!(config.input_buffer, 25);
        assert!(!menu.handle(Action::Right, &mut config));
        for _ in 0..26 {
            assert!(menu.handle(Action::Left, &mut config));
        }
        assert_eq!(config.input_buffer, -1);
        menu.handle(Action::Up, &mut config);
        assert_eq!(menu.selected, ROW_PEER);
    }

    #[test]
    fn streamer_toggles_and_starts_off() {
        let mut menu = Menu {
            open: true,
            selected: ROW_STREAMER,
        };
        let mut config = Config::default();
        assert!(!config.streamer);
        assert!(menu.handle(Action::Right, &mut config));
        assert!(config.streamer);
        assert!(menu.handle(Action::Left, &mut config));
        assert!(!config.streamer);
    }

    #[test]
    fn profile_cycles_auto_then_the_four_manual_modes_and_stealth_starts_on() {
        let mut menu = Menu {
            open: true,
            selected: ROW_NET_PROFILE,
        };
        let mut config = Config::default();
        assert!(config.auto_profile);
        assert_eq!(config.render_mode, RenderMode::Vanilla);
        let before = config.clone();
        assert!(menu.handle(Action::Right, &mut config));
        assert!(!config.auto_profile);
        assert_eq!(config.render_mode, RenderMode::Vanilla);
        assert_eq!(
            profile_action(&before, &config),
            ProfileAction::SelectManual(RenderMode::Vanilla)
        );
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.render_mode, RenderMode::LessLag);
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.render_mode, RenderMode::LessLagUltra);
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.render_mode, RenderMode::LessLagDoubles);
        let saved = config.clone();
        assert!(menu.handle(Action::Right, &mut config));
        assert!(config.auto_profile);
        assert_eq!(config.render_mode, saved.render_mode);
        assert_eq!(profile_action(&saved, &config), ProfileAction::EnableAuto);
        assert!(menu.handle(Action::Left, &mut config));
        assert!(!config.auto_profile);
        assert_eq!(config.render_mode, RenderMode::Vanilla);
        config.auto_profile = false;
        config.render_mode = RenderMode::LessLagCore;
        assert!(menu.handle(Action::Right, &mut config));
        assert!(!config.auto_profile);
        assert_eq!(config.render_mode, RenderMode::Vanilla);
        config.render_mode = RenderMode::DoublesCore;
        assert!(menu.handle(Action::Left, &mut config));
        assert_eq!(config.render_mode, RenderMode::Vanilla);
        let stealth_only = config.clone();
        config.stealth = !config.stealth;
        assert_eq!(
            profile_action(&stealth_only, &config),
            ProfileAction::Unchanged
        );
        config.stealth = stealth_only.stealth;
        let machine = crate::render_policy::Machine::confirmed(
            crate::render_policy::Profile::Doubles,
        );
        assert_eq!(
            net_profile_status(
                RenderMode::LessLagDoubles,
                RenderMode::Vanilla,
                3,
                machine,
                true
            ),
            "Active"
        );
        assert_ne!(
            net_profile_status(RenderMode::Vanilla, RenderMode::Vanilla, 3, machine, true),
            "Active"
        );
        assert_eq!(
            crate::auto_policy::profile_row_label(
                true,
                true,
                RenderMode::Vanilla,
                RenderMode::LessLagDoubles
            ),
            "Auto:LLDbl"
        );
        assert_eq!(platform_label(crate::config::Platform::Console), "Console");
        assert_eq!(platform_label(crate::config::Platform::Emulator), "Emulator");
        let passthrough = crate::render_policy::Machine::passthrough();
        assert_eq!(
            net_profile_value(RenderMode::LessLag, RenderMode::Vanilla, 3, passthrough, true),
            "LessLag*"
        );
        assert_eq!(
            net_profile_value(RenderMode::Vanilla, RenderMode::Vanilla, 3, passthrough, true),
            "Vanilla"
        );
        assert_eq!(
            net_profile_value(RenderMode::LessLag, RenderMode::Vanilla, 3, passthrough, false),
            "LessLag"
        );
        menu.selected = ROW_STEALTH;
        assert!(config.stealth);
        assert_eq!(stealth_value(config.stealth, None), "On");
        assert!(menu.handle(Action::Right, &mut config));
        assert!(!config.stealth);
        assert_eq!(stealth_value(config.stealth, Some(true)), "Off (reconnect)");
        assert_eq!(stealth_value(config.stealth, None), "Off");
        assert!(menu.handle(Action::Left, &mut config));
        assert!(config.stealth);
        assert_eq!(stealth_value(config.stealth, Some(false)), "On (reconnect)");
        assert_eq!(stealth_value(config.stealth, None), "On");
    }
}
