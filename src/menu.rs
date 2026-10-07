//! Portable menu state. Runtime owns controller edges and rendering.

use crate::config::{Config, FxMode, RenderMode};

/// Editable Player Info rows, top to bottom, under Name and Ping.
pub const ROW_NET_LATENCY: usize = 0;
pub const ROW_NET_PROFILE: usize = 1;
pub const ROW_FX: usize = 2;
pub const ROW_STEALTH: usize = 3;
pub const ROW_COUNT: usize = 4;

const MENU_PROFILES: [RenderMode; 3] = [
    RenderMode::Vanilla,
    RenderMode::LessLag,
    RenderMode::LessLagUltra,
];

fn cycle_profile(current: RenderMode, direction: i8) -> RenderMode {
    let Some(index) = MENU_PROFILES
        .iter()
        .position(|mode| *mode == current)
    else {
        return if direction < 0 {
            RenderMode::LessLagUltra
        } else {
            RenderMode::Vanilla
        };
    };
    let next = (index as i8 + direction).rem_euclid(MENU_PROFILES.len() as i8) as usize;
    MENU_PROFILES[next]
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

/// Transmission is not installed. The saved bool is not a live effect.
pub fn stealth_value() -> &'static str {
    "Unavailable"
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
            ROW_NET_LATENCY => {
                let next = (i16::from(config.input_buffer) + i16::from(direction)).clamp(-1, 25) as i8;
                if next == config.input_buffer {
                    return false;
                }
                config.input_buffer = next;
                true
            }
            ROW_NET_PROFILE => {
                let next = cycle_profile(config.render_mode, direction);
                if next == config.render_mode {
                    return false;
                }
                config.render_mode = next;
                true
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
        assert_eq!(menu.selected, ROW_NET_PROFILE);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_FX);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_STEALTH);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        menu.handle(Action::Up, &mut config);
        assert_eq!(menu.selected, ROW_STEALTH);
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
        assert_eq!(menu.selected, ROW_STEALTH);
    }

    #[test]
    fn profile_cycles_three_modes_and_stealth_stays_unavailable() {
        let mut menu = Menu {
            open: true,
            selected: ROW_NET_PROFILE,
        };
        let mut config = Config::default();
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.render_mode, RenderMode::LessLag);
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.render_mode, RenderMode::LessLagUltra);
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.render_mode, RenderMode::Vanilla);
        assert!(menu.handle(Action::Left, &mut config));
        assert_eq!(config.render_mode, RenderMode::LessLagUltra);
        config.render_mode = RenderMode::LessLagCore;
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.render_mode, RenderMode::Vanilla);
        config.render_mode = RenderMode::DoublesCore;
        assert!(menu.handle(Action::Left, &mut config));
        assert_eq!(config.render_mode, RenderMode::LessLagUltra);
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
        assert!(menu.handle(Action::Right, &mut config));
        assert!(config.stealth);
        assert_eq!(stealth_value(), "Unavailable");
        assert!(menu.handle(Action::Left, &mut config));
        assert!(!config.stealth);
        assert_eq!(stealth_value(), "Unavailable");
    }
}
