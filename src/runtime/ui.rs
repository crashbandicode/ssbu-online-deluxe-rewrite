//! Fork-style global Player Info window. No game-owned pane is modified.
use super::{imgui::*, input, overlay_host, publish, record, settings, Code};
use crate::overlay::{FrameTimes, Overlay, View};
use std::{sync::Mutex, time::Instant};

struct UiState {
    controls: Overlay,
    frames: FrameTimes,
    previous_frame: Option<Instant>,
    count: u64,
}
static UI: Mutex<Option<UiState>> = Mutex::new(None);

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

unsafe fn performance(frames: &FrameTimes, width: f32, height: f32, sx: f32, sy: f32) {
    if !begin_table(b"fps_info\0", 100.0 * sx) {
        return;
    }
    row("Resolution", &format!("{width:.0}x{height:.0}"));
    let mean = frames.average();
    let color = match mean {
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
        _ => Vec4 {
            x: 0.45,
            y: 0.90,
            z: 0.45,
            w: 1.0,
        },
    };
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

unsafe extern "C" fn draw() {
    let Ok(mut guard) = UI.try_lock() else {
        return;
    };
    let ui = guard.get_or_insert_with(|| UiState {
        controls: Overlay::default(),
        frames: FrameTimes::default(),
        previous_frame: None,
        count: 0,
    });
    let sampled = ui.count.is_multiple_of(300);
    if sampled {
        record(Code::OverlayEntered, ui.count, ui.controls.view as u64);
    }
    ui.count += 1;
    let now = Instant::now();
    if let Some(previous) = ui.previous_frame.replace(now) {
        ui.frames
            .push(now.duration_since(previous).as_secs_f32() * 1000.0);
    }
    let mut config = settings();
    let previous_view = ui.controls.view;
    if ui.controls.poll(input::buttons(), &mut config) {
        publish(&config);
        record(
            Code::MenuChanged,
            config.effects as u64,
            config.input_buffer as i64 as u64,
        );
    }
    let view = ui.controls.view;
    if view != previous_view {
        record(Code::OverlayView, previous_view as u64, view as u64);
    }
    if view == View::Hidden {
        hidden_window();
    } else {
        let (width, height) = overlay_host::display_size();
        let sx = width / 1920.0;
        let sy = height / 1080.0;
        let window_height = match view {
            View::Performance => 180.0,
            View::Debug => 300.0,
            _ => 280.0,
        };
        igSetNextWindowPos(Vec2::default(), ALWAYS, Vec2::default());
        igSetNextWindowSize(
            Vec2 {
                x: 300.0 * sx,
                y: window_height * sy,
            },
            ALWAYS,
        );
        igPushFont(std::ptr::null_mut(), 16.0 * sy);
        if igBegin(c"Player Info".as_ptr(), std::ptr::null_mut(), WINDOW_FLAGS) {
            if view == View::Settings && begin_table(b"conn_player_info\0", 100.0 * sx) {
                row("Name", "You");
                row("Ping", "---"); // Network telemetry has not been ported yet.
                let buffer = if config.input_buffer == -1 {
                    "Auto".into()
                } else {
                    config.input_buffer.to_string()
                };
                row(
                    if ui.controls.menu.selected == 1 {
                        "> NetLatency"
                    } else {
                        "NetLatency"
                    },
                    &buffer,
                );
                row("NetProfile", "Vanilla");
                row(
                    if ui.controls.menu.selected == 0 {
                        "> FX"
                    } else {
                        "FX"
                    },
                    if config.effects == crate::config::FxMode::Off
                        && config.effects_scope == crate::config::FxScope::Online
                        && !super::online::ready()
                    {
                        "On (tracker unavailable)"
                    } else {
                        config.effects.effective().label()
                    },
                );
                igEndTable();
            }
            if view == View::Debug && begin_table(b"deluxe_debug\0", 100.0 * sx) {
                row(
                    "Build",
                    &crate::build_info::GIT_COMMIT[..crate::build_info::GIT_COMMIT.len().min(12)],
                );
                row("Renderer", "Vanilla");
                row("FX", config.effects.effective().label());
                row("FX scope", config.effects_scope.label());
                row(
                    "Online tracker",
                    if super::online::ready() {
                        "Ready"
                    } else {
                        "Unavailable"
                    },
                );
                row("Reduced FX", "Unavailable");
                row("LessLag", "Unavailable");
                igEndTable();
            }
            performance(&ui.frames, width, height, sx, sy);
        }
        igEnd();
        igPopFont();
    }
    if sampled {
        record(Code::OverlayExited, ui.count, view as u64);
    }
}

pub(super) fn install() {
    overlay_host::install(draw);
}
