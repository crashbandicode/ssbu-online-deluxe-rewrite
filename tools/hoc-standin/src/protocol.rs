//! Pure side of the stand-in: the hoc:clk protocol v1 server semantics of the
//! game-aware fork (`mgr/game_aware_policy.hpp` in intpa/horizon-oc-game-aware)
//! and the HIPC/CMIF framing hoc-clk's own `ipc_server.c` uses. No clock is
//! read or set; the "lease" only decides what the replies report.

pub const BUFFER: usize = 0x100;
pub type Message = [u8; BUFFER];

const SFCI: u32 = 0x4943_4653;
const SFCO: u32 = 0x4F43_4653;
/// CMIF message types (`CmifCommandType`).
pub const KIND_CLOSE: u32 = 2;
pub const KIND_REQUEST: u32 = 4;

pub const CMD_GET_API_VERSION: u32 = 0;
pub const CMD_INFO: u32 = 100;
pub const CMD_ARM: u32 = 101;
pub const CMD_SET_APPLY: u32 = 102;
pub const CMD_DISARM: u32 = 103;

pub const API_VERSION: u32 = 2;
pub const MAGIC: u32 = 0x4147_4F48;
pub const RESULT_GENERIC: u32 = 0x184;
pub const RESULT_NOT_OWNER: u32 = 0xCB84;
pub const RESULT_BAD_ARGS: u32 = 0xCD84;
/// The fork's `PollingIntervalMs` default, reported by cmd 100.
pub const POLL_MS: u32 = 300;
/// `nn::os::GetSystemTick` runs at 19.2 MHz.
pub const TICKS_PER_MS: u64 = 19_200;

fn get_u32(buf: &[u8], offset: usize) -> Option<u32> {
    buf.get(offset..offset + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn put_u32(buf: &mut [u8], offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

const fn align16(offset: usize) -> usize {
    (offset + 15) & !15
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    Truncated,
    BadMagic,
}

/// One received message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request<'a> {
    pub kind: u32,
    /// The kernel-filled process id when the client set `send_pid`.
    pub pid: Option<u64>,
    pub command: u32,
    pub payload: &'a [u8],
}

/// `hipcParseRequest` plus hoc-clk's CMIF header check (magic and version 0).
pub fn parse(msg: &Message) -> Result<Request<'_>, ParseError> {
    let word0 = get_u32(msg, 0).ok_or(ParseError::Truncated)?;
    let word1 = get_u32(msg, 4).ok_or(ParseError::Truncated)?;
    let mut offset = 8;
    let mut pid = None;
    if word1 >> 31 != 0 {
        let special = get_u32(msg, 8).ok_or(ParseError::Truncated)?;
        offset = 12;
        if special & 1 != 0 {
            let low = u64::from(get_u32(msg, 12).ok_or(ParseError::Truncated)?);
            let high = u64::from(get_u32(msg, 16).ok_or(ParseError::Truncated)?);
            pid = Some(low | high << 32);
            offset += 8;
        }
        offset += 4 * (((special >> 1) & 0xf) + ((special >> 5) & 0xf)) as usize;
    }
    let statics = ((word0 >> 16) & 0xf) as usize;
    let buffers = (((word0 >> 20) & 0xf) + ((word0 >> 24) & 0xf) + ((word0 >> 28) & 0xf)) as usize;
    offset += 8 * statics + 12 * buffers;
    let kind = word0 & 0xffff;
    if kind != KIND_REQUEST {
        return Ok(Request {
            kind,
            pid,
            command: 0,
            payload: &[],
        });
    }
    let end = offset + 4 * (word1 & 0x3ff) as usize;
    let start = align16(offset);
    if end > BUFFER || start + 16 > end {
        return Err(ParseError::Truncated);
    }
    if get_u32(msg, start) != Some(SFCI) || get_u32(msg, start + 4) != Some(0) {
        return Err(ParseError::BadMagic);
    }
    Ok(Request {
        kind,
        pid,
        command: get_u32(msg, start + 8).ok_or(ParseError::Truncated)?,
        payload: &msg[start + 16..end],
    })
}

/// hoc-clk's `_ipcServerPrepareResponse`: data words after the plain header,
/// the CMIF out header at the next 16-byte boundary, the payload only on
/// success.
pub fn reply(msg: &mut Message, result: u32, payload: &[u8]) {
    msg.fill(0);
    put_u32(msg, 0, KIND_REQUEST);
    put_u32(msg, 4, ((16 + payload.len() + 16) / 4) as u32);
    put_u32(msg, 16, SFCO);
    put_u32(msg, 24, result);
    if result == 0 {
        msg[32..32 + payload.len()].copy_from_slice(payload);
    }
}

/// A client-side CMIF request (sm registration), framed like libnx.
pub fn request(msg: &mut Message, command: u32, send_pid: bool, payload: &[u8]) {
    msg.fill(0);
    let words = (((16 + 16 + payload.len() + 1) & !1) as u32).div_ceil(4);
    put_u32(msg, 0, KIND_REQUEST);
    put_u32(msg, 4, words | u32::from(send_pid) << 31);
    let data = if send_pid {
        put_u32(msg, 8, 1);
        20
    } else {
        8
    };
    let start = align16(data);
    put_u32(msg, start, SFCI);
    put_u32(msg, start + 8, command);
    msg[start + 16..start + 16 + payload.len()].copy_from_slice(payload);
}

/// A client-side CMIF reply: result and the first moved handle.
pub fn decode_reply(msg: &Message) -> Result<(u32, Option<u32>), ParseError> {
    let word1 = get_u32(msg, 4).ok_or(ParseError::Truncated)?;
    let mut offset = 8;
    let mut moved = None;
    if word1 >> 31 != 0 {
        let special = get_u32(msg, 8).ok_or(ParseError::Truncated)?;
        offset = 12 + if special & 1 != 0 { 8 } else { 0 };
        let copies = ((special >> 1) & 0xf) as usize;
        let moves = ((special >> 5) & 0xf) as usize;
        offset += 4 * copies;
        if moves > 0 {
            moved = get_u32(msg, offset);
        }
        offset += 4 * moves;
    }
    let start = align16(offset);
    if get_u32(msg, start) != Some(SFCO) {
        return Err(ParseError::BadMagic);
    }
    Ok((get_u32(msg, start + 8).ok_or(ParseError::Truncated)?, moved))
}

/// A state change worth one log line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    Armed,
    ApplyOn,
    ApplyOff,
    Disarmed,
}

/// The fork's arm/lease state for its single owner.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Server {
    armed: bool,
    owner: u64,
    apply_until: u64,
    last_apply: bool,
    generation: u32,
}

impl Server {
    pub fn applying(&self, now: u64) -> bool {
        self.armed && now < self.apply_until
    }

    /// Serves one request; returns the result, the reply payload and any event.
    pub fn handle(&mut self, request: &Request, now: u64) -> (u32, Vec<u8>, Option<Event>) {
        let p = request.payload;
        match request.command {
            CMD_GET_API_VERSION => (0, API_VERSION.to_le_bytes().to_vec(), None),
            CMD_INFO => {
                let mut info = Vec::with_capacity(16);
                info.extend_from_slice(&MAGIC.to_le_bytes());
                info.extend_from_slice(&1u16.to_le_bytes());
                info.extend_from_slice(&1u16.to_le_bytes());
                info.extend_from_slice(&5000u32.to_le_bytes());
                info.extend_from_slice(&POLL_MS.to_le_bytes());
                (0, info, None)
            }
            CMD_ARM if p.len() >= 16 => {
                let Some(pid) = request.pid else {
                    return (RESULT_BAD_ARGS, Vec::new(), None);
                };
                if get_u32(p, 8) != Some(1) || get_u32(p, 12) != Some(0) {
                    return (RESULT_BAD_ARGS, Vec::new(), None);
                }
                let mut event = None;
                if !self.armed || self.owner != pid {
                    *self = Self {
                        armed: true,
                        owner: pid,
                        apply_until: 0,
                        last_apply: false,
                        generation: self.generation.wrapping_add(1),
                    };
                    event = Some(Event::Armed);
                }
                let mut out = self.generation.to_le_bytes().to_vec();
                out.extend_from_slice(&[0; 4]);
                (0, out, event)
            }
            CMD_SET_APPLY if p.len() >= 12 => {
                let Some(pid) = request.pid else {
                    return (RESULT_BAD_ARGS, Vec::new(), None);
                };
                if p[0] > 1 || p[1..4] != [0, 0, 0] {
                    return (RESULT_BAD_ARGS, Vec::new(), None);
                }
                if !self.armed || pid != self.owner {
                    return (RESULT_NOT_OWNER, Vec::new(), None);
                }
                let apply = p[0] == 1;
                let mut event = None;
                if apply != self.last_apply {
                    self.generation = self.generation.wrapping_add(1);
                    self.last_apply = apply;
                    event = Some(if apply { Event::ApplyOn } else { Event::ApplyOff });
                }
                let lease = u64::from(get_u32(p, 4).unwrap_or(0).clamp(500, 5000));
                self.apply_until = if apply { now + lease * TICKS_PER_MS } else { 0 };
                let mut out = vec![1, u8::from(self.applying(now)), 0, 0];
                out.extend_from_slice(&self.generation.to_le_bytes());
                (0, out, event)
            }
            CMD_DISARM => {
                let Some(pid) = request.pid else {
                    return (RESULT_BAD_ARGS, Vec::new(), None);
                };
                if !self.armed {
                    return (0, Vec::new(), None);
                }
                if pid != self.owner {
                    return (RESULT_NOT_OWNER, Vec::new(), None);
                }
                *self = Self {
                    generation: self.generation.wrapping_add(1),
                    ..Self::default()
                };
                (0, Vec::new(), Some(Event::Disarmed))
            }
            _ => (RESULT_GENERIC, Vec::new(), None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(buf: &Message, count: usize) -> Vec<u32> {
        (0..count).map(|i| get_u32(buf, i * 4).unwrap()).collect()
    }

    /// The client framing must equal Deluxe's golden layouts
    /// (`src/hoc_game_aware.rs` tests), so the stand-in parses what Deluxe sends.
    #[test]
    fn client_framing_matches_deluxe_golden_words() {
        let mut msg = [0; BUFFER];
        request(&mut msg, 0, true, &0u64.to_le_bytes());
        assert_eq!(
            words(&msg, 14),
            [4, 0x8000_000A, 1, 0, 0, 0, 0, 0, SFCI, 0, 0, 0, 0, 0]
        );
        request(&mut msg, CMD_INFO, false, &[]);
        assert_eq!(words(&msg, 8), [4, 8, 0, 0, SFCI, 0, 100, 0]);
    }

    fn with_pid(msg: &mut Message, pid: u64) {
        msg[12..20].copy_from_slice(&pid.to_le_bytes());
    }

    fn set_apply(apply: u8, lease: u32) -> Vec<u8> {
        let mut args = vec![apply, 0, 0, 0];
        args.extend_from_slice(&lease.to_le_bytes());
        args.extend_from_slice(&7u32.to_le_bytes());
        args
    }

    fn arm_args() -> Vec<u8> {
        let mut args = 0x0100_6A80_0016_E000u64.to_le_bytes().to_vec();
        args.extend_from_slice(&1u32.to_le_bytes());
        args.extend_from_slice(&0u32.to_le_bytes());
        args
    }

    fn serve(server: &mut Server, command: u32, pid: Option<u64>, payload: &[u8], now: u64) -> (u32, Vec<u8>, Option<Event>) {
        let mut msg = [0; BUFFER];
        request(&mut msg, command, pid.is_some(), payload);
        if let Some(pid) = pid {
            with_pid(&mut msg, pid);
        }
        let parsed = parse(&msg).unwrap();
        assert_eq!(parsed.pid, pid);
        assert_eq!(parsed.command, command);
        server.handle(&parsed, now)
    }

    #[test]
    fn handshake_answers_like_the_fork() {
        let mut server = Server::default();
        assert_eq!(serve(&mut server, 0, None, &[], 0).1, 2u32.to_le_bytes());
        let (result, info, _) = serve(&mut server, CMD_INFO, None, &[], 0);
        assert_eq!(result, 0);
        assert_eq!(&info[..4], b"HOGA");
        assert_eq!(&info[4..8], &[1, 0, 1, 0]);
        assert_eq!(serve(&mut server, 99, None, &[], 0).0, RESULT_GENERIC);
        // An Arm whose data words cannot hold the 16-byte arguments (the
        // alignment slack alone leaves 4 bytes) is malformed, as on hoc-clk.
        assert_eq!(serve(&mut server, CMD_ARM, Some(5), &[], 0).0, RESULT_GENERIC);
    }

    #[test]
    fn arm_lease_and_disarm_follow_the_fork_policy() {
        let mut server = Server::default();
        assert_eq!(serve(&mut server, CMD_ARM, None, &arm_args(), 0).0, RESULT_BAD_ARGS);
        let (result, out, event) = serve(&mut server, CMD_ARM, Some(81), &arm_args(), 0);
        assert_eq!((result, event), (0, Some(Event::Armed)));
        assert_eq!(out[..4], 1u32.to_le_bytes());
        // Re-arming the same process is idempotent.
        let (_, out, event) = serve(&mut server, CMD_ARM, Some(81), &arm_args(), 0);
        assert_eq!((out[..4].to_vec(), event), (1u32.to_le_bytes().to_vec(), None));
        // Heartbeat while off: no generation change, not applying.
        let (_, state, event) = serve(&mut server, CMD_SET_APPLY, Some(81), &set_apply(0, 3000), 10);
        assert_eq!((state.clone(), event), (vec![1, 0, 0, 0, 1, 0, 0, 0], None));
        // Apply: lease 3 s, generation 2.
        let (_, state, event) = serve(&mut server, CMD_SET_APPLY, Some(81), &set_apply(1, 3000), 100);
        assert_eq!((state, event), (vec![1, 1, 0, 0, 2, 0, 0, 0], Some(Event::ApplyOn)));
        assert!(server.applying(100 + 3000 * TICKS_PER_MS - 1));
        assert!(!server.applying(100 + 3000 * TICKS_PER_MS));
        // Lease clamps to 500..5000 ms.
        serve(&mut server, CMD_SET_APPLY, Some(81), &set_apply(1, 1), 0);
        assert!(!server.applying(500 * TICKS_PER_MS));
        serve(&mut server, CMD_SET_APPLY, Some(81), &set_apply(1, 60_000), 0);
        assert!(!server.applying(5000 * TICKS_PER_MS));
        // Another process is not the owner; bad flags are refused.
        assert_eq!(serve(&mut server, CMD_SET_APPLY, Some(9), &set_apply(1, 3000), 0).0, RESULT_NOT_OWNER);
        assert_eq!(serve(&mut server, CMD_SET_APPLY, Some(81), &set_apply(2, 3000), 0).0, RESULT_BAD_ARGS);
        assert_eq!(serve(&mut server, CMD_DISARM, Some(9), &[], 0).0, RESULT_NOT_OWNER);
        let (result, _, event) = serve(&mut server, CMD_DISARM, Some(81), &[], 0);
        assert_eq!((result, event), (0, Some(Event::Disarmed)));
        assert!(!server.applying(1));
        // Disarm when not armed succeeds; SetApply then is NotOwner.
        assert_eq!(serve(&mut server, CMD_DISARM, Some(81), &[], 0).0, 0);
        assert_eq!(serve(&mut server, CMD_SET_APPLY, Some(81), &set_apply(0, 3000), 0).0, RESULT_NOT_OWNER);
    }

    #[test]
    fn reply_framing_is_what_deluxe_decodes() {
        let mut msg = [0; BUFFER];
        reply(&mut msg, 0, &[1, 1, 0, 0, 9, 0, 0, 0]);
        assert_eq!(words(&msg, 2), [4, 10]);
        assert_eq!(get_u32(&msg, 16), Some(SFCO));
        assert_eq!(&msg[32..40], &[1, 1, 0, 0, 9, 0, 0, 0]);
        reply(&mut msg, RESULT_NOT_OWNER, &[]);
        assert_eq!(get_u32(&msg, 24), Some(RESULT_NOT_OWNER));
        assert_eq!(decode_reply(&msg), Ok((RESULT_NOT_OWNER, None)));
    }

    #[test]
    fn close_messages_and_bad_headers() {
        let mut msg = [0; BUFFER];
        put_u32(&mut msg, 0, KIND_CLOSE);
        assert_eq!(parse(&msg).unwrap().kind, KIND_CLOSE);
        request(&mut msg, 0, false, &[]);
        put_u32(&mut msg, 20, 1);
        assert_eq!(parse(&msg), Err(ParseError::BadMagic));
        put_u32(&mut msg, 4, 2);
        assert_eq!(parse(&msg), Err(ParseError::Truncated));
    }

    #[test]
    fn sm_register_service_reply_carries_the_port() {
        let mut msg = [0; BUFFER];
        put_u32(&mut msg, 4, 8 | 1 << 31);
        put_u32(&mut msg, 8, 1 << 5);
        put_u32(&mut msg, 12, 0x1234);
        put_u32(&mut msg, 16, SFCO);
        assert_eq!(decode_reply(&msg), Ok((0, Some(0x1234))));
    }
}
