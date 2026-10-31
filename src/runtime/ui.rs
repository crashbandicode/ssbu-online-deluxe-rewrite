//! Fork-style global Player Info window. This window does not write panes.
use super::{cells, imgui::*, input, overlay_host, publish, record, settings, Code};
use crate::hoc_game_aware::{Indicator, MarkTone};
use crate::menu::{
    mark_editing, net_profile_value, opp_line, perf_footer, strip_line, strip_segments, you_line,
    Cell, ProfileAction, Row, YouIcon, HINT_LINE, MARK_SLOT,
};
use crate::overlay::{FrameTimes, Overlay, View};
use crate::pia_rtt::PresentedPing;
use crate::plugin_api::{
    cell_group, page_plugins, rows_for, selectable, PageRow, Registry, RowKind,
};
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

/// The strip's first line (`rows` as coloured segments, or plain text in a
/// colour when there are none) and its second line.
type Strip = (Vec<Row>, String, Option<Vec4>, String);

/// True once the menu draw callback was handed to the ImGui host.
pub(super) fn installed() -> bool {
    INSTALLED.load(std::sync::atomic::Ordering::Acquire)
}

unsafe fn text(value: &str) {
    // Explicit end pointer: values need neither NUL termination nor printf escaping.
    let end = value.as_ptr().add(value.len());
    igTextUnformatted(value.as_ptr().cast(), end.cast());
}

unsafe fn row(label: &str, value: &str) {
    igTableNextRow(0, 0.0);
    igTableNextColumn();
    text(label);
    igTableNextColumn();
    text(value);
}

/// A Performance fact whose value wraps at the column's end instead of being
/// clipped at the window edge (0.8.1): `Clock` with the Horizon-OC gate state
/// and `Memory` outgrew the fixed window width.
unsafe fn wrapped_row(label: &str, value: &str) {
    igTableNextRow(0, 0.0);
    igTableNextColumn();
    text(label);
    igTableNextColumn();
    igPushTextWrapPos(0.0);
    text(value);
    igPopTextWrapPos();
}

unsafe fn paint_text(value: &str, color: Option<Vec4>) {
    if let Some(color) = color {
        igPushStyleColor_Vec4(TEXT, color);
    }
    text(value);
    if color.is_some() {
        igPopStyleColor(1);
    }
}

/// Bordered label and value columns. (The opponent column left in 0.6.0: the
/// peer's facts are the `Opp` summary line of the Settings page.)
unsafe fn begin_table(id: &[u8], label_width: f32) -> bool {
    if !igBeginTable(id.as_ptr().cast(), 2, BORDERS, Vec2::default(), 0.0) {
        return false;
    }
    igTableSetupColumn(c"##label".as_ptr(), WIDTH_FIXED, label_width, 0);
    igTableSetupColumn(c"##value".as_ptr(), WIDTH_STRETCH, 1.0, 0);
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
    let hidden = c"##deluxe_hidden".as_ptr();
    if igBegin(hidden, std::ptr::null_mut(), HIDDEN_FLAGS) {
        let list = igGetWindowDrawList();
        if !list.is_null() {
            let p = igGetWindowPos();
            let end = Vec2 {
                x: p.x + 1.0,
                y: p.y + 1.0,
            };
            ImDrawList_AddRectFilled(list, p, end, 0x01000000, 0.0, 0);
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
/// Frame around the You-line mode icon that L+R+X toggles (0.9.0).
const LIGHT_BLUE: Vec4 = rgb(0.45, 0.78, 1.0);

#[derive(Clone, Copy)]
struct Privacy {
    stealth: bool,
    streamer: bool,
    reconnect: bool,
    /// The game-aware Horizon-OC clock after the privacy icons (0.8.1), in
    /// either mode while the fork answers (0.9.0).
    oc: Option<MarkTone>,
    /// The framed icon while the You row is selected (0.9.0).
    focus: Option<YouIcon>,
}

/// Light-blue frame around one icon cell, `from` to `to` in font units from
/// `origin`, on the current window's draw list.
unsafe fn focus_frame(origin: Vec2, from: (f32, f32), to: (f32, f32)) {
    let size = igGetFontSize();
    let list = igGetWindowDrawList();
    if list.is_null() {
        return;
    }
    ImDrawList_AddRect(
        list,
        Vec2 {
            x: origin.x + from.0 * size,
            y: origin.y + from.1 * size,
        },
        Vec2 {
            x: origin.x + to.0 * size,
            y: origin.y + to.1 * size,
        },
        igGetColorU32_Vec4(LIGHT_BLUE),
        size * 0.15,
        0,
        size * 0.09,
    );
}

/// What the game-aware gate shows for `config` right now (0.8.1).
fn oc_indicator(config: &crate::config::Config) -> Indicator {
    crate::hoc_game_aware::indicator(
        config.hoc_oc,
        super::hoc_client::capability(),
        super::pia_rtt::ready(),
        super::hoc_client::applying(),
    )
}

/// Clock mark (0.8.1), drawn like the privacy icons and one text line tall:
/// green with a hub while Smash's saved Horizon-OC profile is applied, grey
/// while Horizon-OC's own fallback holds, yellow without Pia evidence or with
/// the fork not answering, and (0.9.0) white with a hub for `hoc_oc=always`,
/// where the profile applies everywhere. Quarter arcs and lines only, the
/// primitives the embedded and the borrowed ImGui both provide.
unsafe fn oc_clock(tone: MarkTone, focused: bool) {
    let size = igGetFontSize();
    let origin = igGetCursorScreenPos();
    let list = igGetWindowDrawList();
    igDummy(Vec2 { x: size, y: size });
    if list.is_null() {
        return;
    }
    if focused {
        focus_frame(origin, (-0.12, -0.12), (1.12, 1.12));
    }
    let point = |x: f32, y: f32| Vec2 {
        x: origin.x + x * size,
        y: origin.y + y * size,
    };
    let color = match tone {
        MarkTone::Lit => igGetColorU32_Vec4(GREEN),
        MarkTone::Dim => igGetColorU32_Vec4(GREY),
        MarkTone::Warn => igGetColorU32_Vec4(YELLOW),
        MarkTone::Always => igGetColorU32_Col(TEXT, 1.0),
    };
    let stroke = size * 0.085;
    for (from, corner, to) in [
        ((0.5, 0.05), (0.95, 0.05), (0.95, 0.5)),
        ((0.95, 0.5), (0.95, 0.95), (0.5, 0.95)),
        ((0.5, 0.95), (0.05, 0.95), (0.05, 0.5)),
        ((0.05, 0.5), (0.05, 0.05), (0.5, 0.05)),
    ] {
        ImDrawList_AddBezierQuadratic(
            list,
            point(from.0, from.1),
            point(corner.0, corner.1),
            point(to.0, to.1),
            color,
            stroke,
            8,
        );
    }
    ImDrawList_AddLine(list, point(0.5, 0.5), point(0.5, 0.22), color, stroke);
    ImDrawList_AddLine(list, point(0.5, 0.5), point(0.72, 0.5), color, stroke);
    if matches!(tone, MarkTone::Lit | MarkTone::Always) {
        ImDrawList_AddCircleFilled(list, point(0.5, 0.5), size * 0.1, color, 8);
    }
}

/// Font-independent You-line icons. Reserve one text line so the table and
/// strip fit the geometry just like text, in either embedded or borrowed ImGui.
unsafe fn privacy_icons(privacy: Privacy) {
    let size = igGetFontSize();
    let origin = igGetCursorScreenPos();
    let list = igGetWindowDrawList();
    igDummy(Vec2 {
        x: size * 2.5,
        y: size,
    });
    if list.is_null() {
        return;
    }
    let point = |x: f32, y: f32| Vec2 {
        x: origin.x + x * size,
        y: origin.y + y * size,
    };
    match privacy.focus {
        Some(YouIcon::Stealth) => focus_frame(origin, (-0.12, -0.12), (1.12, 1.12)),
        Some(YouIcon::Streamer) => focus_frame(origin, (1.33, -0.12), (2.57, 1.12)),
        _ => {}
    }
    let stroke = size * 0.085;
    let eye = if privacy.reconnect {
        igGetColorU32_Vec4(YELLOW)
    } else {
        igGetColorU32_Col(TEXT, 1.0)
    };
    if privacy.stealth {
        // Closed lid with lashes: Stealth enabled.
        ImDrawList_AddBezierQuadratic(
            list,
            point(0.05, 0.35),
            point(0.5, 0.95),
            point(0.95, 0.35),
            eye,
            stroke,
            12,
        );
        for (x, y, end_x) in [(0.24, 0.55, 0.16), (0.5, 0.65, 0.5), (0.76, 0.55, 0.84)] {
            ImDrawList_AddLine(list, point(x, y), point(end_x, y + 0.2), eye, stroke);
        }
    } else {
        // Open eye: ordinary announcements are allowed by the saved choice.
        for y in [0.02, 0.98] {
            ImDrawList_AddBezierQuadratic(
                list,
                point(0.05, 0.5),
                point(0.5, y),
                point(0.95, 0.5),
                eye,
                stroke,
                12,
            );
        }
        ImDrawList_AddCircleFilled(list, point(0.5, 0.5), size * 0.13, eye, 12);
    }
    let tv = igGetColorU32_Vec4(if privacy.streamer { GREEN } else { GREY });
    ImDrawList_AddRect(
        list,
        point(1.45, 0.24),
        point(2.45, 0.82),
        tv,
        size * 0.08,
        0,
        stroke,
    );
    ImDrawList_AddLine(list, point(1.7, 0.02), point(1.95, 0.24), tv, stroke);
    ImDrawList_AddLine(list, point(2.2, 0.02), point(1.95, 0.24), tv, stroke);
    ImDrawList_AddLine(list, point(1.95, 0.82), point(1.95, 0.96), tv, stroke);
    ImDrawList_AddLine(list, point(1.72, 0.96), point(2.18, 0.96), tv, stroke);
    if privacy.streamer {
        ImDrawList_AddRectFilled(
            list,
            point(1.58, 0.37),
            point(2.32, 0.69),
            tv,
            size * 0.03,
            0,
        );
    }
    if privacy.reconnect {
        igSameLine(0.0, 0.0);
        paint_text("*", Some(YELLOW));
    }
    if let Some(tone) = privacy.oc {
        igSameLine(0.0, size * 0.3);
        oc_clock(tone, privacy.focus == Some(YouIcon::Oc));
    }
}

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

/// The strip's timing line and its colour: grey until a frame time exists.
fn timing_line(frames: &FrameTimes) -> (String, Vec4) {
    let history = crate::overlay::FRAME_HISTORY;
    let mean = frames.average();
    let latest = (frames.count > 0).then(|| frames.values[(frames.next + history - 1) % history]);
    let color = if latest.is_some() {
        frame_color(mean)
    } else {
        GREY
    };
    (crate::menu::compact_timing_line(mean, latest), color)
}

/// One window row. `value` empty is a label-only row.
fn entry(label: &str, value: &str, selected: bool) -> Row {
    Row {
        label: label.to_owned(),
        value: value.to_owned(),
        selected,
        ..Row::default()
    }
}

/// Facts the Performance table (`strip` false: long labels, the DRS move and
/// adaptive detail, backoff, memory and clock) and the in-match strip (`strip`
/// true: short labels and one `Adp` field) read from the same sources.
fn perf_facts(width: f32, height: f32, strip: bool) -> Vec<(&'static str, String)> {
    let on_off = |on: bool| if on { "On" } else { "Off" }.to_owned();
    let label = |long: &'static str, short: &'static str| if strip { short } else { long };
    let resolution = crate::resolution::status();
    let fighter = super::fighter_main::view();
    let settings = super::settings();
    let counters = crate::perf_scaler::global().counters();
    let armed = crate::resolution::triggers_available(fighter.callback_armed, fighter.count_only);
    let triggers = if armed { "Available" } else { "Unavailable" };
    let applied = crate::resolution::applied_label(resolution.applied);
    let mut facts = vec![
        (label("Resolution", "Res"), applied.to_owned()),
        ("DRS", on_off(resolution.dynamic)),
        (label("Triggers", "Trig"), triggers.to_owned()),
    ];
    let adaptive = &settings.drs_adaptive;
    if !strip {
        facts.push(("DRS moves", drs_moves_label(&settings, counters)));
        facts.push(("DRS adaptive", drs_adaptive_label(&settings, counters)));
    } else if !settings.drs_moves.is_empty() || adaptive.enabled {
        let active = counters.adaptive_active || counters.move_active.is_some();
        let mark = if active { "*" } else { "" };
        facts.push(("Adp", format!("{}{mark}", on_off(adaptive.enabled))));
    }
    if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
        facts.push((label("Output", "Out"), format!("{width:.0}x{height:.0}")));
    }
    if !strip {
        let (mhz, hoc) = (super::clock_probe::mhz(), super::clock_probe::hoc());
        let memory = crate::hang_trace::memory_label(super::hang_trace::memory());
        facts.push(("Backoff", super::dispatcher_backoff::status_label()));
        facts.push(("Memory", memory));
        let mut clock = crate::clock_probe::status_label(mhz, hoc);
        if let Some(gate) = oc_indicator(&settings).label() {
            clock.push_str(", ");
            clock.push_str(gate);
        }
        facts.push(("Clock", clock));
    }
    facts
}

/// The frame-time block of the Performance and Debug views (until 0.5.7 the
/// Settings view ended with it too; since 0.6.0 Settings has the one-line
/// footer). `graph` false (`perf_view=number`) prints the mean as text instead
/// of the 180-sample plot, which is the one ImGui primitive here that scales
/// with the sample count. `ping` is the opponent's ping while the peer shows.
unsafe fn performance(
    frames: &FrameTimes,
    width: f32,
    height: f32,
    sx: f32,
    sy: f32,
    ping: Option<&PresentedPing>,
    graph: bool,
) {
    if !begin_table(b"fps_info\0", 100.0 * sx) {
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
        wrapped_row(label, &value);
    }
    let mean = frames.average();
    let color = frame_color(mean);
    let fps = mean.map_or("---".to_owned(), |ms| format!("{:.0}", 1000.0 / ms));
    igPushStyleColor_Vec4(TEXT, color);
    row("FPS", &fps);
    igPopStyleColor(1);
    let label = mean.map_or("---\0".to_owned(), |ms| format!("{ms:.2} ms\0"));
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
    let size = Vec2 {
        x: igGetContentRegionAvail().x,
        y: 60.0 * sy,
    };
    igPlotLines_FloatPtr(
        c"##frametime".as_ptr(),
        frames.values.as_ptr(),
        frames.count as i32,
        frames.plot_offset() as i32,
        label.as_ptr().cast(),
        0.0,
        35.0,
        size,
        std::mem::size_of::<f32>() as i32,
    );
    igPopStyleColor(2);
    igEndTable();
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
    let moves = settings.drs_moves.serialize();
    let table = &crate::drs_triggers::MOVES;
    let holding = counters
        .move_active
        .and_then(|index| table.get(usize::from(index)));
    holding.map_or(moves.clone(), |entry| {
        format!("{moves} ({} active)", entry.name)
    })
}

/// Debug-view text for the adaptive trigger: threshold and frame counts, and
/// whether it currently holds a report.
fn drs_adaptive_label(
    settings: &crate::config::Config,
    counters: crate::perf_scaler::TriggerCounters,
) -> String {
    let adaptive = &settings.drs_adaptive;
    if !adaptive.enabled {
        return "Off".to_owned();
    }
    let threshold = crate::drs_triggers::format_threshold(adaptive.threshold_hundredths_ms);
    let active = if counters.adaptive_active {
        " (active)"
    } else {
        ""
    };
    let (trigger, release) = (adaptive.trigger_frames, adaptive.release_frames);
    format!("{threshold} ms, {trigger}/{release} frames{active}")
}

/// Peer facts of one draw, shared by the Settings, Performance and Debug views.
struct PeerFacts {
    frame: crate::peer_transport::PeerFrame,
    /// The peer shows (not Streamer, a live station connected): the `Opp`
    /// line, the strip's peer facts and the Performance Ping row.
    show_remote: bool,
    ping: PresentedPing,
}

/// What one draw shows: the settings after this frame's edits, the scene, the
/// peer facts, and whether the ZR edit hold is on (0.6.0: the selected value
/// is marked `< value >` while it is).
struct Frame {
    config: crate::config::Config,
    scene: super::auto_driver::Snapshot,
    peers: PeerFacts,
    editing: bool,
    privacy: Privacy,
}

/// The strip's short label for a Settings or Window row; the `You` / `Opp`
/// summary lines have none (Opp has its own second-line entry) and stay out
/// of the strip's first line.
fn short_label(label: &str) -> Option<&'static str> {
    Some(match label {
        "NetLatency" => "Net",
        "NetProfile" => "Profile",
        "PluginsPage" => "PlugPg",
        "OwnPages" => "OwnPg",
        "PerfPage" => "PerfPg",
        "PerfView" => "PerfView",
        "Hints" => "Hints",
        _ => return None,
    })
}

/// The Settings page (0.6.0): the selectable `You` summary line, the `Opp` summary line
/// while the peer shows (selectable: Left/Right cycles the station), then
/// the match-time rows NetLatency, NetProfile with the engine status folded
/// in. Privacy is drawn as You-line icons; UltraRes is part of NetProfile.
/// Returns the rows, the
/// colour of each value (the `Opp` line takes the ping's stability colour)
/// and the strip's second-line facts (the `Opp` line).
fn settings_rows(
    config: &crate::config::Config,
    selected: usize,
    peers: &PeerFacts,
    scene: &super::auto_driver::Snapshot,
    editing: bool,
) -> (Vec<Row>, Vec<Option<Vec4>>, Vec<String>) {
    use crate::menu::{ROW_NET_LATENCY, ROW_NET_PROFILE, ROW_PEER, ROW_YOU};
    let peer = &peers.frame.peer;
    let name = crate::peer_transport::name_label(&peer.name, peer.name_len);
    let peer_name = name.filter(|_| peer.known).unwrap_or("Unavailable");
    let remote_latency = crate::peer_transport::remote_latency_label(peer.latency);
    let announced = peer.known && !peer.settings.is_empty();
    let peer_settings = if announced { peer.settings } else { "---" };
    let profile = crate::menu::profile_label(config, scene.ready, scene.request);
    let status = crate::menu::net_profile_status(
        scene.request,
        super::renderer::mode(),
        super::nvn::active_count(),
        crate::render_policy::load_machine(),
        super::renderer::status() != "Awaiting window",
    );
    let interface = super::connection_transport::local_interface_label();
    let buffer =
        crate::native_banner::local_latency_label(config.input_buffer, super::auto_buffer());
    let (mut rows, mut colors) = (Vec::new(), Vec::new());
    let mut push = |label: &str, row: Option<usize>, value: &str, color: Option<Vec4>| {
        colors.push(color);
        rows.push(entry(label, value, row == Some(selected)));
    };
    push(
        "You",
        Some(ROW_YOU),
        &you_line(&profile, &buffer, interface),
        None,
    );
    if peers.show_remote {
        let opp = opp_line(
            peer_name,
            peer_settings,
            &remote_latency,
            &peers.ping.opponent,
            peer.interface,
        );
        push("Opp", Some(ROW_PEER), &opp, ping_color(&peers.ping));
    }
    push("NetLatency", Some(ROW_NET_LATENCY), &buffer, None);
    push(
        "NetProfile",
        Some(ROW_NET_PROFILE),
        &net_profile_value(&profile, status),
        None,
    );
    // You uses face-button shortcuts, not Left/Right edits of its summary text.
    mark_editing(&mut rows[1..], editing);
    let facts = rows
        .iter()
        .filter(|row| row.label == "Opp")
        .map(|row| strip_line(std::slice::from_ref(row)))
        .collect();
    (rows, colors, facts)
}

/// The Window page (0.6.0): the three page toggles, PerfView and Hints, the
/// rows that configure the window itself and until 0.5.7 sat at the bottom
/// of Settings.
fn window_rows(config: &crate::config::Config, selected: usize, editing: bool) -> Vec<Row> {
    use crate::menu::{ROW_HINTS, ROW_PAGE_OWN, ROW_PAGE_PERF, ROW_PAGE_PLUGINS, ROW_PERF_VIEW};
    let (page, hide) = (crate::config::page_label, &config.hide_pages);
    let mut rows = vec![
        entry(
            "PluginsPage",
            page(hide.plugins),
            selected == ROW_PAGE_PLUGINS,
        ),
        entry("OwnPages", page(hide.own_pages), selected == ROW_PAGE_OWN),
        entry(
            "PerfPage",
            page(hide.performance),
            selected == ROW_PAGE_PERF,
        ),
        entry(
            "PerfView",
            config.perf_view.label(),
            selected == ROW_PERF_VIEW,
        ),
        entry("Hints", page(!config.hints), selected == ROW_HINTS),
    ];
    mark_editing(&mut rows, editing);
    rows
}

/// Under the Settings table (0.6.0): the one-line performance footer in the
/// frame-time colour (grey until a frame time exists), then, under Settings
/// and Window alike, the dim chord hint line while `hints` is on. The hint
/// wraps to the window width.
unsafe fn footer(frames: &FrameTimes, config: &crate::config::Config, perf: bool, reconnect: bool) {
    if perf {
        let mean = frames.average();
        let resolution = crate::resolution::status();
        let line = perf_footer(
            mean,
            crate::resolution::applied_label(resolution.applied),
            resolution.dynamic,
        );
        let color = if mean.is_some() {
            frame_color(mean)
        } else {
            GREY
        };
        paint_text(&line, Some(color));
    }
    if reconnect {
        paint_text("* Stealth changes after reconnect", Some(YELLOW));
    }
    if config.hints {
        igPushTextWrapPos(0.0);
        paint_text(HINT_LINE, Some(GREY));
        igPopTextWrapPos();
    }
}

/// The drawn part of the shared Plugins page (`page` None) or one own page.
struct PluginPage {
    /// The own page's plugin label, or `Plugins`.
    title: String,
    /// The rows around the selection: a group row or header is label-only,
    /// an option row carries the plugin's current value. The vertical table
    /// adds `...` markers where the list continues past either end.
    rows: Vec<Row>,
    total: usize,
    /// One-based strip page and page count.
    page_index: usize,
    pages: usize,
}

/// Builds the window of `size` rows around `selected` (`paged` for the
/// strip). Only the windowed options are asked for `current`, once per frame;
/// headers are never asked.
fn plugin_rows(
    registry: &Registry,
    page: Option<usize>,
    selected: usize,
    size: usize,
    paged: bool,
) -> PluginPage {
    let plugins = registry.snapshot();
    let own = page.and_then(|page| page_plugins(&plugins).get(page).copied());
    let title = own.map_or("Plugins".to_owned(), |index| plugins[index].label.clone());
    let all = rows_for(&plugins, page);
    let (start, end, page_index, pages) =
        crate::menu::plugin_window(&plugins, &all, selected, size, paged);
    let mut rows = Vec::with_capacity(end - start + 2);
    if !paged && all.is_empty() {
        rows.push(entry("Plugins", "none registered", false));
    }
    if !paged && start > 0 {
        rows.push(entry("...", "", false));
    }
    let mut index = start;
    while index < end {
        let row = all[index];
        // A run of status cells is one entry (one table); the window never
        // cuts it, so the index jumps to the end of the run.
        if let Some((first, last)) = cell_group(&plugins, &all, index) {
            let cells = (first..last)
                .filter_map(|at| match all[at] {
                    PageRow::Option { plugin, option } => Some(Cell {
                        label: plugins[plugin].options[option].label.clone(),
                        tone: registry.value(plugin, &plugins[plugin], option).tone,
                    }),
                    PageRow::Group(_) => None,
                })
                .collect();
            rows.push(Row {
                cells,
                ..Row::default()
            });
            index = last;
            continue;
        }
        rows.push(match row {
            PageRow::Group(plugin) => entry(&plugins[plugin].label, "", false),
            PageRow::Option { plugin, option } => {
                let spec = &plugins[plugin].options[option];
                if spec.kind == RowKind::Header {
                    entry(&spec.label, "", false)
                } else {
                    let value = registry.value(plugin, &plugins[plugin], option);
                    Row {
                        label: spec.label.clone(),
                        value: value.label,
                        tone: value.tone,
                        pending: value.pending,
                        // Only an editable row can hold the cursor, so a
                        // passive status row is never highlighted.
                        selected: index == selected && selectable(&plugins, row),
                        cells: Vec::new(),
                    }
                }
            }
        });
        index += 1;
    }
    if !paged && end < all.len() {
        rows.push(entry("...", "", false));
    }
    PluginPage {
        title,
        rows,
        total: all.len(),
        page_index,
        pages,
    }
}

fn debug_rows(peers: &PeerFacts) -> Vec<Row> {
    let commit = &crate::build_info::GIT_COMMIT[..crate::build_info::GIT_COMMIT.len().min(12)];
    let tracker = if super::online::ready() {
        "Ready"
    } else {
        "Unavailable"
    };
    let mut rows = vec![
        ("Build", commit.to_owned()),
        ("Renderer", super::renderer::mode().label().to_owned()),
        ("Online tracker", tracker.to_owned()),
        ("Engine", super::renderer::status().to_owned()),
        ("Timing", super::capture::status().to_owned()),
    ];
    if peers.show_remote {
        rows.push(("Ping detail", peers.ping.detail.to_owned()));
    }
    rows.push(("Capture", "L+R+Up (30 s)".to_owned()));
    rows.push(("CSS capture", super::capture::css_label()));
    rows.push(("Match capture", super::capture::match_label()));
    rows.into_iter()
        .map(|(label, value)| entry(label, &value, false))
        .collect()
}

/// The vertical table: `rows` in two columns. A label-only row shows its
/// label alone; any other row shows `indent` + label (`> ` when selected) and
/// its value in its tone, or in `colors[index]` when that is set (the `Opp`
/// line's ping colour), with ` (pending)` while a plugin request is
/// outstanding.
unsafe fn draw_table(
    id: &[u8],
    label_width: f32,
    rows: &[Row],
    indent: &str,
    colors: Option<&[Option<Vec4>]>,
    privacy: Option<Privacy>,
) {
    // A status-cell row ends the two-column table, draws its own table, and the
    // rows after it continue in a new two-column table with a distinct ImGui id.
    let mut parts = 0usize;
    let mut open = false;
    for (index, row) in rows.iter().enumerate() {
        if !row.cells.is_empty() {
            if open {
                igEndTable();
                open = false;
            }
            let mut cell_id = id[..id.len() - 1].to_vec();
            cell_id.extend_from_slice(format!("#cells{index}\0").as_bytes());
            let draws: Vec<cells::CellDraw> = row
                .cells
                .iter()
                .map(|cell| cells::CellDraw {
                    label: &cell.label,
                    ok: cell.tone == deluxe_plugin_abi::TONE_OK,
                    color: tone_color(cell.tone),
                })
                .collect();
            cells::table(&cell_id, &draws);
            continue;
        }
        if !open {
            let mut part_id = id[..id.len() - 1].to_vec();
            if parts > 0 {
                part_id.extend_from_slice(format!("#part{parts}").as_bytes());
            }
            part_id.push(0);
            if !begin_table(&part_id, label_width) {
                return;
            }
            parts += 1;
            open = true;
        }
        igTableNextRow(0, 0.0);
        igTableNextColumn();
        let mark = if row.selected { "> " } else { "" };
        if row.value.is_empty() {
            text(&format!("{mark}{}", row.label));
            continue;
        }
        text(&format!("{mark}{indent}{}", row.label));
        if row.label == "You" {
            if let Some(privacy) = privacy {
                igSameLine(0.0, igGetFontSize() * 0.25);
                privacy_icons(privacy);
            }
        }
        igTableNextColumn();
        let pending = if row.pending { " (pending)" } else { "" };
        let color = colors
            .and_then(|colors| colors.get(index).copied().flatten())
            .or_else(|| tone_color(row.tone));
        // Wrap at the column's end: a long `Opp` line (name, profile, buffer,
        // ping, interface) takes a second line instead of being clipped.
        igPushTextWrapPos(0.0);
        paint_text(&format!("{}{pending}", row.value), color);
        igPopTextWrapPos();
    }
    if open {
        igEndTable();
    }
}

/// The in-match strip: `rows` as one unwrapped line of coloured segments
/// (each continues the previous with no extra spacing, so the row measures
/// the same as `strip_line`), or `plain` in `color` when there are no rows;
/// then `second`.
unsafe fn draw_strip((rows, plain, color, second): &Strip, privacy: Option<(Privacy, bool)>) {
    igPushTextWrapPos(-1.0);
    if let Some((privacy, selected)) = privacy {
        text(if selected { ">You:" } else { "You:" });
        igSameLine(0.0, igGetFontSize() * 0.25);
        privacy_icons(privacy);
        igSameLine(0.0, igGetFontSize() * 0.75);
    }
    if rows.is_empty() {
        paint_text(plain, *color);
    }
    for (index, (segment, tone)) in strip_segments(rows).iter().enumerate() {
        if index > 0 {
            igSameLine(0.0, 0.0);
        }
        if segment == MARK_SLOT {
            // A status cell's mark: a check for the ready tone, else a cross.
            cells::mark(
                *tone == deluxe_plugin_abi::TONE_OK,
                tone_color(*tone),
                false,
            );
        } else {
            paint_text(segment, tone_color(*tone));
        }
    }
    if !second.is_empty() {
        text(second);
    }
    igPopTextWrapPos();
}

/// Input, native-pad gating, profile actions, `publish`, the plugin
/// reconcile and the chord records of one draw. Returns what the window shows.
unsafe fn poll(ui: &mut UiState, sample: input::Sample) -> Frame {
    let buttons = sample.buttons;
    let mut config = settings();
    let saved = config.clone();
    let previous_view = ui.controls.view;
    let command_edge = ui.controls.command_edge(buttons);
    let (surface_gen, surface_kind) = super::native_banner::surface_sample();
    let scene = super::auto_driver::snapshot();
    let native = crate::menu::NativePads {
        allow: crate::menu::native_banner_allowed(scene.ready, scene.connection, scene.status),
        surface: crate::menu::NativeSurface::from_byte(if surface_gen != 0 {
            surface_kind
        } else {
            0
        }),
        fresh: surface_gen != 0 && surface_gen != ui.surface_gen,
    };
    ui.surface_gen = surface_gen;
    // Every scene (0.5.6, operator rule): a Left/Right edit in the open window
    // needs ZR held, in menus as much as in Training and matches, so a taunt,
    // a menu D-pad press or a stray edge cannot change a setting (ZL + D-pad
    // swaps the controlled fighter in Training). Up/Down still move the
    // selection. Until 0.5.5 the hold applied only while a fight was live.
    ui.controls.edit_hold = true;
    // The OC icon is a You focus stop, and L+R+X can change `hoc_oc`, only
    // while its clock shows: the game-aware fork answered (0.9.0).
    ui.controls.oc_available = oc_indicator(&config).mark().is_some();
    if ui.controls.poll_native(buttons, &mut config, native) {
        match crate::menu::profile_action(&saved, &config) {
            ProfileAction::EnableAuto => {
                crate::set_auto_profile(true);
            }
            ProfileAction::SelectManual(mode) => {
                crate::set_auto_profile(false);
                crate::select_user_profile(mode);
            }
            ProfileAction::Unchanged => {}
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
        let views = previous_view as u64 | ((view as u64) << 8);
        let pad = (u64::from(sample.npad_id) << 16) | (u64::from(sample.style_flags) << 24);
        record(
            Code::OverlayCommand,
            buttons,
            views | pad | (u64::from(edge) << 56),
        );
    }
    let frame = crate::peer_transport::peer_frame();
    // Live-peer rule (0.5.5): a bound station that has gone silent (no RTT
    // sample for LIVE_MS, as after a declined rematch) counts as disconnected
    // for every peer row, the strip's peer line and the ping cell.
    let live = super::pia_rtt::station_live(frame.selection);
    let show_remote =
        crate::peer_transport::opponent_column_visible(config.streamer, frame.selection, live);
    if matches!(view, View::Settings | View::Window) {
        // The `Opp` line is a stop only while it shows (0.6.0: Streamer hides
        // it, so it is skipped then too).
        let gate = crate::menu::row_gate(show_remote, view == View::Window);
        if !crate::menu::row_visible(ui.controls.menu.selected, gate) {
            ui.controls.menu.selected = crate::menu::move_row(ui.controls.menu.selected, 1, gate);
        }
    }
    let identity = if show_remote {
        crate::peer_transport::display_identity(frame.selection)
    } else {
        crate::pia_rtt::RttIdentity::Busy
    };
    let ping = crate::pia_rtt::present_published(identity, &super::pia_rtt::current_publication());
    super::capture::set_view(view as u8);
    let pressed = buttons & !ui.previous_timing_buttons;
    ui.previous_timing_buttons = buttons;
    let shoulders = crate::overlay::L | crate::overlay::R;
    let debug_chord = view == View::Debug && buttons & shoulders == shoulders;
    if debug_chord && pressed & crate::overlay::UP != 0 {
        super::capture::request();
    }
    #[cfg(feature = "coverage")]
    if debug_chord && pressed & crate::overlay::LEFT != 0 {
        super::coverage::request_manual();
    }
    if view != previous_view {
        record(Code::OverlayView, previous_view as u64, view as u64);
    }
    let peers = PeerFacts {
        frame,
        show_remote,
        ping,
    };
    let oc = oc_indicator(&config).mark();
    let you_selected = view == View::Settings
        && ui.controls.menu.open
        && ui.controls.menu.selected == crate::menu::ROW_YOU;
    let privacy = Privacy {
        stealth: config.stealth,
        streamer: config.streamer,
        reconnect: crate::peer_transport::latched_stealth()
            .is_some_and(|latched| latched != config.stealth),
        oc,
        focus: you_selected.then(|| ui.controls.you_focus.shown(oc.is_some())),
    };
    Frame {
        config,
        scene,
        peers,
        editing: crate::overlay::edit_modifier(buttons),
        privacy,
    }
}

unsafe extern "C" fn draw() {
    let sample = input::read();
    super::capture::observe(sample.buttons);
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
    let frame = poll(ui, sample);
    let view = ui.controls.view;
    if view == View::Hidden {
        hidden_window();
    } else {
        draw_window(ui, view, &frame);
    }
    if sampled {
        record(Code::OverlayExited, ui.count, view as u64);
    }
}

/// The ready-match strip of `view`: Settings fields with the `Opp` and timing
/// facts; the Window fields with the timing line; the plugin page with
/// `title p/N` and the timing line; or the timing line over the short
/// performance facts.
fn strip_lines(
    ui: &UiState,
    view: View,
    frame: &Frame,
    page: Option<&PluginPage>,
    display: Vec2,
) -> Strip {
    let peers = &frame.peers;
    let (timing, color) = timing_line(&ui.frames);
    let selected = ui.controls.menu.selected;
    let short =
        |row: &Row| short_label(&row.label).map(|short| entry(short, &row.value, row.selected));
    match (view, page) {
        (View::Settings, _) => {
            let (rows, _, mut facts) =
                settings_rows(&frame.config, selected, peers, &frame.scene, frame.editing);
            if frame.privacy.reconnect {
                facts.push("* Stealth: reconnect".to_owned());
            }
            facts.push(timing);
            (
                rows.iter().filter_map(short).collect(),
                String::new(),
                None,
                facts.join("   "),
            )
        }
        (View::Window, _) => {
            let rows = window_rows(&frame.config, selected, frame.editing);
            (
                rows.iter().filter_map(short).collect(),
                String::new(),
                None,
                timing,
            )
        }
        (View::Plugins | View::PluginPage, Some(page)) => {
            let plain = format!("{}: none registered", page.title);
            let second = format!(
                "{} {}/{}   {timing}",
                page.title, page.page_index, page.pages
            );
            (page.rows.clone(), plain, None, second)
        }
        _ => {
            let facts = perf_facts(display.x, display.y, true).into_iter();
            let mut details: Vec<String> = facts
                .map(|(label, value)| format!("{label}:{value}"))
                .collect();
            if peers.show_remote {
                details.push(format!("Ping:{}", peers.ping.opponent));
            }
            (Vec::new(), timing, Some(color), details.join("   "))
        }
    }
}

/// The vertical tables of `view`. Settings ends with the one-line performance
/// footer and the hint line, Window with the hint line (0.6.0); Debug and
/// Performance end with the frame-time block.
unsafe fn draw_tables(
    ui: &UiState,
    view: View,
    frame: &Frame,
    page: Option<&PluginPage>,
    scale: Vec2,
    display: Vec2,
) {
    let peers = &frame.peers;
    let selected = ui.controls.menu.selected;
    match (view, page) {
        (View::Settings, _) => {
            let (rows, colors, _) =
                settings_rows(&frame.config, selected, peers, &frame.scene, frame.editing);
            draw_table(
                b"conn_player_info\0",
                110.0 * scale.x,
                &rows,
                "",
                Some(&colors),
                Some(frame.privacy),
            );
            return footer(&ui.frames, &frame.config, true, frame.privacy.reconnect);
        }
        (View::Window, _) => {
            let rows = window_rows(&frame.config, selected, frame.editing);
            draw_table(b"deluxe_window\0", 110.0 * scale.x, &rows, "", None, None);
            return footer(&ui.frames, &frame.config, false, false);
        }
        (View::Plugins | View::PluginPage, Some(page)) => {
            // Option rows sit two spaces under their group row.
            let indent = if page.total == 0 { "" } else { "  " };
            return draw_table(
                b"deluxe_plugins\0",
                150.0 * scale.x,
                &page.rows,
                indent,
                None,
                None,
            );
        }
        (View::Debug, _) => draw_table(
            b"deluxe_debug\0",
            100.0 * scale.x,
            &debug_rows(peers),
            "",
            None,
            None,
        ),
        _ => {}
    }
    let ping = peers.show_remote.then_some(&peers.ping);
    let graph = frame.config.perf_view == crate::config::PerfView::Graph;
    performance(
        &ui.frames, display.x, display.y, scale.x, scale.y, ping, graph,
    );
}

/// Positions, sizes and titles the visible window, then draws the view's
/// body: the compact strip in a ready match, or the vertical tables.
unsafe fn draw_window(ui: &UiState, view: View, frame: &Frame) {
    let (width, height) = overlay_host::display_size();
    let display = Vec2 {
        x: width,
        y: height,
    };
    let scale = Vec2 {
        x: width / 1920.0,
        y: height / 1080.0,
    };
    let scene = &frame.scene;
    // SetNextWindowPos runs before this frame's window is measured. For a
    // compact strip, use the maximum design width to preserve its left edge.
    let shape = crate::menu::menu_shape(scene.ready, scene.status);
    let info_view = match view {
        View::Debug => crate::menu::InfoView::Debug,
        View::Performance => crate::menu::InfoView::Performance,
        View::Plugins | View::PluginPage => crate::menu::InfoView::Plugins,
        View::Window => crate::menu::InfoView::Window,
        View::Settings | View::Hidden => crate::menu::InfoView::Settings,
    };
    let compact = crate::menu::compact_strip(shape, info_view);
    let configured = crate::menu::pane_width(shape, info_view) * scale.x;
    let x = if compact {
        ((width - configured) * 0.5).max(0.0)
    } else {
        let right = crate::hud_place::on_right(scene.ready, scene.connection, scene.status);
        crate::hud_place::origin_x(
            right,
            width,
            None,
            configured,
            crate::hud_place::margin_for(width),
        )
    };
    let own = (view == View::PluginPage).then_some(ui.controls.page);
    let page = matches!(view, View::Plugins | View::PluginPage).then(|| {
        let size = if compact { STRIP_PAGE } else { TABLE_WINDOW };
        plugin_rows(
            &crate::plugin_api::REGISTRY,
            own,
            ui.controls.plugin_row,
            size,
            compact,
        )
    });
    let strip = compact.then(|| strip_lines(ui, view, frame, page.as_ref(), display));
    igSetNextWindowPos(Vec2 { x, y: 0.0 }, ALWAYS, Vec2::default());
    // Let ImGui fit compact rows to content; `configured` remains the
    // positioning and adaptive-font width bound. Zero height requests the
    // content-fit height: compact views are one or two unwrapped lines.
    let fit = if compact { 0.0 } else { configured };
    igSetNextWindowSize(Vec2 { x: fit, y: 0.0 }, ALWAYS);
    if let Some(alpha) = crate::menu::match_window_alpha(shape) {
        igSetNextWindowBgAlpha(alpha);
    }
    if compact {
        igPushStyleVar_Float(WINDOW_BORDER, 0.0);
        let padding = Vec2 {
            x: 12.0 * scale.x,
            y: 3.0 * scale.y,
        };
        igPushStyleVar_Vec2(WINDOW_PADDING, padding);
    }
    let font_size = strip
        .as_ref()
        .map_or(16.0 * scale.y, |(rows, plain, _, second)| {
            let mut first = if rows.is_empty() {
                plain.clone()
            } else {
                strip_line(rows)
            };
            if view == View::Settings {
                // Conservative text-width allowance for the You label and
                // two one-em icons, including the optional reconnect marker,
                // plus the OC clock while it shows (both modes since 0.9.0).
                let clock = if frame.privacy.oc.is_some() {
                    "[ ] "
                } else {
                    ""
                };
                first = format!(">You: [ ] [ ] {clock}*   {first}");
            }
            crate::menu::compact_font_size(&first, second, configured, scale.x, scale.y)
        });
    igPushFont(std::ptr::null_mut(), font_size);
    // An own page is titled with its plugin's label. The `##` suffix keeps
    // ImGui's window identity stable across plugins with the same label.
    let own_title = page
        .as_ref()
        .filter(|_| own.is_some() && !compact)
        .map(|page| &page.title);
    let page_title = own_title
        .and_then(|title| std::ffi::CString::new(format!("{title}##deluxe_plugin_page")).ok());
    let title: *const std::ffi::c_char = match &page_title {
        Some(title) => title.as_ptr(),
        None if compact => c"##deluxe_match_strip".as_ptr(),
        None if view == View::Window => c"Window Options##deluxe_player_info".as_ptr(),
        None => c"Player Info".as_ptr(),
    };
    let flags = WINDOW_FLAGS | if compact { NO_TITLE_BAR } else { 0 };
    if igBegin(title, std::ptr::null_mut(), flags) {
        match &strip {
            Some(strip) => draw_strip(
                strip,
                (view == View::Settings).then_some((
                    frame.privacy,
                    ui.controls.menu.selected == crate::menu::ROW_YOU,
                )),
            ),
            None => draw_tables(ui, view, frame, page.as_ref(), scale, display),
        }
    }
    igEnd();
    igPopFont();
    if compact {
        igPopStyleVar(2);
    }
}

pub(super) fn install() {
    super::response_probe::init();
    if overlay_host::install(draw) {
        INSTALLED.store(true, std::sync::atomic::Ordering::Release);
    }
}
