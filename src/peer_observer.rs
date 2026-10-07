#![cfg_attr(not(any(test, target_os = "switch")), allow(dead_code))]

//! Opt-in diagnostic view of an already-parsed inbound Pia message.
//!
//! `parse_message_area` keeps absent header fields as `None`. The game parser
//! at text+`0x157a1c` copies the previous header and overwrites only the flag
//! bits that are present. [`inherit_header`] is that second rule. The hook
//! reads the game's header, so it does not apply [`inherit_header`] again.
//!
//! The send-site diagnostic is different: it sees plaintext bytes, so it walks
//! them with [`crate::peer_passive::walk_messages`] and then applies
//! [`inherit_header`] from a zero header at the start of that packet. The walk
//! keeps its own length register for frame size: unset is 0, a present length
//! replaces it, and a clear length bit still consumes that many body bytes.
//! `body_len` stays `None` when the bit is clear, so the logical mask can tell
//! a present length from an inherited one. Each packet starts the register at 0.
//!
//! Nothing here sends a packet. Source and destination ids are mixed before
//! they are logged. A name is a bounded escaped sample from a subtype-2 entry,
//! or it is absent. Spare-slot logs keep eight name-window bytes.

use crate::peer_passive::{
    read_response_beacon, walk_messages, Beacon, ParseError, PiaMessage, MESSAGE_AREA_MAX,
};

pub const STATION_SLOTS: usize = 253;
pub const PARSED_MESSAGE_OFFSET: usize = 0x153844;
pub const STATION_INDEX_OFFSET: usize = 0x15f0b8;
/// Index byte on the station in `x0` at [`STATION_INDEX_OFFSET`].
pub const STATION_INDEX_FIELD: usize = 0x38;
/// Source id on the message object. That pointer is still `x20` when the
/// inline hook runs, before `ldrb w20, [x0, #0x38]` replaces `w20`.
pub const MESSAGE_SOURCE_OFFSET: usize = 0x40;
pub const ENTRY_STRIDE: usize = 0xc3;
pub const ENTRY_NAME: usize = 0x51;
pub const ENTRY_NAME_LEN: usize = 0x28;
pub const ENTRY_TAG: usize = 0x79;
pub const NAME_CAP: usize = 16;
/// Plaintext encrypt call. `x0` is the packet. The hooked word is `add x1, sp, #0x28`.
pub const SEND_PLAINTEXT_OFFSET: usize = 0x155878;
pub const SEND_CODE_PACKET: u16 = 41;
pub const SEND_CODE_IDENTITY: u16 = 42;
pub const SEND_CODE_DESTINATION: u16 = 43;
pub const SEND_CODE_DETAIL: u16 = 44;
pub const SEND_CODE_DROP: u16 = 45;
pub const SEND_STATUS_OK: u8 = 0;
pub const SEND_STATUS_TRUNCATED: u8 = 1;
pub const SEND_STATUS_OVERSIZE: u8 = 2;
pub const SEND_STATUS_LENGTH: u8 = 3;
pub const SEND_STATUS_BOUNDS: u8 = 4;
pub const SEND_STATUS_CAPACITY: u8 = 5;
pub const SEND_DETAIL_SPARE: u8 = 1;
pub const SEND_DETAIL_RTT: u8 = 2;
pub const SEND_SPARE_SHORT: u8 = 1;
pub const SEND_SPARE_NONE: u8 = 2;
pub const SEND_SPARE_PRESENT: u8 = 3;
pub const SEND_SPARE_OTHER: u8 = 4;
const SEND_IDENTITY_EARLY: usize = 4;
const SEND_DETAIL_EACH: usize = 2;
pub const SEND_EVENT_CAP: usize = 17;
const MASK_SOURCE_KNOWN: u8 = 1 << 0;
const MASK_SOURCE_INHERITED: u8 = 1 << 1;
const MASK_DEST_KNOWN: u8 = 1 << 2;
const MASK_DEST_INHERITED: u8 = 1 << 3;
const MASK_KIND_KNOWN: u8 = 1 << 4;
const MASK_KIND_INHERITED: u8 = 1 << 5;
const MASK_LENGTH_KNOWN: u8 = 1 << 6;
const MASK_LENGTH_INHERITED: u8 = 1 << 7;
const DETAIL_SEQ_SHIFT: u32 = 50;
const SPARE_HEADER: usize = 0x39;

/// Raw bytes at main text+`0x155878` in the 13.0.5 NSO. Each instruction is stored
/// little-endian, so `add x1, sp, #0x28` is `e1 a3 00 91` and decodes as `0x9100a3e1`.
pub static SEND_PLAINTEXT_GUARD: [u8; 64] = [
    0xe1, 0xa3, 0x00, 0x91, 0xf5, 0xa6, 0xfd, 0x97, 0x60, 0x1a, 0x40, 0xf9, 0xe1, 0x03, 0x14, 0x2a,
    0xb2, 0xfd, 0xff, 0x97, 0x94, 0x06, 0x00, 0x11, 0x60, 0xf9, 0xff, 0xb5, 0x60, 0x1a, 0x40, 0xf9,
    0xbe, 0xfd, 0xff, 0x97, 0x7f, 0x72, 0x0c, 0x79, 0x7f, 0xe2, 0x00, 0x39, 0x7f, 0x22, 0x00, 0xf9,
    0x7f, 0xa2, 0x01, 0x39, 0x7f, 0x3a, 0x00, 0xf9, 0x7f, 0x62, 0x02, 0x39, 0x7f, 0x52, 0x00, 0xf9,
];

/// Bytes at main text+`0x153844` in the 13.0.5 NSO (file offset = text + `0x100`).
pub static PARSED_MESSAGE_GUARD: [u8; 64] = [
    0xa8, 0x02, 0x40, 0xf9, 0x08, 0x06, 0x00, 0xb4, 0x68, 0xc2, 0x41, 0x39, 0x1f, 0x01, 0x00, 0x72,
    0xe9, 0x03, 0x00, 0x32, 0x29, 0x05, 0x89, 0x1a, 0x1f, 0x01, 0x1f, 0x72, 0x2a, 0x09, 0x00, 0x11,
    0x29, 0x01, 0x8a, 0x1a, 0x1f, 0x01, 0x1e, 0x72, 0x2a, 0x11, 0x00, 0x11, 0x29, 0x01, 0x8a, 0x1a,
    0x1f, 0x01, 0x1d, 0x72, 0x2a, 0x21, 0x00, 0x11, 0x29, 0x01, 0x8a, 0x1a, 0x1f, 0x01, 0x1c, 0x72,
];

/// Bytes at main text+`0x15f0b8`. `x0` is the `Station*` the preceding `cbz` kept.
pub static STATION_INDEX_GUARD: [u8; 64] = [
    0x14, 0xe0, 0x40, 0x39, 0x9f, 0xf6, 0x03, 0x71, 0x20, 0x04, 0x00, 0x54, 0xe8, 0x43, 0x40, 0xb9,
    0x1f, 0x05, 0x00, 0x71, 0xc0, 0x01, 0x00, 0x54, 0xa8, 0x03, 0x00, 0x35, 0xa8, 0x42, 0x00, 0x91,
    0xe8, 0x0b, 0x00, 0xf9, 0xe8, 0x03, 0x00, 0x32, 0xe8, 0x0f, 0x00, 0xf9, 0xe8, 0x27, 0x40, 0xf9,
    0xe8, 0x13, 0x00, 0xf9, 0xe8, 0x03, 0x00, 0x91, 0xe2, 0x43, 0x00, 0x91, 0xe0, 0x03, 0x13, 0xaa,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexChange {
    Ignored,
    Unchanged,
    Updated,
}

/// `0xfd` and every index above 252 are ignored. Equal ids do not bump freshness.
pub fn classify_index(previous: u64, index: u8, source_id: u64) -> IndexChange {
    if usize::from(index) >= STATION_SLOTS {
        return IndexChange::Ignored;
    }
    if previous == source_id {
        IndexChange::Unchanged
    } else {
        IndexChange::Updated
    }
}

pub const STATION_PROBE_MESSAGE_PTR: u64 = 1;
pub const STATION_PROBE_STATION_PTR: u64 = 1 << 1;
pub const STATION_PROBE_SOURCE_READ: u64 = 1 << 2;
pub const STATION_PROBE_INDEX_READ: u64 = 1 << 3;
pub const STATION_PROBE_SOURCE_ZERO: u64 = 1 << 4;
pub const STATION_PROBE_INDEX_HIGH: u64 = 1 << 5;
pub const STATION_PROBE_UPDATED: u64 = 1 << 6;
pub const STATION_PROBE_UNCHANGED: u64 = 1 << 7;
pub const STATION_PROBE_IGNORED: u64 = 1 << 8;
pub const STATION_PROBE_INDEX_SHIFT: u32 = 16;

/// Pack one station-hook sample without reading memory.
///
/// Pointer bits are the raw registers. Read bits come from the caller, which
/// has already applied the `0x10000` check. `mix` is 0 only when the source
/// read failed; `mix_id(0)` marks a successful read of a zero id. The index
/// byte sits at bits 16..24 and is `0xff` when that read failed.
pub fn station_probe_flags(
    message: u64,
    station: u64,
    source: Option<u64>,
    index: Option<u8>,
    previous: u64,
) -> (u64, u64) {
    let mut flags = 0u64;
    if message >= 0x10000 {
        flags |= STATION_PROBE_MESSAGE_PTR;
    }
    if station >= 0x10000 {
        flags |= STATION_PROBE_STATION_PTR;
    }
    let mix = match source {
        Some(id) => {
            flags |= STATION_PROBE_SOURCE_READ;
            if id == 0 {
                flags |= STATION_PROBE_SOURCE_ZERO;
            }
            mix_id(id)
        }
        None => 0,
    };
    let index_byte = match index {
        Some(value) => {
            flags |= STATION_PROBE_INDEX_READ;
            if usize::from(value) >= STATION_SLOTS {
                flags |= STATION_PROBE_INDEX_HIGH;
            }
            value
        }
        None => 0xff,
    };
    flags |= u64::from(index_byte) << STATION_PROBE_INDEX_SHIFT;
    let change = match (source, index) {
        (Some(id), Some(value)) => classify_index(previous, value, id),
        _ => IndexChange::Ignored,
    };
    flags |= match change {
        IndexChange::Updated => STATION_PROBE_UPDATED,
        IndexChange::Unchanged => STATION_PROBE_UNCHANGED,
        IndexChange::Ignored => STATION_PROBE_IGNORED,
    };
    (flags, mix)
}

pub fn find_source(stored: &[u64], source_id: u64) -> Option<u8> {
    if source_id == 0 {
        return None;
    }
    stored
        .iter()
        .take(STATION_SLOTS)
        .position(|id| *id == source_id)
        .map(|index| index as u8)
}

/// Subtype bytes 3 and 4 are the manager's disconnect compares.
/// The game observer does not treat them as a station leave.
#[cfg_attr(not(test), allow(dead_code))]
pub fn clears_station(subtype: u8) -> bool {
    subtype == 3 || subtype == 4
}

/// Diagnostic mix. It is not the Pia id and it is not a secret.
pub fn mix_id(id: u64) -> u64 {
    let mixed = id
        .wrapping_add(0xa5a5_a5a5_a5a5_a5a5)
        .wrapping_mul(0x9e37_79b1_85eb_ca87);
    mixed ^ mixed.rotate_left(27)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub struct GameHeader {
    pub flags: u8,
    pub length: u16,
    pub kind_word: u32,
    pub destination_id: u64,
    pub source_id: u64,
}

/// Fields whose flag bit is clear keep the previous header's value.
///
/// The installed hook does not call this. It reads the header the game parser
/// already stored. `parse_message_area` is the other rule: an absent bit is
/// `None` and is not filled from the previous message.
#[cfg_attr(not(test), allow(dead_code))]
pub fn inherit_header(previous: GameHeader, current: GameHeader) -> GameHeader {
    GameHeader {
        flags: current.flags,
        length: if current.flags & 2 == 0 {
            previous.length
        } else {
            current.length
        },
        kind_word: if current.flags & 4 == 0 {
            previous.kind_word
        } else {
            current.kind_word
        },
        destination_id: if current.flags & 8 == 0 {
            previous.destination_id
        } else {
            current.destination_id
        },
        source_id: if current.flags & 16 == 0 {
            previous.source_id
        } else {
            current.source_id
        },
    }
}

pub fn kind_of(kind_word: u32) -> u8 {
    (kind_word >> 24) as u8
}

/// Bytes from the message start to its body.
///
/// `0xbf170` returns `packet + offset + 0x48`, the flags byte. The game then
/// adds 1, plus 1/2/4/8/8 when flag bits 0..4 are set (`0x153854`).
pub fn header_span(flags: u8) -> usize {
    let mut span = 1usize;
    if flags & 1 != 0 {
        span += 1;
    }
    if flags & 2 != 0 {
        span += 2;
    }
    if flags & 4 != 0 {
        span += 4;
    }
    if flags & 8 != 0 {
        span += 8;
    }
    if flags & 16 != 0 {
        span += 8;
    }
    span
}

/// Inclusive wire-length window the game stored at `packet+0x5e8`.
pub const WIRE_LEN_MIN: u32 = 0x20;
pub const WIRE_LEN_MAX: u32 = 0x5c0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageBody {
    /// The header fits and the body length is zero. Nothing is copied.
    Empty,
    Bytes {
        at: u64,
        len: usize,
    },
}

/// Body bytes inside `[packet+0x48, packet+0x28+wire_len)`.
///
/// `message_at` is the flags byte (`0xbf170`). The body begins
/// [`header_span`] bytes later. A length that would pass either end, or an
/// addition that overflows, is rejected.
pub fn message_body_range(
    packet: u64,
    wire_len: u32,
    message_at: u64,
    flags: u8,
    body_len: u16,
) -> Option<MessageBody> {
    if !(WIRE_LEN_MIN..=WIRE_LEN_MAX).contains(&wire_len) {
        return None;
    }
    let area_start = packet.checked_add(0x48)?;
    let area_end = packet.checked_add(0x28)?.checked_add(u64::from(wire_len))?;
    if !(area_start..area_end).contains(&message_at) {
        return None;
    }
    let body_at = message_at.checked_add(header_span(flags) as u64)?;
    if body_at > area_end {
        return None;
    }
    let body_end = body_at.checked_add(u64::from(body_len))?;
    if body_end > area_end {
        return None;
    }
    if body_len == 0 {
        return Some(MessageBody::Empty);
    }
    if u32::from(body_len) > 0x5a0 {
        return None;
    }
    Some(MessageBody::Bytes {
        at: body_at,
        len: usize::from(body_len),
    })
}

/// Log word for one observation.
///
/// Bits 0..43 keep the packed fields except the map marker. Bit 43 is set when
/// `live_generation` is no longer the generation stored with the event. Bits
/// 48..64 are the low 16 bits of that stored generation. The map marker is
/// returned separately so a station update is not logged as a message.
pub fn present_observation(
    packed: u64,
    generation: u64,
    live_generation: Option<u64>,
) -> (u64, bool) {
    let index = (packed & 0xff) as usize;
    let map = packed & (1 << 56) != 0;
    let stale = if index < STATION_SLOTS && live_generation.is_some_and(|live| live != generation) {
        1u64 << 43
    } else {
        0
    };
    let shown = (packed & !(1u64 << 56)) | stale | ((generation & 0xffff) << 48);
    (shown, map)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub source_hash: u64,
    pub index: u8,
    pub kind: u8,
    pub subtype: u8,
    pub interface: u8,
    pub generation: u64,
    pub name: [u8; NAME_CAP],
    pub name_len: u8,
    pub truncated: bool,
    pub interface_known: bool,
    pub disconnect: bool,
    pub map_update: bool,
}

pub fn diagnose(
    source_id: u64,
    kind_word: u32,
    body: &[u8],
    index: Option<(u8, u64)>,
) -> Diagnostic {
    let kind = kind_of(kind_word);
    let subtype = if kind == 0x14 {
        body.first().copied()
    } else {
        None
    };
    let (name, name_len, truncated, interface, interface_known) = if subtype == Some(2) {
        let decoded = decode_primary_name(body);
        let (interface, interface_known) = match read_response_beacon(body) {
            Beacon::Tagged { interface, .. } => (interface as u8, true),
            _ => (0, false),
        };
        match decoded {
            Some(sample) => (
                sample.bytes,
                sample.len,
                sample.truncated,
                interface,
                interface_known,
            ),
            None => ([0; NAME_CAP], 0, false, interface, interface_known),
        }
    } else {
        ([0; NAME_CAP], 0, false, 0, false)
    };
    let (index, generation) = match index {
        Some((index, generation)) => (index, generation),
        None => (0xff, 0),
    };
    Diagnostic {
        source_hash: mix_id(source_id),
        index,
        kind,
        subtype: subtype.unwrap_or(0xff),
        interface,
        generation,
        name,
        name_len,
        truncated,
        interface_known,
        disconnect: false,
        map_update: false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NameSample {
    bytes: [u8; NAME_CAP],
    len: u8,
    truncated: bool,
}

fn decode_primary_name(body: &[u8]) -> Option<NameSample> {
    if body.len() < 2 || body[0] != 2 || body[1] != 0 {
        return None;
    }
    if body.len() < 0x39 + ENTRY_STRIDE {
        return None;
    }
    let count = usize::from(body[0x38]);
    if count == 0 {
        return None;
    }
    let nslots = (body.len() - 0x39) / ENTRY_STRIDE;
    if nslots < count {
        return None;
    }
    let entry = body.get(0x39..0x39 + ENTRY_STRIDE)?;
    decode_entry_name(entry)
}

fn decode_entry_name(entry: &[u8]) -> Option<NameSample> {
    if entry.len() < ENTRY_TAG + 1 {
        return None;
    }
    let window = entry.get(ENTRY_NAME..ENTRY_NAME + ENTRY_NAME_LEN)?;
    let mut raw = [0u8; 64];
    let raw_len = match entry[ENTRY_TAG] {
        1 => copy_until_nul(window, &mut raw),
        2 => utf16_le_into(window, &mut raw),
        _ => return None,
    };
    Some(escape_text(&raw[..raw_len]))
}

fn copy_until_nul(window: &[u8], out: &mut [u8]) -> usize {
    let end = window
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(window.len());
    let len = end.min(out.len());
    out[..len].copy_from_slice(&window[..len]);
    len
}

fn utf16_le_into(window: &[u8], out: &mut [u8]) -> usize {
    let mut units = [0u16; 20];
    let mut count = 0usize;
    let mut index = 0usize;
    while index + 1 < window.len() && count < units.len() {
        let unit = u16::from_le_bytes([window[index], window[index + 1]]);
        index += 2;
        if unit == 0 {
            break;
        }
        units[count] = unit;
        count += 1;
    }
    let mut len = 0usize;
    let mut cursor = 0usize;
    while cursor < count {
        let unit = units[cursor];
        let scalar = if (0xD800..=0xDBFF).contains(&unit) {
            let next = units.get(cursor + 1).copied();
            if let Some(low) = next.filter(|low| (0xDC00..=0xDFFF).contains(low)) {
                cursor += 2;
                let value = 0x10000 + (u32::from(unit - 0xD800) << 10) + u32::from(low - 0xDC00);
                char::from_u32(value).unwrap_or('\u{FFFD}')
            } else {
                cursor += 1;
                '\u{FFFD}'
            }
        } else if (0xDC00..=0xDFFF).contains(&unit) {
            cursor += 1;
            '\u{FFFD}'
        } else {
            cursor += 1;
            char::from_u32(u32::from(unit)).unwrap_or('\u{FFFD}')
        };
        let mut buf = [0u8; 4];
        let encoded = scalar.encode_utf8(&mut buf);
        if len + encoded.len() > out.len() {
            break;
        }
        out[len..len + encoded.len()].copy_from_slice(encoded.as_bytes());
        len += encoded.len();
    }
    len
}

fn escape_text(text: &[u8]) -> NameSample {
    let mut bytes = [0u8; NAME_CAP];
    let mut len = 0usize;
    let mut truncated = false;
    for byte in text {
        if (0x20..0x7f).contains(byte) && *byte != b'\\' {
            if len + 1 > NAME_CAP {
                truncated = true;
                break;
            }
            bytes[len] = *byte;
            len += 1;
        } else if len + 4 > NAME_CAP {
            truncated = true;
            break;
        } else {
            bytes[len] = b'\\';
            bytes[len + 1] = b'x';
            bytes[len + 2] = hex_digit(byte >> 4);
            bytes[len + 3] = hex_digit(byte & 0xf);
            len += 4;
        }
    }
    NameSample {
        bytes,
        len: len as u8,
        truncated,
    }
}

fn hex_digit(nibble: u8) -> u8 {
    let nibble = nibble & 0xf;
    if nibble < 10 {
        b'0' + nibble
    } else {
        b'a' + (nibble - 10)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendEvent {
    pub code: u16,
    pub a: u64,
    pub b: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendPrefix {
    pub version_flags: u8,
    pub connection_id: u8,
    pub wire_byte: u8,
    pub wire_len: u32,
    pub magic_ok: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SendReport {
    pub events: [SendEvent; SEND_EVENT_CAP],
    pub len: usize,
}

impl SendReport {
    const fn empty() -> Self {
        Self {
            events: [SendEvent {
                code: 0,
                a: 0,
                b: 0,
            }; SEND_EVENT_CAP],
            len: 0,
        }
    }
}

/// Read-only send-site summary. `station_ids` is a diagnostic copy of the receive
/// map, indexed like the station hook. A destination that is not in that copy is
/// logged as `0xff`. An unknown id is logged as `0`. [`mix_id`] of zero is not
/// zero, so `0` means the field was not present in this packet. The mask bit
/// says which.
///
/// The map index is diagnostic correlation only. The copy can tear: each slot is
/// loaded independently, with no generation and no receive-path gate. A later
/// transmit identity selection must not treat this index as the peer to mark.
pub fn diagnose_send(prefix: SendPrefix, area: &[u8], seq: u16, station_ids: &[u64]) -> SendReport {
    if !(WIRE_LEN_MIN..=WIRE_LEN_MAX).contains(&prefix.wire_len) {
        return single_packet(&prefix, SEND_STATUS_LENGTH, seq);
    }
    if area.len() > MESSAGE_AREA_MAX {
        return single_packet(&prefix, SEND_STATUS_OVERSIZE, seq);
    }
    let expected = (prefix.wire_len - WIRE_LEN_MIN) as usize;
    if area.len() != expected {
        return single_packet(&prefix, SEND_STATUS_BOUNDS, seq);
    }
    let mut build = SendBuild::new(prefix, seq, station_ids);
    let walked = walk_messages(area, usize::MAX, |message| build.observe(&message));
    let status = match walked {
        Ok(_) => SEND_STATUS_OK,
        Err(ParseError::Truncated) => SEND_STATUS_TRUNCATED,
        Err(ParseError::Oversize) => SEND_STATUS_OVERSIZE,
        Err(ParseError::Capacity) => SEND_STATUS_CAPACITY,
    };
    build.finish(status)
}

/// Rewrite the sequence fields after the packet is accepted into the ring.
pub fn stamp_send_seq(events: &mut [SendEvent], seq: u16) {
    let low = u64::from(seq as u8);
    for event in events {
        match event.code {
            SEND_CODE_PACKET => event.b = (event.b & !0xffff) | u64::from(seq),
            SEND_CODE_IDENTITY => event.b = (event.b & !(0xff << 56)) | (low << 56),
            SEND_CODE_DESTINATION => event.b = (event.b & !(0xff << 24)) | (low << 24),
            SEND_CODE_DETAIL => {
                event.a = (event.a & !(0xff << DETAIL_SEQ_SHIFT)) | (low << DETAIL_SEQ_SHIFT);
            }
            _ => {}
        }
    }
}

fn single_packet(prefix: &SendPrefix, status: u8, seq: u16) -> SendReport {
    let mut report = SendReport::empty();
    push_event(
        &mut report,
        packet_event(prefix, status, &PacketCounts::default(), seq),
    );
    report
}

#[derive(Default)]
struct PacketCounts {
    seen: u16,
    identity_n: u16,
    detail_n: u16,
    kind14: u16,
    kind58: u16,
    inherited_source: u16,
    inherited_dest: u16,
    truncated: bool,
}

fn version_accepted(flags: u8) -> bool {
    crate::peer_passive::custom_write_allowed(flags)
}

fn sat_u8(value: u16) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn packet_event(prefix: &SendPrefix, status: u8, counts: &PacketCounts, seq: u16) -> SendEvent {
    let logged_len = u64::from(u16::try_from(prefix.wire_len).unwrap_or(u16::MAX));
    let mut high = u64::from(status) << 48;
    if counts.truncated {
        high |= 1 << 56;
    }
    if prefix.magic_ok {
        high |= 1 << 57;
    }
    if version_accepted(prefix.version_flags) {
        high |= 1 << 58;
    }
    let a = u64::from(prefix.version_flags)
        | (u64::from(prefix.connection_id) << 8)
        | (u64::from(prefix.wire_byte) << 16)
        | (logged_len << 24)
        | (u64::from(sat_u8(counts.seen)) << 40)
        | high;
    let b = u64::from(seq)
        | (u64::from(sat_u8(counts.identity_n)) << 16)
        | (u64::from(sat_u8(counts.detail_n)) << 24)
        | (u64::from(sat_u8(counts.kind14)) << 32)
        | (u64::from(sat_u8(counts.kind58)) << 40)
        | (u64::from(sat_u8(counts.inherited_source)) << 48)
        | (u64::from(sat_u8(counts.inherited_dest)) << 56);
    SendEvent {
        code: SEND_CODE_PACKET,
        a,
        b,
    }
}

fn meta_word(
    flags: u8,
    kind: u8,
    logical_len: u16,
    msg_index: u8,
    connection: u8,
    mask: u8,
    seq: u16,
) -> u64 {
    u64::from(flags)
        | (u64::from(kind) << 8)
        | (u64::from(logical_len) << 16)
        | (u64::from(msg_index) << 32)
        | (u64::from(connection) << 40)
        | (u64::from(mask) << 48)
        | (u64::from(seq as u8) << 56)
}

fn dest_word(map_index: u8, msg_index: u8, connection: u8, seq: u16, mask: u8) -> u64 {
    u64::from(map_index)
        | (u64::from(msg_index) << 8)
        | (u64::from(connection) << 16)
        | (u64::from(seq as u8) << 24)
        | (u64::from(mask) << 32)
}

#[derive(Clone, Copy)]
struct HeaderRegs {
    length: u16,
    length_known: bool,
    kind: u8,
    kind_known: bool,
    destination: u64,
    destination_known: bool,
    source: u64,
    source_known: bool,
}

impl HeaderRegs {
    const fn unknown() -> Self {
        Self {
            length: 0,
            length_known: false,
            kind: 0,
            kind_known: false,
            destination: 0,
            destination_known: false,
            source: 0,
            source_known: false,
        }
    }
}

fn apply_header(regs: &mut HeaderRegs, message: &PiaMessage) -> u8 {
    let mut mask = 0u8;
    if message.flags & 2 == 0 {
        if regs.length_known {
            mask |= MASK_LENGTH_KNOWN | MASK_LENGTH_INHERITED;
        }
    } else {
        regs.length = message.body_len.unwrap_or(0);
        regs.length_known = true;
        mask |= MASK_LENGTH_KNOWN;
    }
    if message.flags & 4 == 0 {
        if regs.kind_known {
            mask |= MASK_KIND_KNOWN | MASK_KIND_INHERITED;
        }
    } else if let Some(kind) = message.kind {
        regs.kind = kind;
        regs.kind_known = true;
        mask |= MASK_KIND_KNOWN;
    }
    if message.flags & 8 == 0 {
        if regs.destination_known {
            mask |= MASK_DEST_KNOWN | MASK_DEST_INHERITED;
        }
    } else if let Some(destination) = message.first_u64 {
        regs.destination = destination;
        regs.destination_known = true;
        mask |= MASK_DEST_KNOWN;
    }
    if message.flags & 16 == 0 {
        if regs.source_known {
            mask |= MASK_SOURCE_KNOWN | MASK_SOURCE_INHERITED;
        }
    } else if let Some(source) = message.second_u64 {
        regs.source = source;
        regs.source_known = true;
        mask |= MASK_SOURCE_KNOWN;
    }
    mask
}

struct SpareSample {
    count: u8,
    nslots: u8,
    status: u8,
    all_zero: bool,
    name_zero: bool,
    sample: [u8; 8],
}

fn inspect_spare(body: &[u8]) -> SpareSample {
    let mut sample = SpareSample {
        count: body.get(0x38).copied().unwrap_or(0),
        nslots: 0,
        status: SEND_SPARE_SHORT,
        all_zero: false,
        name_zero: false,
        sample: [0; 8],
    };
    if body.len() >= 2 && body[1] != 0 {
        sample.status = SEND_SPARE_OTHER;
        return sample;
    }
    if body.len() < SPARE_HEADER + ENTRY_STRIDE {
        return sample;
    }
    let nslots = (body.len() - SPARE_HEADER) / ENTRY_STRIDE;
    sample.nslots = u8::try_from(nslots).unwrap_or(u8::MAX);
    if nslots < usize::from(sample.count) {
        return sample;
    }
    if nslots == usize::from(sample.count) {
        sample.status = SEND_SPARE_NONE;
        return sample;
    }
    let end = SPARE_HEADER + nslots * ENTRY_STRIDE;
    let Some(entry) = body.get(end - ENTRY_STRIDE..end) else {
        return sample;
    };
    sample.status = SEND_SPARE_PRESENT;
    sample.all_zero = entry.iter().all(|byte| *byte == 0);
    if let Some(name) = entry.get(ENTRY_NAME..ENTRY_NAME + ENTRY_NAME_LEN) {
        sample.name_zero = name.iter().all(|byte| *byte == 0);
        sample.sample[..4].copy_from_slice(&name[..4]);
        sample.sample[4..].copy_from_slice(&name[0x1c..0x20]);
    }
    sample
}

fn spare_event(body: &[u8], msg_index: u8, connection: u8, seq: u16) -> SendEvent {
    let sample = inspect_spare(body);
    let mut a = u64::from(SEND_DETAIL_SPARE)
        | (u64::from(sample.count) << 8)
        | (u64::from(sample.nslots) << 16)
        | (u64::from(sample.status) << 24)
        | (u64::from(msg_index) << 34)
        | (u64::from(connection) << 42)
        | (u64::from(seq as u8) << DETAIL_SEQ_SHIFT);
    if sample.all_zero {
        a |= 1 << 32;
    }
    if sample.name_zero {
        a |= 1 << 33;
    }
    SendEvent {
        code: SEND_CODE_DETAIL,
        a,
        b: u64::from_le_bytes(sample.sample),
    }
}

fn rtt_event(body: &[u8], msg_index: u8, connection: u8, seq: u16) -> SendEvent {
    let present = u8::try_from(body.len().min(8)).unwrap_or(8);
    let type_valid = body.len() >= 4;
    let data_valid = body.len() >= 8;
    let type_word = if type_valid {
        u32::from_be_bytes([body[0], body[1], body[2], body[3]])
    } else {
        0
    };
    let data_word = if data_valid {
        u32::from_be_bytes([body[4], body[5], body[6], body[7]])
    } else {
        0
    };
    let mut a = u64::from(SEND_DETAIL_RTT)
        | (u64::from(present) << 8)
        | (u64::from(msg_index) << 18)
        | (u64::from(connection) << 26)
        | (u64::from(seq as u8) << DETAIL_SEQ_SHIFT);
    if type_valid {
        a |= 1 << 16;
    }
    if data_valid {
        a |= 1 << 17;
    }
    SendEvent {
        code: SEND_CODE_DETAIL,
        a,
        b: u64::from(data_word) | (u64::from(type_word) << 32),
    }
}

#[derive(Clone, Copy)]
struct Logged {
    identity: SendEvent,
    destination: Option<SendEvent>,
}

struct SendBuild<'a> {
    prefix: SendPrefix,
    seq: u16,
    station_ids: &'a [u64],
    regs: HeaderRegs,
    seen: u16,
    inherited_source: u16,
    inherited_dest: u16,
    kind58: u16,
    kind14_sub2: u16,
    early: [Option<Logged>; SEND_IDENTITY_EARLY],
    early_n: usize,
    late_spare: Option<Logged>,
    late_rtt: Option<Logged>,
    spare_details: [SendEvent; SEND_DETAIL_EACH],
    spare_n: usize,
    rtt_details: [SendEvent; SEND_DETAIL_EACH],
    rtt_n: usize,
}

impl<'a> SendBuild<'a> {
    fn new(prefix: SendPrefix, seq: u16, station_ids: &'a [u64]) -> Self {
        Self {
            prefix,
            seq,
            station_ids,
            regs: HeaderRegs::unknown(),
            seen: 0,
            inherited_source: 0,
            inherited_dest: 0,
            kind58: 0,
            kind14_sub2: 0,
            early: [None; SEND_IDENTITY_EARLY],
            early_n: 0,
            late_spare: None,
            late_rtt: None,
            spare_details: [SendEvent {
                code: 0,
                a: 0,
                b: 0,
            }; SEND_DETAIL_EACH],
            spare_n: 0,
            rtt_details: [SendEvent {
                code: 0,
                a: 0,
                b: 0,
            }; SEND_DETAIL_EACH],
            rtt_n: 0,
        }
    }

    fn observe(&mut self, message: &PiaMessage) {
        let msg_index = u8::try_from(self.seen).unwrap_or(u8::MAX);
        self.seen = self.seen.saturating_add(1);
        let mask = apply_header(&mut self.regs, message);
        if mask & MASK_SOURCE_INHERITED != 0 {
            self.inherited_source = self.inherited_source.saturating_add(1);
        }
        if mask & MASK_DEST_INHERITED != 0 {
            self.inherited_dest = self.inherited_dest.saturating_add(1);
        }
        let kind = if self.regs.kind_known {
            self.regs.kind
        } else {
            0xff
        };
        let logical_len = if self.regs.length_known {
            self.regs.length
        } else {
            0
        };
        let identity = SendEvent {
            code: SEND_CODE_IDENTITY,
            a: if self.regs.source_known {
                mix_id(self.regs.source)
            } else {
                0
            },
            b: meta_word(
                message.flags,
                kind,
                logical_len,
                msg_index,
                self.prefix.connection_id,
                mask,
                self.seq,
            ),
        };
        let destination = if self.regs.destination_known {
            Some(SendEvent {
                code: SEND_CODE_DESTINATION,
                a: mix_id(self.regs.destination),
                b: dest_word(
                    find_source(self.station_ids, self.regs.destination).unwrap_or(0xff),
                    msg_index,
                    self.prefix.connection_id,
                    self.seq,
                    mask,
                ),
            })
        } else {
            None
        };
        let logged = Logged {
            identity,
            destination,
        };
        let subtype_two = message.body.first() == Some(&2);
        let is_rtt = self.regs.kind_known && self.regs.kind == 0x58;
        let is_spare = self.regs.kind_known && self.regs.kind == 0x14 && subtype_two;
        if is_rtt {
            self.kind58 = self.kind58.saturating_add(1);
            if self.rtt_n < SEND_DETAIL_EACH {
                self.rtt_details[self.rtt_n] =
                    rtt_event(message.body, msg_index, self.prefix.connection_id, self.seq);
                self.rtt_n += 1;
            }
        }
        if is_spare {
            self.kind14_sub2 = self.kind14_sub2.saturating_add(1);
            if self.spare_n < SEND_DETAIL_EACH {
                self.spare_details[self.spare_n] =
                    spare_event(message.body, msg_index, self.prefix.connection_id, self.seq);
                self.spare_n += 1;
            }
        }
        if self.early_n < SEND_IDENTITY_EARLY {
            self.early[self.early_n] = Some(logged);
            self.early_n += 1;
        } else if is_spare && self.late_spare.is_none() {
            self.late_spare = Some(logged);
        } else if is_rtt && self.late_rtt.is_none() {
            self.late_rtt = Some(logged);
        }
    }

    fn finish(self, status: u8) -> SendReport {
        let mut identity_n = u16::try_from(self.early_n).unwrap_or(u16::MAX);
        if self.late_spare.is_some() {
            identity_n = identity_n.saturating_add(1);
        }
        if self.late_rtt.is_some() {
            identity_n = identity_n.saturating_add(1);
        }
        let detail_n = u16::try_from(self.spare_n + self.rtt_n).unwrap_or(u16::MAX);
        let counts = PacketCounts {
            seen: self.seen,
            identity_n,
            detail_n,
            kind14: self.kind14_sub2,
            kind58: self.kind58,
            inherited_source: self.inherited_source,
            inherited_dest: self.inherited_dest,
            truncated: self.seen > identity_n,
        };
        let mut report = SendReport::empty();
        push_event(
            &mut report,
            packet_event(&self.prefix, status, &counts, self.seq),
        );
        for slot in self.early.into_iter().take(self.early_n).flatten() {
            push_logged(&mut report, slot);
        }
        if let Some(slot) = self.late_spare {
            push_logged(&mut report, slot);
        }
        if let Some(slot) = self.late_rtt {
            push_logged(&mut report, slot);
        }
        for event in self.spare_details.into_iter().take(self.spare_n) {
            push_event(&mut report, event);
        }
        for event in self.rtt_details.into_iter().take(self.rtt_n) {
            push_event(&mut report, event);
        }
        report
    }
}

fn push_event(report: &mut SendReport, event: SendEvent) {
    if report.len < SEND_EVENT_CAP {
        report.events[report.len] = event;
        report.len += 1;
    }
}

fn push_logged(report: &mut SendReport, logged: Logged) {
    push_event(report, logged.identity);
    if let Some(destination) = logged.destination {
        push_event(report, destination);
    }
}

#[cfg(test)]
mod tests {
    use super::{MASK_KIND_INHERITED, MASK_LENGTH_INHERITED, *};
    use crate::peer_passive::{
        parse_message_area, ObservedInterface, ParseError, PiaMessage, MESSAGE_AREA_MAX,
    };

    #[test]
    fn guards_are_the_nso_bytes_at_the_two_sites() {
        assert_eq!(PARSED_MESSAGE_OFFSET, 0x153844);
        assert_eq!(STATION_INDEX_OFFSET, 0x15f0b8);
        assert_eq!(STATION_INDEX_FIELD, 0x38);
        assert_eq!(MESSAGE_SOURCE_OFFSET, 0x40);
        assert_eq!(PARSED_MESSAGE_GUARD.len(), 64);
        assert_eq!(STATION_INDEX_GUARD.len(), 64);
        assert_eq!(&PARSED_MESSAGE_GUARD[..4], &[0xa8, 0x02, 0x40, 0xf9]);
        assert_eq!(
            &STATION_INDEX_GUARD[..8],
            &[0x14, 0xe0, 0x40, 0x39, 0x9f, 0xf6, 0x03, 0x71]
        );
    }

    #[test]
    fn station_probe_flags_separate_a_missed_read_from_a_zero_id() {
        assert_eq!(crate::diagnostics::Code::StationProbe as u16, 64);
        assert_eq!(crate::diagnostics::Code::StationProbeId as u16, 65);
        assert_ne!(46, crate::diagnostics::Code::StationProbe as u16);
        assert_ne!(47, crate::diagnostics::Code::StationProbe as u16);
        let (missed, missed_mix) = station_probe_flags(0, 0, None, None, 0);
        assert_eq!(missed_mix, 0);
        assert_eq!(
            missed,
            STATION_PROBE_IGNORED | (0xff << STATION_PROBE_INDEX_SHIFT)
        );
        let (zero, zero_mix) = station_probe_flags(0x10000, 0x20000, Some(0), Some(1), 0);
        assert_eq!(zero_mix, mix_id(0));
        assert_ne!(zero_mix, 0);
        assert_eq!(
            zero,
            STATION_PROBE_MESSAGE_PTR
                | STATION_PROBE_STATION_PTR
                | STATION_PROBE_SOURCE_READ
                | STATION_PROBE_INDEX_READ
                | STATION_PROBE_SOURCE_ZERO
                | STATION_PROBE_UNCHANGED
                | (1 << STATION_PROBE_INDEX_SHIFT)
        );
        let (updated, updated_mix) = station_probe_flags(0x10000, 0x20000, Some(9), Some(4), 0);
        assert_eq!(updated_mix, mix_id(9));
        assert_eq!(updated & STATION_PROBE_UPDATED, STATION_PROBE_UPDATED);
        assert_eq!(updated & STATION_PROBE_UNCHANGED, 0);
        assert_eq!(updated & STATION_PROBE_SOURCE_ZERO, 0);
        assert_eq!((updated >> STATION_PROBE_INDEX_SHIFT) & 0xff, 4);
        let (high, _) = station_probe_flags(0x10000, 0x20000, Some(9), Some(253), 0);
        assert_eq!(
            high & (STATION_PROBE_INDEX_HIGH | STATION_PROBE_IGNORED | STATION_PROBE_UPDATED),
            STATION_PROBE_INDEX_HIGH | STATION_PROBE_IGNORED
        );
        let (same, _) = station_probe_flags(0x10000, 0x20000, Some(9), Some(2), 9);
        assert_eq!(same & STATION_PROBE_UNCHANGED, STATION_PROBE_UNCHANGED);
        assert_eq!(same & STATION_PROBE_UPDATED, 0);
        assert_eq!(same & STATION_PROBE_SOURCE_ZERO, 0);
    }

    #[test]
    fn index_map_rejects_sentinels_and_bumps_only_on_change() {
        assert_eq!(classify_index(0, 252, 9), IndexChange::Updated);
        assert_eq!(classify_index(9, 252, 9), IndexChange::Unchanged);
        assert_eq!(classify_index(9, 0xfd, 4), IndexChange::Ignored);
        assert_eq!(classify_index(9, 253, 4), IndexChange::Ignored);
        let mut stored = [0u64; STATION_SLOTS];
        stored[0] = 9;
        stored[252] = 4;
        assert_eq!(find_source(&stored, 4), Some(252));
        assert_eq!(find_source(&stored, 0), None);
        assert_eq!(find_source(&stored, 7), None);
        assert!(clears_station(3) && clears_station(4));
        assert!(!clears_station(2));
    }

    #[test]
    fn mixed_id_is_stable_and_not_the_pia_id() {
        assert_eq!(mix_id(0x1234_5678_9abc_def0), mix_id(0x1234_5678_9abc_def0));
        assert_ne!(mix_id(0x1234_5678_9abc_def0), 0x1234_5678_9abc_def0);
        assert_ne!(mix_id(1), mix_id(2));
    }

    #[test]
    fn omitted_header_fields_stay_absent_in_the_raw_parser() {
        let previous = GameHeader {
            flags: 0x1f,
            length: 8,
            kind_word: 0x1400_0000,
            destination_id: 2,
            source_id: 0x1111,
        };
        let current = GameHeader {
            flags: 0x04,
            length: 0,
            kind_word: 0x5800_0000,
            destination_id: 0,
            source_id: 0,
        };
        let inherited = inherit_header(previous, current);
        assert_eq!(inherited.source_id, 0x1111);
        assert_eq!(inherited.length, 8);
        assert_eq!(kind_of(inherited.kind_word), 0x58);
        assert_eq!(header_span(0), 1);
        assert_eq!(header_span(0x01), 2);
        assert_eq!(header_span(0x03), 4);
        assert_eq!(header_span(0x07), 8);
        assert_eq!(header_span(0x0f), 16);
        assert_eq!(header_span(0x1f), 24);
        let mut out = [crate::peer_passive::PiaMessage {
            flags: 0,
            extra: None,
            body_len: None,
            kind: None,
            kind_arg: None,
            first_u64: None,
            second_u64: None,
            body: &[],
        }; 1];
        let n = parse_message_area(&[0x04, 0x58, 0, 0, 0, 0, 0, 0], &mut out).unwrap();
        assert_eq!(n, 1);
        assert_eq!(out[0].kind, Some(0x58));
        assert_eq!(out[0].second_u64, None);
        assert_eq!(out[0].first_u64, None);
    }

    #[test]
    fn subtype_two_name_and_interface_are_bounded() {
        let mut entry = vec![0u8; ENTRY_STRIDE];
        entry[ENTRY_TAG] = 1;
        entry[ENTRY_NAME] = b'A';
        entry[ENTRY_NAME + 1] = 1;
        entry[ENTRY_NAME + 2] = b'Z';
        let sample = decode_entry_name(&entry).unwrap();
        assert_eq!(&sample.bytes[..sample.len as usize], b"A\\x01Z");

        let mut wide = vec![0u8; ENTRY_STRIDE];
        wide[ENTRY_TAG] = 2;
        wide[ENTRY_NAME] = b'B';
        let wide_sample = decode_entry_name(&wide).unwrap();
        assert_eq!(&wide_sample.bytes[..wide_sample.len as usize], b"B");
        wide[ENTRY_TAG] = 9;
        assert!(decode_entry_name(&wide).is_none());

        let mut body = vec![0u8; 0x39 + 2 * ENTRY_STRIDE];
        body[0] = 2;
        body[0x38] = 1;
        body[0x39 + ENTRY_TAG] = 1;
        body[0x39 + ENTRY_NAME] = b'Q';
        let end = body.len();
        body[end - 0x54] = 0x45;
        body[end - 0x53] = 1;
        let seen = diagnose(0x55, 0x1400_0000, &body, Some((4, 3)));
        assert_eq!(seen.kind, 0x14);
        assert_eq!(seen.subtype, 2);
        assert_eq!(seen.index, 4);
        assert_eq!(seen.generation, 3);
        assert_eq!(&seen.name[..seen.name_len as usize], b"Q");
        assert!(seen.interface_known);
        assert_eq!(seen.interface, ObservedInterface::Wifi as u8);
        assert_ne!(seen.source_hash, 0x55);
        assert!(!seen.disconnect);
        assert!(!seen.map_update);
        body[end - 0x53] = 3;
        let emulated = diagnose(0x55, 0x1400_0000, &body, Some((4, 3)));
        assert!(emulated.interface_known);
        assert_eq!(emulated.interface, ObservedInterface::Unspecified as u8);
        let mut leaving = body.clone();
        leaving[0] = 3;
        let cleared = diagnose(0x55, 0x1400_0000, &leaving, Some((4, 8)));
        assert_eq!(cleared.subtype, 3);
        assert!(!cleared.disconnect);
        assert!(clears_station(3) && clears_station(4));
        assert!(!clears_station(2));
        assert_eq!(cleared.name_len, 0);
        assert!(decode_entry_name(&[0u8; 8]).is_none());
        assert_eq!(diagnose(0x55, 0x1400_0000, &[2, 0, 1], None).name_len, 0);
        let packed = u64::from(seen.index)
            | (u64::from(seen.kind) << 8)
            | (u64::from(seen.subtype) << 16)
            | (1u64 << 56);
        let (fresh, map) = present_observation(packed, seen.generation, Some(seen.generation));
        assert!(map);
        assert_eq!(fresh >> 48, seen.generation);
        assert_eq!(fresh & (1 << 43), 0);
        assert_eq!(fresh & (1 << 56), 0);
        let (stale, _) = present_observation(packed, seen.generation, Some(seen.generation + 1));
        assert_ne!(stale & (1 << 43), 0);
        let (unmapped, unmapped_map) = present_observation(0xff, 4, Some(9));
        assert!(!unmapped_map);
        assert_eq!(unmapped & (1 << 43), 0);

        let mut long = [b'A'; 40];
        long[39] = 0;
        let mut long_entry = vec![0u8; ENTRY_STRIDE];
        long_entry[ENTRY_TAG] = 1;
        long_entry[ENTRY_NAME..ENTRY_NAME + 40].copy_from_slice(&long);
        let truncated = decode_entry_name(&long_entry).unwrap();
        assert!(truncated.truncated);
        assert_eq!(truncated.len, NAME_CAP as u8);
    }

    #[test]
    fn name_samples_escape_truncation_and_do_not_decode_tag_one_as_text() {
        let mut entry = vec![0u8; ENTRY_STRIDE];
        entry[ENTRY_TAG] = 1;
        entry[ENTRY_NAME..ENTRY_NAME + 16].fill(b'A');
        let exact = decode_entry_name(&entry).unwrap();
        assert_eq!(exact.len, 16);
        assert!(!exact.truncated);
        entry[ENTRY_NAME + 16] = b'B';
        let over = decode_entry_name(&entry).unwrap();
        assert!(over.truncated);
        assert_eq!(&over.bytes, b"AAAAAAAAAAAAAAAA");

        entry[ENTRY_NAME] = b'\\';
        entry[ENTRY_NAME + 1] = 0;
        assert_eq!(&decode_entry_name(&entry).unwrap().bytes[..4], b"\\x5c");

        entry[ENTRY_NAME] = 0xc3;
        entry[ENTRY_NAME + 1] = 0xa9;
        entry[ENTRY_NAME + 2] = 0;
        let raw = decode_entry_name(&entry).unwrap();
        assert_eq!(&raw.bytes[..raw.len as usize], b"\\xc3\\xa9");

        let mut wide = vec![0u8; ENTRY_STRIDE];
        wide[ENTRY_TAG] = 2;
        wide[ENTRY_NAME] = 0xe9;
        wide[ENTRY_NAME + 1] = 0x00;
        let encoded = decode_entry_name(&wide).unwrap();
        assert_eq!(&encoded.bytes[..encoded.len as usize], b"\\xc3\\xa9");
        wide[ENTRY_NAME] = 0x00;
        wide[ENTRY_NAME + 1] = 0xd8;
        let surrogate = decode_entry_name(&wide).unwrap();
        assert_eq!(
            &surrogate.bytes[..surrogate.len as usize],
            b"\\xef\\xbf\\xbd"
        );
    }

    #[test]
    fn message_body_range_rejects_truncation_overflow_and_keeps_zero_length() {
        let packet = 0x1_0000u64;
        let message = packet + 0x48;
        assert_eq!(
            message_body_range(packet, 0x80, message, 0, 4),
            Some(MessageBody::Bytes {
                at: message + 1,
                len: 4
            })
        );
        assert_eq!(
            message_body_range(packet, 0x80, message, 0, 0),
            Some(MessageBody::Empty)
        );
        let area_end = packet + 0x28 + 0x80;
        let exact = area_end - message - 1;
        assert_eq!(
            message_body_range(packet, 0x80, message, 0, exact as u16),
            Some(MessageBody::Bytes {
                at: message + 1,
                len: exact as usize
            })
        );
        assert!(message_body_range(packet, 0x80, message, 0, exact as u16 + 1).is_none());
        assert!(message_body_range(packet, 0x1f, message, 0, 0).is_none());
        assert!(message_body_range(packet, 0x5c1, message, 0, 0).is_none());
        assert!(message_body_range(packet, 0x100, packet + 0x47, 0, 0).is_none());
        assert!(message_body_range(packet, 0x100, packet + 0x28 + 0x100, 0, 0).is_none());
        assert!(message_body_range(u64::MAX, 0x40, message, 0, 0).is_none());
        assert!(message_body_range(u64::MAX - 0x27, 0x40, message, 0, 1).is_none());
        let wire = 0x30u32;
        let last = packet + 0x28 + u64::from(wire) - 1;
        assert!(message_body_range(packet, wire, last, 0x1f, 0).is_none());
    }

    fn bits(word: u64, shift: u32, width: u32) -> u64 {
        (word >> shift) & ((1u64 << width) - 1)
    }

    fn hosted(bytes: &[u8]) -> (SendPrefix, Vec<u8>) {
        (
            SendPrefix {
                version_flags: 4,
                connection_id: 0x11,
                wire_byte: 0x5a,
                wire_len: 0x20 + u32::try_from(bytes.len()).unwrap(),
                magic_ok: true,
            },
            bytes.to_vec(),
        )
    }

    fn append_frame(
        out: &mut Vec<u8>,
        flags: u8,
        kind: Option<u8>,
        dest: Option<u64>,
        source: Option<u64>,
        body: &[u8],
    ) {
        let start = out.len();
        out.push(flags);
        if flags & 2 != 0 {
            out.extend(u16::try_from(body.len()).unwrap().to_be_bytes());
        }
        if flags & 4 != 0 {
            out.push(kind.unwrap());
            out.extend_from_slice(&[0, 0, 0]);
        }
        if flags & 8 != 0 {
            out.extend(dest.unwrap().to_be_bytes());
        }
        if flags & 16 != 0 {
            out.extend(source.unwrap().to_be_bytes());
        }
        out.extend_from_slice(body);
        while !(out.len() - start).is_multiple_of(4) {
            out.push(0);
        }
    }

    fn report_for(bytes: &[u8], stations: &[u64]) -> SendReport {
        let (prefix, area) = hosted(bytes);
        diagnose_send(prefix, &area, 0x0107, stations)
    }

    fn response(count: u8, slots: usize) -> Vec<u8> {
        let mut body = vec![0u8; SPARE_HEADER + slots * ENTRY_STRIDE];
        body[0] = 2;
        body[0x38] = count;
        body
    }

    #[test]
    fn send_codes_and_guard_match_the_encrypt_site() {
        assert_eq!(SEND_PLAINTEXT_OFFSET, 0x155878);
        // Raw NSO bytes from the 13.0.5 encrypt site. This fixture is not derived
        // from `SEND_PLAINTEXT_GUARD`. The first instruction is `add x1, sp, #0x28`.
        let raw64: [u8; 64] = [
            0xe1, 0xa3, 0x00, 0x91, 0xf5, 0xa6, 0xfd, 0x97, 0x60, 0x1a, 0x40, 0xf9, 0xe1, 0x03,
            0x14, 0x2a, 0xb2, 0xfd, 0xff, 0x97, 0x94, 0x06, 0x00, 0x11, 0x60, 0xf9, 0xff, 0xb5,
            0x60, 0x1a, 0x40, 0xf9, 0xbe, 0xfd, 0xff, 0x97, 0x7f, 0x72, 0x0c, 0x79, 0x7f, 0xe2,
            0x00, 0x39, 0x7f, 0x22, 0x00, 0xf9, 0x7f, 0xa2, 0x01, 0x39, 0x7f, 0x3a, 0x00, 0xf9,
            0x7f, 0x62, 0x02, 0x39, 0x7f, 0x52, 0x00, 0xf9,
        ];
        assert_eq!(SEND_PLAINTEXT_GUARD, raw64);
        let mut first = [0u8; 4];
        first.copy_from_slice(&SEND_PLAINTEXT_GUARD[..4]);
        assert_eq!(u32::from_le_bytes(first), 0x9100a3e1);
        assert_eq!(
            SEND_CODE_PACKET,
            crate::diagnostics::Code::SendPacket as u16
        );
        assert_eq!(
            SEND_CODE_IDENTITY,
            crate::diagnostics::Code::SendIdentity as u16
        );
        assert_eq!(
            SEND_CODE_DESTINATION,
            crate::diagnostics::Code::SendDestination as u16
        );
        assert_eq!(
            SEND_CODE_DETAIL,
            crate::diagnostics::Code::SendDetail as u16
        );
        assert_eq!(SEND_CODE_DROP, crate::diagnostics::Code::SendDrop as u16);
        assert_ne!(mix_id(0), 0);
    }

    #[test]
    fn send_parser_rejects_bad_lengths_before_it_reads_messages() {
        let early = SendPrefix {
            version_flags: 4,
            connection_id: 0x11,
            wire_byte: 0x5a,
            wire_len: 0x1f,
            magic_ok: true,
        };
        let rejected = diagnose_send(early, &[0x04, 0x58, 0, 0, 0, 0, 0, 0], 3, &[]);
        assert_eq!(rejected.len, 1);
        assert_eq!(
            bits(rejected.events[0].a, 48, 8),
            u64::from(SEND_STATUS_LENGTH)
        );
        assert_eq!(bits(rejected.events[0].a, 8, 8), 0x11);
        assert_eq!(bits(rejected.events[0].a, 16, 8), 0x5a);

        let mut huge = vec![0xff; MESSAGE_AREA_MAX + 1];
        huge[0] = 0x04;
        let over = diagnose_send(
            SendPrefix {
                version_flags: 4,
                connection_id: 1,
                wire_byte: 2,
                wire_len: 0x5c0,
                magic_ok: false,
            },
            &huge,
            1,
            &[],
        );
        assert_eq!(over.len, 1);
        assert_eq!(
            bits(over.events[0].a, 48, 8),
            u64::from(SEND_STATUS_OVERSIZE)
        );

        let bounds = diagnose_send(
            SendPrefix {
                version_flags: 4,
                connection_id: 1,
                wire_byte: 2,
                wire_len: 0x40,
                magic_ok: true,
            },
            &[0, 0, 0, 0],
            1,
            &[],
        );
        assert_eq!(
            bits(bounds.events[0].a, 48, 8),
            u64::from(SEND_STATUS_BOUNDS)
        );

        let mut out = [PiaMessage {
            flags: 0,
            extra: None,
            body_len: None,
            kind: None,
            kind_arg: None,
            first_u64: None,
            second_u64: None,
            body: &[],
        }; 1];
        assert_eq!(
            parse_message_area(&[0, 0, 0, 0, 0x02], &mut out),
            Err(ParseError::Capacity)
        );
    }

    #[test]
    fn inherited_ids_are_mixed_and_an_omitted_length_keeps_the_body() {
        let source = 0x1111_2222_3333_4444u64;
        let dest = 0x5555_6666_7777_8888u64;
        let replaced = 0x9999_aaaa_bbbb_ccccu64;
        let mut area = Vec::new();
        append_frame(&mut area, 0x18, None, Some(dest), Some(source), &[]);
        append_frame(
            &mut area,
            0x06,
            Some(0x58),
            None,
            None,
            &[0, 0, 0, 1, 0x10, 0x20, 0x30, 0x40],
        );
        append_frame(&mut area, 0x04, Some(0x14), None, None, &[0; 8]);
        append_frame(&mut area, 0x08, None, Some(replaced), None, &[0; 8]);
        let mut stations = [0u64; 12];
        stations[9] = replaced;
        let report = report_for(&area, &stations);
        let packet = report.events[0];
        assert_eq!(packet.code, SEND_CODE_PACKET);
        assert_eq!(bits(packet.a, 48, 8), u64::from(SEND_STATUS_OK));
        assert_eq!(bits(packet.a, 56, 1), 0);
        assert_eq!(bits(packet.a, 57, 1), 1);
        assert_eq!(bits(packet.a, 58, 1), 1);
        assert_eq!(bits(packet.b, 0, 16), 0x0107);
        assert_eq!(bits(packet.b, 48, 8), 3);
        assert_eq!(bits(packet.b, 56, 8), 2);

        let identities: Vec<_> = report.events[..report.len]
            .iter()
            .filter(|event| event.code == SEND_CODE_IDENTITY)
            .collect();
        assert_eq!(identities.len(), 4);
        assert_eq!(identities[0].a, mix_id(source));
        assert_eq!(bits(identities[0].b, 8, 8), 0xff);
        assert_eq!(bits(identities[0].b, 48, 8) & 0b1111, 0b0101);
        assert_eq!(identities[1].a, mix_id(source));
        assert_eq!(bits(identities[1].b, 8, 8), 0x58);
        assert_eq!(bits(identities[1].b, 16, 16), 8);
        assert_eq!(bits(identities[1].b, 48, 8) & 0b1010_1010, 0b0000_1010);
        assert_eq!(bits(identities[2].b, 8, 8), 0x14);
        assert_eq!(bits(identities[2].b, 48, 8) & MASK_KIND_INHERITED as u64, 0);
        assert_eq!(identities[3].a, mix_id(source));
        assert_eq!(bits(identities[3].b, 8, 8), 0x14);
        assert_ne!(
            bits(identities[3].b, 48, 8) & u64::from(MASK_KIND_INHERITED),
            0
        );

        let destinations: Vec<_> = report.events[..report.len]
            .iter()
            .filter(|event| event.code == SEND_CODE_DESTINATION)
            .collect();
        assert_eq!(destinations.len(), 4);
        assert_eq!(destinations[0].a, mix_id(dest));
        assert_eq!(bits(destinations[0].b, 0, 8), 0xff);
        assert_eq!(destinations[3].a, mix_id(replaced));
        assert_eq!(bits(destinations[3].b, 0, 8), 9);
        assert_eq!(bits(destinations[3].b, 16, 8), 0x11);

        let rtt = report.events[..report.len]
            .iter()
            .find(|event| event.code == SEND_CODE_DETAIL && bits(event.a, 0, 8) == 2)
            .unwrap();
        assert_ne!(rtt.a & (1 << 16), 0);
        assert_ne!(rtt.a & (1 << 17), 0);
        assert_eq!(rtt.b >> 32, 1);
        assert_eq!(rtt.b as u32, 0x1020_3040);
        for event in &report.events[..report.len] {
            assert_ne!(event.a, source);
            assert_ne!(event.b, source);
            assert_ne!(event.a, dest);
            assert_ne!(event.b, dest);
            assert_ne!(event.a, replaced);
            assert_ne!(event.b, replaced);
        }

        let mut bare = Vec::new();
        append_frame(
            &mut bare,
            0x06,
            Some(0x58),
            None,
            None,
            &[0, 0, 0, 1, 1, 2, 3, 4],
        );
        append_frame(
            &mut bare,
            0x04,
            Some(0x58),
            None,
            None,
            &[0, 0, 0, 2, 5, 6, 7, 8],
        );
        let inherited = report_for(&bare, &[]);
        let details: Vec<_> = inherited.events[..inherited.len]
            .iter()
            .filter(|event| event.code == SEND_CODE_DETAIL)
            .collect();
        assert_eq!(details.len(), 2);
        assert_eq!(bits(details[1].a, 8, 8), 8);
        assert_ne!(details[1].a & (1 << 16), 0);
        assert_ne!(details[1].a & (1 << 17), 0);
        assert_eq!(details[1].b >> 32, 2);
        assert_eq!(details[1].b as u32, 0x0506_0708);
        let second = inherited.events[..inherited.len]
            .iter()
            .filter(|event| event.code == SEND_CODE_IDENTITY)
            .nth(1)
            .unwrap();
        assert_eq!(bits(second.b, 16, 16), 8);
        assert_ne!(bits(second.b, 48, 8) & u64::from(MASK_LENGTH_INHERITED), 0);
    }

    #[test]
    fn spare_capacity_reports_zero_fill_name_window_and_keeps_late_messages() {
        let mut zero = response(1, 2);
        let end = zero.len();
        assert_eq!(zero[end - 0x54], 0);
        assert_eq!(zero[end - 0x53], 0);
        let mut framed = Vec::new();
        append_frame(&mut framed, 0x06, Some(0x14), None, None, &zero);
        let report = report_for(&framed, &[]);
        let spare = report.events[..report.len]
            .iter()
            .find(|event| event.code == SEND_CODE_DETAIL)
            .unwrap();
        assert_eq!(bits(spare.a, 0, 8), u64::from(SEND_DETAIL_SPARE));
        assert_eq!(bits(spare.a, 8, 8), 1);
        assert_eq!(bits(spare.a, 16, 8), 2);
        assert_eq!(bits(spare.a, 24, 8), u64::from(SEND_SPARE_PRESENT));
        assert_ne!(spare.a & (1 << 32), 0);
        assert_ne!(spare.a & (1 << 33), 0);
        assert_eq!(spare.b, 0);

        zero[end - ENTRY_STRIDE] = 0x7e;
        let mut outside = Vec::new();
        append_frame(&mut outside, 0x06, Some(0x14), None, None, &zero);
        let outside_report = report_for(&outside, &[]);
        let outside_spare = outside_report.events[..outside_report.len]
            .iter()
            .find(|event| event.code == SEND_CODE_DETAIL)
            .unwrap();
        assert_eq!(outside_spare.a & (1 << 32), 0);
        assert_ne!(outside_spare.a & (1 << 33), 0);

        let mut named = response(1, 2);
        let named_end = named.len();
        let name_at = named_end - ENTRY_STRIDE + ENTRY_NAME;
        named[name_at] = b'Q';
        named[named_end - 0x54] = 0x45;
        named[named_end - 0x53] = 0x02;
        named.push(0xaa);
        let mut named_frame = Vec::new();
        append_frame(&mut named_frame, 0x06, Some(0x14), None, None, &named);
        let named_report = report_for(&named_frame, &[]);
        let named_spare = named_report.events[..named_report.len]
            .iter()
            .find(|event| event.code == SEND_CODE_DETAIL)
            .unwrap();
        assert_eq!(named_spare.a & (1 << 32), 0);
        assert_eq!(named_spare.a & (1 << 33), 0);
        let sample = named_spare.b.to_le_bytes();
        assert_eq!(sample[0], b'Q');
        assert_eq!(sample[6], 0x45);
        assert_eq!(sample[7], 0x02);

        let mut none = Vec::new();
        append_frame(&mut none, 0x06, Some(0x14), None, None, &response(1, 1));
        let none_report = report_for(&none, &[]);
        let none_spare = none_report.events[..none_report.len]
            .iter()
            .find(|event| event.code == SEND_CODE_DETAIL)
            .unwrap();
        assert_eq!(bits(none_spare.a, 24, 8), u64::from(SEND_SPARE_NONE));

        let mut short = Vec::new();
        append_frame(&mut short, 0x06, Some(0x14), None, None, &response(2, 1));
        let short_report = report_for(&short, &[]);
        let short_spare = short_report.events[..short_report.len]
            .iter()
            .find(|event| event.code == SEND_CODE_DETAIL)
            .unwrap();
        assert_eq!(bits(short_spare.a, 24, 8), u64::from(SEND_SPARE_SHORT));
        let mut other = Vec::new();
        append_frame(&mut other, 0x06, Some(0x14), None, None, &[2, 9]);
        let other_report = report_for(&other, &[]);
        let other_spare = other_report.events[..other_report.len]
            .iter()
            .find(|event| event.code == SEND_CODE_DETAIL)
            .unwrap();
        assert_eq!(bits(other_spare.a, 24, 8), u64::from(SEND_SPARE_OTHER));

        let mut crowded = Vec::new();
        for _ in 0..4 {
            append_frame(&mut crowded, 0, None, None, None, &[]);
        }
        append_frame(&mut crowded, 0, None, None, None, &[]);
        append_frame(&mut crowded, 0x06, Some(0x14), None, None, &response(1, 2));
        append_frame(
            &mut crowded,
            0x1e,
            Some(0x58),
            Some(0x42),
            Some(0x43),
            &[0, 0, 0, 9, 0, 0, 0, 0],
        );
        let crowded_report = report_for(&crowded, &[]);
        assert_ne!(crowded_report.events[0].a & (1 << 56), 0);
        let indexes: Vec<_> = crowded_report.events[..crowded_report.len]
            .iter()
            .filter(|event| event.code == SEND_CODE_IDENTITY)
            .map(|event| bits(event.b, 32, 8))
            .collect();
        assert_eq!(indexes, vec![0, 1, 2, 3, 5, 6]);
        assert!(crowded_report.events[..crowded_report.len]
            .iter()
            .any(|event| {
                event.code == SEND_CODE_DETAIL
                    && bits(event.a, 0, 8) == u64::from(SEND_DETAIL_SPARE)
            }));
        let late_rtt = crowded_report.events[..crowded_report.len]
            .iter()
            .find(|event| {
                event.code == SEND_CODE_DETAIL && bits(event.a, 0, 8) == u64::from(SEND_DETAIL_RTT)
            })
            .unwrap();
        assert_ne!(late_rtt.a & (1 << 17), 0);
        assert_eq!(late_rtt.b as u32, 0);
        assert_eq!(late_rtt.b >> 32, 9);
        assert_eq!(bits(crowded_report.events[0].b, 32, 8), 1);
        assert_eq!(bits(crowded_report.events[0].b, 40, 8), 1);

        let mut events = crowded_report.events;
        stamp_send_seq(&mut events[..crowded_report.len], 0x22cd);
        assert_eq!(bits(events[0].b, 0, 16), 0x22cd);
        let identity = events[..crowded_report.len]
            .iter()
            .find(|event| event.code == SEND_CODE_IDENTITY)
            .unwrap();
        assert_eq!(bits(identity.b, 56, 8), 0xcd);
        assert_eq!(bits(identity.b, 40, 8), 0x11);
    }

    #[test]
    fn kind_inherited_from_an_earlier_message_still_selects_the_spare_body() {
        let mut area = Vec::new();
        append_frame(&mut area, 0x04, Some(0x14), None, None, &[]);
        append_frame(&mut area, 0x02, None, None, None, &response(1, 2));
        let report = report_for(&area, &[]);
        let spare = report.events[..report.len]
            .iter()
            .find(|event| event.code == SEND_CODE_DETAIL)
            .unwrap();
        assert_eq!(bits(spare.a, 24, 8), u64::from(SEND_SPARE_PRESENT));
        assert_eq!(bits(spare.a, 34, 8), 1);
        let second = report.events[..report.len]
            .iter()
            .filter(|event| event.code == SEND_CODE_IDENTITY)
            .nth(1)
            .unwrap();
        assert_eq!(bits(second.b, 8, 8), 0x14);
        assert_ne!(bits(second.b, 48, 8) & u64::from(MASK_KIND_INHERITED), 0);
    }

    #[test]
    fn a_truncated_frame_keeps_the_messages_parsed_before_it() {
        let source = 0x0102_0304_0506_0708u64;
        let mut area = Vec::new();
        append_frame(&mut area, 0x10, None, None, Some(source), &[]);
        area.push(0x02);
        let report = report_for(&area, &[]);
        assert_eq!(
            bits(report.events[0].a, 48, 8),
            u64::from(SEND_STATUS_TRUNCATED)
        );
        assert_eq!(bits(report.events[0].a, 40, 8), 1);
        assert_eq!(report.events[1].code, SEND_CODE_IDENTITY);
        assert_eq!(report.events[1].a, mix_id(source));
        assert_eq!(
            report.events[..report.len]
                .iter()
                .filter(|event| event.code == SEND_CODE_DESTINATION)
                .count(),
            0
        );
    }
}
