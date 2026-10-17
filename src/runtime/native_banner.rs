//! CSS and arena title text. Hooks do not read buttons. Settings edges stay in the overlay.
//!
//! CSS `0x1a12f60` is replaced only when its entry words match the 13.0.5 image.
//! The original runs once. Arena inline sites use the same kind of word guard.
//! Style is stored once per pane address, vtable, name, and scene generation.

use super::{auto_driver, hooks::preflight, memory, pia_rtt, record, renderer, settings, Code};
use crate::menu::NativeSurface;
use crate::native_banner::{
    self, BannerPaint, FontMetrics, Observe, CSS_BACKGROUND_STEPS, CSS_TITLE_WALK, TEXT_CAP,
};
use skyline::hooks::{getRegionAddress, A64HookFunction, InlineCtx, Region};
use skyline::nn::ui2d::{Material, MaterialFlags, Pane, PaneFlag, PaneNode, TextBox, TextBoxFlag};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::Mutex;

pub(super) const CSS_UPDATE: usize = 0x1a12f60;
const ARENA_INIT: usize = 0x1887b1c;
const ARENA_UPDATE: usize = 0x18881f0;
const SET_TEXT: usize = 0x37a28a0;

const CSS_WORDS: [(usize, u32); 3] = [
    (CSS_UPDATE, 0xfc18_0fea),
    (CSS_UPDATE + 4, 0x6d01_23e9),
    (CSS_UPDATE + 8, 0xa902_6ffc),
];
const ARENA_INIT_WORDS: [(usize, u32); 3] = [
    (ARENA_INIT, 0x947b_c9d9),
    (ARENA_INIT + 4, 0xf940_1fe0),
    (ARENA_INIT + 8, 0xd001_cda8),
];
const ARENA_UPDATE_WORDS: [(usize, u32); 3] = [
    (ARENA_UPDATE, 0xd101_03ff),
    (ARENA_UPDATE + 4, 0xa901_57f6),
    (ARENA_UPDATE + 8, 0xa902_4ff4),
];
const SETTER_WORDS: [(usize, u32); 3] = [
    (SET_TEXT, 0xf81d_0ffc),
    (SET_TEXT + 4, 0xa901_4ff4),
    (SET_TEXT + 8, 0xa902_7bfd),
];

const REJECT_CHAIN: u8 = 1;
const REJECT_PANE: u8 = 2;
const REJECT_TEXT: u8 = 3;
const REJECT_FIT: u8 = 4;
const REJECT_ROOM: u8 = 5;

static SCENE_GEN: AtomicU64 = AtomicU64::new(0);
static SURFACE_KIND: AtomicU8 = AtomicU8::new(0);
static SURFACE_GEN: AtomicU64 = AtomicU64::new(0);
static LAST_REJECT: AtomicU64 = AtomicU64::new(u64::MAX);
static SETTER_READY: AtomicBool = AtomicBool::new(false);
static OPEN_LOGGED: AtomicU64 = AtomicU64::new(u64::MAX);
static STATE: Mutex<BannerState> = Mutex::new(BannerState::empty(0));

const _: () = assert!(std::mem::offset_of!(Pane, link) == 8);
const _: () = assert!(std::mem::offset_of!(PaneNode, prev) == 0);

#[skyline::from_offset(0x37a28a0)]
fn set_text_string(pane: *mut Pane, text: *const u8);

#[derive(Clone, Copy)]
struct Slot {
    used: bool,
    address: u64,
    vtable: u64,
    generation: u64,
    name: [u8; 24],
    name_len: u8,
    text: [u8; TEXT_CAP],
    text_len: u16,
    font: Option<FontMetrics>,
    colors: [[u8; 4]; 2],
    material: Option<[[u8; 4]; 2]>,
    pos_y: Option<f32>,
    telemetry_applied: bool,
    logged: bool,
}

struct BannerState {
    generation: u64,
    slots: [Slot; 4],
    arena: usize,
    arena_vtable: u64,
    arena_name: [u8; 24],
    arena_len: u8,
    room: [u8; 20],
    room_len: u8,
    background_used: bool,
    background_address: u64,
    background_generation: u64,
    background_visible: bool,
    arena_reject_logged: bool,
}

const fn empty_slot() -> Slot {
    Slot {
        used: false,
        address: 0,
        vtable: 0,
        generation: 0,
        name: [0; 24],
        name_len: 0,
        text: [0; TEXT_CAP],
        text_len: 0,
        font: None,
        colors: [[0; 4]; 2],
        material: None,
        pos_y: None,
        telemetry_applied: false,
        logged: false,
    }
}

impl BannerState {
    const fn empty(generation: u64) -> Self {
        Self {
            generation,
            slots: [empty_slot(); 4],
            arena: 0,
            arena_vtable: 0,
            arena_name: [0; 24],
            arena_len: 0,
            room: [0; 20],
            room_len: 0,
            background_used: false,
            background_address: 0,
            background_generation: 0,
            background_visible: false,
            arena_reject_logged: false,
        }
    }
}

pub(super) fn install() {
    unsafe {
        SETTER_READY.store(setter_available(), Ordering::Release);
        if !SETTER_READY.load(Ordering::Acquire) {
            skyline::println!("[Deluxe] native text setter not used at {:x}", SET_TEXT);
        }
        if words_ok(&CSS_WORDS) && preflight(CSS_UPDATE) {
            skyline::install_hook!(deluxe_rewrite_css_banner);
            record(Code::HookInstalled, CSS_UPDATE as u64, 0);
        } else {
            skyline::println!(
                "[Deluxe] native CSS title hook not installed at {:x}",
                CSS_UPDATE
            );
        }
        if words_ok(&ARENA_INIT_WORDS)
            && words_ok(&ARENA_UPDATE_WORDS)
            && preflight(ARENA_INIT)
            && preflight(ARENA_UPDATE)
        {
            skyline::install_hook!(deluxe_rewrite_arena_room_init);
            skyline::install_hook!(deluxe_rewrite_arena_room_update);
            record(Code::HookInstalled, ARENA_INIT as u64, 0);
            record(Code::HookInstalled, ARENA_UPDATE as u64, 0);
        } else {
            skyline::println!("[Deluxe] native arena title hooks not installed");
        }
        install_local_count();
    }
}

const LOCAL_COUNT: usize = 0x1bd7a80;
const _: () = assert!(LOCAL_COUNT == crate::native_banner::LOCAL_UPDATE);
const LDN_INIT_ID: u64 = 0x1bd7_0001;
const LDN_STATE_ID: u64 = 0x1bd7_0002;
const LDN_SCAN_ID: u64 = 0x1bd7_0003;

static LOCAL_PANE: AtomicUsize = AtomicUsize::new(0);
static LOCAL_GEN: AtomicU64 = AtomicU64::new(0);
static SCENE_WORD_OK: AtomicBool = AtomicBool::new(false);
static SCAN_COUNT: AtomicU8 = AtomicU8::new(0);
static LDN_READY: AtomicBool = AtomicBool::new(false);
static ORIGINAL_INIT: AtomicUsize = AtomicUsize::new(0);
static ORIGINAL_SCAN: AtomicUsize = AtomicUsize::new(0);
static GET_STATE: AtomicUsize = AtomicUsize::new(0);

fn install_local_count() {
    let scene_ok =
        unsafe { renderer::expect(native_banner::LOCAL_SCENE, native_banner::LOCAL_SCENE_WORD) };
    SCENE_WORD_OK.store(scene_ok, Ordering::Release);
    install_ldn();
    let update_ok = unsafe {
        words_ok(&[
            (LOCAL_COUNT, native_banner::LOCAL_UPDATE_WORDS[0]),
            (LOCAL_COUNT + 4, native_banner::LOCAL_UPDATE_WORDS[1]),
            (LOCAL_COUNT + 8, native_banner::LOCAL_UPDATE_WORDS[2]),
            (LOCAL_COUNT + 12, native_banner::LOCAL_UPDATE_WORDS[3]),
        ]) && preflight(LOCAL_COUNT)
    };
    if update_ok {
        skyline::install_hook!(deluxe_rewrite_local_count);
        record(Code::HookInstalled, LOCAL_COUNT as u64, 0);
    } else {
        record(Code::HookRejected, LOCAL_COUNT as u64, 0);
        skyline::println!("[Deluxe] local-wireless count hook not installed at {LOCAL_COUNT:x}");
    }
}

fn install_ldn() {
    let initialize = lookup_symbol(native_banner::LDN_INITIALIZE);
    let get_state = lookup_symbol(native_banner::LDN_GET_STATE);
    let scan = lookup_symbol(native_banner::LDN_SCAN);
    record(Code::Compatibility, LDN_INIT_ID, initialize as u64);
    record(Code::Compatibility, LDN_STATE_ID, get_state as u64);
    record(Code::Compatibility, LDN_SCAN_ID, scan as u64);
    if !native_banner::ldn_symbols_usable(initialize, get_state, scan)
        || !symbol_mapped(initialize)
        || !symbol_mapped(get_state)
        || !symbol_mapped(scan)
    {
        record(Code::HookRejected, LDN_INIT_ID, 0);
        skyline::println!("[Deluxe] LDN count symbols unavailable; original calls kept");
        return;
    }
    unsafe {
        if !hook_symbol(
            scan,
            on_ldn_scan as *const () as *const c_void,
            &ORIGINAL_SCAN,
            LDN_SCAN_ID,
        ) || !hook_symbol(
            initialize,
            on_ldn_initialize as *const () as *const c_void,
            &ORIGINAL_INIT,
            LDN_INIT_ID,
        ) {
            return;
        }
    }
    GET_STATE.store(get_state, Ordering::Release);
    LDN_READY.store(true, Ordering::Release);
}

fn symbol_mapped(address: usize) -> bool {
    address != 0 && unsafe { memory::accessible(address, 4, false) }
}

unsafe fn hook_symbol(
    target: usize,
    replacement: *const c_void,
    original: &AtomicUsize,
    site: u64,
) -> bool {
    if !symbol_mapped(target) {
        record(Code::HookRejected, site, 0);
        return false;
    }
    let before = *(target as *const u32);
    record(Code::HookEntryWord, site, u64::from(before));
    let branch = before & 0xfc00_0000 == 0x1400_0000;
    if before == 0 || before == 0x5800_0051 || branch {
        record(Code::HookRejected, site, u64::from(before));
        return false;
    }
    let mut trampoline: *mut c_void = std::ptr::null_mut();
    A64HookFunction(target as *const c_void, replacement, &mut trampoline);
    if trampoline.is_null() {
        record(Code::HookRejected, site, 1);
        return false;
    }
    original.store(trampoline as usize, Ordering::Release);
    record(Code::HookInstalled, site, trampoline as u64);
    true
}

/// `nn::ldn::Initialize`. Pinned nnsdk 0.6.2 returns `nn::Result` (`u32` in W0).
/// The stored scan count is cleared, then the original runs once and its W0 is returned.
unsafe extern "C" fn on_ldn_initialize() -> u32 {
    SCAN_COUNT.store(0, Ordering::Release);
    let original = ORIGINAL_INIT.load(Ordering::Acquire);
    if original == 0 {
        return 1;
    }
    let original: unsafe extern "C" fn() -> u32 = std::mem::transmute(original);
    original()
}

/// `nn::ldn::Scan(NetworkInfo*, int*, int, ScanFilter const&, int)`.
/// The original runs once. Its W0 is kept across the count store.
/// The count is stored only from a readable `found` pointer.
unsafe extern "C" fn on_ldn_scan(
    info: *mut u8,
    found: *mut i32,
    max_networks: i32,
    filter: *const u8,
    extra: i32,
) -> u32 {
    let original = ORIGINAL_SCAN.load(Ordering::Acquire);
    if original == 0 {
        return 1;
    }
    let original: unsafe extern "C" fn(*mut u8, *mut i32, i32, *const u8, i32) -> u32 =
        std::mem::transmute(original);
    let result = original(info, found, max_networks, filter, extra);
    if result == 0
        && !found.is_null()
        && (found as usize).is_multiple_of(4)
        && memory::accessible(found as usize, 4, false)
    {
        let found = std::ptr::read_volatile(found);
        SCAN_COUNT.store(
            native_banner::store_scan_count(found.min(max_networks.max(0))),
            Ordering::Release,
        );
    } else {
        SCAN_COUNT.store(0, Ordering::Release);
    }
    result
}

unsafe fn network_state() -> Option<u32> {
    let address = GET_STATE.load(Ordering::Acquire);
    if address == 0 {
        return None;
    }
    let get_state: unsafe extern "C" fn() -> u32 = std::mem::transmute(address);
    Some(get_state())
}

/// Scene entry already installed at `0x1bd45e0`. Drop the previous pane, then
/// capture this one. Does not install another patch.
pub(super) unsafe fn enter_local_scene(base: u64) {
    drop_scene();
    if !SCENE_WORD_OK.load(Ordering::Acquire) {
        record(Code::HookRejected, native_banner::LOCAL_SCENE as u64, 0);
        return;
    }
    let Some(address) =
        crate::ui_safety::linked_pane(base as usize, 8, |addr| unsafe { read_ptr(addr) })
    else {
        record(Code::HookRejected, native_banner::LOCAL_SCENE as u64, 1);
        return;
    };
    if !address.is_multiple_of(8)
        || !memory::accessible(address, std::mem::size_of::<TextBox>(), false)
    {
        record(
            Code::PaneRejected,
            native_banner::LOCAL_SCENE as u64,
            address as u64,
        );
        return;
    }
    let pane = &*(address as *const TextBox);
    let text_base = getRegionAddress(Region::Text) as u64;
    let Some(name) = copy_name(&pane.pane) else {
        record(
            Code::PaneRejected,
            native_banner::LOCAL_SCENE as u64,
            address as u64,
        );
        return;
    };
    let identity = native_banner::local_pane_identity(&name, pane.pane.vtable, text_base);
    if !identity || !vtable_ok(pane.pane.vtable) {
        record(
            Code::PaneRejected,
            native_banner::LOCAL_SCENE as u64,
            pane.pane.vtable,
        );
        return;
    }
    LOCAL_PANE.store(address, Ordering::Release);
    LOCAL_GEN.store(SCENE_GEN.load(Ordering::Acquire), Ordering::Release);
    let (first, second) = native_banner::name_words(&name);
    record(Code::PaneField, first, second);
}

#[skyline::hook(offset = LOCAL_COUNT, inline)]
unsafe fn deluxe_rewrite_local_count(_: &InlineCtx) {
    let address = LOCAL_PANE.load(Ordering::Acquire);
    let captured = LOCAL_GEN.load(Ordering::Acquire);
    let scene = SCENE_GEN.load(Ordering::Acquire);
    if address == 0
        || !address.is_multiple_of(8)
        || !memory::accessible(address, std::mem::size_of::<TextBox>(), true)
    {
        return;
    }
    let pane = &*(address as *const TextBox);
    let text_base = getRegionAddress(Region::Text) as u64;
    let Some(name) = copy_name(&pane.pane) else {
        return;
    };
    let identity = native_banner::local_pane_identity(&name, pane.pane.vtable, text_base)
        && vtable_ok(pane.pane.vtable);
    if captured == 0 || scene != captured || !identity {
        return;
    }
    note_surface(NativeSurface::LocalLobby);
    if !native_banner::local_count_active(
        LDN_READY.load(Ordering::Acquire),
        scene,
        captured,
        identity,
    ) {
        return;
    }
    let Some(state) = network_state() else {
        return;
    };
    let config = settings();
    let text = native_banner::local_wireless_text(
        config.streamer,
        &native_banner::local_latency_label(config.input_buffer),
        state == native_banner::LDN_INITIALIZED,
        SCAN_COUNT.load(Ordering::Acquire),
    );
    let _ = write_text(&mut *(address as *mut TextBox), &text);
}

fn lookup_symbol(symbol: &[u8]) -> usize {
    let mut address = 0usize;
    let result = unsafe { skyline::nn::ro::LookupSymbol(&mut address, symbol.as_ptr()) };
    if result == 0 {
        address
    } else {
        0
    }
}

/// Drops arena and title ownership. Scene hooks call this on entry.
pub(super) fn drop_scene() {
    LOCAL_PANE.store(0, Ordering::Release);
    LOCAL_GEN.store(0, Ordering::Release);
    clear_surface();
    LAST_REJECT.store(u64::MAX, Ordering::Relaxed);
    OPEN_LOGGED.store(u64::MAX, Ordering::Relaxed);
    let generation = SCENE_GEN.fetch_add(1, Ordering::Release) + 1;
    if let Ok(mut state) = STATE.try_lock() {
        *state = BannerState::empty(generation);
    }
}

#[skyline::hook(offset = CSS_UPDATE)]
unsafe fn deluxe_rewrite_css_banner(arg: u64) {
    original!()(arg);
    super::capture::note_css_frame();
    paint_css(arg);
}

#[skyline::hook(offset = ARENA_INIT, inline)]
unsafe fn deluxe_rewrite_arena_room_init(ctx: &InlineCtx) {
    let base = ctx.registers[0].x() as usize;
    let Some(address) = crate::ui_safety::linked_pane(base, 8, |addr| unsafe { read_ptr(addr) })
    else {
        clear_arena_state();
        return;
    };
    let Some(name) = pane_name_at(address) else {
        clear_arena_state();
        return;
    };
    if !native_banner::is_arena_room_name(&name) {
        note_arena_name(&name);
        clear_arena_state();
        return;
    }
    let Some(pane) = open_pane(address, false, ARENA_INIT as u64) else {
        clear_arena_state();
        return;
    };
    let room = read_room_id(ctx.registers[3].x() as usize);
    let Ok(mut state) = STATE.try_lock() else {
        return;
    };
    sync_generation(&mut state);
    let Some(name_len) = store_bytes(&mut state.arena_name, name.as_bytes()) else {
        clear_arena(&mut state);
        return;
    };
    state.arena_len = name_len;
    state.arena = pane as usize;
    state.arena_vtable = (*pane).pane.vtable;
    state.room_len = 0;
    if let Some(id) = room {
        if let Some(len) = store_bytes(&mut state.room, id.as_bytes()) {
            state.room_len = len;
        }
    }
}

#[skyline::hook(offset = ARENA_UPDATE, inline)]
unsafe fn deluxe_rewrite_arena_room_update(_: &InlineCtx) {
    let parts = arena_parts();
    let Ok(mut state) = STATE.try_lock() else {
        return;
    };
    sync_generation(&mut state);
    let address = state.arena;
    let saved_vtable = state.arena_vtable;
    if address == 0 {
        return;
    }
    let Some(name) = pane_name_at(address) else {
        clear_arena(&mut state);
        return;
    };
    if !names_equal(&state.arena_name, state.arena_len, &name) {
        clear_arena(&mut state);
        return;
    }
    let Some(pane) = open_pane(address, false, ARENA_UPDATE as u64) else {
        clear_arena(&mut state);
        return;
    };
    if (*pane).pane.vtable != saved_vtable {
        clear_arena(&mut state);
        return;
    }
    note_surface(NativeSurface::Arena);
    if state.room_len == 0 {
        note_reject(ARENA_UPDATE as u64, REJECT_ROOM);
        return;
    }
    let room_len = usize::from(state.room_len).min(state.room.len());
    let room = std::str::from_utf8(&state.room[..room_len])
        .unwrap_or("")
        .to_owned();
    if room.is_empty() {
        note_reject(ARENA_UPDATE as u64, REJECT_ROOM);
        return;
    }
    let (telemetry, ping, remote_lines) = arena_text(&parts, &room);
    paint_one(
        &mut state,
        pane,
        false,
        parts.streamer,
        &telemetry,
        ping,
        remote_lines,
    );
}

unsafe fn paint_css(arg: u64) {
    let Some(address) =
        crate::ui_safety::banner_address(arg as usize, |addr| unsafe { read_ptr(addr) })
    else {
        note_reject(CSS_UPDATE as u64, REJECT_CHAIN);
        return;
    };
    let Some(first) = open_pane(address, true, CSS_UPDATE as u64) else {
        return;
    };
    note_surface(NativeSurface::Css);
    super::online::observe_css_scene();
    let first_pane = first.cast::<Pane>();
    let second = walk_links(first_pane, &CSS_TITLE_WALK).and_then(|pane| unsafe {
        let address = pane as usize;
        if address == first as usize {
            return None;
        }
        open_pane(address, true, CSS_UPDATE as u64)
    });
    let background = second.and_then(|_| unsafe {
        walk_links(first_pane, &CSS_TITLE_WALK[..CSS_BACKGROUND_STEPS]).filter(|pane| {
            let address = *pane as usize;
            address != first as usize
        })
    });
    let (streamer, telemetry, ping, _) = snapshot();
    let Ok(mut state) = STATE.try_lock() else {
        return;
    };
    sync_generation(&mut state);
    if let Some(pane) = background {
        apply_background(&mut state, pane, streamer);
    }
    paint_one(&mut state, first, true, streamer, &telemetry, ping, 0);
    if let Some(pane) = second {
        paint_one(&mut state, pane, true, streamer, &telemetry, ping, 0);
    }
}

fn snapshot() -> (bool, String, Option<[u8; 4]>, u8) {
    let config = settings();
    if config.streamer {
        return (true, String::new(), None, 0);
    }
    let auto = auto_driver::snapshot();
    let profile = crate::auto_policy::profile_row_label(
        config.auto_profile,
        auto.ready,
        config.render_mode,
        auto.request,
    );
    let latency = native_banner::local_latency_label(config.input_buffer);
    let frame = crate::peer_transport::peer_frame();
    let show_remote = native_banner::show_native_remote(crate::peer_transport::station_connected(
        frame.selection,
    ));
    let name = if frame.peer.known {
        crate::peer_transport::name_label(&frame.peer.name, frame.peer.name_len)
            .unwrap_or("Unavailable")
    } else {
        "Unavailable"
    };
    let remote_profile = if frame.peer.known && !frame.peer.settings.is_empty() {
        frame.peer.settings
    } else {
        "---"
    };
    let remote_latency = crate::peer_transport::remote_latency_label(frame.peer.latency);
    let identity = crate::peer_transport::display_identity(frame.selection);
    let (publication, epoch, published_gen, draw_gen) = pia_rtt::current_publication();
    let ping =
        crate::pia_rtt::present_published(identity, publication, epoch, published_gen, draw_gen);
    let text = native_banner::telemetry_text(&native_banner::LineParts {
        you_profile: &profile,
        you_latency: &latency,
        show_remote,
        remote_name: name,
        remote_profile,
        remote_latency: &remote_latency,
        ping: &ping.opponent,
    });
    let remote_lines = u8::from(show_remote);
    (
        false,
        text,
        ping.color.map(native_banner::ping_bytes),
        remote_lines,
    )
}

struct ArenaLine {
    name: String,
    profile: String,
    latency: String,
    ping: String,
}

struct ArenaParts {
    streamer: bool,
    profile: String,
    latency: String,
    lines: Vec<ArenaLine>,
    pane_color: Option<[u8; 4]>,
}

fn arena_parts() -> ArenaParts {
    let config = settings();
    if config.streamer {
        return ArenaParts {
            streamer: true,
            profile: String::new(),
            latency: String::new(),
            lines: Vec::new(),
            pane_color: None,
        };
    }
    let auto = auto_driver::snapshot();
    let profile = crate::auto_policy::profile_row_label(
        config.auto_profile,
        auto.ready,
        config.render_mode,
        auto.request,
    );
    let latency = native_banner::local_latency_label(config.input_buffer);
    let (_frame, roster) = crate::peer_transport::banner_peers();
    let (board, epoch, published_gen, draw_gen) = pia_rtt::current_board();
    if roster.busy || roster.len == 0 {
        return ArenaParts {
            streamer: false,
            profile,
            latency,
            lines: Vec::new(),
            pane_color: None,
        };
    }
    let mut listed = [0u8; crate::pia_rtt::BOARD_CAP];
    let mut listed_len = 0u8;
    let mut lines = Vec::new();
    for peer in roster.peers.iter().take(usize::from(roster.len)) {
        if !peer.known || peer.station.is_none() {
            continue;
        }
        let presented = match peer.station {
            Some(index) => {
                if usize::from(listed_len) < listed.len() {
                    listed[usize::from(listed_len)] = index;
                    listed_len = listed_len.saturating_add(1);
                }
                crate::pia_rtt::present_station(&board, index, epoch, published_gen, draw_gen)
            }
            None => crate::pia_rtt::present_published(
                crate::pia_rtt::RttIdentity::Busy,
                crate::pia_rtt::Publication::unavailable(epoch),
                epoch,
                published_gen,
                draw_gen,
            ),
        };
        let name = crate::peer_transport::name_label(&peer.name, peer.name_len)
            .unwrap_or("Unavailable")
            .to_owned();
        let remote_profile = if peer.settings.is_empty() {
            "---".to_owned()
        } else {
            peer.settings.to_owned()
        };
        let ping = format!(
            "{}{}",
            presented.opponent,
            native_banner::interface_suffix(peer.interface)
        );
        lines.push(ArenaLine {
            name,
            profile: remote_profile,
            latency: crate::peer_transport::remote_latency_label(peer.latency),
            ping,
        });
    }
    let pane_color = crate::pia_rtt::aggregate_stability(
        &board,
        &listed[..usize::from(listed_len)],
        epoch,
        published_gen,
        draw_gen,
    )
    .map(crate::pia_rtt::ping_rgba)
    .map(native_banner::ping_bytes);
    ArenaParts {
        streamer: false,
        profile,
        latency,
        lines,
        pane_color,
    }
}

fn arena_text(parts: &ArenaParts, room: &str) -> (String, Option<[u8; 4]>, u8) {
    if parts.streamer {
        return (String::new(), None, 0);
    }
    let refs: Vec<native_banner::RemoteLine<'_>> = parts
        .lines
        .iter()
        .take(usize::from(native_banner::ARENA_REMOTE_CAP))
        .map(|line| native_banner::RemoteLine {
            name: line.name.as_str(),
            profile: line.profile.as_str(),
            latency: line.latency.as_str(),
            ping: line.ping.as_str(),
        })
        .collect();
    let (text, shown) = native_banner::arena_status(room, &parts.profile, &parts.latency, &refs);
    (text, parts.pane_color, shown)
}

fn note_surface(kind: NativeSurface) {
    SURFACE_KIND.store(kind as u8, Ordering::Relaxed);
    SURFACE_GEN.fetch_add(1, Ordering::Release);
}

pub(super) fn surface_sample() -> (u64, u8) {
    let generation = SURFACE_GEN.load(Ordering::Acquire);
    let kind = SURFACE_KIND.load(Ordering::Relaxed);
    (generation, kind)
}

fn clear_surface() {
    SURFACE_KIND.store(0, Ordering::Relaxed);
    SURFACE_GEN.store(0, Ordering::Release);
}

unsafe fn paint_one(
    state: &mut BannerState,
    pane: *mut TextBox,
    titles_only: bool,
    streamer: bool,
    telemetry: &str,
    ping: Option<[u8; 4]>,
    remote_lines: u8,
) {
    let site = if titles_only {
        CSS_UPDATE
    } else {
        ARENA_UPDATE
    } as u64;
    if SCENE_GEN.load(Ordering::Acquire) != state.generation {
        let generation = SCENE_GEN.load(Ordering::Acquire);
        *state = BannerState::empty(generation);
        return;
    }
    let address = pane as u64;
    if !memory::accessible(pane as usize, std::mem::size_of::<TextBox>(), true)
        || !vtable_ok((*pane).pane.vtable)
    {
        note_reject(site, REJECT_PANE);
        return;
    }
    let vtable = (*pane).pane.vtable;
    let Some(name) = copy_name(&(*pane).pane) else {
        note_reject(site, REJECT_PANE);
        return;
    };
    let name_ok = if titles_only {
        crate::ui_safety::is_title_name(name.as_bytes())
    } else {
        native_banner::is_arena_room_name(&name)
    };
    if !name_ok {
        note_reject(site, REJECT_PANE);
        return;
    }
    let Some(current) = read_current(&*pane) else {
        note_reject(site, REJECT_TEXT);
        return;
    };
    let generation = state.generation;
    let found = find_slot(&state.slots, address, vtable, generation, &name);
    let action = native_banner::observe_style(found.is_some(), found.is_some(), &current);
    let index = match action {
        Observe::Keep | Observe::RefreshText => match found {
            Some(index) => index,
            None => return,
        },
        Observe::Replace => match found
            .or_else(|| address_slot(&state.slots, address, generation))
            .or_else(|| free_slot(&state.slots))
        {
            Some(index) => index,
            None => return,
        },
    };
    match action {
        Observe::Keep => {}
        Observe::RefreshText => {
            let _ = store_text(&mut state.slots[index], &current);
        }
        Observe::Replace => {
            let font = capture_font(&*pane);
            let colors = (*pane).text_colors;
            let material = read_material(&*pane);
            let slot = &mut state.slots[index];
            slot.used = true;
            slot.address = address;
            slot.vtable = vtable;
            slot.generation = generation;
            slot.font = font;
            slot.colors = colors;
            slot.material = material;
            slot.pos_y = if titles_only {
                None
            } else {
                let y = (*pane).pane.pos_y;
                y.is_finite().then_some(y)
            };
            slot.telemetry_applied = false;
            slot.logged = false;
            let Some(name_len) = store_bytes(&mut slot.name, name.as_bytes()) else {
                *slot = empty_slot();
                return;
            };
            slot.name_len = name_len;
            if !store_text(slot, &current) {
                *slot = empty_slot();
                return;
            }
        }
    }
    let saved_text = match slot_text(&state.slots[index]) {
        Some(text) => text.to_owned(),
        None => return,
    };
    let saved_font = state.slots[index].font;
    let saved_colors = state.slots[index].colors;
    let saved_material = state.slots[index].material;
    let live_font = capture_font(&*pane);
    let material_matches = match saved_material {
        Some(saved) => read_material(&*pane) == Some(saved),
        None => true,
    };
    let outcome = native_banner::banner_paint(&native_banner::PaintInput {
        same_identity: true,
        captured: true,
        telemetry_applied: state.slots[index].telemetry_applied,
        streamer,
        current: &current,
        saved_text: &saved_text,
        saved_font,
        live_font,
        saved_colors,
        saved_material,
        material_matches,
        telemetry,
        ping,
        arena: !titles_only,
        saved_pos_y: state.slots[index].pos_y,
        remote_lines,
    });
    state.slots[index].telemetry_applied = outcome.telemetry_applied;
    let BannerPaint::Write {
        text,
        font,
        colors,
        material,
        pos_y,
    } = outcome.paint
    else {
        return;
    };
    if SCENE_GEN.load(Ordering::Acquire) != state.generation {
        let generation = SCENE_GEN.load(Ordering::Acquire);
        *state = BannerState::empty(generation);
        return;
    }
    let needs_text = current != text;
    let wrote = !needs_text || write_text(&mut *pane, &text);
    if !wrote && !streamer {
        note_reject(site, REJECT_FIT);
        return;
    }
    if wrote || streamer {
        apply_style(&mut *pane, font, colors, material);
        if let Some(y) = pos_y {
            apply_pos_y(&mut *pane, y);
        }
    }
    if wrote && !state.slots[index].logged {
        state.slots[index].logged = true;
        let (first, second) = native_banner::name_words(&name);
        record(Code::PaneAccepted, first, second);
        record(Code::PaneField, address, vtable);
    }
}

unsafe fn apply_background(state: &mut BannerState, pane: *mut Pane, streamer: bool) {
    let address = pane as usize;
    if !memory::accessible(address, std::mem::size_of::<Pane>(), true)
        || !vtable_ok((*pane).vtable)
        || copy_name(&*pane).is_none()
    {
        return;
    }
    let address = pane as u64;
    if !state.background_used
        || state.background_address != address
        || state.background_generation != state.generation
    {
        state.background_used = true;
        state.background_address = address;
        state.background_generation = state.generation;
        state.background_visible = ((*pane).flags & (1 << PaneFlag::Visible as u8)) != 0;
    }
    let visible = if streamer {
        state.background_visible
    } else {
        false
    };
    if visible {
        (*pane).flags |= 1 << PaneFlag::Visible as u8;
    } else {
        (*pane).flags &= !(1 << PaneFlag::Visible as u8);
    }
}

unsafe fn walk_links(start: *mut Pane, steps: &[native_banner::PaneLink]) -> Option<*mut Pane> {
    let mut current = start;
    for step in steps {
        let next = match step {
            native_banner::PaneLink::Parent => parent_of(current)?,
            native_banner::PaneLink::Prev => link_prev(current)?,
        };
        if next == start || next == current {
            return None;
        }
        current = next;
    }
    (current != start).then_some(current)
}

unsafe fn parent_of(pane: *mut Pane) -> Option<*mut Pane> {
    let parent = (*pane).parent;
    if parent.is_null() {
        return None;
    }
    let address = parent as usize;
    if !address.is_multiple_of(8)
        || !memory::accessible(address, std::mem::size_of::<Pane>(), false)
    {
        return None;
    }
    if !vtable_ok((*parent).vtable) {
        return None;
    }
    Some(parent)
}

unsafe fn link_prev(pane: *mut Pane) -> Option<*mut Pane> {
    let link = (pane as usize).checked_add(std::mem::offset_of!(Pane, link))?;
    let node = read_ptr(link.checked_add(std::mem::offset_of!(PaneNode, prev))?)?;
    pane_from_link(node)
}

unsafe fn pane_from_link(node: usize) -> Option<*mut Pane> {
    if node < 0x10000 || !node.is_multiple_of(8) {
        return None;
    }
    let address = node.checked_sub(std::mem::offset_of!(Pane, link))?;
    if !address.is_multiple_of(8)
        || !memory::accessible(address, std::mem::size_of::<Pane>(), false)
    {
        return None;
    }
    let pane = address as *mut Pane;
    let list = address.checked_add(std::mem::offset_of!(Pane, children_list))?;
    let next = read_ptr(list.checked_add(std::mem::offset_of!(PaneNode, next))?)?;
    let prev = read_ptr(list.checked_add(std::mem::offset_of!(PaneNode, prev))?)?;
    if next == 0 && prev == 0 {
        return None;
    }
    if !vtable_ok((*pane).vtable) {
        return None;
    }
    Some(pane)
}

unsafe fn open_pane(address: usize, titles_only: bool, site: u64) -> Option<*mut TextBox> {
    let (mapped, remain) = mapped_span(address);
    let reject = |guard: u8, vtable: u64, name: [u8; 16]| unsafe {
        note_open(site, address as u64, guard, vtable, mapped, remain, name);
    };
    if !address.is_multiple_of(8)
        || !memory::accessible(address, std::mem::size_of::<Pane>(), false)
    {
        reject(1, 0, [0; 16]);
        return None;
    }
    let pane = &*(address as *const Pane);
    let vtable = pane.vtable;
    let raw = raw_name(pane);
    if !vtable_ok(vtable) {
        reject(3, vtable, raw);
        return None;
    }
    let Some(name) = copy_name(pane) else {
        reject(2, vtable, raw);
        return None;
    };
    let allowed = if titles_only {
        crate::ui_safety::is_title_name(name.as_bytes())
    } else {
        native_banner::is_arena_room_name(&name)
    };
    if !allowed {
        reject(4, vtable, raw);
        return None;
    }
    if !memory::accessible(address, std::mem::size_of::<TextBox>(), true) {
        reject(5, vtable, raw);
        return None;
    }
    Some(address as *mut TextBox)
}

unsafe fn mapped_span(address: usize) -> (u64, u64) {
    let Some((base, size)) = memory::region(address) else {
        return (0, 0);
    };
    let mapped = u64::try_from(size).unwrap_or(u64::MAX);
    let remain =
        u64::try_from(base.saturating_add(size).saturating_sub(address)).unwrap_or(u64::MAX);
    (mapped, remain)
}

fn raw_name(pane: &Pane) -> [u8; 16] {
    let mut raw = [0u8; 16];
    for (index, byte) in raw.iter_mut().enumerate() {
        *byte = pane.name[index];
    }
    raw
}

unsafe fn note_open(
    site: u64,
    address: u64,
    guard: u8,
    vtable: u64,
    mapped: u64,
    remain: u64,
    name: [u8; 16],
) {
    let key = address ^ site.rotate_left(17) ^ u64::from(guard);
    if OPEN_LOGGED.swap(key, Ordering::Relaxed) == key {
        return;
    }
    let text = getRegionAddress(Region::Text) as u64;
    let data = getRegionAddress(Region::Data) as u64;
    record(
        Code::PaneRejected,
        site,
        (u64::from(guard) << 8) | u64::from(REJECT_PANE),
    );
    record(Code::PaneField, address, vtable);
    record(Code::PaneField, mapped, remain);
    record(
        Code::PaneField,
        pack_half(&name[..8]),
        pack_half(&name[8..]),
    );
    record(Code::PaneField, text, data);
}

fn pack_half(bytes: &[u8]) -> u64 {
    let mut word = 0u64;
    for (index, byte) in bytes.iter().take(8).enumerate() {
        word |= u64::from(*byte) << (index * 8);
    }
    word
}

unsafe fn setter_available() -> bool {
    let base = getRegionAddress(Region::Text) as usize;
    let Some(entry) = base.checked_add(SET_TEXT) else {
        record(Code::HookRejected, SET_TEXT as u64, 0);
        return false;
    };
    if !memory::accessible(entry, 0x1c, false) {
        record(Code::HookRejected, SET_TEXT as u64, 0);
        return false;
    }
    let mut words = [0u32; 7];
    for (index, word) in words.iter_mut().enumerate() {
        *word = *((entry + index * 4) as *const u32);
    }
    let entry_pair = [words[0], words[1]];
    let tail = [words[4], words[5], words[6]];
    if native_banner::chained_text_setter(&entry_pair, &tail) {
        record(Code::HookEntryWord, SET_TEXT as u64, u64::from(words[0]));
        record(
            Code::HookEntryWord,
            (SET_TEXT + 0x10) as u64,
            u64::from(words[4]),
        );
        record(Code::Compatibility, SET_TEXT as u64, u64::from(words[0]));
        skyline::println!(
            "[Deluxe] native text setter chains the existing hook at {:x}",
            SET_TEXT
        );
        return true;
    }
    words_ok(&SETTER_WORDS) && preflight(SET_TEXT)
}

unsafe fn pane_name_at(address: usize) -> Option<String> {
    if !address.is_multiple_of(8)
        || !memory::accessible(address, std::mem::size_of::<Pane>(), false)
    {
        return None;
    }
    copy_name(&*(address as *const Pane))
}

unsafe fn read_current(tb: &TextBox) -> Option<String> {
    let cap = usize::from(tb.text_buf_len);
    if tb.text_buf.is_null() || cap == 0 || cap > native_banner::PANE_SCAN_MAX {
        return None;
    }
    let declared = usize::from(tb.text_len);
    if declared > cap || declared > TEXT_CAP {
        return None;
    }
    let scan = if declared == 0 {
        cap.min(TEXT_CAP)
    } else {
        (declared + 1).min(cap)
    };
    if scan == 0 || scan > TEXT_CAP + 1 {
        return None;
    }
    let byte_len = if tb.is_utf8 {
        scan
    } else if !(tb.text_buf as usize).is_multiple_of(2) {
        return None;
    } else {
        scan.saturating_mul(2)
    };
    if byte_len == 0 || !memory::accessible(tb.text_buf as usize, byte_len, false) {
        return None;
    }
    if tb.is_utf8 {
        let bytes = std::slice::from_raw_parts(tb.text_buf as *const u8, scan);
        native_banner::decode_utf8_capped(bytes)
    } else {
        let units = std::slice::from_raw_parts(tb.text_buf as *const u16, scan);
        native_banner::decode_utf16_capped(units)
    }
}

unsafe fn read_room_id(address: usize) -> Option<String> {
    if !address.is_multiple_of(2)
        || !memory::accessible(address, native_banner::ROOM_ID_UNITS * 2, false)
    {
        return None;
    }
    let units = std::slice::from_raw_parts(address as *const u16, native_banner::ROOM_ID_UNITS);
    let mut copy = [0u16; native_banner::ROOM_ID_UNITS];
    copy.copy_from_slice(units);
    native_banner::decode_room_id(&copy)
}

unsafe fn write_text(tb: &mut TextBox, text: &str) -> bool {
    let Some(text) = native_banner::setter_utf8(text) else {
        return false;
    };
    if !SETTER_READY.load(Ordering::Acquire) {
        return false;
    }
    // Four trailing zeros keep a mistaken 3-byte read inside this buffer.
    let mut raw = [0u8; TEXT_CAP + 4];
    raw[..text.len()].copy_from_slice(text.as_bytes());
    set_text_string(
        std::ptr::from_mut::<TextBox>(tb).cast::<Pane>(),
        raw.as_ptr(),
    );
    tb.bits |= 1u16 << u32::from(TextBoxFlag::IsPTDirty as u8);
    true
}

fn apply_pos_y(tb: &mut TextBox, y: f32) {
    if !y.is_finite() {
        return;
    }
    tb.pane.pos_y = y;
    tb.pane.flags |= 1u8 << u32::from(PaneFlag::IsGlobalMatrixDirty as u8);
}

fn apply_style(
    tb: &mut TextBox,
    font: Option<FontMetrics>,
    colors: [[u8; 4]; 2],
    material: Option<[[u8; 4]; 2]>,
) {
    tb.text_colors = colors;
    tb.bits |= 1u16 << u32::from(TextBoxFlag::IsPTDirty as u8);
    if let Some(font) = font {
        tb.font_size_x = font.font_x;
        tb.font_size_y = font.font_y;
        tb.line_space = font.line_space;
        tb.char_space = font.char_space;
        tb.text_position = font.text_position;
        tb.pane.flags |= 1u8 << u32::from(PaneFlag::IsGlobalMatrixDirty as u8);
    }
    if let Some(colors) = material {
        unsafe {
            let _ = write_material(tb, colors);
        }
    }
}

unsafe fn read_material(tb: &TextBox) -> Option<[[u8; 4]; 2]> {
    let material = tb.p_material;
    if material.is_null()
        || !memory::accessible(material as usize, std::mem::size_of::<Material>(), false)
    {
        return None;
    }
    let material = &*material;
    if material_is_float(material.m_flag) {
        return None;
    }
    Some(material.m_colors.byte_color)
}

unsafe fn write_material(tb: &mut TextBox, colors: [[u8; 4]; 2]) -> bool {
    let material = tb.p_material;
    if material.is_null()
        || !memory::accessible(material as usize, std::mem::size_of::<Material>(), true)
    {
        return false;
    }
    let material = &mut *material;
    if material_is_float(material.m_flag) {
        return false;
    }
    material.m_colors.byte_color = colors;
    true
}

fn material_is_float(flag: u8) -> bool {
    let bits = (1u8 << MaterialFlags::BlackColorFloat as u8)
        | (1u8 << MaterialFlags::WhiteColorFloat as u8);
    flag & bits != 0
}

fn capture_font(tb: &TextBox) -> Option<FontMetrics> {
    let metrics = FontMetrics {
        font_x: tb.font_size_x,
        font_y: tb.font_size_y,
        line_space: tb.line_space,
        char_space: tb.char_space,
        text_position: tb.text_position,
    };
    native_banner::metrics_plausible(&metrics).then_some(metrics)
}

unsafe fn copy_name(pane: &Pane) -> Option<String> {
    let mut raw = [0u8; 25];
    for (index, byte) in raw.iter_mut().enumerate() {
        *byte = pane.name[index];
    }
    native_banner::decode_name(&raw).map(str::to_owned)
}

unsafe fn read_ptr(address: usize) -> Option<usize> {
    if !address.is_multiple_of(8) || !memory::accessible(address, 8, false) {
        return None;
    }
    Some(*(address as *const usize))
}

unsafe fn vtable_ok(vtable: u64) -> bool {
    if !vtable.is_multiple_of(8) {
        return false;
    }
    let text = getRegionAddress(Region::Text) as u64;
    let rodata = getRegionAddress(Region::Rodata) as u64;
    let data = getRegionAddress(Region::Data) as u64;
    let Some(data_end) = native_banner::main_data_end(data) else {
        return false;
    };
    let address = vtable as usize;
    if !memory::accessible(address, 16, false) {
        return false;
    }
    let raw = std::ptr::read_volatile(address as *const [u8; 16]);
    let mut first = [0u8; 8];
    first.copy_from_slice(&raw[..8]);
    native_banner::main_image_vtable(vtable, u64::from_le_bytes(first), text, rodata, data_end)
}

unsafe fn words_ok(words: &[(usize, u32)]) -> bool {
    words
        .iter()
        .all(|(offset, word)| renderer::expect(*offset, *word))
}

fn sync_generation(state: &mut BannerState) {
    let generation = SCENE_GEN.load(Ordering::Acquire);
    if state.generation != generation {
        *state = BannerState::empty(generation);
    }
}

fn clear_arena_state() {
    if let Ok(mut state) = STATE.try_lock() {
        sync_generation(&mut state);
        clear_arena(&mut state);
    }
}

fn clear_arena(state: &mut BannerState) {
    state.arena = 0;
    state.arena_vtable = 0;
    state.arena_name = [0; 24];
    state.arena_len = 0;
}

fn note_arena_name(name: &str) {
    if let Ok(mut state) = STATE.try_lock() {
        sync_generation(&mut state);
        if state.arena_reject_logged {
            return;
        }
        state.arena_reject_logged = true;
        let (first, second) = native_banner::name_words(name);
        record(Code::PaneField, first, second);
    }
}

fn find_slot(
    slots: &[Slot; 4],
    address: u64,
    vtable: u64,
    generation: u64,
    name: &str,
) -> Option<usize> {
    slots.iter().position(|slot| {
        slot.used
            && slot.address == address
            && slot.vtable == vtable
            && slot.generation == generation
            && names_equal(&slot.name, slot.name_len, name)
    })
}

fn free_slot(slots: &[Slot; 4]) -> Option<usize> {
    slots.iter().position(|slot| !slot.used)
}

fn address_slot(slots: &[Slot; 4], address: u64, generation: u64) -> Option<usize> {
    slots
        .iter()
        .position(|slot| slot.used && slot.address == address && slot.generation == generation)
}

fn slot_text(slot: &Slot) -> Option<&str> {
    let len = usize::from(slot.text_len);
    if !slot.used || len > slot.text.len() {
        return None;
    }
    std::str::from_utf8(&slot.text[..len]).ok()
}

fn store_text(slot: &mut Slot, text: &str) -> bool {
    let Some(len) = u16::try_from(text.len())
        .ok()
        .filter(|len| usize::from(*len) <= TEXT_CAP)
    else {
        return false;
    };
    slot.text = [0; TEXT_CAP];
    slot.text[..text.len()].copy_from_slice(text.as_bytes());
    slot.text_len = len;
    true
}

fn store_bytes(dest: &mut [u8], bytes: &[u8]) -> Option<u8> {
    let len = u8::try_from(bytes.len()).ok()?;
    if bytes.is_empty() || usize::from(len) > dest.len() {
        return None;
    }
    dest.fill(0);
    dest[..bytes.len()].copy_from_slice(bytes);
    Some(len)
}

fn names_equal(stored: &[u8], len: u8, live: &str) -> bool {
    let len = usize::from(len);
    len == live.len() && len <= stored.len() && &stored[..len] == live.as_bytes()
}

fn note_reject(site: u64, reason: u8) {
    let packed = site.wrapping_shl(8) | u64::from(reason);
    if LAST_REJECT.swap(packed, Ordering::Relaxed) != packed {
        record(Code::PaneRejected, site, u64::from(reason));
    }
}
