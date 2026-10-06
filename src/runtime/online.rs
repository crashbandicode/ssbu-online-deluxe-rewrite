//! Source-verified 13.0.5 scene entries from the preserved Quickplay fork.
//! Only tracks context; never changes a scene or network state. Unlike the fork,
//! main-menu entry ALWAYS resets this flag, including after Quickplay.
use super::{hooks::preflight, record, Code};
use skyline::hooks::InlineCtx;
use std::sync::atomic::{AtomicBool, Ordering};

static ONLINE: AtomicBool = AtomicBool::new(false);
static READY: AtomicBool = AtomicBool::new(false);

extern "C" {
    #[link_name = "\u{1}_ZN3app9smashball16is_training_modeEv"]
    fn is_training_mode() -> bool;
}

pub(super) fn active() -> bool {
    // Offline Training remains normal even while background matchmaking exists.
    READY.load(Ordering::Relaxed)
        && ONLINE.load(Ordering::Relaxed)
        && !unsafe { is_training_mode() }
}

pub(super) fn ready() -> bool {
    READY.load(Ordering::Relaxed)
}

pub(super) fn mark_match() {
    if ready() {
        set(true, 0x16ccab8);
    }
}

fn set(online: bool, source: u64) {
    let previous = ONLINE.swap(online, Ordering::Relaxed);
    if previous != online {
        record(Code::FxOnlineContext, online as u64, source);
    }
}

#[skyline::hook(offset = 0x235aaa0, inline)]
unsafe fn deluxe_rewrite_main_menu(_: &InlineCtx) {
    set(false, 0x235aaa0);
}

#[skyline::hook(offset = 0x22da160, inline)]
unsafe fn deluxe_rewrite_quickplay(_: &InlineCtx) {
    set(true, 0x22da160);
}

#[skyline::hook(offset = 0x22da090, inline)]
unsafe fn deluxe_rewrite_background_matchmaking(_: &InlineCtx) {
    set(true, 0x22da090);
}

#[skyline::hook(offset = 0x22d9fa0, inline)]
unsafe fn deluxe_rewrite_arena(_: &InlineCtx) {
    set(true, 0x22d9fa0);
}

#[skyline::hook(offset = 0x1bd45e0, inline)]
unsafe fn deluxe_rewrite_local_wireless(_: &InlineCtx) {
    set(true, 0x1bd45e0);
}

pub(super) fn install() {
    // Require all resets and entries together. Unknown/overlapping patches leave
    // Online-only effects normal; explicit All remains usable for offline tests.
    let offsets = [0x235aaa0, 0x22da160, 0x22da090, 0x22d9fa0, 0x1bd45e0];
    if !offsets.iter().all(|&offset| unsafe { preflight(offset) }) {
        record(Code::Compatibility, 3, 0);
        return;
    }
    if !super::hooks::install_match_setup() {
        record(Code::Compatibility, 3, 0x16ccab8);
        return;
    }
    skyline::install_hooks!(
        deluxe_rewrite_main_menu,
        deluxe_rewrite_quickplay,
        deluxe_rewrite_background_matchmaking,
        deluxe_rewrite_arena,
        deluxe_rewrite_local_wireless
    );
    for offset in offsets {
        record(Code::HookInstalled, offset as u64, 0);
    }
    READY.store(true, Ordering::Relaxed);
}
