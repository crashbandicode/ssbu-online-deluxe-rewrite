//! Explicit, startup-only evidence collection for the renderer reconstruction.
//! Candidate offsets are read, never patched or called. Dumps stay outside Git.
use super::{logging, memory, DATA_DIR};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

// Candidate 13.0.5 ranges inferred from the preserved host's +0x5b0 constants.
// They are not hook signatures or validated function addresses.
const RANGES: &[(&str, usize, usize)] = &[
    // Oct 5 console crash PC and first three callers; read-only comparison.
    ("switch_crash_pc", 0x3ce000, 0x1200),
    ("switch_crash_caller1", 0x15f6600, 0x1200),
    ("switch_crash_caller2", 0x15b2600, 0x800),
    ("switch_crash_caller3", 0x15dbc00, 0x800),
    ("switch_crash_object", 0x4e4800, 0x200),
    ("switch_crash_update_tree", 0x22de900, 0x800),
    ("switch_crash_frame_update", 0x3725000, 0x400),
    ("main_loop", 0x374_b500, 0x2800),
    ("present_callsite", 0x374_8000, 0x300),
    ("swap_flush_callsite", 0x374_9a00, 0x300),
    ("swapchain", 0x384_f800, 0x1800),
    ("framebuffer_index", 0x386_af00, 0x600),
    ("gfx_present_acquire", 0x387_0000, 0x800),
    ("window_init", 0x386_0400, 0xc00),
    ("task_workers", 0x354_8700, 0x4a00),
    ("update_models_candidate", 0x003a_bb00, 0x500),
    ("update_models_original", 0x003a_b500, 0x500),
    ("model_vtable", 0x4f6_2900, 0x200),
    ("vi_layer_init", 0x374_4000, 0x500),
];

const SYMBOLS: &[(&str, &[u8])] = &[
    (
        "vi_native_window",
        b"_ZN2nn2vi15GetNativeWindowEPPvPNS0_5LayerE\0",
    ),
    (
        "vi_frame_history",
        b"_ZN2nn2vi16ListFrameHistoryEPNS0_9FrameInfoEiPKNS0_5LayerE\0",
    ),
    (
        "vi_latest_frame",
        b"_ZN2nn2vi20GetLatestFrameNumberEPmPKNS0_5LayerE\0",
    ),
    (
        "vi_display_open",
        b"_ZN2nn2vi18OpenDefaultDisplayEPPNS0_7DisplayE\0",
    ),
    (
        "system_tick_frequency",
        b"_ZN2nn2os22GetSystemTickFrequencyEv\0",
    ),
];

fn dump() -> io::Result<()> {
    let root = Path::new(DATA_DIR).join("render-probe");
    fs::create_dir_all(&root)?;
    let mut report = File::create(root.join("addresses.txt"))?;
    let base = unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as usize };
    writeln!(
        report,
        "build={}\ngame=13.0.5\ntext_base=0x{base:x}\nread_only=true",
        crate::build_info::BUILD_ID
    )?;
    for &(name, offset, len) in RANGES {
        let address = base
            .checked_add(offset)
            .ok_or_else(|| io::Error::other("address overflow"))?;
        let readable = unsafe { memory::accessible(address, len, false) };
        writeln!(report, "range {name} offset=0x{offset:x} address=0x{address:x} size=0x{len:x} readable={readable}")?;
        if readable {
            let bytes = unsafe { std::slice::from_raw_parts(address as *const u8, len) };
            let mut file = File::create(root.join(format!("{name}.bin")))?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
    }
    for &(name, symbol) in SYMBOLS {
        let mut address = 0usize;
        unsafe {
            skyline::nn::ro::LookupSymbol(&mut address, symbol.as_ptr());
        }
        let readable = address != 0 && unsafe { memory::accessible(address, 0x500, false) };
        writeln!(
            report,
            "symbol {name} address=0x{address:x} readable={readable}"
        )?;
        if readable {
            let bytes = unsafe { std::slice::from_raw_parts(address as *const u8, 0x500) };
            let mut file = File::create(root.join(format!("{name}.bin")))?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
    }
    report.sync_all()
}

pub(super) fn dump_if_requested() {
    if !Path::new(DATA_DIR).join("DUMP_RENDER_PROBE").exists() {
        return;
    }
    logging::startup_phase("render_probe_begin");
    match dump() {
        Ok(()) => logging::startup_phase("render_probe_complete"),
        Err(error) => {
            logging::startup_phase("render_probe_failed");
            skyline::println!("[Deluxe] render probe failed: {}", error);
        }
    }
}
