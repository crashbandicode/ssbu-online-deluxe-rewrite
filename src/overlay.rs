//! Scene-independent controller UX and bounded frame statistics.
use crate::{
    config::Config,
    menu::{Action, Menu},
};

pub(crate) const L: u64 = 1 << 6;
pub(crate) const R: u64 = 1 << 7;
const ZL: u64 = 1 << 8;
const ZR: u64 = 1 << 9;
const Y: u64 = 1 << 3;
const PLUS_MINUS: u64 = (1 << 10) | (1 << 11);
const LEFT: u64 = 1 << 12;
pub(crate) const UP: u64 = 1 << 13;
const RIGHT: u64 = 1 << 14;
const DOWN: u64 = 1 << 15;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum View {
    #[default]
    Hidden,
    Settings,
    Performance,
    Debug,
}

#[derive(Default)]
pub struct Overlay {
    pub view: View,
    pub menu: Menu,
    previous_buttons: u64,
}

impl Overlay {
    /// Match the fork: chord cycles views; unmodified D-pad navigates the table.
    /// Edges prevent a held direction from changing a setting every draw call.
    pub fn poll(&mut self, buttons: u64, config: &mut Config) -> bool {
        let pressed = buttons & !self.previous_buttons;
        self.previous_buttons = buttons;
        let shoulders = buttons & (L | R) == L | R;
        let triggers = buttons & (ZL | ZR) == ZL | ZR;
        if shoulders && buttons & (ZL | ZR) != 0 && pressed & Y != 0 {
            config.streamer = !config.streamer;
            return true;
        }
        if self.view != View::Settings && (shoulders || triggers) {
            if pressed & LEFT != 0 {
                crate::peer_transport::cycle_peer(-1);
                return false;
            }
            if pressed & RIGHT != 0 {
                crate::peer_transport::cycle_peer(1);
                return false;
            }
        }
        let chord = shoulders || triggers;
        if chord && pressed & DOWN != 0 {
            self.view = match (self.view, buttons & PLUS_MINUS != 0) {
                (View::Hidden, _) => View::Settings,
                (View::Settings, true) => View::Debug,
                (View::Settings | View::Debug, _) => View::Performance,
                (View::Performance, _) => View::Hidden,
            };
            self.menu.open = self.view == View::Settings;
            return false;
        }
        if self.view != View::Settings {
            return false;
        }
        let action = if pressed & UP != 0 {
            Action::Up
        } else if pressed & DOWN != 0 {
            Action::Down
        } else if pressed & LEFT != 0 {
            Action::Left
        } else if pressed & RIGHT != 0 {
            Action::Right
        } else {
            return false;
        };
        self.menu.handle(action, config)
    }
}

pub const FRAME_HISTORY: usize = 180;
pub struct FrameTimes {
    pub values: [f32; FRAME_HISTORY],
    pub count: usize,
    pub next: usize,
}

impl Default for FrameTimes {
    fn default() -> Self {
        Self {
            values: [0.0; FRAME_HISTORY],
            count: 0,
            next: 0,
        }
    }
}
impl FrameTimes {
    pub fn push(&mut self, milliseconds: f32) {
        if !milliseconds.is_finite() || milliseconds <= 0.0 {
            return;
        }
        self.values[self.next] = milliseconds;
        self.next = (self.next + 1) % FRAME_HISTORY;
        self.count = (self.count + 1).min(FRAME_HISTORY);
    }
    pub fn average(&self) -> Option<f32> {
        (self.count > 0).then(|| self.values[..self.count].iter().sum::<f32>() / self.count as f32)
    }
    pub fn plot_offset(&self) -> usize {
        if self.count == FRAME_HISTORY {
            self.next
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FxMode;
    #[test]
    fn view_cycle_consumes_toggle_down_and_edits_only_in_settings() {
        let mut ui = Overlay::default();
        let mut config = Config::default();
        ui.poll(RIGHT, &mut config);
        assert_eq!(config.effects, FxMode::On);
        ui.poll(L | R | DOWN, &mut config);
        assert_eq!(
            (ui.view, ui.menu.selected),
            (View::Settings, crate::menu::ROW_PEER)
        );
        ui.poll(0, &mut config);
        ui.poll(DOWN, &mut config);
        ui.poll(0, &mut config);
        assert!(ui.poll(RIGHT, &mut config));
        assert_eq!(config.input_buffer, 0);
        assert!(!ui.poll(RIGHT, &mut config));
        ui.poll(0, &mut config);
        ui.poll(DOWN, &mut config);
        ui.poll(0, &mut config);
        ui.poll(DOWN, &mut config);
        ui.poll(0, &mut config);
        assert_eq!(ui.menu.selected, crate::menu::ROW_FX);
        assert!(ui.poll(RIGHT, &mut config));
        assert_eq!(config.effects, FxMode::Off);
        assert!(!ui.poll(RIGHT, &mut config));
        ui.poll(L | R | DOWN, &mut config);
        assert_eq!(ui.view, View::Performance);
        assert!(!ui.poll(RIGHT, &mut config));
        ui.poll(L | R | DOWN, &mut config);
        assert_eq!(ui.view, View::Hidden);
    }
    #[test]
    fn both_shortcuts_and_debug_modifier_preserve_settings() {
        let mut ui = Overlay::default();
        let mut config = Config::default();
        ui.poll(ZL | ZR | DOWN, &mut config);
        ui.poll(0, &mut config);
        ui.poll(ZL | ZR | DOWN | PLUS_MINUS, &mut config);
        assert_eq!(ui.view, View::Debug);
        assert_eq!(config, Config::default());
    }
    #[test]
    fn graph_is_bounded_and_wraps_in_time_order() {
        let mut f = FrameTimes::default();
        assert_eq!(f.average(), None);
        f.push(f32::NAN);
        f.push(0.0);
        assert_eq!(f.count, 0);
        for n in 1..=200 {
            f.push(n as f32);
        }
        assert_eq!((f.count, f.plot_offset()), (180, 20));
        assert_eq!(f.values[f.plot_offset()], 21.0);
        assert_eq!(f.average(), Some(110.5));
    }

    #[test]
    fn streamer_chord_persists_and_peer_cycle_does_not() {
        let mut ui = Overlay::default();
        let mut config = Config::default();
        assert!(!config.streamer);
        assert!(ui.poll(L | R | ZL | Y, &mut config));
        assert!(config.streamer);
        ui.poll(0, &mut config);
        assert!(ui.poll(L | R | ZR | Y, &mut config));
        assert!(!config.streamer);
        ui.poll(0, &mut config);
        let before = config.clone();
        assert!(!ui.poll(L | R | RIGHT, &mut config));
        assert_eq!(config, before);
        assert!(!ui.poll(ZL | ZR | LEFT, &mut config));
        assert_eq!(ui.view, View::Hidden);
    }
}
