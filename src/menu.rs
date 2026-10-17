//! Portable menu state. Runtime owns controller edges and rendering.

use crate::auto_policy::{cycle_profile_choice, Connection, ProfileChoice, SceneStatus};
use crate::config::{Config, Platform, RenderMode};

/// Editable Player Info rows, top to bottom. The FX row left in 0.5.0 with
/// character FX (now the `ssbu-fx-control` plugin); the emulator-only FPS++
/// row left on 2026-10-17 with FPS++.
pub const ROW_PEER: usize = 0;
pub const ROW_NET_LATENCY: usize = 1;
pub const ROW_NET_PROFILE: usize = 2;
pub const ROW_STEALTH: usize = 3;
pub const ROW_STREAMER: usize = 4;
/// Console LLUltra base resolution (`ultra_resolution`, 0.5.3 experiment).
pub const ROW_ULTRA_RES: usize = 5;
/// Window pages in the chord cycle (`hide_pages`, 0.5.3): the shared Plugins
/// page, every plugin's own page, Performance. Settings has no row: it is
/// always in the cycle and shows these three.
pub const ROW_PAGE_PLUGINS: usize = 6;
pub const ROW_PAGE_OWN: usize = 7;
pub const ROW_PAGE_PERF: usize = 8;
/// Frame time as the plot or a number (`perf_view`, 0.5.3).
pub const ROW_PERF_VIEW: usize = 9;
pub const ROW_COUNT: usize = 10;

/// Which editable rows exist. Platform is configured, never a row.
/// UltraRes is the console-only row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RowGate {
    pub peers_connected: bool,
    pub ultra_res_visible: bool,
}

pub fn row_gate(peers_connected: bool, platform: Platform) -> RowGate {
    RowGate {
        peers_connected,
        ultra_res_visible: platform == Platform::Console,
    }
}

pub fn row_visible(row: usize, gate: RowGate) -> bool {
    match row {
        ROW_PEER => gate.peers_connected,
        ROW_ULTRA_RES => gate.ultra_res_visible,
        ROW_NET_LATENCY | ROW_NET_PROFILE | ROW_STEALTH | ROW_STREAMER | ROW_PAGE_PLUGINS
        | ROW_PAGE_OWN | ROW_PAGE_PERF | ROW_PERF_VIEW => true,
        _ => false,
    }
}

/// Lowest visible row. Peer is first only while a station is connected.
pub fn first_row(gate: RowGate) -> usize {
    (0..ROW_COUNT)
        .find(|row| row_visible(*row, gate))
        .unwrap_or(ROW_NET_LATENCY)
}

/// Ready singles or doubles. Training and inactive scenes stay vertical.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuShape {
    Vertical,
    Horizontal,
}

pub fn menu_shape(ready: bool, status: SceneStatus) -> MenuShape {
    if ready && matches!(status, SceneStatus::Singles | SceneStatus::Doubles) {
        MenuShape::Horizontal
    } else {
        MenuShape::Vertical
    }
}

/// Background alpha for an in-match window. Outside a match the style default stays.
pub const MATCH_BG_ALPHA: f32 = 0.42;

pub fn match_window_alpha(shape: MenuShape) -> Option<f32> {
    match shape {
        MenuShape::Horizontal => Some(MATCH_BG_ALPHA),
        MenuShape::Vertical => None,
    }
}

/// Maximum outer width for the in-match Settings/Performance strip at 1920x1080.
/// The UI uses it for left-edge positioning and adaptive text sizing; ImGui
/// sizes the compact background to its content.
pub const MATCH_STRIP_WIDTH: f32 = 1840.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfoView {
    Settings,
    Performance,
    Debug,
    /// Rows registered by separately built plugin NROs (`crate::plugin_api`).
    Plugins,
}

/// Design width at 1920x1080. Match Settings/Performance/Plugins use this as a
/// positioning and text-fit bound while ImGui sizes the background to content;
/// all other views keep their existing compact or vertical pane sizes.
pub fn pane_width(shape: MenuShape, view: InfoView) -> f32 {
    match (view, shape) {
        (InfoView::Settings | InfoView::Performance | InfoView::Plugins, MenuShape::Horizontal) => {
            MATCH_STRIP_WIDTH
        }
        (InfoView::Settings | InfoView::Plugins, MenuShape::Vertical) => 380.0,
        _ => 300.0,
    }
}

/// Only ordinary ready-match views use the borderless, no-title strip. Debug
/// remains the explicit full-detail view.
pub fn compact_strip(shape: MenuShape, view: InfoView) -> bool {
    shape == MenuShape::Horizontal
        && matches!(
            view,
            InfoView::Settings | InfoView::Performance | InfoView::Plugins
        )
}

/// One navigable field in the inline match strip.
#[cfg(any(target_os = "switch", test))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompactField {
    pub label: &'static str,
    pub value: String,
    pub selected: bool,
}

/// Builds one unwrapped row of inline fields. Peer is kept on the optional
/// opponent row so a long name cannot crowd the local controls.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn compact_fields_line<'a>(
    fields: impl IntoIterator<Item = &'a CompactField>,
    peer_only: bool,
) -> String {
    fields
        .into_iter()
        .filter(|field| (field.label == "Peer") == peer_only)
        .map(|field| {
            format!(
                "{}{}:{}",
                if field.selected { ">" } else { "" },
                field.label,
                field.value
            )
        })
        .collect::<Vec<_>>()
        .join("   ")
}

#[cfg(any(target_os = "switch", test))]
pub(crate) fn compact_timing_bar(milliseconds: Option<f32>) -> String {
    let Some(milliseconds) = milliseconds.filter(|value| value.is_finite() && *value > 0.0) else {
        return "[no data]".to_owned();
    };
    const SEGMENTS: usize = 10;
    let filled = ((milliseconds / 35.0).clamp(0.0, 1.0) * SEGMENTS as f32).round() as usize;
    format!("[{}{}]", "#".repeat(filled), "-".repeat(SEGMENTS - filled))
}

#[cfg(any(target_os = "switch", test))]
pub(crate) fn compact_timing_line(average_ms: Option<f32>, latest_ms: Option<f32>) -> String {
    match (
        average_ms.filter(|value| value.is_finite() && *value > 0.0),
        latest_ms.filter(|value| value.is_finite() && *value > 0.0),
    ) {
        (Some(mean), Some(latest)) => format!(
            "FPS:{:.1}  FT:{latest:.2}ms  AVG:{mean:.2}ms  {}",
            1000.0 / mean,
            compact_timing_bar(Some(latest))
        ),
        _ => "FPS:---  FT:---  [no data]".to_owned(),
    }
}

#[cfg(any(target_os = "switch", test))]
fn compact_char_units(value: char) -> f32 {
    if !value.is_ascii() {
        return 1.0;
    }
    match value {
        ' ' => 0.34,
        'i' | 'l' | 'I' | '!' | '.' | ',' | ':' | ';' | '|' | '\'' => 0.4,
        'W' | 'M' | '@' => 0.95,
        _ => 0.72,
    }
}

#[cfg(any(target_os = "switch", test))]
fn compact_text_width(value: &str, font_size: f32) -> f32 {
    value.chars().map(compact_char_units).sum::<f32>() * font_size
}

/// Scales only when a line's conservative width estimate exceeds the content
/// width left inside the strip's maximum outer width after padding.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn compact_font_size(first: &str, second: &str, width: f32, sx: f32, sy: f32) -> f32 {
    let base = 16.0 * sx.min(sy);
    let available = (width - 24.0 * sx).max(1.0);
    let widest = compact_text_width(first, base).max(compact_text_width(second, base));
    if widest > available {
        base * (available / widest) * 0.94
    } else {
        base
    }
}

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

/// Which native pane last ran. `LocalLobby` is latency-only, matching the fork.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum NativeSurface {
    None = 0,
    Css = 1,
    Arena = 2,
    LocalLobby = 3,
}

impl NativeSurface {
    pub const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::Css,
            2 => Self::Arena,
            3 => Self::LocalLobby,
            _ => Self::None,
        }
    }
}

/// One draw's native-edit permission. The overlay edge is the only sampler.
///
/// `surface` is the last pane that opened since the scene changed. `fresh` is
/// true only when that pane ran again after the previous draw.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativePads {
    pub allow: bool,
    pub surface: NativeSurface,
    pub fresh: bool,
}

impl NativePads {
    pub const fn off() -> Self {
        Self {
            allow: false,
            surface: NativeSurface::None,
            fresh: false,
        }
    }
}

/// Fork overlay allow-interact: a ready online scene that is not singles or doubles.
/// Training and inactive menus stay editable. Offline does not.
pub fn native_banner_allowed(ready: bool, connection: Connection, status: SceneStatus) -> bool {
    ready
        && connection.is_online()
        && !matches!(status, SceneStatus::Singles | SceneStatus::Doubles)
}

/// Left/right on the fork native panes. `direction` is -1 or +1.
pub fn edit_latency(config: &mut Config, direction: i8) -> bool {
    if direction == 0 {
        return false;
    }
    let next = (i16::from(config.input_buffer) + i16::from(direction)).clamp(-1, 25) as i8;
    if next == config.input_buffer {
        return false;
    }
    config.input_buffer = next;
    true
}

/// Up/down on CSS and arena. Entering Auto keeps the saved manual mode.
pub fn edit_profile(config: &mut Config, direction: i8) -> bool {
    if direction == 0 {
        return false;
    }
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

/// Applies one native edge. A lobby surface ignores profile. `None` changes nothing.
pub fn apply_native_edges(
    config: &mut Config,
    surface: NativeSurface,
    latency_dir: i8,
    profile_dir: i8,
) -> bool {
    if surface == NativeSurface::None {
        return false;
    }
    let profile_dir = if surface == NativeSurface::LocalLobby {
        0
    } else {
        profile_dir
    };
    let latency = edit_latency(config, latency_dir);
    let profile = edit_profile(config, profile_dir);
    latency || profile
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

/// Steps to the next visible row. Hidden Peer and UltraRes rows are not stops.
pub fn move_row(selected: usize, direction: i8, gate: RowGate) -> usize {
    let mut next = selected;
    for _ in 0..ROW_COUNT {
        next = if direction < 0 {
            (next + ROW_COUNT - 1) % ROW_COUNT
        } else {
            (next + 1) % ROW_COUNT
        };
        if row_visible(next, gate) {
            return next;
        }
    }
    selected
}

impl Menu {
    /// Returns true only when a setting actually changes.
    pub fn handle(&mut self, action: Action, config: &mut Config) -> bool {
        self.handle_peers(action, config, true)
    }

    /// `peers_connected` is mapped-station membership, not a fresh ping.
    pub fn handle_peers(
        &mut self,
        action: Action,
        config: &mut Config,
        peers_connected: bool,
    ) -> bool {
        self.handle_rows(action, config, row_gate(peers_connected, config.platform))
    }

    pub fn handle_rows(&mut self, action: Action, config: &mut Config, gate: RowGate) -> bool {
        match action {
            Action::Toggle => {
                self.open = !self.open;
                if self.open {
                    self.selected = first_row(gate);
                }
                return false;
            }
            Action::Close => {
                self.open = false;
                return false;
            }
            _ if !self.open => return false,
            Action::Up => {
                self.selected = move_row(self.selected, -1, gate);
                return false;
            }
            Action::Down => {
                self.selected = move_row(self.selected, 1, gate);
                return false;
            }
            Action::Left | Action::Right => {}
        }
        if !row_visible(self.selected, gate) {
            self.selected = move_row(self.selected, 1, gate);
            return false;
        }
        let direction: i8 = if action == Action::Left { -1 } else { 1 };
        match self.selected {
            ROW_PEER => {
                crate::peer_transport::cycle_peer(direction);
                false
            }
            ROW_NET_LATENCY => edit_latency(config, direction),
            ROW_NET_PROFILE => edit_profile(config, direction),
            ROW_STEALTH => {
                config.stealth = !config.stealth;
                true
            }
            ROW_STREAMER => {
                config.streamer = !config.streamer;
                true
            }
            ROW_ULTRA_RES => {
                config.ultra_resolution = config.ultra_resolution.cycle(direction);
                true
            }
            ROW_PAGE_PLUGINS => {
                config.hide_pages.plugins = !config.hide_pages.plugins;
                true
            }
            ROW_PAGE_OWN => {
                config.hide_pages.own_pages = !config.hide_pages.own_pages;
                true
            }
            ROW_PAGE_PERF => {
                config.hide_pages.performance = !config.hide_pages.performance;
                true
            }
            ROW_PERF_VIEW => {
                config.perf_view = config.perf_view.cycle();
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
        let _gate = crate::peer_transport::global_gate();
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
        assert_eq!(
            config,
            Config {
                input_buffer: 0,
                ..Config::default()
            }
        );
        menu.handle(Action::Close, &mut config);
        assert!(!menu.open);
    }

    #[test]
    fn navigation_wraps_and_buffer_saturates_without_false_change_reports() {
        let mut menu = Menu {
            open: true,
            selected: 0,
        };
        let mut config = Config {
            platform: Platform::Emulator,
            ..Config::default()
        };
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_NET_PROFILE);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_STEALTH);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_STREAMER);
        // Emulator: UltraRes is hidden, so the page rows follow Streamer.
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PAGE_PLUGINS);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PAGE_OWN);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PAGE_PERF);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PERF_VIEW);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PEER);
        menu.handle(Action::Up, &mut config);
        assert_eq!(menu.selected, ROW_PERF_VIEW);
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
        let machine =
            crate::render_policy::Machine::confirmed(crate::render_policy::Profile::Doubles);
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

    #[test]
    fn page_rows_toggle_the_hidden_pages_and_perf_view_swaps_graph_and_number() {
        use crate::config::{HiddenPages, PerfView};
        let mut menu = Menu {
            open: true,
            selected: ROW_PAGE_PLUGINS,
        };
        let mut config = Config::default();
        for platform in [Platform::Console, Platform::Emulator] {
            let gate = row_gate(false, platform);
            for row in [ROW_PAGE_PLUGINS, ROW_PAGE_OWN, ROW_PAGE_PERF, ROW_PERF_VIEW] {
                assert!(row_visible(row, gate), "{row} on {platform:?}");
            }
        }
        assert!(menu.handle(Action::Right, &mut config));
        assert!(config.hide_pages.plugins);
        assert!(menu.handle(Action::Left, &mut config));
        assert_eq!(config.hide_pages, HiddenPages::NONE);
        menu.selected = ROW_PAGE_OWN;
        assert!(menu.handle(Action::Right, &mut config));
        assert!(config.hide_pages.own_pages);
        menu.selected = ROW_PAGE_PERF;
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(
            config.hide_pages,
            HiddenPages {
                plugins: false,
                own_pages: true,
                performance: true
            }
        );
        menu.selected = ROW_PERF_VIEW;
        assert_eq!(config.perf_view, PerfView::Graph);
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.perf_view, PerfView::Number);
        assert!(menu.handle(Action::Left, &mut config));
        assert_eq!(config.perf_view, PerfView::Graph);
        // Closed, nothing edits.
        menu.open = false;
        assert!(!menu.handle(Action::Right, &mut config));
        assert_eq!(config.perf_view, PerfView::Graph);
    }

    #[test]
    fn native_edges_keep_saved_manual_modes_and_do_not_repeat() {
        assert!(!native_banner_allowed(
            false,
            Connection::Quickplay,
            SceneStatus::Inactive
        ));
        assert!(!native_banner_allowed(
            true,
            Connection::Offline,
            SceneStatus::Inactive
        ));
        assert!(!native_banner_allowed(
            true,
            Connection::Arena,
            SceneStatus::Singles
        ));
        assert!(!native_banner_allowed(
            true,
            Connection::Local,
            SceneStatus::Doubles
        ));
        assert!(native_banner_allowed(
            true,
            Connection::Quickplay,
            SceneStatus::Inactive
        ));
        assert!(native_banner_allowed(
            true,
            Connection::Arena,
            SceneStatus::Training
        ));
        let mut config = Config {
            auto_profile: false,
            render_mode: RenderMode::LessLag,
            ..Config::default()
        };
        assert!(apply_native_edges(&mut config, NativeSurface::Css, 0, -1));
        assert!(!config.auto_profile);
        assert_eq!(config.render_mode, RenderMode::Vanilla);
        assert!(apply_native_edges(&mut config, NativeSurface::Arena, 0, -1));
        assert!(config.auto_profile);
        assert_eq!(config.render_mode, RenderMode::Vanilla);
        config.auto_profile = false;
        config.render_mode = RenderMode::LessLagDoubles;
        assert!(apply_native_edges(&mut config, NativeSurface::Css, 0, 1));
        assert!(config.auto_profile);
        assert_eq!(config.render_mode, RenderMode::LessLagDoubles);
        let saved = config.render_mode;
        assert!(!apply_native_edges(
            &mut config,
            NativeSurface::LocalLobby,
            0,
            1
        ));
        assert_eq!(config.render_mode, saved);
        assert!(config.auto_profile);
        assert!(apply_native_edges(
            &mut config,
            NativeSurface::LocalLobby,
            1,
            1
        ));
        assert_eq!(config.input_buffer, 0);
        assert!(config.auto_profile);
        assert!(!apply_native_edges(&mut config, NativeSurface::None, 1, 1));
        assert_eq!(config.input_buffer, 0);
    }

    #[test]
    fn no_peer_skips_the_selector_and_a_join_restores_it() {
        let _gate = crate::peer_transport::global_gate();
        let mut menu = Menu::default();
        let mut config = Config::default();
        assert!(!menu.handle_peers(Action::Toggle, &mut config, false));
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        // The page and PerfView rows close the list on both platforms.
        menu.handle_peers(Action::Up, &mut config, false);
        assert_eq!(menu.selected, ROW_PERF_VIEW);
        menu.handle_peers(Action::Up, &mut config, false);
        assert_eq!(menu.selected, ROW_PAGE_PERF);
        menu.handle_peers(Action::Up, &mut config, false);
        assert_eq!(menu.selected, ROW_PAGE_OWN);
        menu.handle_peers(Action::Up, &mut config, false);
        assert_eq!(menu.selected, ROW_PAGE_PLUGINS);
        // Console: UltraRes is above them.
        menu.handle_peers(Action::Up, &mut config, false);
        assert_eq!(menu.selected, ROW_ULTRA_RES);
        assert!(menu.handle_peers(Action::Right, &mut config, false));
        assert_eq!(
            config.ultra_resolution,
            crate::config::UltraResolution::P720
        );
        assert!(menu.handle_peers(Action::Left, &mut config, false));
        assert_eq!(
            config.ultra_resolution,
            crate::config::UltraResolution::P576
        );
        menu.handle_peers(Action::Up, &mut config, false);
        assert_eq!(menu.selected, ROW_STREAMER);
        for _ in 0..6 {
            menu.handle_peers(Action::Down, &mut config, false);
        }
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        // Emulator: UltraRes is hidden, so Streamer sits above the page rows.
        config.platform = Platform::Emulator;
        assert!(!row_visible(
            ROW_ULTRA_RES,
            row_gate(false, Platform::Emulator)
        ));
        menu.selected = ROW_PAGE_PLUGINS;
        menu.handle_peers(Action::Up, &mut config, false);
        assert_eq!(menu.selected, ROW_STREAMER);
        menu.selected = ROW_PERF_VIEW;
        menu.handle_peers(Action::Down, &mut config, false);
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        config.platform = Platform::Console;
        config.input_buffer = 2;
        menu.selected = ROW_PEER;
        assert!(!menu.handle_peers(Action::Left, &mut config, false));
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        assert_eq!(config.input_buffer, 2);
        assert!(menu.handle_peers(Action::Left, &mut config, false));
        assert_eq!(config.input_buffer, 1);
        menu.handle_peers(Action::Up, &mut config, true);
        assert_eq!(menu.selected, ROW_PEER);
        assert!(!menu.handle_peers(Action::Left, &mut config, true));
        assert_eq!(config.input_buffer, 1);
    }

    #[test]
    fn match_shape_requires_a_ready_match_and_visibility_uses_platform() {
        assert_eq!(
            menu_shape(true, SceneStatus::Singles),
            MenuShape::Horizontal
        );
        assert_eq!(
            menu_shape(true, SceneStatus::Doubles),
            MenuShape::Horizontal
        );
        assert_eq!(menu_shape(true, SceneStatus::Training), MenuShape::Vertical);
        assert_eq!(menu_shape(true, SceneStatus::Inactive), MenuShape::Vertical);
        assert_eq!(menu_shape(false, SceneStatus::Singles), MenuShape::Vertical);
        assert_eq!(
            match_window_alpha(MenuShape::Horizontal),
            Some(MATCH_BG_ALPHA)
        );
        assert_eq!(match_window_alpha(MenuShape::Vertical), None);
        assert!(row_visible(
            ROW_NET_LATENCY,
            row_gate(false, Platform::Console)
        ));
        assert!(!row_visible(ROW_PEER, row_gate(false, Platform::Emulator)));
        assert!(row_visible(ROW_PEER, row_gate(true, Platform::Console)));
    }

    #[test]
    fn only_ready_match_settings_and_performance_use_the_adaptive_strip() {
        assert!(compact_strip(MenuShape::Horizontal, InfoView::Settings));
        assert!(compact_strip(MenuShape::Horizontal, InfoView::Performance));
        assert!(compact_strip(MenuShape::Horizontal, InfoView::Plugins));
        assert!(!compact_strip(MenuShape::Horizontal, InfoView::Debug));
        assert!(!compact_strip(MenuShape::Vertical, InfoView::Settings));
        assert!(!compact_strip(MenuShape::Vertical, InfoView::Performance));
        assert!(!compact_strip(MenuShape::Vertical, InfoView::Plugins));
        assert_eq!(
            pane_width(MenuShape::Horizontal, InfoView::Plugins),
            MATCH_STRIP_WIDTH
        );
        assert_eq!(pane_width(MenuShape::Vertical, InfoView::Plugins), 380.0);
        assert_eq!(
            pane_width(MenuShape::Horizontal, InfoView::Settings),
            MATCH_STRIP_WIDTH
        );
        assert_eq!(
            pane_width(MenuShape::Horizontal, InfoView::Performance),
            MATCH_STRIP_WIDTH
        );
        assert_eq!(pane_width(MenuShape::Horizontal, InfoView::Debug), 300.0);
        assert_eq!(pane_width(MenuShape::Vertical, InfoView::Settings), 380.0);
        assert_eq!(
            pane_width(MenuShape::Vertical, InfoView::Performance),
            300.0
        );

        for viewport in [1280.0_f32, 1920.0] {
            let scaled =
                pane_width(MenuShape::Horizontal, InfoView::Settings) * (viewport / 1920.0);
            assert!(scaled + 32.0 <= viewport);
        }
    }

    #[test]
    fn compact_field_rows_keep_values_inline_and_mark_the_selected_field() {
        let peer = "P".repeat(64);
        let fields = [
            CompactField {
                label: "Net",
                value: "Auto".to_owned(),
                selected: true,
            },
            CompactField {
                label: "Profile",
                value: "Auto:LLDbl".to_owned(),
                selected: false,
            },
            CompactField {
                label: "Stealth",
                value: "On (reconnect)".to_owned(),
                selected: false,
            },
            CompactField {
                label: "Stream",
                value: "Off".to_owned(),
                selected: false,
            },
            CompactField {
                label: "UltraRes",
                value: "576p".to_owned(),
                selected: false,
            },
            CompactField {
                label: "Peer",
                value: peer.clone(),
                selected: true,
            },
        ];
        let settings = compact_fields_line(fields.iter(), false);
        let opponent = compact_fields_line(fields.iter(), true);
        assert_eq!(
            settings,
            ">Net:Auto   Profile:Auto:LLDbl   Stealth:On (reconnect)   Stream:Off   UltraRes:576p"
        );
        assert_eq!(opponent, format!(">Peer:{peer}"));
        assert!(!settings.contains('\n'));
        assert!(!opponent.contains('\n'));
    }

    #[test]
    fn compact_timing_uses_measured_values_and_does_not_invent_missing_samples() {
        assert_eq!(compact_timing_bar(None), "[no data]");
        assert_eq!(compact_timing_bar(Some(f32::NAN)), "[no data]");
        assert_eq!(
            compact_timing_line(None, None),
            "FPS:---  FT:---  [no data]"
        );
        let measured = compact_timing_line(Some(16.67), Some(16.67));
        assert!(measured.contains("FPS:60.0"));
        assert!(measured.contains("FT:16.67ms"));
        assert!(measured.contains("AVG:16.67ms"));
        assert!(measured.ends_with("[#####-----]"));
    }

    #[test]
    fn maximum_peer_name_and_timing_lines_fit_without_wrapping_at_common_sizes() {
        let peer = "P".repeat(64);
        let fields = [
            CompactField {
                label: "Net",
                value: "Auto".to_owned(),
                selected: false,
            },
            CompactField {
                label: "Profile",
                value: "Auto:LLDbl".to_owned(),
                selected: false,
            },
            CompactField {
                label: "Stealth",
                value: "On (reconnect)".to_owned(),
                selected: false,
            },
            CompactField {
                label: "Stream",
                value: "Off".to_owned(),
                selected: false,
            },
            CompactField {
                label: "UltraRes",
                value: "576p".to_owned(),
                selected: false,
            },
            CompactField {
                label: "Peer",
                value: peer,
                selected: true,
            },
        ];
        let first = compact_fields_line(fields.iter(), false);
        let second = format!(
            "{}   IF:Wi-Fi   Ping:31ms   PeerNet:25   PeerProfile:LessLagDoubles   {}",
            compact_fields_line(fields.iter(), true),
            compact_timing_line(Some(16.67), Some(16.67))
        );
        assert!(!first.contains('\n'));
        assert!(!second.contains('\n'));

        for (viewport_width, viewport_height) in [(1280.0_f32, 720.0_f32), (1920.0, 1080.0)] {
            let sx = viewport_width / 1920.0;
            let sy = viewport_height / 1080.0;
            let width = MATCH_STRIP_WIDTH * sx;
            let available = width - 24.0 * sx;
            let font = compact_font_size(&first, &second, width, sx, sy);
            assert!(compact_text_width(&first, font) <= available);
            assert!(compact_text_width(&second, font) <= available);
        }
    }
}
