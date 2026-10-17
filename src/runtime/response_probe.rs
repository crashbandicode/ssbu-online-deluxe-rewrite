//! Explicit diagnostic pixels tie a guest HID poll to its rendered image.
//! This reports first observation by this callback, not the HID arrival time.
use super::{api_trace, imgui::*, input::Sample, overlay_host, DATA_DIR};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

static ENABLED: AtomicBool = AtomicBool::new(false);
pub(super) fn init() {
    ENABLED.store(
        Path::new(DATA_DIR).join("INPUT_RECEIPT").exists(),
        Ordering::Release,
    );
}

pub(super) unsafe fn draw(sample: Sample) {
    if !ENABLED.load(Ordering::Acquire) {
        return;
    }
    // The callback may run on multiple submissions before one present. Keep
    // a stable identity within that interval; duplicate/missing IDs are visible.
    let serial = super::nvn::present_serial() as u32 & 0x00ff_ffff;
    let down = sample.left_y < -16_000;
    let start = api_trace::start();
    if start != 0 {
        // Both are observations of the same sampled state, with no clock-domain
        // conversion or invented inference about when that state first arrived.
        for (kind, result) in [(6, sample.left_y), (7, sample.update_count as i32)] {
            api_trace::point(start, kind, serial as i32, result);
        }
    }
    let (width, height) = overlay_host::display_size();
    let sx = width / 1920.0;
    let sy = height / 1080.0;
    let origin = Vec2 {
        x: 1100.0 * sx,
        y: 1010.0 * sy,
    };
    igPushStyleVar_Float(WINDOW_BORDER, 0.0);
    igPushStyleVar_Vec2(WINDOW_PADDING, Vec2::default());
    igPushStyleVar_Vec2(WINDOW_MIN_SIZE, Vec2::default());
    igSetNextWindowPos(origin, ALWAYS, Vec2::default());
    igSetNextWindowSize(
        Vec2 {
            x: 680.0 * sx,
            y: 36.0 * sy,
        },
        ALWAYS,
    );
    igSetNextWindowBgAlpha(1.0);
    if igBegin(
        c"##deluxe_response_probe".as_ptr(),
        std::ptr::null_mut(),
        HIDDEN_FLAGS,
    ) {
        let list = igGetWindowDrawList();
        if !list.is_null() {
            // Two colored anchors, 24 serial bits (LSB first), crouch-input bit,
            // then parity over those 25 bits. No rounded edges or text parsing.
            let payload = serial | (u32::from(down) << 24);
            let parity = payload.count_ones() & 1;
            for cell in 0..28 {
                let color = match cell {
                    0 => 0xffff00ff, // magenta
                    1 => 0xffffff00, // cyan
                    27 => {
                        if parity != 0 {
                            0xffffffff
                        } else {
                            0xff000000
                        }
                    }
                    _ => {
                        if payload & (1 << (cell - 2)) != 0 {
                            0xffffffff
                        } else {
                            0xff000000
                        }
                    }
                };
                let x = origin.x + (cell as f32 * 24.0 + 2.0) * sx;
                ImDrawList_AddRectFilled(
                    list,
                    Vec2 {
                        x,
                        y: origin.y + 2.0 * sy,
                    },
                    Vec2 {
                        x: x + 20.0 * sx,
                        y: origin.y + 30.0 * sy,
                    },
                    color,
                    0.0,
                    0,
                );
            }
        }
    }
    igEnd();
    igPopStyleVar(3);
}
