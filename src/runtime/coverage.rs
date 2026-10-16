//! Coverage build only (`tools/Build.ps1 -Coverage`, feature `coverage`).
//!
//! The plugin crate is compiled with `-C instrument-coverage`, so every
//! function increments counters in the `__llvm_prf_cnts` section. This module
//! is the whole "profiler runtime": it defines the hook symbol the
//! instrumented objects reference and, on the diagnostics thread, serialises
//! the counter, data and name sections into `coverage/*.profraw` under the
//! data directory with [`crate::coverage_profile`]. Counters are cumulative
//! for the life of the process, so every file is a superset of the previous
//! one: `live.profraw` is rewritten every ten seconds (a guest abort loses at
//! most that much), a `scene-NNN-P-S.profraw` snapshot is taken at every scene
//! status change (`P` previous, `S` new status byte) so consecutive snapshots
//! can be differenced per scene, and the Debug view's L+R+Left writes a
//! `manual-NNN.profraw`. Hooks only set atomics; no file work on a game thread.
use super::{record, Code, DATA_DIR};
use crate::{
    build_info,
    coverage_profile::{write_profraw, Sections},
};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    ptr::addr_of,
    sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering},
};

#[allow(non_upper_case_globals)]
extern "C" {
    // Bounds of the instrumentation sections. ld.lld synthesises
    // `__start_`/`__stop_` symbols for output sections whose names are C
    // identifiers, which these are; the Skyline linker script leaves them as
    // their own output sections inside the read-write and read-only segments.
    static __start___llvm_prf_cnts: u8;
    static __stop___llvm_prf_cnts: u8;
    static __start___llvm_prf_data: u8;
    static __stop___llvm_prf_data: u8;
    static __start___llvm_prf_names: u8;
    static __stop___llvm_prf_names: u8;
}

/// Every instrumented object references this symbol to pull the profiler
/// runtime into the link. There is no runtime in this build, only the writer
/// below, so the symbol is defined here with no other use.
#[no_mangle]
#[allow(non_upper_case_globals)]
pub static __llvm_profile_runtime: i32 = 0;

const DIR: &str = "coverage";
/// Diagnostics-thread iterations (500 ms each) between `live.profraw` rewrites.
const LIVE_EVERY: u64 = 20;
const REASON_LIVE: u8 = 1;
const REASON_SCENE: u8 = 2;
const REASON_MANUAL: u8 = 3;
const SCENE_BIT: u8 = 1;
const MANUAL_BIT: u8 = 2;

static REQUESTS: AtomicU8 = AtomicU8::new(0);
/// Previous status in bits 0..7, new status in bits 8..15 of the last edge.
static SCENE_EDGE: AtomicU32 = AtomicU32::new(0);
static SEQUENCE: AtomicU32 = AtomicU32::new(0);
static DESCRIBED: AtomicBool = AtomicBool::new(false);
static FAILURE_REPORTED: AtomicBool = AtomicBool::new(false);

/// Scene status changed (game thread, from the scene funnel). The snapshot is
/// written at the next diagnostics poll; a second change before that poll
/// replaces the edge, so the file name carries the latest edge.
pub(super) fn note_scene(previous: u8, status: u8) {
    SCENE_EDGE.store(
        u32::from(previous) | (u32::from(status) << 8),
        Ordering::Release,
    );
    REQUESTS.fetch_or(SCENE_BIT, Ordering::AcqRel);
}

/// Debug view L+R+Left (overlay thread).
pub(super) fn request_manual() {
    REQUESTS.fetch_or(MANUAL_BIT, Ordering::AcqRel);
}

/// Diagnostics thread, every 500 ms.
pub(super) fn poll(iteration: u64) {
    let requests = REQUESTS.swap(0, Ordering::AcqRel);
    if requests & SCENE_BIT != 0 {
        dump(REASON_SCENE, SCENE_EDGE.load(Ordering::Acquire));
    }
    if requests & MANUAL_BIT != 0 {
        dump(REASON_MANUAL, 0);
    }
    if iteration.is_multiple_of(LIVE_EVERY) {
        dump(REASON_LIVE, 0);
    }
}

fn section(start: *const u8, stop: *const u8) -> &'static [u8] {
    let length = (stop as usize).saturating_sub(start as usize);
    // SAFETY: the linker places the three sections contiguously between their
    // start/stop symbols for the life of the module; the slices are read only.
    // Counters change concurrently, which only makes a copy slightly stale.
    unsafe { std::slice::from_raw_parts(start, length) }
}

fn sections() -> Sections<'static> {
    let counters_begin = addr_of!(__start___llvm_prf_cnts);
    let data_begin = addr_of!(__start___llvm_prf_data);
    let names_begin = addr_of!(__start___llvm_prf_names);
    Sections {
        data: section(data_begin, addr_of!(__stop___llvm_prf_data)),
        counters: section(counters_begin, addr_of!(__stop___llvm_prf_cnts)),
        names: section(names_begin, addr_of!(__stop___llvm_prf_names)),
        data_begin: data_begin as u64,
        counters_begin: counters_begin as u64,
        names_begin: names_begin as u64,
    }
}

fn dump(reason: u8, detail: u32) {
    let sequence = SEQUENCE.fetch_add(1, Ordering::AcqRel) + 1;
    let name = match reason {
        REASON_SCENE => format!(
            "scene-{sequence:03}-{}-{}.profraw",
            detail & 0xff,
            (detail >> 8) & 0xff
        ),
        REASON_MANUAL => format!("manual-{sequence:03}.profraw"),
        _ => "live.profraw".to_owned(),
    };
    let detail_word = u64::from(reason) | (u64::from(detail) << 8);
    match write(&Path::new(DATA_DIR).join(DIR), &name) {
        Ok(bytes) => record(Code::CoverageDump, detail_word, bytes as u64),
        Err(error) => {
            record(Code::CoverageDump, detail_word, 0);
            if !FAILURE_REPORTED.swap(true, Ordering::AcqRel) {
                skyline::println!("[Deluxe] coverage dump failed: {}", error);
            }
        }
    }
}

/// Write the profile through a staging file so a reader never sees a partial
/// `live.profraw`; the platform rename does not replace, so the old file is
/// removed first.
fn write(dir: &Path, name: &str) -> io::Result<usize> {
    fs::create_dir_all(dir)?;
    let sections = sections();
    if !DESCRIBED.swap(true, Ordering::AcqRel) {
        let mut file = File::create(dir.join("build.txt"))?;
        writeln!(
            file,
            "build={}\ncommit={}\nfunctions={}\ncounters={}\nnames_bytes={}\ndata_begin=0x{:x}\ncounters_begin=0x{:x}\nnames_begin=0x{:x}",
            build_info::BUILD_ID,
            build_info::GIT_COMMIT,
            sections.functions(),
            sections.counters(),
            sections.names.len(),
            sections.data_begin,
            sections.counters_begin,
            sections.names_begin,
        )?;
        file.sync_all()?;
    }
    let mut buffer = Vec::with_capacity(sections.profraw_len());
    let bytes = write_profraw(&mut buffer, &sections)?;
    let staging = dir.join("writing.profraw");
    let target = dir.join(name);
    {
        let mut file = File::create(&staging)?;
        file.write_all(&buffer)?;
        file.sync_all()?;
    }
    if target.exists() {
        fs::remove_file(&target)?;
    }
    fs::rename(&staging, &target)?;
    Ok(bytes)
}
