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
/// One phase tick. Idle captures do not read the clock or the current thread.
pub(super) unsafe fn mark_thread(kind: u8, index: i32) {
    let begin = start();
    if begin == 0 {
        return;
    }
    TRACE.observe(Event {
        begin,
        end: begin,
        kind,
        index,
        result: thread_label(),
    });
}
fn thread_label() -> i32 {
    let thread = unsafe { skyline::nn::os::GetCurrentThread() as usize as u64 };
    let mixed = thread
        .wrapping_add(0x9E37_79B1_85EB_CA87)
        .wrapping_mul(0x9E37_79B1_85EB_CA87)
        .rotate_left(27);
    (mixed as u32 & 0x7fff_ffff) as i32
}
