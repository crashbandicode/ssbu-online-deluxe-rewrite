//! Scene-independent controller UX and bounded frame statistics.
use crate::{
    config::Config,
    menu::{Action, Menu, NativePads},
};

pub(crate) const L: u64 = 1 << 6;
pub(crate) const R: u64 = 1 << 7;
const ZL: u64 = 1 << 8;
const ZR: u64 = 1 << 9;
const X: u64 = 1 << 2;
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
    /// Appended after Debug so recorded view numbers of older builds hold.
    Plugins,
}

/// Chord + Down walks Hidden → Settings → Plugins → Performance → Hidden.
/// Plugins is skipped while no plugin has registered. Plus/Minus held from
/// Settings or Plugins opens Debug, which then continues to Performance.
pub fn next_view(view: View, debug_modifier: bool, plugins_available: bool) -> View {
    match (view, debug_modifier) {
        (View::Hidden, _) => View::Settings,
        (View::Settings | View::Plugins, true) => View::Debug,
        (View::Settings, false) if plugins_available => View::Plugins,
        (View::Settings | View::Plugins | View::Debug, _) => View::Performance,
        (View::Performance, _) => View::Hidden,
    }
}

#[derive(Default)]
pub struct Overlay {
    pub view: View,
    pub menu: Menu,
    /// Selected flattened row on the Plugins page (`plugin_api::plugin_rows`).
    pub plugin_row: usize,
    previous_buttons: u64,
}

impl Overlay {
    /// Diagnostic edge for a view-command attempt, sampled before `poll_native`.
    /// Held chords and ordinary gameplay do not emit per-frame records. Bit 0
    /// means a new Down press; bit 1 means a new Plus/Minus press. Zero records
    /// a relevant release/chord completion, not an input action.
    #[cfg(any(target_os = "switch", test))]
    pub(crate) fn command_edge(&self, buttons: u64) -> Option<u8> {
        const COMMAND: u64 = L | R | ZL | ZR | PLUS_MINUS | DOWN;
        let previous = self.previous_buttons;
        let chord = |value: u64| value & (L | R) == L | R || value & (ZL | ZR) == ZL | ZR;
        if (buttons ^ previous) & COMMAND == 0
            || !(chord(buttons) || chord(previous))
            || (buttons | previous) & (PLUS_MINUS | DOWN) == 0
        {
            return None;
        }
        let pressed = buttons & !previous;
        Some(u8::from(pressed & DOWN != 0) | (u8::from(pressed & PLUS_MINUS != 0) << 1))
    }

    /// Match the fork: chord cycles views; unmodified D-pad navigates the table.
    /// Edges prevent a held direction from changing a setting every draw call.
    pub fn poll(&mut self, buttons: u64, config: &mut Config) -> bool {
        self.poll_native(buttons, config, NativePads::off())
    }

    /// Same edge as [`Self::poll`]. Native panes do not sample this again.
    /// A Settings view, or a held shoulder/trigger chord, consumes the event first.
    pub fn poll_native(&mut self, buttons: u64, config: &mut Config, native: NativePads) -> bool {
        self.poll_with(buttons, config, native, &crate::plugin_api::REGISTRY)
    }

    /// [`Self::poll_native`] against an explicit plugin registry.
    pub fn poll_with(
        &mut self,
        buttons: u64,
        config: &mut Config,
        native: NativePads,
        plugins: &crate::plugin_api::Registry,
    ) -> bool {
        let pressed = buttons & !self.previous_buttons;
        self.previous_buttons = buttons;
        let shoulders = buttons & (L | R) == L | R;
        let triggers = buttons & (ZL | ZR) == ZL | ZR;
        if shoulders && buttons & (ZL | ZR) != 0 && pressed & Y != 0 {
            config.streamer = !config.streamer;
            return true;
        }
        // Fork FPS++ chord: L+R+ZL+X or L+R+ZR+X. Stored only on emulator.
        if shoulders && buttons & (ZL | ZR) != 0 && pressed & X != 0 {
            let next = crate::fps_boost::saved_after_toggle(
                config.platform == crate::config::Platform::Emulator,
                config.fps_boost,
            );
            if next == config.fps_boost {
                return false;
            }
            config.fps_boost = next;
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
            self.view = next_view(self.view, buttons & PLUS_MINUS != 0, plugins.count() > 0);
            self.menu.open = self.view == View::Settings;
            if self.menu.open {
                let gate = crate::menu::row_gate(
                    crate::peer_transport::opponent_connected(),
                    config.platform,
                );
                if !crate::menu::row_visible(self.menu.selected, gate) {
                    self.menu.selected = crate::menu::move_row(self.menu.selected, 1, gate);
                }
            }
            return false;
        }
        if self.view == View::Plugins {
            // Plugin rows never touch `Config`; the plugin owns the change.
            let rows = crate::plugin_api::plugin_rows(&plugins.snapshot());
            if rows.is_empty() {
                return false;
            }
            self.plugin_row = self.plugin_row.min(rows.len() - 1);
            if pressed & UP != 0 {
                self.plugin_row =
                    crate::plugin_api::move_plugin_row(self.plugin_row, -1, rows.len());
            } else if pressed & DOWN != 0 {
                self.plugin_row =
                    crate::plugin_api::move_plugin_row(self.plugin_row, 1, rows.len());
            } else if !chord && pressed & (LEFT | RIGHT) != 0 {
                let direction: i8 = if pressed & LEFT != 0 { -1 } else { 1 };
                let row = rows[self.plugin_row];
                plugins.step(row.plugin, row.option, direction);
            }
            return false;
        }
        if self.view != View::Settings {
            if native.allow && !chord && !config.streamer {
                let latency_dir = if pressed & LEFT != 0 {
                    -1
                } else if pressed & RIGHT != 0 {
                    1
                } else {
                    0
                };
                let profile_dir = if pressed & DOWN != 0 {
                    -1
                } else if pressed & UP != 0 {
                    1
                } else {
                    0
                };
                // A live pane that has not run since the last draw must not eat
                // the edge. A press with no pane at all is consumed.
                if !native.fresh
                    && native.surface != crate::menu::NativeSurface::None
                    && (latency_dir != 0 || profile_dir != 0)
                {
                    self.previous_buttons &= !(LEFT | RIGHT | DOWN | UP);
                    return false;
                }
                if !native.fresh {
                    return false;
                }
                return crate::menu::apply_native_edges(
                    config,
                    native.surface,
                    latency_dir,
                    profile_dir,
                );
            }
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
        self.menu
            .handle_peers(action, config, crate::peer_transport::opponent_connected())
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
        let _gate = crate::peer_transport::global_gate();
        crate::peer_transport::clear_session();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        ui.poll(RIGHT, &mut config);
        assert_eq!(config.effects, FxMode::On);
        ui.poll(L | R | DOWN, &mut config);
        assert_eq!(
            (ui.view, ui.menu.selected),
            (View::Settings, crate::menu::ROW_NET_LATENCY)
        );
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
        let _gate = crate::peer_transport::global_gate();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        ui.poll(ZL | ZR | DOWN, &mut config);
        ui.poll(0, &mut config);
        ui.poll(ZL | ZR | DOWN | PLUS_MINUS, &mut config);
        assert_eq!(ui.view, View::Debug);
        assert_eq!(config, Config::default());
    }

    #[test]
    fn individual_debug_modifiers_work_with_down_release_and_held_shoulders() {
        let _gate = crate::peer_transport::global_gate();
        crate::peer_transport::clear_session();
        for chord in [L | R, ZL | ZR] {
            for modifier in [1 << 10, 1 << 11] {
                let mut ui = Overlay::default();
                let mut config = Config::default();
                ui.poll(chord | DOWN, &mut config);
                assert_eq!(ui.view, View::Settings);
                ui.poll(chord, &mut config);
                ui.poll(chord | modifier, &mut config);
                assert_eq!(ui.view, View::Settings);
                assert_eq!(ui.command_edge(chord | modifier | DOWN), Some(1));
                ui.poll(chord | modifier | DOWN, &mut config);
                assert_eq!(ui.view, View::Debug);
                for _ in 0..10 {
                    assert_eq!(ui.command_edge(chord | modifier | DOWN), None);
                    ui.poll(chord | modifier | DOWN, &mut config);
                    assert_eq!(ui.view, View::Debug);
                }
                assert_eq!(config, Config::default());
            }
        }
    }

    #[test]
    fn command_trace_distinguishes_late_modifier_without_changing_input() {
        let _gate = crate::peer_transport::global_gate();
        crate::peer_transport::clear_session();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        assert_eq!(ui.command_edge(UP), None);
        assert_eq!(ui.command_edge(L | R), None);
        ui.poll(L | R | DOWN, &mut config);
        ui.poll(L | R, &mut config);
        assert_eq!(ui.view, View::Settings);
        assert_eq!(ui.command_edge(L | R | DOWN), Some(1));
        ui.poll(L | R | DOWN, &mut config);
        assert_eq!(ui.view, View::Performance);
        let late_minus = L | R | DOWN | (1 << 11);
        assert_eq!(ui.command_edge(late_minus), Some(2));
        ui.poll(late_minus, &mut config);
        assert_eq!(ui.view, View::Performance);
        assert_eq!(ui.command_edge(late_minus), None);
        assert_eq!(ui.command_edge(0), Some(0));
        ui.poll(0, &mut config);
        assert_eq!(ui.command_edge(UP), None);
        assert_eq!(config, Config::default());
    }
    #[test]
    fn plugins_view_enters_the_cycle_only_after_a_registration() {
        for debug in [false, true] {
            assert_eq!(next_view(View::Hidden, debug, false), View::Settings);
            assert_eq!(next_view(View::Hidden, debug, true), View::Settings);
            assert_eq!(next_view(View::Debug, debug, true), View::Performance);
            assert_eq!(next_view(View::Performance, debug, true), View::Hidden);
        }
        assert_eq!(next_view(View::Settings, false, false), View::Performance);
        assert_eq!(next_view(View::Settings, false, true), View::Plugins);
        assert_eq!(next_view(View::Settings, true, true), View::Debug);
        assert_eq!(next_view(View::Plugins, false, true), View::Performance);
        assert_eq!(next_view(View::Plugins, true, true), View::Debug);
        assert_eq!(View::Plugins as u8, 4);
        assert_eq!(View::Debug as u8, 3);
    }

    static PLUGIN_STATE: crate::plugin_api::testing::FakePlugin =
        crate::plugin_api::testing::FakePlugin::new(
            [0, 1, 0, 0],
            deluxe_plugin_abi::REQUEST_APPLIED,
        );

    #[test]
    fn plugins_page_navigates_rows_and_requests_without_touching_config() {
        let _gate = crate::peer_transport::global_gate();
        crate::peer_transport::clear_session();
        let registry = crate::plugin_api::Registry::new();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        let css = crate::menu::NativePads {
            allow: true,
            surface: crate::menu::NativeSurface::Css,
            fresh: true,
        };
        // No plugin: the chord skips straight to Performance.
        ui.poll_with(L | R | DOWN, &mut config, css, &registry);
        ui.poll_with(L | R, &mut config, css, &registry);
        ui.poll_with(L | R | DOWN, &mut config, css, &registry);
        assert_eq!(ui.view, View::Performance);
        ui.poll_with(L | R, &mut config, css, &registry);
        ui.poll_with(L | R | DOWN, &mut config, css, &registry);
        assert_eq!(ui.view, View::Hidden);
        ui.poll_with(0, &mut config, css, &registry);

        assert_eq!(
            crate::plugin_api::testing::register_fake(
                &registry,
                "demo",
                "Demo",
                &[
                    ("enabled", "Enabled", &["Off", "On"]),
                    ("mode", "Mode", &["A", "B", "C"]),
                    ("status", "Status", &["Ready"]),
                ],
                &PLUGIN_STATE,
            ),
            0
        );
        ui.poll_with(L | R | DOWN, &mut config, css, &registry);
        assert_eq!(ui.view, View::Settings);
        ui.poll_with(L | R, &mut config, css, &registry);
        ui.poll_with(L | R | DOWN, &mut config, css, &registry);
        assert_eq!(ui.view, View::Plugins);
        assert!(!ui.menu.open);
        ui.poll_with(0, &mut config, css, &registry);
        // Unmodified D-pad edits the plugin row, not NetLatency or a native pane.
        assert!(!ui.poll_with(RIGHT, &mut config, css, &registry));
        assert_eq!(config, Config::default());
        let step = registry.take_last_step().unwrap();
        assert_eq!((step.plugin, step.option, step.requested), (0, 0, 1));
        assert_eq!(
            PLUGIN_STATE.values[0].load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        // A held Right does not repeat.
        assert!(!ui.poll_with(RIGHT, &mut config, css, &registry));
        assert_eq!(registry.take_last_step(), None);
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(DOWN, &mut config, css, &registry);
        assert_eq!(ui.plugin_row, 1);
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(LEFT, &mut config, css, &registry);
        let step = registry.take_last_step().unwrap();
        assert_eq!((step.option, step.requested), (1, 0));
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(DOWN, &mut config, css, &registry);
        assert_eq!(ui.plugin_row, 2);
        ui.poll_with(0, &mut config, css, &registry);
        // The single-choice status row is read-only.
        ui.poll_with(RIGHT, &mut config, css, &registry);
        assert_eq!(registry.take_last_step(), None);
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(DOWN, &mut config, css, &registry);
        assert_eq!(ui.plugin_row, 0);
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(UP, &mut config, css, &registry);
        assert_eq!(ui.plugin_row, 2);
        ui.poll_with(0, &mut config, css, &registry);
        // Chord + Left/Right still cycles the peer rather than a plugin.
        ui.poll_with(L | R | RIGHT, &mut config, css, &registry);
        assert_eq!(registry.take_last_step(), None);
        ui.poll_with(L | R, &mut config, css, &registry);
        ui.poll_with(L | R | DOWN, &mut config, css, &registry);
        assert_eq!(ui.view, View::Performance);
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(L | R | DOWN, &mut config, css, &registry);
        assert_eq!(ui.view, View::Hidden);
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(L | R | DOWN, &mut config, css, &registry);
        ui.poll_with(L | R, &mut config, css, &registry);
        ui.poll_with(L | R | DOWN | PLUS_MINUS, &mut config, css, &registry);
        assert_eq!(ui.view, View::Debug);
        assert_eq!(config, Config::default());
        PLUGIN_STATE.values[0].store(0, std::sync::atomic::Ordering::Relaxed);
        PLUGIN_STATE.values[1].store(1, std::sync::atomic::Ordering::Relaxed);
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
        let _gate = crate::peer_transport::global_gate();
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

    #[test]
    fn fps_chord_toggles_only_on_emulator() {
        let _gate = crate::peer_transport::global_gate();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        assert!(!ui.poll(L | R | ZL | X, &mut config));
        assert!(!config.fps_boost);
        config.platform = crate::config::Platform::Emulator;
        ui.poll(0, &mut config);
        assert!(ui.poll(L | R | ZL | X, &mut config));
        assert!(config.fps_boost);
        ui.poll(0, &mut config);
        assert!(ui.poll(L | R | ZR | X, &mut config));
        assert!(!config.fps_boost);
        config.fps_boost = true;
        config.platform = crate::config::Platform::Console;
        ui.poll(0, &mut config);
        assert!(ui.poll(L | R | ZL | X, &mut config));
        assert!(!config.fps_boost);
    }

    fn native(surface: crate::menu::NativeSurface, allow: bool) -> crate::menu::NativePads {
        crate::menu::NativePads {
            allow,
            surface,
            fresh: true,
        }
    }

    #[test]
    fn hidden_native_dpad_edits_once_and_settings_keeps_the_same_edge() {
        let _gate = crate::peer_transport::global_gate();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        let css = native(crate::menu::NativeSurface::Css, true);
        assert!(ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
        assert!(!ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
        ui.poll_native(0, &mut config, css);
        assert!(ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 1);
        ui.poll_native(0, &mut config, css);
        assert!(!ui.poll_native(
            RIGHT,
            &mut config,
            native(crate::menu::NativeSurface::Css, false)
        ));
        assert_eq!(config.input_buffer, 1);
        ui.poll_native(0, &mut config, css);
        let before = config.clone();
        assert!(!ui.poll_native(L | R | RIGHT, &mut config, css));
        assert_eq!(config, before);
        assert!(!ui.poll_native(L | R | UP, &mut config, css));
        assert!(config.auto_profile);
        ui.poll_native(0, &mut config, css);
        ui.poll_native(L | R | DOWN, &mut config, css);
        assert_eq!(ui.view, View::Settings);
        ui.menu.selected = crate::menu::ROW_NET_LATENCY;
        ui.poll_native(0, &mut config, css);
        assert!(ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 2);
        assert!(!ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 2);
    }

    #[test]
    fn lobby_native_dpad_changes_latency_only() {
        let _gate = crate::peer_transport::global_gate();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        let lobby = native(crate::menu::NativeSurface::LocalLobby, true);
        assert!(!ui.poll_native(UP, &mut config, lobby));
        assert!(config.auto_profile);
        assert_eq!(config.render_mode, crate::config::RenderMode::Vanilla);
        ui.poll_native(0, &mut config, lobby);
        assert!(ui.poll_native(RIGHT, &mut config, lobby));
        assert_eq!(config.input_buffer, 0);
        assert!(config.auto_profile);
    }

    #[test]
    fn a_stale_live_pane_keeps_one_edge_and_no_pane_consumes_it() {
        let _gate = crate::peer_transport::global_gate();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        let css = native(crate::menu::NativeSurface::Css, true);
        let waiting = crate::menu::NativePads {
            allow: true,
            surface: crate::menu::NativeSurface::Css,
            fresh: false,
        };
        let absent = crate::menu::NativePads {
            allow: true,
            surface: crate::menu::NativeSurface::None,
            fresh: false,
        };
        assert!(!ui.poll_native(RIGHT, &mut config, waiting));
        assert_eq!(config.input_buffer, -1);
        assert!(ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
        assert!(!ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
        ui.poll_native(0, &mut config, css);
        assert!(!ui.poll_native(RIGHT, &mut config, absent));
        assert!(!ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
    }

    #[test]
    fn streamer_hides_native_edits_and_a_chord_does_not_edit_twice() {
        let _gate = crate::peer_transport::global_gate();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        let css = native(crate::menu::NativeSurface::Css, true);
        config.streamer = true;
        assert!(!ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, -1);
        config.streamer = false;
        assert!(!ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, -1);
        ui.poll_native(0, &mut config, css);
        config.streamer = true;
        assert!(!ui.poll_native(UP, &mut config, css));
        assert!(config.auto_profile);
        ui.poll_native(0, &mut config, css);
        config.streamer = false;
        let before = config.input_buffer;
        assert!(!ui.poll_native(L | R | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, before);
        ui.poll_native(0, &mut config, css);
        assert!(ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, before + 1);
        assert!(!ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, before + 1);
        ui.view = View::Settings;
        ui.menu.open = true;
        ui.menu.selected = crate::menu::ROW_NET_LATENCY;
        config.streamer = true;
        ui.poll_native(0, &mut config, css);
        assert!(ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, before + 2);
    }
}
