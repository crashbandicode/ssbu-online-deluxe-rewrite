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
//!
//! The table holds [`STATION_CAP`] stations keyed by index (twice the
//! handshake table; arenas hold eight). A new station takes a free slot, else
//! the one with the oldest sample.

/// Closed-manager window. Samples past this are not part of the mean.
pub const RING_CAP: usize = 100;
/// Milliseconds after which a sample is no longer a current peer.
pub const STALE_MS: u64 = 3_000;
/// A sample this far from the caller's clock is reported separately from a
/// merely expired peer. The hook clock and `GetSystemTick` are expected to
/// match; a larger gap means that assumption failed.
pub const CLOCK_SKEW_MS: u64 = 60_000;
pub const MAX_STATION_INDEX: u8 = 252;
/// Stations kept at once.
pub const STATION_CAP: usize = 16;

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

/// Why there is no measured ping. The discriminant is the published reason
/// byte; `1` is a measurement.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PingGap {
    NoneFresh = 0,
    MultiplePeers = 2,
    OtherProtocol = 3,
    Stale = 4,
    /// The observer hooks are not installed.
    Off = 5,
    Clock = 6,
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

/// No stability and no station. `0` is a valid station, so it is not the sentinel.
pub const NO_PEER: u64 = 0xffff;

pub fn ping_rgba(stability: Stability) -> PingRgba {
    let (r, g, b) = match stability {
        Stability::Stable => (0.45, 0.90, 0.45),
        Stability::Inconsistent => (0.95, 0.85, 0.25),
        Stability::Unstable => (0.95, 0.35, 0.35),
    };
    PingRgba { r, g, b, a: 1.0 }
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
    (station <= MAX_STATION_INDEX).then_some((stability, station))
}

/// Which station a draw is allowed to attach a published ping to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RttIdentity {
    /// Mapped remote peer. The publication must name this index.
    Station(u8),
    /// No mapped peer. A measured publication is the single-opponent baseline.
    Baseline,
    /// The name gate was busy. The published number is not used.
    Busy,
}

/// True only when this publication is the same station as the draw snapshot
/// and belongs to the current session epoch.
pub fn rtt_matches(identity: RttIdentity, publication: Publication, epoch: u64) -> bool {
    publication.epoch == epoch
        && match identity {
            RttIdentity::Busy => false,
            RttIdentity::Station(index) => publication.station() == Some(index),
            RttIdentity::Baseline => publication.measured(),
        }
}

/// Measured ping when the publication is current: the diagnostic poll is the
/// only writer of publication bytes, and a draw shows them only while the
/// generation it published is still the current one. Otherwise an uncolored blank.
pub fn present_published(
    identity: RttIdentity,
    publication: Publication,
    session_epoch: u64,
    published_generation: u64,
    draw_generation: u64,
) -> PresentedPing {
    if published_generation == draw_generation && rtt_matches(identity, publication, session_epoch)
    {
        PresentedPing::from_publication(publication)
    } else {
        blank_ping()
    }
}

fn blank_ping() -> PresentedPing {
    PresentedPing {
        opponent: "---".to_owned(),
        performance: None,
        color: None,
        detail: "no peer",
    }
}

/// Age every slot with its own sample tick. A retired generation hides the board.
pub fn expire_board(board: StationBoard, now_tick: u64, epoch: u64) -> StationBoard {
    if board.busy || board.epoch != epoch {
        return StationBoard {
            busy: board.busy,
            ..StationBoard::unavailable(epoch)
        };
    }
    let mut aged = board;
    for slot in aged.slots.iter_mut().take(usize::from(aged.len)) {
        *slot = expire_publication(*slot, now_tick, epoch);
    }
    aged
}

/// Ping for one station in this snapshot. A different station's number is not reused.
pub fn present_station(
    board: &StationBoard,
    index: u8,
    epoch: u64,
    published_generation: u64,
    draw_generation: u64,
) -> PresentedPing {
    let identity = RttIdentity::Station(index);
    match board.station(index) {
        Some(slot) if !board.busy && board.epoch == epoch => present_published(
            identity,
            *slot,
            epoch,
            published_generation,
            draw_generation,
        ),
        _ => blank_ping(),
    }
}

/// Worst fresh station in this snapshot. Empty and stale snapshots have no color.
///
/// `NetworkStability` orders Stable < Inconsistent < Unstable. The public manager
/// FFI is not in the interface crate; this uses that order on the stations whose
/// rings were published together.
pub fn aggregate_stability(
    board: &StationBoard,
    stations: &[u8],
    epoch: u64,
    published_generation: u64,
    draw_generation: u64,
) -> Option<Stability> {
    if board.busy || board.epoch != epoch || published_generation != draw_generation {
        return None;
    }
    stations
        .iter()
        .filter_map(|index| board.station(*index)?.peer())
        .map(|(stability, _)| stability)
        .max_by_key(|stability| *stability as u8)
}

/// One station's ring. `count` is the number of initialized samples, never
/// `count + 1`. The closed manager's warmup loop also reads one never-written
/// zero. That zero is not included here: a steady sample is Stable from the
/// first push.
#[derive(Clone, Copy)]
struct Slot {
    samples: [u32; RING_CAP],
    count: u8,
    head: u8,
    station: u8,
    tick: u64,
    occupied: bool,
}

impl Slot {
    const EMPTY: Self = Self {
        samples: [0; RING_CAP],
        count: 0,
        head: 0,
        station: 0,
        tick: 0,
        occupied: false,
    };

    fn push(&mut self, sample: u32, tick: u64) {
        self.samples[usize::from(self.head)] = sample;
        self.head = (self.head + 1) % RING_CAP as u8;
        self.count = (self.count + 1).min(RING_CAP as u8);
        self.tick = tick;
        self.occupied = true;
    }

    fn initialized(&self) -> &[u32] {
        &self.samples[..usize::from(self.count)]
    }
}

/// Integer mean of initialized samples. `None` when the window is empty.
pub fn window_mean(samples: &[u32]) -> Option<u64> {
    let sum: u128 = samples.iter().map(|sample| u128::from(*sample)).sum();
    sum.checked_div(samples.len() as u128)
        .map(|mean| mean as u64)
}

/// Sample variance of initialized samples only.
///
/// Filled windows divide by 99. Shorter windows divide by the initialized
/// count. Comparison is `var_sum <= threshold * denom` in `u128`, which matches
/// the manager thresholds without an `f64` mantissa and without wrapping a
/// negative delta. The manager's unfilled loop bound of `counter + 1` is not
/// reproduced.
pub fn window_stability(samples: &[u32]) -> Option<Stability> {
    let avg = i128::from(window_mean(samples)?);
    let var_sum: u128 = samples
        .iter()
        .map(|sample| (i128::from(*sample) - avg).unsigned_abs().pow(2))
        .sum();
    let n = samples.len() as u128;
    let denom = if n == RING_CAP as u128 { 99 } else { n };
    Some(match var_sum {
        sum if sum <= 25 * denom => Stability::Stable,
        sum if sum <= 100 * denom => Stability::Inconsistent,
        _ => Stability::Unstable,
    })
}

#[derive(Clone)]
pub struct Table {
    protocol: u64,
    local: u8,
    ticks_per_ms: u64,
    slots: [Slot; STATION_CAP],
    /// Another live protocol's last sample: protocol, tick, ticks per ms.
    foreign: Option<(u64, u64, u64)>,
    pub samples: u64,
    pub foreign_samples: u64,
    pub cleanups: u64,
    last_tick: u64,
}

impl Table {
    pub const fn new() -> Self {
        Self {
            protocol: 0,
            local: 0xfd,
            ticks_per_ms: 0,
            slots: [Slot::EMPTY; STATION_CAP],
            foreign: None,
            samples: 0,
            foreign_samples: 0,
            cleanups: 0,
            last_tick: 0,
        }
    }

    /// `true` when a different protocol replaced a fully stale table and the
    /// slots were cleared. Callers must not treat that as a named identity.
    pub fn observe_rtt(&mut self, sample: Observation) -> bool {
        let remote = station_index(sample.remote);
        let local = station_index(u32::from(sample.local));
        let (Some(remote), Some(local), Ok(rtt_ms)) = (remote, local, u32::try_from(sample.rtt_ms))
        else {
            return false;
        };
        if sample.protocol == 0 || sample.ticks_per_ms == 0 || remote == local {
            return false;
        }
        let adopted = self.protocol != 0 && self.protocol != sample.protocol;
        if adopted && !self.all_current_stale(sample.now_tick) {
            self.foreign = Some((sample.protocol, sample.now_tick, sample.ticks_per_ms));
            self.foreign_samples += 1;
            return false;
        }
        if adopted {
            self.slots = [Slot::EMPTY; STATION_CAP];
        }
        if adopted
            || self
                .foreign
                .is_some_and(|(protocol, ..)| protocol == sample.protocol)
        {
            self.foreign = None;
        }
        self.protocol = sample.protocol;
        self.local = local;
        let at = self.claim(remote, sample.now_tick);
        self.slots[at].push(rtt_ms, sample.now_tick);
        self.ticks_per_ms = sample.ticks_per_ms;
        self.samples += 1;
        self.last_tick = sample.now_tick;
        adopted
    }

    fn find(&self, station: u8) -> Option<usize> {
        self.slots
            .iter()
            .position(|slot| slot.occupied && slot.station == station)
    }

    /// The station's slot, emptied after a [`STALE_MS`] gap; else a free slot;
    /// else the slot with the oldest sample.
    fn claim(&mut self, station: u8, now_tick: u64) -> usize {
        let at = match self.find(station) {
            Some(at) if !matches!(self.age(&self.slots[at], now_tick), Some(age) if age >= STALE_MS) =>
            {
                return at;
            }
            Some(at) => at,
            None => self
                .slots
                .iter()
                .enumerate()
                .min_by_key(|(_, slot)| (slot.occupied, slot.tick))
                .map_or(0, |(at, _)| at),
        };
        self.slots[at] = Slot {
            station,
            ..Slot::EMPTY
        };
        at
    }

    fn age(&self, slot: &Slot, now_tick: u64) -> Option<u64> {
        age_ms(now_tick, slot.tick, self.ticks_per_ms)
    }

    /// True when every stored sample is older than [`STALE_MS`], or none remain.
    fn all_current_stale(&self, now_tick: u64) -> bool {
        !self
            .slots
            .iter()
            .any(|slot| slot.occupied && fresh(self.age(slot, now_tick)))
    }

    pub fn station_left(&mut self, protocol: u64, remote: u32) -> bool {
        let Some(remote) = station_index(remote) else {
            return false;
        };
        if protocol == 0 || protocol != self.protocol {
            return false;
        }
        if let Some(at) = self.find(remote) {
            self.slots[at] = Slot::EMPTY;
        }
        true
    }

    pub fn session_end(&mut self, protocol: u64) -> bool {
        if protocol == 0 {
            return false;
        }
        if self
            .foreign
            .is_some_and(|(foreign, ..)| foreign == protocol)
        {
            self.foreign = None;
        }
        if protocol != self.protocol {
            return false;
        }
        self.protocol = 0;
        self.local = 0xfd;
        self.ticks_per_ms = 0;
        self.last_tick = 0;
        self.slots = [Slot::EMPTY; STATION_CAP];
        self.cleanups += 1;
        true
    }

    /// The single fresh remote station, else the gap found in the same pass:
    /// another live protocol, several peers, nothing stored, clock, stale.
    fn fresh_remote(&self, now_tick: u64) -> Result<usize, PingGap> {
        if self.foreign_fresh(now_tick) {
            return Err(PingGap::OtherProtocol);
        }
        let (mut found, mut occupied, mut freshest, mut skewed) = (None, false, u64::MAX, false);
        for (at, slot) in self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.occupied)
        {
            occupied = true;
            let Some(age) = self.age(slot, now_tick) else {
                skewed = true;
                continue;
            };
            freshest = freshest.min(age);
            if age < STALE_MS && slot.station != self.local && found.replace(at).is_some() {
                return Err(PingGap::MultiplePeers);
            }
        }
        match found {
            Some(at) => Ok(at),
            None if !occupied => Err(PingGap::NoneFresh),
            None if skewed || freshest >= CLOCK_SKEW_MS => Err(PingGap::Clock),
            None => Err(PingGap::Stale),
        }
    }

    /// One mapped station. A missing, local, or stale station is not replaced.
    fn station(&self, index: u8, now_tick: u64) -> Result<usize, PingGap> {
        if self.foreign_fresh(now_tick) {
            return Err(PingGap::OtherProtocol);
        }
        let found = self.find(index).filter(|_| index != self.local);
        let at = found.ok_or(PingGap::NoneFresh)?;
        match self.age(&self.slots[at], now_tick) {
            Some(age) if age < STALE_MS => Ok(at),
            Some(_) => Err(PingGap::Stale),
            None => Err(PingGap::Clock),
        }
    }

    /// A found station's mean, stability and sample stamp, or the gap.
    fn publication_of(&self, found: Result<usize, PingGap>, epoch: u64) -> Publication {
        let Ok(at) = found else {
            return Publication::blank(found.unwrap_err(), epoch);
        };
        let slot = &self.slots[at];
        let samples = slot.initialized();
        let mean = window_mean(samples).map_or(0, |mean| mean as i64);
        let peer = window_stability(samples)
            .map_or(NO_PEER, |stability| pack_peer(stability, slot.station));
        let (tick, per_ms) = (slot.tick, self.ticks_per_ms);
        Publication::from_reading(PingReading::Measured(mean), tick, per_ms, epoch, peer)
    }

    fn foreign_fresh(&self, now_tick: u64) -> bool {
        self.foreign
            .is_some_and(|(_, tick, per_ms)| fresh(age_ms(now_tick, tick, per_ms)))
    }
}

/// `0..=252`. Values above that include the `0xFD` / `0xFE` / `0xFF` sentinels.
pub fn station_index(raw: u32) -> Option<u8> {
    u8::try_from(raw)
        .ok()
        .filter(|index| *index <= MAX_STATION_INDEX)
}

/// Milliseconds since `sample_tick`. `None` when the clock rate is unknown or
/// the clock went backwards; both read as a clock fault.
fn age_ms(now_tick: u64, sample_tick: u64, ticks_per_ms: u64) -> Option<u64> {
    if ticks_per_ms == 0 || now_tick < sample_tick {
        return None;
    }
    Some((now_tick - sample_tick) / ticks_per_ms)
}

fn fresh(age: Option<u64>) -> bool {
    age.is_some_and(|age| age < STALE_MS)
}

/// Mapped remotes published together. Eight matches the handshake table.
pub const BOARD_CAP: usize = 8;

/// One poll's stations. `busy` carries no measurements.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StationBoard {
    pub len: u8,
    pub busy: bool,
    pub epoch: u64,
    pub slots: [Publication; BOARD_CAP],
}

impl StationBoard {
    pub const fn unavailable(epoch: u64) -> Self {
        Self {
            len: 0,
            busy: false,
            epoch,
            slots: [Publication::unavailable(epoch); BOARD_CAP],
        }
    }

    pub const fn busy(epoch: u64) -> Self {
        Self {
            busy: true,
            ..Self::unavailable(epoch)
        }
    }

    fn station(&self, index: u8) -> Option<&Publication> {
        self.slots
            .iter()
            .take(usize::from(self.len))
            .find(|slot| slot.station() == Some(index))
    }
}

const UNAVAILABLE: u64 = 1 << 63;

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
    const fn blank(gap: PingGap, epoch: u64) -> Self {
        Self::from_reading(PingReading::Unavailable(gap), 0, 0, epoch, NO_PEER)
    }

    pub const fn unavailable(epoch: u64) -> Self {
        Self::blank(PingGap::NoneFresh, epoch)
    }

    /// The value shown while the observer hooks are not installed.
    pub const fn off(epoch: u64) -> Self {
        Self::blank(PingGap::Off, epoch)
    }

    pub const fn from_reading(
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

    fn measured(self) -> bool {
        self.bits & UNAVAILABLE == 0
    }

    /// Stability and station of a measured publication.
    fn peer(self) -> Option<(Stability, u8)> {
        self.measured().then(|| unpack_peer(self.peer)).flatten()
    }

    fn station(self) -> Option<u8> {
        self.peer().map(|(_, station)| station)
    }
}

fn detail_text(reading: PingReading) -> &'static str {
    match reading {
        PingReading::Measured(_) => "1 peer",
        PingReading::Unavailable(PingGap::NoneFresh) => "no peer",
        PingReading::Unavailable(PingGap::MultiplePeers) => "multiple peers",
        PingReading::Unavailable(PingGap::OtherProtocol) => "other session",
        PingReading::Unavailable(PingGap::Stale) => "stale",
        PingReading::Unavailable(PingGap::Off) => "hooks not installed",
        PingReading::Unavailable(PingGap::Clock) => "clock",
    }
}

/// Bit 63 set: unavailable, reason in the low byte. Clear: reason `1` in the
/// low byte and the milliseconds (55 bits) from bit 8.
pub const fn pack_reading(reading: PingReading) -> u64 {
    match reading {
        PingReading::Measured(ms) if ms >= 0 => 1 | ((ms as u64 & ((1 << 55) - 1)) << 8),
        PingReading::Measured(_) => UNAVAILABLE | PingGap::Clock as u64,
        PingReading::Unavailable(gap) => UNAVAILABLE | gap as u64,
    }
}

pub fn unpack_reading(bits: u64) -> PingReading {
    if bits & UNAVAILABLE == 0 {
        return PingReading::Measured((bits >> 8) as i64);
    }
    PingReading::Unavailable(match (bits & 0xff) as u8 {
        2 => PingGap::MultiplePeers,
        3 => PingGap::OtherProtocol,
        4 => PingGap::Stale,
        5 => PingGap::Off,
        6 => PingGap::Clock,
        _ => PingGap::NoneFresh,
    })
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
            publication: Publication::off(1),
        }
    }

    pub fn publication(&self) -> Publication {
        self.publication
    }

    pub fn table(&self) -> &Table {
        &self.table
    }

    /// Apply one hook or poll update at `epoch`, then republish for `station`.
    /// A changed epoch (a missed leave or cleanup) discards the table first.
    /// `clock` only decides which samples are still fresh; `None` is the last
    /// sample's tick. A measurement keeps the RTT sample's own tick, so a
    /// later poll cannot make that sample new.
    pub fn accept(
        &mut self,
        epoch: u64,
        station: StationPick,
        clock: Option<u64>,
        update: impl FnOnce(&mut Table) -> bool,
    ) -> bool {
        if self.applied != epoch {
            self.table = Table::new();
            self.applied = epoch;
            self.publication = Publication::unavailable(epoch);
        }
        let changed = update(&mut self.table);
        let clock = clock.unwrap_or(self.table.last_tick);
        let found = match station {
            StationPick::Index(index) => self.table.station(index, clock),
            StationPick::Busy => Err(PingGap::NoneFresh),
            StationPick::Absent => self.table.fresh_remote(clock),
        };
        self.publication = self.table.publication_of(found, epoch);
        changed
    }

    /// Every requested station at one clock. Each slot keeps that station's sample tick.
    pub fn board_for(
        &self,
        clock: u64,
        epoch: u64,
        stations: impl IntoIterator<Item = u8>,
    ) -> StationBoard {
        let mut board = StationBoard::unavailable(epoch);
        if self.applied != epoch {
            return board;
        }
        for (slot, index) in board.slots.iter_mut().zip(stations) {
            *slot = self
                .table
                .publication_of(self.table.station(index, clock), epoch);
            board.len += 1;
        }
        board
    }
}

/// Which station the current draw is allowed to publish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StationPick {
    Index(u8),
    Absent,
    Busy,
}

/// Age a publication when the table lock is unavailable. A missed lifecycle
/// epoch forces unavailable. A measured value expires at [`STALE_MS`].
pub fn expire_publication(published: Publication, now_tick: u64, epoch: u64) -> Publication {
    if published.epoch != epoch {
        return Publication::unavailable(epoch);
    }
    if !published.measured() {
        return published;
    }
    match age_ms(now_tick, published.tick, published.ticks_per_ms) {
        Some(age) if age < STALE_MS => published,
        Some(_) => Publication::blank(PingGap::Stale, epoch),
        None => Publication::blank(PingGap::Clock, epoch),
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
        let reading = publication.reading();
        let detail = detail_text(reading);
        let PingReading::Measured(ms) = reading else {
            return Self {
                detail,
                ..blank_ping()
            };
        };
        Self {
            opponent: format!("{ms}ms"),
            performance: Some(format!("{ms} ms")),
            color: publication
                .peer()
                .map(|(stability, _)| ping_rgba(stability)),
            detail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PingGap::{MultiplePeers, NoneFresh, OtherProtocol, Stale};
    use super::PingReading::{Measured, Unavailable};
    use super::StationPick::{Absent, Busy, Index};
    use super::*;

    const PER_MS: u64 = 1_000;
    const PROTOCOL: u64 = 0x1000;
    const TIMEOUT: u64 = STALE_MS * PER_MS;

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

    fn obs(remote: u32, local: u8, rtt_ms: i64, now_tick: u64) -> Observation {
        at(PROTOCOL, remote, local, rtt_ms, now_tick)
    }

    fn read(table: &Table, now: u64) -> PingReading {
        table.publication_of(table.fresh_remote(now), 1).reading()
    }

    fn read_for(table: &Table, index: u8, now: u64) -> Publication {
        table.publication_of(table.station(index, now), 1)
    }

    fn sample(session: &mut Session, seen: Observation, epoch: u64, pick: StationPick) -> bool {
        session.accept(epoch, pick, Some(seen.now_tick), |t| t.observe_rtt(seen))
    }

    fn poll(session: &mut Session, now: u64, epoch: u64, pick: StationPick) {
        session.accept(epoch, pick, Some(now), |_| false);
    }

    fn leave(session: &mut Session, protocol: u64, remote: u32) -> bool {
        session.accept(1, Absent, None, |t| t.station_left(protocol, remote))
    }

    /// The station's stored mean, whatever the clock or another protocol says.
    fn stored(session: &Session, index: u8) -> Option<u64> {
        let table = &session.table;
        let at = table.find(index)?;
        window_mean(table.slots[at].initialized())
    }

    fn published(station: u8, epoch: u64) -> Publication {
        let peer = pack_peer(Stability::Stable, station);
        Publication::from_reading(Measured(20), 10, PER_MS, epoch, peer)
    }

    #[test]
    fn local_is_excluded_index_zero_is_valid_and_bad_rtts_keep_the_prior_sample() {
        let mut table = Table::new();
        table.observe_rtt(obs(4, 4, 12, 0));
        table.observe_rtt(obs(0, 4, 9, 0));
        assert_eq!(read(&table, 0), Measured(9));
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 0, 0));
        assert_eq!(read(&table, 0), Measured(0));
        table.observe_rtt(obs(2, 1, -1, PER_MS));
        table.observe_rtt(obs(2, 1, i64::MAX, PER_MS));
        assert_eq!(table.samples, 1);
        assert_eq!(read(&table, PER_MS), Measured(0));
    }

    #[test]
    fn sample_expires_at_three_seconds_and_a_leave_frees_the_index() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 15, 0));
        assert_eq!(read(&table, TIMEOUT - 1), Measured(15));
        assert_eq!(read(&table, TIMEOUT), Unavailable(Stale));
        assert!(table.station_left(PROTOCOL, 2));
        assert_eq!(read(&table, 0), Unavailable(NoneFresh));
        table.observe_rtt(obs(2, 1, 4, PER_MS));
        assert_eq!(read(&table, PER_MS), Measured(4));
    }

    #[test]
    fn cleanup_drops_peers_and_a_new_protocol_starts_empty() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 10, 0));
        table.observe_rtt(obs(3, 1, 11, 0));
        assert!(!table.session_end(PROTOCOL + 3));
        assert_eq!(table.cleanups, 0);
        assert!(table.session_end(PROTOCOL));
        assert_eq!(read(&table, 0), Unavailable(NoneFresh));
        table.observe_rtt(at(PROTOCOL + 1, 6, 5, 7, PER_MS));
        assert_eq!(read(&table, PER_MS), Measured(7));
        assert_eq!(table.cleanups, 1);
    }

    #[test]
    fn two_fresh_peers_are_not_labeled_as_one_opponent() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 10, 0));
        table.observe_rtt(obs(3, 1, 40, 0));
        assert_eq!(read(&table, 0), Unavailable(MultiplePeers));
        let ann = read_for(&table, 2, 0);
        assert_eq!(ann.reading(), Measured(10));
        assert_eq!(ann.peer(), Some((Stability::Stable, 2)));
        assert_eq!(read_for(&table, 3, 0).reading(), Measured(40));
        assert_eq!(read_for(&table, 1, 0).reading(), Unavailable(NoneFresh));
        table.station_left(PROTOCOL, 3);
        assert_eq!(read(&table, 0), Measured(10));
        let mut session = Session::new();
        sample(&mut session, obs(2, 1, 10, 0), 1, Index(2));
        sample(&mut session, obs(3, 1, 40, 0), 1, Index(2));
        assert_eq!(session.publication().reading(), Measured(10));
        assert_eq!(session.publication().station(), Some(2));
        poll(&mut session, 0, 1, Index(3));
        assert_eq!(session.publication().reading(), Measured(40));
        poll(&mut session, 0, 1, Busy);
        assert_eq!(session.publication().reading(), Unavailable(NoneFresh));
        poll(&mut session, 0, 1, Absent);
        assert_eq!(session.publication().reading(), Unavailable(MultiplePeers));
    }

    #[test]
    fn a_changed_peer_hides_the_old_rtt_until_the_same_station_is_republished() {
        let old = published(4, 1);
        assert!(rtt_matches(RttIdentity::Station(4), old, 1));
        assert!(!rtt_matches(RttIdentity::Station(9), old, 1));
        let republished = published(9, 1);
        assert!(rtt_matches(RttIdentity::Station(9), republished, 1));
        let shown = present_published(RttIdentity::Station(9), republished, 1, 0, 0);
        assert_eq!(shown.opponent, "20ms");
        let hidden = present_published(RttIdentity::Station(4), republished, 1, 0, 0);
        assert_eq!(hidden, blank_ping());
        assert!(!rtt_matches(RttIdentity::Station(9), republished, 2));
        assert!(!rtt_matches(RttIdentity::Baseline, old, 2));
        assert!(!rtt_matches(RttIdentity::Busy, old, 1));
        assert!(rtt_matches(RttIdentity::Baseline, old, 1));
        let blank = Publication::unavailable(1);
        assert!(!rtt_matches(RttIdentity::Baseline, blank, 1));
        assert!(!rtt_matches(RttIdentity::Station(4), blank, 1));
    }

    #[test]
    fn accepted_cleanup_hides_the_old_publication_until_poll() {
        let old = published(4, 1);
        let shown = |identity: RttIdentity, publication: Publication, published: u64, draw: u64| {
            present_published(identity, publication, 1, published, draw).opponent == "20ms"
        };
        assert!(shown(RttIdentity::Station(4), old, 0, 0));
        assert!(shown(RttIdentity::Baseline, old, 0, 0));
        for draw in 1..3 {
            assert!(!shown(RttIdentity::Baseline, old, 0, draw));
            assert!(!shown(RttIdentity::Station(4), old, 0, draw));
        }
        assert!(shown(RttIdentity::Station(7), published(7, 1), 4, 4));
        assert!(shown(RttIdentity::Baseline, old, 2, 2));
        let replacement = published(9, 1);
        assert!(shown(RttIdentity::Station(9), replacement, 2, 2));
        assert!(!shown(RttIdentity::Station(4), replacement, 2, 2));
        let blank = Publication::unavailable(1);
        assert!(!shown(RttIdentity::Baseline, blank, 2, 2));
    }

    #[test]
    fn another_live_protocol_hides_the_current_peer_until_it_expires() {
        let mut table = Table::new();
        table.observe_rtt(obs(2, 1, 10, 0));
        table.observe_rtt(at(PROTOCOL + 8, 9, 1, 80, 0));
        assert_eq!((table.samples, table.foreign_samples), (1, 1));
        assert_eq!(read(&table, 0), Unavailable(OtherProtocol));
        table.observe_rtt(obs(2, 1, 10, TIMEOUT));
        assert_eq!(read(&table, TIMEOUT), Measured(10));
    }

    #[test]
    fn sentinels_are_rejected_and_252_is_kept() {
        assert_eq!(station_index(252), Some(252));
        assert_eq!([0xfd, 0xfe, 0xff].map(station_index), [None; 3]);
        let mut table = Table::new();
        table.observe_rtt(obs(0xfd, 1, 5, 0));
        table.observe_rtt(obs(252, 1, 6, 0));
        assert_eq!(table.samples, 1);
        assert_eq!(read(&table, 0), Measured(6));
    }

    #[test]
    fn foreign_and_unknown_protocol_leaves_do_not_retire_a_current_station() {
        let foreign = PROTOCOL + 8;
        let mut session = Session::new();
        sample(&mut session, obs(2, 1, 10, 0), 1, Absent);
        sample(&mut session, at(foreign, 9, 1, 80, PER_MS), 1, Absent);
        // Numeric station indices alone cannot associate another protocol's
        // leave with our connected opponent. This result gates runtime cleanup.
        assert!(!leave(&mut session, foreign, 2));
        assert!(!leave(&mut session, PROTOCOL + 3, 2));
        assert_eq!(stored(&session, 2), Some(10));
        assert!(leave(&mut session, PROTOCOL, 2));
        assert_eq!(stored(&session, 2), None);
    }

    #[test]
    fn adoption_waits_until_the_first_protocol_is_stale_and_reports_the_clear() {
        let mut session = Session::new();
        let game = PROTOCOL + 4;
        assert!(!sample(&mut session, obs(2, 1, 50, 0), 1, Absent));
        assert!(!sample(&mut session, obs(3, 1, 60, 0), 1, Absent));
        assert!(!sample(&mut session, at(game, 7, 5, 12, PER_MS), 1, Absent));
        assert_eq!(session.publication().reading(), Unavailable(OtherProtocol));
        assert_eq!(stored(&session, 2), Some(50));
        assert!(sample(&mut session, at(game, 7, 5, 12, TIMEOUT), 1, Absent));
        assert_eq!(session.publication().reading(), Measured(12));
        assert_eq!(stored(&session, 2), None);
        assert_eq!(stored(&session, 3), None);
        assert_eq!(stored(&session, 7), Some(12));
    }

    #[test]
    fn alternating_live_protocols_never_pick_one() {
        let mut session = Session::new();
        let other = PROTOCOL + 9;
        sample(&mut session, obs(2, 1, 10, 0), 1, Absent);
        assert_eq!(session.publication().reading(), Measured(10));
        for step in 1..6u64 {
            let now = step * 200 * PER_MS;
            let next = if step % 2 == 0 {
                obs(2, 1, 10, now)
            } else {
                at(other, 8, 4, 90, now)
            };
            sample(&mut session, next, 1, Absent);
            assert_eq!(session.publication().reading(), Unavailable(OtherProtocol));
            assert_eq!(stored(&session, 8), None);
        }
    }

    #[test]
    fn missed_lifecycle_drops_the_old_peer_until_a_new_sample() {
        let mut session = Session::new();
        sample(&mut session, obs(2, 1, 40, 0), 1, Absent);
        assert_eq!(session.publication().reading(), Measured(40));
        poll(&mut session, 500 * PER_MS, 2, Absent);
        assert_eq!(stored(&session, 2), None);
        assert_eq!(session.publication().reading(), Unavailable(NoneFresh));
        assert_eq!(session.board_for(500 * PER_MS, 1, [2]).len, 0);
        sample(&mut session, obs(2, 1, 4, 600 * PER_MS), 2, Absent);
        assert_eq!(session.publication().reading(), Measured(4));
    }

    #[test]
    fn poll_keeps_the_sample_tick_until_the_reader_expires_it() {
        let mut session = Session::new();
        let sample_tick = 50 * PER_MS;
        sample(&mut session, obs(2, 1, 33, sample_tick), 1, Absent);
        let first = session.publication();
        assert_eq!(first.tick, sample_tick);
        poll(&mut session, sample_tick + TIMEOUT - 1, 1, Absent);
        assert_eq!(session.publication(), first);
        let stale_at = sample_tick + TIMEOUT;
        let expired = expire_publication(first, stale_at, 1);
        assert_eq!(expired.reading(), Unavailable(Stale));
        assert_eq!(expired.peer, NO_PEER);
        assert_eq!(expire_publication(expired, stale_at * 4, 1), expired);
        assert_eq!(expire_publication(first, stale_at - 1, 1), first);
        let reset = expire_publication(first, PER_MS, 2);
        assert_eq!(reset, Publication::unavailable(2));
        poll(&mut session, stale_at, 1, Absent);
        assert_eq!(session.publication(), expired);
        poll(&mut session, stale_at + 10 * PER_MS, 1, Absent);
        assert_eq!(session.publication(), expired);
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
        let publication = table.publication_of(table.fresh_remote(0), 1);
        assert_eq!(publication.reading(), Measured(20));
        assert_eq!(publication.peer(), Some((Stability::Stable, 2)));
        assert_eq!((publication.tick, publication.ticks_per_ms), (0, PER_MS));
    }

    #[test]
    fn variance_boundaries_use_initialized_samples_and_signed_deltas() {
        let mut alternating = [0u32; 100];
        for (index, sample) in alternating.iter_mut().enumerate() {
            *sample = if index % 2 == 0 { 45 } else { 55 };
        }
        assert_eq!(window_mean(&alternating), Some(50));
        let inconsistent = Some(Stability::Inconsistent);
        assert_eq!(window_stability(&alternating), inconsistent);
        for (index, sample) in alternating.iter_mut().enumerate() {
            *sample = if index % 2 == 0 { 40 } else { 60 };
        }
        assert_eq!(window_stability(&alternating), Some(Stability::Unstable));
        assert_eq!(window_stability(&[45, 55]), Some(Stability::Stable));
        assert_eq!(window_stability(&[10, 30]), inconsistent);
        assert_eq!(window_stability(&[0, u32::MAX]), Some(Stability::Unstable));
        assert!(window_stability(&[]).is_none());
        assert!(window_mean(&[]).is_none());
    }

    #[test]
    fn ring_wrap_expiry_leave_and_protocol_reset_drop_old_history() {
        let mut table = Table::new();
        for index in 0..100 {
            table.observe_rtt(obs(2, 1, 5, index * PER_MS));
        }
        assert_eq!(read(&table, 99 * PER_MS), Measured(5));
        table.observe_rtt(obs(2, 1, 105, 99 * PER_MS));
        assert_eq!(read(&table, 99 * PER_MS), Measured(6));
        let stale_push = 99 * PER_MS + TIMEOUT;
        table.observe_rtt(obs(2, 1, 10, stale_push));
        assert_eq!(read(&table, stale_push), Measured(10));
        table.station_left(PROTOCOL, 2);
        assert_eq!(read(&table, stale_push), Unavailable(NoneFresh));
        table.observe_rtt(obs(2, 1, 4, stale_push));
        assert_eq!(read(&table, stale_push), Measured(4));
        table.session_end(PROTOCOL);
        table.observe_rtt(at(PROTOCOL + 1, 6, 5, 8, stale_push));
        let publication = table.publication_of(table.fresh_remote(stale_push), 1);
        assert_eq!(publication.reading(), Measured(8));
        assert_eq!(publication.peer(), Some((Stability::Stable, 6)));
    }

    #[test]
    fn a_seventeenth_fresh_station_evicts_the_stalest() {
        let mut table = Table::new();
        for station in 0..STATION_CAP as u32 {
            let tick = (u64::from(station) + 1) * 100 * PER_MS;
            table.observe_rtt(obs(station + 1, 0, 10 + i64::from(station), tick));
        }
        let now = (STATION_CAP as u64 + 1) * 100 * PER_MS;
        assert_eq!(read_for(&table, 1, now).reading(), Measured(10));
        table.observe_rtt(obs(40, 0, 99, now));
        assert_eq!(read_for(&table, 40, now).reading(), Measured(99));
        assert_eq!(read_for(&table, 1, now).reading(), Unavailable(NoneFresh));
        assert_eq!(read_for(&table, 2, now).reading(), Measured(11));
        assert_eq!(read_for(&table, 16, now).reading(), Measured(25));
        table.observe_rtt(obs(2, 0, 13, now));
        assert_eq!(read_for(&table, 2, now).reading(), Measured(12));
        assert_eq!(table.samples, STATION_CAP as u64 + 2);
    }

    #[test]
    fn overflow_is_rejected_and_a_stale_snapshot_has_no_colour() {
        let mut session = Session::new();
        sample(&mut session, obs(2, 1, 20, 0), 1, Absent);
        sample(&mut session, obs(2, 1, i64::MAX, PER_MS), 1, Absent);
        assert_eq!(session.publication().reading(), Measured(20));
        assert_eq!(session.table().samples, 1);
        let presented = PresentedPing::from_publication(session.publication());
        assert_eq!(presented.opponent, "20ms");
        assert_eq!(presented.performance.as_deref(), Some("20 ms"));
        assert_eq!(presented.color, Some(ping_rgba(Stability::Stable)));
        assert_eq!(presented.detail, "1 peer");
        let expired = expire_publication(session.publication(), TIMEOUT, 1);
        let stale = PresentedPing::from_publication(expired);
        assert_eq!(stale.opponent, "---");
        assert_eq!(stale.performance, None);
        assert_eq!(stale.color, None);
        assert_eq!(stale.detail, "stale");
        let off = PresentedPing::from_publication(Publication::off(1));
        assert_eq!(off.detail, "hooks not installed");
        assert_eq!(Publication::off(1), Session::new().publication());
    }

    #[test]
    fn board_keeps_each_station_age_and_the_worst_color() {
        let mut session = Session::new();
        sample(&mut session, obs(4, 1, 10, 0), 1, Index(4));
        sample(&mut session, obs(4, 1, 10_000, 0), 1, Index(4));
        let now = TIMEOUT - 1;
        sample(&mut session, obs(9, 1, 20, now), 1, Index(9));
        let board = session.board_for(now, 1, [4, 9]);
        assert_eq!((board.len, board.busy), (2, false));
        assert_eq!(present_station(&board, 4, 1, 3, 3).opponent, "5005ms");
        assert_eq!(present_station(&board, 9, 1, 3, 3).opponent, "20ms");
        assert_eq!(present_station(&board, 2, 1, 3, 3).opponent, "---");
        assert_eq!(present_station(&board, 4, 1, 3, 4).opponent, "---");
        let worst = aggregate_stability(&board, &[4, 9], 1, 3, 3);
        assert_eq!(worst, Some(Stability::Unstable));
        let stable = Some(Stability::Stable);
        assert_eq!(aggregate_stability(&board, &[9], 1, 3, 3), stable);
        let aged = expire_board(board, TIMEOUT, 1);
        assert_eq!(present_station(&aged, 4, 1, 3, 3).opponent, "---");
        assert_eq!(present_station(&aged, 9, 1, 3, 3).opponent, "20ms");
        assert_eq!(aggregate_stability(&aged, &[4, 9], 1, 3, 3), stable);
        assert_eq!(expire_board(board, now, 2), StationBoard::unavailable(2));
        let busy = StationBoard::busy(1);
        assert_eq!(expire_board(busy, now, 1), busy);
        assert_eq!(present_station(&busy, 9, 1, 3, 3).opponent, "---");
        assert_eq!(aggregate_stability(&busy, &[9], 1, 3, 3), None);
        let many = session.board_for(now, 1, [0u8; BOARD_CAP + 3]);
        assert_eq!(usize::from(many.len), BOARD_CAP);
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
}
