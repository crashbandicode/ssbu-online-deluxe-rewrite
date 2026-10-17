//! Opt-in spans around existing CPU calls. Zero clock reads while idle.
use crate::capture::{Event, Ring, API_ROWS};
pub(super) static TRACE: Ring<Event> = Ring::new(API_ROWS);
/// One atomic load. Hooks that only aggregate while a capture runs ask this.
pub(super) fn active() -> bool {
    TRACE.active()
}
pub(super) fn start() -> u64 {
    if TRACE.active() {
        unsafe { skyline::nn::os::GetSystemTick() as u64 }
    } else {
        0
    }
}
/// Span row from `start()` to now; nothing when the capture was idle at `start()`.
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
/// Point row at `begin`; nothing when the capture was idle at `start()`.
pub(super) fn point(begin: u64, kind: u8, index: i32, result: i32) {
    if begin == 0 {
        return;
    }
    TRACE.observe(Event {
        begin,
        end: begin,
        kind,
        index,
        result,
    });
}
/// One phase tick. Idle captures do not read the clock or the current thread.
pub(super) unsafe fn mark_thread(kind: u8, index: i32) {
    let begin = start();
    if begin != 0 {
        point(begin, kind, index, thread_label());
    }
}
/// Hashed label of the calling SDK thread; the pure function is in `capture`.
pub(super) fn thread_label() -> i32 {
    let thread = unsafe { skyline::nn::os::GetCurrentThread() as usize as u64 };
    crate::capture::thread_label(thread)
}
