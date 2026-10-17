//! Fork-style global Player Info window. This window does not write panes.
use super::{imgui::*, input, overlay_host, publish, record, settings, Code};
use crate::menu::{strip_line, strip_segments, Row};
use crate::overlay::{FrameTimes, Overlay, View};
use crate::pia_rtt::PresentedPing;
use crate::plugin_api::{page_plugins, rows_for, PageRow, Registry, RowKind};
use std::{sync::Mutex, time::Instant};

struct UiState {
    controls: Overlay,
    frames: FrameTimes,
    previous_frame: Option<Instant>,
    count: u64,
    previous_timing_buttons: u64,
    surface_gen: u64,
}
static UI: Mutex<Option<UiState>> = Mutex::new(None);
static INSTALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
const NO_TITLE_BAR: i32 = 1 << 0;
/// Fields per compact-strip page. The strip never scales text below
/// readability for a long plugin list; it pages instead.
const STRIP_PAGE: usize = 4;
/// Rows per vertical window before a plugin list is clipped around the selection.
const TABLE_WINDOW: usize = 10;

/// True once the menu draw callback was handed to the ImGui host.
pub(super) fn installed() -> bool {
    INSTALLED.load(std::sync::atomic::Ordering::Acquire)
}

unsafe fn text(value: &str) {
    // Explicit end pointer: values need neither NUL termination nor printf escaping.
    igTextUnformatted(
        value.as_ptr().cast(),
        value.as_ptr().add(value.len()).cast(),
    );
}

unsafe fn row(label: &str, value: &str) {
    igTableNextRow(0, 0.0);
    igTableNextColumn();
    text(label);
    igTableNextColumn();
    text(value);
}

/// Bordered label and value columns, plus the untitled opponent column when
/// `columns` is 3.
unsafe fn begin_table(id: &[u8], columns: i32, label_width: f32) -> bool {
    if !igBeginTable(id.as_ptr().cast(), columns, BORDERS, Vec2::default(), 0.0) {
        return false;
    }
    igTableSetupColumn(c"##label".as_ptr(), WIDTH_FIXED, label_width, 0);
    igTableSetupColumn(c"##value".as_ptr(), WIDTH_STRETCH, 1.0, 0);
    if columns > 2 {
        igTableSetupColumn(c"##peer".as_ptr(), WIDTH_STRETCH, 1.0, 0);
    }
    true
}

/// Keep one near-transparent primitive like the fork: the backend requires a
/// nonempty command list even while the visible window is hidden.
unsafe fn hidden_window() {
    igPushStyleVar_Float(WINDOW_BORDER, 0.0);
    igPushStyleVar_Vec2(WINDOW_PADDING, Vec2::default());
    igPushStyleVar_Vec2(WINDOW_MIN_SIZE, Vec2::default());
    igSetNextWindowPos(Vec2::default(), ALWAYS, Vec2::default());
    igSetNextWindowSize(Vec2 { x: 1.0, y: 1.0 }, ALWAYS);
    igSetNextWindowBgAlpha(0.0);
    if igBegin(
        c"##deluxe_hidden".as_ptr(),
        std::ptr::null_mut(),
        HIDDEN_FLAGS,
    ) {
        let list = igGetWindowDrawList();
        if !list.is_null() {
            let p = igGetWindowPos();
            ImDrawList_AddRectFilled(
                list,
                p,
                Vec2 {
                    x: p.x + 1.0,
                    y: p.y + 1.0,
                },
                0x01000000,
                0.0,
                0,
            );
        }
    }
    igEnd();
    igPopStyleVar(3);
}

const fn rgb(r: f32, g: f32, b: f32) -> Vec4 {
    Vec4 {
        x: r,
        y: g,
        z: b,
        w: 1.0,
    }
}

/// Frame-time and `TONE_OK` green.
const GREEN: Vec4 = rgb(0.45, 0.90, 0.45);
/// Frame-time and `TONE_WARN` yellow.
const YELLOW: Vec4 = rgb(0.95, 0.85, 0.25);
/// Frame-time and `TONE_ERROR` red.
const RED: Vec4 = rgb(0.95, 0.35, 0.35);
/// Timing line without samples.
const GREY: Vec4 = rgb(0.70, 0.70, 0.70);

fn frame_color(mean: Option<f32>) -> Vec4 {
    match mean {
        Some(ms) if ms > 19.17 => RED,
        Some(ms) if ms > 17.0 => YELLOW,
        _ => GREEN,
    }
}

/// The colour `pia_rtt` presents the opponent's ping in, when it has one.
fn ping_color(ping: &PresentedPing) -> Option<Vec4> {
    ping.color.map(|color| Vec4 {
        x: color.r,
        y: color.g,
        z: color.b,
        w: color.a,
    })
}

/// Text colour for a plugin `TONE_*` byte; neutral keeps the style default.
fn tone_color(tone: u8) -> Option<Vec4> {
    match tone {
        deluxe_plugin_abi::TONE_OK => Some(GREEN),
        deluxe_plugin_abi::TONE_WARN => Some(YELLOW),
        deluxe_plugin_abi::TONE_ERROR => Some(RED),
        _ => None,
    }
}

fn latest_frame_time(frames: &FrameTimes) -> Option<f32> {
    if frames.count == 0 {
        return None;
    }
    let index = (frames.next + crate::overlay::FRAME_HISTORY - 1) % crate::overlay::FRAME_HISTORY;
    Some(frames.values[index])
}

fn timing_line(frames: &FrameTimes) -> (String, Vec4) {
    let mean = frames.average();
    let latest = latest_frame_time(frames);
    let color = if mean.is_some() && latest.is_some() {
        frame_color(mean)
    } else {
        GREY
    };
    (crate::menu::compact_timing_line(mean, latest), color)
}

/// Facts the Performance table (`strip` false: long labels, the DRS move and
/// adaptive detail, backoff, memory and clock) and the in-match strip (`strip`
/// true: short labels and one `Adp` field) read from the same sources.
fn perf_facts(width: f32, height: f32, strip: bool) -> Vec<(&'static str, String)> {
    let on_off = |on: bool| if on { "On" } else { "Off" }.to_owned();
    let resolution = crate::resolution::status();
    let fighter = super::fighter_main::view();
    let settings = super::settings();
    let counters = crate::perf_scaler::global().counters();
    let mut facts = vec![
        (
            if strip { "Res" } else { "Resolution" },
            crate::resolution::applied_label(resolution.applied).to_owned(),
        ),
        ("DRS", on_off(resolution.dynamic)),
        (
            if strip { "Trig" } else { "Triggers" },
            if crate::resolution::triggers_available(fighter.callback_armed, fighter.count_only) {
                "Available"
            } else {
                "Unavailable"
            }
            .to_owned(),
        ),
    ];
    if !strip {
        facts.push(("DRS moves", drs_moves_label(&settings, counters)));
        facts.push(("DRS adaptive", drs_adaptive_label(&settings, counters)));
    } else if !settings.drs_moves.is_empty() || settings.drs_adaptive.enabled {
        facts.push((
            "Adp",
            format!(
                "{}{}",
                on_off(settings.drs_adaptive.enabled),
                if counters.adaptive_active || counters.move_active.is_some() {
                    "*"
                } else {
                    ""
                }
            ),
        ));
    }
    if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
        facts.push((
            if strip { "Out" } else { "Output" },
            format!("{width:.0}x{height:.0}"),
        ));
    }
    if !strip {
        facts.push(("Backoff", super::dispatcher_backoff::status_label()));
        facts.push((
            "Memory",
            crate::hang_trace::memory_label(super::hang_trace::memory()),
        ));
        facts.push((
            "Clock",
            crate::clock_probe::status_label(super::clock_probe::mhz(), super::clock_probe::hoc()),
        ));
    }
    facts
}

/// The frame-time block of the vertical window. `graph` false (`perf_view=number`)
/// prints the mean as text instead of the 180-sample plot, which is the one
/// ImGui primitive here that scales with the sample count. `ping` is the
/// opponent's ping while the opponent column shows.
unsafe fn performance(
    frames: &FrameTimes,
    width: f32,
    height: f32,
    sx: f32,
    sy: f32,
    ping: Option<&PresentedPing>,
    graph: bool,
) {
    if !begin_table(b"fps_info\0", 2, 100.0 * sx) {
        return;
    }
    if let Some(label) = ping.and_then(|ping| ping.performance.as_ref()) {
        igTableNextRow(0, 0.0);
        igTableNextColumn();
        text("Ping");
        igTableNextColumn();
        paint_text(label, ping.and_then(ping_color));
    }
    for (label, value) in perf_facts(width, height, false) {
        row(label, &value);
    }
    let mean = frames.average();
    let color = frame_color(mean);
    igPushStyleColor_Vec4(TEXT, color);
    row(
        "FPS",
        &mean
            .map(|ms| format!("{:.0}", 1000.0 / ms))
            .unwrap_or_else(|| "---".into()),
    );
    igPopStyleColor(1);
    let label = mean
        .map(|ms| format!("{ms:.2} ms\0"))
        .unwrap_or_else(|| "---\0".into());
    if !graph {
        igPushStyleColor_Vec4(TEXT, color);
        row("FrameTime", label.trim_end_matches('\0'));
        igPopStyleColor(1);
        igEndTable();
        return;
    }
    igTableNextRow(0, 65.0 * sy);
    igTableNextColumn();
    text("FrameTime");
    igTableNextColumn();
    igPushStyleColor_Vec4(PLOT_LINES, color);
    igPushStyleColor_Vec4(PLOT_LINES_HOVERED, color);
    let available = igGetContentRegionAvail();
    igPlotLines_FloatPtr(
        c"##frametime".as_ptr(),
        frames.values.as_ptr(),
        frames.count as i32,
        frames.plot_offset() as i32,
        label.as_ptr().cast(),
        0.0,
        35.0,
        Vec2 {
            x: available.x,
            y: 60.0 * sy,
        },
        std::mem::size_of::<f32>() as i32,
    );
    igPopStyleColor(2);
    igEndTable();
}

/// Peer facts of one draw, shared by the Settings, Performance and Debug views.
struct PeerFacts<'a> {
    frame: crate::peer_transport::PeerFrame,
    /// A station is connected (the Peer row exists).
    connected: bool,
    /// The opponent column is visible (not Streamer, a station connected).
    show_remote: bool,
    ping: &'a PresentedPing,
}

/// One Settings row: the shared `row` (label, local value, selection), the
/// opponent column, and the strip field (short label and value) when the row
/// is in the strip. `ping` marks the opponent cell drawn in the ping colour.
struct SettingCell {
    row: Row,
    remote: String,
    ping: bool,
    strip: Option<(&'static str, String)>,
}

struct CompactLines {
    first: String,
    second: String,
    first_color: Option<Vec4>,
    /// When set, the first row is drawn as these `(text, tone)` segments on
    /// one line instead of `first`; their concatenation equals `first`.
    first_segments: Option<Vec<(String, u8)>>,
}

fn marked(on: bool, label: &str) -> String {
    if on {
        format!("> {label}")
    } else {
        label.to_owned()
    }
}

/// Rows the settings window actually draws (Platform is omitted) and, with
/// the opponent column, the `IF:` / `Ping:` / `PeerNet:` / `PeerProfile:`
/// facts of the strip's second line.
fn setting_cells(
    config: &crate::config::Config,
    selected: usize,
    compact_match: bool,
    peers: &PeerFacts,
    scene: &super::auto_driver::Snapshot,
) -> (Vec<SettingCell>, Vec<String>) {
    use crate::menu::{
        ROW_NET_LATENCY, ROW_NET_PROFILE, ROW_PAGE_OWN, ROW_PAGE_PERF, ROW_PAGE_PLUGINS, ROW_PEER,
        ROW_PERF_VIEW, ROW_STEALTH, ROW_STREAMER, ROW_ULTRA_RES,
    };
    let show_remote = peers.show_remote;
    let peer = &peers.frame.peer;
    let dash = || {
        if show_remote {
            "---".to_owned()
        } else {
            String::new()
        }
    };
    let when_remote = |value: String| if show_remote { value } else { String::new() };
    let peer_name = if show_remote {
        peer.known
            .then(|| crate::peer_transport::name_label(&peer.name, peer.name_len))
            .flatten()
            .unwrap_or("Unavailable")
            .to_owned()
    } else {
        String::new()
    };
    let cell = |label: &str,
                row: Option<usize>,
                local: String,
                remote: String,
                short: Option<&'static str>| {
        SettingCell {
            strip: short.map(|short| (short, local.clone())),
            row: Row {
                label: label.to_owned(),
                value: local,
                selected: row.is_some_and(|row| row == selected),
                ..Row::default()
            },
            remote,
            ping: false,
        }
    };
    let mut cells = vec![
        cell("Name", None, "You".to_owned(), peer_name.clone(), None),
        cell(
            "Interface",
            None,
            super::connection_transport::local_interface_label().to_owned(),
            when_remote(peer.interface.to_owned()),
            None,
        ),
    ];
    if peers.connected && (show_remote || compact_match) {
        // The table names the peer on the Name row; the strip's own Peer line
        // carries the name, or "Hidden" without the opponent column.
        let shown = if show_remote {
            peer_name
        } else {
            "Hidden".to_owned()
        };
        let local = if show_remote {
            String::new()
        } else {
            shown.clone()
        };
        cells.push(SettingCell {
            strip: Some(("Peer", shown)),
            ..cell("Peer", Some(ROW_PEER), local, String::new(), None)
        });
    }
    if show_remote {
        cells.push(SettingCell {
            ping: true,
            ..cell(
                "Ping",
                None,
                "---".to_owned(),
                peers.ping.opponent.clone(),
                None,
            )
        });
    }
    let buffer = if config.input_buffer == -1 {
        "Auto".to_owned()
    } else {
        config.input_buffer.to_string()
    };
    let remote_latency = when_remote(crate::peer_transport::remote_latency_label(peer.latency));
    cells.push(cell(
        "NetLatency",
        Some(ROW_NET_LATENCY),
        buffer,
        remote_latency.clone(),
        Some("Net"),
    ));
    let profile = crate::auto_policy::profile_row_label(
        config.auto_profile,
        scene.ready,
        config.render_mode,
        scene.request,
    );
    let profile_state = crate::menu::net_profile_status(
        scene.request,
        super::renderer::mode(),
        super::nvn::active_count(),
        crate::render_policy::load_machine(),
        super::renderer::status() != "Awaiting window",
    );
    let peer_settings = if show_remote && peer.known && !peer.settings.is_empty() {
        peer.settings.to_owned()
    } else {
        dash()
    };
    cells.push(cell(
        "NetProfile",
        Some(ROW_NET_PROFILE),
        profile,
        peer_settings.clone(),
        Some("Profile"),
    ));
    cells.push(cell(
        "Status",
        None,
        profile_state.to_owned(),
        String::new(),
        None,
    ));
    // The toggle rows: Stealth, Streamer, UltraRes (console only), then the
    // window pages in the chord cycle and the frame-time presentation
    // (0.5.3). Settings has no page toggle: it is always in the cycle.
    let stealth =
        crate::menu::stealth_value(super::stealth(), crate::peer_transport::latched_stealth());
    let page_label = crate::config::page_label;
    let mut toggles: Vec<(usize, &str, &str, &str)> = vec![
        (ROW_STEALTH, "Stealth", "Stealth", stealth),
        (
            ROW_STREAMER,
            "Streamer",
            "Stream",
            if config.streamer { "On" } else { "Off" },
        ),
    ];
    if config.platform == crate::config::Platform::Console {
        toggles.push((
            ROW_ULTRA_RES,
            "UltraRes",
            "UltraRes",
            config.ultra_resolution.label(),
        ));
    }
    toggles.extend([
        (
            ROW_PAGE_PLUGINS,
            "PluginsPage",
            "PlugPg",
            page_label(config.hide_pages.plugins),
        ),
        (
            ROW_PAGE_OWN,
            "OwnPages",
            "OwnPg",
            page_label(config.hide_pages.own_pages),
        ),
        (
            ROW_PAGE_PERF,
            "PerfPage",
            "PerfPg",
            page_label(config.hide_pages.performance),
        ),
        (
            ROW_PERF_VIEW,
            "PerfView",
            "PerfView",
            config.perf_view.label(),
        ),
    ]);
    for (row, label, short, value) in toggles {
        cells.push(cell(
            label,
            Some(row),
            value.to_owned(),
            dash(),
            Some(short),
        ));
    }
    let mut facts = Vec::new();
    if show_remote {
        if !peer.interface.is_empty() {
            facts.push(format!("IF:{}", peer.interface));
        }
        facts.push(format!("Ping:{}", peers.ping.opponent));
        if remote_latency != "---" {
            facts.push(format!("PeerNet:{remote_latency}"));
        }
        if peer_settings != "---" {
            facts.push(format!("PeerProfile:{peer_settings}"));
        }
    }
    (cells, facts)
}

unsafe fn paint_text(value: &str, color: Option<Vec4>) {
    if let Some(color) = color {
        igPushStyleColor_Vec4(TEXT, color);
        text(value);
        igPopStyleColor(1);
    } else {
        text(value);
    }
}

/// Strip rows of the Settings cells: the Peer line alone (`peer`), kept on
/// the optional opponent row so a long name cannot crowd the local controls,
/// or every other field.
fn settings_strip(cells: &[SettingCell], peer: bool) -> Vec<Row> {
    cells
        .iter()
        .filter_map(|cell| {
            let (short, value) = cell.strip.as_ref()?;
            ((*short == "Peer") == peer).then(|| Row {
                label: (*short).to_owned(),
                value: value.clone(),
                selected: cell.row.selected,
                ..Row::default()
            })
        })
        .collect()
}

fn compact_settings_lines(
    cells: &[SettingCell],
    remote_facts: &[String],
    frames: &FrameTimes,
) -> CompactLines {
    let mut supporting = Vec::with_capacity(remote_facts.len() + 2);
    let peer = strip_line(&settings_strip(cells, true));
    if !peer.is_empty() {
        supporting.push(peer);
    }
    supporting.extend(remote_facts.iter().cloned());
    supporting.push(timing_line(frames).0);
    CompactLines {
        first: strip_line(&settings_strip(cells, false)),
        second: supporting.join("   "),
        first_color: None,
        first_segments: None,
    }
}

/// Debug-view text for the 0.5.2 move table: the enabled set, and the move
/// currently holding a report when there is one.
fn drs_moves_label(
    settings: &crate::config::Config,
    counters: crate::perf_scaler::TriggerCounters,
) -> String {
    if settings.drs_moves.is_empty() {
        return "Off".to_owned();
    }
    let holding = counters
        .move_active
        .and_then(|index| crate::drs_triggers::MOVES.get(usize::from(index)))
        .map(|entry| entry.name);
    match holding {
        Some(name) => format!("{} ({name} active)", settings.drs_moves.serialize()),
        None => settings.drs_moves.serialize(),
    }
}

/// Debug-view text for the adaptive trigger: threshold and frame counts, and
/// whether it currently holds a report.
fn drs_adaptive_label(
    settings: &crate::config::Config,
    counters: crate::perf_scaler::TriggerCounters,
) -> String {
    if !settings.drs_adaptive.enabled {
        return "Off".to_owned();
    }
    format!(
        "{} ms, {}/{} frames{}",
        crate::drs_triggers::format_threshold(settings.drs_adaptive.threshold_hundredths_ms),
        settings.drs_adaptive.trigger_frames,
        settings.drs_adaptive.release_frames,
        if counters.adaptive_active {
            " (active)"
        } else {
            ""
        }
    )
}

fn compact_performance_lines(
    frames: &FrameTimes,
    width: f32,
    height: f32,
    peers: &PeerFacts,
) -> CompactLines {
    let (timing, color) = timing_line(frames);
    let mut details: Vec<String> = perf_facts(width, height, true)
        .into_iter()
        .map(|(label, value)| format!("{label}:{value}"))
        .collect();
    if peers.show_remote {
        details.push(format!("Ping:{}", peers.ping.opponent));
    }
    CompactLines {
        first: timing,
        second: details.join("   "),
        first_color: Some(color),
        first_segments: None,
    }
}

/// The drawn part of the shared Plugins page (`page` None) or one own page.
struct PluginPage {
    /// The own page's plugin label, or `Plugins`.
    title: String,
    /// The window of rows around the selection: a group row or header is
    /// label-only, an option row carries the plugin's current value.
    rows: Vec<Row>,
    /// Index of `rows[0]` in the full list, and that list's length.
    start: usize,
    total: usize,
    /// One-based strip page and page count.
    page_index: usize,
    pages: usize,
}

/// Builds the window of `size` rows around `selected` (`paged` for the
/// strip). Only the windowed options are asked for `current`, once per frame;
/// headers are never asked.
fn plugin_page(
    registry: &Registry,
    page: Option<usize>,
    selected: usize,
    size: usize,
    paged: bool,
) -> PluginPage {
    let plugins = registry.snapshot();
    let title = page
        .and_then(|page| {
            page_plugins(&plugins)
                .get(page)
                .map(|index| plugins[*index].label.clone())
        })
        .unwrap_or_else(|| "Plugins".to_owned());
    let all = rows_for(&plugins, page);
    let (start, end, page_index, pages) = crate::menu::window(selected, all.len(), size, paged);
    let rows = all[start..end]
        .iter()
        .enumerate()
        .map(|(offset, row)| match *row {
            PageRow::Group(plugin) => Row {
                label: plugins[plugin].label.clone(),
                ..Row::default()
            },
            PageRow::Option { plugin, option } => {
                let spec = &plugins[plugin].options[option];
                if spec.kind == RowKind::Header {
                    return Row {
                        label: spec.label.clone(),
                        ..Row::default()
                    };
                }
                let value = registry.value(plugin, &plugins[plugin], option);
                Row {
                    label: spec.label.clone(),
                    value: value.label,
                    tone: value.tone,
                    pending: value.pending,
                    selected: start + offset == selected,
                }
            }
        })
        .collect();
    PluginPage {
        title,
        rows,
        start,
        total: all.len(),
        page_index,
        pages,
    }
}

/// Compact Plugins page or own page: the strip page containing the
/// selection, then `<title> p/N` and the same timing line Settings uses.
fn compact_plugin_lines(page: &PluginPage, frames: &FrameTimes) -> CompactLines {
    let (timing, _) = timing_line(frames);
    let (first, first_segments) = if page.rows.is_empty() {
        (format!("{}: none registered", page.title), None)
    } else {
        (strip_line(&page.rows), Some(strip_segments(&page.rows)))
    };
    CompactLines {
        first,
        second: format!(
            "{} {}/{}   {timing}",
            page.title, page.page_index, page.pages
        ),
        first_color: None,
        first_segments,
    }
}

/// Vertical Plugins page or own page: the window around the selection, with
/// `...` markers where the list continues past either end.
unsafe fn draw_plugins_vertical(page: &PluginPage, sx: f32) {
    let marker = |label: &str, value: &str| Row {
        label: label.to_owned(),
        value: value.to_owned(),
        ..Row::default()
    };
    let mut shown = Vec::with_capacity(page.rows.len() + 2);
    if page.total == 0 {
        shown.push(marker("Plugins", "none registered"));
    }
    if page.start > 0 {
        shown.push(marker("...", ""));
    }
    shown.extend_from_slice(&page.rows);
    if page.start + page.rows.len() < page.total {
        shown.push(marker("...", ""));
    }
    // Option rows sit two spaces under their group row.
    let indent = if page.total == 0 { "" } else { "  " };
    draw_rows(b"deluxe_plugins\0", 150.0 * sx, &shown, indent, None);
}

/// The vertical table Settings and the plugin pages share: `rows` in two
/// columns, or three with the opponent `remote` cells (text and colour per
/// row). A label-only row shows its label alone; any other row shows
/// `indent` + label (`> ` when selected) and its value in its tone, with
/// ` (pending)` while a plugin request is outstanding.
unsafe fn draw_rows(
    id: &[u8],
    label_width: f32,
    rows: &[Row],
    indent: &str,
    remote: Option<&[(String, Option<Vec4>)]>,
) {
    if !begin_table(id, if remote.is_some() { 3 } else { 2 }, label_width) {
        return;
    }
    for (index, row) in rows.iter().enumerate() {
        igTableNextRow(0, 0.0);
        igTableNextColumn();
        if row.value.is_empty() {
            text(&marked(row.selected, &row.label));
            continue;
        }
        text(&marked(row.selected, &format!("{indent}{}", row.label)));
        igTableNextColumn();
        let value = if row.pending {
            format!("{} (pending)", row.value)
        } else {
            row.value.clone()
        };
        paint_text(&value, tone_color(row.tone));
        if let Some((peer, color)) = remote.and_then(|cells| cells.get(index)) {
            igTableNextColumn();
            paint_text(peer, *color);
        }
    }
    igEndTable();
}

unsafe fn draw_compact_lines(lines: &CompactLines) {
    igPushTextWrapPos(-1.0);
    match &lines.first_segments {
        Some(segments) => {
            // One unwrapped line: each segment continues the previous with no
            // extra spacing, so the row measures the same as `lines.first`.
            for (index, (segment, tone)) in segments.iter().enumerate() {
                if index > 0 {
                    igSameLine(0.0, 0.0);
                }
                paint_text(segment, tone_color(*tone));
            }
        }
        None => paint_text(&lines.first, lines.first_color),
    }
    if !lines.second.is_empty() {
        text(&lines.second);
    }
    igPopTextWrapPos();
}

unsafe fn draw_settings_vertical(cells: &[SettingCell], peers: &PeerFacts, sx: f32) {
    let rows: Vec<Row> = cells.iter().map(|cell| cell.row.clone()).collect();
    let remote: Vec<(String, Option<Vec4>)> = cells
        .iter()
        .map(|cell| {
            (
                cell.remote.clone(),
                cell.ping.then(|| ping_color(peers.ping)).flatten(),
            )
        })
        .collect();
    draw_rows(
        b"conn_player_info\0",
        110.0 * sx,
        &rows,
        "",
        peers.show_remote.then_some(remote.as_slice()),
    );
}

unsafe fn draw_debug(peers: &PeerFacts, sx: f32) {
    if !begin_table(b"deluxe_debug\0", 2, 100.0 * sx) {
        return;
    }
    row(
        "Build",
        &crate::build_info::GIT_COMMIT[..crate::build_info::GIT_COMMIT.len().min(12)],
    );
    row("Renderer", super::renderer::mode().label());
    row(
        "Online tracker",
        if super::online::ready() {
            "Ready"
        } else {
            "Unavailable"
        },
    );
    row("Engine", super::renderer::status());
    row("Timing", super::capture::status());
    if peers.show_remote {
        row("Ping detail", peers.ping.detail);
    }
    row("Capture", "L+R+Up (30 s)");
    row("CSS capture", &super::capture::css_label());
    row("Match capture", &super::capture::match_label());
    igEndTable();
}

unsafe extern "C" fn draw() {
    let sample = input::read();
    let buttons = sample.buttons;
    super::capture::observe(buttons);
    let Ok(mut guard) = UI.try_lock() else {
        return;
    };
    let ui = guard.get_or_insert_with(|| UiState {
        controls: Overlay::default(),
        frames: FrameTimes::default(),
        previous_frame: None,
        count: 0,
        previous_timing_buttons: 0,
        surface_gen: 0,
    });
    let sampled = ui.count.is_multiple_of(300);
    if sampled {
        record(Code::OverlayEntered, ui.count, ui.controls.view as u64);
    }
    ui.count += 1;
    super::response_probe::draw(sample);
    let now = Instant::now();
    if let Some(previous) = ui.previous_frame.replace(now) {
        ui.frames
            .push(now.duration_since(previous).as_secs_f32() * 1000.0);
    }
    let mut config = settings();
    let saved = config.clone();
    let previous_view = ui.controls.view;
    let command_edge = ui.controls.command_edge(buttons);
    let (surface_gen, surface_kind) = super::native_banner::surface_sample();
    let surface_fresh = surface_gen != 0 && surface_gen != ui.surface_gen;
    ui.surface_gen = surface_gen;
    let scene = super::auto_driver::snapshot();
    let native = crate::menu::NativePads {
        allow: crate::menu::native_banner_allowed(scene.ready, scene.connection, scene.status),
        surface: if surface_gen != 0 {
            crate::menu::NativeSurface::from_byte(surface_kind)
        } else {
            crate::menu::NativeSurface::None
        },
        fresh: surface_fresh,
    };
    // Training and matches: a Left/Right edit in the window needs ZR held so
    // a taunt or a stray press cannot change a setting mid-game (ZL + D-pad
    // swaps the controlled fighter in Training).
    ui.controls.edit_hold =
        scene.ready && scene.status != crate::auto_policy::SceneStatus::Inactive;
    if ui.controls.poll_native(buttons, &mut config, native) {
        match crate::menu::profile_action(&saved, &config) {
            crate::menu::ProfileAction::EnableAuto => {
                crate::set_auto_profile(true);
            }
            crate::menu::ProfileAction::SelectManual(mode) => {
                crate::set_auto_profile(false);
                crate::select_user_profile(mode);
            }
            crate::menu::ProfileAction::Unchanged => {}
        }
        publish(&config);
        // `a` carried the FX mode until 0.5.0 and is always 0 now.
        record(Code::MenuChanged, 0, config.input_buffer as i64 as u64);
    }
    crate::plugin_api::REGISTRY.reconcile_pending();
    if let Some(step) = crate::plugin_api::REGISTRY.take_last_step() {
        record(
            Code::PluginRequest,
            (step.plugin as u64 & 0xffff) | ((step.option as u64 & 0xffff) << 16),
            (step.requested as u64 & 0xffff) | ((u64::from(step.status.code()) & 0xff) << 16),
        );
    }
    let view = ui.controls.view;
    if let Some(edge) = command_edge {
        record(
            Code::OverlayCommand,
            buttons,
            previous_view as u64
                | ((view as u64) << 8)
                | (u64::from(sample.npad_id) << 16)
                | (u64::from(sample.style_flags) << 24)
                | (u64::from(edge) << 56),
        );
    }
    let frame = crate::peer_transport::peer_frame();
    let connected = crate::peer_transport::station_connected(frame.selection);
    let show_remote =
        crate::peer_transport::opponent_column_visible(config.streamer, frame.selection);
    if view == View::Settings {
        let gate = crate::menu::row_gate(connected, config.platform);
        if !crate::menu::row_visible(ui.controls.menu.selected, gate) {
            ui.controls.menu.selected = crate::menu::move_row(ui.controls.menu.selected, 1, gate);
        }
    }
    let (publication, epoch, published_gen, draw_gen) = super::pia_rtt::current_publication();
    let identity = if show_remote {
        crate::peer_transport::display_identity(frame.selection)
    } else {
        crate::pia_rtt::RttIdentity::Busy
    };
    let ping =
        crate::pia_rtt::present_published(identity, publication, epoch, published_gen, draw_gen);
    let peers = PeerFacts {
        frame,
        connected,
        show_remote,
        ping: &ping,
    };
    super::capture::set_view(view as u8);
    let pressed = buttons & !ui.previous_timing_buttons;
    ui.previous_timing_buttons = buttons;
    let shoulders = crate::overlay::L | crate::overlay::R;
    if view == View::Debug && buttons & shoulders == shoulders && pressed & crate::overlay::UP != 0
    {
        super::capture::request();
    }
    #[cfg(feature = "coverage")]
    if view == View::Debug
        && buttons & shoulders == shoulders
        && pressed & crate::overlay::LEFT != 0
    {
        super::coverage::request_manual();
    }
    if view != previous_view {
        record(Code::OverlayView, previous_view as u64, view as u64);
    }
    if view == View::Hidden {
        hidden_window();
    } else {
        draw_window(ui, view, &config, &scene, &peers);
    }
    if sampled {
        record(Code::OverlayExited, ui.count, view as u64);
    }
}

/// Positions, sizes and titles the visible window, then draws the view's
/// body: the compact strip in a ready match, or the vertical tables.
unsafe fn draw_window(
    ui: &UiState,
    view: View,
    config: &crate::config::Config,
    scene: &super::auto_driver::Snapshot,
    peers: &PeerFacts,
) {
    let (width, height) = overlay_host::display_size();
    let sx = width / 1920.0;
    let sy = height / 1080.0;
    // SetNextWindowPos runs before this frame's window is measured. For a
    // compact strip, use the maximum design width to preserve its left edge.
    let shape = crate::menu::menu_shape(scene.ready, scene.status);
    let info_view = match view {
        View::Debug => crate::menu::InfoView::Debug,
        View::Performance => crate::menu::InfoView::Performance,
        View::Plugins | View::PluginPage => crate::menu::InfoView::Plugins,
        View::Settings | View::Hidden => crate::menu::InfoView::Settings,
    };
    let own_page = (view == View::PluginPage).then_some(ui.controls.page);
    let compact_strip = crate::menu::compact_strip(shape, info_view);
    let configured = crate::menu::pane_width(shape, info_view) * sx;
    let x = if compact_strip {
        ((width - configured) * 0.5).max(0.0)
    } else {
        crate::hud_place::origin_x(
            crate::hud_place::on_right(scene.ready, scene.connection, scene.status),
            width,
            None,
            configured,
            crate::hud_place::margin_for(width),
        )
    };
    let settings = (view == View::Settings).then(|| {
        setting_cells(
            config,
            ui.controls.menu.selected,
            compact_strip,
            peers,
            scene,
        )
    });
    let plugins = matches!(view, View::Plugins | View::PluginPage).then(|| {
        plugin_page(
            &crate::plugin_api::REGISTRY,
            own_page,
            ui.controls.plugin_row,
            if compact_strip {
                STRIP_PAGE
            } else {
                TABLE_WINDOW
            },
            compact_strip,
        )
    });
    let compact_lines = if compact_strip {
        match view {
            View::Settings => settings
                .as_ref()
                .map(|(cells, facts)| compact_settings_lines(cells, facts, &ui.frames)),
            View::Performance => Some(compact_performance_lines(&ui.frames, width, height, peers)),
            View::Plugins | View::PluginPage => plugins
                .as_ref()
                .map(|page| compact_plugin_lines(page, &ui.frames)),
            View::Hidden | View::Debug => None,
        }
    } else {
        None
    };
    igSetNextWindowPos(Vec2 { x, y: 0.0 }, ALWAYS, Vec2::default());
    igSetNextWindowSize(
        Vec2 {
            // Let ImGui fit compact rows to content. `configured` remains
            // the positioning and adaptive-font width bound.
            x: if compact_strip { 0.0 } else { configured },
            // Zero requests ImGui's content-fit height. Compact match views
            // contain only one or two unwrapped text lines.
            y: 0.0,
        },
        ALWAYS,
    );
    if let Some(alpha) = crate::menu::match_window_alpha(shape) {
        igSetNextWindowBgAlpha(alpha);
    }
    if compact_strip {
        igPushStyleVar_Float(WINDOW_BORDER, 0.0);
        igPushStyleVar_Vec2(
            WINDOW_PADDING,
            Vec2 {
                x: 12.0 * sx,
                y: 3.0 * sy,
            },
        );
    }
    let font_size = compact_lines
        .as_ref()
        .map(|lines| {
            crate::menu::compact_font_size(&lines.first, &lines.second, configured, sx, sy)
        })
        .unwrap_or(16.0 * sy);
    igPushFont(std::ptr::null_mut(), font_size);
    // An own page is titled with its plugin's label. The `##` suffix keeps
    // ImGui's window identity stable across plugins with the same label.
    let page_title = plugins
        .as_ref()
        .filter(|_| own_page.is_some() && !compact_strip)
        .and_then(|page| {
            std::ffi::CString::new(format!("{}##deluxe_plugin_page", page.title)).ok()
        });
    let title: *const std::ffi::c_char = match &page_title {
        Some(title) => title.as_ptr(),
        None if compact_strip => c"##deluxe_match_strip".as_ptr(),
        None => c"Player Info".as_ptr(),
    };
    let flags = if compact_strip {
        WINDOW_FLAGS | NO_TITLE_BAR
    } else {
        WINDOW_FLAGS
    };
    if igBegin(title, std::ptr::null_mut(), flags) {
        match (&compact_lines, view) {
            (Some(lines), _) => draw_compact_lines(lines),
            (None, View::Settings) => {
                if let Some((cells, _)) = &settings {
                    draw_settings_vertical(cells, peers, sx);
                }
            }
            (None, View::Plugins | View::PluginPage) => {
                if let Some(page) = &plugins {
                    draw_plugins_vertical(page, sx);
                }
            }
            (None, View::Debug) => draw_debug(peers, sx),
            (None, View::Performance | View::Hidden) => {}
        }
        // Settings, Debug and Performance end with the frame-time table.
        if compact_lines.is_none() && !matches!(view, View::Plugins | View::PluginPage) {
            performance(
                &ui.frames,
                width,
                height,
                sx,
                sy,
                peers.show_remote.then_some(peers.ping),
                config.perf_view == crate::config::PerfView::Graph,
            );
        }
    }
    igEnd();
    igPopFont();
    if compact_strip {
        igPopStyleVar(2);
    }
}

pub(super) fn install() {
    super::response_probe::init();
    if overlay_host::install(draw) {
        INSTALLED.store(true, std::sync::atomic::Ordering::Release);
    }
}
