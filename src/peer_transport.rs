//! Manager connection-id handshake and the fork's in-payload announcements.
//!
//! Field positions are the manager instructions cited in
//! `grok-connection-id-implementation.md`. Header u64s are not connection keys.
//! Hooks call this module; nothing here waits, allocates, or writes a file.
#![cfg_attr(not(target_os = "switch"), allow(dead_code))]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::connection_transport::mix_id;
use crate::peer_passive::{
    beacon_tag_offset, custom_write_allowed, decode_rtt_reply_payload, read_be_u64,
    read_response_beacon, rtt_type_editable, walk_messages, Beacon, ForkPayload,
};
use crate::render_policy::Profile;

const SLOTS: usize = 8;
const NAME_CAP: usize = 64;
const LOCATION_STRIDE: usize = 0xc3;
const BEACON_HEADER: usize = 0x39;
const ENTRY_NAME: usize = 0x51;
const ENTRY_NAME_LEN: usize = 0x28;
const ENTRY_TAG: usize = 0x79;
const KIND_STATION: u8 = 0x14;
const KIND_RTT: u8 = 0x58;
const RTT_BODY_MIN: usize = 0x10;
/// No station selected, or the local index is not known yet.
const NONE_STATION: u8 = 255;
/// Mapped remotes copied under one gate hold. Eight is the handshake table.
pub const REMOTE_CAP: usize = SLOTS;
/// Fork preset bits: buffer (bits 0-1), index (2-3), vsync (12), render opts
/// (13), with the plain and the bit-14 (`++`) label. Masked by `PRESET_MASK`.
const PRESETS: [(Profile, u16, &str, &str); 4] = [
    (Profile::Vanilla, 0x100b, "Vanilla", "Vanilla++"),
    (Profile::LessLag, 0x2006, "LessLag", "LessLag++"),
    (Profile::Ultra, 0x2002, "LLUltra", "LLUltra++"),
    (Profile::Doubles, 0x2007, "LLDoubles", "LLDoubles++"),
];
const PRESET_MASK: u16 = 0x300f;
/// Leave bits that name a station `0..=252`; word 3 stops at bit 60.
const LEAVE_MASK: [u64; 4] = [u64::MAX, u64::MAX, u64::MAX, (1 << 61) - 1];

static BUSY: AtomicBool = AtomicBool::new(false);
struct TableCell(std::cell::UnsafeCell<Table>);
unsafe impl Sync for TableCell {}
static TABLE: TableCell = TableCell(std::cell::UnsafeCell::new(Table::empty()));
static COUNTERS: [AtomicU64; 10] = [const { AtomicU64::new(0) }; 10];
static LAST_MIX: AtomicU64 = AtomicU64::new(0);
/// Bumped by every cleanup, including when `BUSY` is already held.
/// A later gate holder drops the table before any read or write.
static SESSION_GEN: AtomicU64 = AtomicU64::new(1);
/// Station leaves published while the gate may be busy, one bit per index.
static LEAVE_BITS: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];

/// Diagnostic counters drained by `runtime::connection_transport` into events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Counter {
    Learned,
    Ambiguous,
    Confirmed,
    Cleanups,
    BeaconWrites,
    BeaconSkips,
    PayloadWrites,
    PayloadSkips,
    StealthSuppress,
    Contentions,
}

fn bump(counter: Counter) {
    COUNTERS[counter as usize].fetch_add(1, Ordering::Relaxed);
}

pub fn counter(counter: Counter) -> u64 {
    COUNTERS[counter as usize].load(Ordering::Relaxed)
}

/// Mixed constant id of the last learned or confirmed peer.
pub fn last_mix() -> u64 {
    LAST_MIX.load(Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pending {
    constant_id: u64,
    incoming: u8,
    outgoing: u8,
    poisoned: bool,
}

#[derive(Clone, Copy, Debug)]
struct Peer {
    constant_id: u64,
    incoming: u8,
    outgoing: u8,
    modded: bool,
    interface_raw: u8,
    name: [u8; NAME_CAP],
    name_len: u8,
    /// Fork `latency_bits` and `profile_bits` once a kind-`0x58` reply arrived.
    payload: Option<(u8, u16)>,
}

#[derive(Clone, Copy, Debug)]
struct Table {
    /// `SESSION_GEN` value already applied to this table.
    applied_gen: u64,
    /// Stealth bit latched by the first request of the session.
    session: Option<bool>,
    pending: [Option<Pending>; SLOTS],
    peers: [Option<Peer>; SLOTS],
    /// Station index `0..=252` to constant id. `0` means unmapped.
    index_id: [u64; 253],
    selected: u8,
    local_index: u8,
}

impl Table {
    const fn empty() -> Self {
        Self {
            applied_gen: 1,
            session: None,
            pending: [None; SLOTS],
            peers: [None; SLOTS],
            index_id: [0; 253],
            selected: NONE_STATION,
            local_index: NONE_STATION,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Announce {
    pub latency: i8,
    /// Committed profile while the machine is live; `None` sends no profile.
    /// `Profile::Doubles` is the fork's `less_lag_doubles` bits, not `doubles_core`.
    pub profile: Option<Profile>,
    /// Ultelier resolution discriminant. Zero when no profile is sent.
    pub resolution: u8,
    /// Actual dynamic-resolution state. False when the resolution guard rejected.
    pub dynamic: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyWrite {
    pub at: usize,
    pub bytes: [u8; 4],
    pub len: u8,
}

/// One send's inputs beyond the message bytes.
#[derive(Clone, Copy)]
struct Send {
    outgoing_byte: u8,
    announce: Option<Announce>,
    interface_byte: u8,
    saved_stealth: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShownPeer {
    pub known: bool,
    pub name: [u8; NAME_CAP],
    pub name_len: u8,
    pub interface: &'static str,
    pub settings: &'static str,
    /// Buffer from the same chosen peer as the name, interface, and settings.
    pub latency: RemoteLatency,
    /// Station index that owns this name, interface, and settings.
    pub station: Option<u8>,
}

const BLANK_PEER: ShownPeer = ShownPeer {
    known: false,
    name: [0; NAME_CAP],
    name_len: 0,
    interface: "Unknown",
    settings: "",
    latency: RemoteLatency::Unknown,
    station: None,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteRoster {
    pub peers: [ShownPeer; REMOTE_CAP],
    pub len: u8,
    pub busy: bool,
}

const EMPTY_ROSTER: RemoteRoster = RemoteRoster {
    peers: [BLANK_PEER; REMOTE_CAP],
    len: 0,
    busy: false,
};

/// Announced input buffer for the peer in this snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteLatency {
    /// Cleanup, no chosen peer, unmodded, no payload, or a byte outside Auto and 0..=25.
    Unknown,
    Auto,
    Buffer(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Station(u8),
    /// No mapped remote peer. One confirmed peer may still be shown by name.
    None,
    /// The gate was busy. Callers must not substitute a different station.
    Busy,
}

/// One gate hold for the remote name and the station that must own the ping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeerFrame {
    pub selection: Selection,
    pub peer: ShownPeer,
}

/// Player Info text. Streamer draws a blank instead of calling this.
pub fn remote_latency_label(latency: RemoteLatency) -> String {
    match latency {
        RemoteLatency::Unknown => "---".to_owned(),
        RemoteLatency::Auto => "Auto".to_owned(),
        RemoteLatency::Buffer(value) => value.to_string(),
    }
}

/// Owned name bytes. Invalid UTF-8 is absent rather than a torn borrow.
pub fn name_label(name: &[u8], len: u8) -> Option<&str> {
    let bytes = &name[..usize::from(len).min(name.len())];
    std::str::from_utf8(bytes)
        .ok()
        .filter(|text| !text.is_empty())
}

/// Saved preference versus the latched session. `None` means no session.
pub fn stealth_label(saved: bool, latched: Option<bool>) -> &'static str {
    match (latched, saved) {
        (None, false) | (Some(false), false) => "Off",
        (None, true) | (Some(true), true) => "On",
        (Some(true), false) => "Off (reconnect)",
        (Some(false), true) => "On (reconnect)",
    }
}

/// Local nifm type. Byte 3 and every value other than 1 or 2 are Unknown.
/// A text base is not an interface, and it is not a console-or-emulator label.
pub fn local_link_label(byte: u8) -> &'static str {
    match byte {
        1 => "Wi-Fi",
        2 => "Ethernet",
        _ => "Unknown",
    }
}

pub fn interface_label(raw: u8, modded: bool) -> &'static str {
    if modded {
        local_link_label(raw)
    } else {
        "Unknown"
    }
}

/// Fork `from_settings` names a preset from buffer, index, vsync, and render
/// opts after `from_bits` accepts the resolution. Dynamic resolution may differ
/// between console and emulator singles. Bit 14 is the fork boost bit: it adds
/// `++` to a preset and does not select another one. Bit 15 and an
/// out-of-range resolution stay Other.
pub fn settings_label(bits: u16) -> &'static str {
    if bits & (1 << 15) != 0 || (bits >> 4) & 0x7f > 4 {
        return "Other";
    }
    let preset = PRESETS
        .iter()
        .find(|preset| preset.1 == (bits & PRESET_MASK));
    let boost = bits & (1 << 14) != 0;
    preset.map_or("Other", |preset| if boost { preset.3 } else { preset.2 })
}

/// Fork `RenderProfileSettings::to_bits` without bit 14.
///
/// Buffer, index, vsync, and render opts follow the committed profile. The
/// resolution discriminant and dynamic-resolution bit are the configured
/// state passed in. Bit 14 (the fork's FPS boost) is never sent: FPS++ was
/// removed on 2026-10-17. It is still decoded from peers.
pub fn profile_bits(profile: Profile, resolution: u8, dynamic: bool) -> u16 {
    let preset = PRESETS.iter().find(|preset| preset.0 == profile);
    let preset = preset.map_or(0, |preset| preset.1);
    preset | (u16::from(resolution.min(4)) << 4) | (u16::from(dynamic) << 11)
}

/// Wire profile while the machine is live, in a match or in menus. A
/// passthrough machine sends no profile, so a saved menu mode is not
/// advertised. Resolution and dynamic resolution are the configured state,
/// not a guessed preset.
pub fn announce_from(
    latency: i8,
    live: bool,
    committed: Profile,
    setting: crate::resolution::Setting,
) -> Announce {
    Announce {
        latency,
        profile: live.then_some(committed),
        resolution: if live { setting.level as u8 } else { 0 },
        dynamic: live && setting.dynamic,
    }
}

/// The only entry to the table. A busy gate copies nothing and returns `None`.
/// A published cleanup generation drops the table and pending leaves are
/// applied before `update` sees it.
fn with_table<T>(update: impl FnOnce(&mut Table) -> T) -> Option<T> {
    if BUSY.swap(true, Ordering::AcqRel) {
        bump(Counter::Contentions);
        return None;
    }
    let table = unsafe { &mut *TABLE.0.get() };
    let gen = SESSION_GEN.load(Ordering::Acquire);
    if table.applied_gen != gen {
        *table = Table::empty();
        table.applied_gen = gen;
    }
    apply_leaves(table);
    let value = update(table);
    BUSY.store(false, Ordering::Release);
    Some(value)
}

fn apply_leaves(table: &mut Table) {
    let mut any = false;
    for (word, bits) in LEAVE_BITS.iter().enumerate() {
        let mut seen = bits.swap(0, Ordering::AcqRel) & LEAVE_MASK[word];
        any |= seen != 0;
        while seen != 0 {
            let index = word * 64 + seen.trailing_zeros() as usize;
            seen &= seen - 1;
            let id = std::mem::take(&mut table.index_id[index]);
            if usize::from(table.selected) == index {
                table.selected = NONE_STATION;
            }
            // One constant id is one opponent. A rematch can leave the same
            // id bound at an older index; dropping only the reported index
            // keeps their column up.
            if id != 0 {
                retire_constant(table, id);
            } else {
                retire_only_confirmed(table);
            }
        }
    }
    if any {
        resolve_selection(table);
    }
}

/// A station left that no constant id was ever bound to (the station-index
/// hook binds only when the game resolves that station by id, so a leave can
/// arrive with no binding). With exactly one confirmed peer that peer is the
/// opponent who left: an opponent who declines a rematch otherwise stays on
/// the banner through the single-peer fallback until the whole session ends.
fn retire_only_confirmed(table: &mut Table) {
    if let Some(only) = only_confirmed(table) {
        retire_constant(table, only.constant_id);
    }
}

fn only_confirmed(table: &Table) -> Option<Peer> {
    let mut confirmed = table.peers.iter().flatten();
    match (confirmed.next(), confirmed.next()) {
        (Some(only), None) => Some(*only),
        _ => None,
    }
}

fn retire_constant(table: &mut Table, id: u64) {
    if id == 0 {
        return;
    }
    for slot in table.index_id.iter_mut().filter(|slot| **slot == id) {
        *slot = 0;
    }
    for slot in table
        .pending
        .iter_mut()
        .filter(|s| s.is_some_and(|p| p.constant_id == id))
    {
        *slot = None;
    }
    for slot in table
        .peers
        .iter_mut()
        .filter(|s| s.is_some_and(|p| p.constant_id == id))
    {
        *slot = None;
    }
}

fn peer_by_id(table: &Table, id: u64) -> Option<Peer> {
    let mut peers = table.peers.iter().flatten();
    peers.find(|peer| peer.constant_id == id).copied()
}

fn selected_peer(table: &Table) -> Option<Peer> {
    let id = *table.index_id.get(usize::from(table.selected))?;
    if id == 0 {
        return None;
    }
    peer_by_id(table, id)
}

/// Mapped stations whose constant id is a confirmed peer, excluding the local index.
fn candidates(table: &Table) -> ([u8; 253], usize) {
    let mut out = [0u8; 253];
    let mut count = 0usize;
    for (index, id) in table.index_id.iter().enumerate() {
        if index as u8 != table.local_index && *id != 0 && peer_by_id(table, *id).is_some() {
            out[count] = index as u8;
            count += 1;
        }
    }
    (out, count)
}

fn resolve_selection(table: &mut Table) {
    let (found, count) = candidates(table);
    if count == 0 {
        table.selected = NONE_STATION;
    } else if !found[..count].contains(&table.selected) {
        table.selected = found[0];
    }
}

/// Always-on station map. Diagnostic copies are not consulted.
pub fn bind_station(index: u32, constant_id: u64) {
    if index > 252 || constant_id == 0 {
        return;
    }
    let _ = with_table(|table| {
        table.index_id[index as usize] = constant_id;
        resolve_selection(table);
    });
}

pub fn note_local(index: u32) {
    if index > 252 {
        return;
    }
    let _ = with_table(|table| {
        table.local_index = index as u8;
        resolve_selection(table);
    });
}

/// Publishes the leave even when the gate is busy. The next holder drops it.
pub fn clear_station(index: u32) {
    if index > 252 {
        return;
    }
    LEAVE_BITS[index as usize / 64].fetch_or(1u64 << (index % 64), Ordering::Release);
    let _ = with_table(|_| ());
}

/// Left (`-1`) or Right (`1`) across the mapped remote stations.
pub fn cycle_peer(direction: i8) {
    let _ = with_table(|table| {
        let (found, count) = candidates(table);
        if direction == 0 || count < 2 {
            return resolve_selection(table);
        }
        let current = found[..count]
            .iter()
            .position(|index| *index == table.selected);
        let step = if direction < 0 { count - 1 } else { 1 };
        table.selected = found[(current.unwrap_or(0) + step) % count];
    });
}

/// Drops authorization even when the table gate is busy. The generation is
/// published before the attempt, so the next holder cannot use the old peers.
pub fn clear_session() {
    SESSION_GEN.fetch_add(1, Ordering::Release);
    bump(Counter::Cleanups);
    let _ = with_table(|_| ());
}

fn shown(peer: &Peer, station: Option<u8>) -> ShownPeer {
    ShownPeer {
        known: true,
        name: peer.name,
        name_len: peer.name_len,
        interface: interface_label(peer.interface_raw, peer.modded),
        settings: peer.payload.map_or("", |(_, bits)| settings_label(bits)),
        latency: match peer.payload {
            Some((255, _)) if peer.modded => RemoteLatency::Auto,
            Some((byte @ 0..=25, _)) if peer.modded => RemoteLatency::Buffer(byte),
            _ => RemoteLatency::Unknown,
        },
        station,
    }
}

/// The selected station, the roster of mapped remotes, and the latched
/// stealth state, all from one table state. With no mapped station the one
/// confirmed peer is shown by name without a selection.
fn snapshot(table: &mut Table) -> (PeerFrame, RemoteRoster, Option<bool>) {
    resolve_selection(table);
    let (found, count) = candidates(table);
    let mut roster = EMPTY_ROSTER;
    for index in found.into_iter().take(count.min(REMOTE_CAP)) {
        let Some(peer) = peer_by_id(table, table.index_id[usize::from(index)]) else {
            continue;
        };
        roster.peers[usize::from(roster.len)] = shown(&peer, Some(index));
        roster.len += 1;
    }
    let fallback = only_confirmed(table).map_or(BLANK_PEER, |peer| shown(&peer, None));
    let sel = table.selected;
    let (selection, peer) = match selected_peer(table) {
        Some(peer) => (Selection::Station(sel), shown(&peer, Some(sel))),
        None => (Selection::None, fallback),
    };
    (PeerFrame { selection, peer }, roster, table.session)
}

/// Selected peer and every mapped remote, copied under one gate hold.
/// Busy carries no names. Local and unmapped peers are omitted from the roster.
pub fn banner_peers() -> (PeerFrame, RemoteRoster) {
    match with_table(snapshot) {
        Some((frame, roster, _)) => (frame, roster),
        None => {
            let peer = BLANK_PEER;
            let selection = Selection::Busy;
            let roster = RemoteRoster {
                busy: true,
                ..EMPTY_ROSTER
            };
            (PeerFrame { selection, peer }, roster)
        }
    }
}

pub fn peer_frame() -> PeerFrame {
    banner_peers().0
}

/// Snapshot under the same nonblocking gate as the name. Busy is not `None`.
pub fn selection_snapshot() -> Selection {
    peer_frame().selection
}

pub fn latched_stealth() -> Option<bool> {
    with_table(snapshot)?.2
}

pub fn opponent_connected() -> bool {
    station_connected(selection_snapshot())
}

/// A mapped station is connected even while its ping is still pending.
pub fn station_connected(selection: Selection) -> bool {
    matches!(selection, Selection::Station(_))
}

/// Opponent column, ping cell, and remote banner line. Streamer hides the
/// remote rows; absent and busy stay blank.
pub fn opponent_column_visible(streamer: bool, selection: Selection) -> bool {
    !streamer && station_connected(selection)
}

/// Draw rule for the publication sitting next to this snapshot.
pub fn display_identity(selection: Selection) -> crate::pia_rtt::RttIdentity {
    match selection {
        Selection::Station(index) => crate::pia_rtt::RttIdentity::Station(index),
        Selection::None => crate::pia_rtt::RttIdentity::Baseline,
        Selection::Busy => crate::pia_rtt::RttIdentity::Busy,
    }
}

pub fn observe_incoming(
    source_id: u64,
    incoming_byte: u8,
    kind: u8,
    body: &[u8],
    saved_stealth: bool,
) {
    let _ = with_table(|table| match (kind, body.first().copied().unwrap_or(0)) {
        (KIND_STATION, 1) => note_request(table, false, body, saved_stealth),
        (KIND_STATION, 2) => note_response(table, source_id, body),
        // Kind `0x14` subtype 3 or 4. The manager compares those bytes as a
        // disconnect. The source id is the same constant id a subtype-2
        // response confirmed. A zero id matches nothing. Vanilla settings
        // are not a leave.
        (KIND_STATION, 3 | 4) => {
            retire_constant(table, source_id);
            resolve_selection(table);
        }
        (KIND_RTT, _) => note_rtt(table, incoming_byte, body),
        _ => {}
    });
}

/// Kind `0x14` subtype 1. The first request of a session latches the saved
/// stealth bit. A second request for the same id with other bytes poisons
/// the pending slot; a full table is ambiguous.
fn note_request(table: &mut Table, outgoing: bool, body: &[u8], saved_stealth: bool) {
    let Some(pending) = parse_pending(outgoing, body) else {
        return;
    };
    table.session.get_or_insert(saved_stealth);
    for existing in table.pending.iter_mut().flatten() {
        if existing.constant_id == pending.constant_id {
            if existing.incoming != pending.incoming || existing.outgoing != pending.outgoing {
                existing.poisoned = true;
                bump(Counter::Ambiguous);
            }
            return;
        }
    }
    let Some(slot) = table.pending.iter_mut().find(|slot| slot.is_none()) else {
        return bump(Counter::Ambiguous);
    };
    *slot = Some(pending);
    bump(Counter::Learned);
    LAST_MIX.store(mix_id(pending.constant_id), Ordering::Relaxed);
}

/// The constant id, incoming byte, and outgoing byte swap with the direction.
/// Only an inverse request with both ids present is ready; everything else,
/// including a truncated body or an unsupported location tag, is ignored.
fn parse_pending(outgoing: bool, body: &[u8]) -> Option<Pending> {
    if body.first() != Some(&1) {
        return None;
    }
    let mut cursor = 2 + ((usize::from(*body.get(1)?) & 0x7f) << 1);
    let connection_id = *body.get(cursor)?;
    let is_inverse = *body.get(cursor + 1)?;
    cursor += 2;
    let target = read_be_u64(body, cursor)?;
    cursor += 8 + 4;
    let inverse_connection_id = *body.get(cursor)?;
    // Two typed fields named by the leading tags, then a type-6 field, then the id.
    let location = body.get(cursor + 1..)?;
    let mut at = 2usize;
    for tag in [*location.first()?, *location.get(1)?, 6] {
        at += match tag {
            2..=5 => 2,
            6 => 6,
            _ => return None,
        };
    }
    let location = read_be_u64(location, at)?;
    if is_inverse != 1 || target == 0 || location == 0 {
        return None;
    }
    let (constant_id, incoming, outgoing) = if outgoing {
        (target, inverse_connection_id, connection_id)
    } else {
        (location, connection_id, inverse_connection_id)
    };
    Some(Pending {
        constant_id,
        incoming,
        outgoing,
        poisoned: false,
    })
}

/// Kind `0x14` subtype 2 from the pending constant id: the first player name,
/// the response beacon for modded + interface. The peer replaces any entry
/// sharing its id, incoming byte, or outgoing byte.
fn note_response(table: &mut Table, source_id: u64, body: &[u8]) {
    if source_id == 0 {
        return bump(Counter::Ambiguous);
    }
    let same_id = |slot: &&mut Option<Pending>| slot.is_some_and(|p| p.constant_id == source_id);
    let Some(slot) = table.pending.iter_mut().find(same_id) else {
        return;
    };
    let pending = match *slot {
        Some(pending) if !pending.poisoned => pending,
        _ => return bump(Counter::Ambiguous),
    };
    *slot = None;
    let (modded, interface_raw) = match read_response_beacon(body) {
        Beacon::Tagged { raw, .. } => (true, raw),
        _ => (false, 0),
    };
    let (name, name_len) = first_player_name(body);
    let peer = Peer {
        constant_id: source_id,
        incoming: pending.incoming,
        outgoing: pending.outgoing,
        modded,
        interface_raw,
        name,
        name_len,
        payload: None,
    };
    for slot in &mut table.peers {
        let same = slot.is_some_and(|existing| {
            existing.constant_id == peer.constant_id
                || existing.outgoing == peer.outgoing
                || existing.incoming == peer.incoming
        });
        if same {
            *slot = None;
        }
    }
    let free = table.peers.iter_mut().find(|slot| slot.is_none());
    match free {
        Some(slot) => *slot = Some(peer),
        None => bump(Counter::Ambiguous),
    }
    resolve_selection(table);
    if table
        .peers
        .iter()
        .flatten()
        .any(|p| p.constant_id == source_id)
    {
        bump(Counter::Confirmed);
        LAST_MIX.store(mix_id(source_id), Ordering::Relaxed);
    }
}

/// Kind `0x58` reply payload for the one modded peer whose incoming byte matches.
fn note_rtt(table: &mut Table, incoming_byte: u8, body: &[u8]) {
    let payload = decode_rtt_reply_payload(body);
    let Some(ForkPayload::Accepted {
        latency_bits,
        profile_bits,
        ..
    }) = payload
    else {
        return;
    };
    let peers = table.peers.iter_mut().flatten();
    let mut matching = peers.filter(|peer| peer.modded && peer.incoming == incoming_byte);
    if let (Some(peer), None) = (matching.next(), matching.next()) {
        peer.payload = Some((latency_bits, profile_bits));
    }
}

/// First location entry of a subtype-2 body: tag 1 is UTF-8, tag 2 is UTF-16
/// LE up to 20 units. Anything else, or an undecodable name, is empty.
fn first_player_name(body: &[u8]) -> ([u8; NAME_CAP], u8) {
    let mut out = [0u8; NAME_CAP];
    let entry = body.get(BEACON_HEADER..BEACON_HEADER + LOCATION_STRIDE);
    let Some(entry) = entry.filter(|_| body[0] == 2 && body[0x38] != 0) else {
        return (out, 0);
    };
    let window = &entry[ENTRY_NAME..ENTRY_NAME + ENTRY_NAME_LEN];
    let mut len = 0usize;
    match entry[ENTRY_TAG] {
        1 => {
            let end = window.iter().position(|b| *b == 0).unwrap_or(window.len());
            if let Ok(text) = std::str::from_utf8(&window[..end]) {
                out[..text.len()].copy_from_slice(text.as_bytes());
                len = text.len();
            }
        }
        2 => {
            for pair in window.as_chunks::<2>().0.iter().take(20) {
                let unit = u16::from_le_bytes(*pair);
                if unit == 0 {
                    break;
                }
                let mut encoded = [0u8; 4];
                let Some(ch) = char::from_u32(u32::from(unit)) else {
                    return ([0; NAME_CAP], 0);
                };
                let raw = ch.encode_utf8(&mut encoded).as_bytes();
                if len + raw.len() > NAME_CAP {
                    return ([0; NAME_CAP], 0);
                }
                out[len..len + raw.len()].copy_from_slice(raw);
                len += raw.len();
            }
        }
        _ => {}
    }
    (out, len as u8)
}

/// Plans the packet edits under the gate and applies them once.
///
/// The writes are our own announcement bytes in our own outgoing packet. A
/// cleanup published before the plan drops it; one racing the store is not
/// reverted, since peer state is untouched and the packet is already
/// addressed to the departing peer (simplified 2026-10-17).
pub fn commit_outgoing(
    version_flags: u8,
    outgoing_byte: u8,
    area: &[u8],
    announce: Option<Announce>,
    interface_byte: u8,
    saved_stealth: bool,
    apply: impl FnOnce(&[Option<BodyWrite>]),
) {
    if !custom_write_allowed(version_flags) {
        return;
    }
    let send = Send {
        outgoing_byte,
        announce,
        interface_byte,
        saved_stealth,
    };
    let _ = with_table(|table| {
        let gen = SESSION_GEN.load(Ordering::Acquire);
        let mut writes = [None; 8];
        let mut count = 0usize;
        let walked = walk_messages(area, 16, |message| {
            if count >= writes.len() {
                return;
            }
            let body_at = (message.body.as_ptr() as usize).checked_sub(area.as_ptr() as usize);
            let write = match message.kind {
                Some(KIND_STATION) => {
                    let write = beacon_write(table, message.body, body_at, send);
                    if message.body.first() == Some(&1) {
                        note_request(table, true, message.body, send.saved_stealth);
                    }
                    write
                }
                Some(KIND_RTT) => payload_write(table, message.body, body_at, send),
                _ => None,
            };
            if write.is_some() {
                writes[count] = write;
                count += 1;
            }
        });
        if walked.is_ok() && SESSION_GEN.load(Ordering::Acquire) == gen {
            apply(&writes);
        }
    });
}

/// The latched session bit, or the saved one before any request latched it.
fn suppressed(table: &Table, send: Send) -> bool {
    table.session.unwrap_or(send.saved_stealth)
}

/// Response beacon tag and interface byte in the spare slot of our own
/// kind-`0x14` subtype-2 body.
fn beacon_write(table: &Table, body: &[u8], at: Option<usize>, send: Send) -> Option<BodyWrite> {
    let Some(body_at) = at else {
        return bump_none(Counter::BeaconSkips);
    };
    let tag_at = beacon_tag_offset(body)?;
    if suppressed(table, send) {
        bump(Counter::StealthSuppress);
        return bump_none(Counter::BeaconSkips);
    }
    bump(Counter::BeaconWrites);
    Some(BodyWrite {
        at: body_at + tag_at,
        bytes: [0x45, send.interface_byte, 0, 0],
        len: 2,
    })
}

/// v3 announce in the unused `body+4` of an editable RTT body, only when
/// exactly one confirmed modded peer owns the outgoing byte.
fn payload_write(table: &Table, body: &[u8], at: Option<usize>, send: Send) -> Option<BodyWrite> {
    if body.len() < RTT_BODY_MIN || !rtt_type_editable(body) {
        return None;
    }
    let Some(body_at) = at else {
        return bump_none(Counter::PayloadSkips);
    };
    let peers = table.peers.iter().flatten();
    if peers
        .filter(|p| p.modded && p.outgoing == send.outgoing_byte)
        .count()
        != 1
    {
        return bump_none(Counter::PayloadSkips);
    }
    if suppressed(table, send) {
        bump(Counter::StealthSuppress);
        return bump_none(Counter::PayloadSkips);
    }
    let Some(announce) = send.announce else {
        return bump_none(Counter::PayloadSkips);
    };
    let Some(profile) = announce.profile else {
        return bump_none(Counter::PayloadSkips);
    };
    bump(Counter::PayloadWrites);
    let bits = profile_bits(profile, announce.resolution, announce.dynamic);
    Some(BodyWrite {
        at: body_at + 4,
        bytes: [3, announce.latency as u8, bits as u8, (bits >> 8) as u8],
        len: 4,
    })
}

fn bump_none(counter: Counter) -> Option<BodyWrite> {
    bump(counter);
    None
}

/// Serializes tests that use the process-wide station table.
/// Runtime `with_table` stays non-blocking. Do not take this lock again while held.
#[cfg(test)]
pub(crate) fn global_gate() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pia_rtt::{
        pack_peer, rtt_matches, PingReading, Publication, RttIdentity, Stability,
    };
    use crate::resolution::{Level, Setting};
    use RemoteLatency::{Auto, Buffer, Unknown};
    use Selection::{Busy, Station};

    const ANN: u64 = 0x1122_3344_5566_7788;
    const TARGET: u64 = 0xaabb_ccdd_eeff_0011;
    const BOB: u64 = 0x99;
    const SETTING: Setting = Setting {
        level: Level::P1080,
        dynamic: false,
    };

    fn bits(profile: Profile) -> u16 {
        profile_bits(profile, 0, matches!(profile, Profile::Doubles))
    }

    fn ultra() -> Option<Announce> {
        Some(announce_from(4, true, Profile::Ultra, SETTING))
    }

    fn request(inverse: u8, connection: u8, inverse_connection: u8) -> Vec<u8> {
        let mut body = vec![1, 0x02, 0xaa, 0xbb, 0xcc, 0xdd, connection, inverse];
        body.extend_from_slice(&TARGET.to_be_bytes());
        body.extend_from_slice(&[1, 2, 3, 4, inverse_connection, 2, 2, 0x11, 0x22, 0x33, 0x44]);
        body.extend_from_slice(&[0u8; 6]);
        body.extend_from_slice(&ANN.to_be_bytes());
        body
    }

    /// Subtype-2 body with one named entry and `slots` location slots.
    fn response(name: &[u8], beacon: bool, slots: u8) -> Vec<u8> {
        let end = BEACON_HEADER + usize::from(slots) * LOCATION_STRIDE;
        let mut body = vec![0u8; end];
        (body[0], body[0x38], body[BEACON_HEADER + ENTRY_TAG]) = (2, 1, 1);
        body[BEACON_HEADER + ENTRY_NAME..][..name.len()].copy_from_slice(name);
        if beacon {
            (body[end - 0x54], body[end - 0x53]) = (0x45, 1);
        }
        body
    }

    fn samus() -> Vec<u8> {
        response(b"Samus", true, 2)
    }

    fn rtt_reply(latency: u8, bits: u16) -> Vec<u8> {
        let mut body = vec![0, 0, 0, 1, 3, latency, bits as u8, (bits >> 8) as u8];
        body.extend_from_slice(&[0u8; 8]);
        body
    }

    /// One framed message of `kind` with `body`, 4-aligned. The body starts at 7.
    fn area(kind: u8, body: &[u8]) -> Vec<u8> {
        let mut area = vec![0x06];
        area.extend_from_slice(&(body.len() as u16).to_be_bytes());
        area.extend_from_slice(&[kind, 0, 0, 0]);
        area.extend_from_slice(body);
        area.resize((area.len() + 3) & !3, 0);
        area
    }

    fn rtt_area(kind: u32) -> Vec<u8> {
        let mut body = kind.to_be_bytes().to_vec();
        body.resize(16, 0);
        area(KIND_RTT, &body)
    }

    fn station(source: u64, incoming: u8, body: &[u8]) {
        observe_incoming(source, incoming, KIND_STATION, body, false);
    }

    fn rtt(incoming: u8, body: &[u8]) {
        observe_incoming(1, incoming, KIND_RTT, body, false);
    }

    /// First planned write of one plaintext version-4 send.
    fn plan(
        out: u8,
        area: &[u8],
        announce: Option<Announce>,
        iface: u8,
        stealth: bool,
    ) -> Option<BodyWrite> {
        let mut first = None;
        commit_outgoing(4, out, area, announce, iface, stealth, |writes| {
            first = writes.iter().flatten().next().copied();
        });
        first
    }

    fn current() -> ShownPeer {
        peer_frame().peer
    }

    fn name(peer: &ShownPeer) -> Option<&str> {
        name_label(&peer.name, peer.name_len)
    }

    fn table<T>(read: impl FnOnce(&mut Table) -> T) -> T {
        with_table(read).expect("gate")
    }

    fn named(id: u64, name: &[u8], interface_raw: u8, profile: Profile) -> Peer {
        let mut bytes = [0u8; NAME_CAP];
        bytes[..name.len()].copy_from_slice(name);
        Peer {
            constant_id: id,
            incoming: 1,
            outgoing: 2,
            modded: true,
            interface_raw,
            name: bytes,
            name_len: name.len() as u8,
            payload: Some((0, bits(profile))),
        }
    }

    fn ann() -> Peer {
        named(11, b"Ann", 2, Profile::Vanilla)
    }

    fn bob() -> Peer {
        named(22, b"Bob", 1, Profile::LessLag)
    }

    fn seed(slot: usize, peer: Peer) {
        table(|table| {
            table.peers[slot] = Some(peer);
            resolve_selection(table);
        });
    }

    /// Fresh global table: no leaves, gate free, new session, `peers` seeded.
    fn reset(peers: &[Peer]) {
        for bits in &LEAVE_BITS {
            bits.store(0, Ordering::Relaxed);
        }
        BUSY.store(false, Ordering::Release);
        clear_session();
        for (slot, peer) in peers.iter().enumerate() {
            seed(slot, *peer);
        }
    }

    /// Samus confirmed through the handshake at incoming 0x10, outgoing 0x20.
    fn confirmed() {
        station(0, 0, &request(1, 0x10, 0x20));
        station(ANN, 0x10, &samus());
    }

    fn published(station: u8, epoch: u64) -> Publication {
        let peer = pack_peer(Stability::Stable, station);
        Publication::from_reading(PingReading::Measured(20), 10, 1, epoch, peer)
    }

    fn matches(selection: Selection, publication: Publication, epoch: u64) -> bool {
        rtt_matches(display_identity(selection), publication, epoch)
    }

    #[test]
    fn response_before_request_does_not_confirm_then_order_does_and_a_reused_byte_resets() {
        let _gate = global_gate();
        reset(&[]);
        station(ANN, 0x10, &samus());
        assert!(!current().known);
        station(0, 0, &request(1, 0x10, 0x20));
        assert_eq!(latched_stealth(), Some(false));
        station(ANN, 0x10, &samus());
        let peer = current();
        assert_eq!(name(&peer), Some("Samus"));
        assert_eq!((peer.interface, peer.settings), ("Wi-Fi", ""));
        assert_eq!(peer.latency, Unknown);
        // Outgoing requests swap the direction. An inverse flag of 0, a short
        // body and an unsupported location tag do not register.
        let out = parse_pending(true, &request(1, 0x10, 0x20)).unwrap();
        assert_eq!(out.constant_id, TARGET);
        assert_eq!((out.incoming, out.outgoing), (0x20, 0x10));
        let inbound = parse_pending(false, &request(1, 0x10, 0x20)).unwrap();
        assert_eq!(inbound.constant_id, ANN);
        assert_eq!((inbound.incoming, inbound.outgoing), (0x10, 0x20));
        assert!(parse_pending(false, &request(0, 0x10, 0x20)).is_none());
        assert!(parse_pending(false, &request(1, 1, 2)[..8]).is_none());
        let mut bad = request(1, 1, 2);
        bad[21] = 9;
        assert!(parse_pending(false, &bad).is_none());
        assert_eq!(interface_label(1, false), "Unknown");
        assert_eq!(local_link_label(3), "Unknown");
        // A new request reusing the incoming byte drops the old payload.
        rtt(0x10, &rtt_reply(7, bits(Profile::Vanilla)));
        let peer = current();
        assert_eq!((peer.settings, peer.latency), ("Vanilla", Buffer(7)));
        let mut again = request(1, 0x10, 0x21);
        again[8..16].copy_from_slice(&0x99u64.to_be_bytes());
        let loc = again.len() - 8;
        again[loc..].copy_from_slice(&0x77u64.to_be_bytes());
        station(0, 0, &again);
        station(0x77, 0x10, &response(b"Ridley", true, 2));
        let peer = current();
        assert_eq!(name(&peer), Some("Ridley"));
        assert_eq!((peer.settings, peer.latency), ("", Unknown));
        assert_eq!(table(|table| table.peers.iter().flatten().count()), 1);
    }

    #[test]
    fn a_disputed_pending_id_a_busy_gate_and_a_busy_cleanup_do_not_write() {
        let _gate = global_gate();
        reset(&[]);
        station(0, 0, &request(1, 0x10, 0x20));
        station(0, 0, &request(1, 0x12, 0x23));
        station(ANN, 0x10, &samus());
        assert!(!current().known);
        assert!(table(|table| table.pending[0]).is_some_and(|p| p.poisoned));
        reset(&[]);
        assert!(!BUSY.swap(true, Ordering::AcqRel));
        let before = counter(Counter::Contentions);
        assert!(with_table(|_| ()).is_none());
        station(0, 0, &request(1, 0x10, 0x20));
        BUSY.store(false, Ordering::Release);
        assert_eq!(counter(Counter::Contentions), before + 2);
        assert!(table(|table| table.pending.iter().all(Option::is_none)));
        // A cleanup published while the gate is busy drops the old
        // authorization on the next holder.
        station(0, 0, &request(1, 0x44, 0x55));
        station(ANN, 0x44, &samus());
        assert!(current().known);
        assert_eq!(latched_stealth(), Some(false));
        assert!(!BUSY.swap(true, Ordering::AcqRel));
        let gen = SESSION_GEN.load(Ordering::Acquire);
        clear_session();
        assert!(SESSION_GEN.load(Ordering::Acquire) > gen);
        BUSY.store(false, Ordering::Release);
        assert!(plan(0x55, &rtt_area(0), ultra(), 0, false).is_none());
        assert!(!current().known);
        assert_eq!(latched_stealth(), None);
        // An encrypted header never reaches the plan.
        let mut called = false;
        commit_outgoing(0x84, 0x55, &rtt_area(0), None, 0, false, |_| called = true);
        assert!(!called);
    }

    #[test]
    fn beacon_and_payload_follow_the_stealth_latch_and_touch_only_their_slots() {
        let _gate = global_gate();
        reset(&[]);
        let body = response(b"A", false, 2);
        let mut beacon = area(KIND_STATION, &body);
        let original = beacon.clone();
        // Before any request latches the session the saved bit decides.
        assert!(plan(0x55, &beacon, None, 1, true).is_none());
        assert_eq!(latched_stealth(), None);
        let write = plan(0x55, &beacon, None, 1, false).expect("beacon");
        assert_eq!((write.len, &write.bytes[..2]), (2, &[0x45, 1][..]));
        assert!(plan(0x55, &rtt_area(0), ultra(), 1, true).is_none());
        assert_eq!(latched_stealth(), None);
        // Once latched Off, a saved On no longer suppresses the beacon.
        observe_incoming(0, 0, KIND_STATION, &request(1, 0x10, 0x20), false);
        observe_incoming(0, 0, KIND_STATION, &request(1, 0x10, 0x20), true);
        assert_eq!(latched_stealth(), Some(false));
        assert!(plan(0x55, &beacon, None, 1, true).is_some());
        assert_eq!(stealth_label(true, Some(false)), "On (reconnect)");
        assert_eq!(stealth_label(false, Some(true)), "Off (reconnect)");
        assert_eq!(stealth_label(true, None), "On");
        // Latched On suppresses both writes.
        reset(&[]);
        observe_incoming(0, 0, KIND_STATION, &request(1, 0x44, 0x55), true);
        assert!(plan(0x55, &beacon, ultra(), 2, true).is_none());
        reset(&[]);
        station(0, 0, &request(1, 0x44, 0x55));
        station(ANN, 0x44, &response(b"A", true, 2));
        let write = plan(0x55, &beacon, ultra(), 2, false).expect("beacon");
        assert_eq!((write.len, &write.bytes[..2]), (2, &[0x45, 2][..]));
        assert_eq!(write.at, 7 + body.len() - 0x54);
        beacon[write.at..write.at + 2].copy_from_slice(&write.bytes[..2]);
        let changed = original.iter().zip(&beacon).filter(|(a, b)| a != b).count();
        assert_eq!(changed, 2);
        // The v3 payload goes to `body+4` of RTT types 0 and 1 only.
        for kind in [0u32, 1] {
            let payload = plan(0x55, &rtt_area(kind), ultra(), 0, false).expect("payload");
            assert_eq!((payload.len, payload.at), (4, 7 + 4));
            assert_eq!(&payload.bytes[..2], &[3, 4]);
            let sent = u16::from_le_bytes([payload.bytes[2], payload.bytes[3]]);
            assert_eq!(sent, bits(Profile::Ultra));
        }
        for kind in [2u32, 0x100, 0xffff_ffff] {
            assert!(plan(0x55, &rtt_area(kind), ultra(), 0, false).is_none());
        }
        let rtt_body = rtt_area(0);
        assert!(plan(0x01, &rtt_body, ultra(), 0, false).is_none());
        assert!(plan(0x55, &rtt_body, None, 0, false).is_none());
        let mut quiet = ultra().unwrap();
        quiet.profile = None;
        assert!(plan(0x55, &rtt_body, Some(quiet), 0, false).is_none());
    }

    #[test]
    fn v2_and_v3_payloads_decode_lldoubles_and_boost_without_sending_bit_14() {
        let _gate = global_gate();
        let doubles = bits(Profile::Doubles);
        assert_eq!(doubles, 3 | (1 << 2) | (1 << 11) | (1 << 13));
        assert_eq!(settings_label(doubles), "LLDoubles");
        let console_ultra = profile_bits(Profile::Ultra, 3, true);
        assert_eq!((console_ultra >> 4) & 0x7f, 3);
        assert_eq!(settings_label(console_ultra), "LLUltra");
        let less = bits(Profile::LessLag);
        assert_eq!(settings_label(less | (1 << 11)), "LessLag");
        assert_eq!(settings_label(doubles | (5 << 4)), "Other");
        assert_eq!(settings_label(doubles | (1 << 15)), "Other");
        assert_eq!(settings_label(doubles | (1 << 14)), "LLDoubles++");
        assert_eq!(settings_label(less | (1 << 14)), "LessLag++");
        let quiet_doubles = announce_from(4, true, Profile::Doubles, SETTING);
        assert_eq!(quiet_doubles.profile, Some(Profile::Doubles));
        assert!(!quiet_doubles.dynamic);
        let passthrough = announce_from(4, false, Profile::LessLag, SETTING);
        assert_eq!((passthrough.profile, passthrough.latency), (None, 4));
        reset(&[]);
        confirmed();
        let rtt_body = rtt_area(0);
        let sent = plan(0x20, &rtt_body, Some(quiet_doubles), 0, false).expect("payload");
        let sent = u16::from_le_bytes([sent.bytes[2], sent.bytes[3]]);
        assert_eq!(sent & ((1 << 11) | (1 << 14)), 0);
        assert_eq!(settings_label(sent), "LLDoubles");
        assert!(plan(0x20, &rtt_body, Some(passthrough), 0, false).is_none());
        let mut v2 = vec![0, 0, 0, 1, 2, 5, 0xff, 0xff];
        v2.extend_from_slice(&[0u8; 8]);
        rtt(0x10, &v2);
        assert_eq!(current().settings, "Other");
        let boosted = less | (1 << 14);
        v2[4..8].copy_from_slice(&[2, 1, boosted as u8, (boosted >> 8) as u8]);
        rtt(0x10, &v2);
        assert_eq!(current().settings, "LessLag");
        rtt(0x10, &rtt_reply(1, boosted));
        assert_eq!(current().settings, "LessLag++");
        rtt(0x10, &rtt_reply(4, doubles));
        let peer = current();
        assert_eq!((peer.settings, peer.latency), ("LLDoubles", Buffer(4)));
        let cases = [(255, Auto), (25, Buffer(25)), (26, Unknown), (254, Unknown)];
        for (byte, latency) in cases {
            rtt(0x10, &rtt_reply(byte, less));
            assert_eq!(current().latency, latency);
        }
        // A reply whose incoming byte matches no modded peer changes nothing.
        rtt(0x11, &rtt_reply(9, doubles));
        let peer = current();
        assert_eq!((peer.settings, peer.latency), ("LessLag", Unknown));
        assert_eq!(remote_latency_label(Unknown), "---");
        assert_eq!(remote_latency_label(Auto), "Auto");
        assert_eq!(remote_latency_label(Buffer(25)), "25");
    }

    #[test]
    fn one_selected_station_owns_the_name_and_cycles_without_the_local_index() {
        let _gate = global_gate();
        reset(&[ann()]);
        let alone = peer_frame();
        assert!(alone.peer.known && alone.peer.station.is_none());
        assert_eq!(alone.selection, Selection::None);
        assert_eq!(name(&alone.peer), Some("Ann"));
        let mut plain = ann();
        plain.modded = false;
        seed(0, plain);
        let peer = current();
        assert_eq!((peer.interface, peer.latency), ("Unknown", Unknown));
        reset(&[ann(), bob()]);
        assert!(!current().known);
        note_local(0);
        bind_station(0, 11);
        bind_station(4, 11);
        bind_station(9, 22);
        bind_station(253, 22);
        let frame = peer_frame();
        assert_eq!(frame.selection, Station(4));
        let peer = frame.peer;
        assert_eq!(name(&peer), Some("Ann"));
        assert_eq!((peer.interface, peer.settings), ("Ethernet", "Vanilla"));
        cycle_peer(1);
        let peer = current();
        assert_eq!((peer.station, name(&peer)), (Some(9), Some("Bob")));
        assert_eq!((peer.interface, peer.settings), ("Wi-Fi", "LessLag"));
        cycle_peer(-1);
        assert_eq!(current().station, Some(4));
        cycle_peer(0);
        assert_eq!(current().station, Some(4));
        table(|table| table.index_id[4] = 0);
        assert_eq!(current().station, Some(9));
        seed(0, named(33, b"Cora", 2, Profile::Ultra));
        bind_station(4, 33);
        assert_eq!(current().station, Some(9));
        cycle_peer(1);
        let peer = current();
        assert_eq!((name(&peer), peer.settings), (Some("Cora"), "LLUltra"));
        let mut bad = named(44, &[0xff, 0xfe], 1, Profile::Vanilla);
        bad.payload = None;
        seed(2, bad);
        bind_station(5, 44);
        cycle_peer(1);
        let peer = current();
        assert_eq!((peer.station, peer.settings), (Some(5), ""));
        assert!(name(&peer).is_none());
        let (found, count) = table(|table| candidates(table));
        assert_eq!(&found[..count], &[4, 5, 9]);
    }

    #[test]
    fn roster_skips_local_and_unmapped_and_a_busy_snapshot_copies_no_names() {
        let _gate = global_gate();
        let mut ann = ann();
        ann.payload = Some((4, bits(Profile::Vanilla)));
        let mut bob = bob();
        bob.payload = Some((255, bits(Profile::LessLag)));
        reset(&[ann, bob, named(33, b"Cyd", 1, Profile::Ultra)]);
        note_local(1);
        bind_station(1, 11);
        bind_station(4, 11);
        bind_station(9, 22);
        bind_station(300, 22);
        bind_station(5, 0);
        let (frame, roster) = banner_peers();
        assert_eq!(frame.selection, Station(4));
        assert!(!roster.busy);
        assert_eq!(roster.len, 2);
        let (first, second) = (&roster.peers[0], &roster.peers[1]);
        assert_eq!((first.station, name(first)), (Some(4), Some("Ann")));
        assert_eq!((first.settings, first.latency), ("Vanilla", Buffer(4)));
        assert_eq!((second.station, name(second)), (Some(9), Some("Bob")));
        assert_eq!((second.settings, second.latency), ("LessLag", Auto));
        assert!(roster.peers[2..].iter().all(|peer| !peer.known));
        assert!(!BUSY.swap(true, Ordering::AcqRel));
        let (frame, roster) = banner_peers();
        assert_eq!((selection_snapshot(), latched_stealth()), (Busy, None));
        assert!(!opponent_connected());
        BUSY.store(false, Ordering::Release);
        assert!(roster.busy);
        assert_eq!(roster.len, 0);
        assert!(roster.peers.iter().all(|peer| !peer.known));
        assert_eq!(frame.selection, Busy);
        assert!(!frame.peer.known);
        assert_eq!(display_identity(Busy), RttIdentity::Busy);
    }

    #[test]
    fn leaves_retire_the_only_confirmed_peer_and_identified_disconnects_retire_aliases() {
        let _gate = global_gate();
        reset(&[]);
        confirmed();
        note_local(1);
        // No station binding yet: Samus shows through the single-peer
        // fallback without a station selection.
        let frame = peer_frame();
        assert!(frame.peer.known && !station_connected(frame.selection));
        assert_eq!(frame.selection, Selection::None);
        // The opponent declines the rematch: the game reports station 4
        // leaving, which nothing bound. The only confirmed peer is them.
        clear_station(4);
        assert!(!current().known);
        // Two confirmed peers: an unbound leave cannot tell them apart and
        // retires neither.
        seed(0, named(0x55, b"Ann", 1, Profile::Vanilla));
        seed(1, named(BOB, b"Bob", 1, Profile::LessLag));
        clear_station(70);
        assert_eq!(table(|table| table.peers.iter().flatten().count()), 2);
        // Kind 0x14 subtype 3/4 from a confirmed id retires every alias of
        // that id; a later join shows the peer again.
        reset(&[]);
        confirmed();
        note_local(1);
        bind_station(4, ANN);
        rtt(0x10, &rtt_reply(4, bits(Profile::Vanilla)));
        let frame = peer_frame();
        assert_eq!(frame.selection, Station(4));
        assert_eq!(name(&frame.peer), Some("Samus"));
        assert_eq!(frame.peer.settings, "Vanilla");
        assert!(opponent_column_visible(false, frame.selection));
        assert!(!opponent_column_visible(true, frame.selection));
        bind_station(7, ANN);
        station(ANN, 0, &[4, 0]);
        let gone = peer_frame();
        assert_eq!(gone.selection, Selection::None);
        assert!(!gone.peer.known);
        let ids = table(|table| (table.index_id[4], table.index_id[7]));
        assert_eq!(ids, (0, 0));
        seed(1, named(BOB, b"Bob", 1, Profile::LessLag));
        bind_station(9, BOB);
        station(0, 0, &[3]);
        let kept = peer_frame();
        assert_eq!(kept.selection, Station(9));
        assert_eq!(name(&kept.peer), Some("Bob"));
        station(BOB, 0, &[3]);
        assert!(!current().known);
        station(0, 0, &request(1, 0x30, 0x40));
        station(ANN, 0x30, &samus());
        bind_station(4, ANN);
        let back = peer_frame();
        assert_eq!(back.selection, Station(4));
        assert_eq!(name(&back.peer), Some("Samus"));
        assert_eq!(display_identity(back.selection), RttIdentity::Station(4));
        assert_eq!(display_identity(Selection::None), RttIdentity::Baseline);
    }

    #[test]
    fn station_leaves_drop_aliases_and_cycling_or_leaving_hides_the_rtt_until_republish() {
        let _gate = global_gate();
        reset(&[ann(), bob()]);
        note_local(1);
        bind_station(4, 11);
        bind_station(9, 22);
        let frame = peer_frame();
        assert_eq!(frame.selection, Station(4));
        assert_eq!(frame.peer.settings, "Vanilla");
        assert!(opponent_connected());
        let old = published(4, 3);
        assert!(matches(frame.selection, old, 3));
        cycle_peer(1);
        let frame = peer_frame();
        assert_eq!(frame.selection, Station(9));
        assert_eq!(name(&frame.peer), Some("Bob"));
        assert!(!matches(frame.selection, old, 3));
        let republished = published(9, 3);
        assert!(matches(frame.selection, republished, 3));
        // Bob leaves: the selection falls back to Ann and his ping is hidden
        // until the poll republishes hers.
        clear_station(9);
        let frame = peer_frame();
        assert_eq!(frame.selection, Station(4));
        assert!(!matches(frame.selection, republished, 3));
        seed(1, bob());
        bind_station(9, 22);
        bind_station(7, 11);
        assert_eq!(selection_snapshot(), Station(4));
        // A leave published while the gate is busy lands on the next holder
        // and takes Ann's other alias with her; Bob stays.
        assert!(!BUSY.swap(true, Ordering::AcqRel));
        clear_station(7);
        assert_eq!(selection_snapshot(), Busy);
        BUSY.store(false, Ordering::Release);
        let frame = peer_frame();
        assert_eq!(frame.selection, Station(9));
        assert_eq!(frame.peer.settings, "LessLag");
        let ids = table(|table| (table.index_id[4], table.index_id[7]));
        assert_eq!(ids, (0, 0));
        clear_station(9);
        let frame = peer_frame();
        assert_eq!(frame.selection, Selection::None);
        assert!(!frame.peer.known && !opponent_connected());
        assert!(!opponent_column_visible(false, frame.selection));
        clear_station(300);
        clear_session();
        assert!(!matches(peer_frame().selection, republished, 4));
        assert!(matches(Selection::None, published(2, 4), 4));
        assert!(!matches(Busy, published(2, 4), 4));
    }
}
