//! Source-verified 13.0.5 scene entries from the preserved Quickplay fork.
//! Only tracks context; never changes a scene or network state. Every hook's
//! connection write follows `auto_policy::entry_connection`. Main-menu entry
//! resets the connection unless Quickplay is live (0.4.2; background
//! matchmaking passes through the main menu and the match starts from there,
//! see `auto_policy::main_menu_connection`); a Quickplay that turns out stale
//! is demoted five seconds into a match without the online-latency hook. The
//! fork's Quickplay menu sites write no connection: the offline Games & More
//! flow reaches `0x22da160` too. The stage's own words decide instead: online
//! Quickplay or Elite words promote an Offline connection (0.10.2), and the
//! game's Training mode makes the stage offline.
//! An unreadable stage context is ignored.
use super::{auto_driver, hooks::preflight, record, Code};
use crate::auto_policy::{self, Entry, SceneStatus, StageWords};
use skyline::hooks::InlineCtx;

extern "C" {
    #[link_name = "\u{1}_ZN3app9smashball16is_training_modeEv"]
    fn is_training_mode() -> bool;

    #[link_name = "\u{1}_ZN3app7fighter23get_fighter_entry_countEv"]
    fn get_fighter_entry_count() -> i32;
}

const STAGE_SETUP: usize = 0x25d9288;

pub(super) fn ready() -> bool {
    auto_driver::ready()
}

/// The match capture's CSS end lives in `capture::note_css_frame`, which the
/// same character-select hook calls on every update, before the pane walk.
pub(super) fn observe_css_scene() {
    if !ready() {
        return;
    }
    let status = auto_policy::classify_css(unsafe { is_training_mode() });
    auto_driver::set_scene(None, Some(status), super::native_banner::CSS_UPDATE as u64);
}

pub(super) fn mark_match() {
    auto_driver::mark_match();
}

fn reject_stage() {
    super::clear_auto_buffer();
    record(Code::HookRejected, STAGE_SETUP as u64, 0);
}

#[skyline::hook(offset = 0x235aaa0, inline)]
unsafe fn deluxe_rewrite_main_menu(_: &InlineCtx) {
    super::native_banner::drop_scene();
    auto_driver::note_main_menu(0x235aaa0);
}

#[skyline::hook(offset = 0x22da160, inline)]
unsafe fn deluxe_rewrite_quickplay(_: &InlineCtx) {
    super::native_banner::drop_scene();
    auto_driver::note_menu_entry(Entry::QuickplayMenu, Some(SceneStatus::Inactive), 0x22da160);
}

#[skyline::hook(offset = 0x22da090, inline)]
unsafe fn deluxe_rewrite_background_matchmaking(_: &InlineCtx) {
    super::native_banner::drop_scene();
    auto_driver::note_menu_entry(Entry::QuickplayMenu, Some(SceneStatus::Inactive), 0x22da090);
}

#[skyline::hook(offset = 0x22d9fa0, inline)]
unsafe fn deluxe_rewrite_arena(_: &InlineCtx) {
    super::native_banner::drop_scene();
    auto_driver::note_menu_entry(Entry::ArenaMenu, None, 0x22d9fa0);
}

#[skyline::hook(offset = 0x1bd45e0, inline)]
unsafe fn deluxe_rewrite_local_wireless(ctx: &InlineCtx) {
    super::native_banner::enter_local_scene(ctx.registers[0].x());
    auto_driver::note_menu_entry(Entry::LocalWireless, Some(SceneStatus::Inactive), 0x1bd45e0);
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
    // Waiting-room lobby practice counts as Training whatever stage it loads.
    let mode = super::scene_mode::current();
    let lobby = mode.is_some_and(|(initialized, melee, word)| {
        auto_policy::is_waiting_practice(initialized, melee, word)
    });
    let training_mode = is_training_mode();
    let status =
        auto_policy::classify_stage(stage_id, training_mode || lobby, get_fighter_entry_count());
    // The game's words decide the connection: an online stage the menu hooks
    // missed (Elite Smash on the console) is Quickplay, and the game's Training
    // mode is offline whatever a hook stored. Before the match buffer opens: a
    // connection change clears it.
    let online = matches!(mode, Some((true, melee, word))
        if auto_policy::is_online_stage(true, melee, word));
    auto_driver::note_stage_words(
        StageWords {
            training_mode,
            lobby,
            online,
        },
        mode.map_or(0, |(_, _, word)| word),
        STAGE_SETUP as u64,
    );
    // Reset on each stage entry, even if two consecutive stages classify alike.
    // A result from the previous match must never become this match's value.
    match status {
        SceneStatus::Singles | SceneStatus::Doubles => super::begin_auto_buffer_match(),
        SceneStatus::Training => super::clear_auto_buffer(),
        SceneStatus::Inactive => super::end_auto_buffer_match(),
    }
    record(Code::StageLoaded, u64::from(stage_id), status as u64);
    let (fields, word) = mode.map_or((0, 0), |(initialized, melee, word)| {
        (
            u64::from(melee) | u64::from(initialized) << 8 | 1 << 9,
            u64::from(word),
        )
    });
    record(Code::SceneMode, fields | u64::from(lobby) << 10, word);
    auto_driver::set_scene(None, Some(status), STAGE_SETUP as u64);
    super::capture::note_stage(status);
}

pub(super) fn install() {
    // Require all resets and entries together. Unknown/overlapping patches leave
    // the tracker not ready, so the exported online context never reports
    // online. Auto also stays off this path: a failed guard does not rewrite
    // the request.
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
    // A failed guard leaves lobby practice classified as before (bit 9 clear).
    super::scene_mode::init();
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
