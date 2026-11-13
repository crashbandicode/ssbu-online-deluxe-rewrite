//! The overlay backend's counters for `renderer-status.txt`: from this NRO's
//! embedded archive, or, when Deluxe draws through a standalone
//! `libimgui_smash.nro` that exports them (imgui-smash 1.2.0), from that host.
//! A standalone without the getters is labeled and not read.
use crate::imgui_host::{
    font_dynamic_line, font_status_lines, host_clients_line, renderer_status_lines, DrawStats,
    FontStats, FrameGuardStats, HostClients, StatsHost,
};
use std::io::Write;

extern "C" {
    fn imgui_smash_draw_stats(out: *mut DrawStats) -> i32;
    fn imgui_smash_frame_guard_stats(out: *mut FrameGuardStats) -> i32;
    fn imgui_smash_font_stats(out: *mut FontStats) -> i32;
    fn imgui_smash_font_dynamic_set(out: *mut u32, cap: u32) -> i32;
}

type DrawGetter = unsafe extern "C" fn(*mut DrawStats) -> i32;
type GuardGetter = unsafe extern "C" fn(*mut FrameGuardStats) -> i32;
type FontGetter = unsafe extern "C" fn(*mut FontStats) -> i32;
type DynamicGetter = unsafe extern "C" fn(*mut u32, u32) -> i32;
type ClientsGetter = unsafe extern "C" fn(*mut HostClients) -> i32;

struct Getters {
    host: StatsHost,
    draw: DrawGetter,
    guard: GuardGetter,
    font: FontGetter,
    dynamic: DynamicGetter,
    clients: Option<ClientsGetter>,
}

fn getters() -> Option<Getters> {
    if super::overlay_host::embedded() {
        return Some(Getters {
            host: StatsHost::Embedded,
            draw: imgui_smash_draw_stats,
            guard: imgui_smash_frame_guard_stats,
            font: imgui_smash_font_stats,
            dynamic: imgui_smash_font_dynamic_set,
            clients: None,
        });
    }
    let external = super::overlay_host::external_stats()?;
    // Addresses bound to the standalone's own text by overlay_host; the
    // layouts are the archive's, version-checked by the line formatters.
    unsafe {
        Some(Getters {
            host: StatsHost::External,
            draw: std::mem::transmute::<usize, DrawGetter>(external.draw),
            guard: std::mem::transmute::<usize, GuardGetter>(external.guard),
            font: std::mem::transmute::<usize, FontGetter>(external.font),
            dynamic: std::mem::transmute::<usize, DynamicGetter>(external.dynamic),
            clients: external
                .clients
                .map(|address| std::mem::transmute::<usize, ClientsGetter>(address)),
        })
    }
}

pub(super) fn write_status(out: &mut impl Write) {
    let Some(getters) = getters() else {
        let _ = writeln!(out, "imgui_stats=not_embedded");
        return;
    };
    let mut draw = DrawStats::default();
    let mut guard = FrameGuardStats::default();
    unsafe {
        let _ = (getters.draw)(&mut draw);
        let _ = (getters.guard)(&mut guard);
    }
    let _ = writeln!(out, "{}", renderer_status_lines(getters.host, &draw, &guard));
    let mut fonts = FontStats::default();
    let wrote = unsafe { (getters.font)(&mut fonts) } != 0;
    let _ = writeln!(out, "{}", font_status_lines(&fonts, wrote));
    let mut set = [0u32; 64];
    let total = unsafe { (getters.dynamic)(set.as_mut_ptr(), set.len() as u32) };
    let shown = (total.max(0) as usize).min(set.len());
    let _ = writeln!(out, "{}", font_dynamic_line(&set[..shown], total));
    if let Some(clients) = getters.clients {
        let mut record = HostClients::default();
        let wrote = unsafe { clients(&mut record) } != 0;
        let _ = writeln!(out, "{}", host_clients_line(&record, wrote));
    }
}
