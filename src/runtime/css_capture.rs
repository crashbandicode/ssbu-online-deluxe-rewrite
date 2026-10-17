//! Opt-in character-select hooks (0.3.1). Absent by default: the two hooks
//! below exist only when `CAPTURE_CSS` or `CAPTURE_MATCH` is in the data
//! directory at boot. They add rows to the finite API trace and change no
//! task, wait, priority, affinity or clock. Contract: tangent evidence
//! `css-budget/hand/CSS_CAPTURE_CONTRACT.md`.
//!
//! Rows (see `capture`): 66 dependency wait on any thread, 77 UI job span.
//! Arming on character-select entry and the visit state live in `capture`.
use super::hang_trace::{self, Hook};
use super::{api_trace, record, Code};
use crate::capture::{image_offset, KIND_DEPENDENCY_WAIT, KIND_UI_JOB};
use skyline::hooks::{getRegionAddress, Region};
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

/// Dependency wait on a counter (`WAIT_WORKER` in `render_sequence`).
const WAIT_WORKER: usize = 0x354ccd0;
/// The serial UI layout/animation job that MainThread waits for on the CSS.
const UI_JOB: usize = 0x375e260;
const WAIT_WORDS: [u32; 5] = [0xd10183ff, 0xa90167fa, 0xa9025ff8, 0xa90357f6, 0xa9044ff4];
const JOB_WORDS: [u32; 5] = [0xd10283ff, 0xa9046ffc, 0xa90567fa, 0xa9065ff8, 0xa90757f6];
/// `HOOKS` bits: 1 wait, 2 job. Both live means the CSS capture may arm.
pub(super) const HOOKS_ALL: u8 = 3;

static TEXT_BASE: AtomicUsize = AtomicUsize::new(0);
static HOOKS: AtomicU8 = AtomicU8::new(0);

#[skyline::hook(offset = WAIT_WORKER)]
unsafe fn css_capture_wait_worker(slot: u32, counter: *mut u32) -> u64 {
    let begin = api_trace::start();
    hang_trace::enter(Hook::CssWait);
    let result = original!()(slot, counter);
    hang_trace::exit(Hook::CssWait);
    if begin != 0 {
        let index = image_offset(
            counter as usize as u64,
            TEXT_BASE.load(Ordering::Relaxed) as u64,
        );
        api_trace::end(
            begin,
            KIND_DEPENDENCY_WAIT,
            index,
            api_trace::thread_label(),
        );
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
        api_trace::end(begin, KIND_UI_JOB, -1, api_trace::thread_label());
    }
    result
}

unsafe fn words_match(site: usize, words: &[u32; 5]) -> bool {
    words
        .iter()
        .enumerate()
        .all(|(n, &word)| super::renderer::expect(site + n * 4, word))
}

/// Startup, after `capture::install` read the flags and the renderer family
/// passed its own preflight on these sites. Code 104: `a` is the hook mask,
/// `b` bit 0 whether `CAPTURE_CSS` arms on character-select entry.
pub(super) fn install() {
    TEXT_BASE.store(
        unsafe { getRegionAddress(Region::Text) as usize },
        Ordering::Relaxed,
    );
    let flags = super::capture::flags();
    if flags == 0 {
        record(Code::CssCapture, 0, 0);
        return;
    }
    let mut hooks = 0u8;
    unsafe {
        if words_match(WAIT_WORKER, &WAIT_WORDS) && super::hooks::preflight(WAIT_WORKER) {
            skyline::install_hook!(css_capture_wait_worker);
            hooks |= 1;
            record(
                Code::HookInstalled,
                WAIT_WORKER as u64,
                u64::from(KIND_DEPENDENCY_WAIT),
            );
        }
        if words_match(UI_JOB, &JOB_WORDS) && super::hooks::preflight(UI_JOB) {
            skyline::install_hook!(css_capture_ui_job);
            hooks |= 2;
            record(Code::HookInstalled, UI_JOB as u64, u64::from(KIND_UI_JOB));
        }
    }
    HOOKS.store(hooks, Ordering::Release);
    let arming = flags & super::capture::FLAG_CSS != 0;
    record(Code::CssCapture, u64::from(hooks), u64::from(arming));
    if arming {
        skyline::println!(
            "[Deluxe] CSS capture hooks {hooks:#x}; arming on character-select entry"
        );
    } else {
        skyline::println!("[Deluxe] CSS capture hooks {hooks:#x} for the match capture");
    }
}

pub(super) fn hooks() -> u8 {
    HOOKS.load(Ordering::Acquire)
}

pub(super) fn hooks_ready() -> bool {
    hooks() == HOOKS_ALL
}
