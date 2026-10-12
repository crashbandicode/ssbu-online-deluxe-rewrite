//! Opt-in, bounded `nn::vi` frame-history probe. It reports compositor
//! bookkeeping (frame numbers and SDK-converted enqueue/present ticks) and
//! nothing about scanout or input latency.
//!
//! Enabled only when `DATA_DIR/DUMP_PRESENTATION_HISTORY` exists at boot.
//! The layer pointer is captured from a symbol-bound function hook on
//! `nn::vi::GetNativeWindow`; its lifetime is guarded by a symbol-bound hook
//! on `nn::vi::DestroyLayer` and a mutex that is held for the whole duration
//! of every query, so a query can never run against a destroyed layer and
//! destruction can never race a query. All symbols are resolved at runtime
//! with `nn::ro::LookupSymbol`; no text offsets are used. The query functions
//! lock an SDK mutex internally (`nnosLockMutex`), so calling them from the
//! diagnostic thread is legitimate.
//!
//! Queries run from the existing diagnostic thread (`poll` every 500 ms):
//! ten one-second samples of the 8-entry history, with explicit stop rules
//! for unsupported, empty, not-yet and timeout cases. File I/O happens only
//! on that thread, plus one status write at install.
use super::{logging, memory, record, Code, DATA_DIR};
use crate::presentation::{
    self as model, Latest, Observation, Outcome, RawFrameInfo, RunState, SampleMeta, CAPACITY,
};
use skyline::hooks::{getRegionAddress, Region};
use skyline::libc::c_void;
use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{
        atomic::{AtomicU8, AtomicUsize, Ordering},
        Mutex,
    },
    time::Instant,
};

const FLAG_FILE: &str = "DUMP_PRESENTATION_HISTORY";
const REPORT_FILE: &str = "presentation-history.txt";
/// Record site ids for this module (0x200 is the overlay host).
const SITE: u64 = 0x300;
const SITE_GET_NATIVE_WINDOW: u64 = SITE + 1;
const SITE_DESTROY_LAYER: u64 = SITE + 2;
const SITE_LIST_FRAME_HISTORY: u64 = SITE + 3;
const SITE_LATEST_FRAME_NUMBER: u64 = SITE + 4;
const OVERALL_DEADLINE_SECS: u64 = 90;
const SAMPLE_INTERVAL_MS: u128 = 1000;

const SYM_GET_NATIVE_WINDOW: &[u8] = b"_ZN2nn2vi15GetNativeWindowEPPvPNS0_5LayerE\0";
const SYM_DESTROY_LAYER: &[u8] = b"_ZN2nn2vi12DestroyLayerEPNS0_5LayerE\0";
const SYM_LIST_FRAME_HISTORY: &[u8] =
    b"_ZN2nn2vi16ListFrameHistoryEPNS0_9FrameInfoEiPKNS0_5LayerE\0";
const SYM_LATEST_FRAME_NUMBER: &[u8] = b"_ZN2nn2vi20GetLatestFrameNumberEPmPKNS0_5LayerE\0";

type GetNativeWindowFn = unsafe extern "C" fn(*mut *mut c_void, *mut c_void) -> u32;
type DestroyLayerFn = unsafe extern "C" fn(*mut c_void);
type ListFrameHistoryFn = unsafe extern "C" fn(*mut RawFrameInfo, i32, *const c_void) -> i32;
type LatestFrameNumberFn = unsafe extern "C" fn(*mut u64, *const c_void) -> u32;

const STATE_OFF: u8 = 0;
const STATE_ARMED: u8 = 1;
const STATE_RUNNING: u8 = 2;
const STATE_DONE: u8 = 3;
const STATE_REFUSED: u8 = 4;

static STATE: AtomicU8 = AtomicU8::new(STATE_OFF);
static ORIGINAL_GET_NATIVE_WINDOW: AtomicUsize = AtomicUsize::new(0);
static ORIGINAL_DESTROY_LAYER: AtomicUsize = AtomicUsize::new(0);
static LIST_FRAME_HISTORY: AtomicUsize = AtomicUsize::new(0);
static LATEST_FRAME_NUMBER: AtomicUsize = AtomicUsize::new(0);

struct LayerSlot {
    pointer: usize,
    generation: u64,
    distinct: u32,
}
/// Held by the hooks only for a few instructions and by the sampler for the
/// whole query, which is what makes the layer pointer valid during a query.
static LAYER: Mutex<LayerSlot> = Mutex::new(LayerSlot {
    pointer: 0,
    generation: 0,
    distinct: 0,
});

struct Run {
    started: Instant,
    last_sample: Option<Instant>,
    state: RunState,
    report: model::Report,
}
/// Diagnostic thread only.
static RUN: Mutex<Option<Run>> = Mutex::new(None);

fn lookup(symbol: &[u8]) -> usize {
    let mut address = 0usize;
    unsafe {
        skyline::nn::ro::LookupSymbol(&mut address, symbol.as_ptr());
    }
    address
}

fn text_offset(address: usize) -> u64 {
    let base = unsafe { getRegionAddress(Region::Text) } as usize;
    address.wrapping_sub(base) as u64
}

fn write_report(text: &str) {
    if let Ok(mut file) = File::create(Path::new(DATA_DIR).join(REPORT_FILE)) {
        let _ = file.write_all(text.as_bytes());
        let _ = file.sync_all();
    }
}

fn refuse(phase: &str, site: u64, detail: u64, header: &[(&str, String)], reason: &str) {
    STATE.store(STATE_REFUSED, Ordering::Release);
    record(Code::HookRejected, site, detail);
    logging::startup_phase(phase);
    let mut report = model::Report::new(header);
    report.push(format!("outcome=refused reason={reason}"));
    write_report(&report.render());
}

/// Installs one symbol-bound function hook with entry-word evidence before
/// and after. A failure refuses sampling; an installed observer may remain.
unsafe fn hook(
    target: usize,
    replacement: *const c_void,
    original: &AtomicUsize,
    site: u64,
) -> bool {
    if target == 0 || !memory::accessible(target, 20, false) {
        record(Code::HookRejected, site, 0);
        return false;
    }
    let offset = text_offset(target);
    let before = *(target as *const u32);
    record(Code::HookEntryWord, offset, before as u64);
    logging::install_checkpoint(Code::HookEntryWord, offset, before as u64);
    if before == 0x5800_0051 || before & 0xfc00_0000 == 0x1400_0000 {
        // Another plugin already owns this entry; chaining is normal, record it.
        record(Code::Compatibility, site, before as u64);
    }
    record(Code::HookInstalling, offset, target as u64);
    let mut trampoline: *mut c_void = std::ptr::null_mut();
    skyline::hooks::A64HookFunction(target as *const c_void, replacement, &mut trampoline);
    if trampoline.is_null() {
        record(Code::HookRejected, site, 1);
        return false;
    }
    original.store(trampoline as usize, Ordering::Release);
    let after = *(target as *const u32);
    let installed = after == 0x5800_0051 || after & 0xfc00_0000 == 0x1400_0000;
    record(
        if installed {
            Code::HookInstalled
        } else {
            Code::HookRejected
        },
        offset,
        after as u64,
    );
    installed
}

unsafe extern "C" fn get_native_window_hook(out: *mut *mut c_void, layer: *mut c_void) -> u32 {
    let original = ORIGINAL_GET_NATIVE_WINDOW.load(Ordering::Acquire);
    if original == 0 {
        // Unreachable once installed; mirrors the SDK's own failure value.
        return model::RESULT_VI_FAILURE;
    }
    let original: GetNativeWindowFn = std::mem::transmute(original);
    let result = original(out, layer);
    if result == 0 && !layer.is_null() {
        let mut slot = LAYER.lock().unwrap_or_else(|poison| poison.into_inner());
        if slot.pointer == 0 {
            slot.pointer = layer as usize;
            slot.generation += 1;
            slot.distinct += 1;
        } else if slot.pointer != layer as usize {
            slot.distinct += 1;
        }
    }
    result
}

unsafe extern "C" fn destroy_layer_hook(layer: *mut c_void) {
    {
        // Blocks until any in-flight query on this layer has returned.
        let mut slot = LAYER.lock().unwrap_or_else(|poison| poison.into_inner());
        if slot.pointer == layer as usize && slot.pointer != 0 {
            slot.pointer = 0;
            slot.generation += 1;
        }
    }
    let original = ORIGINAL_DESTROY_LAYER.load(Ordering::Acquire);
    if original != 0 {
        let original: DestroyLayerFn = std::mem::transmute(original);
        original(layer);
    }
}

fn header(symbols: [usize; 4]) -> Vec<(&'static str, String)> {
    vec![
        ("build", crate::build_info::BUILD_ID.to_owned()),
        ("metric", "compositor_frame_history".to_owned()),
        (
            "source",
            "nn::vi::ListFrameHistory and nn::vi::GetLatestFrameNumber, resolved by symbol at runtime".to_owned(),
        ),
        (
            "layer_capture",
            "nn::vi::GetNativeWindow symbol hook; lifetime guarded by nn::vi::DestroyLayer symbol hook plus a mutex held for each query".to_owned(),
        ),
        (
            "unit",
            format!(
                "nn_os_tick at {} Hz; the SDK converts compositor nanoseconds with nnosConvertNanoSecondsToTick (SDK map 0x71005248D0); verify with presented deltas near 320000 ticks at 60 Hz",
                model::TICK_HZ
            ),
        ),
        (
            "claims",
            "compositor bookkeeping only; not scanout, not display latency, not input latency".to_owned(),
        ),
        (
            "symbols",
            format!(
                "get_native_window=0x{:x} destroy_layer=0x{:x} list_frame_history=0x{:x} latest_frame_number=0x{:x}",
                symbols[0], symbols[1], symbols[2], symbols[3]
            ),
        ),
    ]
}

/// Call once from plugin `main` after the version and legacy-host checks.
/// Does nothing unless the flag file exists.
pub(super) fn install() {
    if STATE.load(Ordering::Acquire) != STATE_OFF || !Path::new(DATA_DIR).join(FLAG_FILE).exists() {
        return;
    }
    logging::startup_phase("presentation_probe_begin");
    let symbols = [
        lookup(SYM_GET_NATIVE_WINDOW),
        lookup(SYM_DESTROY_LAYER),
        lookup(SYM_LIST_FRAME_HISTORY),
        lookup(SYM_LATEST_FRAME_NUMBER),
    ];
    let header = header(symbols);
    let names = [
        "get_native_window",
        "destroy_layer",
        "list_frame_history",
        "latest_frame_number",
    ];
    for (n, address) in symbols.iter().enumerate() {
        if *address == 0 {
            refuse(
                "presentation_probe_symbol_missing",
                SITE + 1 + n as u64,
                0,
                &header,
                &format!("symbol_missing:{}", names[n]),
            );
            return;
        }
    }
    LIST_FRAME_HISTORY.store(symbols[2], Ordering::Release);
    LATEST_FRAME_NUMBER.store(symbols[3], Ordering::Release);
    record(
        Code::HookEntryWord,
        SITE_LIST_FRAME_HISTORY,
        text_offset(symbols[2]),
    );
    record(
        Code::HookEntryWord,
        SITE_LATEST_FRAME_NUMBER,
        text_offset(symbols[3]),
    );

    let native_window_hooked = unsafe {
        hook(
            symbols[0],
            get_native_window_hook as *const c_void,
            &ORIGINAL_GET_NATIVE_WINDOW,
            SITE_GET_NATIVE_WINDOW,
        )
    };
    if !native_window_hooked {
        refuse(
            "presentation_probe_hook_failed",
            SITE_GET_NATIVE_WINDOW,
            2,
            &header,
            "get_native_window_hook_failed",
        );
        return;
    }
    let destroy_hooked = unsafe {
        hook(
            symbols[1],
            destroy_layer_hook as *const c_void,
            &ORIGINAL_DESTROY_LAYER,
            SITE_DESTROY_LAYER,
        )
    };
    if !destroy_hooked {
        // The layer observer stays passive; without lifetime protection no
        // query is ever issued.
        refuse(
            "presentation_probe_lifetime_hook_failed",
            SITE_DESTROY_LAYER,
            2,
            &header,
            "destroy_layer_hook_failed_no_queries",
        );
        return;
    }

    let mut report = model::Report::new(&header);
    report.push("state=armed waiting_for_layer".to_owned());
    write_report(&report.render());
    *RUN.lock().unwrap_or_else(|poison| poison.into_inner()) = Some(Run {
        started: Instant::now(),
        last_sample: None,
        state: RunState::default(),
        report: model::Report::new(&header),
    });
    STATE.store(STATE_ARMED, Ordering::Release);
    logging::startup_phase("presentation_probe_armed");
}

struct Query {
    poll_tick: u64,
    cost_ticks: u64,
    latest: Latest,
    capacity: Option<i32>,
    count: i32,
    entries: [RawFrameInfo; CAPACITY],
}

/// Runs the SDK queries while holding the layer lock. Returns `None` when no
/// valid layer is currently captured.
fn query(first: bool) -> Option<(Query, u64, u32)> {
    let slot = LAYER.lock().unwrap_or_else(|poison| poison.into_inner());
    if slot.pointer == 0 {
        return None;
    }
    let layer = slot.pointer as *const c_void;
    let latest_fn: LatestFrameNumberFn =
        unsafe { std::mem::transmute(LATEST_FRAME_NUMBER.load(Ordering::Acquire)) };
    let list_fn: ListFrameHistoryFn =
        unsafe { std::mem::transmute(LIST_FRAME_HISTORY.load(Ordering::Acquire)) };
    let mut entries = [RawFrameInfo::default(); CAPACITY];
    let mut frame = 0u64;
    let start = unsafe { skyline::nn::os::GetSystemTick() } as u64;
    let capacity = first.then(|| unsafe { list_fn(std::ptr::null_mut(), 0, layer) });
    let count = if capacity.is_none() || capacity == Some(CAPACITY as i32) {
        unsafe { list_fn(entries.as_mut_ptr(), CAPACITY as i32, layer) }
    } else {
        0
    };
    // These are separate SDK calls, not an atomic snapshot. Read latest after
    // history so a frame queued between calls does not look newer than latest.
    let result = unsafe { latest_fn(&mut frame, layer) };
    let end = unsafe { skyline::nn::os::GetSystemTick() } as u64;
    Some((
        Query {
            poll_tick: end,
            cost_ticks: end.wrapping_sub(start),
            latest: model::classify_latest(result, frame),
            capacity,
            count,
            entries,
        },
        slot.generation,
        slot.distinct,
    ))
}

/// Call from the existing diagnostic sink loop (every 500 ms). Bounded: ten
/// settled samples, at most twenty attempts, ninety seconds overall.
pub(super) fn poll() {
    let state = STATE.load(Ordering::Acquire);
    if state != STATE_ARMED && state != STATE_RUNNING {
        return;
    }
    let mut guard = RUN.lock().unwrap_or_else(|poison| poison.into_inner());
    let Some(run) = guard.as_mut() else {
        return;
    };
    let deadline_passed = run.started.elapsed().as_secs() >= OVERALL_DEADLINE_SECS;
    if !deadline_passed
        && run
            .last_sample
            .is_some_and(|last| last.elapsed().as_millis() < SAMPLE_INTERVAL_MS)
    {
        return;
    }
    let first = run.state.capacity_query.is_none();
    let Some((query, generation, distinct)) = query(first) else {
        if run.state.layer_seen {
            run.report
                .push("layer_destroyed_before_completion".to_owned());
            finish(run, true);
        } else if deadline_passed {
            finish(run, true);
        }
        return;
    };
    STATE.store(STATE_RUNNING, Ordering::Release);
    run.last_sample = Some(Instant::now());
    if !run.state.layer_seen {
        run.state.layer_seen = true;
        run.report.push(format!(
            "layer_generation={generation} distinct_layers={distinct}"
        ));
    }
    if let Some(capacity) = query.capacity {
        run.state.capacity_query = Some(capacity);
        run.report
            .push(format!("capacity_query={capacity} expected={CAPACITY}"));
    }
    let latest_frame = match query.latest {
        Latest::Frame(n) => Some(n),
        _ => None,
    };
    let check = model::check_sample(&query.entries, query.count, query.poll_tick, latest_frame);
    let observation = match query.latest {
        Latest::Error(code) => Observation::Error(code),
        Latest::NotYet => Observation::NotYet,
        Latest::Frame(_) if !check.is_valid() => Observation::Invalid,
        Latest::Frame(_) if check.count == 0 => Observation::Empty,
        Latest::Frame(_) => Observation::Ok,
    };
    run.state.record(observation, &check.presented_deltas);
    let meta = SampleMeta {
        index: run.state.attempts,
        poll_tick: query.poll_tick,
        cost_ticks: query.cost_ticks,
        latest: query.latest,
        count: query.count,
    };
    run.report.push_sample(&meta, &query.entries, &check);
    finish(run, deadline_passed);
}

/// Writes the report; marks the run done when the outcome is settled.
fn finish(run: &mut Run, deadline_passed: bool) {
    let outcome = run.state.outcome(deadline_passed);
    let mut report = run.report.clone();
    report.push_summary(&run.state, outcome);
    write_report(&report.render());
    if outcome != Outcome::Pending {
        STATE.store(STATE_DONE, Ordering::Release);
        record(Code::Compatibility, SITE, outcome.code());
        logging::startup_phase("presentation_probe_done");
    }
}
