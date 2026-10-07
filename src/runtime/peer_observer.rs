//! Opt-in inbound observer. Hooks publish atomics. The diagnostic thread logs.
//!
//! One compare-exchange gate covers map updates, the ring, and the poll copy.
//! A failed exchange returns immediately. The poll copies under the gate and
//! calls `record` only after releasing it.
use super::{record, Code};
use crate::peer_observer::{
    classify_index, diagnose, diagnose_send, find_source, message_body_range, present_observation,
    stamp_send_seq, station_probe_flags, Diagnostic, IndexChange, MessageBody, SendPrefix,
    MESSAGE_SOURCE_OFFSET, PARSED_MESSAGE_GUARD, PARSED_MESSAGE_OFFSET, SEND_CODE_DESTINATION,
    SEND_CODE_DETAIL, SEND_CODE_DROP, SEND_CODE_IDENTITY, SEND_CODE_PACKET, SEND_PLAINTEXT_GUARD,
    SEND_PLAINTEXT_OFFSET, STATION_INDEX_FIELD, STATION_INDEX_GUARD, STATION_INDEX_OFFSET,
    STATION_SLOTS, WIRE_LEN_MAX, WIRE_LEN_MIN,
};
use crate::peer_passive::{custom_write_allowed, MESSAGE_AREA_MAX, PIA_MAGIC};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};

const RING: usize = 4;

struct Slot {
    hash: AtomicU64,
    packed: AtomicU64,
    generation: AtomicU64,
    name0: AtomicU64,
    name1: AtomicU64,
}

impl Slot {
    const fn empty() -> Self {
        Self {
            hash: AtomicU64::new(0),
            packed: AtomicU64::new(0),
            generation: AtomicU64::new(0),
            name0: AtomicU64::new(0),
            name1: AtomicU64::new(0),
        }
    }
}

#[derive(Clone, Copy)]
struct Copied {
    hash: u64,
    shown: u64,
    map: bool,
    name0: u64,
    name1: u64,
    name_len: usize,
}

static INSTALLED: AtomicBool = AtomicBool::new(false);
static BUSY: AtomicBool = AtomicBool::new(false);
static HEAD: AtomicU64 = AtomicU64::new(0);
static LAST: AtomicU64 = AtomicU64::new(0);
static DROPPED: AtomicU64 = AtomicU64::new(0);
static LOGGED_DROPS: AtomicU64 = AtomicU64::new(0);
static CLEAR_ALL: AtomicBool = AtomicBool::new(false);
static PENDING: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];
static IDS: [AtomicU64; STATION_SLOTS] = [const { AtomicU64::new(0) }; STATION_SLOTS];
static GENS: [AtomicU64; STATION_SLOTS] = [const { AtomicU64::new(0) }; STATION_SLOTS];
static SLOTS: [Slot; RING] = [const { Slot::empty() }; RING];

const SEND_RING: usize = 64;

struct SendSlot {
    code: AtomicU64,
    a: AtomicU64,
    b: AtomicU64,
}

impl SendSlot {
    const fn empty() -> Self {
        Self {
            code: AtomicU64::new(0),
            a: AtomicU64::new(0),
            b: AtomicU64::new(0),
        }
    }
}

#[derive(Clone, Copy)]
struct CopiedSend {
    code: u16,
    a: u64,
    b: u64,
}

static SEND_INSTALLED: AtomicBool = AtomicBool::new(false);
static SEND_BUSY: AtomicBool = AtomicBool::new(false);
static SEND_HEAD: AtomicU64 = AtomicU64::new(0);
static SEND_LAST: AtomicU64 = AtomicU64::new(0);
static SEND_DROPPED: AtomicU64 = AtomicU64::new(0);
static SEND_LOGGED_DROPS: AtomicU64 = AtomicU64::new(0);
static SEND_SEQ: AtomicU64 = AtomicU64::new(0);
static SEND_SLOTS: [SendSlot; SEND_RING] = [const { SendSlot::empty() }; SEND_RING];
static MAP_HITS: AtomicU64 = AtomicU64::new(0);
static MAP_FLAGS: AtomicU64 = AtomicU64::new(0);
static MAP_MIX: AtomicU64 = AtomicU64::new(0);
static MAP_PUBLISHED: AtomicU64 = AtomicU64::new(u64::MAX);
static LOG_LEARNED: AtomicU64 = AtomicU64::new(0);
static LOG_CONFIRMED: AtomicU64 = AtomicU64::new(0);
static LOG_CLEANUPS: AtomicU64 = AtomicU64::new(0);
static LOG_BEACON_WRITES: AtomicU64 = AtomicU64::new(0);
static LOG_BEACON_SKIPS: AtomicU64 = AtomicU64::new(0);
static LOG_PAYLOAD_WRITES: AtomicU64 = AtomicU64::new(0);
static LOG_PAYLOAD_SKIPS: AtomicU64 = AtomicU64::new(0);
static LOG_STEALTH: AtomicU64 = AtomicU64::new(0);
static LOG_AMBIGUOUS: AtomicU64 = AtomicU64::new(0);
static LOG_CONTENTIONS: AtomicU64 = AtomicU64::new(0);
/// Cached `nn::nifm::GetInternetConnectionStatus` type byte. `0` is unknown.
static LOCAL_INTERFACE: AtomicU8 = AtomicU8::new(0);

extern "C" {
    #[link_name = "\u{1}_ZN2nn4nifm27GetInternetConnectionStatusEPNS0_24InternetConnectionStatusE"]
    fn GetInternetConnectionStatus(out: *mut u8) -> u32;
}

/// Manager wrapper `0x10860`: pass a zeroed status object and keep the low
/// byte of the first word on success. Failure leaves that byte at 0. Values
/// other than Wifi (1) and Ethernet (2), including the manager's emulated 3,
/// stay 0. This byte is not a platform label.
pub(super) fn refresh_local_facts() {
    let mut raw = [0u8; 12];
    let result = unsafe { GetInternetConnectionStatus(raw.as_mut_ptr()) };
    let byte = if result == 0 && matches!(raw[0], 1 | 2) {
        raw[0]
    } else {
        0
    };
    LOCAL_INTERFACE.store(byte, Ordering::Relaxed);
}

pub(super) fn local_interface_label() -> &'static str {
    crate::peer_transport::local_link_label(LOCAL_INTERFACE.load(Ordering::Relaxed))
}

pub(super) fn install() {
    refresh_local_facts();
    if !super::pia_rtt::ready() {
        record(Code::HookRejected, 0, 11);
        skyline::println!(
            "[Deluxe] connection transport not installed; Pia cleanup hook is not active"
        );
        return;
    }
    let parsed_guard =
        unsafe { guard_matches(PARSED_MESSAGE_OFFSET, PARSED_MESSAGE_GUARD.as_slice()) };
    let send_guard =
        unsafe { guard_matches(SEND_PLAINTEXT_OFFSET, SEND_PLAINTEXT_GUARD.as_slice()) };
    if !parsed_guard || !send_guard {
        let offset = if !parsed_guard {
            PARSED_MESSAGE_OFFSET
        } else {
            SEND_PLAINTEXT_OFFSET
        };
        record(Code::HookRejected, offset as u64, 0);
        skyline::println!(
            "[Deluxe] connection transport guard mismatch at {offset:#x}; not installed"
        );
        return;
    }
    skyline::install_hooks!(
        deluxe_rewrite_observe_parsed_message,
        deluxe_rewrite_observe_send_plaintext
    );
    record(
        Code::HookInstalled,
        PARSED_MESSAGE_OFFSET as u64,
        SEND_PLAINTEXT_OFFSET as u64,
    );
    skyline::println!("[Deluxe] connection transport installed");
    if unsafe { guard_matches(STATION_INDEX_OFFSET, STATION_INDEX_GUARD.as_slice()) } {
        skyline::install_hook!(deluxe_rewrite_observe_station_index);
        record(
            Code::HookInstalled,
            PARSED_MESSAGE_OFFSET as u64,
            STATION_INDEX_OFFSET as u64,
        );
        skyline::println!("[Deluxe] station map installed");
    } else {
        record(Code::HookRejected, STATION_INDEX_OFFSET as u64, 0x153844);
        skyline::println!("[Deluxe] station map guard mismatch; map not installed");
    }
    if !super::settings().peer_observer {
        return;
    }
    SEND_INSTALLED.store(true, Ordering::Release);
    INSTALLED.store(true, Ordering::Release);
    skyline::println!("[Deluxe] peer observer installed");
}

/// Confirmed RTT station leave. A disabled observer returns before the gate.
pub(super) fn invalidate_index(index: u32) {
    if !INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let Ok(index) = u8::try_from(index) else {
        return;
    };
    if usize::from(index) >= STATION_SLOTS {
        return;
    }
    if enter() {
        retire_index(index);
        exit();
        return;
    }
    let word = usize::from(index) / 64;
    PENDING[word].fetch_or(1u64 << (index % 64), Ordering::Release);
}

/// Confirmed RTT session end for the active protocol. Disabled returns immediately.
pub(super) fn invalidate_session() {
    if !INSTALLED.load(Ordering::Acquire) {
        return;
    }
    if enter() {
        retire_all();
        exit();
        return;
    }
    CLEAR_ALL.store(true, Ordering::Release);
}

pub(super) fn poll() {
    refresh_local_facts();
    drain_send();
    drain_station_probe();
    drain_transport();
    if !INSTALLED.load(Ordering::Acquire) {
        return;
    }
    if !enter() {
        return;
    }
    let dropped = DROPPED.load(Ordering::Relaxed);
    let logged = LOGGED_DROPS.load(Ordering::Relaxed);
    if dropped != logged {
        LOGGED_DROPS.store(dropped, Ordering::Relaxed);
    }
    let mut batch = [Copied {
        hash: 0,
        shown: 0,
        map: false,
        name0: 0,
        name1: 0,
        name_len: 0,
    }; RING];
    let mut count = 0usize;
    let head = HEAD.load(Ordering::Relaxed);
    let mut next = LAST.load(Ordering::Relaxed);
    while next != head && count < RING {
        let slot = &SLOTS[next as usize % RING];
        let packed = slot.packed.load(Ordering::Relaxed);
        let generation = slot.generation.load(Ordering::Relaxed);
        let index = (packed & 0xff) as usize;
        let live = if index < STATION_SLOTS {
            Some(GENS[index].load(Ordering::Relaxed))
        } else {
            None
        };
        let (shown, map) = present_observation(packed, generation, live);
        batch[count] = Copied {
            hash: slot.hash.load(Ordering::Relaxed),
            shown,
            map,
            name0: slot.name0.load(Ordering::Relaxed),
            name1: slot.name1.load(Ordering::Relaxed),
            name_len: ((packed >> 32) & 0xff) as usize,
        };
        count += 1;
        next = next.wrapping_add(1);
    }
    LAST.store(next, Ordering::Relaxed);
    exit();
    if dropped != logged {
        record(Code::PeerObserveDrop, dropped, 0);
    }
    for item in &batch[..count] {
        let code = if item.map {
            Code::PeerObserveMap
        } else {
            Code::PeerObserve
        };
        record(code, item.hash, item.shown);
        if item.name_len > 0 {
            record(Code::PeerObserveName, item.name0, item.name1);
        }
    }
}

fn note_index(index: u8, source_id: u64) {
    if usize::from(index) >= STATION_SLOTS {
        return;
    }
    if !enter() {
        DROPPED.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let slot = usize::from(index);
    let previous = IDS[slot].load(Ordering::Relaxed);
    if classify_index(previous, index, source_id) != IndexChange::Updated {
        exit();
        return;
    }
    IDS[slot].store(source_id, Ordering::Relaxed);
    let generation = GENS[slot].fetch_add(1, Ordering::Relaxed).wrapping_add(1);
    let mut sample = diagnose(source_id, 0, &[], Some((index, generation)));
    sample.map_update = true;
    if !enqueue(sample) {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
    exit();
}

fn enter() -> bool {
    if BUSY
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return false;
    }
    apply_pending();
    true
}

fn exit() {
    BUSY.store(false, Ordering::Release);
}

fn apply_pending() {
    if CLEAR_ALL.swap(false, Ordering::AcqRel) {
        retire_all();
    }
    for (word, pending) in PENDING.iter().enumerate() {
        let mut bits = pending.swap(0, Ordering::AcqRel);
        while bits != 0 {
            let bit = bits.trailing_zeros() as usize;
            let index = word * 64 + bit;
            if index < STATION_SLOTS {
                retire_index(index as u8);
            }
            bits &= bits - 1;
        }
    }
}

fn retire_index(index: u8) {
    let Some((id, generation)) = zero_slot(index) else {
        return;
    };
    let mut sample = diagnose(id, 0, &[], Some((index, generation)));
    sample.map_update = true;
    if !enqueue(sample) {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}

fn retire_all() {
    let mut any = false;
    for index in 0..STATION_SLOTS {
        if zero_slot(index as u8).is_some() {
            any = true;
        }
    }
    if !any {
        return;
    }
    let mut sample = diagnose(0, 0, &[], None);
    sample.map_update = true;
    if !enqueue(sample) {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}

fn zero_slot(index: u8) -> Option<(u64, u64)> {
    let slot = usize::from(index);
    let previous = IDS[slot].load(Ordering::Relaxed);
    if previous == 0 {
        return None;
    }
    IDS[slot].store(0, Ordering::Relaxed);
    let generation = GENS[slot].fetch_add(1, Ordering::Relaxed).wrapping_add(1);
    Some((previous, generation))
}

fn enqueue(sample: Diagnostic) -> bool {
    let head = HEAD.load(Ordering::Relaxed);
    let last = LAST.load(Ordering::Relaxed);
    if head.wrapping_sub(last) >= RING as u64 {
        return false;
    }
    let slot = &SLOTS[head as usize % RING];
    slot.hash.store(sample.source_hash, Ordering::Relaxed);
    slot.packed.store(pack(&sample), Ordering::Relaxed);
    slot.generation.store(sample.generation, Ordering::Relaxed);
    let (name0, name1) = name_words(&sample.name);
    slot.name0.store(name0, Ordering::Relaxed);
    slot.name1.store(name1, Ordering::Relaxed);
    HEAD.store(head.wrapping_add(1), Ordering::Relaxed);
    true
}

fn pack(sample: &Diagnostic) -> u64 {
    u64::from(sample.index)
        | (u64::from(sample.kind) << 8)
        | (u64::from(sample.subtype) << 16)
        | (u64::from(sample.interface) << 24)
        | (u64::from(sample.name_len) << 32)
        | (u64::from(sample.truncated) << 40)
        | (u64::from(sample.interface_known) << 41)
        | (u64::from(sample.disconnect) << 42)
        | (u64::from(sample.map_update) << 56)
}

fn name_words(name: &[u8; 16]) -> (u64, u64) {
    let mut first = [0u8; 8];
    let mut second = [0u8; 8];
    first.copy_from_slice(&name[..8]);
    second.copy_from_slice(&name[8..]);
    (u64::from_le_bytes(first), u64::from_le_bytes(second))
}

fn lookup(source_id: u64) -> Option<(u8, u64)> {
    let mut stored = [0u64; STATION_SLOTS];
    for (slot, id) in stored.iter_mut().zip(IDS.iter()) {
        *slot = id.load(Ordering::Relaxed);
    }
    let index = find_source(&stored, source_id)?;
    let generation = GENS[usize::from(index)].load(Ordering::Relaxed);
    Some((index, generation))
}

unsafe fn guard_matches(offset: usize, expected: &[u8]) -> bool {
    let base = getRegionAddress(Region::Text) as usize;
    let end = getRegionAddress(Region::Rodata) as usize;
    let Some(address) = base.checked_add(offset) else {
        return false;
    };
    let Some(stop) = address.checked_add(expected.len()) else {
        return false;
    };
    if address % 4 != 0 || stop > end {
        return false;
    }
    let bytes = std::slice::from_raw_parts(address as *const u8, expected.len());
    bytes == expected
}

fn read_at<T: Copy>(base: u64, offset: usize) -> Option<T> {
    let address = (base as usize).checked_add(offset)?;
    if address < 0x10000 {
        return None;
    }
    Some(unsafe { (address as *const T).read_unaligned() })
}

fn drain_station_probe() {
    if !INSTALLED.load(Ordering::Acquire) {
        return;
    }
    let hits = MAP_HITS.load(Ordering::Relaxed);
    let previous = MAP_PUBLISHED.swap(hits, Ordering::Relaxed);
    if previous == hits {
        return;
    }
    let flags = MAP_FLAGS.load(Ordering::Relaxed);
    let mix = MAP_MIX.load(Ordering::Relaxed);
    record(Code::StationProbe, hits, flags);
    record(Code::StationProbeId, mix, flags);
}

#[skyline::hook(offset = STATION_INDEX_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_station_index(ctx: &InlineCtx) {
    // The displaced instruction is `ldrb w20, [x0, #0x38]` and has not run.
    // `x20` is still the message whose `+0x40` was the GetStationById argument.
    let message = ctx.registers[20].x();
    let station = ctx.registers[0].x();
    let source = read_at::<u64>(message, MESSAGE_SOURCE_OFFSET);
    let index = read_at::<u8>(station, STATION_INDEX_FIELD);
    if let (Some(source_id), Some(index)) = (source, index) {
        if source_id != 0 {
            crate::peer_transport::bind_station(u32::from(index), source_id);
        }
    }
    if !super::settings().peer_observer {
        return;
    }
    let previous = match index {
        Some(value) if usize::from(value) < STATION_SLOTS => {
            IDS[usize::from(value)].load(Ordering::Relaxed)
        }
        _ => 0,
    };
    let (flags, mix) = station_probe_flags(message, station, source, index, previous);
    MAP_MIX.store(mix, Ordering::Relaxed);
    MAP_FLAGS.store(flags, Ordering::Relaxed);
    MAP_HITS.fetch_add(1, Ordering::Relaxed);
    let Some(source_id) = source else {
        return;
    };
    let Some(index) = index else {
        return;
    };
    note_index(index, source_id);
}

#[skyline::hook(offset = PARSED_MESSAGE_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_parsed_message(ctx: &InlineCtx) {
    let packet = ctx.registers[22].x();
    let message = ctx.registers[21].x();
    let Some(wire_len) = read_at::<u32>(packet, 0x5e8) else {
        return;
    };
    let Some(message_at) = read_at::<u64>(message, 0) else {
        return;
    };
    let Some(flags) = read_at::<u8>(message, 0x30) else {
        return;
    };
    let Some(body_len) = read_at::<u16>(message, 0x32) else {
        return;
    };
    let Some(kind_word) = read_at::<u32>(message, 0x34) else {
        return;
    };
    let Some(source_id) = read_at::<u64>(message, 0x40) else {
        return;
    };
    let kind = crate::peer_observer::kind_of(kind_word);
    let record_peer = kind == 0x14 && super::settings().peer_observer;
    if kind != 0x14 && kind != 0x58 {
        return;
    }
    let Some(range) = message_body_range(packet, wire_len, message_at, flags, body_len) else {
        if record_peer {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
        return;
    };
    let mut inline_body = [0u8; 0x5a0];
    let body = match range {
        MessageBody::Empty => &[][..],
        MessageBody::Bytes { at, len } => {
            if at < 0x10000 {
                DROPPED.fetch_add(1, Ordering::Relaxed);
                return;
            }
            unsafe {
                core::ptr::copy_nonoverlapping(at as *const u8, inline_body.as_mut_ptr(), len);
            }
            &inline_body[..len]
        }
    };
    let incoming = read_at::<u8>(packet, 0x2d).unwrap_or(0);
    crate::peer_transport::observe_incoming(source_id, incoming, kind, body, super::stealth());
    if !record_peer {
        return;
    }
    if !enter() {
        DROPPED.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let sample = diagnose(source_id, kind_word, body, lookup(source_id));
    if !enqueue(sample) {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
    exit();
}

fn drain_send() {
    if !SEND_INSTALLED.load(Ordering::Acquire) {
        return;
    }
    if !send_enter() {
        return;
    }
    let dropped = SEND_DROPPED.load(Ordering::Relaxed);
    let logged = SEND_LOGGED_DROPS.load(Ordering::Relaxed);
    if dropped != logged {
        SEND_LOGGED_DROPS.store(dropped, Ordering::Relaxed);
    }
    let seq = SEND_SEQ.load(Ordering::Relaxed);
    let head = SEND_HEAD.load(Ordering::Relaxed);
    let mut next = SEND_LAST.load(Ordering::Relaxed);
    let mut batch = [CopiedSend {
        code: 0,
        a: 0,
        b: 0,
    }; SEND_RING];
    let mut count = 0usize;
    while next != head && count < SEND_RING {
        let slot = &SEND_SLOTS[next as usize % SEND_RING];
        batch[count] = CopiedSend {
            code: slot.code.load(Ordering::Relaxed) as u16,
            a: slot.a.load(Ordering::Relaxed),
            b: slot.b.load(Ordering::Relaxed),
        };
        count += 1;
        next = next.wrapping_add(1);
    }
    SEND_LAST.store(next, Ordering::Relaxed);
    send_exit();
    if dropped != logged {
        record(Code::SendDrop, dropped, seq);
    }
    for event in &batch[..count] {
        if let Some(code) = send_code(event.code) {
            record(code, event.a, event.b);
        }
    }
}

fn send_code(code: u16) -> Option<Code> {
    Some(match code {
        SEND_CODE_PACKET => Code::SendPacket,
        SEND_CODE_IDENTITY => Code::SendIdentity,
        SEND_CODE_DESTINATION => Code::SendDestination,
        SEND_CODE_DETAIL => Code::SendDetail,
        SEND_CODE_DROP => Code::SendDrop,
        _ => return None,
    })
}

fn send_enter() -> bool {
    SEND_BUSY
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

fn send_exit() {
    SEND_BUSY.store(false, Ordering::Release);
}

fn note_send_drop(count: u64) {
    SEND_DROPPED.fetch_add(count, Ordering::Relaxed);
}

fn publish_send(report: &crate::peer_observer::SendReport) {
    if report.len == 0 {
        return;
    }
    if !send_enter() {
        note_send_drop(report.len as u64);
        return;
    }
    let seq = SEND_SEQ.fetch_add(1, Ordering::Relaxed) as u16;
    let mut events = report.events;
    stamp_send_seq(&mut events[..report.len], seq);
    let head = SEND_HEAD.load(Ordering::Relaxed);
    let last = SEND_LAST.load(Ordering::Relaxed);
    let used = head.wrapping_sub(last) as usize;
    let room = SEND_RING.saturating_sub(used.min(SEND_RING));
    let written = room.min(report.len);
    for (offset, event) in events[..written].iter().enumerate() {
        let slot = &SEND_SLOTS[(head as usize).wrapping_add(offset) % SEND_RING];
        slot.code.store(u64::from(event.code), Ordering::Relaxed);
        slot.a.store(event.a, Ordering::Relaxed);
        slot.b.store(event.b, Ordering::Relaxed);
    }
    SEND_HEAD.store(head.wrapping_add(written as u64), Ordering::Relaxed);
    if written < report.len {
        note_send_drop((report.len - written) as u64);
    }
    send_exit();
}

fn copy_bytes(base: u64, offset: usize, dest: &mut [u8]) -> bool {
    let Some(start) = (base as usize).checked_add(offset) else {
        return false;
    };
    if start < 0x10000 || dest.len() > MESSAGE_AREA_MAX || start.checked_add(dest.len()).is_none() {
        return false;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(start as *const u8, dest.as_mut_ptr(), dest.len());
    }
    true
}

fn read_magic(base: u64) -> Option<bool> {
    let mut bytes = [0u8; 4];
    if !copy_bytes(base, 8, &mut bytes) {
        return None;
    }
    let magic_be = u32::from_be_bytes(bytes);
    let magic_le = u32::from_le_bytes(bytes);
    Some(magic_be == PIA_MAGIC || magic_le == PIA_MAGIC)
}

fn observe_send_packet(packet: u64) {
    let Some(version_flags) = read_at::<u8>(packet, 0xc) else {
        note_send_drop(1);
        return;
    };
    let Some(connection_id) = read_at::<u8>(packet, 0xd) else {
        note_send_drop(1);
        return;
    };
    let Some(wire_byte) = read_at::<u8>(packet, 0x2d) else {
        note_send_drop(1);
        return;
    };
    let Some(wire_len) = read_at::<u32>(packet, 0x5e8) else {
        note_send_drop(1);
        return;
    };
    let Some(magic_ok) = read_magic(packet) else {
        note_send_drop(1);
        return;
    };
    let mut area = [0u8; MESSAGE_AREA_MAX];
    let area_len = if (WIRE_LEN_MIN..=WIRE_LEN_MAX).contains(&wire_len) {
        (wire_len - WIRE_LEN_MIN) as usize
    } else {
        0
    };
    if area_len > 0 && !copy_bytes(packet, 0x48, &mut area[..area_len]) {
        note_send_drop(1);
        return;
    }
    // Diagnostic correlation only. These relaxed loads are not one generation
    // of the receive map and do not take its gate, so the copy can tear.
    // Transmit identity selection must not use this snapshot.
    let mut stations = [0u64; STATION_SLOTS];
    for (slot, id) in stations.iter_mut().zip(IDS.iter()) {
        *slot = id.load(Ordering::Relaxed);
    }
    let report = diagnose_send(
        SendPrefix {
            version_flags,
            connection_id,
            wire_byte,
            wire_len,
            magic_ok,
        },
        &area[..area_len],
        0,
        &stations,
    );
    if super::settings().peer_observer {
        publish_send(&report);
    }
    if !magic_ok || area_len == 0 || !custom_write_allowed(version_flags) {
        return;
    }
    let area_bytes = &area[..area_len];
    let mut saved = [None; 8];
    crate::peer_transport::commit_outgoing(
        version_flags,
        connection_id,
        area_bytes,
        Some(current_announce()),
        LOCAL_INTERFACE.load(Ordering::Relaxed),
        super::stealth(),
        |writes, revert| {
            if revert {
                restore_body_writes(&saved);
            } else {
                saved = stage_body_writes(packet, area_len, writes);
            }
        },
    );
}

/// Wire profile is the machine's committed preset while that machine is live.
///
/// Menus and matches use the same rule. A saved mode is not sent while the
/// machine is passthrough, and core modes are not a live profile. Auto is not
/// a wire value. `Profile::Doubles` is `AnnounceMode::LessLagDoubles`. The
/// resolution discriminant and dynamic-resolution bit are the configured
/// state. A rejected resolution guard keeps that bit clear. Saved latency is
/// unchanged.
fn current_announce() -> crate::peer_transport::Announce {
    let settings = super::settings();
    let decision = crate::render_policy::load_machine().decision();
    let setting = crate::resolution::active_setting(
        decision.live,
        crate::resolution::hooks_ready(),
        decision.committed,
        settings.platform,
    );
    crate::peer_transport::announce_from(
        settings.input_buffer,
        decision.live,
        decision.committed,
        setting,
    )
}

#[derive(Clone, Copy)]
struct SavedBody {
    addr: usize,
    bytes: [u8; 4],
    len: u8,
}

fn body_addr(packet: u64, area_len: usize, at: usize, len: usize) -> Option<usize> {
    let end = at.checked_add(len)?;
    if len == 0 || len > 4 || end > area_len {
        return None;
    }
    let start = (packet as usize)
        .checked_add(0x48)
        .and_then(|base| base.checked_add(at))?;
    if start < 0x10000 || start.checked_add(len).is_none() {
        return None;
    }
    Some(start)
}

fn stage_body_writes(
    packet: u64,
    area_len: usize,
    writes: &[Option<crate::peer_transport::BodyWrite>],
) -> [Option<SavedBody>; 8] {
    let mut saved = [None; 8];
    for (slot, write) in saved.iter_mut().zip(writes.iter().flatten()) {
        let len = usize::from(write.len);
        let Some(start) = body_addr(packet, area_len, write.at, len) else {
            continue;
        };
        let mut bytes = [0u8; 4];
        unsafe {
            core::ptr::copy_nonoverlapping(start as *const u8, bytes.as_mut_ptr(), len);
            core::ptr::copy_nonoverlapping(write.bytes.as_ptr(), start as *mut u8, len);
        }
        *slot = Some(SavedBody {
            addr: start,
            bytes,
            len: write.len,
        });
    }
    saved
}

fn restore_body_writes(saved: &[Option<SavedBody>]) {
    for slot in saved.iter().flatten() {
        let len = usize::from(slot.len);
        unsafe {
            core::ptr::copy_nonoverlapping(slot.bytes.as_ptr(), slot.addr as *mut u8, len);
        }
    }
}

fn drain_transport() {
    let now = crate::peer_transport::counters();
    let learned = counter_delta(now.learned, &LOG_LEARNED);
    if learned != 0 {
        record(Code::TransportHandshake, now.last_mix, learned);
    }
    let confirmed = counter_delta(now.confirmed, &LOG_CONFIRMED);
    if confirmed != 0 {
        record(Code::TransportConfirm, now.last_mix, confirmed);
    }
    let cleanups = counter_delta(now.cleanups, &LOG_CLEANUPS);
    if cleanups != 0 {
        record(Code::TransportCleanup, cleanups, 0);
    }
    let beacon_writes = counter_delta(now.beacon_writes, &LOG_BEACON_WRITES);
    let beacon_skips = counter_delta(now.beacon_skips, &LOG_BEACON_SKIPS);
    if beacon_writes != 0 || beacon_skips != 0 {
        record(Code::TransportBeacon, beacon_writes, beacon_skips);
    }
    let payload_writes = counter_delta(now.payload_writes, &LOG_PAYLOAD_WRITES);
    let payload_skips = counter_delta(now.payload_skips, &LOG_PAYLOAD_SKIPS);
    if payload_writes != 0 || payload_skips != 0 {
        record(Code::TransportPayload, payload_writes, payload_skips);
    }
    let stealth = counter_delta(now.stealth_suppress, &LOG_STEALTH);
    if stealth != 0 {
        record(Code::TransportStealth, stealth, now.last_mix);
    }
    let ambiguous = counter_delta(now.ambiguous, &LOG_AMBIGUOUS);
    let contentions = counter_delta(now.contentions, &LOG_CONTENTIONS);
    if ambiguous != 0 || contentions != 0 {
        record(Code::TransportSkip, ambiguous, contentions);
    }
}

fn counter_delta(now: u64, slot: &AtomicU64) -> u64 {
    let previous = slot.swap(now, Ordering::Relaxed);
    now.wrapping_sub(previous)
}

#[skyline::hook(offset = SEND_PLAINTEXT_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_send_plaintext(ctx: &InlineCtx) {
    observe_send_packet(ctx.registers[0].x());
}
