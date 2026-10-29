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
    }
}
