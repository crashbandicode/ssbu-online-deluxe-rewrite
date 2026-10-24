//! Portable menu state. Runtime owns controller edges and rendering.

use crate::auto_policy::{cycle_profile_choice, Connection, ProfileChoice, SceneStatus};
use crate::config::{Config, Platform, RenderMode};

/// Selectable Player Info rows, top to bottom. Settings has You, the optional Opp
/// selector and two controls. Privacy edits require You. Console LLUltra is one
/// NetProfile stop; L+R+X cycles its saved resolution in place. The remaining
/// rows configure the Window Options page.
///
/// The `You` summary line: ZR+X/Y edit privacy; Left/Right do nothing.
pub const ROW_YOU: usize = 0;
/// The `Opp` summary line: Left/Right cycles the selected remote station.
pub const ROW_PEER: usize = 1;
pub const ROW_NET_LATENCY: usize = 2;
pub const ROW_NET_PROFILE: usize = 3;
/// Window pages in the chord cycle (`hide_pages`, 0.5.3): the shared Plugins
/// page, every plugin's own page, Performance. Settings and Window have no
/// row: they are always in the cycle.
pub const ROW_PAGE_PLUGINS: usize = 4;
pub const ROW_PAGE_OWN: usize = 5;
pub const ROW_PAGE_PERF: usize = 6;
/// Frame time on the Performance and Debug pages as the plot or a number
/// (`perf_view`, 0.5.3).
pub const ROW_PERF_VIEW: usize = 7;
/// The chord hint line under the Settings and Window tables (`hints`, 0.6.0).
pub const ROW_HINTS: usize = 8;
pub const ROW_COUNT: usize = 9;

/// Which editable rows exist. `window` selects the Window Options page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RowGate {
    pub peers_connected: bool,
    pub window: bool,
}

pub fn row_gate(peers_connected: bool, window: bool) -> RowGate {
    RowGate {
        peers_connected,
        window,
    }
}

pub fn row_visible(row: usize, gate: RowGate) -> bool {
    match row {
        ROW_PEER => !gate.window && gate.peers_connected,
        ROW_YOU | ROW_NET_LATENCY | ROW_NET_PROFILE => !gate.window,
        ROW_PAGE_PLUGINS | ROW_PAGE_OWN | ROW_PAGE_PERF | ROW_PERF_VIEW | ROW_HINTS => gate.window,
        _ => false,
    }
}

/// Lowest visible row of the page: You on Settings, PluginsPage on Window.
pub fn first_row(gate: RowGate) -> usize {
    (0..ROW_COUNT)
        .find(|row| row_visible(*row, gate))
        .unwrap_or(if gate.window {
            ROW_PAGE_PLUGINS
        } else {
            ROW_YOU
        })
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
    /// The window's own configuration: page toggles, PerfView, Hints (0.6.0).
    Window,
}

/// Design width at 1920x1080. Match Settings/Performance/Plugins/Window use
/// this as a positioning and text-fit bound while ImGui sizes the background
/// to content; all other views keep their existing compact or vertical pane sizes.
pub fn pane_width(shape: MenuShape, view: InfoView) -> f32 {
    match (view, shape) {
        (
            InfoView::Settings | InfoView::Performance | InfoView::Plugins | InfoView::Window,
            MenuShape::Horizontal,
        ) => MATCH_STRIP_WIDTH,
        (InfoView::Settings | InfoView::Plugins | InfoView::Window, MenuShape::Vertical) => 380.0,
        _ => 300.0,
    }
}

/// Only ordinary ready-match views use the borderless, no-title strip. Debug
/// remains the explicit full-detail view.
pub fn compact_strip(shape: MenuShape, view: InfoView) -> bool {
    shape == MenuShape::Horizontal
        && matches!(
            view,
            InfoView::Settings | InfoView::Performance | InfoView::Plugins | InfoView::Window
        )
}

/// One drawn window row. The vertical table and the compact strip draw the
/// same list, for Settings cells and plugin rows alike.
#[cfg(any(target_os = "switch", test))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Row {
    pub label: String,
    /// Empty for a label-only row (a plugin group or header); choice labels
    /// are never empty.
    pub value: String,
    /// `deluxe_plugin_abi::TONE_*` of the value text.
    pub tone: u8,
    pub pending: bool,
    pub selected: bool,
    /// A run of plugin status cells drawn as one table (headings separated
    /// by bars, a check or cross centered beneath each). Empty for every
    /// other row; when set, `label` and `value` are empty.
    pub cells: Vec<Cell>,
}

/// One column of a status-cell table. `tone` is the plugin's ready/not-ready
/// contract: `TONE_OK` draws a check mark, any other tone a cross in that
/// tone's color. No value text is shown.
#[cfg(any(target_os = "switch", test))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Cell {
    pub label: String,
    pub tone: u8,
}

/// Placeholder segment the strip renderer replaces with a drawn mark. It is a
/// control character, which a plugin label can never contain.
#[cfg(any(target_os = "switch", test))]
pub(crate) const MARK_SLOT: &str = "\u{1}";

/// The display window of a plugin page of `rows`. Starts from [`window`] and
/// then keeps a status-cell table whole: a window edge that would cut one is
/// moved to the table's edge. On the strip (`paged`) the final page also takes
/// the passive rows (status, cell, header) after the last selectable row, since
/// the cursor can never reach a page that holds only those.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn plugin_window(
    plugins: &[std::sync::Arc<crate::plugin_api::Plugin>],
    rows: &[crate::plugin_api::PageRow],
    selected: usize,
    size: usize,
    paged: bool,
) -> (usize, usize, usize, usize) {
    use crate::plugin_api::{cell_group, selectable};
    let (mut start, mut end, page, mut pages) = window(selected, rows.len(), size, paged);
    if paged
        && end < rows.len()
        && rows[..end].iter().any(|row| selectable(plugins, *row))
        && rows[end..].iter().all(|row| !selectable(plugins, *row))
    {
        end = rows.len();
        pages = page;
    }
    if start < end {
        if let Some((first, _)) = cell_group(plugins, rows, start) {
            start = first;
        }
        if let Some((_, last)) = cell_group(plugins, rows, end - 1) {
            end = last;
        }
    }
    (start, end, page, pages)
}

/// `(text, tone)` segments of one strip line so the renderer can colour each
/// value: `>Label:` and the three-space separators stay neutral, the value
/// and its pending marker `~` take the row's tone, and a label-only row is
/// its label alone, without the colon. Concatenating the texts gives
/// [`strip_line`] exactly, so font fitting can use the joined string.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn strip_segments(rows: &[Row]) -> Vec<(String, u8)> {
    let mut segments = Vec::with_capacity(rows.len() * 3);
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            segments.push(("   ".to_owned(), deluxe_plugin_abi::TONE_NEUTRAL));
        }
        if !row.cells.is_empty() {
            for (cell, entry) in row.cells.iter().enumerate() {
                if cell > 0 {
                    segments.push((FACT_SEPARATOR.to_owned(), deluxe_plugin_abi::TONE_NEUTRAL));
                }
                segments.push((format!("{}:", entry.label), deluxe_plugin_abi::TONE_NEUTRAL));
                segments.push((MARK_SLOT.to_owned(), entry.tone));
            }
            continue;
        }
        if row.value.is_empty() {
            segments.push((row.label.clone(), deluxe_plugin_abi::TONE_NEUTRAL));
            continue;
        }
        segments.push((
            format!("{}{}:", if row.selected { ">" } else { "" }, row.label),
            deluxe_plugin_abi::TONE_NEUTRAL,
        ));
        segments.push((
            format!("{}{}", row.value, if row.pending { "~" } else { "" }),
            row.tone,
        ));
    }
    segments
}

/// One unwrapped strip line of `rows`.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn strip_line(rows: &[Row]) -> String {
    strip_segments(rows)
        .into_iter()
        .map(|(text, _)| text)
        .collect()
}

/// Rows shown around `selected`: with `paged`, the page of `size` rows that
/// contains it (the strip pages rather than shrinking its text); otherwise a
/// centred window of at most `size` rows (the vertical table). Returns
/// `(start, end, page, pages)` with one-based page numbers for display.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn window(
    selected: usize,
    total: usize,
    size: usize,
    paged: bool,
) -> (usize, usize, usize, usize) {
    if total == 0 || size == 0 {
        return (0, 0, 1, 1);
    }
    let selected = selected.min(total - 1);
    if paged {
        let page = selected / size;
        let start = page * size;
        return (
            start,
            (start + size).min(total),
            page + 1,
            total.div_ceil(size),
        );
    }
    let start = selected
        .saturating_sub(size / 2)
        .min(total.saturating_sub(size));
    (start, (start + size).min(total), 1, 1)
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

/// Separator between the facts of one summary line, the one the native
/// character-select banner already uses (ASCII: the window's font is the
/// ImGui host's and its glyph coverage is not ours to assume).
#[cfg(any(target_os = "switch", test))]
pub(crate) const FACT_SEPARATOR: &str = " | ";

/// Facts joined with [`FACT_SEPARATOR`]; an unknown fact (`---` or empty) is
/// left out rather than shown as a placeholder.
#[cfg(any(target_os = "switch", test))]
fn join_facts(facts: &[&str]) -> String {
    facts
        .iter()
        .filter(|fact| !fact.is_empty() && **fact != "---")
        .copied()
        .collect::<Vec<_>>()
        .join(FACT_SEPARATOR)
}

/// The `You` summary line of the Settings page (0.6.0): profile, input
/// buffer, local interface. Its selection enables the privacy icon shortcuts;
/// the rows below it control peer selection, latency and profile.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn you_line(profile: &str, buffer: &str, interface: &str) -> String {
    join_facts(&[profile, buffer, interface])
}

/// The `Opp` summary line (0.6.0): the selected peer's name, announced
/// profile, announced buffer, ping and interface, in that order, with what the
/// peer has not announced left out. It replaces the Name, Interface, Peer and
/// Ping rows and the opponent column.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn opp_line(
    name: &str,
    settings: &str,
    latency: &str,
    ping: &str,
    interface: &str,
) -> String {
    join_facts(&[name, settings, latency, ping, interface])
}

/// The NetProfile value with the engine status folded in (0.6.0; the Status
/// row is gone): `LessLag | Swap`. An empty status, or the `Vanilla` status
/// that only says no render-opts profile is armed, leaves the profile alone.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn net_profile_value(profile: &str, status: &str) -> String {
    if status.is_empty() || status == "Vanilla" {
        profile.to_owned()
    } else {
        format!("{profile}{FACT_SEPARATOR}{status}")
    }
}

/// The one-line performance footer of the Settings page (0.6.0): frame rate,
/// mean frame time, the applied internal resolution and the DRS state. The
/// full block and the plot live on the Performance page.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn perf_footer(mean_ms: Option<f32>, resolution: &str, drs: bool) -> String {
    let drs = if drs { "DRS on" } else { "DRS off" };
    match mean_ms.filter(|value| value.is_finite() && *value > 0.0) {
        Some(ms) => format!(
            "{:.1} fps{FACT_SEPARATOR}{ms:.2} ms{FACT_SEPARATOR}{resolution}{FACT_SEPARATOR}{drs}",
            1000.0 / ms
        ),
        None => format!(
            "--- fps{FACT_SEPARATOR}--- ms{FACT_SEPARATOR}{resolution}{FACT_SEPARATOR}{drs}"
        ),
    }
}

/// The dim hint line under the Settings and Window tables (0.6.0, `hints`).
/// It wraps to the window width; every chord it names is in CONTROLS.md.
pub const HINT_LINE: &str =
    "ZR+Left/Right edits   ZR+X Stealth   ZR+Y Streamer   L+R+Down next page   L+R+X Vanilla+/LLUltra";

/// Shows the edit state (0.6.0): while the ZR hold is on, the selected row's
/// value reads `< value >`; without it the value is plain, so the hold rule
/// explains itself. Label-only rows are never marked.
#[cfg(any(target_os = "switch", test))]
pub(crate) fn mark_editing(rows: &mut [Row], editing: bool) {
    if !editing {
        return;
    }
    for row in rows
        .iter_mut()
        .filter(|row| row.selected && !row.value.is_empty())
    {
        row.value = format!("< {} >", row.value);
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

/// Local profile text shared by the window and native banner. Console Ultra
/// suffixes name the saved base resolution, not a new renderer or the current
/// DRS level. Auto and the post-match Vanilla cleanup label retain their meaning.
pub fn profile_label(config: &Config, ready: bool, requested: RenderMode) -> String {
    if !config.auto_profile
        && config.platform == Platform::Console
        && config.render_mode == RenderMode::LessLagUltra
        && requested != RenderMode::Vanilla
    {
        config.ultra_resolution.profile_label().to_owned()
    } else {
        crate::auto_policy::profile_row_label(
            config.auto_profile,
            ready,
            config.render_mode,
            requested,
            config.vanilla_plus,
        )
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

/// NetProfile and native CSS/arena edits. Ultra is one stop on console and
/// emulator. Either direction leaves it for the adjacent profile and keeps the
/// saved resolution, which is shown again on reentry. Entering Auto keeps the
/// saved manual mode.
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

/// Steps to the next visible row. Hidden Opp and the other page's rows are
/// not stops.
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

    /// Settings page. `peers_connected` is mapped-station membership, not a
    /// fresh ping.
    pub fn handle_peers(
        &mut self,
        action: Action,
        config: &mut Config,
        peers_connected: bool,
    ) -> bool {
        self.handle_rows(action, config, row_gate(peers_connected, false))
    }

    /// Window page (0.6.0): the page toggles, PerfView and Hints.
    pub fn handle_window(&mut self, action: Action, config: &mut Config) -> bool {
        self.handle_rows(action, config, row_gate(false, true))
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
            ROW_HINTS => {
                config.hints = !config.hints;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UltraResolution;

    #[test]
    fn opening_focuses_the_first_row_and_closed_input_cannot_edit_settings() {
        let _gate = crate::peer_transport::global_gate();
        let mut menu = Menu {
            open: false,
            selected: ROW_NET_PROFILE,
        };
        let mut config = Config::default();
        assert!(!menu.handle(Action::Right, &mut config));
        assert_eq!(config, Config::default());
        assert!(!menu.handle(Action::Toggle, &mut config));
        assert_eq!(menu.selected, ROW_YOU);
        assert!(!menu.handle(Action::Right, &mut config));
        assert_eq!(config.input_buffer, -1);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PEER);
        assert!(!menu.handle(Action::Right, &mut config));
        assert_eq!(config, Config::default());
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
            selected: ROW_YOU,
        };
        let mut config = Config {
            platform: Platform::Emulator,
            ..Config::default()
        };
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PEER);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_NET_PROFILE);
        // NetProfile is the last Settings control on both platforms.
        menu.handle(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_YOU);
        menu.handle(Action::Up, &mut config);
        assert_eq!(menu.selected, ROW_NET_PROFILE);
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
        assert_eq!(config.ultra_resolution, UltraResolution::P576);
        assert!(menu.handle(Action::Right, &mut config));
        assert_eq!(config.render_mode, RenderMode::LessLagDoubles);
        assert_eq!(config.ultra_resolution, UltraResolution::P576);
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
                RenderMode::LessLagDoubles,
                false
            ),
            "Auto:LLDbl"
        );
        assert!(config.stealth);
        assert_eq!(stealth_value(config.stealth, None), "On");
        config.stealth = false;
        assert!(!config.stealth);
        assert_eq!(stealth_value(config.stealth, Some(true)), "Off (reconnect)");
        assert_eq!(stealth_value(config.stealth, None), "Off");
        config.stealth = true;
        assert!(config.stealth);
        assert_eq!(stealth_value(config.stealth, Some(false)), "On (reconnect)");
        assert_eq!(stealth_value(config.stealth, None), "On");
    }

    #[test]
    fn every_saved_ultra_resolution_is_one_stop_either_direction_and_is_recalled() {
        for platform in [Platform::Console, Platform::Emulator] {
            for resolution in [
                UltraResolution::P576,
                UltraResolution::P720,
                UltraResolution::P1080,
            ] {
                for direction in [1_i8, -1] {
                    let mut config = Config {
                        platform,
                        auto_profile: false,
                        render_mode: RenderMode::LessLagUltra,
                        ultra_resolution: resolution,
                        vanilla_plus: false,
                        input_buffer: 4,
                        ..Config::default()
                    };
                    let before = config.clone();
                    assert!(apply_native_edges(
                        &mut config,
                        NativeSurface::Css,
                        0,
                        direction
                    ));
                    let mode = if direction > 0 {
                        RenderMode::LessLagDoubles
                    } else {
                        RenderMode::LessLag
                    };
                    assert_eq!(
                        profile_action(&before, &config),
                        ProfileAction::SelectManual(mode)
                    );
                    assert_eq!(
                        config,
                        Config {
                            render_mode: mode,
                            ..before
                        }
                    );
                    let away = config.clone();
                    assert!(edit_profile(&mut config, -direction));
                    assert_eq!(
                        profile_action(&away, &config),
                        ProfileAction::SelectManual(RenderMode::LessLagUltra)
                    );
                    assert_eq!(config.render_mode, RenderMode::LessLagUltra);
                    assert_eq!(config.ultra_resolution, resolution);
                    assert!(!config.auto_profile);
                    assert_eq!(crate::config::parse(&config.serialize()).unwrap(), config);
                }
                for direction in [1_i8, -1] {
                    let mut auto = Config {
                        platform,
                        auto_profile: true,
                        render_mode: RenderMode::LessLagUltra,
                        ultra_resolution: resolution,
                        ..Config::default()
                    };
                    assert!(edit_profile(&mut auto, direction));
                    assert!(!auto.auto_profile);
                    assert_eq!(auto.render_mode, RenderMode::Vanilla);
                    assert_eq!(auto.ultra_resolution, resolution);
                    assert_eq!(crate::config::parse(&auto.serialize()).unwrap(), auto);
                }
            }
        }
        let mut config = Config {
            auto_profile: false,
            render_mode: RenderMode::LessLagUltra,
            ultra_resolution: UltraResolution::P1080,
            ..Config::default()
        };
        let before = config.clone();
        assert!(!edit_profile(&mut config, 0));
        assert_eq!(config, before);
    }

    #[test]
    fn profile_edits_recall_saved_ultra_resolution_and_emulator_has_no_extra_stops() {
        for resolution in [
            UltraResolution::P576,
            UltraResolution::P720,
            UltraResolution::P1080,
        ] {
            for (start, direction) in [(RenderMode::LessLag, 1), (RenderMode::LessLagDoubles, -1)] {
                let mut config = Config {
                    auto_profile: false,
                    render_mode: start,
                    ultra_resolution: resolution,
                    ..Config::default()
                };
                assert!(edit_profile(&mut config, direction));
                assert_eq!(config.render_mode, RenderMode::LessLagUltra);
                assert_eq!(config.ultra_resolution, resolution);
            }
            let mut config = Config {
                platform: Platform::Emulator,
                auto_profile: false,
                render_mode: RenderMode::LessLag,
                ultra_resolution: resolution,
                ..Config::default()
            };
            assert!(edit_profile(&mut config, 1));
            assert_eq!(config.render_mode, RenderMode::LessLagUltra);
            assert_eq!(profile_label(&config, true, config.render_mode), "LLUltra");
            assert!(edit_profile(&mut config, 1));
            assert_eq!(config.render_mode, RenderMode::LessLagDoubles);
            assert_eq!(config.ultra_resolution, resolution);
            assert!(edit_profile(&mut config, 1));
            assert!(config.auto_profile);
            assert_eq!(config.ultra_resolution, resolution);
        }
    }

    #[test]
    fn local_ultra_labels_preserve_ini_auto_and_cleanup_semantics() {
        for (key, label) in [
            ("576", "LLUltra"),
            ("720", "LLUltra+"),
            ("1080", "LLUltra++"),
        ] {
            let mut config = crate::config::parse(&format!(
                "version=1\nrender_mode=llultra\nultra_resolution={key}\n"
            ))
            .unwrap();
            assert!(!config.auto_profile);
            assert_eq!(
                profile_label(&config, true, RenderMode::LessLagUltra),
                label
            );
            assert_eq!(
                profile_label(&config, true, RenderMode::Vanilla),
                "Vanilla+*"
            );
            config.vanilla_plus = false;
            assert_eq!(
                profile_label(&config, true, RenderMode::LessLagUltra),
                label
            );
            assert_eq!(
                profile_label(&config, true, RenderMode::Vanilla),
                "Vanilla*"
            );
            config.render_mode = RenderMode::LessLag;
            assert_eq!(profile_label(&config, true, RenderMode::LessLag), "LessLag");
            config.auto_profile = true;
            assert_eq!(
                profile_label(&config, true, RenderMode::LessLagUltra),
                "Auto:Ultra"
            );
            assert_eq!(
                profile_label(&config, false, RenderMode::LessLagUltra),
                "Auto (unavailable)"
            );
        }
        let old = crate::config::parse("version=1\nrender_mode=llultra\n").unwrap();
        assert_eq!(old.ultra_resolution, UltraResolution::P576);
        assert_eq!(
            profile_label(&old, true, RenderMode::LessLagUltra),
            "LLUltra"
        );
        for alias in ["llultra+", "llultra++"] {
            assert!(crate::config::parse(&format!("version=1\nrender_mode={alias}\n")).is_err());
        }
    }

    #[test]
    fn window_page_rows_toggle_the_hidden_pages_perf_view_and_hints() {
        use crate::config::{HiddenPages, PerfView};
        let mut menu = Menu {
            open: true,
            selected: ROW_PAGE_PLUGINS,
        };
        let mut config = Config::default();
        for platform in [Platform::Console, Platform::Emulator] {
            let window = row_gate(true, true);
            let settings = row_gate(true, false);
            for row in [
                ROW_PAGE_PLUGINS,
                ROW_PAGE_OWN,
                ROW_PAGE_PERF,
                ROW_PERF_VIEW,
                ROW_HINTS,
            ] {
                assert!(row_visible(row, window), "{row} on {platform:?}");
                assert!(!row_visible(row, settings), "{row} on Settings");
            }
            for row in [ROW_YOU, ROW_PEER, ROW_NET_LATENCY, ROW_NET_PROFILE] {
                assert!(!row_visible(row, window), "{row} on Window");
            }
            assert_eq!(
                (0..ROW_COUNT)
                    .filter(|row| row_visible(*row, settings))
                    .count(),
                4
            );
            assert_eq!(first_row(window), ROW_PAGE_PLUGINS);
        }
        // The Settings handler never reaches a Window row: the selection is
        // moved onto the page instead and nothing changes.
        assert!(!menu.handle(Action::Right, &mut config));
        assert_eq!(config, Config::default());
        assert_eq!(menu.selected, ROW_YOU);
        menu.selected = ROW_PAGE_PLUGINS;
        assert!(menu.handle_window(Action::Right, &mut config));
        assert!(config.hide_pages.plugins);
        assert!(menu.handle_window(Action::Left, &mut config));
        assert_eq!(config.hide_pages, HiddenPages::NONE);
        menu.handle_window(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PAGE_OWN);
        assert!(menu.handle_window(Action::Right, &mut config));
        assert!(config.hide_pages.own_pages);
        menu.handle_window(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PAGE_PERF);
        assert!(menu.handle_window(Action::Right, &mut config));
        assert_eq!(
            config.hide_pages,
            HiddenPages {
                plugins: false,
                own_pages: true,
                performance: true
            }
        );
        menu.handle_window(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PERF_VIEW);
        assert_eq!(config.perf_view, PerfView::Graph);
        assert!(menu.handle_window(Action::Right, &mut config));
        assert_eq!(config.perf_view, PerfView::Number);
        assert!(menu.handle_window(Action::Left, &mut config));
        assert_eq!(config.perf_view, PerfView::Graph);
        menu.handle_window(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_HINTS);
        assert!(config.hints);
        assert!(menu.handle_window(Action::Right, &mut config));
        assert!(!config.hints);
        assert!(menu.handle_window(Action::Left, &mut config));
        assert!(config.hints);
        // The page wraps onto itself, never into the Settings rows.
        menu.handle_window(Action::Down, &mut config);
        assert_eq!(menu.selected, ROW_PAGE_PLUGINS);
        menu.handle_window(Action::Up, &mut config);
        assert_eq!(menu.selected, ROW_HINTS);
        // Closed, nothing edits.
        menu.open = false;
        assert!(!menu.handle_window(Action::Right, &mut config));
        assert!(config.hints);
    }

    #[test]
    fn summary_lines_footer_and_edit_marker_read_as_designed() {
        assert_eq!(
            you_line("Vanilla+", "Auto", "Wi-Fi"),
            "Vanilla+ | Auto | Wi-Fi"
        );
        assert_eq!(
            opp_line("Mario", "LessLag", "3", "23ms", "Ethernet"),
            "Mario | LessLag | 3 | 23ms | Ethernet"
        );
        // Unannounced facts drop out instead of showing placeholders.
        assert_eq!(opp_line("Mario", "---", "---", "---", ""), "Mario");
        assert_eq!(
            opp_line("Unavailable", "---", "---", "31ms", "Wi-Fi"),
            "Unavailable | 31ms | Wi-Fi"
        );
        assert_eq!(net_profile_value("LessLag", "Swap"), "LessLag | Swap");
        assert_eq!(
            net_profile_value("Auto:LLDbl", "Active"),
            "Auto:LLDbl | Active"
        );
        assert_eq!(net_profile_value("Vanilla+", ""), "Vanilla+");
        assert_eq!(net_profile_value("Vanilla+", "Vanilla"), "Vanilla+");
        assert_eq!(net_profile_value("Auto:Van+", "Vanilla"), "Auto:Van+");
        assert_eq!(
            perf_footer(Some(16.67), "1080p", true),
            "60.0 fps | 16.67 ms | 1080p | DRS on"
        );
        assert_eq!(
            perf_footer(None, "Unknown", false),
            "--- fps | --- ms | Unknown | DRS off"
        );
        assert_eq!(
            perf_footer(Some(f32::NAN), "720p", false),
            "--- fps | --- ms | 720p | DRS off"
        );
        assert!(HINT_LINE.is_ascii());
        assert!(HINT_LINE.contains("ZR+Left/Right"));
        assert!(HINT_LINE.contains("ZR+X Stealth"));
        assert!(HINT_LINE.contains("ZR+Y Streamer"));
        assert!(HINT_LINE.contains("L+R+X Vanilla+/LLUltra"));
        let mut rows = vec![
            strip_row("You", "Vanilla+ | Auto | Wi-Fi", false),
            strip_row("NetLatency", "Auto", true),
            strip_row("Kirby", "", true),
        ];
        mark_editing(&mut rows, false);
        assert_eq!(rows[1].value, "Auto");
        mark_editing(&mut rows, true);
        assert_eq!(rows[0].value, "Vanilla+ | Auto | Wi-Fi");
        assert_eq!(rows[1].value, "< Auto >");
        assert_eq!(rows[2].value, "", "label-only rows are never marked");
        assert_eq!(strip_line(&rows[1..2]), ">NetLatency:< Auto >");
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
        assert_eq!(menu.selected, ROW_YOU);
        // You is always reachable, including without a peer. Privacy has no
        // separate rows; the optional Opp selector returns on a join.
        let before = config.clone();
        assert!(!menu.handle_peers(Action::Left, &mut config, false));
        assert!(!menu.handle_peers(Action::Right, &mut config, false));
        assert_eq!(config, before, "You has no Left/Right value edit");
        menu.handle_peers(Action::Up, &mut config, false);
        assert_eq!(menu.selected, ROW_NET_PROFILE);
        menu.handle_peers(Action::Down, &mut config, false);
        assert_eq!(menu.selected, ROW_YOU);
        menu.handle_peers(Action::Down, &mut config, false);
        assert_eq!(menu.selected, ROW_NET_LATENCY);
        // Emulator has the same compact navigation.
        config.platform = Platform::Emulator;
        menu.selected = ROW_YOU;
        menu.handle_peers(Action::Up, &mut config, false);
        assert_eq!(menu.selected, ROW_NET_PROFILE);
        menu.handle_peers(Action::Down, &mut config, false);
        assert_eq!(menu.selected, ROW_YOU);
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
    fn match_shape_requires_a_ready_match_and_peer_visibility_is_gated() {
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
        assert!(row_visible(ROW_YOU, row_gate(false, false)));
        assert!(row_visible(ROW_YOU, row_gate(true, false)));
        assert!(!row_visible(ROW_YOU, row_gate(true, true)));
        assert!(row_visible(ROW_NET_LATENCY, row_gate(false, false)));
        assert!(!row_visible(ROW_PEER, row_gate(false, false)));
        assert!(row_visible(ROW_PEER, row_gate(true, false)));
        assert!(!row_visible(ROW_PEER, row_gate(true, true)));
    }

    #[test]
    fn only_ready_match_settings_and_performance_use_the_adaptive_strip() {
        assert!(compact_strip(MenuShape::Horizontal, InfoView::Settings));
        assert!(compact_strip(MenuShape::Horizontal, InfoView::Performance));
        assert!(compact_strip(MenuShape::Horizontal, InfoView::Plugins));
        assert!(compact_strip(MenuShape::Horizontal, InfoView::Window));
        assert!(!compact_strip(MenuShape::Horizontal, InfoView::Debug));
        assert!(!compact_strip(MenuShape::Vertical, InfoView::Settings));
        assert!(!compact_strip(MenuShape::Vertical, InfoView::Performance));
        assert!(!compact_strip(MenuShape::Vertical, InfoView::Plugins));
        assert!(!compact_strip(MenuShape::Vertical, InfoView::Window));
        assert_eq!(
            pane_width(MenuShape::Horizontal, InfoView::Plugins),
            MATCH_STRIP_WIDTH
        );
        assert_eq!(pane_width(MenuShape::Vertical, InfoView::Plugins), 380.0);
        assert_eq!(pane_width(MenuShape::Vertical, InfoView::Window), 380.0);
        assert_eq!(
            pane_width(MenuShape::Horizontal, InfoView::Window),
            MATCH_STRIP_WIDTH
        );
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

    fn strip_row(label: &str, value: &str, selected: bool) -> Row {
        Row {
            label: label.to_owned(),
            value: value.to_owned(),
            selected,
            ..Row::default()
        }
    }

    fn settings_strip(peer: &str) -> (Vec<Row>, Vec<Row>) {
        (
            vec![
                strip_row("Net", "Auto", true),
                strip_row("Profile", "Auto:LLDbl | Active", false),
            ],
            vec![strip_row(
                "Opp",
                &opp_line(peer, "LessLagDoubles", "25", "31ms", "Wi-Fi"),
                true,
            )],
        )
    }

    #[test]
    fn strip_rows_keep_values_inline_and_mark_the_selected_field() {
        let peer = "P".repeat(64);
        let (fields, opponent) = settings_strip(&peer);
        let settings = strip_line(&fields);
        let opponent = strip_line(&opponent);
        assert_eq!(settings, ">Net:Auto   Profile:Auto:LLDbl | Active");
        assert_eq!(
            opponent,
            format!(">Opp:{peer} | LessLagDoubles | 25 | 31ms | Wi-Fi")
        );
        assert!(!settings.contains('\n'));
        assert!(!opponent.contains('\n'));
        // Plugin rows: a group or header is its label alone, a value takes its
        // tone and the pending marker, and the texts join to the line.
        let rows = [
            strip_row("Kirby", "", false),
            Row {
                tone: deluxe_plugin_abi::TONE_WARN,
                pending: true,
                ..strip_row("Hammer", "Variant 3", true)
            },
            Row {
                tone: deluxe_plugin_abi::TONE_ERROR,
                ..strip_row("Health", "Fail", false)
            },
        ];
        assert_eq!(
            strip_segments(&rows),
            vec![
                ("Kirby".to_owned(), deluxe_plugin_abi::TONE_NEUTRAL),
                ("   ".to_owned(), deluxe_plugin_abi::TONE_NEUTRAL),
                (">Hammer:".to_owned(), deluxe_plugin_abi::TONE_NEUTRAL),
                ("Variant 3~".to_owned(), deluxe_plugin_abi::TONE_WARN),
                ("   ".to_owned(), deluxe_plugin_abi::TONE_NEUTRAL),
                ("Health:".to_owned(), deluxe_plugin_abi::TONE_NEUTRAL),
                ("Fail".to_owned(), deluxe_plugin_abi::TONE_ERROR),
            ]
        );
        assert_eq!(
            strip_line(&rows),
            "Kirby   >Hammer:Variant 3~   Health:Fail"
        );
        assert_eq!(strip_line(&[]), "");
    }

    #[test]
    fn windows_page_the_strip_and_centre_the_table() {
        // Paged (strip): the page of four containing the selection.
        assert_eq!(window(0, 0, 4, true), (0, 0, 1, 1));
        assert_eq!(window(0, 3, 4, true), (0, 3, 1, 1));
        assert_eq!(window(3, 9, 4, true), (0, 4, 1, 3));
        assert_eq!(window(4, 9, 4, true), (4, 8, 2, 3));
        assert_eq!(window(8, 9, 4, true), (8, 9, 3, 3));
        assert_eq!(window(50, 9, 4, true), (8, 9, 3, 3));
        // Centred (table): ten rows around the selection, clamped to the list.
        assert_eq!(window(0, 0, 10, false), (0, 0, 1, 1));
        assert_eq!(window(2, 6, 10, false), (0, 6, 1, 1));
        assert_eq!(window(0, 25, 10, false), (0, 10, 1, 1));
        assert_eq!(window(12, 25, 10, false), (7, 17, 1, 1));
        assert_eq!(window(24, 25, 10, false), (15, 25, 1, 1));
        assert_eq!(window(3, 5, 0, false), (0, 0, 1, 1));
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
        let (fields, opponent) = settings_strip(&peer);
        let first = strip_line(&fields);
        // 0.6.0: the second line is the Opp summary (name, profile, buffer,
        // ping, interface) and the timing line.
        let second = format!(
            "{}   {}",
            strip_line(&opponent),
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
    /// A page shaped like the Elite one: two headers, 12 value rows, one passive
    /// status row and three status cells (19 rows), registered on its own page.
    fn status_page() -> (
        crate::plugin_api::Registry,
        Vec<std::sync::Arc<crate::plugin_api::Plugin>>,
        Vec<crate::plugin_api::PageRow>,
    ) {
        use crate::plugin_api::testing::{
            descriptor_with, fake_current, fake_request, FakePlugin, Persist,
        };
        static PAGE: FakePlugin = FakePlugin::new([0, 0, 0, 0], deluxe_plugin_abi::REQUEST_APPLIED);
        const TONES: &[u8] = &[
            deluxe_plugin_abi::TONE_OK,
            deluxe_plugin_abi::TONE_WARN,
            deluxe_plugin_abi::TONE_ERROR,
        ];
        let mut rows: Vec<(String, String, &[&str])> = Vec::new();
        let mut persist = Vec::new();
        let mut add =
            |id: String, label: &str, choices: &'static [&'static str], flags: Persist| {
                rows.push((id, label.to_owned(), choices));
                persist.push(flags);
            };
        let header = Persist {
            header: true,
            no_persist: true,
            ..Persist::default()
        };
        let value = Persist {
            no_persist: true,
            ..Persist::default()
        };
        add("h0".into(), "Stage pool", &[], header);
        add("pool".into(), "Legal stages", &["Off", "On"], value);
        for index in 0..11 {
            add(format!("stage{index}"), "Stage", &["Off", "On"], value);
        }
        add("h1".into(), "Counterpick", &[], header);
        add("enabled".into(), "Counterpick", &["Off", "On"], value);
        add(
            "needs".into(),
            "Needs pool",
            &["No", "Yes"],
            Persist {
                no_persist: true,
                read_only: true,
                ..Persist::default()
            },
        );
        for (id, label) in [
            ("pool_hook", "Pool"),
            ("result_hook", "Results"),
            ("save_state", "Save"),
        ] {
            add(
                id.into(),
                label,
                &["Ready", "Saving", "Not saved"],
                Persist {
                    no_persist: true,
                    read_only: true,
                    cell: true,
                    tones: Some(TONES),
                    ..Persist::default()
                },
            );
        }
        let borrowed: Vec<(&str, &str, &[&str])> = rows
            .iter()
            .map(|(id, label, choices)| (id.as_str(), label.as_str(), *choices))
            .collect();
        let mut holder = descriptor_with(
            "elite",
            "Elite",
            &borrowed,
            None,
            &persist,
            &PAGE as *const FakePlugin as *mut core::ffi::c_void,
            Some(fake_current),
            Some(fake_request),
        );
        holder.desc.abi_version = deluxe_plugin_abi::CELL_ABI_VERSION;
        holder.desc.flags = deluxe_plugin_abi::PLUGIN_FLAG_OWN_PAGE;
        let registry = crate::plugin_api::Registry::new();
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        let plugins = registry.snapshot();
        let rows = crate::plugin_api::rows_for(&plugins, Some(0));
        (registry, plugins, rows)
    }

    #[test]
    fn the_status_table_stays_whole_and_reachable_on_the_table_and_the_strip() {
        let (_registry, plugins, rows) = status_page();
        assert_eq!(rows.len(), 19);
        let selectable: Vec<usize> = (0..rows.len())
            .filter(|index| crate::plugin_api::selectable(&plugins, rows[*index]))
            .collect();
        // Value rows only: the pool, the eleven stages and Counterpick.
        assert_eq!(selectable, (1..13).chain([14]).collect::<Vec<_>>());
        // The vertical table: a centered window of ten. Near the top it shows no
        // table; once its end reaches one cell it takes all three.
        assert_eq!(plugin_window(&plugins, &rows, 1, 10, false), (0, 10, 1, 1));
        assert_eq!(plugin_window(&plugins, &rows, 12, 10, false), (7, 19, 1, 1));
        assert_eq!(plugin_window(&plugins, &rows, 14, 10, false), (9, 19, 1, 1));
        // The strip pages four rows. The last page also holds the passive rows
        // after the final selectable row, so the table is reachable in a match.
        assert_eq!(plugin_window(&plugins, &rows, 1, 4, true), (0, 4, 1, 5));
        assert_eq!(plugin_window(&plugins, &rows, 9, 4, true), (8, 12, 3, 5));
        assert_eq!(plugin_window(&plugins, &rows, 12, 4, true), (12, 19, 4, 4));
        assert_eq!(plugin_window(&plugins, &rows, 14, 4, true), (12, 19, 4, 4));
        // No window ever cuts the table.
        for selected in 0..rows.len() {
            for (size, paged) in [(10, false), (4, true), (3, true), (6, false)] {
                let (start, end, _, _) = plugin_window(&plugins, &rows, selected, size, paged);
                assert!(start < end && end <= rows.len(), "{selected}/{size}");
                let cuts = |edge: usize| edge > 16 && edge < 19;
                assert!(
                    !cuts(start) && !cuts(end),
                    "{selected}/{size}/{paged}: {start}..{end}"
                );
            }
        }
    }

    #[test]
    fn a_window_edge_inside_the_table_moves_to_the_table_edge() {
        // A passive status row followed by a three-cell table.
        use crate::plugin_api::testing::{
            descriptor_with, fake_current, fake_request, FakePlugin, Persist,
        };
        static ONLY: FakePlugin = FakePlugin::new([0, 0, 0, 0], deluxe_plugin_abi::REQUEST_APPLIED);
        const TONES: &[u8] = &[deluxe_plugin_abi::TONE_OK, deluxe_plugin_abi::TONE_ERROR];
        let choices: &[&str] = &["Ready", "Unavailable"];
        let cell = Persist {
            cell: true,
            tones: Some(TONES),
            ..Persist::default()
        };
        let mut holder = descriptor_with(
            "only",
            "Only",
            &[
                ("a", "A", choices),
                ("b", "B", choices),
                ("c", "C", choices),
                ("d", "D", choices),
            ],
            None,
            &[
                Persist {
                    read_only: true,
                    ..Persist::default()
                },
                cell,
                cell,
                cell,
            ],
            &ONLY as *const FakePlugin as *mut core::ffi::c_void,
            Some(fake_current),
            Some(fake_request),
        );
        holder.desc.abi_version = deluxe_plugin_abi::CELL_ABI_VERSION;
        holder.desc.flags = deluxe_plugin_abi::PLUGIN_FLAG_OWN_PAGE;
        let registry = crate::plugin_api::Registry::new();
        assert_eq!(unsafe { registry.register_raw(&holder.desc) }, 0);
        let plugins = registry.snapshot();
        let rows = crate::plugin_api::rows_for(&plugins, Some(0));
        assert!(rows
            .iter()
            .all(|row| !crate::plugin_api::selectable(&plugins, *row)));
        // The end edge inside the table grows to its end; the start edge inside
        // it grows back to its start. Nothing is selectable, so no tail rule.
        assert_eq!(plugin_window(&plugins, &rows, 0, 2, false), (0, 4, 1, 1));
        assert_eq!(plugin_window(&plugins, &rows, 3, 2, false), (1, 4, 1, 1));
        assert_eq!(plugin_window(&plugins, &rows, 0, 2, true), (0, 4, 1, 2));
        assert_eq!(plugin_window(&plugins, &rows, 3, 2, true), (1, 4, 2, 2));
    }

    #[test]
    fn a_cell_run_is_one_strip_entry_with_a_drawn_mark_per_column() {
        let cells = |tones: [u8; 3]| Row {
            cells: ["Pool", "Results", "Save"]
                .iter()
                .zip(tones)
                .map(|(label, tone)| Cell {
                    label: (*label).to_owned(),
                    tone,
                })
                .collect(),
            ..Row::default()
        };
        let ok = deluxe_plugin_abi::TONE_OK;
        let warn = deluxe_plugin_abi::TONE_WARN;
        let rows = [strip_row("Counterpick", "On", true), cells([ok, ok, warn])];
        let segments = strip_segments(&rows);
        let neutral = deluxe_plugin_abi::TONE_NEUTRAL;
        assert_eq!(
            segments,
            vec![
                (">Counterpick:".to_owned(), neutral),
                ("On".to_owned(), neutral),
                ("   ".to_owned(), neutral),
                ("Pool:".to_owned(), neutral),
                (MARK_SLOT.to_owned(), ok),
                (" | ".to_owned(), neutral),
                ("Results:".to_owned(), neutral),
                (MARK_SLOT.to_owned(), ok),
                (" | ".to_owned(), neutral),
                ("Save:".to_owned(), neutral),
                (MARK_SLOT.to_owned(), warn),
            ]
        );
        // The line the font fit measures is exactly those texts, and the slot
        // can never be typed into a plugin label.
        assert_eq!(
            strip_line(&rows),
            ">Counterpick:On   Pool:\u{1} | Results:\u{1} | Save:\u{1}"
        );
        assert!(!deluxe_plugin_abi::valid_label(MARK_SLOT.as_bytes()));
    }
}
