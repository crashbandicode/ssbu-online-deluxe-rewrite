#![cfg_attr(not(any(test, target_os = "switch")), allow(dead_code))]

//! Hook sites and frame geometry for the connection transport.
//!
//! `runtime::connection_transport` installs three inline hooks: the parsed
//! inbound message (`0x153844`), the station-index site (`0x15f0b8`) and the
//! pre-encryption send site (`0x155878`). The parsed-message hook feeds
//! [`crate::peer_transport::observe_incoming`], the station hook binds a
//! station index to its constant id, and the send hook commits the
//! fork-compatible announcements. None of that is opt-in.
//!
//! The opt-in peer observer that shared these sites (event codes 37 to 45, 64
//! and 65; config key `peer_observer`) was removed on 2026-10-17. What stays
//! here is what the live transport reads: the guards, the message-body window
//! and the diagnostic id mix.
//!
//! Nothing here sends a packet. Source and destination ids are mixed before
//! they are logged.

use crate::peer_passive::{PACKET_LEN_MAX, PACKET_LEN_MIN};

pub const PARSED_MESSAGE_OFFSET: usize = 0x153844;
pub const STATION_INDEX_OFFSET: usize = 0x15f0b8;
/// Index byte on the station in `x0` at [`STATION_INDEX_OFFSET`].
pub const STATION_INDEX_FIELD: usize = 0x38;
/// Source id on the message object. That pointer is still `x20` when the
/// inline hook runs, before `ldrb w20, [x0, #0x38]` replaces `w20`.
pub const MESSAGE_SOURCE_OFFSET: usize = 0x40;
/// Plaintext encrypt call. `x0` is the packet. The hooked word is `add x1, sp, #0x28`.
pub const SEND_PLAINTEXT_OFFSET: usize = 0x155878;

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

/// Diagnostic mix. It is not the Pia id and it is not a secret.
pub fn mix_id(id: u64) -> u64 {
    let mixed = id
        .wrapping_add(0xa5a5_a5a5_a5a5_a5a5)
        .wrapping_mul(0x9e37_79b1_85eb_ca87);
    mixed ^ mixed.rotate_left(27)
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

/// Inclusive wire-length window the game stored at `packet+0x5e8`. The same
/// window `peer_passive` checks before its parser runs.
pub const WIRE_LEN_MIN: u32 = PACKET_LEN_MIN;
pub const WIRE_LEN_MAX: u32 = PACKET_LEN_MAX;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guards_are_the_nso_bytes_at_the_three_sites() {
        assert_eq!(PARSED_MESSAGE_OFFSET, 0x153844);
        assert_eq!(STATION_INDEX_OFFSET, 0x15f0b8);
        assert_eq!(SEND_PLAINTEXT_OFFSET, 0x155878);
        assert_eq!(STATION_INDEX_FIELD, 0x38);
        assert_eq!(MESSAGE_SOURCE_OFFSET, 0x40);
        assert_eq!(PARSED_MESSAGE_GUARD.len(), 64);
        assert_eq!(STATION_INDEX_GUARD.len(), 64);
        assert_eq!(&PARSED_MESSAGE_GUARD[..4], &[0xa8, 0x02, 0x40, 0xf9]);
        assert_eq!(
            &STATION_INDEX_GUARD[..8],
            &[0x14, 0xe0, 0x40, 0x39, 0x9f, 0xf6, 0x03, 0x71]
        );
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
    }

    #[test]
    fn mixed_id_is_stable_and_not_the_pia_id() {
        assert_eq!(mix_id(0x1234_5678_9abc_def0), mix_id(0x1234_5678_9abc_def0));
        assert_ne!(mix_id(0x1234_5678_9abc_def0), 0x1234_5678_9abc_def0);
        assert_ne!(mix_id(1), mix_id(2));
        assert_ne!(mix_id(0), 0);
    }

    #[test]
    fn header_span_follows_the_flag_bits_and_kind_is_the_top_byte() {
        assert_eq!(kind_of(0x5800_0000), 0x58);
        assert_eq!(kind_of(0x1400_00ff), 0x14);
        assert_eq!(header_span(0), 1);
        assert_eq!(header_span(0x01), 2);
        assert_eq!(header_span(0x03), 4);
        assert_eq!(header_span(0x07), 8);
        assert_eq!(header_span(0x0f), 16);
        assert_eq!(header_span(0x1f), 24);
    }

    #[test]
    fn message_body_range_rejects_truncation_overflow_and_keeps_zero_length() {
        assert_eq!(WIRE_LEN_MIN, 0x20);
        assert_eq!(WIRE_LEN_MAX, 0x5c0);
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
