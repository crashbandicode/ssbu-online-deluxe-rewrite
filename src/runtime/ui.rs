//! Fork-style global Player Info window. This window does not write panes.
use super::{imgui::*, input, overlay_host, publish, record, settings, Code};
use crate::menu::CompactField;
use crate::overlay::{FrameTimes, Overlay, View};
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

unsafe fn begin_table(id: &[u8], label_width: f32) -> bool {
    if !igBeginTable(id.as_ptr().cast(), 2, BORDERS, Vec2::default(), 0.0) {
        return false;
    }
    igTableSetupColumn(c"##label".as_ptr(), WIDTH_FIXED, label_width, 0);
    igTableSetupColumn(c"##value".as_ptr(), WIDTH_STRETCH, 1.0, 0);
    true
}

/// Label, local "You", and one opponent column. The opponent name stays empty
/// until a name source exists. Fork columns are not titled.
unsafe fn begin_player_table(id: &[u8], label_width: f32) -> bool {
    if !igBeginTable(id.as_ptr().cast(), 3, BORDERS, Vec2::default(), 0.0) {
        return false;
    }
    igTableSetupColumn(c"##label".as_ptr(), WIDTH_FIXED, label_width, 0);
    igTableSetupColumn(c"##you".as_ptr(), WIDTH_STRETCH, 1.0, 0);
    igTableSetupColumn(c"##peer".as_ptr(), WIDTH_STRETCH, 1.0, 0);
    true
}

unsafe fn row3(label: &str, you: &str, peer: &str) {
    igTableNextRow(0, 0.0);
    igTableNextColumn();
    text(label);
    igTableNextColumn();
    text(you);
    igTableNextColumn();
    text(peer);
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

fn frame_color(mean: Option<f32>) -> Vec4 {
    match mean {
        Some(ms) if ms > 19.17 => Vec4 {
            x: 0.95,
            y: 0.35,
            z: 0.35,
            w: 1.0,
        },
        Some(ms) if ms > 17.0 => Vec4 {
            x: 0.95,
            y: 0.85,
            z: 0.25,
            w: 1.0,
        },
        Some(_) => Vec4 {
            x: 0.45,
            y: 0.90,
            z: 0.45,
            w: 1.0,
        },
        None => Vec4 {
            x: 0.45,
            y: 0.90,
            z: 0.45,
            w: 1.0,
        },
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
        Vec4 {
            x: 0.70,
            y: 0.70,
            z: 0.70,
            w: 1.0,
        }
    };
    (crate::menu::compact_timing_line(mean, latest), color)
}

unsafe fn performance(
    frames: &FrameTimes,
    width: f32,
    height: f32,
    sx: f32,
    sy: f32,
    ping: &crate::pia_rtt::PresentedPing,
) {
    if !begin_table(b"fps_info\0", 100.0 * sx) {
        return;
    }
    if let Some(label) = &ping.performance {
        igTableNextRow(0, 0.0);
        igTableNextColumn();
        text("Ping");
        igTableNextColumn();
        if let Some(color) = ping.color {
            igPushStyleColor_Vec4(
                TEXT,
                Vec4 {
                    x: color.r,
                    y: color.g,
                    z: color.b,
                    w: color.a,
                },
            );
        }
        text(label);
        if ping.color.is_some() {
            igPopStyleColor(1);
        }
    }
    let resolution = crate::resolution::status();
    row(
        "Resolution",
        crate::resolution::applied_label(resolution.applied),
    );
    row("DRS", if resolution.dynamic { "On" } else { "Off" });
    let fighter = super::fighter_main::view();
    row(
        "Triggers",
        if crate::resolution::triggers_available(fighter.callback_armed, fighter.count_only) {
            "Available"
        } else {
            "Unavailable"
        },
    );
    if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
        row("Output", &format!("{width:.0}x{height:.0}"));
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
    igTableNextRow(0, 65.0 * sy);
    igTableNextColumn();
    text("FrameTime");
    igTableNextColumn();
    let label = mean
        .map(|ms| format!("{ms:.2} ms\0"))
        .unwrap_or_else(|| "---\0".into());
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

struct SettingCell {
    label: String,
    local: String,
    remote: String,
    ping: bool,
    compact: Option<CompactField>,
}

struct CompactLines {
    first: String,
    second: String,
    first_color: Option<Vec4>,
    second_color: Option<Vec4>,
}

fn marked(on: bool, label: &str) -> String {
    if on {
        format!("> {label}")
    } else {
        label.to_owned()
    }
}

/// Rows the settings window actually draws. Platform is omitted. FPS++ is
/// omitted unless the configured platform is an emulator.
fn setting_cells(
    config: &crate::config::Config,
    selected: usize,
    peers_connected: bool,
    compact_match: bool,
    show_remote: bool,
    frame: crate::peer_transport::PeerFrame,
    ping_opponent: &str,
) -> Vec<SettingCell> {
    let mut cells = Vec::new();
    let peer_name = if show_remote && frame.peer.known {
        crate::peer_transport::name_label(&frame.peer.name, frame.peer.name_len)
            .unwrap_or("Unavailable")
            .to_owned()
    } else if show_remote {
        "Unavailable".to_owned()
    } else {
        String::new()
    };
    let peer_interface = if show_remote {
        frame.peer.interface.to_owned()
    } else {
        String::new()
    };
    cells.push(SettingCell {
        label: "Name".to_owned(),
        local: "You".to_owned(),
        remote: peer_name.clone(),
        ping: false,
        compact: None,
    });
    cells.push(SettingCell {
        label: "Interface".to_owned(),
        local: super::peer_observer::local_interface_label().to_owned(),
        remote: peer_interface,
        ping: false,
        compact: None,
    });
    if peers_connected && (show_remote || compact_match) {
        cells.push(SettingCell {
            label: marked(selected == crate::menu::ROW_PEER, "Peer"),
            local: if show_remote {
                String::new()
            } else {
                "Hidden".to_owned()
            },
            remote: String::new(),
            ping: false,
            compact: Some(CompactField {
                label: "Peer",
                value: if show_remote {
                    peer_name
                } else {
                    "Hidden".to_owned()
                },
                selected: selected == crate::menu::ROW_PEER,
            }),
        });
    }
    if show_remote {
        cells.push(SettingCell {
            label: "Ping".to_owned(),
            local: "---".to_owned(),
            remote: ping_opponent.to_owned(),
            ping: true,
            compact: None,
        });
    }
    let buffer = if config.input_buffer == -1 {
        "Auto".to_owned()
    } else {
        config.input_buffer.to_string()
    };
    let remote_latency = if show_remote {
        crate::peer_transport::remote_latency_label(frame.peer.latency)
    } else {
        String::new()
    };
    cells.push(SettingCell {
        label: marked(selected == crate::menu::ROW_NET_LATENCY, "NetLatency"),
        local: buffer.clone(),
        remote: remote_latency,
        ping: false,
        compact: Some(CompactField {
            label: "Net",
            value: buffer,
            selected: selected == crate::menu::ROW_NET_LATENCY,
        }),
    });
    let machine = crate::render_policy::load_machine();
    let textures = super::nvn::active_count();
    let applied = super::renderer::status() != "Awaiting window";
    let auto = super::auto_driver::snapshot();
    let profile = crate::fps_boost::append_applied_suffix(
        &crate::auto_policy::profile_row_label(
            config.auto_profile,
            auto.ready,
            config.render_mode,
            auto.request,
        ),
        super::fps_boost::applied(),
    );
    let profile_state = crate::menu::net_profile_status(
        auto.request,
        super::renderer::mode(),
        textures,
        machine,
        applied,
    );
    let peer_settings = if show_remote && frame.peer.known && !frame.peer.settings.is_empty() {
        frame.peer.settings.to_owned()
    } else if show_remote {
        "---".to_owned()
    } else {
        String::new()
    };
    cells.push(SettingCell {
        label: marked(selected == crate::menu::ROW_NET_PROFILE, "NetProfile"),
        local: profile.clone(),
        remote: peer_settings,
        ping: false,
        compact: Some(CompactField {
            label: "Profile",
            value: profile,
            selected: selected == crate::menu::ROW_NET_PROFILE,
        }),
    });
    cells.push(SettingCell {
        label: "Status".to_owned(),
        local: profile_state.to_owned(),
        remote: String::new(),
        ping: false,
        compact: None,
    });
    let fx = crate::menu::fx_row_value(
        config.effects,
        config.effects_scope == crate::config::FxScope::Online && !super::online::ready(),
        crate::eff_variant::availability(),
    )
    .to_owned();
    let compact_fx = if fx == "Reduced (tracker unavailable)" {
        "Reduced!".to_owned()
    } else {
        fx.clone()
    };
    cells.push(SettingCell {
        label: marked(selected == crate::menu::ROW_FX, "FX"),
        local: fx,
        remote: if show_remote {
            "---".to_owned()
        } else {
            String::new()
        },
        ping: false,
        compact: Some(CompactField {
            label: "FX",
            value: compact_fx,
            selected: selected == crate::menu::ROW_FX,
        }),
    });
    let stealth =
        crate::menu::stealth_value(super::stealth(), crate::peer_transport::latched_stealth())
            .to_owned();
    cells.push(SettingCell {
        label: marked(selected == crate::menu::ROW_STEALTH, "Stealth"),
        local: stealth.clone(),
        remote: if show_remote {
            "---".to_owned()
        } else {
            String::new()
        },
        ping: false,
        compact: Some(CompactField {
            label: "Stealth",
            value: stealth,
            selected: selected == crate::menu::ROW_STEALTH,
        }),
    });
    let streamer = if config.streamer { "On" } else { "Off" }.to_owned();
    cells.push(SettingCell {
        label: marked(selected == crate::menu::ROW_STREAMER, "Streamer"),
        local: streamer.clone(),
        remote: if show_remote {
            "---".to_owned()
        } else {
            String::new()
        },
        ping: false,
        compact: Some(CompactField {
            label: "Stream",
            value: streamer,
            selected: selected == crate::menu::ROW_STREAMER,
        }),
    });
    if config.platform == crate::config::Platform::Emulator {
        let fps =
            crate::fps_boost::settings_value(config.fps_boost, super::fps_boost::status_label())
                .to_owned();
        cells.push(SettingCell {
            label: marked(selected == crate::menu::ROW_FPS, "FPS++"),
            local: fps.clone(),
            remote: if show_remote {
                "---".to_owned()
            } else {
                String::new()
            },
            ping: false,
            compact: Some(CompactField {
                label: "FPS++",
                value: fps,
                selected: selected == crate::menu::ROW_FPS,
            }),
        });
    }
    cells
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

fn compact_fields_line(cells: &[SettingCell], peer_only: bool) -> String {
    crate::menu::compact_fields_line(
        cells.iter().filter_map(|cell| cell.compact.as_ref()),
        peer_only,
    )
}

fn compact_settings_lines(
    cells: &[SettingCell],
    show_remote: bool,
    ping: &crate::pia_rtt::PresentedPing,
    frames: &FrameTimes,
) -> CompactLines {
    let (timing, _) = timing_line(frames);
    let mut supporting = Vec::with_capacity(5);
    let peer = compact_fields_line(cells, true);
    if !peer.is_empty() {
        supporting.push(peer);
    }
    if show_remote {
        if let Some(interface) = cells.iter().find(|cell| cell.label == "Interface") {
            if !interface.remote.is_empty() {
                supporting.push(format!("IF:{}", interface.remote));
            }
        }
        supporting.push(format!("Ping:{}", ping.opponent));
        if let Some(latency) = cells.iter().find(|cell| {
            cell.compact
                .as_ref()
                .is_some_and(|field| field.label == "Net")
        }) {
            if !latency.remote.is_empty() && latency.remote != "---" {
                supporting.push(format!("PeerNet:{}", latency.remote));
            }
        }
        if let Some(profile) = cells.iter().find(|cell| {
            cell.compact
                .as_ref()
                .is_some_and(|field| field.label == "Profile")
        }) {
            if !profile.remote.is_empty() && profile.remote != "---" {
                supporting.push(format!("PeerProfile:{}", profile.remote));
            }
        }
    }
    supporting.push(timing);
    CompactLines {
        first: compact_fields_line(cells, false),
        second: supporting.join("   "),
        first_color: None,
        second_color: None,
    }
}

fn compact_performance_lines(
    frames: &FrameTimes,
    width: f32,
    height: f32,
    show_remote: bool,
    ping: &crate::pia_rtt::PresentedPing,
) -> CompactLines {
    let (timing, color) = timing_line(frames);
    let resolution = crate::resolution::status();
    let fighter = super::fighter_main::view();
    let triggers =
        crate::resolution::triggers_available(fighter.callback_armed, fighter.count_only);
    let mut details = vec![
        format!(
            "Res:{}",
            crate::resolution::applied_label(resolution.applied)
        ),
        format!("DRS:{}", if resolution.dynamic { "On" } else { "Off" }),
        format!(
            "Trig:{}",
            if triggers { "Available" } else { "Unavailable" }
        ),
    ];
    if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
        details.push(format!("Out:{width:.0}x{height:.0}"));
    }
    if show_remote {
        details.push(format!("Ping:{}", ping.opponent));
    }
    CompactLines {
        first: timing,
        second: details.join("   "),
        first_color: Some(color),
        second_color: None,
    }
}

/// Compact Plugins page: one page of `Label:Value` fields containing the
/// selection, then the page indicator and the same timing line Settings uses.
fn compact_plugin_lines(
    registry: &crate::plugin_api::Registry,
    selected: usize,
    frames: &FrameTimes,
) -> CompactLines {
    use crate::plugin_api::{compact_label, compact_line, compact_page, plugin_rows, COMPACT_PAGE};
    let plugins = registry.snapshot();
    let rows = plugin_rows(&plugins);
    let (start, end, page, pages) = compact_page(selected, rows.len(), COMPACT_PAGE);
    let fields: Vec<_> = rows[start..end]
        .iter()
        .enumerate()
        .map(|(offset, row)| {
            let plugin = &plugins[row.plugin];
            let option = &plugin.options[row.option];
            (
                compact_label(&plugin.label, &option.label, plugin.options.len()),
                registry.value(row.plugin, plugin, row.option),
                start + offset == selected,
            )
        })
        .collect();
    let (timing, _) = timing_line(frames);
    let first = if fields.is_empty() {
        "Plugins: none registered".to_owned()
    } else {
        compact_line(&fields)
    };
    CompactLines {
        first,
        second: format!("Plugins {page}/{pages}   {timing}"),
        first_color: None,
        second_color: None,
    }
}

/// Vertical Plugins page: a header row per plugin, then its option rows. Long
/// lists are clipped to a window around the selection with `...` markers.
unsafe fn draw_plugins_vertical(registry: &crate::plugin_api::Registry, selected: usize, sx: f32) {
    use crate::plugin_api::{plugin_rows, vertical_window, VERTICAL_WINDOW};
    if !begin_table(b"deluxe_plugins\0", 150.0 * sx) {
        return;
    }
    let plugins = registry.snapshot();
    let rows = plugin_rows(&plugins);
    if rows.is_empty() {
        row("Plugins", "none registered");
        igEndTable();
        return;
    }
    let (start, end) = vertical_window(selected, rows.len(), VERTICAL_WINDOW);
    if start > 0 {
        row("...", "");
    }
    let mut previous_plugin = usize::MAX;
    for (index, entry) in rows.iter().enumerate().take(end).skip(start) {
        let plugin = &plugins[entry.plugin];
        if entry.plugin != previous_plugin {
            row(&plugin.label, "");
            previous_plugin = entry.plugin;
        }
        let option = &plugin.options[entry.option];
        let value = registry.value(entry.plugin, plugin, entry.option);
        let value_text = if value.pending {
            format!("{} (pending)", value.label)
        } else {
            value.label
        };
        row(
            &marked(index == selected, &format!("  {}", option.label)),
            &value_text,
        );
    }
    if end < rows.len() {
        row("...", "");
    }
    igEndTable();
}

unsafe fn draw_compact_lines(lines: &CompactLines) {
    igPushTextWrapPos(-1.0);
    paint_text(&lines.first, lines.first_color);
    if !lines.second.is_empty() {
        paint_text(&lines.second, lines.second_color);
    }
    igPopTextWrapPos();
}

unsafe fn draw_settings_vertical(
    cells: &[SettingCell],
    remote: bool,
    sx: f32,
    ping_color: Option<Vec4>,
) {
    let began = if remote {
        begin_player_table(b"conn_player_info\0", 110.0 * sx)
    } else {
        begin_table(b"conn_player_info\0", 110.0 * sx)
    };
    if !began {
        return;
    }
    for cell in cells {
        if remote {
            if cell.ping {
                igTableNextRow(0, 0.0);
                igTableNextColumn();
                text(&cell.label);
                igTableNextColumn();
                text(&cell.local);
                igTableNextColumn();
                paint_text(&cell.remote, ping_color);
            } else {
                row3(&cell.label, &cell.local, &cell.remote);
            }
        } else {
            row(&cell.label, &cell.local);
        }
    }
    igEndTable();
}

unsafe extern "C" fn draw() {
    let sample = input::read();
    let buttons = sample.buttons;
    super::timing::observe(buttons);
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
        record(
            Code::MenuChanged,
            config.effects as u64,
            config.input_buffer as i64 as u64,
        );
    }
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
    super::timing::set_view(view as u8);
    let pressed = buttons & !ui.previous_timing_buttons;
    ui.previous_timing_buttons = buttons;
    let shoulders = crate::overlay::L | crate::overlay::R;
    if view == View::Debug && buttons & shoulders == shoulders && pressed & crate::overlay::UP != 0
    {
        super::timing::request();
    }
    if view != previous_view {
        record(Code::OverlayView, previous_view as u64, view as u64);
    }
    if view == View::Hidden {
        hidden_window();
    } else {
        let (width, height) = overlay_host::display_size();
        let sx = width / 1920.0;
        let sy = height / 1080.0;
        let fps_visible = config.platform == crate::config::Platform::Emulator;
        // SetNextWindowPos runs before this frame's window is measured. For a
        // compact strip, use the maximum design width to preserve its left edge.
        let scene = super::auto_driver::snapshot();
        let shape = crate::menu::menu_shape(scene.ready, scene.status);
        let info_view = match view {
            View::Debug => crate::menu::InfoView::Debug,
            View::Performance => crate::menu::InfoView::Performance,
            View::Plugins => crate::menu::InfoView::Plugins,
            View::Settings | View::Hidden => crate::menu::InfoView::Settings,
        };
        let compact_strip = crate::menu::compact_strip(shape, info_view);
        let window_width = crate::menu::pane_width(shape, info_view);
        let configured = window_width * sx;
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
        let cells = (view == View::Settings).then(|| {
            setting_cells(
                &config,
                ui.controls.menu.selected,
                connected,
                compact_strip,
                show_remote,
                frame,
                &ping.opponent,
            )
        });
        let compact_lines = if compact_strip {
            match view {
                View::Settings => cells
                    .as_deref()
                    .map(|cells| compact_settings_lines(cells, show_remote, &ping, &ui.frames)),
                View::Performance => Some(compact_performance_lines(
                    &ui.frames,
                    width,
                    height,
                    show_remote,
                    &ping,
                )),
                View::Plugins => Some(compact_plugin_lines(
                    &crate::plugin_api::REGISTRY,
                    ui.controls.plugin_row,
                    &ui.frames,
                )),
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
        let title = if compact_strip {
            c"##deluxe_match_strip"
        } else {
            c"Player Info"
        };
        let flags = if compact_strip {
            WINDOW_FLAGS | NO_TITLE_BAR
        } else {
            WINDOW_FLAGS
        };
        if igBegin(title.as_ptr(), std::ptr::null_mut(), flags) {
            if view == View::Settings {
                let ping_color = ping.color.map(|color| Vec4 {
                    x: color.r,
                    y: color.g,
                    z: color.b,
                    w: color.a,
                });
                if !compact_strip {
                    if let Some(cells) = cells.as_deref() {
                        draw_settings_vertical(cells, show_remote, sx, ping_color);
                    }
                }
            }
            if view == View::Plugins && !compact_strip {
                draw_plugins_vertical(&crate::plugin_api::REGISTRY, ui.controls.plugin_row, sx);
            }
            if view == View::Debug && begin_table(b"deluxe_debug\0", 100.0 * sx) {
                row(
                    "Build",
                    &crate::build_info::GIT_COMMIT[..crate::build_info::GIT_COMMIT.len().min(12)],
                );
                row("Renderer", super::renderer::mode().label());
                row("FX", config.effects.label());
                row("FX scope", config.effects_scope.label());
                row(
                    "Online tracker",
                    if super::online::ready() {
                        "Ready"
                    } else {
                        "Unavailable"
                    },
                );
                row(
                    "Reduced FX",
                    match config.effects {
                        crate::config::FxMode::Reduced
                            if config.effects_scope == crate::config::FxScope::Online
                                && !super::online::ready() =>
                        {
                            "Saved (tracker unavailable)"
                        }
                        crate::config::FxMode::Reduced => "Selected",
                        _ => "Not selected",
                    },
                );
                let tracker_blocked = config.effects_scope == crate::config::FxScope::Online
                    && !super::online::ready();
                row(
                    "Palutena FX",
                    crate::menu::fx_file_debug(
                        config.effects,
                        tracker_blocked,
                        crate::eff_variant::availability(),
                        "sibling ready",
                    ),
                );
                row(
                    "Edge FX",
                    crate::menu::fx_file_debug(
                        config.effects,
                        tracker_blocked,
                        crate::eff_variant::edge_availability(),
                        "sibling ready",
                    ),
                );
                row(
                    "Ink FX",
                    crate::menu::fx_file_debug(
                        config.effects,
                        tracker_blocked,
                        crate::eff_variant::ink_availability(),
                        "sibling ready",
                    ),
                );
                row(
                    "Chrom FX",
                    crate::menu::fx_file_debug(
                        config.effects,
                        tracker_blocked,
                        crate::eff_variant::chrom_availability(),
                        "life 3 to 2 candidate",
                    ),
                );
                row("Engine", super::renderer::status());
                if fps_visible {
                    row("FPS++", super::fps_boost::status_label());
                }
                row("Timing", super::timing::status());
                if show_remote {
                    row("Ping detail", ping.detail);
                }
                row("Capture", "L+R+Up (10 s)");
                igEndTable();
            }
            if let Some(lines) = compact_lines.as_ref() {
                draw_compact_lines(lines);
            } else if view != View::Plugins {
                let quiet_ping = crate::pia_rtt::PresentedPing {
                    opponent: String::new(),
                    performance: None,
                    color: None,
                    detail: "",
                };
                performance(
                    &ui.frames,
                    width,
                    height,
                    sx,
                    sy,
                    if show_remote { &ping } else { &quiet_ping },
                );
            }
        }
        igEnd();
        igPopFont();
        if compact_strip {
            igPopStyleVar(2);
        }
    }
    if sampled {
        record(Code::OverlayExited, ui.count, view as u64);
    }
}

pub(super) fn install() {
    super::response_probe::init();
    if overlay_host::install(draw) {
        INSTALLED.store(true, std::sync::atomic::Ordering::Release);
    }
}
