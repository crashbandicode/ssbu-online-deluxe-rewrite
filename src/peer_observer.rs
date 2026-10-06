#![cfg_attr(not(any(test, target_os = "switch")), allow(dead_code))]

//! Opt-in diagnostic view of an already-parsed inbound Pia message.
//!
//! `parse_message_area` keeps absent header fields as `None`. The game parser
//! at text+`0x157a1c` copies the previous header and overwrites only the flag
//! bits that are present. [`inherit_header`] is that second rule. The hook
//! reads the game's header, so it does not apply [`inherit_header`] again.
//!
//! Nothing here sends a packet. Source ids are mixed before they are logged.
//! A name is a bounded escaped sample from a subtype-2 entry, or it is absent.

use crate::peer_passive::{read_response_beacon, Beacon};

pub const STATION_SLOTS: usize = 253;
pub const PARSED_MESSAGE_OFFSET: usize = 0x153844;
pub const STATION_INDEX_OFFSET: usize = 0x15f0b8;
pub const ENTRY_STRIDE: usize = 0xc3;
pub const ENTRY_NAME: usize = 0x51;
pub const ENTRY_NAME_LEN: usize = 0x28;
pub const ENTRY_TAG: usize = 0x79;
pub const NAME_CAP: usize = 16;

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
    Bytes { at: u64, len: usize },
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
    let stale = if index < STATION_SLOTS && live_generation.is_some_and(|live| live != generation)
    {
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
    let subtype = if kind == 0x14 { body.first().copied() } else { None };
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
    let end = window.iter().position(|byte| *byte == 0).unwrap_or(window.len());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer_passive::{parse_message_area, ObservedInterface};

    #[test]
    fn guards_are_the_nso_bytes_at_the_two_sites() {
        assert_eq!(PARSED_MESSAGE_OFFSET, 0x153844);
        assert_eq!(STATION_INDEX_OFFSET, 0x15f0b8);
        assert_eq!(PARSED_MESSAGE_GUARD.len(), 64);
        assert_eq!(STATION_INDEX_GUARD.len(), 64);
        assert_eq!(&PARSED_MESSAGE_GUARD[..4], &[0xa8, 0x02, 0x40, 0xf9]);
        assert_eq!(&STATION_INDEX_GUARD[..8], &[0x14, 0xe0, 0x40, 0x39, 0x9f, 0xf6, 0x03, 0x71]);
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
        assert_eq!(
            &decode_entry_name(&entry).unwrap().bytes[..4],
            b"\\x5c"
        );

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
        assert_eq!(&surrogate.bytes[..surrogate.len as usize], b"\\xef\\xbf\\xbd");
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
}
