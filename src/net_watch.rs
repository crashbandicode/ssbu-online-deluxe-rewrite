//! Who went quiet first (0.8.2): telemetry for an online match that ends early.
//! Pia's RTT replies are the per-station evidence that packets still flow both
//! ways; the console's own `nifm` status says whether this side's link held.
//! Everything here is observed: no packet, setting or game value is written.
//!
//! Rows (all in `events.log` and the stage log):
//! - 121 `PeerLeft`: Pia's station-left hook. `a` = station (bits 0–7),
//!   accepted by the RTT table (bit 8); `b` = ms since that station's last RTT
//!   sample ([`NEVER`] if none).
//! - 122 `SessionEnded`: Pia's session cleanup. `a` = accepted (bit 0);
//!   `b` = ms since the newest RTT sample from any station.
//! - 123 `RttGap`: a station's RTT samples resumed after at least [`GAP_MS`]
//!   of silence. `a` = station; `b` = the gap in ms.
//! - 124 `LocalLink`: this console's `nifm` status changed. See [`pack_link`].
//! - 125 `RttWindow`: the RTT samples of the last 5 s, see [`Window::pack`].

/// RTT samples arrive about twice a second; a gap this long lost at least one.
pub const GAP_MS: u64 = 1_000;
/// A longer silence is a new session, not a gap inside one.
pub const SESSION_GAP_MS: u64 = 60_000;
/// Windows are written while a sample arrived within this long.
pub const ACTIVE_MS: u64 = 30_000;
/// "No sample yet" in a ms field.
pub const NEVER: u64 = 0xffff_ffff;
pub const TICKS_PER_MS: u64 = 19_200;

/// Milliseconds from `last` to `now` (19.2 MHz ticks), [`NEVER`] when `last`
/// is 0, saturated below [`NEVER`].
pub fn ms_since(last: u64, now: u64) -> u64 {
    if last == 0 {
        return NEVER;
    }
    (now.saturating_sub(last) / TICKS_PER_MS).min(NEVER - 1)
}

/// The gap worth a row when a station's sample arrives at `now`.
pub fn gap_ms(last: u64, now: u64) -> Option<u64> {
    let gap = ms_since(last, now);
    (GAP_MS..SESSION_GAP_MS).contains(&gap).then_some(gap)
}

/// One 5 s window of RTT samples from every remote station.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Window {
    pub count: u64,
    pub sum: u64,
    pub min: u64,
    pub max: u64,
    pub max_gap: u64,
}

impl Window {
    pub const EMPTY: Self = Self {
        count: 0,
        sum: 0,
        min: u64::MAX,
        max: 0,
        max_gap: 0,
    };

    /// Code 125's arguments. `a`: count (bits 0–15), min ms (16–31), max ms
    /// (32–47), mean ms (48–63), each saturated. `b`: the longest gap in ms
    /// (0–31) and ms since the newest sample at the window's end (32–63).
    pub fn pack(self, silent_ms: u64) -> (u64, u64) {
        let field = |value: u64| value.min(0xffff);
        let mean = self.sum.checked_div(self.count).unwrap_or(0);
        let min = if self.count == 0 { 0 } else { self.min };
        let a = field(self.count) | field(min) << 16 | field(self.max) << 32 | field(mean) << 48;
        (a, self.max_gap.min(NEVER) | silent_ms.min(NEVER) << 32)
    }
}

/// Code 124's arguments from `nn::nifm::GetInternetConnectionStatus`: `a` =
/// the status object's first 8 bytes (interface in byte 0: 1 Wi-Fi,
/// 2 Ethernet; Wi-Fi strength 0–3 in byte 4), `b` = the call's result (bits
/// 0–31, 0 while connected) and bytes 8–11, the connection state (32–63).
/// Kept raw so a different object layout still reads back.
pub fn pack_link(result: u32, raw: [u8; 12]) -> (u64, u64) {
    let [a0, a1, a2, a3, a4, a5, a6, a7, s0, s1, s2, s3] = raw;
    let a = u64::from_le_bytes([a0, a1, a2, a3, a4, a5, a6, a7]);
    let state = u32::from_le_bytes([s0, s1, s2, s3]);
    (a, u64::from(result) | u64::from(state) << 32)
}

/// States kept for the `net` line's path; a match passes about ten.
const PATH_CAP: usize = 24;

/// The stage log's closing `net` line, built from the rows the stage saw,
/// including the melee-sequence path and sync check from `match_state`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Summary {
    path: Vec<u8>,
    sync_status: Option<u64>,
    samples: u64,
    weighted: u64,
    min: Option<u64>,
    max: u64,
    max_gap: u64,
    gaps: u64,
    peer_left: u64,
    session_ended: u64,
    link_changes: u64,
    link_down: bool,
    last_silent: Option<u64>,
}

impl Summary {
    pub fn row(&mut self, code: u16, a: u64, b: u64) {
        use crate::diagnostics::Code;
        match code {
            c if c == Code::RttWindow as u16 => {
                let count = a & 0xffff;
                if count > 0 {
                    self.samples += count;
                    self.weighted += count * (a >> 48);
                    let min = a >> 16 & 0xffff;
                    self.min = Some(self.min.map_or(min, |m| m.min(min)));
                    self.max = self.max.max(a >> 32 & 0xffff);
                }
                self.max_gap = self.max_gap.max(b & NEVER);
                self.last_silent = Some(b >> 32);
            }
            c if c == Code::RttGap as u16 => {
                self.gaps += 1;
                self.max_gap = self.max_gap.max(b);
            }
            c if c == Code::PeerLeft as u16 => self.peer_left += 1,
            c if c == Code::SessionEnded as u16 => self.session_ended += 1,
            c if c == Code::LocalLink as u16 => {
                self.link_changes += 1;
                self.link_down = b & NEVER != 0;
            }
            c if c == Code::MatchState as u16 => {
                let id = (a & 0xff) as u8;
                if self.path.len() < PATH_CAP && self.path.last() != Some(&id) {
                    self.path.push(id);
                }
            }
            c if c == Code::SyncCheckFailed as u16 => self.sync_status = Some(a),
            _ => {}
        }
    }

    pub fn line(&self) -> String {
        let ms = |value: Option<u64>| value.map_or("none".to_string(), |v| v.to_string());
        let path: Vec<&str> = self
            .path
            .iter()
            .map(|&id| {
                crate::match_state::NAMES
                    .get(usize::from(id))
                    .map_or("?", |name| name.trim_start_matches("State"))
            })
            .collect();
        let path = if path.is_empty() {
            "none".to_string()
        } else {
            path.join(">")
        };
        format!(
            "net rtt_samples={} rtt_min_ms={} rtt_mean_ms={} rtt_max_ms={} max_gap_ms={} gaps={} peer_left={} session_ended={} link_changes={} link_down_at_end={} silent_ms_at_last_window={} sync_check_failed={} states={path}\n",
            self.samples,
            ms(self.min),
            ms(self.weighted.checked_div(self.samples)),
            self.max,
            self.max_gap,
            self.gaps,
            self.peer_left,
            self.session_ended,
            self.link_changes,
            self.link_down,
            ms(self.last_silent),
            ms(self.sync_status),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Code;

    const S: u64 = 19_200_000;

    #[test]
    fn ages_and_gaps_use_system_ticks() {
        assert_eq!(ms_since(0, 5 * S), NEVER);
        assert_eq!(ms_since(S, 3 * S), 2_000);
        assert_eq!(ms_since(3 * S, S), 0);
        assert_eq!(gap_ms(S, S + S / 2), None);
        assert_eq!(gap_ms(S, 2 * S), Some(1_000));
        assert_eq!(gap_ms(S, 90 * S), None);
        assert_eq!(gap_ms(0, S), None);
    }

    #[test]
    fn a_window_packs_its_samples_and_silence() {
        let window = Window {
            count: 10,
            sum: 10 * 23,
            min: 18,
            max: 40,
            max_gap: 1_250,
        };
        let (a, b) = window.pack(300);
        assert_eq!(a & 0xffff, 10);
        assert_eq!(a >> 16 & 0xffff, 18);
        assert_eq!(a >> 32 & 0xffff, 40);
        assert_eq!(a >> 48, 23);
        assert_eq!(b & NEVER, 1_250);
        assert_eq!(b >> 32, 300);
        let (a, b) = Window::EMPTY.pack(NEVER);
        assert_eq!(a, 0);
        assert_eq!(b >> 32, NEVER);
    }

    #[test]
    fn link_status_keeps_every_byte_and_the_result() {
        let raw = [1, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0];
        assert_eq!(pack_link(0, raw), (0x3_0000_0001, 4 << 32));
        assert_eq!(pack_link(0x2a8, [0; 12]), (0, 0x2a8));
    }

    #[test]
    fn the_summary_reads_a_stage_that_ended_on_a_leave() {
        let mut summary = Summary::default();
        let (a, b) = Window {
            count: 10,
            sum: 200,
            min: 15,
            max: 30,
            max_gap: 0,
        }
        .pack(400);
        summary.row(Code::RttWindow as u16, a, b);
        let (a, b) = Window {
            count: 6,
            sum: 6 * 40,
            min: 35,
            max: 50,
            max_gap: 1_800,
        }
        .pack(2_100);
        summary.row(Code::RttWindow as u16, a, b);
        summary.row(Code::RttGap as u16, 3, 2_500);
        summary.row(Code::PeerLeft as u16, 1 | 1 << 8, 2_400);
        let (a, b) = pack_link(0, [1, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]);
        summary.row(Code::LocalLink as u16, a, b);
        let state =
            |name| u64::from(crate::match_state::id_of(crate::match_state::hash40(name)).unwrap());
        for name in [
            "StateMelee",
            "StateWaitForSyncWhenEnding",
            "StateWaitForSyncWhenEnding",
        ] {
            summary.row(Code::MatchState as u16, state(name) | 4 << 8, 0xff);
        }
        summary.row(Code::SyncCheckFailed as u16, 8, 1);
        summary.row(
            Code::MatchState as u16,
            state("StateDisconnectSyncCommunication"),
            0,
        );
        assert_eq!(
            summary.line(),
            "net rtt_samples=16 rtt_min_ms=15 rtt_mean_ms=27 rtt_max_ms=50 max_gap_ms=2500 gaps=1 peer_left=1 session_ended=0 link_changes=1 link_down_at_end=false silent_ms_at_last_window=2100 sync_check_failed=8 states=Melee>WaitForSyncWhenEnding>DisconnectSyncCommunication\n"
        );
        let (a, b) = pack_link(0x2a8, [0; 12]);
        summary.row(Code::LocalLink as u16, a, b);
        assert!(summary.line().contains("link_down_at_end=true"));
        let empty = Summary::default().line();
        assert!(empty.contains("rtt_min_ms=none"));
        assert!(empty.ends_with("sync_check_failed=none states=none\n"));
    }
}
