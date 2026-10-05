//! Minimal C ABI from the pinned imgui-smash cimgui.h (see IMGUI_DEPENDENCY.md).
//! Opaque C++ objects remain owned by the embedded backend.
#![allow(non_snake_case)]
use core::ffi::{c_char, c_void};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(super) struct Vec2 {
    pub x: f32,
    pub y: f32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

pub(super) const WINDOW_FLAGS: i32 =
    (1 << 1) | (1 << 2) | (1 << 5) | (1 << 8) | (1 << 9) | (1 << 12) | (1 << 16) | (1 << 17);
pub(super) const HIDDEN_FLAGS: i32 = WINDOW_FLAGS | (1 << 0) | (1 << 3) | (1 << 7) | (1 << 13);
pub(super) const BORDERS: i32 = (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10);
pub(super) const WIDTH_FIXED: i32 = 1 << 4;
pub(super) const WIDTH_STRETCH: i32 = 1 << 3;
pub(super) const ALWAYS: i32 = 1;
pub(super) const TEXT: i32 = 0;
pub(super) const PLOT_LINES: i32 = 41;
pub(super) const PLOT_LINES_HOVERED: i32 = 42;
pub(super) const WINDOW_PADDING: i32 = 2;
pub(super) const WINDOW_BORDER: i32 = 4;
pub(super) const WINDOW_MIN_SIZE: i32 = 5;

extern "C" {
    pub(super) fn igBegin(name: *const c_char, open: *mut bool, flags: i32) -> bool;
    pub(super) fn igEnd();
    pub(super) fn igSetNextWindowPos(pos: Vec2, cond: i32, pivot: Vec2);
    pub(super) fn igSetNextWindowSize(size: Vec2, cond: i32);
    pub(super) fn igSetNextWindowBgAlpha(alpha: f32);
    pub(super) fn igPushFont(font: *mut c_void, size: f32);
    pub(super) fn igPopFont();
    pub(super) fn igPushStyleVar_Float(index: i32, value: f32);
    pub(super) fn igPushStyleVar_Vec2(index: i32, value: Vec2);
    pub(super) fn igPopStyleVar(count: i32);
    pub(super) fn igPushStyleColor_Vec4(index: i32, value: Vec4);
    pub(super) fn igPopStyleColor(count: i32);
    pub(super) fn igTextUnformatted(text: *const c_char, end: *const c_char);
    pub(super) fn igBeginTable(
        id: *const c_char,
        columns: i32,
        flags: i32,
        size: Vec2,
        inner_width: f32,
    ) -> bool;
    pub(super) fn igEndTable();
    pub(super) fn igTableSetupColumn(label: *const c_char, flags: i32, width: f32, user_id: u32);
    pub(super) fn igTableNextRow(flags: i32, height: f32);
    pub(super) fn igTableNextColumn() -> bool;
    pub(super) fn igGetContentRegionAvail() -> Vec2;
    pub(super) fn igGetWindowDrawList() -> *mut c_void;
    pub(super) fn igGetWindowPos() -> Vec2;
    pub(super) fn ImDrawList_AddRectFilled(
        list: *mut c_void,
        min: Vec2,
        max: Vec2,
        color: u32,
        rounding: f32,
        flags: i32,
    );
    pub(super) fn igPlotLines_FloatPtr(
        label: *const c_char,
        values: *const f32,
        count: i32,
        offset: i32,
        overlay: *const c_char,
        min: f32,
        max: f32,
        size: Vec2,
        stride: i32,
    );
}
