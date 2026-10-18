//! CSS, arena and local-wireless pane text. Hooks do not read buttons.
//! Settings edges stay in the overlay.
//!
//! CSS `0x1a12f60` is replaced only when its entry words match the 13.0.5 image.
//! The original runs once. Arena and local-wireless inline sites use the same
//! kind of word guard. Each pane's game text and style are captured once per
//! pane address, vtable, name and scene generation; Streamer restores them.

use super::{auto_driver, hooks::preflight, memory, pia_rtt, record, renderer, settings, Code};
use crate::menu::NativeSurface;
use crate::native_banner::{
    self, FontMetrics, PaneKind, PaneLink, RemoteLine, Style, Styled, ARENA_ROOM_NAME,
    CSS_BACKGROUND_STEPS, CSS_TITLE_WALK, TEXT_CAP,
};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use skyline::nn::ui2d::{Material, MaterialFlags, Pane, PaneFlag, PaneNode, TextBox, TextBoxFlag};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::Mutex;

pub(super) const CSS_UPDATE: usize = 0x1a12f60;
const ARENA_INIT: usize = 0x1887b1c;
const ARENA_UPDATE: usize = 0x18881f0;
const SET_TEXT: usize = 0x37a28a0;
const LOCAL_COUNT: usize = native_banner::LOCAL_UPDATE;

/// 13.0.5 entry words of each site, checked before its hook is installed.
const CSS_WORDS: [u32; 3] = [0xfc18_0fea, 0x6d01_23e9, 0xa902_6ffc];
const ARENA_INIT_WORDS: [u32; 3] = [0x947b_c9d9, 0xf940_1fe0, 0xd001_cda8];
const ARENA_UPDATE_WORDS: [u32; 3] = [0xd101_03ff, 0xa901_57f6, 0xa902_4ff4];
const SETTER_WORDS: [u32; 3] = [0xf81d_0ffc, 0xa901_4ff4, 0xa902_7bfd];

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
/// The lobby pane the scene entry captured, and the scene generation it belongs to.
static LOCAL_PANE: AtomicUsize = AtomicUsize::new(0);
static LOCAL_GEN: AtomicU64 = AtomicU64::new(0);
static SCENE_WORD_OK: AtomicBool = AtomicBool::new(false);

const _: () = assert!(std::mem::offset_of!(Pane, link) == 8);
const _: () = assert!(std::mem::offset_of!(PaneNode, prev) == 0);

#[skyline::from_offset(0x37a28a0)]
fn set_text_string(pane: *mut Pane, text: *const u8);

/// One pane whose game text and style were captured this scene.
struct Slot {
    address: u64,
    vtable: u64,
    name: String,
    captured: Styled,
    /// Our telemetry was the last thing written; Streamer restores once.
    applied: bool,
    logged: bool,
}

/// The arena room pane from the init site, with the room id read there.
struct ArenaPane {
    address: usize,
    vtable: u64,
    name: String,
    room: String,
}

struct BannerState {
    generation: u64,
    slots: [Option<Slot>; 4],
    arena: Option<ArenaPane>,
    /// CSS title background address and whether the game showed it.
    background: Option<(u64, bool)>,
    arena_reject_logged: bool,
}

impl BannerState {
    const fn empty(generation: u64) -> Self {
        Self {
            generation,
            slots: [const { None }; 4],
            arena: None,
            background: None,
            arena_reject_logged: false,
        }
    }
}

pub(super) fn install() {
    unsafe {
        SETTER_READY.store(setter_available(), Ordering::Release);
        if !SETTER_READY.load(Ordering::Acquire) {
            skyline::println!("[Deluxe] native text setter not used at {SET_TEXT:x}");
        }
        if note_install(
            words_ok(CSS_UPDATE, &CSS_WORDS) && preflight(CSS_UPDATE),
            CSS_UPDATE,
        ) {
            skyline::install_hook!(deluxe_rewrite_css_banner);
        }
        let arena = words_ok(ARENA_INIT, &ARENA_INIT_WORDS)
            && words_ok(ARENA_UPDATE, &ARENA_UPDATE_WORDS)
            && preflight(ARENA_INIT)
            && preflight(ARENA_UPDATE);
        if note_install(arena, ARENA_INIT) && note_install(arena, ARENA_UPDATE) {
            skyline::install_hook!(deluxe_rewrite_arena_room_init);
            skyline::install_hook!(deluxe_rewrite_arena_room_update);
        }
        SCENE_WORD_OK.store(
            renderer::expect(native_banner::LOCAL_SCENE, native_banner::LOCAL_SCENE_WORD),
            Ordering::Release,
        );
        let local = words_ok(LOCAL_COUNT, &native_banner::LOCAL_UPDATE_WORDS);
        if note_install(local && preflight(LOCAL_COUNT), LOCAL_COUNT) {
            skyline::install_hook!(deluxe_rewrite_local_count);
        }
    }
}

fn note_install(ok: bool, site: usize) -> bool {
    if ok {
        record(Code::HookInstalled, site as u64, 0);
    } else {
        record(Code::HookRejected, site as u64, 0);
        skyline::println!("[Deluxe] native pane hook not installed at {site:x}");
    }
    ok
}

/// Scene entry already installed at `0x1bd45e0`. Drop the previous pane, then
/// capture this one. Does not install another patch.
pub(super) unsafe fn enter_local_scene(base: u64) {
    drop_scene();
    let site = native_banner::LOCAL_SCENE as u64;
    if !SCENE_WORD_OK.load(Ordering::Acquire) {
        record(Code::HookRejected, site, 0);
        return;
    }
    let Some(address) =
        crate::ui_safety::linked_pane(base as usize, 8, |addr| unsafe { read_ptr(addr) })
    else {
        record(Code::HookRejected, site, 1);
        return;
    };
    let identity = checked_pane(address).and_then(|pane| unsafe {
        let name = pane_name(&*pane)?;
        name_ok(PaneKind::Local, &name, (*pane).vtable).then_some(name)
    });
    let Some(name) = identity else {
        record(Code::PaneRejected, site, address as u64);
        return;
    };
    LOCAL_PANE.store(address, Ordering::Release);
    LOCAL_GEN.store(SCENE_GEN.load(Ordering::Acquire), Ordering::Release);
    let (first, second) = native_banner::name_words(&name);
    record(Code::PaneField, first, second);
}

/// The game's own count update. The hook sits right after the dirty query at
/// the site entry, so the game's refresh of its count lands after ours in the
/// call that refreshes it and is captured as the pane's game text.
#[skyline::hook(offset = LOCAL_COUNT, inline)]
unsafe fn deluxe_rewrite_local_count(_: &InlineCtx) {
    let address = LOCAL_PANE.load(Ordering::Acquire);
    let captured = LOCAL_GEN.load(Ordering::Acquire);
    let live = native_banner::local_count_active(SCENE_GEN.load(Ordering::Acquire), captured);
    if address == 0 || !live {
        return;
    }
    let Some(pane) = open_pane(address, PaneKind::Local, LOCAL_COUNT as u64) else {
        return;
    };
    note_surface(NativeSurface::LocalLobby);
    let config = settings();
    let text = (!config.streamer).then(|| native_banner::local_lobby_text(config.input_buffer));
    let Ok(mut state) = STATE.try_lock() else {
        return;
    };
    sync_generation(&mut state);
    let telemetry = text.as_deref().map(|text| (text, None));
    paint_one(&mut state, pane, PaneKind::Local, telemetry, 0);
}

/// Drops arena, lobby and title ownership. Scene hooks call this on entry.
pub(super) fn drop_scene() {
    LOCAL_PANE.store(0, Ordering::Release);
    LOCAL_GEN.store(0, Ordering::Release);
    SURFACE_KIND.store(0, Ordering::Relaxed);
    SURFACE_GEN.store(0, Ordering::Release);
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
    let Ok(mut state) = STATE.try_lock() else {
        return;
    };
    sync_generation(&mut state);
    state.arena = None;
    let Some(address) = crate::ui_safety::linked_pane(base, 8, |addr| unsafe { read_ptr(addr) })
    else {
        return;
    };
    let Some(name) = checked_pane(address).and_then(|pane| unsafe { pane_name(&*pane) }) else {
        return;
    };
    if name != ARENA_ROOM_NAME {
        note_arena_name(&mut state, &name);
        return;
    }
    let Some(pane) = open_pane(address, PaneKind::Arena, ARENA_INIT as u64) else {
        return;
    };
    state.arena = Some(ArenaPane {
        address,
        vtable: (*pane).pane.vtable,
        name,
        room: read_room_id(ctx.registers[3].x() as usize).unwrap_or_default(),
    });
}

#[skyline::hook(offset = ARENA_UPDATE, inline)]
unsafe fn deluxe_rewrite_arena_room_update(_: &InlineCtx) {
    let Ok(mut state) = STATE.try_lock() else {
        return;
    };
    sync_generation(&mut state);
    let Some(arena) = &state.arena else {
        return;
    };
    let pane =
        open_pane(arena.address, PaneKind::Arena, ARENA_UPDATE as u64).filter(|pane| unsafe {
            (**pane).pane.vtable == arena.vtable
                && pane_name(&(**pane).pane).as_deref() == Some(arena.name.as_str())
        });
    let Some(pane) = pane else {
        state.arena = None;
        return;
    };
    note_surface(NativeSurface::Arena);
    let room = arena.room.clone();
    if room.is_empty() {
        note_reject(ARENA_UPDATE as u64, REJECT_ROOM);
        return;
    }
    let telemetry = telemetry(Some(&room));
    let lines = telemetry
        .as_ref()
        .map_or(0, |telemetry| telemetry.remote_lines);
    let args = telemetry
        .as_ref()
        .map(|telemetry| (telemetry.text.as_str(), telemetry.ping));
    paint_one(&mut state, pane, PaneKind::Arena, args, lines);
}

unsafe fn paint_css(arg: u64) {
    let Some(address) =
        crate::ui_safety::banner_address(arg as usize, |addr| unsafe { read_ptr(addr) })
    else {
        note_reject(CSS_UPDATE as u64, REJECT_CHAIN);
        return;
    };
    let Some(first) = open_pane(address, PaneKind::Css, CSS_UPDATE as u64) else {
        return;
    };
    note_surface(NativeSurface::Css);
    super::online::observe_css_scene();
    let first_pane = first.cast::<Pane>();
    let second = walk_links(first_pane, &CSS_TITLE_WALK)
        .and_then(|pane| unsafe { open_pane(pane as usize, PaneKind::Css, CSS_UPDATE as u64) });
    let background = second
        .and_then(|_| unsafe { walk_links(first_pane, &CSS_TITLE_WALK[..CSS_BACKGROUND_STEPS]) });
    let telemetry = telemetry(None);
    let Ok(mut state) = STATE.try_lock() else {
        return;
    };
    sync_generation(&mut state);
    if let Some(pane) = background {
        apply_background(&mut state, pane, telemetry.is_none());
    }
    let args = telemetry
        .as_ref()
        .map(|telemetry| (telemetry.text.as_str(), telemetry.ping));
    paint_one(&mut state, first, PaneKind::Css, args, 0);
    if let Some(pane) = second {
        paint_one(&mut state, pane, PaneKind::Css, args, 0);
    }
}

/// One frame's status text. `None` in Streamer.
struct Telemetry {
    text: String,
    ping: Option<[u8; 4]>,
    remote_lines: u8,
}

/// CSS (`room` None) shows the selected peer while its station is connected;
/// the arena shows every known mapped station through the published board
/// and colours the pane by the worst fresh one.
fn telemetry(room: Option<&str>) -> Option<Telemetry> {
    let config = settings();
    if config.streamer {
        return None;
    }
    let auto = auto_driver::snapshot();
    let profile = crate::auto_policy::profile_row_label(
        config.auto_profile,
        auto.ready,
        config.render_mode,
        auto.request,
        config.vanilla_plus,
    );
    let latency = native_banner::local_latency_label(config.input_buffer);
    let (remotes, ping) = if room.is_some() {
        arena_remotes()
    } else {
        css_remote()
    };
    let (text, remote_lines) = native_banner::status_text(room, &profile, &latency, &remotes);
    Some(Telemetry {
        text,
        ping,
        remote_lines,
    })
}

fn css_remote() -> (Vec<RemoteLine>, Option<[u8; 4]>) {
    let frame = crate::peer_transport::peer_frame();
    let identity = crate::peer_transport::display_identity(frame.selection);
    let ping = crate::pia_rtt::present_published(identity, &pia_rtt::current_publication());
    let color = ping.color.map(native_banner::ping_bytes);
    if !crate::peer_transport::station_connected(frame.selection)
        || !super::pia_rtt::station_live(frame.selection)
    {
        return (Vec::new(), color);
    }
    (vec![remote_line(&frame.peer, ping.opponent)], color)
}

fn arena_remotes() -> (Vec<RemoteLine>, Option<[u8; 4]>) {
    let (_, roster) = crate::peer_transport::banner_peers();
    if roster.busy {
        return (Vec::new(), None);
    }
    let board = pia_rtt::current_board();
    let mut listed = Vec::new();
    let mut lines = Vec::new();
    for peer in roster.peers.iter().take(usize::from(roster.len)) {
        let (Some(index), true) = (peer.station, peer.known) else {
            continue;
        };
        listed.push(index);
        let presented = crate::pia_rtt::present_station(&board, index);
        let suffix = native_banner::interface_suffix(peer.interface);
        lines.push(remote_line(peer, format!("{}{suffix}", presented.opponent)));
    }
    let color = crate::pia_rtt::aggregate_stability(&board, &listed)
        .map(crate::pia_rtt::ping_rgba)
        .map(native_banner::ping_bytes);
    (lines, color)
}

/// Blank fields become `status_text`'s fallbacks (`Unavailable`, `---`).
fn remote_line(peer: &crate::peer_transport::ShownPeer, ping: String) -> RemoteLine {
    let known = |text: &str| if peer.known { text } else { "" }.to_owned();
    RemoteLine {
        name: known(crate::peer_transport::name_label(&peer.name, peer.name_len).unwrap_or("")),
        profile: known(peer.settings),
        latency: crate::peer_transport::remote_latency_label(peer.latency),
        ping,
    }
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

fn site(kind: PaneKind) -> u64 {
    match kind {
        PaneKind::Css => CSS_UPDATE as u64,
        PaneKind::Arena => ARENA_UPDATE as u64,
        PaneKind::Local => LOCAL_COUNT as u64,
    }
}

/// The pane names this mod may write, by site. The lobby pane is also tied
/// to its probed vtable.
unsafe fn name_ok(kind: PaneKind, name: &str, vtable: u64) -> bool {
    match kind {
        PaneKind::Css => crate::ui_safety::is_title_name(name.as_bytes()),
        PaneKind::Arena => name == ARENA_ROOM_NAME,
        PaneKind::Local => {
            native_banner::local_pane_identity(name, vtable, getRegionAddress(Region::Text) as u64)
        }
    }
}

/// `pane` passed `open_pane` in this call. Keeps the slot, refreshes its
/// saved words when the game wrote again, or captures the style once; then
/// writes what the host decides and logs the first write.
unsafe fn paint_one(
    state: &mut BannerState,
    pane: *mut TextBox,
    kind: PaneKind,
    telemetry: Option<(&str, Option<[u8; 4]>)>,
    remote_lines: u8,
) {
    let site = site(kind);
    let address = pane as u64;
    let vtable = (*pane).pane.vtable;
    let Some(name) = pane_name(&(*pane).pane) else {
        note_reject(site, REJECT_PANE);
        return;
    };
    let Some(current) = read_current(&*pane) else {
        note_reject(site, REJECT_TEXT);
        return;
    };
    let game_text = !current.is_empty() && !native_banner::is_our_text(&current);
    let same = |slot: &Slot| slot.address == address && slot.vtable == vtable && slot.name == name;
    let found = state
        .slots
        .iter()
        .position(|slot| slot.as_ref().is_some_and(same));
    let index = match found {
        Some(index) => index,
        None if !game_text => return,
        None => {
            let free =
                |slot: &Option<Slot>| slot.as_ref().is_none_or(|slot| slot.address == address);
            let Some(index) = state.slots.iter().position(free) else {
                return;
            };
            let pos_y = (kind == PaneKind::Arena).then_some((*pane).pane.pos_y);
            state.slots[index] = Some(Slot {
                address,
                vtable,
                name: name.clone(),
                captured: Styled {
                    text: String::new(),
                    style: Style {
                        font: capture_font(&*pane),
                        colors: (*pane).text_colors,
                        material: material(&*pane, false)
                            .map(|material| (*material).m_colors.byte_color),
                        pos_y: pos_y.filter(|y| y.is_finite()),
                    },
                },
                applied: false,
                logged: false,
            });
            index
        }
    };
    let Some(slot) = state.slots[index].as_mut() else {
        return;
    };
    if game_text {
        slot.captured.text.clone_from(&current);
    }
    let ours = native_banner::is_our_text(&current) || slot.applied;
    let Some(write) = native_banner::paint(&slot.captured, ours, telemetry, kind, remote_lines)
    else {
        return;
    };
    slot.applied = telemetry.is_some();
    let wrote = current == write.text || write_text(&mut *pane, &write.text);
    if !wrote && telemetry.is_some() {
        note_reject(site, REJECT_FIT);
        return;
    }
    apply_style(&mut *pane, &write.style);
    if wrote && !slot.logged {
        slot.logged = true;
        let (first, second) = native_banner::name_words(&name);
        record(Code::PaneAccepted, first, second);
        record(Code::PaneField, address, vtable);
    }
}

/// Hides the CSS title background; Streamer restores its captured visibility.
unsafe fn apply_background(state: &mut BannerState, pane: *mut Pane, streamer: bool) {
    let address = pane as u64;
    if !memory::accessible(pane as usize, std::mem::size_of::<Pane>(), true)
        || pane_name(&*pane).is_none()
    {
        return;
    }
    let bit = 1u8 << PaneFlag::Visible as u8;
    if !matches!(state.background, Some((saved, _)) if saved == address) {
        state.background = Some((address, (*pane).flags & bit != 0));
    }
    if streamer && matches!(state.background, Some((_, true))) {
        (*pane).flags |= bit;
    } else {
        (*pane).flags &= !bit;
    }
}

unsafe fn walk_links(start: *mut Pane, steps: &[PaneLink]) -> Option<*mut Pane> {
    let mut current = start;
    for step in steps {
        let next = match step {
            PaneLink::Parent => checked_pane((*current).parent as usize)?,
            PaneLink::Prev => link_prev(current)?,
        };
        if next == start || next == current {
            return None;
        }
        current = next;
    }
    (current != start).then_some(current)
}

unsafe fn link_prev(pane: *mut Pane) -> Option<*mut Pane> {
    let link = (pane as usize).checked_add(std::mem::offset_of!(Pane, link))?;
    let node = read_ptr(link.checked_add(std::mem::offset_of!(PaneNode, prev))?)?;
    if node < 0x10000 || !node.is_multiple_of(8) {
        return None;
    }
    let pane = checked_pane(node.checked_sub(std::mem::offset_of!(Pane, link))?)?;
    let list = (pane as usize).checked_add(std::mem::offset_of!(Pane, children_list))?;
    let next = read_ptr(list.checked_add(std::mem::offset_of!(PaneNode, next))?)?;
    let prev = read_ptr(list.checked_add(std::mem::offset_of!(PaneNode, prev))?)?;
    (next != 0 || prev != 0).then_some(pane)
}

/// An aligned, readable pane whose vtable sits in the main image.
unsafe fn checked_pane(address: usize) -> Option<*mut Pane> {
    (address >= 0x10000
        && address.is_multiple_of(8)
        && memory::accessible(address, std::mem::size_of::<Pane>(), false)
        && vtable_ok((*(address as *const Pane)).vtable))
    .then_some(address as *mut Pane)
}

/// A checked pane with an allowed name whose text box is writable.
unsafe fn open_pane(address: usize, kind: PaneKind, site: u64) -> Option<*mut TextBox> {
    let Some(pane) = checked_pane(address) else {
        note_open(site, address as u64, 1, 0);
        return None;
    };
    let vtable = (*pane).vtable;
    let Some(name) = pane_name(&*pane) else {
        note_open(site, address as u64, 2, vtable);
        return None;
    };
    if !name_ok(kind, &name, vtable) {
        note_open(site, address as u64, 4, vtable);
        return None;
    }
    if !memory::accessible(address, std::mem::size_of::<TextBox>(), true) {
        note_open(site, address as u64, 5, vtable);
        return None;
    }
    Some(address as *mut TextBox)
}

/// One rejected open per site, address and guard.
fn note_open(site: u64, address: u64, guard: u8, vtable: u64) {
    let key = address ^ site.rotate_left(17) ^ u64::from(guard);
    if OPEN_LOGGED.swap(key, Ordering::Relaxed) == key {
        return;
    }
    let reason = (u64::from(guard) << 8) | u64::from(REJECT_PANE);
    record(Code::PaneRejected, site, reason);
    record(Code::PaneField, address, vtable);
}

unsafe fn setter_available() -> bool {
    let base = getRegionAddress(Region::Text) as usize;
    let entry = base
        .checked_add(SET_TEXT)
        .filter(|entry| memory::accessible(*entry, 0x1c, false));
    let Some(entry) = entry else {
        record(Code::HookRejected, SET_TEXT as u64, 0);
        return false;
    };
    let word = |index: usize| *((entry + index * 4) as *const u32);
    let tail = [word(4), word(5), word(6)];
    if native_banner::chained_text_setter(&[word(0), word(1)], &tail) {
        record(Code::Compatibility, SET_TEXT as u64, u64::from(word(0)));
        skyline::println!("[Deluxe] native text setter chains the existing hook at {SET_TEXT:x}");
        return true;
    }
    words_ok(SET_TEXT, &SETTER_WORDS) && preflight(SET_TEXT)
}

unsafe fn read_current(tb: &TextBox) -> Option<String> {
    let cap = usize::from(tb.text_buf_len);
    let declared = usize::from(tb.text_len);
    if tb.text_buf.is_null()
        || cap == 0
        || cap > native_banner::PANE_SCAN_MAX
        || declared > cap
        || declared > TEXT_CAP
    {
        return None;
    }
    let scan = if declared == 0 {
        cap.min(TEXT_CAP)
    } else {
        (declared + 1).min(cap)
    };
    let unit = if tb.is_utf8 { 1 } else { 2 };
    if !(tb.text_buf as usize).is_multiple_of(unit)
        || !memory::accessible(tb.text_buf as usize, scan * unit, false)
    {
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
    let units = std::ptr::read_volatile(address as *const [u16; native_banner::ROOM_ID_UNITS]);
    native_banner::decode_room_id(&units)
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

unsafe fn apply_style(tb: &mut TextBox, style: &Style) {
    tb.text_colors = style.colors;
    tb.bits |= 1u16 << u32::from(TextBoxFlag::IsPTDirty as u8);
    if let Some(font) = style.font {
        tb.font_size_x = font.font_x;
        tb.font_size_y = font.font_y;
        tb.line_space = font.line_space;
        tb.char_space = font.char_space;
        tb.text_position = font.text_position;
    }
    if let Some(y) = style.pos_y.filter(|y| y.is_finite()) {
        tb.pane.pos_y = y;
    }
    if style.font.is_some() || style.pos_y.is_some() {
        tb.pane.flags |= 1u8 << u32::from(PaneFlag::IsGlobalMatrixDirty as u8);
    }
    if let Some(colors) = style.material {
        if let Some(material) = material(tb, true) {
            (*material).m_colors.byte_color = colors;
        }
    }
}

/// The text box's byte-colour material, when it is mapped (for `write` when
/// it must be written) and not a float-colour material.
unsafe fn material(tb: &TextBox, write: bool) -> Option<*mut Material> {
    let float = (1u8 << MaterialFlags::BlackColorFloat as u8)
        | (1u8 << MaterialFlags::WhiteColorFloat as u8);
    let material = tb.p_material;
    (!material.is_null()
        && memory::accessible(material as usize, std::mem::size_of::<Material>(), write)
        && (*material).m_flag & float == 0)
        .then_some(material)
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

fn pane_name(pane: &Pane) -> Option<String> {
    native_banner::decode_name(&pane.name).map(str::to_owned)
}

unsafe fn read_ptr(address: usize) -> Option<usize> {
    if !address.is_multiple_of(8) || !memory::accessible(address, 8, false) {
        return None;
    }
    Some(*(address as *const usize))
}

unsafe fn vtable_ok(vtable: u64) -> bool {
    let region = |region| getRegionAddress(region) as u64;
    let (text, rodata, data) = (
        region(Region::Text),
        region(Region::Rodata),
        region(Region::Data),
    );
    let Some(data_end) = data.checked_add(native_banner::MAIN_DATA_BYTES) else {
        return false;
    };
    let address = vtable as usize;
    if !vtable.is_multiple_of(8) || !memory::accessible(address, 16, false) {
        return false;
    }
    let entry = std::ptr::read_volatile(address as *const u64);
    native_banner::main_image_vtable(vtable, entry, text, rodata, data_end)
}

/// Every word at `site` matches the 13.0.5 image.
unsafe fn words_ok(site: usize, words: &[u32]) -> bool {
    words
        .iter()
        .enumerate()
        .all(|(index, word)| renderer::expect(site + index * 4, *word))
}

fn sync_generation(state: &mut BannerState) {
    let generation = SCENE_GEN.load(Ordering::Acquire);
    if state.generation != generation {
        *state = BannerState::empty(generation);
    }
}

/// The init site named another pane. Logged once per scene.
fn note_arena_name(state: &mut BannerState, name: &str) {
    if state.arena_reject_logged {
        return;
    }
    state.arena_reject_logged = true;
    let (first, second) = native_banner::name_words(name);
    record(Code::PaneField, first, second);
}

fn note_reject(site: u64, reason: u8) {
    let packed = site.wrapping_shl(8) | u64::from(reason);
    if LAST_REJECT.swap(packed, Ordering::Relaxed) != packed {
        record(Code::PaneRejected, site, u64::from(reason));
    }
}
