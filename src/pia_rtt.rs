#![cfg_attr(not(target_os = "switch"), allow(dead_code))]

//! Read-only Pia RTT selection. Hooks copy integers into this table; it does
//! not send packets or keep a protocol pointer to dereference later.
//!
//! A different protocol is stored only when every current sample is stale, or
//! after that protocol's cleanup. The old slots are cleared on adoption. While
//! both protocols still have a fresh sample, the reading stays unavailable.
//!
//! A station index is valid only for `0..=252`. `0xFD`, `0xFE`, and `0xFF`
//! are Pia sentinels. Local is whatever the live protocol stored at `+0x38`,
//! not a fixed index 0.
//!
//! Each station keeps a 100-sample millisecond ring. The displayed value is
//! the integer mean. Variance uses only initialized samples: the closed
//! manager's warmup loop also includes one unwritten zero, and that zero is
//! intentionally omitted so a steady sample is Stable immediately. A gap of
//! three seconds resets the ring before the next push. Empty and stale
//! readings publish no stability colour.

/// Closed-manager window. Samples past this are not part of the mean.
pub const RING_CAP: usize = 100;
/// Milliseconds after which a sample is no longer a current peer.
pub const STALE_MS: u64 = 3_000;
/// A sample this far from the caller's clock is reported separately from a
/// merely expired peer. The hook clock and `GetSystemTick` are expected to
/// match; a larger gap means that assumption failed.
pub const CLOCK_SKEW_MS: u64 = 60_000;
pub const MAX_STATION_INDEX: u8 = 252;
const STATION_SLOTS: usize = MAX_STATION_INDEX as usize + 1;

/// `.text+0x15f104`, 64 bytes. Covers the signed divide through `umaddl`.
pub const RTT_ARITHMETIC_OFFSET: usize = 0x15f104;
pub static RTT_ARITHMETIC_GUARD: [u8; 64] = [
    0xf5, 0x27, 0x40, 0xf9, 0xe0, 0x43, 0x00, 0x91, 0xff, 0x0b, 0x00, 0xf9, 0x30, 0x71, 0xfd, 0x97,
    0xf6, 0x0b, 0x40, 0xf9, 0x5a, 0x71, 0xfd, 0x97, 0x68, 0x82, 0x41, 0x39, 0x9f, 0x02, 0x08, 0x6b,
    0x02, 0x01, 0x00, 0x54, 0x08, 0x00, 0x40, 0xf9, 0xc9, 0x02, 0x15, 0xcb, 0x21, 0x0d, 0xc8, 0x9a,
    0x6a, 0x2e, 0x40, 0xf9, 0x0b, 0x0b, 0x80, 0x52, 0x80, 0x2a, 0xab, 0x9b, 0xbc, 0x02, 0x00, 0x94,
];
/// `.text+0x15f13c`, the `umaddl` the sample hook replaces.
pub const RTT_SAMPLE_OFFSET: usize = 0x15f13c;
pub static RTT_SAMPLE_GUARD: [u8; 64] = [
    0x80, 0x2a, 0xab, 0x9b, 0xbc, 0x02, 0x00, 0x94, 0xfd, 0x7b, 0x47, 0xa9, 0xf4, 0x4f, 0x46, 0xa9,
    0xf6, 0x57, 0x45, 0xa9, 0xff, 0x03, 0x02, 0x91, 0xc0, 0x03, 0x5f, 0xd6, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x09, 0x80, 0x41, 0x39, 0x28, 0x1c, 0x00, 0x12, 0x1f, 0x01, 0x09, 0x6b,
    0xe2, 0x02, 0x00, 0x54, 0xff, 0x03, 0x01, 0xd1, 0xf6, 0x57, 0x01, 0xa9, 0xf4, 0x4f, 0x02, 0xa9,
];
/// `.text+0x15f240`, station-left entry.
pub const STATION_LEAVE_OFFSET: usize = 0x15f240;
pub static STATION_LEAVE_GUARD: [u8; 64] = [
    0x09, 0x80, 0x41, 0x39, 0x28, 0x1c, 0x00, 0x12, 0x1f, 0x01, 0x09, 0x6b, 0xc2, 0x00, 0x00, 0x54,
    0x08, 0x2c, 0x40, 0xf9, 0x29, 0x1c, 0x40, 0x92, 0x0a, 0x0b, 0x80, 0x52, 0x20, 0x21, 0x0a, 0x9b,
    0x3c, 0x02, 0x00, 0x14, 0xc0, 0x03, 0x5f, 0xd6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0x43, 0x01, 0xd1, 0xf7, 0x0b, 0x00, 0xf9, 0xf6, 0x57, 0x02, 0xa9, 0xf4, 0x4f, 0x03, 0xa9,
];
/// `.text+0x15ec90`, session-cleanup entry.
pub const SESSION_END_OFFSET: usize = 0x15ec90;
pub static SESSION_END_GUARD: [u8; 64] = [
    0xf5, 0x0f, 0x1d, 0xf8, 0xf4, 0x4f, 0x01, 0xa9, 0xfd, 0x7b, 0x02, 0xa9, 0xfd, 0x83, 0x00, 0x91,
    0xf3, 0x03, 0x00, 0xaa, 0x1f, 0x88, 0x01, 0x39, 0x00, 0x2c, 0x40, 0xf9, 0x20, 0x02, 0x00, 0xb4,
    0x68, 0xc2, 0x40, 0x79, 0xe8, 0x01, 0x00, 0x34, 0xa6, 0x03, 0x00, 0x94, 0x68, 0xc2, 0x40, 0x79,
    0x1f, 0x09, 0x00, 0x71, 0x63, 0x01, 0x00, 0x54, 0xf4, 0x03, 0x00, 0x32, 0x15, 0x0b, 0x80, 0x52,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Observation {
    /// Address bits of the live `RttProtocol`. Compared, never dereferenced.
    pub protocol: u64,
    pub remote: u32,
    pub local: u8,
    pub rtt_ms: i64,
    pub now_tick: u64,
    pub ticks_per_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PingReading {
    Measured(i64),
    Unavailable(PingGap),
}

/// Fork display colours. Empty and stale readings have no colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PingRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stability {
    Stable = 0,
    Inconsistent = 1,
    Unstable = 2,
}

pub const PING_STABLE: PingRgba = PingRgba {
    r: 0.45,
    g: 0.90,
    b: 0.45,
    a: 1.0,
};
pub const PING_INCONSISTENT: PingRgba = PingRgba {
    r: 0.95,
    g: 0.85,
    b: 0.25,
    a: 1.0,
};
pub const PING_UNSTABLE: PingRgba = PingRgba {
    r: 0.95,
    g: 0.35,
    b: 0.35,
    a: 1.0,
};

/// No stability and no station. `0` is a valid station, so it is not the sentinel.
pub const NO_PEER: u64 = 0xffff;

pub fn ping_rgba(stability: Stability) -> PingRgba {
    match stability {
        Stability::Stable => PING_STABLE,
        Stability::Inconsistent => PING_INCONSISTENT,
        Stability::Unstable => PING_UNSTABLE,
    }
}

/// Opponent cell. Invalid interface metadata contributes an empty suffix.
pub fn format_opponent_ping(mean: Option<u64>) -> String {
    match mean {
        Some(ms) => format!("{ms}ms"),
        None => "---".to_owned(),
    }
}

/// Performance-view cell. Shown only for one fresh peer.
pub fn format_performance_ping(mean: u64) -> String {
    format!("{mean} ms")
}

pub fn pack_peer(stability: Stability, station: u8) -> u64 {
    u64::from(stability as u8) | (u64::from(station) << 8)
}

pub fn unpack_peer(peer: u64) -> Option<(Stability, u8)> {
    let stability = match (peer & 0xff) as u8 {
        0 => Stability::Stable,
        1 => Stability::Inconsistent,
        2 => Stability::Unstable,
        _ => return None,
    };
    let station = (peer >> 8) as u8;
    if station > MAX_STATION_INDEX {
        return None;
    }
    Some((stability, station))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PingGap {
    NoneFresh,
    Stale,
    MultiplePeers,
    OtherProtocol,
    Clock,
}

/// Fixed ring. `count` is the number of initialized samples, never `count + 1`.
/// The closed manager's warmup loop also reads one never-written zero. That
/// zero is not included here: a steady sample is Stable from the first push.
#[derive(Clone, Copy)]
struct Ring {
    samples: [u32; RING_CAP],
    count: u8,
    head: u8,
    sum: u64,
}

impl Ring {
    const EMPTY: Self = Self {
        samples: [0; RING_CAP],
        count: 0,
        head: 0,
        sum: 0,
    };

    fn reset(&mut self) {
        *self = Self::EMPTY;
    }

    fn push(&mut self, sample: u32) {
        let index = usize::from(self.head);
        if usize::from(self.count) == RING_CAP {
            self.sum -= u64::from(self.samples[index]);
        }
        self.samples[index] = sample;
        self.sum += u64::from(sample);
        self.head = if index + 1 == RING_CAP {
            0
        } else {
            self.head.wrapping_add(1)
        };
        if usize::from(self.count) < RING_CAP {
            self.count += 1;
        }
    }

    fn initialized(&self) -> &[u32] {
        &self.samples[..usize::from(self.count)]
    }

    #[cfg(test)]
    fn latest(&self) -> Option<u32> {
        if self.count == 0 {
            return None;
        }
        let last = if self.head == 0 {
            RING_CAP - 1
        } else {
            usize::from(self.head) - 1
        };
        Some(self.samples[last])
    }
}

#[derive(Clone, Copy)]
struct Slot {
    ring: Ring,
    tick: u64,
    occupied: bool,
}

impl Slot {
    const EMPTY: Self = Self {
        ring: Ring::EMPTY,
        tick: 0,
        occupied: false,
    };
}

/// Integer mean of initialized samples. `None` when the window is empty.
pub fn window_mean(samples: &[u32]) -> Option<u64> {
    if samples.is_empty() {
        return None;
    }
    let sum = samples.iter().fold(0u128, |sum, sample| sum + u128::from(*sample));
    Some((sum / (samples.len() as u128)) as u64)
}

/// Sample variance of initialized samples only.
///
/// Filled windows divide by 99. Shorter windows divide by the initialized
/// count. Comparison is `var_sum <= threshold * denom` in `u128`, which matches
/// the manager thresholds without an `f64` mantissa and without wrapping a
/// negative delta. The manager's unfilled loop bound of `counter + 1` is not
/// reproduced.
pub fn window_stability(samples: &[u32]) -> Option<Stability> {
    let n = samples.len();
    if n == 0 {
        return None;
    }
    let avg = i128::from(window_mean(samples)?);
    let mut var_sum = 0u128;
    for sample in samples {
        let delta = i128::from(*sample) - avg;
        let magnitude = delta.unsigned_abs();
        var_sum += magnitude * magnitude;
    }
    let denom = if n == RING_CAP { 99u128 } else { n as u128 };
    Some(if var_sum <= 25 * denom {
        Stability::Stable
    } else if var_sum <= 100 * denom {
        Stability::Inconsistent
    } else {
        Stability::Unstable
    })
}

#[derive(Clone)]
pub struct Table {
    protocol: u64,
    local: u8,
    ticks_per_ms: u64,
    slots: [Slot; STATION_SLOTS],
    foreign_protocol: u64,
    foreign_tick: u64,
    foreign_per_ms: u64,
    pub samples: u64,
    pub rejected_negative: u64,
    pub rejected_index: u64,
    pub rejected_overflow: u64,
    pub foreign_samples: u64,
    pub leaves: u64,
    pub cleanups: u64,
    last_tick: u64,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    pub const fn new() -> Self {
        Self {
            protocol: 0,
            local: 0xfd,
            ticks_per_ms: 0,
            slots: [Slot::EMPTY; STATION_SLOTS],
            foreign_protocol: 0,
            foreign_tick: 0,
            foreign_per_ms: 0,
            samples: 0,
            rejected_negative: 0,
            rejected_index: 0,
            rejected_overflow: 0,
            foreign_samples: 0,
            leaves: 0,
            cleanups: 0,
            last_tick: 0,
        }
    }

    /// `true` when a different protocol replaced a fully stale table and the
    /// slots were cleared. Callers must not treat that as a named identity.
    pub fn observe_rtt(&mut self, sample: Observation) -> bool {
        if sample.protocol == 0 || sample.ticks_per_ms == 0 {
            self.rejected_index += 1;
            return false;
        }
        let Some(remote) = station_index(sample.remote) else {
            self.rejected_index += 1;
            return false;
        };
        if station_index(u32::from(sample.local)).is_none() {
            self.rejected_index += 1;
            return false;
        }
        if sample.rtt_ms < 0 {
            self.rejected_negative += 1;
            return false;
        }
        let Ok(rtt_ms) = u32::try_from(sample.rtt_ms) else {
            self.rejected_overflow += 1;
            return false;
        };
        if remote == sample.local {
            return false;
        }
        let mut adopted = false;
        if self.protocol == 0 {
            self.protocol = sample.protocol;
            if self.foreign_protocol == sample.protocol {
                self.clear_foreign();
            }
        } else if self.protocol != sample.protocol {
            if !self.all_current_stale(sample.now_tick) {
                self.foreign_protocol = sample.protocol;
                self.foreign_tick = sample.now_tick;
                self.foreign_per_ms = sample.ticks_per_ms;
                self.foreign_samples += 1;
                return false;
            }
            self.clear_slots();
            self.protocol = sample.protocol;
            self.local = 0xfd;
            self.clear_foreign();
            adopted = true;
        }
        self.local = sample.local;
        let reset = {
            let slot = &self.slots[usize::from(remote)];
            slot.occupied
                && self.ticks_per_ms != 0
                && matches!(
                    age_ms(sample.now_tick, slot.tick, self.ticks_per_ms),
                    Age::Ms(age) if age >= STALE_MS
                )
        };
        if reset {
            let slot = &mut self.slots[usize::from(remote)];
            slot.ring.reset();
            slot.occupied = false;
        }
        self.ticks_per_ms = sample.ticks_per_ms;
        let slot = &mut self.slots[usize::from(remote)];
        slot.ring.push(rtt_ms);
        slot.tick = sample.now_tick;
        slot.occupied = true;
        self.samples += 1;
        self.last_tick = sample.now_tick;
        adopted
    }

    fn clear_slots(&mut self) {
        self.slots.fill(Slot::EMPTY);
    }

    /// True when every stored sample is older than [`STALE_MS`], or none remain.
    fn all_current_stale(&self, now_tick: u64) -> bool {
        !self.slots.iter().any(|slot| {
            slot.occupied
                && matches!(
                    age_ms(now_tick, slot.tick, self.ticks_per_ms),
                    Age::Ms(age) if age < STALE_MS
                )
        })
    }

    fn fresh_stamp(&self, now_tick: u64) -> (u64, u64) {
        for index in 0..=MAX_STATION_INDEX {
            let slot = &self.slots[usize::from(index)];
            if !slot.occupied || index == self.local {
                continue;
            }
            if matches!(
                age_ms(now_tick, slot.tick, self.ticks_per_ms),
                Age::Ms(age) if age < STALE_MS
            ) {
                return (slot.tick, self.ticks_per_ms);
            }
        }
        (0, 0)
    }

    pub fn station_left(&mut self, protocol: u64, remote: u32) -> bool {
        let Some(remote) = station_index(remote) else {
            return false;
        };
        if protocol == 0 || protocol != self.protocol {
            return false;
        }
        self.slots[usize::from(remote)] = Slot::EMPTY;
        self.leaves += 1;
        true
    }

    pub fn session_end(&mut self, protocol: u64) -> bool {
        if protocol == 0 {
            return false;
        }
        if protocol == self.foreign_protocol {
            self.clear_foreign();
        }
        if protocol != self.protocol {
            return false;
        }
        self.protocol = 0;
        self.local = 0xfd;
        self.ticks_per_ms = 0;
        self.last_tick = 0;
        self.clear_slots();
        self.cleanups += 1;
        true
    }

    pub fn reading(&self, now_tick: u64) -> PingReading {
        if self.foreign_fresh(now_tick) {
            return PingReading::Unavailable(PingGap::OtherProtocol);
        }
        let mut fresh_count = 0u16;
        let mut fresh_rtt = 0i64;
        let mut occupied = 0u16;
        let mut freshest = u64::MAX;
        let mut backwards = false;
        for index in 0..=MAX_STATION_INDEX {
            let slot = &self.slots[usize::from(index)];
            if !slot.occupied {
                continue;
            }
            occupied += 1;
            match age_ms(now_tick, slot.tick, self.ticks_per_ms) {
                Age::Backwards => backwards = true,
                Age::Unknown => {}
                Age::Ms(age) => {
                    freshest = freshest.min(age);
                    if age < STALE_MS && index != self.local {
                        fresh_count += 1;
                        fresh_rtt = window_mean(slot.ring.initialized())
                            .map(|mean| mean as i64)
                            .unwrap_or(0);
                    }
                }
            }
        }
        if fresh_count > 1 {
            return PingReading::Unavailable(PingGap::MultiplePeers);
        }
        if fresh_count == 1 {
            return PingReading::Measured(fresh_rtt);
        }
        if occupied == 0 {
            return PingReading::Unavailable(PingGap::NoneFresh);
        }
        if backwards || freshest >= CLOCK_SKEW_MS {
            return PingReading::Unavailable(PingGap::Clock);
        }
        PingReading::Unavailable(PingGap::Stale)
    }

    fn fresh_peer(&self, now_tick: u64) -> Option<(Stability, u8)> {
        let mut found = None;
        for index in 0..=MAX_STATION_INDEX {
            let slot = &self.slots[usize::from(index)];
            if !slot.occupied || index == self.local {
                continue;
            }
            if matches!(
                age_ms(now_tick, slot.tick, self.ticks_per_ms),
                Age::Ms(age) if age < STALE_MS
            ) {
                if found.is_some() {
                    return None;
                }
                found = Some((window_stability(slot.ring.initialized())?, index));
            }
        }
        found
    }

    fn clear(&mut self) {
        self.protocol = 0;
        self.local = 0xfd;
        self.ticks_per_ms = 0;
        self.clear_slots();
        self.foreign_protocol = 0;
        self.foreign_tick = 0;
        self.foreign_per_ms = 0;
        self.samples = 0;
        self.rejected_negative = 0;
        self.rejected_index = 0;
        self.rejected_overflow = 0;
        self.foreign_samples = 0;
        self.leaves = 0;
        self.cleanups = 0;
        self.last_tick = 0;
    }

    fn foreign_fresh(&self, now_tick: u64) -> bool {
        self.foreign_protocol != 0
            && matches!(
                age_ms(now_tick, self.foreign_tick, self.foreign_per_ms),
                Age::Ms(age) if age < STALE_MS
            )
    }

    fn clear_foreign(&mut self) {
        self.foreign_protocol = 0;
        self.foreign_tick = 0;
        self.foreign_per_ms = 0;
    }
}

/// `0..=252`. Values above that include the `0xFD` / `0xFE` / `0xFF` sentinels.
pub fn station_index(raw: u32) -> Option<u8> {
    let Ok(index) = u8::try_from(raw) else {
        return None;
    };
    if index > MAX_STATION_INDEX {
        None
    } else {
        Some(index)
    }
}

enum Age {
    Ms(u64),
    Backwards,
    Unknown,
}

pub const REASON_NONE: u8 = 0;
pub const REASON_MEASURED: u8 = 1;
pub const REASON_MULTIPLE: u8 = 2;
pub const REASON_OTHER: u8 = 3;
pub const REASON_STALE: u8 = 4;
pub const REASON_OFF: u8 = 5;
pub const REASON_CLOCK: u8 = 6;

/// One UI publication. `bits` packs the reading and the reason together.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Publication {
    pub bits: u64,
    pub tick: u64,
    pub ticks_per_ms: u64,
    pub epoch: u64,
    /// Stability in the low byte and station index in the next byte.
    /// [`NO_PEER`] means there is no colour and no opponent identity.
    pub peer: u64,
}

impl Publication {
    pub fn unavailable(epoch: u64) -> Self {
        Self::from_reading(
            PingReading::Unavailable(PingGap::NoneFresh),
            0,
            0,
            epoch,
            NO_PEER,
        )
    }

    pub fn from_reading(
        reading: PingReading,
        tick: u64,
        ticks_per_ms: u64,
        epoch: u64,
        peer: u64,
    ) -> Self {
        Self {
            bits: pack_reading(reading),
            tick,
            ticks_per_ms,
            epoch,
            peer,
        }
    }

    pub fn reading(self) -> PingReading {
        unpack_reading(self.bits)
    }
}

pub fn detail_text(reason: u8) -> &'static str {
    match reason {
        REASON_MEASURED => "1 peer",
        REASON_MULTIPLE => "multiple peers",
        REASON_OTHER => "other session",
        REASON_STALE => "stale",
        REASON_CLOCK => "clock",
        REASON_OFF => "hooks not installed",
        _ => "no peer",
    }
}

pub fn pack_reading(reading: PingReading) -> u64 {
    match reading {
        PingReading::Measured(ms) if ms >= 0 => {
            let magnitude = (ms as u64).min((1u64 << 55) - 1);
            u64::from(REASON_MEASURED) | (magnitude << 8)
        }
        PingReading::Measured(_) => (1u64 << 63) | u64::from(REASON_CLOCK),
        PingReading::Unavailable(gap) => (1u64 << 63) | u64::from(reason_of(gap)),
    }
}

pub fn unpack_reading(bits: u64) -> PingReading {
    if bits & (1u64 << 63) != 0 {
        PingReading::Unavailable(gap_of((bits & 0xff) as u8))
    } else {
        PingReading::Measured((bits >> 8) as i64)
    }
}

fn reason_of(gap: PingGap) -> u8 {
    match gap {
        PingGap::NoneFresh => REASON_NONE,
        PingGap::MultiplePeers => REASON_MULTIPLE,
        PingGap::OtherProtocol => REASON_OTHER,
        PingGap::Stale => REASON_STALE,
        PingGap::Clock => REASON_CLOCK,
    }
}

fn gap_of(reason: u8) -> PingGap {
    match reason {
        REASON_MULTIPLE => PingGap::MultiplePeers,
        REASON_OTHER => PingGap::OtherProtocol,
        REASON_STALE => PingGap::Stale,
        REASON_CLOCK => PingGap::Clock,
        _ => PingGap::NoneFresh,
    }
}

/// Table plus the epoch that makes a missed leave or cleanup discard old slots.
#[derive(Clone)]
pub struct Session {
    table: Table,
    applied: u64,
    publication: Publication,
}

impl Session {
    pub const fn new() -> Self {
        Self {
            table: Table::new(),
            applied: 1,
            publication: Publication {
                bits: (1u64 << 63) | (REASON_OFF as u64),
                tick: 0,
                ticks_per_ms: 0,
                epoch: 1,
                peer: NO_PEER,
            },
        }
    }

    pub fn publication(&self) -> Publication {
        self.publication
    }

    #[cfg(test)]
    fn stored_rtt(&self, index: u8) -> Option<i64> {
        let slot = self.table.slots[usize::from(index)];
        if slot.occupied {
            slot.ring.latest().map(i64::from)
        } else {
            None
        }
    }

    pub fn samples(&self) -> u64 {
        self.table.samples
    }

    pub fn foreign_samples(&self) -> u64 {
        self.table.foreign_samples
    }

    pub fn cleanups(&self) -> u64 {
        self.table.cleanups
    }

    pub fn accept_sample(&mut self, sample: Observation, epoch: u64) -> bool {
        self.reconcile(epoch);
        let adopted = self.table.observe_rtt(sample);
        self.publish(sample.now_tick, epoch);
        adopted
    }

    pub fn accept_leave(&mut self, protocol: u64, remote: u32, epoch: u64) -> bool {
        self.reconcile(epoch);
        let cleared = self.table.station_left(protocol, remote);
        self.publish(self.table.last_tick, epoch);
        cleared
    }

    pub fn accept_cleanup(&mut self, protocol: u64, epoch: u64) -> bool {
        self.reconcile(epoch);
        let cleared = self.table.session_end(protocol);
        self.publish(self.table.last_tick, epoch);
        cleared
    }

    pub fn accept_poll(&mut self, now_tick: u64, epoch: u64) {
        self.reconcile(epoch);
        self.publish(now_tick, epoch);
    }

    fn reconcile(&mut self, epoch: u64) {
        if self.applied == epoch {
            return;
        }
        self.table.clear();
        self.applied = epoch;
        self.publication = Publication::unavailable(epoch);
    }

    /// `clock` only decides which samples are still fresh. A measurement keeps
    /// the RTT sample's own tick, so a later poll cannot make that sample new.
    fn publish(&mut self, clock: u64, epoch: u64) {
        let reading = self.table.reading(clock);
        let (tick, ticks_per_ms, peer) = match reading {
            PingReading::Measured(_) => {
                let (tick, ticks_per_ms) = self.table.fresh_stamp(clock);
                let peer = self
                    .table
                    .fresh_peer(clock)
                    .map(|(stability, station)| pack_peer(stability, station))
                    .unwrap_or(NO_PEER);
                (tick, ticks_per_ms, peer)
            }
            PingReading::Unavailable(_) => (0, 0, NO_PEER),
        };
        self.publication = Publication::from_reading(reading, tick, ticks_per_ms, epoch, peer);
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Age a publication when the table lock is unavailable. A missed lifecycle
/// epoch forces unavailable. A measured value expires at [`STALE_MS`].
pub fn expire_publication(published: Publication, now_tick: u64, epoch: u64) -> Publication {
    if published.epoch != epoch {
        return Publication::unavailable(epoch);
    }
    if !matches!(published.reading(), PingReading::Measured(_)) {
        return published;
    }
    match age_ms(now_tick, published.tick, published.ticks_per_ms) {
        Age::Ms(age) if age < STALE_MS => published,
        Age::Ms(_) => Publication::from_reading(
            PingReading::Unavailable(PingGap::Stale),
            0,
            0,
            epoch,
            NO_PEER,
        ),
        _ => Publication::from_reading(
            PingReading::Unavailable(PingGap::Clock),
            0,
            0,
            epoch,
            NO_PEER,
        ),
    }
}

/// One draw's ping cells. Colour is present only for a measured peer.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentedPing {
    pub opponent: String,
    pub performance: Option<String>,
    pub color: Option<PingRgba>,
    pub detail: &'static str,
}

impl PresentedPing {
    pub fn from_publication(publication: Publication) -> Self {
        let detail = detail_text((publication.bits & 0xff) as u8);
        match publication.reading() {
            PingReading::Measured(ms) if ms >= 0 => {
                let mean = ms as u64;
                let color = unpack_peer(publication.peer).map(|(stability, _)| ping_rgba(stability));
                Self {
                    opponent: format_opponent_ping(Some(mean)),
                    performance: Some(format_performance_ping(mean)),
                    color,
                    detail,
                }
            }
            _ => Self {
                opponent: format_opponent_ping(None),
                performance: None,
                color: None,
                detail,
            },
        }
    }
}

fn age_ms(now_tick: u64, sample_tick: u64, ticks_per_ms: u64) -> Age {
    if ticks_per_ms == 0 {
        return Age::Unknown;
    }
    if now_tick < sample_tick {
        return Age::Backwards;
    }
    Age::Ms(now_tick.saturating_sub(sample_tick) / ticks_per_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PER_MS: u64 = 1_000;
    const PROTOCOL: u64 = 0x1000;

    fn obs(remote: u32, local: u8, rtt_ms: i64, now_tick: u64) -> Observation {
        Observation {
            protocol: PROTOCOL,
            remote,
            local,
            rtt_ms,
            now_tick,
            ticks_per_ms: PER_MS,
        }
    }

    #[test]
    fn local_peer_is_excluded_and_index_zero_can_be_the_opponent() {
        let mut table = Table::new();
        table.observe_rtt(obs(4, 4, 12, 0));
        table.observe_rtt(obs(0, 4, 9, 0));
        assert_eq!(table.reading(0), PingReading::Measured(9));
    }

    #[test]
    fn zero_rtt_is_a_measurement() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 0, 0));
        assert_eq!(table.reading(0), PingReading::Measured(0));
    }

    #[test]
    fn negative_rtt_is_rejected_without_clearing_a_prior_sample() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 5, 0));
        table.observe_rtt(obs(2, 1, -1, PER_MS));
        assert_eq!(table.rejected_negative, 1);
        assert_eq!(table.reading(PER_MS), PingReading::Measured(5));
    }

    #[test]
    fn sample_expires_at_three_seconds() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 15, 0));
        let fresh = STALE_MS * PER_MS - 1;
        let stale = STALE_MS * PER_MS;
        assert_eq!(table.reading(fresh), PingReading::Measured(15));
        assert_eq!(
            table.reading(stale),
            PingReading::Unavailable(PingGap::Stale)
        );
    }

    #[test]
    fn leave_clears_an_index_and_the_next_sample_reuses_it() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 10, 0));
        table.station_left(PROTOCOL, 2);
        assert_eq!(table.reading(0), PingReading::Unavailable(PingGap::NoneFresh));
        table.observe_rtt(obs(2, 1, 4, PER_MS));
        assert_eq!(table.reading(PER_MS), PingReading::Measured(4));
    }

    #[test]
    fn session_cleanup_drops_peers_and_a_new_protocol_starts_empty() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 10, 0));
        table.observe_rtt(obs(3, 1, 11, 0));
        table.session_end(PROTOCOL);
        assert_eq!(table.reading(0), PingReading::Unavailable(PingGap::NoneFresh));
        table.observe_rtt(Observation {
            protocol: PROTOCOL + 1,
            remote: 6,
            local: 5,
            rtt_ms: 7,
            now_tick: PER_MS,
            ticks_per_ms: PER_MS,
        });
        assert_eq!(table.reading(PER_MS), PingReading::Measured(7));
        assert_eq!(table.cleanups, 1);
    }

    #[test]
    fn two_fresh_peers_are_not_labeled_as_one_opponent() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 10, 0));
        table.observe_rtt(obs(3, 1, 40, 0));
        assert_eq!(
            table.reading(0),
            PingReading::Unavailable(PingGap::MultiplePeers)
        );
        table.station_left(PROTOCOL, 3);
        assert_eq!(table.reading(0), PingReading::Measured(10));
    }

    #[test]
    fn another_live_protocol_hides_the_current_peer_until_it_expires() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 10, 0));
        table.observe_rtt(Observation {
            protocol: PROTOCOL + 8,
            remote: 9,
            local: 1,
            rtt_ms: 80,
            now_tick: 0,
            ticks_per_ms: PER_MS,
        });
        assert_eq!(table.samples, 1);
        assert_eq!(
            table.reading(0),
            PingReading::Unavailable(PingGap::OtherProtocol)
        );
        let edge = STALE_MS * PER_MS;
        table.observe_rtt(obs(2, 1, 10, edge));
        assert_eq!(table.reading(edge), PingReading::Measured(10));
    }

    #[test]
    fn sentinels_are_rejected_and_252_is_kept() {
        assert_eq!(station_index(252), Some(252));
        assert_eq!(station_index(0xfd), None);
        assert_eq!(station_index(0xfe), None);
        assert_eq!(station_index(0xff), None);
        let mut table = Table::new();
        table.observe_rtt(obs(0xfd, 1, 5, 0));
        table.observe_rtt(obs(252, 1, 6, 0));
        assert_eq!(table.rejected_index, 1);
        assert_eq!(table.reading(0), PingReading::Measured(6));
    }

    #[test]
    fn cleanup_of_a_different_protocol_keeps_the_current_peer() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 10, 0));
        table.session_end(PROTOCOL + 3);
        assert_eq!(table.reading(0), PingReading::Measured(10));
        assert_eq!(table.cleanups, 0);
    }

    #[test]
    fn guard_windows_are_the_verified_nso_bytes() {
        assert_eq!(RTT_ARITHMETIC_GUARD.len(), 64);
        assert_eq!(&RTT_ARITHMETIC_GUARD[56..], &RTT_SAMPLE_GUARD[..8]);
        assert_eq!(&RTT_SAMPLE_GUARD[..4], &[0x80, 0x2a, 0xab, 0x9b]);
        assert_eq!(&STATION_LEAVE_GUARD[..4], &[0x09, 0x80, 0x41, 0x39]);
        assert_eq!(&SESSION_END_GUARD[..4], &[0xf5, 0x0f, 0x1d, 0xf8]);
        assert_eq!(RTT_SAMPLE_OFFSET - RTT_ARITHMETIC_OFFSET, 0x38);
    }

    fn at(protocol: u64, remote: u32, local: u8, rtt_ms: i64, now_tick: u64) -> Observation {
        Observation {
            protocol,
            remote,
            local,
            rtt_ms,
            now_tick,
            ticks_per_ms: PER_MS,
        }
    }

    #[test]
    fn background_then_continuing_match_adopts_only_after_the_first_is_stale() {
        let mut session = Session::new();
        let epoch = 1;
        let background = PROTOCOL;
        let match_protocol = PROTOCOL + 4;
        session.accept_sample(at(background, 2, 1, 50, 0), epoch);
        session.accept_sample(at(background, 3, 1, 60, 0), epoch);
        session.accept_sample(at(match_protocol, 7, 5, 12, PER_MS), epoch);
        assert_eq!(
            session.publication().reading(),
            PingReading::Unavailable(PingGap::OtherProtocol)
        );
        assert_eq!(session.stored_rtt(2), Some(50));
        let later = STALE_MS * PER_MS;
        session.accept_sample(at(match_protocol, 7, 5, 12, later), epoch);
        assert_eq!(session.publication().reading(), PingReading::Measured(12));
        assert_eq!(session.stored_rtt(2), None);
        assert_eq!(session.stored_rtt(3), None);
        assert_eq!(session.stored_rtt(7), Some(12));
    }

    #[test]
    fn adopting_a_stale_protocol_reports_the_slot_clear() {
        let mut session = Session::new();
        assert!(!session.accept_sample(at(PROTOCOL, 2, 1, 10, 0), 1));
        assert!(!session.accept_sample(at(PROTOCOL, 2, 1, 11, PER_MS), 1));
        assert!(!session.accept_sample(at(PROTOCOL + 1, 4, 1, 12, PER_MS), 1));
        assert_eq!(session.stored_rtt(2), Some(11));
        assert!(session.accept_sample(
            at(PROTOCOL + 1, 4, 1, 12, (STALE_MS + 1) * PER_MS),
            1
        ));
        assert_eq!(session.stored_rtt(2), None);
        assert_eq!(session.stored_rtt(4), Some(12));
    }

    #[test]
    fn alternating_live_protocols_never_pick_one() {
        let mut session = Session::new();
        let epoch = 1;
        let primary = PROTOCOL;
        let other = PROTOCOL + 9;
        session.accept_sample(at(primary, 2, 1, 10, 0), epoch);
        assert_eq!(session.publication().reading(), PingReading::Measured(10));
        for step in 1..6u64 {
            let now = step * 200 * PER_MS;
            let sample = if step % 2 == 0 {
                at(primary, 2, 1, 10, now)
            } else {
                at(other, 8, 4, 90, now)
            };
            session.accept_sample(sample, epoch);
            assert_eq!(
                session.publication().reading(),
                PingReading::Unavailable(PingGap::OtherProtocol)
            );
            assert_eq!(session.stored_rtt(8), None);
        }
    }

    #[test]
    fn missed_lifecycle_drops_the_old_peer_until_a_new_sample() {
        let mut session = Session::new();
        let mut epoch = 1u64;
        session.accept_sample(at(PROTOCOL, 2, 1, 40, 0), epoch);
        assert_eq!(session.publication().reading(), PingReading::Measured(40));
        epoch += 1;
        session.accept_poll(500 * PER_MS, epoch);
        assert_eq!(session.stored_rtt(2), None);
        assert_eq!(
            session.publication().reading(),
            PingReading::Unavailable(PingGap::NoneFresh)
        );
        session.accept_sample(at(PROTOCOL, 2, 1, 4, 600 * PER_MS), epoch);
        assert_eq!(session.publication().reading(), PingReading::Measured(4));
    }

    #[test]
    fn poll_without_the_table_expires_a_measured_value() {
        let mut session = Session::new();
        session.accept_sample(at(PROTOCOL, 2, 1, 40, 0), 1);
        let published = session.publication();
        assert_eq!(
            expire_publication(published, STALE_MS * PER_MS - 1, 1).reading(),
            PingReading::Measured(40)
        );
        let expired = expire_publication(published, STALE_MS * PER_MS, 1);
        assert_eq!(
            expired.reading(),
            PingReading::Unavailable(PingGap::Stale)
        );
        assert_eq!(
            expire_publication(expired, STALE_MS * PER_MS * 4, 1).reading(),
            PingReading::Unavailable(PingGap::Stale)
        );
        assert_eq!(
            expire_publication(published, PER_MS, 2).reading(),
            PingReading::Unavailable(PingGap::NoneFresh)
        );
    }

    #[test]
    fn repeated_poll_keeps_the_sample_tick_until_the_reader_expires_it() {
        let mut session = Session::new();
        let sample_tick = 50 * PER_MS;
        session.accept_sample(at(PROTOCOL, 2, 1, 33, sample_tick), 1);
        assert_eq!(session.publication().tick, sample_tick);
        let still_fresh = sample_tick + (STALE_MS - 1) * PER_MS;
        session.accept_poll(still_fresh, 1);
        assert_eq!(session.publication().tick, sample_tick);
        assert_eq!(session.publication().reading(), PingReading::Measured(33));
        let stale_at = sample_tick + STALE_MS * PER_MS;
        assert_eq!(
            expire_publication(session.publication(), stale_at, 1).reading(),
            PingReading::Unavailable(PingGap::Stale)
        );
        session.accept_poll(stale_at, 1);
        assert_ne!(session.publication().tick, stale_at);
        assert_eq!(
            session.publication().reading(),
            PingReading::Unavailable(PingGap::Stale)
        );
        session.accept_poll(stale_at + 10 * PER_MS, 1);
        assert_eq!(
            session.publication().reading(),
            PingReading::Unavailable(PingGap::Stale)
        );
    }

    #[test]
    fn steady_twenty_is_stable_from_the_first_sample() {
        assert_eq!(window_stability(&[20]), Some(Stability::Stable));
        assert_eq!(window_mean(&[20]), Some(20));
        let steady = [20u32; 16];
        assert_eq!(window_stability(&steady), Some(Stability::Stable));
        assert_eq!(window_mean(&steady), Some(20));
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 20, 0));
        assert_eq!(table.reading(0), PingReading::Measured(20));
        assert_eq!(table.fresh_peer(0), Some((Stability::Stable, 2)));
    }

    #[test]
    fn variance_boundaries_use_initialized_samples_and_signed_deltas() {
        let mut alternating = [0u32; 100];
        for (index, sample) in alternating.iter_mut().enumerate() {
            *sample = if index % 2 == 0 { 45 } else { 55 };
        }
        assert_eq!(window_mean(&alternating), Some(50));
        assert_eq!(
            window_stability(&alternating),
            Some(Stability::Inconsistent)
        );
        for (index, sample) in alternating.iter_mut().enumerate() {
            *sample = if index % 2 == 0 { 40 } else { 60 };
        }
        assert_eq!(window_stability(&alternating), Some(Stability::Unstable));
        assert_eq!(window_stability(&[45, 55]), Some(Stability::Stable));
        assert_eq!(window_stability(&[10, 30]), Some(Stability::Inconsistent));
        assert_eq!(
            window_stability(&[0, u32::MAX]),
            Some(Stability::Unstable)
        );
        assert!(window_stability(&[]).is_none());
    }

    #[test]
    fn ring_wrap_expiry_leave_and_protocol_reset_drop_old_history() {
        let mut table = Table::new();
        for index in 0..100 {
            table.observe_rtt(obs(2, 1, 5, index * PER_MS));
        }
        assert_eq!(table.reading(99 * PER_MS), PingReading::Measured(5));
        table.observe_rtt(obs(2, 1, 105, 99 * PER_MS));
        assert_eq!(table.reading(99 * PER_MS), PingReading::Measured(6));
        let stale_push = 99 * PER_MS + STALE_MS * PER_MS;
        table.observe_rtt(obs(2, 1, 10, stale_push));
        assert_eq!(table.reading(stale_push), PingReading::Measured(10));
        table.station_left(PROTOCOL, 2);
        assert_eq!(
            table.reading(stale_push),
            PingReading::Unavailable(PingGap::NoneFresh)
        );
        table.observe_rtt(obs(2, 1, 4, stale_push));
        assert_eq!(table.reading(stale_push), PingReading::Measured(4));
        table.session_end(PROTOCOL);
        table.observe_rtt(Observation {
            protocol: PROTOCOL + 1,
            remote: 6,
            local: 5,
            rtt_ms: 8,
            now_tick: stale_push,
            ticks_per_ms: PER_MS,
        });
        assert_eq!(table.reading(stale_push), PingReading::Measured(8));
        assert_eq!(table.fresh_peer(stale_push), Some((Stability::Stable, 6)));
    }

    #[test]
    fn overflow_is_rejected_and_a_stale_snapshot_has_no_colour() {
        let mut session = Session::new();
        session.accept_sample(at(PROTOCOL, 2, 1, 20, 0), 1);
        session.accept_sample(at(PROTOCOL, 2, 1, i64::MAX, PER_MS), 1);
        assert_eq!(session.publication().reading(), PingReading::Measured(20));
        assert_eq!(session.table.rejected_overflow, 1);
        let presented = PresentedPing::from_publication(session.publication());
        assert_eq!(presented.opponent, "20ms");
        assert_eq!(presented.performance.as_deref(), Some("20 ms"));
        assert_eq!(presented.color, Some(PING_STABLE));
        assert_eq!(ping_rgba(Stability::Inconsistent), PING_INCONSISTENT);
        assert_eq!(ping_rgba(Stability::Unstable), PING_UNSTABLE);
        let expired = expire_publication(session.publication(), STALE_MS * PER_MS, 1);
        let stale = PresentedPing::from_publication(expired);
        assert_eq!(stale.opponent, "---");
        assert_eq!(stale.performance, None);
        assert_eq!(stale.color, None);
        assert_eq!(expired.peer, NO_PEER);
    }
}
