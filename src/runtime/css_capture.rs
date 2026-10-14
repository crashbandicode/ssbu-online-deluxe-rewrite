//! Opt-in character-select capture (0.3.1). Absent by default: the three
//! hooks below exist only when `CAPTURE_CSS` is in the data directory at boot.
//! They add rows to the existing finite API trace and change no task, wait,
//! priority, affinity or clock. Contract: tangent evidence
//! `css-budget/hand/CSS_CAPTURE_CONTRACT.md`.
//!
//! Rows (see `render_trace`): 64 UI dispatch, 66 dependency wait on any
//! thread, 77 UI job span, 62/63 one read-only thread setup per thread. The
//! per-frame backoff rows 60/61 come from `dispatcher_backoff::frame_rows`.
//! A capture is armed automatically when the character select is entered.
use super::hang_trace::{self, Hook};
use super::{api_trace, memory, record, Code};
use crate::render_trace::{
    core_info, dispatch_info, image_offset, pointer_halves, Event, KIND_DEPENDENCY_WAIT,
    KIND_THREAD_SETUP_INFO, KIND_THREAD_SETUP_POINTER, KIND_UI_DISPATCH, KIND_UI_JOB,
};
use skyline::hooks::{getRegionAddress, Region};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering};

/// Game dispatcher job registration (`INIT_WORKER` in `render_sequence`).
const INIT_WORKER: usize = 0x3549720;
/// Dependency wait on a counter (`WAIT_WORKER` in `render_sequence`).
const WAIT_WORKER: usize = 0x354ccd0;
/// The serial UI layout/animation job that MainThread waits for on the CSS.
const UI_JOB: usize = 0x375e260;
/// Generic child callback registered by the UI list walk.
const SUBMIT_WORKER: usize = 0x35487f0;
/// Parent aggregate of the UI list; its counter is `+0x10` (`0x5333160`).
const UI_AGGREGATE: usize = 0x5333150;
const INIT_WORDS: [u32; 5] = [0xa9bb67fa, 0xa9015ff8, 0xa90257f6, 0xa9034ff4, 0xa9047bfd];
const WAIT_WORDS: [u32; 5] = [0xd10183ff, 0xa90167fa, 0xa9025ff8, 0xa90357f6, 0xa9044ff4];
const JOB_WORDS: [u32; 5] = [0xd10283ff, 0xa9046ffc, 0xa90567fa, 0xa9065ff8, 0xa90757f6];
const FLAG: &str = "CAPTURE_CSS";
const CSS_IDLE_TICKS: u64 = crate::timing::TICK_HZ;
const THREAD_SLOTS: usize = 8;

static TEXT_BASE: AtomicUsize = AtomicUsize::new(0);
/// Bit 0 init, bit 1 wait, bit 2 job. 7 means every hook is live.
static HOOKS: AtomicU8 = AtomicU8::new(0);
static FLAG_PRESENT: AtomicBool = AtomicBool::new(false);
static GET_PRIORITY: AtomicUsize = AtomicUsize::new(0);
static GET_CORE_MASK: AtomicUsize = AtomicUsize::new(0);
static IN_CSS: AtomicBool = AtomicBool::new(false);
static LAST_CSS_TICK: AtomicU64 = AtomicU64::new(0);
static ARMED_CAPTURES: AtomicU32 = AtomicU32::new(0);
/// Threads already described in the current capture.
static THREADS: [AtomicUsize; THREAD_SLOTS] = [const { AtomicUsize::new(0) }; THREAD_SLOTS];

type GetPriorityFn = unsafe extern "C" fn(*const u8) -> i32;
type GetCoreMaskFn = unsafe extern "C" fn(*mut i32, *mut u64, *const u8);

fn base() -> usize {
    TEXT_BASE.load(Ordering::Relaxed)
}

fn point(begin: u64, kind: u8, index: i32, result: i32) {
    api_trace::TRACE.observe(Event {
        begin,
        end: begin,
        kind,
        index,
        result,
    });
}

/// First sight of a thread in this capture: two read-only rows. Nothing is set.
unsafe fn describe_thread(tick: u64, thread: usize) {
    if thread == 0 {
        return;
    }
    for slot in &THREADS {
        let seen = slot.load(Ordering::Relaxed);
        if seen == thread {
            return;
        }
        if seen == 0 {
            match slot.compare_exchange(0, thread, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_) => break,
                Err(existing) if existing == thread => return,
                Err(_) => continue,
            }
        }
    }
    let (low, high) = pointer_halves(thread as u64);
    point(tick, KIND_THREAD_SETUP_POINTER, low, high);
    let priority = match GET_PRIORITY.load(Ordering::Relaxed) {
        0 => -1,
        address => {
            let f: GetPriorityFn = std::mem::transmute(address);
            f(thread as *const u8)
        }
    };
    let info = match GET_CORE_MASK.load(Ordering::Relaxed) {
        0 => -1,
        address => {
            let f: GetCoreMaskFn = std::mem::transmute(address);
            let mut ideal = -1i32;
            let mut mask = 0u64;
            f(&mut ideal, &mut mask, thread as *const u8);
            core_info(ideal, mask)
        }
    };
    point(tick, KIND_THREAD_SETUP_INFO, priority, info);
}

/// Is this job a child of the UI aggregate? Walks the parent chain the way
/// the diagnostic probe did; every pointer is checked before it is read.
unsafe fn child_of_ui_aggregate(job: usize) -> bool {
    if !job.is_multiple_of(8) || !memory::accessible(job, 0x18, false) {
        return false;
    }
    // A record about to be registered still has a zero counter of its own.
    if *((job + 0x10) as *const u32) != 0 {
        return false;
    }
    let target = base() + UI_AGGREGATE;
    let mut parent = *((job + 8) as *const usize);
    for _ in 0..8 {
        if parent == target {
            return true;
        }
        if parent == 0 || !parent.is_multiple_of(8) || !memory::accessible(parent, 0x18, false) {
            return false;
        }
        let next = *((parent + 8) as *const usize);
        if next == parent {
            return false;
        }
        parent = next;
    }
    false
}

#[skyline::hook(offset = INIT_WORKER)]
unsafe fn css_capture_init_worker(
    job: *mut u8,
    function: *const u8,
    worker: u32,
    priority: u32,
) -> u64 {
    let begin = api_trace::start();
    if begin != 0
        && function as usize == base() + SUBMIT_WORKER
        && child_of_ui_aggregate(job as usize)
    {
        point(
            begin,
            KIND_UI_DISPATCH,
            dispatch_info(worker, priority),
            api_trace::thread_label(),
        );
    }
    hang_trace::enter(Hook::CssInit);
    let result = original!()(job, function, worker, priority);
    hang_trace::exit(Hook::CssInit);
    result
}

#[skyline::hook(offset = WAIT_WORKER)]
unsafe fn css_capture_wait_worker(slot: u32, counter: *mut u32) -> u64 {
    let begin = api_trace::start();
    hang_trace::enter(Hook::CssWait);
    let result = original!()(slot, counter);
    hang_trace::exit(Hook::CssWait);
    if begin != 0 {
        let thread = skyline::nn::os::GetCurrentThread() as usize;
        describe_thread(begin, thread);
        api_trace::TRACE.observe(Event {
            begin,
            end: skyline::nn::os::GetSystemTick() as u64,
            kind: KIND_DEPENDENCY_WAIT,
            index: image_offset(counter as usize as u64, base() as u64),
            result: crate::render_trace::thread_label(thread as u64),
        });
    }
    result
}

#[skyline::hook(offset = UI_JOB)]
unsafe fn css_capture_ui_job(object: *mut u8) -> u64 {
    let begin = api_trace::start();
    hang_trace::enter(Hook::CssUiJob);
    let result = original!()(object);
    hang_trace::exit(Hook::CssUiJob);
    if begin != 0 {
        let thread = skyline::nn::os::GetCurrentThread() as usize;
        describe_thread(begin, thread);
        api_trace::TRACE.observe(Event {
            begin,
            end: skyline::nn::os::GetSystemTick() as u64,
            kind: KIND_UI_JOB,
            index: -1,
            result: crate::render_trace::thread_label(thread as u64),
        });
    }
    result
}

unsafe fn words_match(site: usize, words: &[u32; 5]) -> bool {
    words
        .iter()
        .enumerate()
        .all(|(n, &word)| super::renderer::expect(site + n * 4, word))
}

unsafe fn lookup(name: &[u8]) -> usize {
    let mut address = 0usize;
    skyline::nn::ro::LookupSymbol(&mut address, name.as_ptr());
    address
}

/// Startup, after the renderer family passed its own preflight on these sites.
pub(super) fn install() {
    TEXT_BASE.store(
        unsafe { getRegionAddress(Region::Text) as usize },
        Ordering::Relaxed,
    );
    let present = std::path::Path::new(super::DATA_DIR).join(FLAG).is_file();
    FLAG_PRESENT.store(present, Ordering::Release);
    // `CAPTURE_MATCH` wants the same read-only rows in a match; only
    // `CAPTURE_CSS` arms on character-select entry.
    if !present && !super::match_capture::flag_present() {
        record(Code::CssCapture, 0, 0);
        return;
    }
    let mut hooks = 0u8;
    unsafe {
        GET_PRIORITY.store(
            lookup(b"_ZN2nn2os17GetThreadPriorityEPKNS0_10ThreadTypeE\0"),
            Ordering::Relaxed,
        );
        GET_CORE_MASK.store(
            lookup(b"_ZN2nn2os17GetThreadCoreMaskEPiPmPKNS0_10ThreadTypeE\0"),
            Ordering::Relaxed,
        );
        if words_match(INIT_WORKER, &INIT_WORDS) && super::hooks::preflight(INIT_WORKER) {
            skyline::install_hook!(css_capture_init_worker);
            hooks |= 1;
            record(
                Code::HookInstalled,
                INIT_WORKER as u64,
                u64::from(KIND_UI_DISPATCH),
            );
        }
        if words_match(WAIT_WORKER, &WAIT_WORDS) && super::hooks::preflight(WAIT_WORKER) {
            skyline::install_hook!(css_capture_wait_worker);
            hooks |= 2;
            record(
                Code::HookInstalled,
                WAIT_WORKER as u64,
                u64::from(KIND_DEPENDENCY_WAIT),
            );
        }
        if words_match(UI_JOB, &JOB_WORDS) && super::hooks::preflight(UI_JOB) {
            skyline::install_hook!(css_capture_ui_job);
            hooks |= 4;
            record(Code::HookInstalled, UI_JOB as u64, u64::from(KIND_UI_JOB));
        }
    }
    HOOKS.store(hooks, Ordering::Release);
    record(
        Code::CssCapture,
        u64::from(hooks),
        u64::from(present)
            | (u64::from(GET_PRIORITY.load(Ordering::Relaxed) != 0) << 1)
            | (u64::from(GET_CORE_MASK.load(Ordering::Relaxed) != 0) << 2),
    );
    if present {
        skyline::println!(
            "[Deluxe] CSS capture hooks {hooks:#x}; arming on character-select entry"
        );
    } else {
        skyline::println!("[Deluxe] CSS capture hooks {hooks:#x} for the match capture");
    }
}

fn hooks_ready() -> bool {
    HOOKS.load(Ordering::Acquire) == 7
}

/// The game's character-select update ran (native banner hook). The first
/// call of a visit records the entry and, with every hook live, arms a capture.
pub(super) fn note_css_frame() {
    let tick = unsafe { skyline::nn::os::GetSystemTick() as u64 };
    LAST_CSS_TICK.store(tick, Ordering::Relaxed);
    if IN_CSS.swap(true, Ordering::AcqRel) {
        return;
    }
    let ready = hooks_ready() && FLAG_PRESENT.load(Ordering::Acquire);
    record(Code::CssEntered, tick, u64::from(ready));
    if ready && super::timing::request_css() {
        ARMED_CAPTURES.fetch_add(1, Ordering::Relaxed);
    }
}

/// A character-select visit is open (an update ran within the last second).
pub(super) fn in_css() -> bool {
    IN_CSS.load(Ordering::Acquire)
}

/// Diagnostics thread, every 500 ms: close the visit one second after the
/// last update and forget the described threads between captures.
pub(super) fn poll() {
    let now = unsafe { skyline::nn::os::GetSystemTick() as u64 };
    if IN_CSS.load(Ordering::Acquire)
        && now.wrapping_sub(LAST_CSS_TICK.load(Ordering::Relaxed)) > CSS_IDLE_TICKS
    {
        IN_CSS.store(false, Ordering::Release);
        record(Code::CssExited, now, 0);
    }
    if !api_trace::active() {
        for slot in &THREADS {
            slot.store(0, Ordering::Relaxed);
        }
    }
}

pub(super) fn status_label() -> String {
    if !FLAG_PRESENT.load(Ordering::Acquire) {
        return match HOOKS.load(Ordering::Acquire) {
            0 => "Off (no CAPTURE_CSS)".to_string(),
            hooks => format!("Hooks {hooks:#x} for CAPTURE_MATCH; not arming"),
        };
    }
    match HOOKS.load(Ordering::Acquire) {
        7 => format!(
            "Armed; {} capture(s) on CSS entry",
            ARMED_CAPTURES.load(Ordering::Relaxed)
        ),
        0 => "Unavailable (words differ)".to_string(),
        hooks => format!("Partial {hooks:#x}; not arming"),
    }
}
