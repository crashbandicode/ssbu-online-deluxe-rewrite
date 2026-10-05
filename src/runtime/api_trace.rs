//! Opt-in spans around existing CPU calls. Zero clock reads while idle.
use crate::render_trace::{Event, Trace};
pub(super) static TRACE: Trace = Trace::new();
pub(super) fn start() -> u64 {
    if TRACE.active() {
        unsafe { skyline::nn::os::GetSystemTick() as u64 }
    } else {
        0
    }
}
pub(super) fn end(begin: u64, kind: u8, index: i32, result: i32) {
    if begin == 0 {
        return;
    }
    TRACE.observe(Event {
        begin,
        end: unsafe { skyline::nn::os::GetSystemTick() as u64 },
        kind,
        index,
        result,
    });
}
