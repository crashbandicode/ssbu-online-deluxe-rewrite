//! Connection transport hooks. Nothing here is opt-in.
//!
//! The parsed-message hook copies the body of kind `0x14` and `0x58` messages
//! into [`crate::peer_transport::observe_incoming`]; the station-index hook
//! binds a station index to its constant id; the send hook commits the
//! fork-compatible announcements at the pre-encryption site. The diagnostics
//! poll publishes the transport counters.
//!
//! The opt-in peer observer that shared these hooks (ring, station map,
//! send probe; config key `peer_observer`) was removed on 2026-10-17.
use super::{record, Code};
use crate::connection_transport::{
    kind_of, message_body_range, MessageBody, MESSAGE_SOURCE_OFFSET, PARSED_MESSAGE_GUARD,
    PARSED_MESSAGE_OFFSET, SEND_PLAINTEXT_GUARD, SEND_PLAINTEXT_OFFSET, STATION_INDEX_FIELD,
    STATION_INDEX_GUARD, STATION_INDEX_OFFSET, WIRE_LEN_MAX, WIRE_LEN_MIN,
};
use crate::peer_passive::{custom_write_allowed, MESSAGE_AREA_MAX, PIA_MAGIC};
use crate::peer_transport::{BodyWrite, Counter};
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

/// Transport counter values already drained into events, indexed by [`Counter`].
static LOGGED: [AtomicU64; 10] = [const { AtomicU64::new(0) }; 10];
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
}

/// Diagnostics thread only.
pub(super) fn poll() {
    refresh_local_facts();
    drain_transport();
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
    let kind = kind_of(kind_word);
    if kind != 0x14 && kind != 0x58 {
        return;
    }
    let Some(range) = message_body_range(packet, wire_len, message_at, flags, body_len) else {
        return;
    };
    let mut inline_body = [0u8; 0x5a0];
    let body = match range {
        MessageBody::Empty => &[][..],
        MessageBody::Bytes { at, len } => {
            if at < 0x10000 {
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
        return;
    };
    let Some(connection_id) = read_at::<u8>(packet, 0xd) else {
        return;
    };
    let Some(wire_len) = read_at::<u32>(packet, 0x5e8) else {
        return;
    };
    let Some(magic_ok) = read_magic(packet) else {
        return;
    };
    let mut area = [0u8; MESSAGE_AREA_MAX];
    let area_len = if (WIRE_LEN_MIN..=WIRE_LEN_MAX).contains(&wire_len) {
        (wire_len - WIRE_LEN_MIN) as usize
    } else {
        0
    };
    if area_len > 0 && !copy_bytes(packet, 0x48, &mut area[..area_len]) {
        return;
    }
    if !magic_ok || area_len == 0 || !custom_write_allowed(version_flags) {
        return;
    }
    crate::peer_transport::commit_outgoing(
        version_flags,
        connection_id,
        &area[..area_len],
        Some(current_announce()),
        LOCAL_INTERFACE.load(Ordering::Relaxed),
        super::stealth(),
        |writes| apply_body_writes(packet, area_len, writes),
    );
}

/// Wire profile is the machine's committed preset while that machine is live.
///
/// Menus and matches use the same rule. A saved mode is not sent while the
/// machine is passthrough, and core modes are not a live profile. Auto is not
/// a wire value. `Profile::Doubles` is the fork's `less_lag_doubles` bits. The
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
        settings.ultra_resolution.level(),
    );
    crate::peer_transport::announce_from(
        settings.input_buffer,
        decision.live,
        decision.committed,
        setting,
    )
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

/// Stores the planned announcement bytes into our own outgoing packet. Nothing
/// is saved or put back: a cleanup racing this store cannot corrupt peer state,
/// and the packet is already addressed to the departing peer (2026-10-17).
fn apply_body_writes(packet: u64, area_len: usize, writes: &[Option<BodyWrite>]) {
    for write in writes.iter().flatten() {
        let len = usize::from(write.len);
        let Some(start) = body_addr(packet, area_len, write.at, len) else {
            continue;
        };
        unsafe {
            core::ptr::copy_nonoverlapping(write.bytes.as_ptr(), start as *mut u8, len);
        }
    }
}

fn drain_transport() {
    let last_mix = crate::peer_transport::last_mix();
    let learned = counter_delta(Counter::Learned);
    if learned != 0 {
        record(Code::TransportHandshake, last_mix, learned);
    }
    let confirmed = counter_delta(Counter::Confirmed);
    if confirmed != 0 {
        record(Code::TransportConfirm, last_mix, confirmed);
    }
    let cleanups = counter_delta(Counter::Cleanups);
    if cleanups != 0 {
        record(Code::TransportCleanup, cleanups, 0);
    }
    let (writes, skips) = (
        counter_delta(Counter::BeaconWrites),
        counter_delta(Counter::BeaconSkips),
    );
    if writes != 0 || skips != 0 {
        record(Code::TransportBeacon, writes, skips);
    }
    let (writes, skips) = (
        counter_delta(Counter::PayloadWrites),
        counter_delta(Counter::PayloadSkips),
    );
    if writes != 0 || skips != 0 {
        record(Code::TransportPayload, writes, skips);
    }
    let stealth = counter_delta(Counter::StealthSuppress);
    if stealth != 0 {
        record(Code::TransportStealth, stealth, last_mix);
    }
    let (ambiguous, contentions) = (
        counter_delta(Counter::Ambiguous),
        counter_delta(Counter::Contentions),
    );
    if ambiguous != 0 || contentions != 0 {
        record(Code::TransportSkip, ambiguous, contentions);
    }
}

fn counter_delta(counter: Counter) -> u64 {
    let now = crate::peer_transport::counter(counter);
    now.wrapping_sub(LOGGED[counter as usize].swap(now, Ordering::Relaxed))
}

#[skyline::hook(offset = SEND_PLAINTEXT_OFFSET, inline)]
unsafe fn deluxe_rewrite_observe_send_plaintext(ctx: &InlineCtx) {
    observe_send_packet(ctx.registers[0].x());
}
