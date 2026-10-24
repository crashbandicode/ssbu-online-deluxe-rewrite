//! Passive plugin status cells (`OPTION_FLAG_CELL`): a heading row with a
//! check or cross centered beneath each heading. The marks are drawn as
//! lines, like the You-line icons, because the window's font coverage is the
//! ImGui host's and not ours to assume. Nothing here reads a file, and the
//! callers pass in what to draw, so a frame asks the plugin once per cell.
use super::imgui::*;

/// One column: its heading, whether the plugin reported it ready (a check;
/// anything else is a cross) and the color of the mark (`None` is text color).
pub(super) struct CellDraw<'a> {
    pub label: &'a str,
    pub ok: bool,
    pub color: Option<Vec4>,
}

unsafe fn text_width(value: &str) -> f32 {
    let end = value.as_ptr().add(value.len());
    igCalcTextSize(value.as_ptr().cast(), end.cast(), false, -1.0).x
}

unsafe fn centered_text(value: &str) {
    let room = igGetContentRegionAvail().x - text_width(value);
    igSetCursorPosX(igGetCursorPosX() + (room * 0.5).max(0.0));
    let end = value.as_ptr().add(value.len());
    igTextUnformatted(value.as_ptr().cast(), end.cast());
}

/// A check (`ok`) or a cross, one text line high and wide. `center` places it
/// in the middle of the remaining width of a table cell.
pub(super) unsafe fn mark(ok: bool, color: Option<Vec4>, center: bool) {
    let size = igGetFontSize();
    if center {
        let room = igGetContentRegionAvail().x - size;
        igSetCursorPosX(igGetCursorPosX() + (room * 0.5).max(0.0));
    }
    let origin = igGetCursorScreenPos();
    let list = igGetWindowDrawList();
    igDummy(Vec2 { x: size, y: size });
    if list.is_null() {
        return;
    }
    let shade = match color {
        Some(color) => igGetColorU32_Vec4(color),
        None => igGetColorU32_Col(TEXT, 1.0),
    };
    let point = |x: f32, y: f32| Vec2 {
        x: origin.x + x * size,
        y: origin.y + y * size,
    };
    let stroke = (size * 0.12).max(1.0);
    if ok {
        ImDrawList_AddLine(list, point(0.14, 0.54), point(0.40, 0.80), shade, stroke);
        ImDrawList_AddLine(list, point(0.40, 0.80), point(0.86, 0.22), shade, stroke);
    } else {
        ImDrawList_AddLine(list, point(0.20, 0.22), point(0.80, 0.78), shade, stroke);
        ImDrawList_AddLine(list, point(0.80, 0.22), point(0.20, 0.78), shade, stroke);
    }
}

/// One table of `cells`: headings in the first row with vertical bars between
/// the columns, a centered mark under each in the second. `id` is
/// NUL-terminated and unique within the window.
pub(super) unsafe fn table(id: &[u8], cells: &[CellDraw]) {
    if cells.is_empty()
        || !igBeginTable(
            id.as_ptr().cast(),
            cells.len() as i32,
            BORDERS,
            Vec2::default(),
            0.0,
        )
    {
        return;
    }
    for _ in cells {
        igTableSetupColumn(c"##cell".as_ptr(), WIDTH_STRETCH, 1.0, 0);
    }
    igTableNextRow(0, 0.0);
    for cell in cells {
        igTableNextColumn();
        centered_text(cell.label);
    }
    igTableNextRow(0, 0.0);
    for cell in cells {
        igTableNextColumn();
        mark(cell.ok, cell.color, true);
    }
    igEndTable();
}
