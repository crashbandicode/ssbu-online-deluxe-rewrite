//! Scene-independent controller UX and bounded frame statistics.
use crate::{
    config::{Config, HiddenPages},
    menu::{Action, Menu, NativePads},
};

pub(crate) const L: u64 = 1 << 6;
pub(crate) const R: u64 = 1 << 7;
const ZL: u64 = 1 << 8;
const ZR: u64 = 1 << 9;
/// L+R+X toggles Vanilla+ (0.5.6). The fork's retired FPS++ chord used X with
/// a trigger held; that combination still does nothing.
const X: u64 = 1 << 2;
const Y: u64 = 1 << 3;
const PLUS_MINUS: u64 = (1 << 10) | (1 << 11);
pub(crate) const LEFT: u64 = 1 << 12;
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
    /// One plugin's own page (`PLUGIN_FLAG_OWN_PAGE`, 0.5.0); `Overlay::page`
    /// says which. Appended after Plugins for the same reason.
    PluginPage,
}

/// Chord + Down walks Hidden → Settings → Plugins → each own page →
/// Performance → Hidden. The shared Plugins page is skipped while no plugin
/// has rows on it, and the own pages are the `page_count` plugins that asked
/// for one (`page` is the current index). `hidden` (the INI's `hide_pages`,
/// 0.5.3) drops the shared Plugins page and/or Performance from the walk, so
/// a plugin's own page sits right after Settings and the window hides right
/// after it. Plus/Minus held with the chord opens Debug from Settings,
/// Plugins or an own page and keeps Debug open while it stays held; Debug
/// then continues to Performance (or hides) on a plain chord. Returns the
/// next view and own-page index.
pub fn next_view(
    view: View,
    page: usize,
    debug_modifier: bool,
    shared_available: bool,
    page_count: usize,
    hidden: HiddenPages,
) -> (View, usize) {
    let after_pages = if hidden.performance {
        (View::Hidden, 0)
    } else {
        (View::Performance, 0)
    };
    match (view, debug_modifier) {
        (View::Hidden, _) => (View::Settings, 0),
        (View::Settings | View::Plugins | View::PluginPage, true) => (View::Debug, page),
        (View::Debug, true) => (View::Debug, page),
        (View::Settings, false) if shared_available && !hidden.plugins => (View::Plugins, 0),
        (View::Settings | View::Plugins, false) if page_count > 0 && !hidden.own_pages => {
            (View::PluginPage, 0)
        }
        (View::PluginPage, false) if page + 1 < page_count => (View::PluginPage, page + 1),
        (View::Settings | View::Plugins | View::PluginPage | View::Debug, _) => after_pages,
        (View::Performance, _) => (View::Hidden, 0),
    }
}

/// Which window command a sample completes, if any. A command fires once
/// when its whole combination is held, in any press order, and cannot fire
/// again until at least one of its buttons is released. Hardware logs showed
/// D-pad Down registering a frame before both bumpers, which the old
/// "Down edge while the chord is held" rule silently ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Chord + Down: cycle views (Plus/Minus also held selects Debug).
    Cycle { debug_modifier: bool },
    /// Chord + Plus/Minus without Down: open Debug, or hide it when open.
    Debug,
}

/// Latched combination state between samples.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommandLatch {
    cycle_held: bool,
    debug_held: bool,
}

impl CommandLatch {
    pub fn sample(&mut self, buttons: u64) -> Option<Command> {
        // L+R only (0.5.3): ZL+ZR no longer opens the window, so a trigger
        // chord during play cannot bring it up and ZR stays the edit hold.
        let chord = buttons & (L | R) == L | R;
        let cycle = chord && buttons & DOWN != 0;
        let debug = chord && buttons & PLUS_MINUS != 0 && buttons & DOWN == 0;
        let fire_cycle = cycle && !self.cycle_held;
        let fire_debug = debug && !self.debug_held;
        self.cycle_held = cycle;
        // A Debug toggle that was already consumed must not re-fire when Down
        // is added and removed while Plus/Minus stays held.
        self.debug_held = chord && buttons & PLUS_MINUS != 0;
        if fire_cycle {
            Some(Command::Cycle {
                debug_modifier: buttons & PLUS_MINUS != 0,
            })
        } else if fire_debug {
            Some(Command::Debug)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct Overlay {
    pub view: View,
    pub menu: Menu,
    /// Selected row on the Plugins page or the current own page (an index
    /// into `plugin_api::rows_for`, which on the shared page counts the group
    /// rows); reset when the view changes.
    pub plugin_row: usize,
    /// Which own page is shown while `view == View::PluginPage`
    /// (index into `plugin_api::page_plugins`).
    pub page: usize,
    /// A Left/Right edit in the open window needs ZR held: the D-pad also
    /// taunts, moves menus and the character-select hand, and a plain press
    /// must not change a setting. The runtime sets this in every scene since
    /// 0.5.6 (operator rule; until 0.5.5 only while a fight was live). ZR,
    /// not ZL, because ZL + D-pad Left/Right in Training swaps which fighter
    /// the player controls. Up/Down still move the selection.
    pub edit_hold: bool,
    previous_buttons: u64,
    latch: CommandLatch,
}

/// The hold that unlocks a native pane edit (character select, window
/// closed): ZL alone (ZR is the in-window edit hold; ZL+ZR together is
/// neither, and since 0.5.3 it is not the window chord either).
const fn native_modifier(buttons: u64) -> bool {
    buttons & ZL != 0 && buttons & ZR == 0
}

/// The hold that unlocks a value edit in the open window while `edit_hold`
/// is on: ZR alone.
const fn edit_modifier(buttons: u64) -> bool {
    buttons & ZR != 0 && buttons & ZL == 0
}

impl Overlay {
    /// Diagnostic edge for a view-command attempt, sampled before `poll_native`.
    /// Held chords and ordinary gameplay do not emit per-frame records. Bit 0
    /// means a new Down press; bit 1 means a new Plus/Minus press. Zero records
    /// a relevant release/chord completion, not an input action.
    #[cfg(any(target_os = "switch", test))]
    pub(crate) fn command_edge(&self, buttons: u64) -> Option<u8> {
        const COMMAND: u64 = L | R | PLUS_MINUS | DOWN;
        let previous = self.previous_buttons;
        let chord = |value: u64| value & (L | R) == L | R;
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
        if shoulders && buttons & (ZL | ZR) != 0 && pressed & Y != 0 {
            config.streamer = !config.streamer;
            return true;
        }
        // Vanilla+ (0.5.6): L+R+X with neither trigger held flips the flag
        // live and saves it. It is a flag on Vanilla, not a NetProfile entry,
        // so the row keeps its three choices. The fork's FPS++ chord
        // (L+R+ZL/ZR+X) left with FPS++ on 2026-10-17 and stays inert.
        if shoulders && buttons & (ZL | ZR) == 0 && pressed & X != 0 {
            config.vanilla_plus = !config.vanilla_plus;
            return true;
        }
        if self.view != View::Settings && shoulders {
            if pressed & LEFT != 0 {
                crate::peer_transport::cycle_peer(-1);
                return false;
            }
            if pressed & RIGHT != 0 {
                crate::peer_transport::cycle_peer(1);
                return false;
            }
        }
        let chord = shoulders;
        let command = self.latch.sample(buttons);
        if let Some(command) = command {
            let previous = (self.view, self.page);
            (self.view, self.page) = match command {
                Command::Cycle { debug_modifier } => {
                    let snapshot = plugins.snapshot();
                    next_view(
                        self.view,
                        self.page,
                        debug_modifier,
                        crate::plugin_api::has_shared_rows(&snapshot),
                        crate::plugin_api::page_plugins(&snapshot).len(),
                        config.hide_pages,
                    )
                }
                Command::Debug if self.view == View::Debug => (View::Hidden, 0),
                Command::Debug => (View::Debug, self.page),
            };
            if (self.view, self.page) != previous {
                self.plugin_row = 0;
            }
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
        if matches!(self.view, View::Plugins | View::PluginPage) {
            // Plugin rows never touch `Config`; the plugin owns the change.
            // Group and header rows never hold the selection.
            let page = (self.view == View::PluginPage).then_some(self.page);
            let snapshot = plugins.snapshot();
            let rows = crate::plugin_api::rows_for(&snapshot, page);
            let direction: i8 = if pressed & UP != 0 {
                -1
            } else if pressed & DOWN != 0 {
                1
            } else {
                0
            };
            self.plugin_row =
                crate::plugin_api::move_selection(&snapshot, &rows, self.plugin_row, direction);
            if direction == 0
                && !chord
                && pressed & (LEFT | RIGHT) != 0
                && !(self.edit_hold && !edit_modifier(buttons))
            {
                if let Some(crate::plugin_api::PageRow::Option { plugin, option }) =
                    rows.get(self.plugin_row)
                {
                    plugins.step(*plugin, *option, if pressed & LEFT != 0 { -1 } else { 1 });
                }
            }
            return false;
        }
        if self.view != View::Settings {
            // Native pane edits need ZL held: the D-pad also moves the
            // character-select hand, and plain presses rewrote the profile and
            // the buffer on the console (23 unintended saves on 2026-10-14).
            let native_modifier = native_modifier(buttons);
            if native.allow && !chord && !config.streamer && native_modifier {
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
        } else if pressed & (LEFT | RIGHT) != 0 {
            // Mid-game a value edit needs ZR held (see `edit_hold`).
            if self.edit_hold && !edit_modifier(buttons) {
                return false;
            }
            if pressed & LEFT != 0 {
                Action::Left
            } else {
                Action::Right
            }
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
    #[test]
    fn view_cycle_consumes_toggle_down_and_edits_only_in_settings() {
        let _gate = crate::peer_transport::global_gate();
        crate::peer_transport::clear_session();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        ui.poll(RIGHT, &mut config);
        assert_eq!(config, Config::default());
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
        assert_eq!(ui.menu.selected, crate::menu::ROW_STEALTH);
        assert!(ui.poll(RIGHT, &mut config));
        assert!(!config.stealth);
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
        // ZL+ZR is not a window chord any more (0.5.3).
        ui.poll(ZL | ZR | DOWN, &mut config);
        assert_eq!(ui.view, View::Hidden);
        ui.poll(0, &mut config);
        ui.poll(ZL | ZR | DOWN | PLUS_MINUS, &mut config);
        assert_eq!(ui.view, View::Hidden);
        ui.poll(0, &mut config);
        ui.poll(L | R | DOWN, &mut config);
        ui.poll(0, &mut config);
        ui.poll(L | R | DOWN | PLUS_MINUS, &mut config);
        assert_eq!(ui.view, View::Debug);
        assert_eq!(config, Config::default());
    }

    #[test]
    fn individual_debug_modifiers_work_with_down_release_and_held_shoulders() {
        let _gate = crate::peer_transport::global_gate();
        crate::peer_transport::clear_session();
        for modifier in [1 << 10, 1 << 11] {
            let mut ui = Overlay::default();
            let mut config = Config::default();
            ui.poll(ZL | ZR | modifier, &mut config);
            assert_eq!(ui.view, View::Hidden, "triggers never open Debug");
            ui.poll(0, &mut config);
        }
        for chord in [L | R] {
            for modifier in [1 << 10, 1 << 11] {
                let mut ui = Overlay::default();
                let mut config = Config::default();
                ui.poll(chord | DOWN, &mut config);
                assert_eq!(ui.view, View::Settings);
                ui.poll(chord, &mut config);
                // Chord + Plus/Minus is itself the Debug command now; adding
                // Down while the modifier stays held keeps Debug open.
                ui.poll(chord | modifier, &mut config);
                assert_eq!(ui.view, View::Debug);
                assert_eq!(ui.command_edge(chord | modifier | DOWN), Some(1));
                ui.poll(chord | modifier | DOWN, &mut config);
                assert_eq!(ui.view, View::Debug);
                for _ in 0..10 {
                    assert_eq!(ui.command_edge(chord | modifier | DOWN), None);
                    ui.poll(chord | modifier | DOWN, &mut config);
                    assert_eq!(ui.view, View::Debug);
                }
                // Releasing Down, then the modifier, then pressing it again hides Debug.
                ui.poll(chord | modifier, &mut config);
                assert_eq!(ui.view, View::Debug);
                ui.poll(chord, &mut config);
                ui.poll(chord | modifier, &mut config);
                assert_eq!(ui.view, View::Hidden);
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
    fn window_commands_fire_on_completion_in_any_press_order_and_once_per_hold() {
        let _gate = crate::peer_transport::global_gate();
        crate::peer_transport::clear_session();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        // Hardware order: Down lands first, the bumpers a frame later.
        ui.poll(DOWN, &mut config);
        assert_eq!(ui.view, View::Hidden);
        ui.poll(L | DOWN, &mut config);
        assert_eq!(ui.view, View::Hidden);
        ui.poll(L | R | DOWN, &mut config);
        assert_eq!(ui.view, View::Settings);
        // Holding everything does not repeat.
        for _ in 0..5 {
            ui.poll(L | R | DOWN, &mut config);
            assert_eq!(ui.view, View::Settings);
        }
        // Releasing only Down re-arms the command; the bumpers may stay held.
        ui.poll(L | R, &mut config);
        ui.poll(L | R | DOWN, &mut config);
        assert_eq!(ui.view, View::Performance);
        // Releasing a bumper re-arms it too.
        ui.poll(R | DOWN, &mut config);
        ui.poll(L | R | DOWN, &mut config);
        assert_eq!(ui.view, View::Hidden);
        ui.poll(0, &mut config);
        // Direct Debug toggle: chord + Minus, no Down, from any view.
        ui.poll(L | R | (1 << 11), &mut config);
        assert_eq!(ui.view, View::Debug);
        ui.poll(L | R | (1 << 11), &mut config);
        assert_eq!(ui.view, View::Debug);
        ui.poll(L | R, &mut config);
        ui.poll(L | R | (1 << 11), &mut config);
        assert_eq!(ui.view, View::Hidden);
        ui.poll(0, &mut config);
        // Triggers do nothing, in either order.
        ui.poll(ZL | ZR, &mut config);
        ui.poll(ZL | ZR | DOWN, &mut config);
        assert_eq!(ui.view, View::Hidden);
        ui.poll(0, &mut config);
        ui.poll(DOWN, &mut config);
        ui.poll(ZL | ZR | DOWN, &mut config);
        assert_eq!(ui.view, View::Hidden);
        assert_eq!(config, Config::default());
        let mut latch = CommandLatch::default();
        assert_eq!(latch.sample(L | R), None);
        assert_eq!(
            latch.sample(L | R | DOWN | (1 << 10)),
            Some(Command::Cycle {
                debug_modifier: true
            })
        );
        assert_eq!(latch.sample(L | R | DOWN | (1 << 10)), None);
        assert_eq!(latch.sample(L | R | (1 << 10)), None);
        assert_eq!(latch.sample(L | R), None);
        assert_eq!(latch.sample(L | R | (1 << 10)), Some(Command::Debug));
    }

    #[test]
    fn plugins_view_enters_the_cycle_only_after_a_registration() {
        let none = HiddenPages::NONE;
        for debug in [false, true] {
            assert_eq!(
                next_view(View::Hidden, 0, debug, false, 0, none),
                (View::Settings, 0)
            );
            assert_eq!(
                next_view(View::Hidden, 0, debug, true, 2, none),
                (View::Settings, 0)
            );
            assert_eq!(
                next_view(View::Performance, 0, debug, true, 2, none),
                (View::Hidden, 0)
            );
        }
        assert_eq!(
            next_view(View::Debug, 1, false, true, 2, none),
            (View::Performance, 0)
        );
        assert_eq!(
            next_view(View::Debug, 1, true, true, 2, none),
            (View::Debug, 1)
        );
        assert_eq!(
            next_view(View::Settings, 0, false, false, 0, none),
            (View::Performance, 0)
        );
        assert_eq!(
            next_view(View::Settings, 0, false, true, 0, none),
            (View::Plugins, 0)
        );
        assert_eq!(
            next_view(View::Settings, 0, true, true, 0, none),
            (View::Debug, 0)
        );
        // Own pages follow the shared page, or Settings when nothing is shared.
        assert_eq!(
            next_view(View::Settings, 0, false, false, 2, none),
            (View::PluginPage, 0)
        );
        assert_eq!(
            next_view(View::Plugins, 0, false, true, 2, none),
            (View::PluginPage, 0)
        );
        assert_eq!(
            next_view(View::PluginPage, 0, false, true, 2, none),
            (View::PluginPage, 1)
        );
        assert_eq!(
            next_view(View::PluginPage, 1, false, true, 2, none),
            (View::Performance, 0)
        );
        assert_eq!(
            next_view(View::Plugins, 0, false, true, 0, none),
            (View::Performance, 0)
        );
        assert_eq!(
            next_view(View::PluginPage, 1, true, true, 2, none),
            (View::Debug, 1)
        );
        assert_eq!(
            next_view(View::Plugins, 0, true, true, 0, none),
            (View::Debug, 0)
        );
        assert_eq!(View::Plugins as u8, 4);
        assert_eq!(View::PluginPage as u8, 5);
        assert_eq!(View::Debug as u8, 3);
    }

    #[test]
    fn hide_pages_drops_the_shared_page_and_performance_from_the_walk() {
        let both = HiddenPages {
            plugins: true,
            own_pages: false,
            performance: true,
        };
        // Settings → own page → Hidden, even with shared rows registered.
        assert_eq!(
            next_view(View::Hidden, 0, false, true, 1, both),
            (View::Settings, 0)
        );
        assert_eq!(
            next_view(View::Settings, 0, false, true, 1, both),
            (View::PluginPage, 0)
        );
        assert_eq!(
            next_view(View::PluginPage, 0, false, true, 1, both),
            (View::Hidden, 0)
        );
        // Without an own page Settings hides at once; Debug hides too.
        assert_eq!(
            next_view(View::Settings, 0, false, true, 0, both),
            (View::Hidden, 0)
        );
        assert_eq!(
            next_view(View::Debug, 0, false, true, 0, both),
            (View::Hidden, 0)
        );
        // Debug stays reachable from Settings and an own page.
        assert_eq!(
            next_view(View::Settings, 0, true, true, 1, both),
            (View::Debug, 0)
        );
        assert_eq!(
            next_view(View::PluginPage, 0, true, true, 1, both),
            (View::Debug, 0)
        );
        // Each flag alone.
        let plugins = HiddenPages {
            plugins: true,
            ..HiddenPages::NONE
        };
        assert_eq!(
            next_view(View::Settings, 0, false, true, 0, plugins),
            (View::Performance, 0)
        );
        assert_eq!(
            next_view(View::Settings, 0, false, true, 2, plugins),
            (View::PluginPage, 0)
        );
        let own = HiddenPages {
            own_pages: true,
            ..HiddenPages::NONE
        };
        assert_eq!(
            next_view(View::Settings, 0, false, false, 2, own),
            (View::Performance, 0)
        );
        assert_eq!(
            next_view(View::Plugins, 0, false, true, 2, own),
            (View::Performance, 0)
        );
        let everything = HiddenPages {
            plugins: true,
            own_pages: true,
            performance: true,
        };
        assert_eq!(
            next_view(View::Settings, 0, false, true, 2, everything),
            (View::Hidden, 0),
            "Settings stays; everything else is skipped"
        );
        let performance = HiddenPages {
            performance: true,
            ..HiddenPages::NONE
        };
        assert_eq!(
            next_view(View::Settings, 0, false, true, 0, performance),
            (View::Plugins, 0)
        );
        assert_eq!(
            next_view(View::Plugins, 0, false, true, 0, performance),
            (View::Hidden, 0)
        );
        // A stale Performance view still hides.
        assert_eq!(
            next_view(View::Performance, 0, false, true, 0, both),
            (View::Hidden, 0)
        );

        // Through the overlay: the INI value drives the walk.
        let _gate = crate::peer_transport::global_gate();
        crate::peer_transport::clear_session();
        let registry = crate::plugin_api::Registry::new();
        let mut ui = Overlay::default();
        let mut config = Config {
            hide_pages: both,
            ..Config::default()
        };
        let none = crate::menu::NativePads::off();
        ui.poll_with(L | R | DOWN, &mut config, none, &registry);
        assert_eq!(ui.view, View::Settings);
        ui.poll_with(L | R, &mut config, none, &registry);
        ui.poll_with(L | R | DOWN, &mut config, none, &registry);
        assert_eq!(ui.view, View::Hidden, "no Performance page");
        assert_eq!(config.hide_pages, both, "the chord never edits the key");
    }

    static PLUGIN_STATE: crate::plugin_api::testing::FakePlugin =
        crate::plugin_api::testing::FakePlugin::new(
            [0, 1, 0, 0],
            deluxe_plugin_abi::REQUEST_APPLIED,
        );

    #[test]
    fn edit_hold_needs_zl_for_value_changes_but_not_for_navigation() {
        let _gate = crate::peer_transport::global_gate();
        crate::peer_transport::clear_session();
        let registry = crate::plugin_api::Registry::new();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        let none = crate::menu::NativePads::off();
        ui.edit_hold = true;
        // Settings: a plain Right is ignored, ZR+Right edits, Down still moves.
        ui.poll_with(L | R | DOWN, &mut config, none, &registry);
        ui.poll_with(0, &mut config, none, &registry);
        assert_eq!(ui.view, View::Settings);
        assert!(!ui.poll_with(RIGHT, &mut config, none, &registry));
        assert_eq!(config, Config::default());
        ui.poll_with(0, &mut config, none, &registry);
        // ZL is the Training fighter-swap modifier, never the edit hold.
        assert!(!ui.poll_with(ZL | RIGHT, &mut config, none, &registry));
        assert_eq!(config, Config::default());
        ui.poll_with(0, &mut config, none, &registry);
        assert!(ui.poll_with(ZR | RIGHT, &mut config, none, &registry));
        assert_eq!(config.input_buffer, 0);
        ui.poll_with(ZR, &mut config, none, &registry);
        // ZL+ZR is not the edit hold (and no longer a window chord).
        assert!(!ui.poll_with(ZL | ZR | RIGHT, &mut config, none, &registry));
        ui.poll_with(0, &mut config, none, &registry);
        let before = ui.menu.selected;
        ui.poll_with(DOWN, &mut config, none, &registry);
        assert_ne!(ui.menu.selected, before, "navigation never needs the hold");
        ui.poll_with(0, &mut config, none, &registry);
        // Without the hold flag (menus) a plain Right edits as before.
        ui.edit_hold = false;
        ui.poll_with(UP, &mut config, none, &registry);
        ui.poll_with(0, &mut config, none, &registry);
        assert!(ui.poll_with(RIGHT, &mut config, none, &registry));
        ui.poll_with(0, &mut config, none, &registry);

        // Plugin page: the same rule.
        ui.edit_hold = true;
        let buffer = config.input_buffer;
        assert_eq!(
            crate::plugin_api::testing::register_fake(
                &registry,
                "held",
                "Held",
                &[("enabled", "Enabled", &["Off", "On"])],
                &HOLD_STATE,
            ),
            0
        );
        ui.poll_with(L | R | DOWN, &mut config, none, &registry);
        ui.poll_with(0, &mut config, none, &registry);
        assert_eq!(ui.view, View::Plugins);
        ui.poll_with(RIGHT, &mut config, none, &registry);
        assert_eq!(registry.take_last_step(), None, "plain Right ignored");
        ui.poll_with(0, &mut config, none, &registry);
        ui.poll_with(ZL | RIGHT, &mut config, none, &registry);
        assert_eq!(registry.take_last_step(), None, "ZL is not the hold");
        ui.poll_with(0, &mut config, none, &registry);
        ui.poll_with(ZR | RIGHT, &mut config, none, &registry);
        let step = registry.take_last_step().unwrap();
        assert_eq!((step.option, step.requested), (0, 1));
        assert_eq!(
            config.input_buffer, buffer,
            "plugin edits never touch Config"
        );
    }

    static HOLD_STATE: crate::plugin_api::testing::FakePlugin =
        crate::plugin_api::testing::FakePlugin::new(
            [0, 0, 0, 0],
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
        // Row 0 of the shared page is the "Demo" group row: the selection
        // settles on the first option. Unmodified D-pad edits the plugin row,
        // not NetLatency or a native pane.
        assert_eq!(ui.plugin_row, 1);
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
        assert_eq!(ui.plugin_row, 2);
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(LEFT, &mut config, css, &registry);
        let step = registry.take_last_step().unwrap();
        assert_eq!((step.option, step.requested), (1, 0));
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(DOWN, &mut config, css, &registry);
        assert_eq!(ui.plugin_row, 3);
        ui.poll_with(0, &mut config, css, &registry);
        // The single-choice status row is read-only.
        ui.poll_with(RIGHT, &mut config, css, &registry);
        assert_eq!(registry.take_last_step(), None);
        ui.poll_with(0, &mut config, css, &registry);
        // Wrapping skips the group row in both directions.
        ui.poll_with(DOWN, &mut config, css, &registry);
        assert_eq!(ui.plugin_row, 1);
        ui.poll_with(0, &mut config, css, &registry);
        ui.poll_with(UP, &mut config, css, &registry);
        assert_eq!(ui.plugin_row, 3);
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
    fn shoulder_x_toggles_vanilla_plus_once_per_press_without_a_trigger() {
        let _gate = crate::peer_transport::global_gate();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        assert!(config.vanilla_plus);
        // One edge per press: holding the chord does not flip it again.
        assert!(ui.poll(L | R | X, &mut config));
        assert!(!config.vanilla_plus);
        assert!(!ui.poll(L | R | X, &mut config));
        assert!(!config.vanilla_plus);
        ui.poll(0, &mut config);
        assert!(ui.poll(L | R | X, &mut config));
        assert!(config.vanilla_plus);
        ui.poll(0, &mut config);
        // X alone, or with one bumper, is the game's button.
        let before = config.clone();
        assert!(!ui.poll(X, &mut config));
        assert!(!ui.poll(L | X, &mut config));
        assert_eq!(config, before);
        assert_eq!(ui.view, View::Hidden);
    }

    #[test]
    fn the_retired_fps_chord_changes_nothing() {
        let _gate = crate::peer_transport::global_gate();
        let mut ui = Overlay::default();
        for platform in [
            crate::config::Platform::Console,
            crate::config::Platform::Emulator,
        ] {
            let mut config = Config {
                platform,
                ..Config::default()
            };
            let before = config.clone();
            assert!(!ui.poll(L | R | ZL | X, &mut config));
            ui.poll(0, &mut config);
            assert!(!ui.poll(L | R | ZR | X, &mut config));
            ui.poll(0, &mut config);
            assert_eq!(config, before);
            assert_eq!(ui.view, View::Hidden);
        }
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
        // A plain D-pad press moves the character-select hand; it edits nothing.
        assert!(!ui.poll_native(RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, -1);
        assert!(!ui.poll_native(UP, &mut config, css));
        assert!(config.auto_profile);
        ui.poll_native(0, &mut config, css);
        // ZL held with the press edits, once per press.
        assert!(ui.poll_native(ZL | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
        assert!(!ui.poll_native(ZL | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
        ui.poll_native(ZL, &mut config, css);
        assert!(ui.poll_native(ZL | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 1);
        ui.poll_native(0, &mut config, css);
        assert!(!ui.poll_native(
            ZL | RIGHT,
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
        // Both triggers are the view chord, not the edit modifier.
        assert!(!ui.poll_native(ZL | ZR | RIGHT, &mut config, css));
        assert_eq!(config, before);
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
        assert!(!ui.poll_native(ZL | UP, &mut config, lobby));
        assert!(config.auto_profile);
        assert_eq!(config.render_mode, crate::config::RenderMode::Vanilla);
        ui.poll_native(0, &mut config, lobby);
        assert!(ui.poll_native(ZL | RIGHT, &mut config, lobby));
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
        assert!(!ui.poll_native(ZL | RIGHT, &mut config, waiting));
        assert_eq!(config.input_buffer, -1);
        assert!(ui.poll_native(ZL | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
        assert!(!ui.poll_native(ZL | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
        ui.poll_native(0, &mut config, css);
        assert!(!ui.poll_native(ZL | RIGHT, &mut config, absent));
        assert!(!ui.poll_native(ZL | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, 0);
    }

    #[test]
    fn streamer_hides_native_edits_and_a_chord_does_not_edit_twice() {
        let _gate = crate::peer_transport::global_gate();
        let mut ui = Overlay::default();
        let mut config = Config::default();
        let css = native(crate::menu::NativeSurface::Css, true);
        config.streamer = true;
        assert!(!ui.poll_native(ZL | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, -1);
        config.streamer = false;
        assert!(!ui.poll_native(ZL | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, -1);
        ui.poll_native(0, &mut config, css);
        config.streamer = true;
        assert!(!ui.poll_native(ZL | UP, &mut config, css));
        assert!(config.auto_profile);
        ui.poll_native(0, &mut config, css);
        config.streamer = false;
        let before = config.input_buffer;
        assert!(!ui.poll_native(L | R | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, before);
        ui.poll_native(0, &mut config, css);
        assert!(ui.poll_native(ZL | RIGHT, &mut config, css));
        assert_eq!(config.input_buffer, before + 1);
        assert!(!ui.poll_native(ZL | RIGHT, &mut config, css));
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
