//! Horizon-OC game-aware gate, pure side (0.9.0). The wire format and the
//! semantics are protocol v1, the contract shared with the hoc-clk fork
//! (`hoc:clk` commands 100-103; `GetApiVersion` stays 2 so the stock overlay
//! keeps working). This module frames the IPC messages, classifies the
//! handshake, decides when Smash's saved profile may apply, paces the
//! heartbeat and names the indicator state. It never carries a clock value
//! and never touches `config.ini`: the fork only chooses between the saved
//! title profile and Horizon-OC's own global/stock fallback.
//!
//! Framing follows libnx: `cmifMakeRequest` over `hipcMakeRequest` for CMIF
//! (sm `RegisterClient`/`GetServiceHandle`, every hoc:clk command) and
//! `tipcMakeRequest` for sm `AtmosphereHasService`, which never blocks.
//! Atmosphère's sm defers `GetServiceHandle` for an unregistered service
//! until it registers, so the client asks `HasService` first.

use crate::online_session::TICKS_PER_SECOND;

/// The thread's TLS IPC message buffer.
pub const IPC_BUFFER_SIZE: usize = 0x100;
pub type IpcBuffer = [u8; IPC_BUFFER_SIZE];

/// Named port of the service manager.
pub const SM_PORT: &[u8; 4] = b"sm:\0";
/// `SmServiceName` of the hoc-clk sysmodule.
pub const SERVICE: [u8; 8] = *b"hoc:clk\0";

pub const SM_REGISTER_CLIENT: u32 = 0;
pub const SM_GET_SERVICE_HANDLE: u32 = 1;
pub const SM_ATMOSPHERE_HAS_SERVICE: u32 = 65100;

pub const CMD_GET_API_VERSION: u32 = 0;
pub const CMD_INFO: u32 = 100;
pub const CMD_ARM: u32 = 101;
pub const CMD_SET_APPLY: u32 = 102;
pub const CMD_DISARM: u32 = 103;

/// `HOCCLK_IPC_API_VERSION`; the fork does not change it.
pub const API_VERSION: u32 = 2;
/// `'HOGA'` as stored little-endian.
pub const MAGIC: u32 = 0x4147_4F48;
pub const PROTOCOL_VERSION: u16 = 1;
pub const CAP_TITLE_GATE: u16 = 1;
pub const CLIENT_ABI: u32 = 1;
/// Lease per heartbeat. Three missed heartbeats drop the profile.
pub const LEASE_MS: u32 = 3000;

/// `HOCCLK_ERROR(Generic)`: stock hoc-clk 2.6.0 answers every unknown command
/// with it, so cmd 100 failing with this result means "stock Horizon-OC".
pub const RESULT_GENERIC: u32 = 0x184;
pub const RESULT_NOT_APPLICATION: u32 = 0xC984;
pub const RESULT_NOT_OWNER: u32 = 0xCB84;
pub const RESULT_BAD_ARGS: u32 = 0xCD84;

const CMIF_IN_MAGIC: u32 = 0x4943_4653;
const CMIF_OUT_MAGIC: u32 = 0x4F43_4653;
const HIPC_REQUEST: u32 = 4;
const TIPC_COMMAND_BASE: u32 = 16;

pub const HEARTBEAT_TICKS: u64 = TICKS_PER_SECOND;
/// Re-handshake after an IPC error or a missing service.
pub const RETRY_TICKS: u64 = 5 * TICKS_PER_SECOND;
/// Stock or incompatible Horizon-OC only changes when hoc-clk is replaced.
pub const RECHECK_TICKS: u64 = 30 * TICKS_PER_SECOND;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireError {
    Truncated,
    BadMagic,
}

fn put_u32(buf: &mut IpcBuffer, offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn get_u32(buf: &[u8], offset: usize) -> Option<u32> {
    buf.get(offset..offset + 4)
        .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

const fn align16(offset: usize) -> usize {
    (offset + 15) & !15
}

/// CMIF request without buffers or handles; `send_pid` adds the special
/// header and the u64 the kernel overwrites with the caller's process id.
/// `payload` is at most 16 bytes here.
pub fn encode_cmif(buf: &mut IpcBuffer, command: u32, send_pid: bool, payload: &[u8]) {
    buf.fill(0);
    // libnx: 16 bytes of alignment slack, the 16-byte header, the payload,
    // rounded to a half-word, then to words.
    let size = (16 + 16 + payload.len() + 1) & !1;
    let words = size.div_ceil(4) as u32;
    put_u32(buf, 0, HIPC_REQUEST);
    put_u32(buf, 4, words | (u32::from(send_pid) << 31));
    let data = if send_pid {
        put_u32(buf, 8, 1);
        8 + 4 + 8
    } else {
        8
    };
    let start = align16(data);
    put_u32(buf, start, CMIF_IN_MAGIC);
    put_u32(buf, start + 8, command);
    buf[start + 16..start + 16 + payload.len()].copy_from_slice(payload);
}

/// TIPC request: the command rides in the message type, data follows the
/// header directly.
pub fn encode_tipc(buf: &mut IpcBuffer, command: u32, payload: &[u8]) {
    buf.fill(0);
    put_u32(buf, 0, command + TIPC_COMMAND_BASE);
    put_u32(buf, 4, payload.len().div_ceil(4) as u32);
    buf[8..8 + payload.len()].copy_from_slice(payload);
}

/// Where a response's data words start and how many, plus the first moved
/// handle. Statics and buffer descriptors are skipped by count.
fn parse_hipc(buf: &IpcBuffer) -> Result<(usize, usize, Option<u32>), WireError> {
    let word0 = get_u32(buf, 0).ok_or(WireError::Truncated)?;
    let word1 = get_u32(buf, 4).ok_or(WireError::Truncated)?;
    let mut offset = 8;
    let mut moved = None;
    if word1 >> 31 != 0 {
        let special = get_u32(buf, 8).ok_or(WireError::Truncated)?;
        offset = 12;
        if special & 1 != 0 {
            offset += 8;
        }
        let copies = ((special >> 1) & 0xf) as usize;
        let moves = ((special >> 5) & 0xf) as usize;
        offset += 4 * copies;
        if moves > 0 {
            moved = Some(get_u32(buf, offset).ok_or(WireError::Truncated)?);
        }
        offset += 4 * moves;
    }
    let statics = ((word0 >> 16) & 0xf) as usize;
    let buffers = ((word0 >> 20) & 0xf) + ((word0 >> 24) & 0xf) + ((word0 >> 28) & 0xf);
    offset += 8 * statics + 12 * buffers as usize;
    let words = (word1 & 0x3ff) as usize;
    if offset + 4 * words > IPC_BUFFER_SIZE {
        return Err(WireError::Truncated);
    }
    Ok((offset, words, moved))
}

/// A decoded response: the result, up to 32 payload bytes and a moved handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Reply {
    pub result: u32,
    pub payload: [u8; 32],
    pub len: usize,
    pub moved: Option<u32>,
}

impl Reply {
    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.len]
    }
}

fn reply(result: u32, bytes: &[u8], moved: Option<u32>) -> Reply {
    let mut payload = [0; 32];
    let len = bytes.len().min(payload.len());
    payload[..len].copy_from_slice(&bytes[..len]);
    Reply {
        result,
        payload,
        len,
        moved,
    }
}

pub fn decode_cmif(buf: &IpcBuffer) -> Result<Reply, WireError> {
    let (data, words, moved) = parse_hipc(buf)?;
    let end = data + 4 * words;
    let start = align16(data);
    if start + 16 > end {
        return Err(WireError::Truncated);
    }
    if get_u32(buf, start) != Some(CMIF_OUT_MAGIC) {
        return Err(WireError::BadMagic);
    }
    let result = get_u32(buf, start + 8).ok_or(WireError::Truncated)?;
    Ok(reply(result, &buf[start + 16..end], moved))
}

pub fn decode_tipc(buf: &IpcBuffer) -> Result<Reply, WireError> {
    let (data, words, moved) = parse_hipc(buf)?;
    if words == 0 {
        return Err(WireError::Truncated);
    }
    let result = get_u32(buf, data).ok_or(WireError::Truncated)?;
    Ok(reply(result, &buf[data + 4..data + 4 * words], moved))
}

/// `HocClkGameAwareInfo`, 16 bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Info {
    pub version: u16,
    pub caps: u16,
    pub max_lease_ms: u32,
    pub poll_ms: u32,
}

pub fn decode_info(payload: &[u8]) -> Option<Info> {
    if payload.len() < 16 || get_u32(payload, 0)? != MAGIC {
        return None;
    }
    Some(Info {
        version: u16::from_le_bytes([payload[4], payload[5]]),
        caps: u16::from_le_bytes([payload[6], payload[7]]),
        max_lease_ms: get_u32(payload, 8)?,
        poll_ms: get_u32(payload, 12)?,
    })
}

/// `HocClkGameAwareArmArgs`, 16 bytes.
pub fn arm_args(program_id: u64) -> [u8; 16] {
    let mut args = [0; 16];
    args[..8].copy_from_slice(&program_id.to_le_bytes());
    args[8..12].copy_from_slice(&CLIENT_ABI.to_le_bytes());
    args
}

/// `HocClkGameAwareSetApplyArgs`, 12 bytes.
pub fn set_apply_args(apply: bool, lease_ms: u32, seq: u32) -> [u8; 12] {
    let mut args = [0; 12];
    args[0] = u8::from(apply);
    args[4..8].copy_from_slice(&lease_ms.to_le_bytes());
    args[8..12].copy_from_slice(&seq.to_le_bytes());
    args
}

/// `HocClkGameAwareState`, 8 bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct State {
    pub armed: bool,
    pub applying: bool,
    pub generation: u32,
}

pub fn decode_state(payload: &[u8]) -> Option<State> {
    if payload.len() < 8 {
        return None;
    }
    Some(State {
        armed: payload[0] != 0,
        applying: payload[1] != 0,
        generation: get_u32(payload, 4)?,
    })
}

/// What the handshake found.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum Capability {
    /// No handshake has finished yet.
    #[default]
    Unknown = 0,
    /// `hoc:clk` is not registered (Horizon-OC absent or its sysmodule exited).
    NoService = 1,
    /// Stock hoc-clk: API version 2 and cmd 100 is unknown (`0x184`).
    Absent = 2,
    /// Another API version, protocol version or reply shape.
    Incompatible = 3,
    /// The game-aware fork answered protocol v1.
    Present = 4,
    /// The fork was present and a later call failed; retrying.
    Unavailable = 5,
}

impl Capability {
    pub const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::NoService,
            2 => Self::Absent,
            3 => Self::Incompatible,
            4 => Self::Present,
            5 => Self::Unavailable,
            _ => Self::Unknown,
        }
    }
}

/// Classifies the `GetApiVersion` answer and the cmd 100 reply.
pub fn classify(api_version: u32, info_result: u32, info_payload: &[u8]) -> Capability {
    if api_version != API_VERSION {
        return Capability::Incompatible;
    }
    match info_result {
        0 => match decode_info(info_payload) {
            Some(info) if info.version == PROTOCOL_VERSION && info.caps & CAP_TITLE_GATE != 0 => {
                Capability::Present
            }
            _ => Capability::Incompatible,
        },
        RESULT_GENERIC => Capability::Absent,
        _ => Capability::Incompatible,
    }
}

/// How long to wait before the next handshake after `capability`.
pub const fn retry_ticks(capability: Capability) -> u64 {
    match capability {
        Capability::Absent | Capability::Incompatible => RECHECK_TICKS,
        _ => RETRY_TICKS,
    }
}

/// `hoc_oc` in the INI. `Always` is Horizon-OC's own behavior (the saved
/// profile whenever Smash runs); `Online` lets the profile apply only while the
/// shared online session is live.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum OcMode {
    #[default]
    Always = 0,
    Online = 1,
}

impl OcMode {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Online => "online",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "always" => Some(Self::Always),
            "online" => Some(Self::Online),
            _ => None,
        }
    }

    pub const fn from_byte(value: u8) -> Self {
        match value {
            1 => Self::Online,
            _ => Self::Always,
        }
    }

    pub const fn toggled(self) -> Self {
        match self {
            Self::Always => Self::Online,
            Self::Online => Self::Always,
        }
    }
}

/// The fork is armed only in `Online` mode with the capability present.
pub const fn want_armed(mode: OcMode, capability: Capability) -> bool {
    matches!(mode, OcMode::Online) && matches!(capability, Capability::Present)
}

/// The saved profile may apply: armed and the online session is live.
pub const fn want_apply(mode: OcMode, capability: Capability, online: bool) -> bool {
    want_armed(mode, capability) && online
}

/// Paces `SetApply`: at once on a change, otherwise every
/// [`HEARTBEAT_TICKS`] in both states, so a restarted hoc-clk (which comes
/// back unarmed) is noticed within a second even while the profile is off.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Heartbeat {
    last: Option<(bool, u64)>,
}

impl Heartbeat {
    pub fn due(&self, apply: bool, now: u64) -> bool {
        match self.last {
            None => true,
            Some((sent, tick)) => sent != apply || now.wrapping_sub(tick) >= HEARTBEAT_TICKS,
        }
    }

    pub fn sent(&mut self, apply: bool, now: u64) {
        self.last = Some((apply, now));
    }

    pub fn reset(&mut self) {
        self.last = None;
    }
}

/// What the You-line mark and the Performance `Clock` fact show.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Indicator {
    /// No fork: nothing is shown and ZR+L3 does nothing.
    Hidden,
    /// Fork present, `Always`: Horizon-OC decides as it always has.
    Always,
    /// `Online`, session live: the saved profile is applied.
    Applying,
    /// `Online`, offline or idle: Horizon-OC's global/stock fallback.
    Idle,
    /// `Online` but the Pia observer is not installed, so the profile never
    /// applies.
    NoSignal,
    /// `Online` and the fork stopped answering; retrying.
    Unavailable,
}

/// The mark's tone on the You line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarkTone {
    Lit,
    Dim,
    Warn,
}

pub const fn indicator(
    mode: OcMode,
    capability: Capability,
    pia_ready: bool,
    applying: bool,
) -> Indicator {
    match (capability, mode) {
        (Capability::Present, OcMode::Always) => Indicator::Always,
        (Capability::Present, OcMode::Online) if !pia_ready => Indicator::NoSignal,
        (Capability::Present, OcMode::Online) if applying => Indicator::Applying,
        (Capability::Present, OcMode::Online) => Indicator::Idle,
        (Capability::Unavailable, OcMode::Online) => Indicator::Unavailable,
        _ => Indicator::Hidden,
    }
}

impl Indicator {
    /// Appended to the Performance `Clock` fact.
    pub const fn label(self) -> Option<&'static str> {
        match self {
            Self::Hidden => None,
            Self::Always => Some("OC always"),
            Self::Applying => Some("OC online: on"),
            Self::Idle => Some("OC online: idle"),
            Self::NoSignal => Some("OC online: no signal"),
            Self::Unavailable => Some("OC online: unavailable"),
        }
    }

    /// The `OC` mark after the You-line privacy icons; `Online` mode only.
    pub const fn mark(self) -> Option<MarkTone> {
        match self {
            Self::Hidden | Self::Always => None,
            Self::Applying => Some(MarkTone::Lit),
            Self::Idle => Some(MarkTone::Dim),
            Self::NoSignal | Self::Unavailable => Some(MarkTone::Warn),
        }
    }

    /// ZR+L3 changes the saved mode only while the fork answered.
    pub const fn toggle_available(capability: Capability) -> bool {
        matches!(capability, Capability::Present | Capability::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(buf: &IpcBuffer, count: usize) -> Vec<u32> {
        (0..count).map(|i| get_u32(buf, i * 4).unwrap()).collect()
    }

    #[test]
    fn cmif_without_pid_matches_libnx_layout() {
        let mut buf = [0xAA; IPC_BUFFER_SIZE];
        encode_cmif(&mut buf, CMD_INFO, false, &[]);
        // Request type, 8 data words; the header sits at the first 16-byte
        // boundary after the two header words.
        assert_eq!(
            words(&buf, 8),
            [4, 8, 0, 0, 0x4943_4653, 0, 100, 0]
        );
        assert!(buf[32..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn cmif_with_pid_carries_the_special_header_and_placeholder() {
        let mut buf = [0; IPC_BUFFER_SIZE];
        encode_cmif(&mut buf, SM_REGISTER_CLIENT, true, &0u64.to_le_bytes());
        // 16 + 16 + 8 = 40 bytes -> 10 words; special header bit; send_pid;
        // pid placeholder; data starts at 20, the header at 32.
        assert_eq!(
            words(&buf, 14),
            [4, 0x8000_000A, 1, 0, 0, 0, 0, 0, 0x4943_4653, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn get_service_handle_sends_the_service_name() {
        let mut buf = [0; IPC_BUFFER_SIZE];
        encode_cmif(&mut buf, SM_GET_SERVICE_HANDLE, false, &SERVICE);
        assert_eq!(words(&buf, 2), [4, 10]);
        assert_eq!(get_u32(&buf, 24), Some(1));
        assert_eq!(&buf[32..40], b"hoc:clk\0");
    }

    #[test]
    fn set_apply_and_arm_sizes() {
        let mut buf = [0; IPC_BUFFER_SIZE];
        encode_cmif(&mut buf, CMD_SET_APPLY, true, &set_apply_args(true, LEASE_MS, 7));
        assert_eq!(words(&buf, 3), [4, 0x8000_000B, 1]);
        assert_eq!(get_u32(&buf, 40), Some(102));
        assert_eq!(&buf[48..60], &[1, 0, 0, 0, 0xB8, 0x0B, 0, 0, 7, 0, 0, 0]);
        encode_cmif(&mut buf, CMD_ARM, true, &arm_args(0x0100_6A80_0016_E000));
        assert_eq!(words(&buf, 3), [4, 0x8000_000C, 1]);
        assert_eq!(
            &buf[48..64],
            &[0x00, 0xE0, 0x16, 0x00, 0x80, 0x6A, 0x00, 0x01, 1, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn has_service_is_tipc() {
        let mut buf = [0; IPC_BUFFER_SIZE];
        encode_tipc(&mut buf, SM_ATMOSPHERE_HAS_SERVICE, &SERVICE);
        assert_eq!(words(&buf, 2), [65116, 2]);
        assert_eq!(&buf[8..16], b"hoc:clk\0");
    }

    /// hoc-clk's own response shape (`_ipcServerPrepareResponse`).
    fn hoc_response(result: u32, payload: &[u8]) -> IpcBuffer {
        let mut buf = [0; IPC_BUFFER_SIZE];
        put_u32(&mut buf, 0, 4);
        put_u32(&mut buf, 4, ((16 + payload.len() + 16) / 4) as u32);
        put_u32(&mut buf, 16, CMIF_OUT_MAGIC);
        put_u32(&mut buf, 24, result);
        buf[32..32 + payload.len()].copy_from_slice(payload);
        buf
    }

    fn info_bytes(magic: u32, version: u16, caps: u16) -> [u8; 16] {
        let mut info = [0; 16];
        info[..4].copy_from_slice(&magic.to_le_bytes());
        info[4..6].copy_from_slice(&version.to_le_bytes());
        info[6..8].copy_from_slice(&caps.to_le_bytes());
        info[8..12].copy_from_slice(&5000u32.to_le_bytes());
        info[12..16].copy_from_slice(&300u32.to_le_bytes());
        info
    }

    #[test]
    fn decodes_a_hoc_info_reply() {
        let buf = hoc_response(0, &info_bytes(MAGIC, 1, 1));
        let reply = decode_cmif(&buf).unwrap();
        assert_eq!(reply.result, 0);
        assert_eq!(
            decode_info(reply.payload()),
            Some(Info {
                version: 1,
                caps: 1,
                max_lease_ms: 5000,
                poll_ms: 300
            })
        );
        assert_eq!(classify(2, reply.result, reply.payload()), Capability::Present);
    }

    #[test]
    fn classifies_stock_and_incompatible_servers() {
        let stock = decode_cmif(&hoc_response(RESULT_GENERIC, &[])).unwrap();
        assert_eq!(classify(2, stock.result, stock.payload()), Capability::Absent);
        assert_eq!(classify(3, 0, &info_bytes(MAGIC, 1, 1)), Capability::Incompatible);
        assert_eq!(classify(2, 0, &info_bytes(0x1234, 1, 1)), Capability::Incompatible);
        assert_eq!(classify(2, 0, &info_bytes(MAGIC, 2, 1)), Capability::Incompatible);
        assert_eq!(classify(2, 0, &info_bytes(MAGIC, 1, 0)), Capability::Incompatible);
        assert_eq!(classify(2, 0, &[0; 8]), Capability::Incompatible);
        assert_eq!(classify(2, RESULT_NOT_OWNER, &[]), Capability::Incompatible);
    }

    #[test]
    fn rejects_bad_magic_and_truncation() {
        let mut buf = hoc_response(0, &[]);
        put_u32(&mut buf, 16, CMIF_IN_MAGIC);
        assert_eq!(decode_cmif(&buf), Err(WireError::BadMagic));
        let mut short = hoc_response(0, &[]);
        put_u32(&mut short, 4, 2);
        assert_eq!(decode_cmif(&short), Err(WireError::Truncated));
        let mut huge = hoc_response(0, &[]);
        put_u32(&mut huge, 4, 0x3ff);
        assert_eq!(decode_cmif(&huge), Err(WireError::Truncated));
    }

    #[test]
    fn decodes_a_moved_service_handle() {
        let mut buf = [0; IPC_BUFFER_SIZE];
        put_u32(&mut buf, 0, 4);
        put_u32(&mut buf, 4, 8 | 1 << 31);
        put_u32(&mut buf, 8, 1 << 5);
        put_u32(&mut buf, 12, 0xF00D);
        put_u32(&mut buf, 16, CMIF_OUT_MAGIC);
        let reply = decode_cmif(&buf).unwrap();
        assert_eq!(reply.result, 0);
        assert_eq!(reply.moved, Some(0xF00D));
    }

    #[test]
    fn decodes_a_tipc_has_service_reply() {
        let mut buf = [0; IPC_BUFFER_SIZE];
        put_u32(&mut buf, 0, 16);
        put_u32(&mut buf, 4, 2);
        put_u32(&mut buf, 12, 1);
        let reply = decode_tipc(&buf).unwrap();
        assert_eq!((reply.result, reply.payload()[0]), (0, 1));
        put_u32(&mut buf, 4, 0);
        assert_eq!(decode_tipc(&buf), Err(WireError::Truncated));
    }

    #[test]
    fn decodes_the_state_reply() {
        assert_eq!(
            decode_state(&[1, 1, 0, 0, 9, 0, 0, 0]),
            Some(State {
                armed: true,
                applying: true,
                generation: 9
            })
        );
        assert_eq!(decode_state(&[1, 0, 0]), None);
    }

    #[test]
    fn result_constants_match_hocclk_error() {
        let hocclk = |desc: u32| 388 | (desc << 9);
        assert_eq!(RESULT_GENERIC, hocclk(0));
        assert_eq!(RESULT_NOT_APPLICATION, hocclk(100));
        assert_eq!(RESULT_NOT_OWNER, hocclk(101));
        assert_eq!(RESULT_BAD_ARGS, hocclk(102));
        assert_eq!(&MAGIC.to_le_bytes(), b"HOGA");
    }

    #[test]
    fn the_gate_needs_online_mode_the_fork_and_a_live_session() {
        use Capability::*;
        for capability in [Unknown, NoService, Absent, Incompatible, Present, Unavailable] {
            for online in [false, true] {
                assert!(!want_apply(OcMode::Always, capability, online));
                assert_eq!(
                    want_apply(OcMode::Online, capability, online),
                    capability == Present && online
                );
            }
        }
        assert!(want_armed(OcMode::Online, Present));
        assert!(!want_armed(OcMode::Always, Present));
        assert!(!want_armed(OcMode::Online, Absent));
    }

    #[test]
    fn heartbeat_sends_changes_at_once_and_repeats_both_states() {
        let mut beat = Heartbeat::default();
        assert!(beat.due(false, 0));
        beat.sent(false, 0);
        assert!(!beat.due(false, HEARTBEAT_TICKS - 1));
        assert!(beat.due(false, HEARTBEAT_TICKS));
        assert!(beat.due(true, 1));
        beat.sent(true, 1);
        assert!(!beat.due(true, 2));
        assert!(beat.due(true, 1 + HEARTBEAT_TICKS));
        beat.reset();
        assert!(beat.due(true, 2));
    }

    #[test]
    fn retry_pacing() {
        assert_eq!(retry_ticks(Capability::Absent), RECHECK_TICKS);
        assert_eq!(retry_ticks(Capability::Incompatible), RECHECK_TICKS);
        assert_eq!(retry_ticks(Capability::NoService), RETRY_TICKS);
        assert_eq!(retry_ticks(Capability::Unavailable), RETRY_TICKS);
    }

    #[test]
    fn indicator_states() {
        use Capability::*;
        let online = OcMode::Online;
        assert_eq!(indicator(online, Present, true, true), Indicator::Applying);
        assert_eq!(indicator(online, Present, true, false), Indicator::Idle);
        assert_eq!(indicator(online, Present, false, true), Indicator::NoSignal);
        assert_eq!(indicator(OcMode::Always, Present, true, true), Indicator::Always);
        assert_eq!(indicator(online, Unavailable, true, false), Indicator::Unavailable);
        assert_eq!(indicator(OcMode::Always, Unavailable, true, false), Indicator::Hidden);
        for capability in [Unknown, NoService, Absent, Incompatible] {
            assert_eq!(indicator(online, capability, true, true), Indicator::Hidden);
        }
        assert_eq!(Indicator::Applying.mark(), Some(MarkTone::Lit));
        assert_eq!(Indicator::Idle.mark(), Some(MarkTone::Dim));
        assert_eq!(Indicator::NoSignal.mark(), Some(MarkTone::Warn));
        assert_eq!(Indicator::Always.mark(), None);
        assert_eq!(Indicator::Hidden.label(), None);
        assert_eq!(Indicator::Idle.label(), Some("OC online: idle"));
        assert!(Indicator::toggle_available(Present));
        assert!(!Indicator::toggle_available(Absent));
    }

    #[test]
    fn mode_keys_round_trip() {
        for mode in [OcMode::Always, OcMode::Online] {
            assert_eq!(OcMode::parse(mode.key()), Some(mode));
            assert_eq!(OcMode::from_byte(mode as u8), mode);
            assert_eq!(mode.toggled().toggled(), mode);
        }
        assert_eq!(OcMode::parse("on"), None);
        for byte in 0..=5 {
            assert_eq!(Capability::from_byte(byte) as u8, byte);
        }
    }
}
