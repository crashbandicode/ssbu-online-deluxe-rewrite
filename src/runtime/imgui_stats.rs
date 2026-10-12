//! Embedded archive counters from the pinned C++ backend.
//! A host that is not embedded is labeled and is not read through this copy.
use std::io::Write;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct DrawStats {
    #[allow(dead_code)]
    version: u32,
    #[allow(dead_code)]
    size: u32,
    submitted: u64,
    reused: u64,
    pending_skips: u64,
    failed: u64,
    max_command_bytes: u64,
    max_control_bytes: u64,
    geometry_skips: u64,
    command_overflow_skips: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct FrameGuardStats {
    #[allow(dead_code)]
    version: u32,
    #[allow(dead_code)]
    size: u32,
    accepted: u64,
    busy_skips: u64,
}

const _: () = assert!(std::mem::size_of::<DrawStats>() == 72);
const _: () = assert!(std::mem::size_of::<FrameGuardStats>() == 24);

extern "C" {
    fn imgui_smash_draw_stats(out: *mut DrawStats) -> i32;
    fn imgui_smash_frame_guard_stats(out: *mut FrameGuardStats) -> i32;
}

pub(super) fn write_status(out: &mut impl Write) {
    if !super::overlay_host::embedded() {
        let _ = writeln!(out, "imgui_stats=not_embedded");
        return;
    }
    let mut draw = DrawStats::default();
    let mut guard = FrameGuardStats::default();
    unsafe {
        let _ = imgui_smash_draw_stats(&mut draw);
        let _ = imgui_smash_frame_guard_stats(&mut guard);
    }
    let _ = writeln!(
        out,
        "imgui_stats=embedded\n\
         imgui_frame_accepted={}\n\
         imgui_frame_busy={}\n\
         imgui_draw_submitted={}\n\
         imgui_draw_reused={}\n\
         imgui_draw_pending={}\n\
         imgui_draw_failed={}\n\
         imgui_draw_geometry={}\n\
         imgui_draw_overflow={}\n\
         imgui_draw_max_command={}\n\
         imgui_draw_max_control={}",
        guard.accepted,
        guard.busy_skips,
        draw.submitted,
        draw.reused,
        draw.pending_skips,
        draw.failed,
        draw.geometry_skips,
        draw.command_overflow_skips,
        draw.max_command_bytes,
        draw.max_control_bytes,
    );
}
