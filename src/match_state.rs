//! How a match ended (0.8.2): the melee sequence's state changes and the
//! game's own sync-communication check, observed read-only.
//!
//! The sequence is a state machine keyed by Hash40 of each state's name (CRC-32
//! of the lower-cased name | length << 32). Its command processor
//! `.text+0x135b810` reads each queued command `{u32 kind, u64 hash}`; the hook
//! at [`STATE_COMMAND_OFFSET`] sees the target hash in x26, the command in x23
//! and the previous state's hash (`machine+0x128`) in x8. Only the [`NAMES`]
//! of the melee sequence (base class `.text+0x1341d50`, online melee
//! `.text+0x14d2130`) are recorded.
//!
//! The wait-for-sync states call `.text+0x1502bd0` each update; it returns 1
//! while the sync link is up. Otherwise it stores the network component's status
//! byte at `manager+0x58` and returns 2, and the state queues
//! `StateDisconnectSyncCommunication`. The hook at [`SYNC_STATUS_OFFSET`]
//! runs only on that failure path, with the status in w0.
//!
//! Rows:
//! - 126 `MatchState`: `a` = target state id ([`NAMES`] index, bits 0–7) and
//!   command kind (8–15); `b` = the previous state's id ([`UNKNOWN`] if not one).
//! - 127 `SyncCheckFailed`: `a` = the status byte; `b` = failed checks since the
//!   last wait-for-sync state began. One row per new status.
//!
//! The match's own lockstep (`S` = its session object) carries a desync check:
//! every input record holds the XOR of all game-RNG words for its frame, and
//! `.text+0x16cf1d0` compares a peer's value with ours. A mismatch ends the
//! session with kind 2 and broadcasts an abort (control message type 7) whose
//! receiver ends at once; the match then ends through
//! `StateDisconnectSyncCommunication` with result 10, "No contest". Every
//! session end passes `.text+0x16ce7c0(S, kind)`, whose caller names the
//! trigger ([`sync_end_trigger`]).
//! - 128 `SyncEnd`: [`pack_sync_end`]; `b` = the caller's `.text` offset.
//! - 129 `DesyncSeen`: this console found a peer's check value differing from
//!   its own. `a` = ours (bits 0–31), the peer's station (32–39); `b` = the
//!   peer's value (0–31), the frame (32–47). At most eight per session.
//! - 130 `PeerAbort`: a peer's abort message arrived. `a` = the sender's station
//!   (bits 0–7), the reason (8–23, 2 = desync); `b` = the local frame.
//!
//! Sites and registers are from the 2026-10-29 static study of the 13.0.5 image
//! (`usb-desync-1029/re/RE-FINDINGS.md`, sections 3.3–3.6).

/// The melee sequence's states: the base class's, then the online melee's.
pub const NAMES: [&str; 22] = [
    "StateSetupMiiFighter",
    "StateWaitForMelee",
    "StateSetupCamera",
    "StateMelee",
    "StateWaitForRuleToFinish",
    "StateWaitForSyncWhenBeginning",
    "StateHowToPlayDescription",
    "StateAppearanceProduction",
    "StateEndProduction",
    "StateSuddenDeathPreparations",
    "StateSuddenDeathStartingProduction",
    "StateSuddenDeath",
    "StatePause",
    "StateOperatorHUDDone",
    "StateDisconnectSyncCommunication",
    "StateResetTraining",
    "StateRetryPreparations",
    "StateContinuePreparations",
    "StateSkipMelee",
    "StateResult",
    "StateApplyParameters",
    "StateWaitForSyncWhenEnding",
];
/// Not one of [`NAMES`].
pub const UNKNOWN: u64 = 0xff;

/// `.text+0x135b8bc`, `str x8, [x19, #0x130]` after the command's hash load.
pub const STATE_COMMAND_OFFSET: usize = 0x135b8bc;
pub static STATE_COMMAND_GUARD: [u8; 64] = [
    0x68, 0x9a, 0x00, 0xf9, 0xe8, 0x02, 0x40, 0xb9, 0x08, 0x05, 0x00, 0x51, 0x1f, 0x0d, 0x00, 0x71,
    0x88, 0x0d, 0x00, 0x54, 0x28, 0x7b, 0xa8, 0xb8, 0x08, 0x01, 0x19, 0x8b, 0x00, 0x01, 0x1f, 0xd6,
    0x68, 0x5e, 0x40, 0xf9, 0x69, 0x0e, 0x08, 0x8b, 0x34, 0x19, 0x40, 0xf9, 0x14, 0x12, 0x00, 0xb4,
    0x88, 0x12, 0x40, 0xb9, 0x08, 0x05, 0x00, 0x11, 0x88, 0x12, 0x00, 0xb9, 0x9b, 0x06, 0x40, 0xf9,
];
/// `.text+0x1502c60`, `and w8, w0, #0xff` after the status call.
pub const SYNC_STATUS_OFFSET: usize = 0x1502c60;
pub static SYNC_STATUS_GUARD: [u8; 64] = [
    0x08, 0x1c, 0x00, 0x12, 0x1f, 0x25, 0x00, 0x71, 0x68, 0x22, 0xc8, 0x1a, 0x09, 0x64, 0x80, 0x52,
    0x08, 0x01, 0x09, 0x0a, 0x04, 0x99, 0x40, 0x7a, 0x80, 0x62, 0x01, 0x39, 0x80, 0x04, 0x00, 0x54,
    0xc1, 0x02, 0x40, 0xf9, 0x34, 0xaf, 0x95, 0xd2, 0x74, 0x15, 0xa1, 0xf2, 0x74, 0x02, 0xc0, 0xf2,
    0xe0, 0x23, 0x00, 0x91, 0xe2, 0x03, 0x14, 0xaa, 0x4a, 0x02, 0x01, 0x94, 0xf3, 0x07, 0x40, 0xf9,
];

/// `.text+0x16ce7c0`, the lockstep session-end function's entry
/// (`sub sp, sp, #0xb0`): x0 = S, w1 = kind, x30 = the caller.
pub const SYNC_END_OFFSET: usize = 0x16ce7c0;
pub static SYNC_END_GUARD: [u8; 64] = [
    0xff, 0xc3, 0x02, 0xd1, 0xef, 0x3b, 0x01, 0x6d, 0xed, 0x33, 0x02, 0x6d, 0xeb, 0x2b, 0x03, 0x6d,
    0xe9, 0x23, 0x04, 0x6d, 0xfc, 0x6f, 0x05, 0xa9, 0xfa, 0x67, 0x06, 0xa9, 0xf8, 0x5f, 0x07, 0xa9,
    0xf6, 0x57, 0x08, 0xa9, 0xf4, 0x4f, 0x09, 0xa9, 0xfd, 0x7b, 0x0a, 0xa9, 0xfd, 0x83, 0x02, 0x91,
    0x28, 0x1c, 0x00, 0x12, 0xf3, 0x03, 0x00, 0xaa, 0x1f, 0x09, 0x00, 0x71, 0x61, 0x00, 0x00, 0x54,
];
/// `.text+0x16cf304`, `ldrb w8, [x0, #0x5f]`, reached only on a check-value
/// mismatch: w8 = ours, w9 = the peer's, w1 = its station, x2 = our record.
pub const DESYNC_OFFSET: usize = 0x16cf304;
pub static DESYNC_GUARD: [u8; 64] = [
    0x08, 0x7c, 0x41, 0x39, 0x08, 0x01, 0x00, 0x32, 0x08, 0x7c, 0x01, 0x39, 0xc0, 0x03, 0x5f, 0xd6,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x43, 0x03, 0xd1,
    0xe8, 0x33, 0x00, 0xfd, 0xfc, 0x6f, 0x07, 0xa9, 0xfa, 0x67, 0x08, 0xa9, 0xf8, 0x5f, 0x09, 0xa9,
    0xf6, 0x57, 0x0a, 0xa9, 0xf4, 0x4f, 0x0b, 0xa9, 0xfd, 0x7b, 0x0c, 0xa9, 0xfd, 0x03, 0x03, 0x91,
];
/// `.text+0x16cc2c4`, `ldrb w1, [x19, #0x1]`, the control-message dispatcher's
/// type-7 (abort) case: x19 = S, x20 = the message bytes.
pub const PEER_ABORT_OFFSET: usize = 0x16cc2c4;
pub static PEER_ABORT_GUARD: [u8; 64] = [
    0x61, 0x06, 0x40, 0x39, 0x62, 0x0e, 0x40, 0x79, 0x23, 0x03, 0x80, 0x52, 0xe0, 0x03, 0x13, 0xaa,
    0xe4, 0x03, 0x1f, 0x2a, 0x16, 0x06, 0x00, 0x94, 0xc0, 0x00, 0x00, 0x34, 0xe3, 0x03, 0x00, 0x2a,
    0x62, 0x16, 0x40, 0xf9, 0x60, 0x32, 0x40, 0xb9, 0x21, 0x10, 0x80, 0x52, 0xd4, 0x06, 0x00, 0x94,
    0x88, 0x12, 0x40, 0x78, 0x61, 0xb6, 0x40, 0x79, 0x7f, 0x9e, 0x00, 0x79, 0x7f, 0x42, 0x01, 0x79,
];
/// Mismatch rows kept per session; the compare runs for several frames.
pub const DESYNC_ROWS: u64 = 8;

/// What ended a lockstep session, from the caller of `.text+0x16ce7c0`.
pub fn sync_end_trigger(caller: u64) -> &'static str {
    match caller {
        0x16d6b08 => "desync",
        0x16cfc18 => "peer_silent",
        0x16cfb8c => "session_timeout",
        0x16cf4f4 => "station_lost_before_start",
        0x16d7908 | 0x16d7edc | 0x16d9190 => "participant_missing_at_start",
        0x16d5ec0 => "referee",
        0x1563258 => "end_judge",
        0x1569718 | 0x1568824 => "normal",
        _ => "other",
    }
}

/// Code 128's `a`: kind (bits 0–7), desync already seen (`S+0x5f` bit 0, bit
/// 8), already ending (`S+0x63` non-zero, bit 9), the local station (16–23),
/// the frame (32–47).
pub fn pack_sync_end(kind: u32, desync_seen: bool, ending: bool, local: u8, frame: u16) -> u64 {
    u64::from(kind & 0xff)
        | u64::from(desync_seen) << 8
        | u64::from(ending) << 9
        | u64::from(local) << 16
        | u64::from(frame) << 32
}

/// The game's Hash40.
pub const fn hash40(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut crc = 0xffff_ffffu32;
    let mut i = 0;
    while i < bytes.len() {
        crc ^= bytes[i].to_ascii_lowercase() as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 != 0 {
                crc >> 1 ^ 0xedb8_8320
            } else {
                crc >> 1
            };
            bit += 1;
        }
        i += 1;
    }
    (!crc) as u64 | (bytes.len() as u64 & 0xff) << 32
}

const HASHES: [u64; NAMES.len()] = {
    let mut hashes = [0; NAMES.len()];
    let mut i = 0;
    while i < NAMES.len() {
        hashes[i] = hash40(NAMES[i]);
        i += 1;
    }
    hashes
};

/// The [`NAMES`] index of a state hash.
pub fn id_of(hash: u64) -> Option<u8> {
    HASHES.iter().position(|&h| h == hash).map(|i| i as u8)
}

/// Whether the state starts a sync wait, so the next failure is reported.
pub fn begins_sync_wait(id: u8) -> bool {
    matches!(
        NAMES.get(usize::from(id)),
        Some(&"StateWaitForSyncWhenBeginning" | &"StateWaitForSyncWhenEnding")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash40_matches_the_games_constants() {
        // Stored by the static initializers at .text+0x417a04..0x417a90.
        assert_eq!(hash40("StateDisconnectSyncCommunication"), 0x20_0d38_059b);
        assert_eq!(hash40("StateResult"), 0x0b_ebc1_a983);
        assert_eq!(hash40("StateWaitForSyncWhenEnding"), 0x1a_4598_9e2f);
        assert_eq!(hash40("StateApplyParameters"), 0x14_4455_da8a);
        assert_eq!(hash40("StateSkipMelee"), 0x0e_af5c_fcdf);
        assert_eq!(hash40("StateEndProduction"), 0x12_42eb_c731);
        assert_eq!(hash40("StatePause"), 0x0a_1e5c_6b30);
        assert_eq!(hash40("StateOperatorHUDDone"), 0x14_386b_6162);
    }

    #[test]
    fn ids_name_every_state_once() {
        for (i, name) in NAMES.iter().enumerate() {
            assert_eq!(id_of(hash40(name)), Some(i as u8), "{name}");
        }
        assert_eq!(id_of(0), None);
        assert_eq!(id_of(hash40("StateLoadAsyncForStaffRoll")), None);
        assert!(begins_sync_wait(
            id_of(hash40("StateWaitForSyncWhenEnding")).unwrap()
        ));
        assert!(!begins_sync_wait(id_of(hash40("StateResult")).unwrap()));
    }

    #[test]
    fn guards_start_with_the_hooked_instructions() {
        assert_eq!(&STATE_COMMAND_GUARD[..4], &[0x68, 0x9a, 0x00, 0xf9]);
        assert_eq!(&SYNC_STATUS_GUARD[..4], &[0x08, 0x1c, 0x00, 0x12]);
        // sub sp, sp, #0xb0 / ldrb w8, [x0, #0x5f] / ldrb w1, [x19, #0x1]
        assert_eq!(&SYNC_END_GUARD[..4], &[0xff, 0xc3, 0x02, 0xd1]);
        assert_eq!(&DESYNC_GUARD[..4], &[0x08, 0x7c, 0x41, 0x39]);
        assert_eq!(&PEER_ABORT_GUARD[..4], &[0x61, 0x06, 0x40, 0x39]);
    }

    #[test]
    fn session_ends_name_their_trigger_and_pack() {
        assert_eq!(sync_end_trigger(0x16d6b08), "desync");
        assert_eq!(sync_end_trigger(0x16cfc18), "peer_silent");
        assert_eq!(sync_end_trigger(0x1569718), "normal");
        assert_eq!(sync_end_trigger(0x1234), "other");
        let a = pack_sync_end(2, true, false, 1, 0x1a2b);
        assert_eq!(a & 0xff, 2);
        assert_eq!(a >> 8 & 0b11, 0b01);
        assert_eq!(a >> 16 & 0xff, 1);
        assert_eq!(a >> 32 & 0xffff, 0x1a2b);
    }
}
