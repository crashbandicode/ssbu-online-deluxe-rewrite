//! Portable menu state. Runtime owns controller edges and rendering.

use crate::config::{Config, FxMode};

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
            Action::Up | Action::Down => {
                self.selected = (self.selected + 1) % 2;
                return false;
            }
            Action::Left | Action::Right => {}
        }
        if self.selected == 0 {
            config.effects = match config.effects {
                FxMode::On => FxMode::Off,
                FxMode::Off | FxMode::Reduced => FxMode::On,
            };
            return true;
        }
        let direction = if action == Action::Left { -1 } else { 1 };
        let next = (i16::from(config.input_buffer) + direction).clamp(-1, 25) as i8;
        if next == config.input_buffer {
            return false;
        }
        config.input_buffer = next;
        true
    }

    /// Two short lines fit the existing native CSS text pane.
    pub fn summary(&self, config: &Config) -> String {
        let buffer = if config.input_buffer == -1 {
            "Auto".to_owned()
        } else {
            config.input_buffer.to_string()
        };
        if !self.open {
            return format!(
                "Deluxe (Vanilla) | FX {} | Buffer {buffer}",
                config.effects.effective().label()
            );
        }
        let fx_cursor = if self.selected == 0 { ">" } else { " " };
        let buffer_cursor = if self.selected == 0 { " " } else { ">" };
        format!(
            "{fx_cursor} FX: {} (Reduced unavailable)\n{buffer_cursor} Input buffer: {buffer}",
            config.effects.effective().label()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_focuses_fx_and_closed_input_cannot_edit_settings() {
        let mut menu = Menu {
            open: false,
            selected: 1,
        };
        let mut config = Config::default();
        assert!(!menu.handle(Action::Right, &mut config));
        assert_eq!(config, Config::default());
        assert!(!menu.handle(Action::Toggle, &mut config));
        assert_eq!(menu.selected, 0);
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.effects, FxMode::Off);
        menu.handle(Action::Close, &mut config);
        assert!(!menu.open);
    }

    #[test]
    fn fx_cycles_only_available_modes_even_after_unsupported_state() {
        let mut menu = Menu {
            open: true,
            selected: 0,
        };
        let mut config = Config {
            effects: FxMode::Reduced,
            ..Config::default()
        };
        for expected in [FxMode::On, FxMode::Off, FxMode::On, FxMode::Off] {
            assert!(menu.handle(Action::Left, &mut config));
            assert_eq!(config.effects, expected);
        }
        assert!(menu.summary(&config).contains("Reduced unavailable"));
        assert!(menu.summary(&config).len() <= 120);
    }

    #[test]
    fn navigation_wraps_and_buffer_saturates_without_false_change_reports() {
        let mut menu = Menu {
            open: true,
            selected: 0,
        };
        let mut config = Config::default();
        menu.handle(Action::Up, &mut config);
        assert_eq!(menu.selected, 1);
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
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, 0);
    }
}
