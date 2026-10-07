//! Source-verified 13.0.5 scene entries from the preserved Quickplay fork.
//! Only tracks context; never changes a scene or network state. Unlike the fork,
//! main-menu entry ALWAYS resets the connection, including after Quickplay.
//! An unreadable stage context is ignored. Offline matches are not relabeled
//! Quickplay.
use super::{auto_driver, hooks::preflight, record, Code};
use crate::auto_policy::{self, Connection, SceneStatus};
use skyline::hooks::InlineCtx;

extern "C" {
    #[link_name = "\u{1}_ZN3app9smashball16is_training_modeEv"]
    fn is_training_mode() -> bool;

    #[link_name = "\u{1}_ZN3app7fighter23get_fighter_entry_countEv"]
    fn get_fighter_entry_count() -> i32;
}

const STAGE_SETUP: usize = 0x25d9288;

pub(super) fn active() -> bool {
    // Offline Training remains normal even while background matchmaking exists.
    if !auto_driver::fx_online() {
        return false;
    }
    unsafe { !is_training_mode() }
}

pub(super) fn ready() -> bool {
    auto_driver::ready()
}

pub(super) fn mark_match() {
    auto_driver::mark_match();
}

fn reject_stage() {
    record(Code::HookRejected, STAGE_SETUP as u64, 0);
}

#[skyline::hook(offset = 0x235aaa0, inline)]
unsafe fn deluxe_rewrite_main_menu(_: &InlineCtx) {
    super::native_banner::drop_scene();
    auto_driver::set_scene(
        Some(Connection::Offline),
        Some(SceneStatus::Inactive),
        0x235aaa0,
    );
}

#[skyline::hook(offset = 0x22da160, inline)]
unsafe fn deluxe_rewrite_quickplay(_: &InlineCtx) {
    super::native_banner::drop_scene();
    auto_driver::set_scene(
        Some(Connection::Quickplay),
        Some(SceneStatus::Inactive),
        0x22da160,
    );
}

#[skyline::hook(offset = 0x22da090, inline)]
unsafe fn deluxe_rewrite_background_matchmaking(_: &InlineCtx) {
    super::native_banner::drop_scene();
    auto_driver::set_scene(
        Some(Connection::Quickplay),
        Some(SceneStatus::Inactive),
        0x22da090,
    );
}

#[skyline::hook(offset = 0x22d9fa0, inline)]
unsafe fn deluxe_rewrite_arena(_: &InlineCtx) {
    super::native_banner::drop_scene();
    auto_driver::set_scene(Some(Connection::Arena), None, 0x22d9fa0);
}

#[skyline::hook(offset = 0x1bd45e0, inline)]
unsafe fn deluxe_rewrite_local_wireless(_: &InlineCtx) {
    super::native_banner::drop_scene();
    auto_driver::set_scene(
        Some(Connection::Local),
        Some(SceneStatus::Inactive),
        0x1bd45e0,
    );
}

#[skyline::hook(offset = STAGE_SETUP, inline)]
unsafe fn deluxe_rewrite_stage_setup(ctx: &InlineCtx) {
    super::native_banner::drop_scene();
    let base = ctx.registers[0].x() as usize;
    let Some(stage_addr) = base.checked_add(8) else {
        reject_stage();
        return;
    };
    if !super::memory::accessible(stage_addr, 4, false) {
        reject_stage();
        return;
    }
    let stage_id = *(stage_addr as *const u32);
    let status = auto_policy::classify_stage(
        stage_id,
        is_training_mode(),
        get_fighter_entry_count(),
    );
    auto_driver::set_scene(None, Some(status), STAGE_SETUP as u64);
}

pub(super) fn install() {
    // Require all resets and entries together. Unknown/overlapping patches leave
    // Online-only effects normal; explicit All remains usable for offline tests.
    // Auto also stays off this path: a failed guard does not rewrite the request.
    let offsets = [0x235aaa0, 0x22da160, 0x22da090, 0x22d9fa0, 0x1bd45e0];
    let stage_words = unsafe {
        super::renderer::expect(STAGE_SETUP, 0xaa0003f8)
            && super::renderer::expect(STAGE_SETUP + 4, 0xd63f0120)
            && super::renderer::expect(STAGE_SETUP + 8, 0xf940e316)
    };
    if !offsets.iter().all(|&offset| unsafe { preflight(offset) })
        || !stage_words
        || !super::hooks::install_match_setup()
    {
        record(Code::Compatibility, 3, STAGE_SETUP as u64);
        return;
    }
    skyline::install_hooks!(
        deluxe_rewrite_main_menu,
        deluxe_rewrite_quickplay,
        deluxe_rewrite_background_matchmaking,
        deluxe_rewrite_arena,
        deluxe_rewrite_local_wireless,
        deluxe_rewrite_stage_setup
    );
    for offset in offsets {
        record(Code::HookInstalled, offset as u64, 0);
    }
    record(Code::HookInstalled, STAGE_SETUP as u64, 0);
    auto_driver::set_ready(true, STAGE_SETUP as u64);
}
