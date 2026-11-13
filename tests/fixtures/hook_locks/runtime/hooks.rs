// Fixture for tests/hook_locks.rs (read as text, never compiled): a hook reaches a mutex through a helper and
// through a method call, and an export reaches a OnceLock initialiser.
use crate::state;
use crate::state::level;

mod shared;

#[skyline::hook(offset = 0x10, inline)]
unsafe fn on_poll(ctx: &mut InlineCtx) {
    state::bump();
    let ring = state::Ring::new();
    ring.push_row(1);
    let (level, mut count) = (ctx.registers[0].x(), 0u64);
    count += level as u64;
}

unsafe extern "C" fn on_present() {
    self::shared::fetch();
    state::try_peek();
    state::install_only();
}

unsafe extern "C" fn thread_main() {
    state::thread_only();
}

fn spawn_worker() {
    std::thread::spawn(move || {
        state::thread_only();
    });
}
