#![cfg_attr(not(test), allow(dead_code))]

//! Passive Pia message and fork-payload checks. Helpers are used by
//! `peer_transport` and `runtime::peer_observer`. This module does not install hooks.
//!
//! The message walk matches `libssbu_pia_manager` `0x41a8`. The packet length
//! window matches the checks at `0x5cf0` and `0x5b28`. The connection-response
//! beacon offsets match `0x2c3c`-`0x2c88`, and the interface clamp matches
//! `0x36e8`-`0x36fc`. The 4-byte payload codec matches the fork's
//! `normalize_and_parse_data` (`reference/quickplay/src/net/pia/mod.rs`).
//!
//! This module does not write packets, send a beacon, or choose a Stealth
//! mode. It also does not identify a game RTT station. The manager matches a
//! connection-id byte (`H+5` on send, `H+0x25` on receive) against its own
//! station object. Those bytes are not the RTT station index `0..=252`.

/// Pia packet magic at `H+0`. `H` is the packet object plus 8.
pub const PIA_MAGIC: u32 = 0x32ab_9864;
/// Pia header version byte at `H+4`.
pub const PIA_VERSION: u8 = 4;
/// Inclusive wire-length window checked before the parser runs.
pub const PACKET_LEN_MIN: u32 = 0x20;
pub const PACKET_LEN_MAX: u32 = 0x5c0;
/// Message area is `wire_len - 0x20`, so the largest slice the hooks parse.
pub const MESSAGE_AREA_MAX: usize = (PACKET_LEN_MAX - PACKET_LEN_MIN) as usize;

const TERMINATOR: u8 = 0xff;
const KIND_PRESENT: u8 = 4;
const FIRST_U64_PRESENT: u8 = 8;
const SECOND_U64_PRESENT: u8 = 16;
const BEACON_HEADER: usize = 0x39;
const LOCATION_STRIDE: usize = 0xc3;
const BEACON_TAG: u8 = 0x45;
const RTT_REPLY: [u8; 4] = [0x00, 0x00, 0x00, 0x01];
const FORK_PAYLOAD_LEN: usize = 4;
const RTT_BODY_MIN: usize = 0x10;

/// One framed message. Absent flag bits stay `None`.
///
/// The body length used to frame the message is a separate register. It starts
/// at 0 for each walk, a present length replaces it, and a clear length bit
/// reuses it. `body_len` records only the present field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PiaMessage<'a> {
    pub flags: u8,
    pub extra: Option<u8>,
    pub body_len: Option<u16>,
    pub kind: Option<u8>,
    pub kind_arg: Option<u32>,
    pub first_u64: Option<u64>,
    pub second_u64: Option<u64>,
    pub body: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Longer than the slice the manager passes to `0x41a8`.
    Oversize,
    /// A field, body, or alignment pad runs past the buffer.
    Truncated,
    /// `out` cannot hold another message.
    Capacity,
}

/// Magic, version, and the `0x20..=0x5c0` length window.
pub fn packet_prefix_accepted(magic: u32, version: u8, wire_len: u32) -> bool {
    magic == PIA_MAGIC
        && version == PIA_VERSION
        && (PACKET_LEN_MIN..=PACKET_LEN_MAX).contains(&wire_len)
}

/// Pia header byte at `H+4`. The low 7 bits are the version. Bit 7 is set
/// once the message area is encrypted. Custom writes require version 4 with
/// that bit clear.
pub fn custom_write_allowed(version_flags: u8) -> bool {
    version_flags & 0x80 == 0 && version_flags & 0x7f == PIA_VERSION
}

/// Kind-`0x58` body word. Only big-endian types 0 and 1 leave `body+4` unused
/// by the game. Every other type is left unchanged.
pub fn rtt_type_editable(body: &[u8]) -> bool {
    let Some(bytes) = body.get(..4) else {
        return false;
    };
    matches!(
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        0 | 1
    )
}

/// Walk `area` the way `0x41a8` walks the message list.
///
/// `0xFF` ends the list and the bytes after it are ignored. The end of the
/// buffer also ends the list. Each message's next start is 4-aligned relative
/// to that message's start, and the pad must fit. Bits above bit 4 add no
/// field; the manager ignores them. On `Err`, `out` is unspecified.
pub fn parse_message_area<'a>(
    area: &'a [u8],
    out: &mut [PiaMessage<'a>],
) -> Result<usize, ParseError> {
    let mut count = 0usize;
    walk_messages(area, out.len(), |message| {
        out[count] = message;
        count += 1;
    })
}

/// Same byte walk as [`parse_message_area`]. `limit` is checked before a
/// message is parsed, so a full output does not turn a later truncated frame
/// into [`ParseError::Truncated`]. The callback must not keep the body borrow.
///
/// Body framing follows the manager parser at `0x41a8`: the length register is
/// local to this call, starts at 0, and is replaced only when flag bit 1 is
/// set. A later message with that bit clear still consumes the previous
/// length, then 4-aligns. The next call starts at 0 again.
pub fn walk_messages<'a>(
    area: &'a [u8],
    limit: usize,
    mut on_message: impl FnMut(PiaMessage<'a>),
) -> Result<usize, ParseError> {
    if area.len() > MESSAGE_AREA_MAX {
        return Err(ParseError::Oversize);
    }
    let mut rest = area;
    let mut count = 0usize;
    let mut framed_len = 0u16;
    while !rest.is_empty() {
        let flags = rest[0];
        if flags == TERMINATOR {
            break;
        }
        if count == limit {
            return Err(ParseError::Capacity);
        }
        let (extra, mut cursor) = if flags & 1 == 0 {
            (None, 1usize)
        } else if rest.len() == 1 {
            return Err(ParseError::Truncated);
        } else {
            (Some(rest[1]), 2usize)
        };
        let body_len = if flags & 2 == 0 {
            None
        } else {
            let end = cursor + 2;
            if end > rest.len() {
                return Err(ParseError::Truncated);
            }
            let value = u16::from_be_bytes([rest[cursor], rest[cursor + 1]]);
            cursor = end;
            framed_len = value;
            Some(value)
        };
        let (kind, kind_arg) = if flags & KIND_PRESENT == 0 {
            (None, None)
        } else {
            let Some(end) = cursor.checked_add(4) else {
                return Err(ParseError::Truncated);
            };
            if cursor >= rest.len() || end > rest.len() {
                return Err(ParseError::Truncated);
            }
            let kind = rest[cursor];
            let arg = (u32::from(rest[cursor + 1]) << 16)
                | (u32::from(rest[cursor + 2]) << 8)
                | u32::from(rest[cursor + 3]);
            cursor = end;
            (Some(kind), Some(arg))
        };
        let first_u64 = read_be_u64(rest, flags, FIRST_U64_PRESENT, &mut cursor)?;
        let second_u64 = read_be_u64(rest, flags, SECOND_U64_PRESENT, &mut cursor)?;
        let body_bytes = usize::from(framed_len);
        let Some(body_end) = cursor.checked_add(body_bytes) else {
            return Err(ParseError::Truncated);
        };
        if body_end > rest.len() {
            return Err(ParseError::Truncated);
        }
        let aligned = (body_end + 3) & !3;
        if aligned > rest.len() {
            return Err(ParseError::Truncated);
        }
        on_message(PiaMessage {
            flags,
            extra,
            body_len,
            kind,
            kind_arg,
            first_u64,
            second_u64,
            body: &rest[cursor..body_end],
        });
        count += 1;
        rest = &rest[aligned..];
    }
    Ok(count)
}

fn read_be_u64(
    rest: &[u8],
    flags: u8,
    bit: u8,
    cursor: &mut usize,
) -> Result<Option<u64>, ParseError> {
    if flags & bit == 0 {
        return Ok(None);
    }
    let Some(end) = cursor.checked_add(8) else {
        return Err(ParseError::Truncated);
    };
    if end > rest.len() {
        return Err(ParseError::Truncated);
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&rest[*cursor..end]);
    *cursor = end;
    Ok(Some(u64::from_be_bytes(bytes)))
}

/// Fork `PiaCustomNetPacket` (`version`, `latency_bits`, native LE `u16`).
///
/// Version 0 is ignored, including a short buffer whose first byte is 0.
/// Version 2 clears profile bits 14 and 15. Version 3 is kept. Any other
/// version is a mismatch. The fork's `read_from_bytes` requires exactly four
/// bytes once version 0 has been excluded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForkPayload {
    Ignored,
    Accepted {
        version: u8,
        latency_bits: u8,
        profile_bits: u16,
    },
    Invalid,
    VersionMismatch,
}

pub fn decode_fork_payload(data: &[u8]) -> ForkPayload {
    if data.first() == Some(&0) {
        return ForkPayload::Ignored;
    }
    if data.len() != FORK_PAYLOAD_LEN {
        return ForkPayload::Invalid;
    }
    let version = data[0];
    let latency_bits = data[1];
    let mut profile_bits = u16::from_le_bytes([data[2], data[3]]);
    if version == 3 {
        return ForkPayload::Accepted {
            version,
            latency_bits,
            profile_bits,
        };
    }
    if version == 2 {
        profile_bits &= !((1 << 14) | (1 << 15));
        return ForkPayload::Accepted {
            version,
            latency_bits,
            profile_bits,
        };
    }
    ForkPayload::VersionMismatch
}

/// Four payload bytes of an incoming RTT reply, when the manager would reach
/// its receive hook. `None` means this body is not that reply. The bytes are
/// not cleared.
pub fn decode_rtt_reply_payload(body: &[u8]) -> Option<ForkPayload> {
    if body.len() < RTT_BODY_MIN || body[0..4] != RTT_REPLY {
        return None;
    }
    Some(decode_fork_payload(&body[4..8]))
}

/// Observed interface after the receive clamp. `0` includes every value other
/// than Wifi and Ethernet. Emulated (`3`) collapses to `0`; it is not a suffix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservedInterface {
    Unspecified = 0,
    Wifi = 1,
    Ethernet = 2,
}

pub fn clamp_observed_interface(raw: u8) -> ObservedInterface {
    match raw {
        1 => ObservedInterface::Wifi,
        2 => ObservedInterface::Ethernet,
        _ => ObservedInterface::Unspecified,
    }
}

/// Read-only view of a kind-`0x14` subtype-2 body whose second byte is 0.
/// Other bodies are [`Beacon::NotThisBody`]. No byte is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beacon {
    NotThisBody,
    /// Length or `nslots < count`. The manager returns without reading a tag.
    NotRead,
    /// `nslots == count`. The spare tail slot is absent, so the tag is not read.
    NoSpareSlot,
    /// Tag was not `0x45`. The manager forces the interface to 0.
    TagMissing,
    Tagged {
        raw: u8,
        interface: ObservedInterface,
    },
}

pub fn read_response_beacon(body: &[u8]) -> Beacon {
    if body.len() < 2 || body[0] != 2 || body[1] != 0 {
        return Beacon::NotThisBody;
    }
    if body.len() < BEACON_HEADER + LOCATION_STRIDE {
        return Beacon::NotRead;
    }
    let span = body.len() - BEACON_HEADER;
    let nslots = span / LOCATION_STRIDE;
    let count = usize::from(body[0x38]);
    if nslots < count {
        return Beacon::NotRead;
    }
    if nslots == count {
        return Beacon::NoSpareSlot;
    }
    let end = BEACON_HEADER + nslots * LOCATION_STRIDE;
    let (Some(tag_at), Some(iface_at)) = (end.checked_sub(0x54), end.checked_sub(0x53)) else {
        return Beacon::NotRead;
    };
    if iface_at >= body.len() {
        return Beacon::NotRead;
    }
    let tag = body[tag_at];
    if tag != BEACON_TAG {
        return Beacon::TagMissing;
    }
    let raw = body[iface_at];
    Beacon::Tagged {
        raw,
        interface: clamp_observed_interface(raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(area: &[u8]) -> Result<Vec<PiaMessage<'_>>, ParseError> {
        let mut out = [PiaMessage {
            flags: 0,
            extra: None,
            body_len: None,
            kind: None,
            kind_arg: None,
            first_u64: None,
            second_u64: None,
            body: &[],
        }; 8];
        match parse_message_area(area, &mut out) {
            Ok(n) => Ok(out[..n].to_vec()),
            Err(err) => Err(err),
        }
    }

    #[test]
    fn packet_length_window_is_inclusive_at_both_ends() {
        assert!(packet_prefix_accepted(PIA_MAGIC, 4, 0x20));
        assert!(packet_prefix_accepted(PIA_MAGIC, 4, 0x5c0));
        assert!(!packet_prefix_accepted(PIA_MAGIC, 4, 0x1f));
        assert!(!packet_prefix_accepted(PIA_MAGIC, 4, 0x5c1));
        assert!(!packet_prefix_accepted(PIA_MAGIC, 3, 0x20));
        assert!(!packet_prefix_accepted(0, 4, 0x20));
    }

    #[test]
    fn empty_and_terminator_produce_no_message() {
        assert_eq!(parse(&[]).unwrap().len(), 0);
        assert_eq!(parse(&[0xff]).unwrap().len(), 0);
        assert_eq!(parse(&[0xff, 1, 2, 3]).unwrap().len(), 0);
        let mut max = vec![0xff; MESSAGE_AREA_MAX];
        assert_eq!(parse(&max).unwrap().len(), 0);
        max.push(0xff);
        assert_eq!(parse(&max), Err(ParseError::Oversize));
    }

    #[test]
    fn short_buffers_and_padding_must_fit() {
        assert_eq!(parse(&[0x00]), Err(ParseError::Truncated));
        assert_eq!(parse(&[0x00, 0x00]), Err(ParseError::Truncated));
        assert_eq!(parse(&[0x00, 0x00, 0x00]), Err(ParseError::Truncated));
        let one = parse(&[0x00, 0x00, 0x00, 0x00]).unwrap();
        assert_eq!(one.len(), 1);
        assert!(one[0].body.is_empty());
        assert_eq!(one[0].extra, None);
        assert_eq!(parse(&[0x01]), Err(ParseError::Truncated));
        assert_eq!(parse(&[0x02, 0x00]), Err(ParseError::Truncated));
    }

    #[test]
    fn flags_fields_are_big_endian_and_high_bits_add_nothing() {
        let high = parse(&[0x20, 0x11, 0x22, 0x33]).unwrap();
        assert_eq!(high[0].flags, 0x20);
        assert_eq!(high[0].extra, None);
        assert!(high[0].body.is_empty());

        let unaligned = parse(&[0x02, 0x00, 0x01, 0xaa]).unwrap();
        assert_eq!(unaligned[0].body_len, Some(0x0001));
        assert_eq!(unaligned[0].body, &[0xaa]);
        assert_eq!(parse(&[0x02, 0x12, 0x34, 0x00]), Err(ParseError::Truncated));

        let mut area = vec![0x1f, 0xab];
        area.extend_from_slice(&0x0001u16.to_be_bytes());
        area.push(0x14);
        area.extend_from_slice(&[0x01, 0x02, 0x03]);
        area.extend_from_slice(&0x1122_3344_5566_7788u64.to_be_bytes());
        area.extend_from_slice(&0x99aa_bbcc_ddee_ff00u64.to_be_bytes());
        area.push(0x7e);
        area.extend_from_slice(&[0, 0, 0]);
        let parsed = parse(&area).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].extra, Some(0xab));
        assert_eq!(parsed[0].body_len, Some(1));
        assert_eq!(parsed[0].kind, Some(0x14));
        assert_eq!(parsed[0].kind_arg, Some(0x01_0203));
        assert_eq!(parsed[0].first_u64, Some(0x1122_3344_5566_7788));
        assert_eq!(parsed[0].second_u64, Some(0x99aa_bbcc_ddee_ff00));
        assert_eq!(parsed[0].body, &[0x7e]);
    }

    #[test]
    fn messages_are_four_aligned_and_a_second_truncation_fails() {
        let mut area = vec![0x00, 0, 0, 0, 0x02, 0x00, 0x01, 0xaa];
        let two = parse(&area).unwrap();
        assert_eq!(two.len(), 2);
        assert_eq!(two[1].body, &[0xaa]);
        area.pop();
        assert_eq!(parse(&area), Err(ParseError::Truncated));
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
            parse_message_area(&[0, 0, 0, 0, 0, 0, 0, 0], &mut out),
            Err(ParseError::Capacity)
        );
    }

    #[test]
    fn custom_writes_require_plaintext_version_4_and_rtt_type_0_or_1() {
        assert!(custom_write_allowed(4));
        assert!(!custom_write_allowed(4 | 0x80));
        assert!(!custom_write_allowed(0x83));
        assert!(!custom_write_allowed(3));
        assert!(!custom_write_allowed(0));
        assert!(rtt_type_editable(&[0, 0, 0, 0]));
        assert!(rtt_type_editable(&[0, 0, 0, 1, 9]));
        assert!(!rtt_type_editable(&[0, 0, 0, 2]));
        assert!(!rtt_type_editable(&[0, 0, 1, 0]));
        assert!(!rtt_type_editable(&[0, 0, 0]));
    }

    #[test]
    fn fork_payload_versions_and_rtt_reply_marker() {
        assert_eq!(decode_fork_payload(&[]), ForkPayload::Invalid);
        assert_eq!(decode_fork_payload(&[0]), ForkPayload::Ignored);
        assert_eq!(decode_fork_payload(&[0, 9, 0, 0, 1]), ForkPayload::Ignored);
        assert_eq!(
            decode_fork_payload(&[3, 4, 0x02, 0x01]),
            ForkPayload::Accepted {
                version: 3,
                latency_bits: 4,
                profile_bits: 0x0102,
            }
        );
        assert_eq!(
            decode_fork_payload(&[2, 7, 0xff, 0xff]),
            ForkPayload::Accepted {
                version: 2,
                latency_bits: 7,
                profile_bits: 0x3fff,
            }
        );
        assert_eq!(
            decode_fork_payload(&[4, 0, 0, 0]),
            ForkPayload::VersionMismatch
        );
        assert_eq!(decode_fork_payload(&[3, 0, 0]), ForkPayload::Invalid);
        assert_eq!(decode_fork_payload(&[3, 0, 0, 0, 0]), ForkPayload::Invalid);

        let mut reply = vec![0, 0, 0, 1, 3, 8, 0x34, 0x12];
        reply.extend_from_slice(&[0; 8]);
        assert_eq!(
            decode_rtt_reply_payload(&reply),
            Some(ForkPayload::Accepted {
                version: 3,
                latency_bits: 8,
                profile_bits: 0x1234,
            })
        );
        reply[0] = 1;
        assert_eq!(decode_rtt_reply_payload(&reply), None);
        assert_eq!(decode_rtt_reply_payload(&[0, 0, 0, 1, 3, 0, 0, 0]), None);
    }

    #[test]
    fn beacon_spare_slot_and_interface_clamp() {
        assert_eq!(read_response_beacon(&[1, 0]), Beacon::NotThisBody);
        assert_eq!(read_response_beacon(&[2, 1]), Beacon::NotThisBody);
        let mut short = vec![0u8; BEACON_HEADER + LOCATION_STRIDE - 1];
        short[0] = 2;
        assert_eq!(read_response_beacon(&short), Beacon::NotRead);

        let len = BEACON_HEADER + 2 * LOCATION_STRIDE;
        let mut body = vec![0u8; len];
        body[0] = 2;
        body[0x38] = 2;
        body[len - 0x54] = BEACON_TAG;
        body[len - 0x53] = 2;
        assert_eq!(read_response_beacon(&body), Beacon::NoSpareSlot);

        body[0x38] = 3;
        assert_eq!(read_response_beacon(&body), Beacon::NotRead);

        body[0x38] = 1;
        assert_eq!(
            read_response_beacon(&body),
            Beacon::Tagged {
                raw: 2,
                interface: ObservedInterface::Ethernet,
            }
        );
        body[len - 0x53] = 1;
        assert_eq!(
            read_response_beacon(&body),
            Beacon::Tagged {
                raw: 1,
                interface: ObservedInterface::Wifi,
            }
        );
        for raw in [0u8, 3, 9, 255] {
            body[len - 0x53] = raw;
            assert_eq!(
                read_response_beacon(&body),
                Beacon::Tagged {
                    raw,
                    interface: ObservedInterface::Unspecified,
                }
            );
        }
        body[len - 0x54] = 0;
        assert_eq!(read_response_beacon(&body), Beacon::TagMissing);
    }

    #[test]
    fn slot_division_matches_the_umulh_constant() {
        let magic: u128 = 0xa80a_80a8_0a80_a80b;
        for span in 0..=MESSAGE_AREA_MAX {
            let high = (((span as u128) * magic) >> 64) as u64;
            assert_eq!(
                u128::from(high >> 7),
                (span as u128) / (LOCATION_STRIDE as u128)
            );
        }
    }

    #[test]
    fn inherited_length_frames_the_body_and_resets_each_walk() {
        let mut first = vec![0x02, 0x00, 0x04, 0x11, 0x22, 0x33, 0x44];
        while !first.len().is_multiple_of(4) {
            first.push(0);
        }
        let split = first.len();
        first.push(0x00);
        first.extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd]);
        while !(first.len() - split).is_multiple_of(4) {
            first.push(0);
        }
        first.push(0xff);
        let parsed = parse(&first).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].body_len, Some(4));
        assert_eq!(parsed[0].body, &[0x11, 0x22, 0x33, 0x44]);
        assert_eq!(parsed[1].body_len, None);
        assert_eq!(parsed[1].flags, 0x00);
        assert_eq!(parsed[1].body, &[0xaa, 0xbb, 0xcc, 0xdd]);

        let mut aligned = vec![0x02, 0x00, 0x01, 0x7e];
        let second = aligned.len();
        aligned.push(0x00);
        aligned.push(0x5a);
        while !(aligned.len() - second).is_multiple_of(4) {
            aligned.push(0);
        }
        let bodies = parse(&aligned).unwrap();
        assert_eq!(bodies.len(), 2);
        assert_eq!(bodies[1].body_len, None);
        assert_eq!(bodies[1].body, &[0x5a]);

        let mut short = vec![0x02, 0x00, 0x04, 1, 2, 3, 4, 0];
        short.push(0x00);
        short.extend_from_slice(&[9, 9]);
        assert_eq!(parse(&short), Err(ParseError::Truncated));

        let pad = vec![0x02, 0x00, 0x01, 0x7e, 0x00, 0x5a];
        assert_eq!(parse(&pad), Err(ParseError::Truncated));

        let mut held = [PiaMessage {
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
            parse_message_area(&first, &mut held),
            Err(ParseError::Capacity)
        );
        assert_eq!(held[0].body, &[0x11, 0x22, 0x33, 0x44]);

        let reset = parse(&[0x00, 0, 0, 0]).unwrap();
        assert_eq!(reset.len(), 1);
        assert_eq!(reset[0].body_len, None);
        assert!(reset[0].body.is_empty());
    }
}
