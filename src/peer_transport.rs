//! Manager connection-id handshake and the fork's in-payload announcements.
//!
//! Field positions are the manager instructions cited in
//! `grok-connection-id-implementation.md`. Header u64s are not connection keys.
//! Hooks call this module; nothing here waits, allocates, or writes a file.
#![cfg_attr(not(target_os = "switch"), allow(dead_code))]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::peer_observer::mix_id;
use crate::peer_passive::{read_response_beacon, walk_messages, Beacon, ForkPayload};

const SLOTS: usize = 8;
const NAME_CAP: usize = 64;
const LOCATION_STRIDE: usize = 0xc3;
const ENTRY_NAME: usize = 0x51;
const ENTRY_NAME_LEN: usize = 0x28;
const ENTRY_TAG: usize = 0x79;
const BEACON_HEADER: usize = 0x39;
const KIND_STATION: u8 = 0x14;
const KIND_RTT: u8 = 0x58;
const RTT_BODY_MIN: usize = 0x10;

static BUSY: AtomicBool = AtomicBool::new(false);
struct TableCell(std::cell::UnsafeCell<Table>);
unsafe impl Sync for TableCell {}
static TABLE: TableCell = TableCell(std::cell::UnsafeCell::new(Table::empty()));
static LEARNED: AtomicU64 = AtomicU64::new(0);
static AMBIGUOUS: AtomicU64 = AtomicU64::new(0);
static CONFIRMED: AtomicU64 = AtomicU64::new(0);
static CLEANUPS: AtomicU64 = AtomicU64::new(0);
static BEACON_WRITES: AtomicU64 = AtomicU64::new(0);
static BEACON_SKIPS: AtomicU64 = AtomicU64::new(0);
static PAYLOAD_WRITES: AtomicU64 = AtomicU64::new(0);
static PAYLOAD_SKIPS: AtomicU64 = AtomicU64::new(0);
static STEALTH_SUPPRESS: AtomicU64 = AtomicU64::new(0);
static CONTENTIONS: AtomicU64 = AtomicU64::new(0);
static LAST_MIX: AtomicU64 = AtomicU64::new(0);
/// Bumped by every cleanup, including when `BUSY` is already held.
/// A later gate holder drops the table before any read or write.
static SESSION_GEN: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestParse {
    NotRequest,
    Truncated,
    LocationUnsupported,
    Ready(ConnectionRequest),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConnectionRequest {
    pub connection_id: u8,
    pub is_inverse: u8,
    pub target_constant_id: u64,
    pub inverse_connection_id: u8,
    pub location_constant_id: u64,
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
    payload_known: bool,
    latency_bits: u8,
    profile_bits: u16,
}

#[derive(Clone, Copy, Debug)]
struct Table {
    epoch: u64,
    /// `SESSION_GEN` value already applied to this table.
    applied_gen: u64,
    session: bool,
    latched_stealth: bool,
    pending: [Option<Pending>; SLOTS],
    peers: [Option<Peer>; SLOTS],
    /// Station index `0..=252` to constant id. `0` means unmapped.
    index_id: [u64; 253],
    /// `255` means no selection.
    selected: u8,
    /// `255` means the local index is not known yet.
    local_index: u8,
}

impl Table {
    const fn empty() -> Self {
        Self {
            epoch: 1,
            applied_gen: 1,
            session: false,
            latched_stealth: false,
            pending: [None; SLOTS],
            peers: [None; SLOTS],
            index_id: [0; 253],
            selected: NONE_STATION,
            local_index: NONE_STATION,
        }
    }
}

const NONE_STATION: u8 = 255;
static LEAVE_BITS: [AtomicU64; 4] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnounceMode {
    Vanilla,
    LessLag,
    Ultra,
    /// Fork `RenderProfileSettings::less_lag_doubles`. Not `doubles_core`.
    /// Announced only when the live machine has committed that profile.
    LessLagDoubles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Announce {
    pub latency: i8,
    pub profile: Option<AnnounceMode>,
    /// Ultelier resolution discriminant. Zero when no profile is sent.
    pub resolution: u8,
    /// Actual dynamic-resolution state. False when the resolution guard rejected.
    pub dynamic: bool,
    /// Fork bit 14. Set only when this client has read back the native-window write.
    pub boost_applied: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyWrite {
    pub at: usize,
    pub bytes: [u8; 4],
    pub len: u8,
}

#[derive(Clone, Copy, Debug)]
struct View {
    known: u8,
    name: [u8; NAME_CAP],
    name_len: u8,
    interface_raw: u8,
    modded: u8,
    settings: u8,
    /// `254` is cleared. `255` is Auto. `0..=25` is the announced buffer.
    latency: u8,
    latched: u8,
    station: u8,
}

impl View {
    const fn empty() -> Self {
        Self {
            known: 0,
            name: [0; NAME_CAP],
            name_len: 0,
            interface_raw: 0,
            modded: 0,
            settings: 0,
            latency: LATENCY_CLEARED,
            latched: 0,
            station: NONE_STATION,
        }
    }
}

/// Wire byte that is not Auto and not a buffer of 0..=25.
const LATENCY_CLEARED: u8 = 254;

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

/// Mapped remotes copied under one gate hold. Eight is the handshake table.
pub const REMOTE_CAP: usize = SLOTS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteRoster {
    pub peers: [ShownPeer; REMOTE_CAP],
    pub len: u8,
    pub busy: bool,
}

impl RemoteRoster {
    const fn blank(busy: bool) -> Self {
        Self {
            peers: [blank_peer(); REMOTE_CAP],
            len: 0,
            busy,
        }
    }
}

const fn blank_peer() -> ShownPeer {
    ShownPeer {
        known: false,
        name: [0; NAME_CAP],
        name_len: 0,
        interface: "Unknown",
        settings: "",
        latency: RemoteLatency::Unknown,
        station: None,
    }
}

/// Announced input buffer for the peer in this snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteLatency {
    /// Cleanup, no chosen peer, unmodded, no payload, or a byte outside Auto and 0..=25.
    Unknown,
    Auto,
    Buffer(u8),
}

/// Player Info text. Streamer draws a blank instead of calling this.
pub fn remote_latency_label(latency: RemoteLatency) -> String {
    match latency {
        RemoteLatency::Unknown => "---".to_owned(),
        RemoteLatency::Auto => "Auto".to_owned(),
        RemoteLatency::Buffer(value) => value.to_string(),
    }
}

fn display_latency(bits: u8) -> u8 {
    match bits {
        0..=25 | 255 => bits,
        _ => LATENCY_CLEARED,
    }
}

fn latency_from_byte(byte: u8) -> RemoteLatency {
    match byte {
        255 => RemoteLatency::Auto,
        0..=25 => RemoteLatency::Buffer(byte),
        _ => RemoteLatency::Unknown,
    }
}

/// Owned name bytes. Invalid UTF-8 is absent rather than a torn borrow.
pub fn name_label(name: &[u8], len: u8) -> Option<&str> {
    let end = usize::from(len).min(name.len());
    let bytes = &name[..end];
    if bytes.is_empty() {
        return None;
    }
    std::str::from_utf8(bytes).ok()
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

/// Wire profile while the machine is live, in a match or in menus.
///
/// A passthrough machine sends no profile, so a saved menu mode is not
/// advertised. `Profile::Doubles` is fork LLDoubles. Resolution and dynamic
/// resolution come from [`Announce`], not from a guessed preset.
pub fn announced_profile(
    live: bool,
    committed: crate::render_policy::Profile,
) -> Option<AnnounceMode> {
    if !live {
        return None;
    }
    Some(match committed {
        crate::render_policy::Profile::Vanilla => AnnounceMode::Vanilla,
        crate::render_policy::Profile::LessLag => AnnounceMode::LessLag,
        crate::render_policy::Profile::Ultra => AnnounceMode::Ultra,
        crate::render_policy::Profile::Doubles => AnnounceMode::LessLagDoubles,
    })
}

/// Fork `RenderProfileSettings::to_bits` without bit 14.
///
/// Buffer, index, vsync, and render opts follow the committed profile. The
/// resolution discriminant and dynamic-resolution bit are the configured
/// state passed in. [`encode_v3`] sets bit 14 only when the boost write
/// read back as applied.
pub fn profile_bits(mode: AnnounceMode, resolution: u8, dynamic: bool) -> u16 {
    let (buffer, index, vsync, opts) = match mode {
        AnnounceMode::Vanilla => (3u16, 2u16, 1u16, 0u16),
        AnnounceMode::LessLag => (2, 1, 0, 1),
        AnnounceMode::Ultra => (2, 0, 0, 1),
        AnnounceMode::LessLagDoubles => (3, 1, 0, 1),
    };
    let resolution = u16::from(resolution.min(4));
    let dynamic = u16::from(dynamic);
    buffer | (index << 2) | (resolution << 4) | (dynamic << 11) | (vsync << 12) | (opts << 13)
}

pub fn announce_from(
    latency: i8,
    live: bool,
    committed: crate::render_policy::Profile,
    setting: crate::resolution::Setting,
    boost_applied: bool,
) -> Announce {
    let profile = announced_profile(live, committed);
    Announce {
        latency,
        profile,
        resolution: if live { setting.level as u8 } else { 0 },
        dynamic: live && setting.dynamic,
        boost_applied: profile.is_some() && boost_applied,
    }
}

/// Fork `from_settings` names a preset from buffer, index, vsync, and render
/// opts after `from_bits` accepts the resolution. Dynamic resolution may differ
/// between console and emulator singles. Bit 14 is the fork boost bit and does
/// not select another preset. Bit 15 and an out-of-range resolution stay Other.
pub fn settings_label(bits: u16) -> &'static str {
    let resolution = (bits >> 4) & 0x7f;
    if (bits & (1 << 15)) != 0 || resolution > 4 {
        return "Other";
    }
    match (
        bits & 0b11,
        (bits >> 2) & 0b11,
        (bits >> 12) & 1,
        (bits >> 13) & 1,
    ) {
        (3, 2, 1, 0) => "Vanilla",
        (2, 1, 0, 1) => "LessLag",
        (2, 0, 0, 1) => "LLUltra",
        (3, 1, 0, 1) => "LLDoubles",
        _ => "Other",
    }
}

fn settings_code(bits: u16) -> u8 {
    let base = match settings_label(bits) {
        "Vanilla" => 1,
        "LessLag" => 2,
        "LLUltra" => 3,
        "Other" => 4,
        "LLDoubles" => 5,
        _ => 0,
    };
    if base != 0 && base != 4 && (bits & (1 << 14)) != 0 {
        base | 0x80
    } else {
        base
    }
}

fn label_from_code(code: u8) -> &'static str {
    match code {
        1 => "Vanilla",
        2 => "LessLag",
        3 => "LLUltra",
        4 => "Other",
        5 => "LLDoubles",
        0x81 => "Vanilla++",
        0x82 => "LessLag++",
        0x83 => "LLUltra++",
        0x85 => "LLDoubles++",
        _ => "",
    }
}

pub fn encode_v3(announce: Announce) -> [u8; 4] {
    let mut bits = match announce.profile {
        Some(mode) => profile_bits(mode, announce.resolution, announce.dynamic),
        None => 0,
    };
    if announce.boost_applied && announce.profile.is_some() {
        bits |= 1 << 14;
    }
    [3, announce.latency as u8, bits as u8, (bits >> 8) as u8]
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
    if !modded {
        return "Unknown";
    }
    local_link_label(raw)
}

fn enter() -> bool {
    BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

fn leave() {
    BUSY.store(false, Ordering::Release);
}

fn with_table<T>(update: impl FnOnce(&mut Table) -> T) -> Option<T> {
    if !enter() {
        CONTENTIONS.fetch_add(1, Ordering::Relaxed);
        return None;
    }
    let table = unsafe { &mut *TABLE.0.get() };
    apply_cleanup(table);
    apply_leaves(table);
    let value = update(table);
    leave();
    Some(value)
}

/// UI snapshot. Contention shows nothing rather than copying the table unsynchronized.
fn try_view() -> Option<View> {
    if !enter() {
        return None;
    }
    let table = unsafe { &mut *TABLE.0.get() };
    apply_cleanup(table);
    apply_leaves(table);
    let view = view_of(table);
    leave();
    Some(view)
}

fn apply_cleanup(table: &mut Table) {
    let gen = SESSION_GEN.load(Ordering::Acquire);
    if table.applied_gen == gen {
        return;
    }
    let next = table.epoch.wrapping_add(1);
    *table = Table::empty();
    table.epoch = next;
    table.applied_gen = gen;
}

fn view_of(table: &mut Table) -> View {
    resolve_selection(table);
    let mut confirmed = 0usize;
    let mut only = None;
    for peer in table.peers.iter().flatten() {
        confirmed += 1;
        only = Some(*peer);
    }
    let mut view = View::empty();
    view.latched = if table.session {
        if table.latched_stealth {
            2
        } else {
            1
        }
    } else {
        0
    };
    let chosen = selected_peer(table).or(if confirmed == 1 { only } else { None });
    if let Some(peer) = chosen {
        view.known = 1;
        view.name = peer.name;
        view.name_len = peer.name_len;
        view.interface_raw = peer.interface_raw;
        view.modded = u8::from(peer.modded);
        view.settings = if peer.payload_known {
            settings_code(peer.profile_bits)
        } else {
            0
        };
        view.latency = if peer.modded && peer.payload_known {
            display_latency(peer.latency_bits)
        } else {
            LATENCY_CLEARED
        };
        view.station = table.selected;
    }
    view
}

fn station_ok(index: u32) -> Option<u8> {
    if index <= 252 {
        Some(index as u8)
    } else {
        None
    }
}

fn apply_leaves(table: &mut Table) {
    let mut seen = [0u64; 4];
    let mut any = false;
    for (word, bits) in seen.iter_mut().zip(LEAVE_BITS.iter()) {
        *word = bits.load(Ordering::Acquire);
        any |= *word != 0;
    }
    if !any {
        return;
    }
    for index in 0..253u16 {
        let word = usize::from(index) / 64;
        let bit = index % 64;
        if seen[word] & (1u64 << bit) != 0 {
            let id = table.index_id[usize::from(index)];
            table.index_id[usize::from(index)] = 0;
            if table.selected == index as u8 {
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
    for (word, bits) in seen.iter().zip(LEAVE_BITS.iter()) {
        if *word != 0 {
            bits.fetch_and(!word, Ordering::AcqRel);
        }
    }
    resolve_selection(table);
}

/// A station left that no constant id was ever bound to (the station map
/// hook is part of the opt-in peer observer, so the binding is usually
/// absent). With exactly one confirmed peer that peer is the opponent who
/// left: an opponent who declines a rematch otherwise stays on the banner
/// through the single-peer fallback until the whole session ends.
fn retire_only_confirmed(table: &mut Table) {
    let mut confirmed = table.peers.iter().flatten();
    let (Some(only), None) = (confirmed.next(), confirmed.next()) else {
        return;
    };
    retire_constant(table, only.constant_id);
}

fn bind_index(table: &mut Table, index: u8, constant_id: u64) {
    if index > 252 || constant_id == 0 {
        return;
    }
    table.index_id[usize::from(index)] = constant_id;
    resolve_selection(table);
}

fn note_local_index(table: &mut Table, index: u8) {
    if index > 252 {
        return;
    }
    table.local_index = index;
    resolve_selection(table);
}

fn candidates(table: &Table) -> ([u8; 253], usize) {
    let mut out = [0u8; 253];
    let mut count = 0usize;
    for index in 0..=252u8 {
        if index == table.local_index || count == out.len() {
            continue;
        }
        let id = table.index_id[usize::from(index)];
        if id != 0 && peer_by_id(table, id).is_some() {
            out[count] = index;
            count += 1;
        }
    }
    (out, count)
}

fn peer_by_id(table: &Table, id: u64) -> Option<Peer> {
    table
        .peers
        .iter()
        .flatten()
        .find(|peer| peer.constant_id == id)
        .copied()
}

fn selected_peer(table: &Table) -> Option<Peer> {
    let index = table.selected;
    if index > 252 {
        return None;
    }
    let id = table.index_id[usize::from(index)];
    if id == 0 {
        return None;
    }
    peer_by_id(table, id)
}

fn resolve_selection(table: &mut Table) {
    let (found, count) = candidates(table);
    if count == 0 {
        table.selected = NONE_STATION;
        return;
    }
    if found[..count].contains(&table.selected) {
        return;
    }
    table.selected = found[0];
}

fn cycle_table(table: &mut Table, direction: i8) {
    if direction == 0 {
        return;
    }
    let (found, count) = candidates(table);
    if count < 2 {
        resolve_selection(table);
        return;
    }
    let current = found[..count]
        .iter()
        .position(|index| *index == table.selected)
        .unwrap_or(0);
    let step = if direction < 0 { count - 1 } else { 1 };
    table.selected = found[(current + step) % count];
}

/// Always-on station map. Diagnostic copies are not consulted.
pub fn bind_station(index: u32, constant_id: u64) {
    let Some(index) = station_ok(index) else {
        return;
    };
    let _ = with_table(|table| bind_index(table, index, constant_id));
}

pub fn note_local(index: u32) {
    let Some(index) = station_ok(index) else {
        return;
    };
    let _ = with_table(|table| note_local_index(table, index));
}

/// Publishes the leave even when the gate is busy. The next holder drops it.
pub fn clear_station(index: u32) {
    let Some(index) = station_ok(index) else {
        return;
    };
    let word = usize::from(index) / 64;
    LEAVE_BITS[word].fetch_or(1u64 << (u32::from(index) % 64), Ordering::Release);
    if !enter() {
        return;
    }
    let table = unsafe { &mut *TABLE.0.get() };
    apply_cleanup(table);
    apply_leaves(table);
    leave();
}

pub fn cycle_peer(direction: i8) {
    let _ = with_table(|table| cycle_table(table, direction));
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

pub fn peer_frame() -> PeerFrame {
    let Some(view) = try_view() else {
        return PeerFrame {
            selection: Selection::Busy,
            peer: unknown_peer(),
        };
    };
    frame_from(view)
}

fn frame_from(view: View) -> PeerFrame {
    let station = if view.station <= 252 {
        Some(view.station)
    } else {
        None
    };
    let peer = ShownPeer {
        known: view.known == 1,
        name: view.name,
        name_len: view.name_len,
        interface: interface_label(view.interface_raw, view.modded == 1),
        settings: label_from_code(view.settings),
        latency: latency_from_byte(view.latency),
        station: if view.known == 1 { station } else { None },
    };
    let selection = match (view.known == 1, station) {
        (true, Some(index)) => Selection::Station(index),
        _ => Selection::None,
    };
    PeerFrame { selection, peer }
}

/// Stations the RTT poll may publish, copied under the same gate as the names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicationTargets {
    pub selected: crate::pia_rtt::StationPick,
    pub stations: [u8; REMOTE_CAP],
    pub len: u8,
    pub busy: bool,
}

pub fn publication_targets() -> PublicationTargets {
    match try_banner_peers() {
        None => PublicationTargets {
            selected: crate::pia_rtt::StationPick::Busy,
            stations: [0; REMOTE_CAP],
            len: 0,
            busy: true,
        },
        Some((view, roster)) => {
            let selected = match frame_from(view).selection {
                Selection::Station(index) => crate::pia_rtt::StationPick::Index(index),
                Selection::None => crate::pia_rtt::StationPick::Absent,
                Selection::Busy => crate::pia_rtt::StationPick::Busy,
            };
            let mut stations = [0u8; REMOTE_CAP];
            let mut len = 0u8;
            for peer in roster.peers.iter().take(usize::from(roster.len)) {
                let Some(index) = peer.station else {
                    continue;
                };
                if usize::from(len) >= stations.len() {
                    break;
                }
                stations[usize::from(len)] = index;
                len = len.saturating_add(1);
            }
            PublicationTargets {
                selected,
                stations,
                len,
                busy: roster.busy,
            }
        }
    }
}

/// Selected peer and every mapped remote, copied under one gate hold.
/// Busy carries no names. Local and unmapped peers are omitted from the roster.
pub fn banner_peers() -> (PeerFrame, RemoteRoster) {
    match try_banner_peers() {
        Some((view, roster)) => (frame_from(view), roster),
        None => (
            PeerFrame {
                selection: Selection::Busy,
                peer: unknown_peer(),
            },
            RemoteRoster::blank(true),
        ),
    }
}

fn try_banner_peers() -> Option<(View, RemoteRoster)> {
    if !enter() {
        return None;
    }
    let table = unsafe { &mut *TABLE.0.get() };
    apply_cleanup(table);
    apply_leaves(table);
    let roster = roster_of(table);
    let view = view_of(table);
    leave();
    Some((view, roster))
}

fn roster_of(table: &mut Table) -> RemoteRoster {
    resolve_selection(table);
    let (found, count) = candidates(table);
    let mut roster = RemoteRoster::blank(false);
    for index in found.into_iter().take(count) {
        if usize::from(roster.len) >= REMOTE_CAP {
            break;
        }
        let id = table.index_id[usize::from(index)];
        let Some(peer) = peer_by_id(table, id) else {
            continue;
        };
        roster.peers[usize::from(roster.len)] = show_station(&peer, index);
        roster.len += 1;
    }
    roster
}

fn show_station(peer: &Peer, station: u8) -> ShownPeer {
    ShownPeer {
        known: true,
        name: peer.name,
        name_len: peer.name_len,
        interface: interface_label(peer.interface_raw, peer.modded),
        settings: if peer.payload_known {
            label_from_code(settings_code(peer.profile_bits))
        } else {
            ""
        },
        latency: if peer.modded && peer.payload_known {
            latency_from_byte(display_latency(peer.latency_bits))
        } else {
            RemoteLatency::Unknown
        },
        station: Some(station),
    }
}

/// Snapshot under the same nonblocking gate as the name. Busy is not `None`.
pub fn selection_snapshot() -> Selection {
    peer_frame().selection
}

#[cfg(test)]
pub fn selected_station() -> Option<u8> {
    match selection_snapshot() {
        Selection::Station(index) => Some(index),
        Selection::None | Selection::Busy => None,
    }
}

/// Streamer hides remote rows. It is not an input to packet planning.
pub fn remote_rows_visible(streamer: bool) -> bool {
    !streamer
}

/// A mapped station is connected even while its ping is still pending.
pub fn station_connected(selection: Selection) -> bool {
    matches!(selection, Selection::Station(_))
}

/// Opponent column, ping cell, and remote banner line. Absent and busy stay blank.
pub fn opponent_column_visible(streamer: bool, selection: Selection) -> bool {
    remote_rows_visible(streamer) && station_connected(selection)
}

pub fn opponent_connected() -> bool {
    station_connected(peer_frame().selection)
}

/// Draw rule for the publication sitting next to this snapshot.
pub fn display_identity(selection: Selection) -> crate::pia_rtt::RttIdentity {
    match selection {
        Selection::Station(index) => crate::pia_rtt::RttIdentity::Station(index),
        Selection::None => crate::pia_rtt::RttIdentity::Baseline,
        Selection::Busy => crate::pia_rtt::RttIdentity::Busy,
    }
}

fn unknown_peer() -> ShownPeer {
    blank_peer()
}

pub fn latched_stealth() -> Option<bool> {
    let view = try_view()?;
    match view.latched {
        1 => Some(false),
        2 => Some(true),
        _ => None,
    }
}

/// Drops authorization even when the table gate is busy. The generation is
/// published before the attempt, so the next holder cannot use the old peers.
pub fn clear_session() {
    SESSION_GEN.fetch_add(1, Ordering::Release);
    CLEANUPS.fetch_add(1, Ordering::Relaxed);
    if !enter() {
        CONTENTIONS.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let table = unsafe { &mut *TABLE.0.get() };
    apply_cleanup(table);
    apply_leaves(table);
    leave();
}

#[cfg(test)]
fn clear_table(table: &mut Table) {
    let next = table.epoch.wrapping_add(1);
    *table = Table::empty();
    table.epoch = next;
    CLEANUPS.fetch_add(1, Ordering::Relaxed);
}

#[cfg(test)]
fn seed_global_peer(slot: usize, peer: Peer) {
    let _ = with_table(|table| {
        if slot < SLOTS {
            table.peers[slot] = Some(peer);
            resolve_selection(table);
        }
    });
}

pub fn observe_incoming(
    source_id: u64,
    incoming_byte: u8,
    kind: u8,
    body: &[u8],
    saved_stealth: bool,
) {
    let _ = with_table(|table| {
        observe_incoming_table(table, source_id, incoming_byte, kind, body, saved_stealth)
    });
}

fn observe_incoming_table(
    table: &mut Table,
    source_id: u64,
    incoming_byte: u8,
    kind: u8,
    body: &[u8],
    saved_stealth: bool,
) {
    if kind == KIND_STATION {
        let subtype = body.first().copied().unwrap_or(0);
        if subtype == 1 {
            note_request(table, Direction::Incoming, body, saved_stealth);
        } else if subtype == 2 {
            note_response(table, source_id, body);
        } else if subtype == 3 || subtype == 4 {
            forget_id(table, source_id);
        }
        return;
    }
    if kind == KIND_RTT {
        note_rtt(table, incoming_byte, body);
    }
}

#[cfg(test)]
pub fn plan_outgoing(
    outgoing_byte: u8,
    area: &[u8],
    announce: Option<Announce>,
    interface_byte: u8,
    saved_stealth: bool,
) -> [Option<BodyWrite>; 8] {
    with_table(|table| {
        plan_outgoing_table(
            table,
            outgoing_byte,
            area,
            announce,
            interface_byte,
            saved_stealth,
        )
    })
    .unwrap_or([None; 8])
}

/// Plans and applies packet edits while the gate is held.
///
/// `edit(writes, false)` stores the planned bytes. `edit(writes, true)` puts
/// the previous bytes back. A cleanup generation published before either call
/// drops the plan; one published during the store is reverted before return.
pub fn commit_outgoing(
    version_flags: u8,
    outgoing_byte: u8,
    area: &[u8],
    announce: Option<Announce>,
    interface_byte: u8,
    saved_stealth: bool,
    mut edit: impl FnMut(&[Option<BodyWrite>], bool),
) {
    if !crate::peer_passive::custom_write_allowed(version_flags) {
        return;
    }
    if !enter() {
        CONTENTIONS.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let table = unsafe { &mut *TABLE.0.get() };
    apply_cleanup(table);
    apply_leaves(table);
    let gen = SESSION_GEN.load(Ordering::Acquire);
    let writes = plan_outgoing_table(
        table,
        outgoing_byte,
        area,
        announce,
        interface_byte,
        saved_stealth,
    );
    if SESSION_GEN.load(Ordering::Acquire) != gen {
        apply_cleanup(table);
        apply_leaves(table);
        leave();
        return;
    }
    edit(&writes, false);
    if SESSION_GEN.load(Ordering::Acquire) != gen {
        edit(&writes, true);
        apply_cleanup(table);
        apply_leaves(table);
    }
    leave();
}

fn plan_outgoing_table(
    table: &mut Table,
    outgoing_byte: u8,
    area: &[u8],
    announce: Option<Announce>,
    interface_byte: u8,
    saved_stealth: bool,
) -> [Option<BodyWrite>; 8] {
    let mut writes = [None; 8];
    let mut count = 0usize;
    let walked = walk_messages(area, 16, |message| {
        if count >= writes.len() {
            return;
        }
        let body_at = message_body_at(area, message.body);
        if message.kind == Some(KIND_STATION) {
            if let Some(write) =
                beacon_write(table, message.body, body_at, interface_byte, saved_stealth)
            {
                writes[count] = Some(write);
                count += 1;
            }
            if message.body.first() == Some(&1) {
                note_request(table, Direction::Outgoing, message.body, saved_stealth);
            }
        } else if message.kind == Some(KIND_RTT) {
            if let Some(write) = payload_write(
                table,
                outgoing_byte,
                message.body,
                body_at,
                announce,
                saved_stealth,
            ) {
                writes[count] = Some(write);
                count += 1;
            }
        }
    });
    if walked.is_err() {
        [None; 8]
    } else {
        writes
    }
}

fn message_body_at(area: &[u8], body: &[u8]) -> Option<usize> {
    let area_start = area.as_ptr() as usize;
    let body_start = body.as_ptr() as usize;
    body_start.checked_sub(area_start)
}

fn note_request(table: &mut Table, direction: Direction, body: &[u8], saved_stealth: bool) {
    let RequestParse::Ready(request) = parse_request(body) else {
        return;
    };
    let Some(pending) = pending_from(direction, &request) else {
        return;
    };
    if !table.session {
        table.session = true;
        table.latched_stealth = saved_stealth;
    }
    match pending_slot(table, pending) {
        PendingStore::Stored => {
            LEARNED.fetch_add(1, Ordering::Relaxed);
            LAST_MIX.store(mix_id(pending.constant_id), Ordering::Relaxed);
        }
        PendingStore::Duplicate => {}
        PendingStore::Poisoned | PendingStore::Full => {
            AMBIGUOUS.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn note_response(table: &mut Table, source_id: u64, body: &[u8]) {
    if source_id == 0 {
        AMBIGUOUS.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let Some(index) = pending_index(table, source_id) else {
        return;
    };
    let Some(pending) = table.pending[index] else {
        return;
    };
    if pending.poisoned {
        AMBIGUOUS.fetch_add(1, Ordering::Relaxed);
        return;
    }
    table.pending[index] = None;
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
        payload_known: false,
        latency_bits: 0,
        profile_bits: 0,
    };
    if !install_peer(table, peer) {
        AMBIGUOUS.fetch_add(1, Ordering::Relaxed);
        return;
    }
    CONFIRMED.fetch_add(1, Ordering::Relaxed);
    LAST_MIX.store(mix_id(source_id), Ordering::Relaxed);
}

fn note_rtt(table: &mut Table, incoming_byte: u8, body: &[u8]) {
    let Some(payload) = crate::peer_passive::decode_rtt_reply_payload(body) else {
        return;
    };
    let ForkPayload::Accepted {
        latency_bits,
        profile_bits,
        ..
    } = payload
    else {
        return;
    };
    let mut matches = 0usize;
    let mut slot = None;
    for (index, peer) in table.peers.iter().enumerate() {
        if let Some(peer) = peer {
            if peer.modded && peer.incoming == incoming_byte {
                matches += 1;
                slot = Some(index);
            }
        }
    }
    if matches != 1 {
        return;
    }
    let Some(index) = slot else {
        return;
    };
    if let Some(peer) = table.peers[index].as_mut() {
        peer.payload_known = true;
        peer.latency_bits = latency_bits;
        peer.profile_bits = profile_bits;
    }
}

fn retire_constant(table: &mut Table, id: u64) {
    if id == 0 {
        return;
    }
    for slot in &mut table.index_id {
        if *slot == id {
            *slot = 0;
        }
    }
    for slot in &mut table.pending {
        if slot.is_some_and(|pending| pending.constant_id == id) {
            *slot = None;
        }
    }
    for slot in &mut table.peers {
        if slot.is_some_and(|peer| peer.constant_id == id) {
            *slot = None;
        }
    }
}

/// Kind `0x14` subtype 3 or 4. The manager compares those bytes as a disconnect.
/// The source id is the same constant id a subtype-2 response confirmed.
/// A zero id matches nothing. Vanilla settings are not a leave.
fn forget_id(table: &mut Table, source_id: u64) {
    retire_constant(table, source_id);
    resolve_selection(table);
}

fn custom_suppressed(table: &Table, saved_stealth: bool) -> bool {
    if table.session {
        table.latched_stealth
    } else {
        saved_stealth
    }
}

fn beacon_write(
    table: &Table,
    body: &[u8],
    body_at: Option<usize>,
    interface_byte: u8,
    saved_stealth: bool,
) -> Option<BodyWrite> {
    let Some(body_at) = body_at else {
        BEACON_SKIPS.fetch_add(1, Ordering::Relaxed);
        return None;
    };
    let tag_at = spare_tag_offset(body)?;
    if custom_suppressed(table, saved_stealth) {
        STEALTH_SUPPRESS.fetch_add(1, Ordering::Relaxed);
        BEACON_SKIPS.fetch_add(1, Ordering::Relaxed);
        return None;
    }
    let mut bytes = [0u8; 4];
    bytes[0] = 0x45;
    bytes[1] = interface_byte;
    BEACON_WRITES.fetch_add(1, Ordering::Relaxed);
    Some(BodyWrite {
        at: body_at + tag_at,
        bytes,
        len: 2,
    })
}

fn payload_write(
    table: &Table,
    outgoing_byte: u8,
    body: &[u8],
    body_at: Option<usize>,
    announce: Option<Announce>,
    saved_stealth: bool,
) -> Option<BodyWrite> {
    if body.len() < RTT_BODY_MIN || !crate::peer_passive::rtt_type_editable(body) {
        return None;
    }
    let Some(body_at) = body_at else {
        PAYLOAD_SKIPS.fetch_add(1, Ordering::Relaxed);
        return None;
    };
    let matches = confirmed_outgoing(table, outgoing_byte);
    if matches != 1 {
        PAYLOAD_SKIPS.fetch_add(1, Ordering::Relaxed);
        return None;
    }
    if custom_suppressed(table, saved_stealth) {
        STEALTH_SUPPRESS.fetch_add(1, Ordering::Relaxed);
        PAYLOAD_SKIPS.fetch_add(1, Ordering::Relaxed);
        return None;
    }
    let Some(announce) = announce else {
        PAYLOAD_SKIPS.fetch_add(1, Ordering::Relaxed);
        return None;
    };
    if announce.profile.is_none() {
        PAYLOAD_SKIPS.fetch_add(1, Ordering::Relaxed);
        return None;
    };
    PAYLOAD_WRITES.fetch_add(1, Ordering::Relaxed);
    Some(BodyWrite {
        at: body_at + 4,
        bytes: encode_v3(announce),
        len: 4,
    })
}

fn confirmed_outgoing(table: &Table, outgoing_byte: u8) -> usize {
    table
        .peers
        .iter()
        .flatten()
        .filter(|peer| peer.modded && peer.outgoing == outgoing_byte)
        .count()
}

fn spare_tag_offset(body: &[u8]) -> Option<usize> {
    if body.len() < 2 || body[0] != 2 || body[1] != 0 {
        return None;
    }
    if body.len() < BEACON_HEADER + LOCATION_STRIDE {
        return None;
    }
    let nslots = (body.len() - BEACON_HEADER) / LOCATION_STRIDE;
    let count = usize::from(body[0x38]);
    if nslots <= count {
        return None;
    }
    let end = BEACON_HEADER + nslots * LOCATION_STRIDE;
    let tag_at = end.checked_sub(0x54)?;
    let iface_at = end.checked_sub(0x53)?;
    if iface_at >= body.len() {
        return None;
    }
    Some(tag_at)
}

enum PendingStore {
    Stored,
    Duplicate,
    Poisoned,
    Full,
}

fn pending_from(direction: Direction, request: &ConnectionRequest) -> Option<Pending> {
    if request.is_inverse != 1
        || request.target_constant_id == 0
        || request.location_constant_id == 0
    {
        return None;
    }
    let (constant_id, incoming, outgoing) = match direction {
        Direction::Outgoing => (
            request.target_constant_id,
            request.inverse_connection_id,
            request.connection_id,
        ),
        Direction::Incoming => (
            request.location_constant_id,
            request.connection_id,
            request.inverse_connection_id,
        ),
    };
    if constant_id == 0 {
        return None;
    }
    Some(Pending {
        constant_id,
        incoming,
        outgoing,
        poisoned: false,
    })
}

fn pending_slot(table: &mut Table, pending: Pending) -> PendingStore {
    for existing in table.pending.iter_mut().flatten() {
        if existing.constant_id == pending.constant_id {
            if existing.incoming == pending.incoming && existing.outgoing == pending.outgoing {
                return PendingStore::Duplicate;
            }
            existing.poisoned = true;
            return PendingStore::Poisoned;
        }
    }
    for slot in &mut table.pending {
        if slot.is_none() {
            *slot = Some(pending);
            return PendingStore::Stored;
        }
    }
    PendingStore::Full
}

fn pending_index(table: &Table, constant_id: u64) -> Option<usize> {
    table
        .pending
        .iter()
        .position(|slot| slot.is_some_and(|pending| pending.constant_id == constant_id))
}

fn install_peer(table: &mut Table, peer: Peer) -> bool {
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
    for slot in &mut table.peers {
        if slot.is_none() {
            *slot = Some(peer);
            resolve_selection(table);
            return true;
        }
    }
    resolve_selection(table);
    false
}

pub fn parse_request(body: &[u8]) -> RequestParse {
    if body.first() != Some(&1) {
        return RequestParse::NotRequest;
    }
    let Some(&length_byte) = body.get(1) else {
        return RequestParse::Truncated;
    };
    let prefix = (usize::from(length_byte) & 0x7f) << 1;
    let mut cursor = 2usize.saturating_add(prefix);
    if cursor > body.len() {
        return RequestParse::Truncated;
    }
    let Some(connection_id) = body.get(cursor).copied() else {
        return RequestParse::Truncated;
    };
    cursor += 1;
    let Some(is_inverse) = body.get(cursor).copied() else {
        return RequestParse::Truncated;
    };
    cursor += 1;
    let Some(target_constant_id) = read_be_u64(body, cursor) else {
        return RequestParse::Truncated;
    };
    cursor += 8;
    if body.len() < cursor + 4 {
        return RequestParse::Truncated;
    }
    cursor += 4;
    let Some(inverse_connection_id) = body.get(cursor).copied() else {
        return RequestParse::Truncated;
    };
    cursor += 1;
    let Some(location_constant_id) = location_constant_id(&body[cursor..]) else {
        if body.len() < cursor + 2 {
            return RequestParse::Truncated;
        }
        return RequestParse::LocationUnsupported;
    };
    RequestParse::Ready(ConnectionRequest {
        connection_id,
        is_inverse,
        target_constant_id,
        inverse_connection_id,
        location_constant_id,
    })
}

fn location_constant_id(bytes: &[u8]) -> Option<u64> {
    if bytes.len() < 2 {
        return None;
    }
    let mut cursor = 2usize;
    cursor = consume_typed(bytes, cursor, bytes[0])?;
    cursor = consume_typed(bytes, cursor, bytes[1])?;
    cursor = consume_typed(bytes, cursor, 6)?;
    read_be_u64(bytes, cursor)
}

fn consume_typed(bytes: &[u8], cursor: usize, tag: u8) -> Option<usize> {
    let size = match tag {
        2..=5 => 2,
        6 => 6,
        _ => return None,
    };
    let next = cursor.checked_add(size)?;
    if next > bytes.len() {
        None
    } else {
        Some(next)
    }
}

fn read_be_u64(bytes: &[u8], at: usize) -> Option<u64> {
    let end = at.checked_add(8)?;
    let raw = bytes.get(at..end)?;
    let mut wide = [0u8; 8];
    wide.copy_from_slice(raw);
    Some(u64::from_be_bytes(wide))
}

fn first_player_name(body: &[u8]) -> ([u8; NAME_CAP], u8) {
    if body.len() < BEACON_HEADER + LOCATION_STRIDE || body.first() != Some(&2) {
        return ([0; NAME_CAP], 0);
    }
    let count = usize::from(body[0x38]);
    if count == 0 {
        return ([0; NAME_CAP], 0);
    }
    let entry = &body[BEACON_HEADER..];
    if entry.len() < LOCATION_STRIDE {
        return ([0; NAME_CAP], 0);
    }
    let tag = entry[ENTRY_TAG];
    let window = &entry[ENTRY_NAME..ENTRY_NAME + ENTRY_NAME_LEN];
    decode_name(tag, window).unwrap_or(([0; NAME_CAP], 0))
}

fn decode_name(tag: u8, window: &[u8]) -> Option<([u8; NAME_CAP], u8)> {
    let mut out = [0u8; NAME_CAP];
    if tag == 1 {
        let end = window
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(window.len());
        let raw = std::str::from_utf8(&window[..end]).ok()?.as_bytes();
        if raw.is_empty() || raw.len() > NAME_CAP {
            return None;
        }
        out[..raw.len()].copy_from_slice(raw);
        return Some((out, raw.len() as u8));
    }
    if tag == 2 {
        return decode_utf16(window);
    }
    None
}

fn decode_utf16(window: &[u8]) -> Option<([u8; NAME_CAP], u8)> {
    let mut out = [0u8; NAME_CAP];
    let mut len = 0usize;
    let mut index = 0usize;
    while index + 1 < window.len() && index / 2 < 20 {
        let unit = u16::from_le_bytes([window[index], window[index + 1]]);
        index += 2;
        if unit == 0 {
            break;
        }
        let ch = char::from_u32(u32::from(unit))?;
        let mut encoded = [0u8; 4];
        let raw = ch.encode_utf8(&mut encoded).as_bytes();
        if len + raw.len() > NAME_CAP {
            return None;
        }
        out[len..len + raw.len()].copy_from_slice(raw);
        len += raw.len();
    }
    if len == 0 {
        return None;
    }
    Some((out, len as u8))
}

pub struct TransportCounters {
    pub learned: u64,
    pub ambiguous: u64,
    pub confirmed: u64,
    pub cleanups: u64,
    pub beacon_writes: u64,
    pub beacon_skips: u64,
    pub payload_writes: u64,
    pub payload_skips: u64,
    pub stealth_suppress: u64,
    pub contentions: u64,
    pub last_mix: u64,
}

pub fn counters() -> TransportCounters {
    TransportCounters {
        learned: LEARNED.load(Ordering::Relaxed),
        ambiguous: AMBIGUOUS.load(Ordering::Relaxed),
        confirmed: CONFIRMED.load(Ordering::Relaxed),
        cleanups: CLEANUPS.load(Ordering::Relaxed),
        beacon_writes: BEACON_WRITES.load(Ordering::Relaxed),
        beacon_skips: BEACON_SKIPS.load(Ordering::Relaxed),
        payload_writes: PAYLOAD_WRITES.load(Ordering::Relaxed),
        payload_skips: PAYLOAD_SKIPS.load(Ordering::Relaxed),
        stealth_suppress: STEALTH_SUPPRESS.load(Ordering::Relaxed),
        contentions: CONTENTIONS.load(Ordering::Relaxed),
        last_mix: LAST_MIX.load(Ordering::Relaxed),
    }
}

#[cfg(test)]
fn shown_from(table: &mut Table) -> ShownPeer {
    frame_from(view_of(table)).peer
}

/// Serializes tests that use the process-wide station table.
/// Runtime `enter` stays non-blocking. Do not take this lock again while held.
#[cfg(test)]
pub(crate) fn global_gate() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preset_bits(mode: AnnounceMode) -> u16 {
        profile_bits(mode, 0, matches!(mode, AnnounceMode::LessLagDoubles))
    }

    fn announce(latency: i8, mode: AnnounceMode) -> Announce {
        Announce {
            latency,
            profile: Some(mode),
            resolution: 0,
            dynamic: matches!(mode, AnnounceMode::LessLagDoubles),
            boost_applied: false,
        }
    }

    fn location_prefix() -> Vec<u8> {
        let mut bytes = vec![2, 2];
        bytes.extend_from_slice(&[0x11, 0x22]);
        bytes.extend_from_slice(&[0x33, 0x44]);
        bytes.extend_from_slice(&[0u8; 6]);
        bytes.extend_from_slice(&0x1122_3344_5566_7788u64.to_be_bytes());
        bytes
    }

    fn request(inverse: u8, connection: u8, inverse_connection: u8) -> Vec<u8> {
        let mut body = vec![1, 0x02, 0xaa, 0xbb, 0xcc, 0xdd];
        body.push(connection);
        body.push(inverse);
        body.extend_from_slice(&0xaabb_ccdd_eeff_0011u64.to_be_bytes());
        body.extend_from_slice(&0x0102_0304u32.to_be_bytes());
        body.push(inverse_connection);
        body.extend(location_prefix());
        body
    }

    fn response(source_name: &[u8], beacon: bool, count: u8, slots: u8) -> Vec<u8> {
        let mut body = vec![0u8; BEACON_HEADER + usize::from(slots) * LOCATION_STRIDE];
        body[0] = 2;
        body[1] = 0;
        body[0x38] = count;
        if count > 0 {
            body[BEACON_HEADER + ENTRY_TAG] = 1;
            let name_at = BEACON_HEADER + ENTRY_NAME;
            body[name_at..name_at + source_name.len()].copy_from_slice(source_name);
        }
        if beacon {
            let end = BEACON_HEADER + usize::from(slots) * LOCATION_STRIDE;
            body[end - 0x54] = 0x45;
            body[end - 0x53] = 1;
        }
        body
    }

    #[test]
    fn direction_swap_and_inverse_gate() {
        let parsed = parse_request(&request(1, 0x10, 0x20));
        let RequestParse::Ready(ready) = parsed else {
            panic!("request");
        };
        let outgoing = pending_from(Direction::Outgoing, &ready).unwrap();
        assert_eq!(outgoing.constant_id, 0xaabb_ccdd_eeff_0011);
        assert_eq!(outgoing.incoming, 0x20);
        assert_eq!(outgoing.outgoing, 0x10);
        let incoming = pending_from(Direction::Incoming, &ready).unwrap();
        assert_eq!(incoming.constant_id, 0x1122_3344_5566_7788);
        assert_eq!(incoming.incoming, 0x10);
        assert_eq!(incoming.outgoing, 0x20);
        assert!(pending_from(Direction::Outgoing, &{
            let RequestParse::Ready(mut raw) = parse_request(&request(0, 0x10, 0x20)) else {
                panic!("inverse");
            };
            raw.is_inverse = 0;
            raw
        })
        .is_none());
    }

    #[test]
    fn truncated_prefix_and_unsupported_location_do_not_register() {
        assert_eq!(parse_request(&[1, 0x7f]), RequestParse::Truncated);
        let mut short = request(1, 1, 2);
        short.truncate(8);
        assert_eq!(parse_request(&short), RequestParse::Truncated);
        let mut bad = request(1, 1, 2);
        let tag_at = bad.len() - location_prefix().len();
        bad[tag_at] = 9;
        assert_eq!(parse_request(&bad), RequestParse::LocationUnsupported);
        assert!(matches!(parse_request(&[0x58]), RequestParse::NotRequest));
    }

    fn feed(table: &mut Table, source: u64, incoming: u8, kind: u8, body: &[u8], stealth: bool) {
        observe_incoming_table(table, source, incoming, kind, body, stealth);
    }

    #[test]
    fn response_before_request_does_not_confirm_and_order_then_does() {
        let mut table = Table::empty();
        let early = response(b"Samus", true, 1, 2);
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x10,
            KIND_STATION,
            &early,
            false,
        );
        assert!(!shown_from(&mut table).known);
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            false,
        );
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x10,
            KIND_STATION,
            &early,
            false,
        );
        let shown = shown_from(&mut table);
        assert!(shown.known);
        assert_eq!(&shown.name[..5], b"Samus");
        assert_eq!(shown.interface, "Wi-Fi");
    }

    #[test]
    fn reused_connection_byte_does_not_keep_the_old_payload() {
        let mut table = Table::empty();
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            false,
        );
        let body = response(b"Samus", true, 1, 2);
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x10,
            KIND_STATION,
            &body,
            false,
        );
        let mut reply = vec![0, 0, 0, 1, 3, 7, 0x0b, 0x10];
        reply.extend_from_slice(&[0u8; 8]);
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x10,
            KIND_RTT,
            &reply,
            false,
        );
        let shown = shown_from(&mut table);
        assert_eq!(shown.settings, "Vanilla");
        assert_eq!(shown.latency, RemoteLatency::Buffer(7));
        let mut again = request(1, 0x10, 0x21);
        again[8..16].copy_from_slice(&0x99u64.to_be_bytes());
        let loc = again.len() - 8;
        again[loc..].copy_from_slice(&0x77u64.to_be_bytes());
        feed(&mut table, 0, 0, KIND_STATION, &again, false);
        let renamed = response(b"Ridley", true, 1, 2);
        feed(&mut table, 0x77, 0x10, KIND_STATION, &renamed, false);
        let shown = shown_from(&mut table);
        assert_eq!(&shown.name[..6], b"Ridley");
        assert_eq!(shown.settings, "");
        assert_eq!(shown.latency, RemoteLatency::Unknown);
        clear_table(&mut table);
        assert!(table.peers.iter().all(Option::is_none));
        assert!(!table.session);
    }

    fn global_gate() -> std::sync::MutexGuard<'static, ()> {
        super::global_gate()
    }

    #[test]
    fn a_disputed_pending_id_and_a_busy_gate_do_not_write() {
        let _gate = global_gate();
        let mut table = Table::empty();
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            false,
        );
        let other = request(1, 0x12, 0x23);
        feed(&mut table, 0, 0, KIND_STATION, &other, false);
        let body = response(b"Samus", true, 1, 2);
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x10,
            KIND_STATION,
            &body,
            false,
        );
        assert!(!shown_from(&mut table).known);
        assert!(BUSY
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok());
        assert!(with_table(|live| {
            observe_incoming_table(live, 0, 0, KIND_STATION, &request(1, 0x10, 0x20), false);
        })
        .is_none());
        BUSY.store(false, Ordering::Release);
        assert!(counters().contentions >= 1);
    }

    #[test]
    fn cleanup_while_the_gate_is_busy_drops_the_old_authorization() {
        let _gate = global_gate();
        BUSY.store(false, Ordering::Release);
        clear_session();
        observe_incoming(0, 0, KIND_STATION, &request(1, 0x44, 0x55), false);
        observe_incoming(
            0x1122_3344_5566_7788,
            0x44,
            KIND_STATION,
            &response(b"Samus", true, 1, 2),
            false,
        );
        assert!(peer_frame().peer.known);
        assert!(BUSY
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok());
        let before = SESSION_GEN.load(Ordering::Acquire);
        clear_session();
        assert!(SESSION_GEN.load(Ordering::Acquire) > before);
        BUSY.store(false, Ordering::Release);
        let mut rtt = vec![0x06];
        rtt.extend_from_slice(&16u16.to_be_bytes());
        rtt.extend_from_slice(&[0x58, 0, 0, 0]);
        rtt.extend_from_slice(&[0u8; 16]);
        let aligned = (rtt.len() + 3) & !3;
        rtt.resize(aligned, 0);
        let writes = plan_outgoing(0x55, &rtt, Some(announce(4, AnnounceMode::Ultra)), 0, false);
        assert!(writes.iter().flatten().next().is_none());
        assert!(!peer_frame().peer.known);
        assert_eq!(peer_frame().peer.interface, "Unknown");
        assert_eq!(latched_stealth(), None);
    }

    #[test]
    fn a_beacon_before_the_latch_follows_the_saved_stealth_bit() {
        let mut table = Table::empty();
        let body = response(b"A", false, 1, 2);
        let mut area = vec![0x06];
        area.extend_from_slice(&(body.len() as u16).to_be_bytes());
        area.extend_from_slice(&[0x14, 0, 0, 0]);
        area.extend_from_slice(&body);
        let aligned = (area.len() + 3) & !3;
        area.resize(aligned, 0);
        let hidden = plan_outgoing_table(&mut table, 0x55, &area, None, 1, true);
        assert!(hidden.iter().all(Option::is_none));
        assert!(!table.session);
        let shown = plan_outgoing_table(&mut table, 0x55, &area, None, 1, false);
        let write = shown.iter().flatten().next().expect("beacon");
        assert_eq!(&write.bytes[..2], &[0x45, 1]);
        let mut rtt = vec![0x06];
        rtt.extend_from_slice(&16u16.to_be_bytes());
        rtt.extend_from_slice(&[0x58, 0, 0, 0]);
        rtt.extend_from_slice(&[0u8; 16]);
        let aligned = (rtt.len() + 3) & !3;
        rtt.resize(aligned, 0);
        let hidden_payload = plan_outgoing_table(
            &mut table,
            0x55,
            &rtt,
            Some(announce(4, AnnounceMode::Ultra)),
            1,
            true,
        );
        assert!(hidden_payload.iter().all(Option::is_none));
        assert!(!table.session);
    }

    #[test]
    fn interface_labels_do_not_invent_a_type() {
        assert_eq!(local_link_label(1), "Wi-Fi");
        assert_eq!(local_link_label(2), "Ethernet");
        assert_eq!(local_link_label(0), "Unknown");
        assert_eq!(local_link_label(3), "Unknown");
        assert_eq!(interface_label(1, true), "Wi-Fi");
        assert_eq!(interface_label(2, true), "Ethernet");
        assert_eq!(interface_label(3, true), "Unknown");
        assert_eq!(interface_label(1, false), "Unknown");
    }

    #[test]
    fn rtt_payload_is_written_only_for_types_0_and_1() {
        let mut table = Table::empty();
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x44, 0x55),
            false,
        );
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x44,
            KIND_STATION,
            &response(b"A", true, 1, 2),
            false,
        );
        let announce = Some(announce(4, AnnounceMode::Ultra));
        let area_for = |kind: u32| {
            let mut rtt = vec![0x06];
            rtt.extend_from_slice(&16u16.to_be_bytes());
            rtt.extend_from_slice(&[0x58, 0, 0, 0]);
            rtt.extend_from_slice(&kind.to_be_bytes());
            rtt.extend_from_slice(&[0u8; 12]);
            let aligned = (rtt.len() + 3) & !3;
            rtt.resize(aligned, 0);
            rtt
        };
        for kind in [0u32, 1] {
            let writes = plan_outgoing_table(&mut table, 0x55, &area_for(kind), announce, 0, false);
            let payload = writes.iter().flatten().next().expect("payload");
            assert_eq!(payload.len, 4);
            assert_eq!(payload.bytes[0], 3);
        }
        for kind in [2u32, 0x100, 0xffff_ffff] {
            let area = area_for(kind);
            let before = area.clone();
            let writes = plan_outgoing_table(&mut table, 0x55, &area, announce, 0, false);
            assert!(writes.iter().all(Option::is_none));
            assert_eq!(area, before);
        }
    }

    #[test]
    fn beacon_and_payload_bytes_change_only_the_spare_and_modded_slots() {
        let mut table = Table::empty();
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x44, 0x55),
            true,
        );
        let body = response(b"A", false, 1, 2);
        let mut area = vec![0x06];
        area.extend_from_slice(&(body.len() as u16).to_be_bytes());
        area.extend_from_slice(&[0x14, 0, 0, 0]);
        area.extend_from_slice(&body);
        let aligned = (area.len() + 3) & !3;
        area.resize(aligned, 0);
        let original = area.clone();
        let announce = announce(4, AnnounceMode::Ultra);
        let writes = plan_outgoing_table(&mut table, 0x55, &area, Some(announce), 2, true);
        assert!(writes.iter().all(Option::is_none));
        clear_table(&mut table);
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x44, 0x55),
            false,
        );
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x44,
            KIND_STATION,
            &response(b"A", true, 1, 2),
            false,
        );
        let writes = plan_outgoing_table(&mut table, 0x55, &area, Some(announce), 2, false);
        let write = writes.iter().flatten().next().expect("beacon");
        assert_eq!(write.len, 2);
        assert_eq!(&write.bytes[..2], &[0x45, 2]);
        area[write.at] = write.bytes[0];
        area[write.at + 1] = write.bytes[1];
        let changed = original
            .iter()
            .zip(area.iter())
            .filter(|(before, after)| before != after)
            .count();
        assert_eq!(changed, 2);
        let mut rtt = vec![0x06];
        rtt.extend_from_slice(&16u16.to_be_bytes());
        rtt.extend_from_slice(&[0x58, 0, 0, 0]);
        rtt.extend_from_slice(&[0u8; 16]);
        let rtt_aligned = (rtt.len() + 3) & !3;
        rtt.resize(rtt_aligned, 0);
        let before = rtt.clone();
        let skipped = plan_outgoing_table(&mut table, 0x01, &rtt, Some(announce), 0, false);
        assert!(skipped.iter().flatten().next().is_none());
        assert_eq!(rtt, before);
        let writes = plan_outgoing_table(&mut table, 0x55, &rtt, Some(announce), 0, false);
        let payload = writes.iter().flatten().next().expect("payload");
        assert_eq!(payload.len, 4);
        assert_eq!(payload.bytes[0], 3);
        assert_eq!(payload.bytes[1], 4);
        assert_eq!(
            u16::from_le_bytes([payload.bytes[2], payload.bytes[3]]),
            preset_bits(AnnounceMode::Ultra)
        );
        let quiet = plan_outgoing_table(&mut table, 0x55, &rtt, None, 0, false);
        assert!(quiet.iter().flatten().next().is_none());
    }

    #[test]
    fn v2_and_v3_payloads_decode_and_a_cleanup_drops_the_latch() {
        let mut table = Table::empty();
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            false,
        );
        assert!(!table.latched_stealth);
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            true,
        );
        assert!(!table.latched_stealth);
        let body = response(b"Samus", true, 1, 2);
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x10,
            KIND_STATION,
            &body,
            false,
        );
        let mut v2 = vec![0, 0, 0, 1, 2, 5, 0xff, 0xff];
        v2.extend_from_slice(&[0u8; 8]);
        feed(&mut table, 1, 0x10, KIND_RTT, &v2, false);
        assert_eq!(shown_from(&mut table).settings, "Other");
        let less = preset_bits(AnnounceMode::LessLag);
        let mut v3 = vec![0, 0, 0, 1, 3, 1, less as u8, (less >> 8) as u8];
        v3.extend_from_slice(&[0u8; 8]);
        feed(&mut table, 1, 0x10, KIND_RTT, &v3, false);
        assert_eq!(shown_from(&mut table).settings, "LessLag");
        clear_table(&mut table);
        assert!(!table.session);
        assert!(!shown_from(&mut table).known);
    }

    #[test]
    fn lldoubles_bits_match_the_fork_and_console_singles_still_decode() {
        let doubles = preset_bits(AnnounceMode::LessLagDoubles);
        assert_eq!(doubles, 3 | (1 << 2) | (1 << 11) | (1 << 13));
        assert_eq!(doubles & (1 << 14), 0);
        assert_eq!(settings_label(doubles), "LLDoubles");
        let rejected = profile_bits(AnnounceMode::LessLagDoubles, 0, false);
        assert_eq!(rejected & (1 << 11), 0);
        assert_eq!(rejected & (1 << 14), 0);
        assert_eq!(settings_label(rejected), "LLDoubles");
        let console_ultra = profile_bits(AnnounceMode::Ultra, 3, true);
        assert_eq!((console_ultra >> 4) & 0x7f, 3);
        assert_ne!(console_ultra & (1 << 11), 0);
        assert_eq!(console_ultra & (1 << 14), 0);
        assert_eq!(settings_label(console_ultra), "LLUltra");
        let console_less = profile_bits(AnnounceMode::LessLag, 0, true);
        assert_eq!(settings_label(console_less), "LessLag");
        assert_ne!(console_less & (1 << 11), 0);
        let quiet_doubles = announce_from(
            4,
            true,
            crate::render_policy::Profile::Doubles,
            crate::resolution::Setting {
                level: crate::resolution::Level::P1080,
                dynamic: false,
            },
            false,
        );
        assert_eq!(quiet_doubles.profile, Some(AnnounceMode::LessLagDoubles));
        assert!(!quiet_doubles.dynamic);
        assert_eq!(
            profile_bits(
                quiet_doubles.profile.unwrap(),
                quiet_doubles.resolution,
                quiet_doubles.dynamic
            ) & (1 << 11),
            0
        );
        assert_eq!(
            announced_profile(false, crate::render_policy::Profile::LessLag),
            None
        );
        assert_eq!(
            announced_profile(true, crate::render_policy::Profile::Vanilla),
            Some(AnnounceMode::Vanilla)
        );
        assert_eq!(
            announced_profile(true, crate::render_policy::Profile::Doubles),
            Some(AnnounceMode::LessLagDoubles)
        );
        assert_eq!(
            settings_label(preset_bits(AnnounceMode::LessLag) | (1 << 11)),
            "LessLag"
        );
        assert_eq!(
            settings_label(preset_bits(AnnounceMode::Ultra) | (3 << 4) | (1 << 11)),
            "LLUltra"
        );
        assert_eq!(settings_label(doubles | (5 << 4)), "Other");
        assert_eq!(settings_label(doubles | (1 << 14)), "LLDoubles");
        assert_eq!(
            label_from_code(settings_code(doubles | (1 << 14))),
            "LLDoubles++"
        );
        assert_eq!(settings_label(doubles | (1 << 15)), "Other");
        let encoded = encode_v3(announce(4, AnnounceMode::LessLagDoubles));
        assert_eq!(encoded[0], 3);
        assert_eq!(encoded[1], 4);
        assert_eq!(u16::from_le_bytes([encoded[2], encoded[3]]), doubles);
        let mut table = Table::empty();
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            false,
        );
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x10,
            KIND_STATION,
            &response(b"Samus", true, 1, 2),
            false,
        );
        let mut reply = vec![0, 0, 0, 1, 3, 4, encoded[2], encoded[3]];
        reply.extend_from_slice(&[0u8; 8]);
        feed(&mut table, 1, 0x10, KIND_RTT, &reply, false);
        assert_eq!(shown_from(&mut table).settings, "LLDoubles");
        assert_eq!(shown_from(&mut table).latency, RemoteLatency::Buffer(4));
    }

    #[test]
    fn applied_boost_uses_fork_bit_14_and_v2_still_clears_it() {
        let less = preset_bits(AnnounceMode::LessLag);
        let boosted = less | (1 << 14);
        assert_eq!(settings_label(boosted), "LessLag");
        assert_eq!(label_from_code(settings_code(boosted)), "LessLag++");
        let mut announced = announce(4, AnnounceMode::LessLag);
        announced.boost_applied = true;
        let encoded = encode_v3(announced);
        assert_eq!(u16::from_le_bytes([encoded[2], encoded[3]]), boosted);
        let quiet = announce_from(
            4,
            false,
            crate::render_policy::Profile::LessLag,
            crate::resolution::Setting {
                level: crate::resolution::Level::P1080,
                dynamic: false,
            },
            true,
        );
        assert!(!quiet.boost_applied);
        assert_eq!(encode_v3(quiet), [3, 4, 0, 0]);
        let mut table = Table::empty();
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            false,
        );
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x10,
            KIND_STATION,
            &response(b"Samus", true, 1, 2),
            false,
        );
        let mut v2 = vec![0, 0, 0, 1, 2, 1, boosted as u8, (boosted >> 8) as u8];
        v2.extend_from_slice(&[0u8; 8]);
        feed(&mut table, 1, 0x10, KIND_RTT, &v2, false);
        assert_eq!(shown_from(&mut table).settings, "LessLag");
        let mut v3 = vec![0, 0, 0, 1, 3, 1, boosted as u8, (boosted >> 8) as u8];
        v3.extend_from_slice(&[0u8; 8]);
        feed(&mut table, 1, 0x10, KIND_RTT, &v3, false);
        assert_eq!(shown_from(&mut table).settings, "LessLag++");
    }

    fn rtt_reply(latency: u8, bits: u16) -> Vec<u8> {
        let mut body = vec![0, 0, 0, 1, 3, latency, bits as u8, (bits >> 8) as u8];
        body.extend_from_slice(&[0u8; 8]);
        body
    }

    #[test]
    fn chosen_identity_latency_follows_one_payload_and_clears() {
        assert_eq!(remote_latency_label(RemoteLatency::Unknown), "---");
        assert_eq!(remote_latency_label(RemoteLatency::Auto), "Auto");
        assert_eq!(remote_latency_label(RemoteLatency::Buffer(0)), "0");
        assert_eq!(remote_latency_label(RemoteLatency::Buffer(25)), "25");

        let mut table = Table::empty();
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            false,
        );
        feed(
            &mut table,
            0x1122_3344_5566_7788,
            0x10,
            KIND_STATION,
            &response(b"Samus", true, 1, 2),
            false,
        );
        let waiting = shown_from(&mut table);
        assert!(waiting.known);
        assert_eq!(waiting.settings, "");
        assert_eq!(waiting.latency, RemoteLatency::Unknown);
        let bits = preset_bits(AnnounceMode::Vanilla);
        for (byte, latency) in [
            (255, RemoteLatency::Auto),
            (0, RemoteLatency::Buffer(0)),
            (25, RemoteLatency::Buffer(25)),
            (26, RemoteLatency::Unknown),
            (254, RemoteLatency::Unknown),
        ] {
            feed(&mut table, 1, 0x10, KIND_RTT, &rtt_reply(byte, bits), false);
            let shown = shown_from(&mut table);
            assert_eq!(shown.settings, "Vanilla");
            assert_eq!(name_label(&shown.name, shown.name_len), Some("Samus"));
            assert_eq!(shown.latency, latency);
        }

        clear_table(&mut table);
        let mut plain = named(11, b"Ann", 2, AnnounceMode::Vanilla);
        plain.modded = false;
        plain.latency_bits = 4;
        table.peers[0] = Some(plain);
        let unmodded = shown_from(&mut table);
        assert_eq!(unmodded.interface, "Unknown");
        assert_eq!(unmodded.latency, RemoteLatency::Unknown);
        let mut quiet = named(11, b"Ann", 2, AnnounceMode::Vanilla);
        quiet.payload_known = false;
        quiet.latency_bits = 8;
        table.peers[0] = Some(quiet);
        let unpaid = shown_from(&mut table);
        assert_eq!(unpaid.settings, "");
        assert_eq!(unpaid.latency, RemoteLatency::Unknown);

        let mut ann = named(11, b"Ann", 2, AnnounceMode::Vanilla);
        ann.latency_bits = 255;
        let mut bob = named(22, b"Bob", 1, AnnounceMode::LessLag);
        bob.latency_bits = 12;
        table.peers[0] = Some(ann);
        table.peers[1] = Some(bob);
        assert_eq!(shown_from(&mut table).latency, RemoteLatency::Unknown);
        table.local_index = 0;
        bind_index(&mut table, 4, 11);
        bind_index(&mut table, 9, 22);
        let shown = shown_from(&mut table);
        assert_eq!(shown.station, Some(4));
        assert_eq!(name_label(&shown.name, shown.name_len), Some("Ann"));
        assert_eq!(shown.settings, "Vanilla");
        assert_eq!(shown.interface, "Ethernet");
        assert_eq!(shown.latency, RemoteLatency::Auto);
        cycle_table(&mut table, 1);
        let shown = shown_from(&mut table);
        assert_eq!(shown.station, Some(9));
        assert_eq!(name_label(&shown.name, shown.name_len), Some("Bob"));
        assert_eq!(shown.settings, "LessLag");
        assert_eq!(shown.interface, "Wi-Fi");
        assert_eq!(shown.latency, RemoteLatency::Buffer(12));
        clear_table(&mut table);
        let cleared = shown_from(&mut table);
        assert!(!cleared.known);
        assert_eq!(cleared.settings, "");
        assert_eq!(cleared.latency, RemoteLatency::Unknown);
    }

    #[test]
    fn reconnect_label_keeps_the_latched_session() {
        assert_eq!(stealth_label(false, None), "Off");
        assert_eq!(stealth_label(true, None), "On");
        assert_eq!(stealth_label(true, Some(false)), "On (reconnect)");
        assert_eq!(stealth_label(false, Some(true)), "Off (reconnect)");
        assert_eq!(stealth_label(true, Some(true)), "On");
    }

    fn named(id: u64, name: &[u8], interface_raw: u8, mode: AnnounceMode) -> Peer {
        let mut bytes = [0u8; NAME_CAP];
        let len = name.len().min(NAME_CAP);
        bytes[..len].copy_from_slice(&name[..len]);
        Peer {
            constant_id: id,
            incoming: 1,
            outgoing: 2,
            modded: true,
            interface_raw,
            name: bytes,
            name_len: len as u8,
            payload_known: true,
            latency_bits: 0,
            profile_bits: preset_bits(mode),
        }
    }

    #[test]
    fn one_selected_station_owns_the_name_and_cycles_without_the_local_index() {
        let mut table = Table::empty();
        assert!(!shown_from(&mut table).known);
        table.peers[0] = Some(named(11, b"Ann", 2, AnnounceMode::Vanilla));
        let alone = shown_from(&mut table);
        assert!(alone.known);
        assert_eq!(alone.station, None);
        assert_eq!(name_label(&alone.name, alone.name_len), Some("Ann"));
        table.peers[1] = Some(named(22, b"Bob", 1, AnnounceMode::LessLag));
        assert!(!shown_from(&mut table).known);
        table.local_index = 0;
        bind_index(&mut table, 0, 11);
        bind_index(&mut table, 4, 11);
        bind_index(&mut table, 9, 22);
        bind_index(&mut table, 253, 22);
        let shown = shown_from(&mut table);
        assert_eq!(shown.station, Some(4));
        assert_eq!(name_label(&shown.name, shown.name_len), Some("Ann"));
        assert_eq!(shown.interface, "Ethernet");
        assert_eq!(shown.settings, "Vanilla");
        cycle_table(&mut table, 1);
        let shown = shown_from(&mut table);
        assert_eq!(shown.station, Some(9));
        assert_eq!(name_label(&shown.name, shown.name_len), Some("Bob"));
        assert_eq!(shown.interface, "Wi-Fi");
        assert_eq!(shown.settings, "LessLag");
        cycle_table(&mut table, -1);
        assert_eq!(shown_from(&mut table).station, Some(4));
        table.index_id[4] = 0;
        table.selected = NONE_STATION;
        let shown = shown_from(&mut table);
        assert_eq!(shown.station, Some(9));
        table.peers[0] = Some(named(33, b"Cora", 2, AnnounceMode::Ultra));
        bind_index(&mut table, 4, 33);
        assert_eq!(shown_from(&mut table).station, Some(9));
        cycle_table(&mut table, 1);
        let shown = shown_from(&mut table);
        assert_eq!(shown.station, Some(4));
        assert_eq!(name_label(&shown.name, shown.name_len), Some("Cora"));
        assert_eq!(shown.settings, "LLUltra");
        let mut bad = named(44, &[0xff, 0xfe], 1, AnnounceMode::Vanilla);
        bad.name[0] = 0xff;
        bad.name[1] = 0xfe;
        bad.name_len = 2;
        table.peers[2] = Some(bad);
        bind_index(&mut table, 5, 44);
        cycle_table(&mut table, 1);
        let shown = shown_from(&mut table);
        assert_eq!(shown.station, Some(5));
        assert!(name_label(&shown.name, shown.name_len).is_none());
        let (found, count) = candidates(&table);
        assert!(found[..count].iter().all(|index| *index != 0));
    }

    #[test]
    fn arena_roster_keeps_each_station_identity_and_skips_local_and_unmapped() {
        let mut table = Table::empty();
        let mut ann = named(11, b"Ann", 2, AnnounceMode::Vanilla);
        ann.latency_bits = 4;
        let mut bob = named(22, b"Bob", 1, AnnounceMode::LessLag);
        bob.latency_bits = 255;
        table.peers[0] = Some(ann);
        table.peers[1] = Some(bob);
        table.peers[2] = Some(named(33, b"Cyd", 1, AnnounceMode::Ultra));
        table.local_index = 1;
        bind_index(&mut table, 1, 11);
        bind_index(&mut table, 4, 11);
        bind_index(&mut table, 9, 22);
        let roster = roster_of(&mut table);
        assert!(!roster.busy);
        assert_eq!(roster.len, 2);
        assert_eq!(roster.peers[0].station, Some(4));
        assert_eq!(
            name_label(&roster.peers[0].name, roster.peers[0].name_len),
            Some("Ann")
        );
        assert_eq!(roster.peers[0].settings, "Vanilla");
        assert_eq!(roster.peers[0].latency, RemoteLatency::Buffer(4));
        assert_eq!(roster.peers[1].station, Some(9));
        assert_eq!(
            name_label(&roster.peers[1].name, roster.peers[1].name_len),
            Some("Bob")
        );
        assert_eq!(roster.peers[1].settings, "LessLag");
        assert_eq!(roster.peers[1].latency, RemoteLatency::Auto);
        assert!(roster.peers[2..].iter().all(|peer| !peer.known));
    }

    #[test]
    fn busy_banner_snapshot_copies_no_names() {
        let _gate = global_gate();
        BUSY.store(true, Ordering::Release);
        let (frame, roster) = banner_peers();
        BUSY.store(false, Ordering::Release);
        assert!(roster.busy);
        assert_eq!(roster.len, 0);
        assert!(roster.peers.iter().all(|peer| !peer.known));
        assert_eq!(frame.selection, Selection::Busy);
        assert!(!frame.peer.known);
    }

    #[test]
    fn publication_targets_follow_the_roster_and_go_busy_without_names() {
        let _gate = global_gate();
        for bits in &LEAVE_BITS {
            bits.store(0, Ordering::Relaxed);
        }
        BUSY.store(false, Ordering::Release);
        clear_session();
        seed_global_peer(0, named(11, b"Ann", 2, AnnounceMode::Vanilla));
        seed_global_peer(1, named(22, b"Bob", 1, AnnounceMode::LessLag));
        note_local(1);
        bind_station(4, 11);
        bind_station(9, 22);
        let targets = publication_targets();
        assert!(!targets.busy);
        assert_eq!(targets.len, 2);
        assert_eq!(targets.stations[0], 4);
        assert_eq!(targets.stations[1], 9);
        assert_eq!(targets.selected, crate::pia_rtt::StationPick::Index(4));
        BUSY.store(true, Ordering::Release);
        let hidden = publication_targets();
        BUSY.store(false, Ordering::Release);
        assert!(hidden.busy);
        assert_eq!(hidden.len, 0);
        assert_eq!(hidden.selected, crate::pia_rtt::StationPick::Busy);
        clear_session();
    }

    #[test]
    fn a_busy_leave_or_session_clear_drops_that_station_on_the_next_holder() {
        let _gate = global_gate();
        for bits in &LEAVE_BITS {
            bits.store(0, Ordering::Relaxed);
        }
        BUSY.store(false, Ordering::Release);
        clear_session();
        seed_global_peer(0, named(11, b"Ann", 2, AnnounceMode::Vanilla));
        seed_global_peer(1, named(22, b"Bob", 1, AnnounceMode::LessLag));
        note_local(1);
        bind_station(4, 11);
        bind_station(9, 22);
        bind_station(300, 22);
        assert_eq!(selection_snapshot(), Selection::Station(4));
        assert!(BUSY
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok());
        clear_station(4);
        assert_eq!(selection_snapshot(), Selection::Busy);
        BUSY.store(false, Ordering::Release);
        let shown = peer_frame().peer;
        assert_eq!(shown.station, Some(9));
        assert_eq!(name_label(&shown.name, shown.name_len), Some("Bob"));
        assert_eq!(shown.settings, "LessLag");
        seed_global_peer(0, named(33, b"Cora", 2, AnnounceMode::Ultra));
        bind_station(4, 33);
        assert_eq!(selected_station(), Some(9));
        cycle_peer(1);
        let shown = peer_frame().peer;
        assert_eq!(shown.station, Some(4));
        assert_eq!(name_label(&shown.name, shown.name_len), Some("Cora"));
        assert!(BUSY
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok());
        clear_session();
        BUSY.store(false, Ordering::Release);
        assert_eq!(selection_snapshot(), Selection::None);
        assert!(!peer_frame().peer.known);
        assert!(!remote_rows_visible(true));
        assert!(remote_rows_visible(false));
        clear_session();
    }

    #[test]
    fn a_connected_station_stays_visible_before_its_ping_and_leave_clears_it() {
        let _gate = global_gate();
        clear_session();
        assert!(!opponent_connected());
        assert!(!opponent_column_visible(false, Selection::None));
        assert!(!opponent_column_visible(false, Selection::Busy));
        assert!(!station_connected(Selection::None));
        let mut pending = named(44, b"Ann", 2, AnnounceMode::Vanilla);
        pending.payload_known = false;
        pending.modded = false;
        seed_global_peer(0, pending);
        bind_station(4, 44);
        let frame = peer_frame();
        assert!(station_connected(frame.selection));
        assert_eq!(frame.peer.latency, RemoteLatency::Unknown);
        assert!(opponent_column_visible(false, frame.selection));
        assert!(!opponent_column_visible(true, frame.selection));
        clear_session();
        assert!(!opponent_connected());
        assert_eq!(peer_frame().selection, Selection::None);
        assert!(!opponent_column_visible(false, peer_frame().selection));
    }

    #[test]
    fn an_unbound_station_leave_retires_the_only_confirmed_peer() {
        let _gate = global_gate();
        let mut table = Table::empty();
        let ann = 0x1122_3344_5566_7788u64;
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            false,
        );
        feed(
            &mut table,
            ann,
            0x10,
            KIND_STATION,
            &response(b"Ann", true, 1, 2),
            false,
        );
        table.local_index = 1;
        // No station map (peer observer off): Ann shows through the
        // single-peer fallback without a station selection.
        let shown = frame_from(view_of(&mut table));
        assert!(shown.peer.known);
        assert_eq!(shown.selection, Selection::None);
        // The opponent declines the rematch: the game reports station 4
        // leaving, which nothing bound. The only confirmed peer is them.
        LEAVE_BITS[0].fetch_or(1 << 4, Ordering::Release);
        apply_leaves(&mut table);
        let gone = frame_from(view_of(&mut table));
        assert!(!gone.peer.known);
        assert_eq!(gone.selection, Selection::None);
        // Two confirmed peers: an unbound leave cannot tell them apart and
        // retires neither.
        table.peers[0] = Some(named(0x55, b"Ann", 1, AnnounceMode::Vanilla));
        table.peers[1] = Some(named(0x99, b"Bob", 1, AnnounceMode::LessLag));
        LEAVE_BITS[0].fetch_or(1 << 5, Ordering::Release);
        apply_leaves(&mut table);
        assert_eq!(table.peers.iter().flatten().count(), 2);
    }

    #[test]
    fn identified_disconnect_retires_aliases_and_a_later_join_shows_again() {
        let mut table = Table::empty();
        let ann = 0x1122_3344_5566_7788u64;
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x10, 0x20),
            false,
        );
        feed(
            &mut table,
            ann,
            0x10,
            KIND_STATION,
            &response(b"Ann", true, 1, 2),
            false,
        );
        table.local_index = 1;
        bind_index(&mut table, 4, ann);
        let bits = preset_bits(AnnounceMode::Vanilla);
        feed(&mut table, 1, 0x10, KIND_RTT, &rtt_reply(4, bits), false);
        let frame = frame_from(view_of(&mut table));
        assert_eq!(frame.selection, Selection::Station(4));
        assert_eq!(frame.peer.settings, "Vanilla");
        assert_eq!(
            name_label(&frame.peer.name, frame.peer.name_len),
            Some("Ann")
        );
        assert!(opponent_column_visible(false, frame.selection));
        // Both indices belong to the departing peer before its disconnect.
        bind_index(&mut table, 7, ann);
        feed(&mut table, ann, 0, KIND_STATION, &[4, 0], false);
        let gone = frame_from(view_of(&mut table));
        assert_eq!(gone.selection, Selection::None);
        assert!(!gone.peer.known);
        assert!(!opponent_column_visible(false, gone.selection));
        assert_eq!(table.index_id[4], 0);
        assert_eq!(table.index_id[7], 0);
        let bob = 0x99;
        table.peers[1] = Some(named(bob, b"Bob", 1, AnnounceMode::LessLag));
        bind_index(&mut table, 9, bob);
        feed(&mut table, 0, 0, KIND_STATION, &[3], false);
        let kept = frame_from(view_of(&mut table));
        assert_eq!(kept.selection, Selection::Station(9));
        assert_eq!(name_label(&kept.peer.name, kept.peer.name_len), Some("Bob"));
        assert_eq!(kept.peer.settings, "LessLag");
        feed(&mut table, bob, 0, KIND_STATION, &[3], false);
        let local = frame_from(view_of(&mut table));
        assert_eq!(local.selection, Selection::None);
        assert!(!local.peer.known);
        feed(
            &mut table,
            0,
            0,
            KIND_STATION,
            &request(1, 0x30, 0x40),
            false,
        );
        feed(
            &mut table,
            ann,
            0x30,
            KIND_STATION,
            &response(b"Ann", true, 1, 2),
            false,
        );
        bind_index(&mut table, 4, ann);
        let back = frame_from(view_of(&mut table));
        assert_eq!(back.selection, Selection::Station(4));
        assert_eq!(name_label(&back.peer.name, back.peer.name_len), Some("Ann"));
        assert!(opponent_column_visible(false, back.selection));
    }

    #[test]
    fn station_leave_drops_alias_indices_and_keeps_the_other_peer() {
        let _gate = global_gate();
        for bits in &LEAVE_BITS {
            bits.store(0, Ordering::Relaxed);
        }
        BUSY.store(false, Ordering::Release);
        clear_session();
        seed_global_peer(0, named(11, b"Ann", 2, AnnounceMode::Vanilla));
        seed_global_peer(1, named(22, b"Bob", 1, AnnounceMode::LessLag));
        note_local(1);
        bind_station(4, 11);
        bind_station(7, 11);
        bind_station(9, 22);
        let frame = peer_frame();
        assert_eq!(frame.peer.settings, "Vanilla");
        assert!(opponent_column_visible(false, frame.selection));
        clear_station(7);
        let frame = peer_frame();
        assert_eq!(frame.selection, Selection::Station(9));
        assert_eq!(
            name_label(&frame.peer.name, frame.peer.name_len),
            Some("Bob")
        );
        assert_eq!(frame.peer.settings, "LessLag");
        clear_station(9);
        let frame = peer_frame();
        assert_eq!(frame.selection, Selection::None);
        assert!(!frame.peer.known);
        assert!(!opponent_column_visible(false, frame.selection));
        clear_session();
    }

    fn published(station: u8, epoch: u64) -> crate::pia_rtt::Publication {
        crate::pia_rtt::Publication::from_reading(
            crate::pia_rtt::PingReading::Measured(20),
            10,
            1,
            epoch,
            crate::pia_rtt::pack_peer(crate::pia_rtt::Stability::Stable, station),
        )
    }

    #[test]
    fn cycling_or_leaving_a_peer_hides_its_rtt_until_republish() {
        let _gate = global_gate();
        for bits in &LEAVE_BITS {
            bits.store(0, Ordering::Relaxed);
        }
        BUSY.store(false, Ordering::Release);
        clear_session();
        seed_global_peer(0, named(11, b"Ann", 2, AnnounceMode::Vanilla));
        seed_global_peer(1, named(22, b"Bob", 1, AnnounceMode::LessLag));
        note_local(1);
        bind_station(4, 11);
        bind_station(9, 22);
        let frame = peer_frame();
        assert_eq!(frame.selection, Selection::Station(4));
        assert_eq!(
            name_label(&frame.peer.name, frame.peer.name_len),
            Some("Ann")
        );
        assert_eq!(frame.peer.interface, "Ethernet");
        assert_eq!(frame.peer.settings, "Vanilla");
        let old = published(4, 3);
        assert!(crate::pia_rtt::rtt_matches(
            display_identity(frame.selection),
            old,
            3
        ));
        cycle_peer(1);
        let frame = peer_frame();
        assert_eq!(frame.selection, Selection::Station(9));
        assert_eq!(
            name_label(&frame.peer.name, frame.peer.name_len),
            Some("Bob")
        );
        assert!(!crate::pia_rtt::rtt_matches(
            display_identity(frame.selection),
            old,
            3
        ));
        let republished = published(9, 3);
        assert!(crate::pia_rtt::rtt_matches(
            display_identity(frame.selection),
            republished,
            3
        ));
        clear_station(9);
        let frame = peer_frame();
        assert_eq!(frame.selection, Selection::Station(4));
        assert!(!crate::pia_rtt::rtt_matches(
            display_identity(frame.selection),
            republished,
            3
        ));
        clear_session();
        let frame = peer_frame();
        assert_eq!(frame.selection, Selection::None);
        assert!(!frame.peer.known);
        assert!(!crate::pia_rtt::rtt_matches(
            display_identity(frame.selection),
            republished,
            4
        ));
        assert!(crate::pia_rtt::rtt_matches(
            display_identity(Selection::None),
            published(2, 4),
            4
        ));
        assert!(!crate::pia_rtt::rtt_matches(
            display_identity(Selection::Busy),
            published(2, 4),
            4
        ));
        clear_session();
    }
}
