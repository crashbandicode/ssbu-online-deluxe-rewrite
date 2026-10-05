//! Portable decoding, validation and reporting for `nn::vi` frame-history
//! samples. The values are compositor bookkeeping that the SDK converts to
//! `nn::os` ticks (19.2 MHz) before returning them. Nothing here measures
//! scanout, display latency or input latency, and no field is ever turned
//! into a Rust enum before validation.
//!
//! ABI facts this module relies on (live 13.0.5 disassembly plus SDK map):
//! `ListFrameHistory(out, count, layer)` returns the constant 8 when `out` is
//! null or `count` is 0, otherwise the number of 40-byte entries written;
//! each entry is `{status u64 (1 enqueued, 2 presented), frame_number u64,
//! enqueue_tick i64, present_tick i64, vblank_tick i64}`, with `vblank_tick`
//! written as a copy of `present_tick` in this SDK build; the times are
//! produced by `nnosConvertNanoSecondsToTick`. `GetLatestFrameNumber` returns
//! 0 on success, 0x1672 while no frame has been presented, 0x272 on failure.

pub const TICK_HZ: u64 = 19_200_000;
/// `ListFrameHistory(null, 0, layer)` returns this constant.
pub const CAPACITY: usize = 8;
pub const ENTRY_BYTES: usize = 40;
pub const SAMPLES_WANTED: u32 = 10;
pub const MAX_ATTEMPTS: u32 = 20;
pub const MAX_CONSECUTIVE_ERRORS: u32 = 3;
pub const MAX_INVALID_SAMPLES: u32 = 3;
pub const MIN_EMPTY_FOR_VERDICT: u32 = 3;
pub const MAX_NOT_YET: u32 = 10;
/// Timestamps further than this from the sampling tick are implausible.
pub const PLAUSIBLE_WINDOW_TICKS: i64 = 60 * TICK_HZ as i64;

pub const STATUS_ENQUEUED: u64 = 1;
pub const STATUS_PRESENTED: u64 = 2;

pub const RESULT_OK: u32 = 0;
pub const RESULT_NO_FRAME_YET: u32 = 0x1672;
pub const RESULT_VI_FAILURE: u32 = 0x272;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RawFrameInfo {
    pub status: u64,
    pub frame_number: u64,
    pub enqueue_tick: i64,
    pub present_tick: i64,
    pub vblank_tick: i64,
}
const _: () = assert!(core::mem::size_of::<RawFrameInfo>() == ENTRY_BYTES);
const _: () = assert!(core::mem::align_of::<RawFrameInfo>() == 8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Latest {
    Frame(u64),
    NotYet,
    Error(u32),
}

/// Interprets `GetLatestFrameNumber` by its raw result value only.
pub fn classify_latest(result: u32, value: u64) -> Latest {
    match result {
        RESULT_OK => Latest::Frame(value),
        RESULT_NO_FRAME_YET => Latest::NotYet,
        other => Latest::Error(other),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SampleCheck {
    pub count: usize,
    pub presented: usize,
    pub enqueued: usize,
    pub issues: Vec<&'static str>,
    pub notes: Vec<&'static str>,
    /// Consecutive `present_tick` differences among presented entries, in array order.
    pub presented_deltas: Vec<i64>,
    pub newest_present: Option<i64>,
    pub frame_span: Option<(u64, u64)>,
}

impl SampleCheck {
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

fn push_unique(list: &mut Vec<&'static str>, item: &'static str) {
    if !list.contains(&item) {
        list.push(item);
    }
}

/// Validates one `ListFrameHistory` result without trusting any field.
pub fn check_sample(
    entries: &[RawFrameInfo],
    count: i32,
    now_tick: u64,
    latest: Option<u64>,
) -> SampleCheck {
    let mut check = SampleCheck::default();
    if count < 0 || count as usize > CAPACITY || count as usize > entries.len() {
        check.issues.push("count_out_of_range");
        return check;
    }
    let used = &entries[..count as usize];
    check.count = used.len();
    let now = now_tick as i64;
    let plausible = |tick: i64| {
        (i128::from(now) - i128::from(tick)).abs() <= i128::from(PLAUSIBLE_WINDOW_TICKS)
    };
    let mut issues = Vec::new();
    let mut notes = Vec::new();
    if used.is_empty() {
        notes.push("empty");
    }
    let mut previous_frame: Option<u64> = None;
    let mut previous_present: Option<i64> = None;
    let mut span: Option<(u64, u64)> = None;
    for entry in used {
        if let Some(previous) = previous_frame {
            if entry.frame_number < previous {
                push_unique(&mut issues, "frame_order");
            }
        }
        previous_frame = Some(entry.frame_number);
        span = Some(match span {
            Some((low, high)) => (low.min(entry.frame_number), high.max(entry.frame_number)),
            None => (entry.frame_number, entry.frame_number),
        });
        if !plausible(entry.enqueue_tick) {
            push_unique(&mut issues, "enqueue_tick_implausible");
        }
        match entry.status {
            STATUS_ENQUEUED => {
                check.enqueued += 1;
                if entry.present_tick != 0 || entry.vblank_tick != 0 {
                    push_unique(&mut issues, "enqueued_with_present");
                }
            }
            STATUS_PRESENTED => {
                check.presented += 1;
                if entry.present_tick < entry.enqueue_tick {
                    push_unique(&mut issues, "present_before_enqueue");
                }
                if !plausible(entry.present_tick) {
                    push_unique(&mut issues, "present_tick_implausible");
                }
                if entry.vblank_tick != entry.present_tick {
                    push_unique(&mut notes, "vblank_differs_from_present");
                }
                if let Some(previous) = previous_present {
                    let delta = entry.present_tick.saturating_sub(previous);
                    if delta <= 0 {
                        push_unique(&mut issues, "present_not_monotonic");
                    } else {
                        check.presented_deltas.push(delta);
                    }
                }
                previous_present = Some(entry.present_tick);
                check.newest_present = Some(
                    check
                        .newest_present
                        .map_or(entry.present_tick, |newest| newest.max(entry.present_tick)),
                );
            }
            _ => push_unique(&mut issues, "status_invalid"),
        }
    }
    if let (Some(latest_frame), Some((_, high))) = (latest, span) {
        if high > latest_frame {
            push_unique(&mut issues, "frame_beyond_latest");
        } else if latest_frame.saturating_sub(high) > CAPACITY as u64 {
            push_unique(&mut notes, "history_stale_vs_latest");
        }
    }
    check.frame_span = span;
    check.issues = issues;
    check.notes = notes;
    check
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    Ok,
    Empty,
    Invalid,
    Error(u32),
    NotYet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    Ok,
    EmptyHistory,
    NoFramesYet,
    Unsupported,
    LayerNotSeen,
    Timeout,
}

impl Outcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ok => "ok",
            Self::EmptyHistory => "empty_history",
            Self::NoFramesYet => "no_frames_yet",
            Self::Unsupported => "unsupported",
            Self::LayerNotSeen => "layer_not_seen",
            Self::Timeout => "timeout",
        }
    }
    pub fn code(self) -> u64 {
        match self {
            Self::Pending => 0,
            Self::Ok => 1,
            Self::EmptyHistory => 2,
            Self::NoFramesYet => 3,
            Self::Unsupported => 4,
            Self::LayerNotSeen => 5,
            Self::Timeout => 6,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunState {
    pub layer_seen: bool,
    pub capacity_query: Option<i32>,
    pub attempts: u32,
    pub ok: u32,
    pub empty: u32,
    pub invalid: u32,
    pub errors: u32,
    pub consecutive_errors: u32,
    pub not_yet: u32,
    pub last_error: u32,
    pub deltas: Vec<i64>,
}

impl RunState {
    pub fn record(&mut self, observation: Observation, deltas: &[i64]) {
        self.attempts += 1;
        match observation {
            Observation::Ok => {
                self.ok += 1;
                self.consecutive_errors = 0;
                self.deltas.extend_from_slice(deltas);
            }
            Observation::Empty => {
                self.empty += 1;
                self.consecutive_errors = 0;
            }
            Observation::Invalid => {
                self.invalid += 1;
                self.consecutive_errors = 0;
            }
            Observation::Error(code) => {
                self.errors += 1;
                self.consecutive_errors += 1;
                self.last_error = code;
            }
            Observation::NotYet => {
                self.not_yet += 1;
                self.consecutive_errors = 0;
            }
        }
    }

    /// `deadline_passed` covers both the wall-clock deadline and the attempt cap.
    pub fn outcome(&self, deadline_passed: bool) -> Outcome {
        if matches!(self.capacity_query, Some(c) if c != CAPACITY as i32)
            || self.consecutive_errors >= MAX_CONSECUTIVE_ERRORS
            || self.invalid >= MAX_INVALID_SAMPLES
        {
            return Outcome::Unsupported;
        }
        if !self.layer_seen {
            return if deadline_passed {
                Outcome::LayerNotSeen
            } else {
                Outcome::Pending
            };
        }
        if self.ok == 0 && self.not_yet >= MAX_NOT_YET {
            return Outcome::NoFramesYet;
        }
        let settled = self.ok + self.empty;
        if settled >= SAMPLES_WANTED || deadline_passed || self.attempts >= MAX_ATTEMPTS {
            if self.ok > 0 {
                return Outcome::Ok;
            }
            if self.empty >= MIN_EMPTY_FOR_VERDICT {
                return Outcome::EmptyHistory;
            }
            return Outcome::Timeout;
        }
        Outcome::Pending
    }
}

/// Exact integer conversion: ticks * 1_000_000 / 19_200_000.
pub fn ticks_to_micros(ticks: i64) -> i64 {
    (i128::from(ticks) * 1_000_000 / i128::from(TICK_HZ)) as i64
}

/// Lower median of the values (no averaging), `None` when empty.
pub fn median(values: &[i64]) -> Option<i64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    Some(sorted[(sorted.len() - 1) / 2])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleMeta {
    pub index: u32,
    pub poll_tick: u64,
    pub cost_ticks: u64,
    pub latest: Latest,
    pub count: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    lines: Vec<String>,
}

impl Report {
    pub fn new(header: &[(&str, String)]) -> Self {
        Self {
            lines: header
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect(),
        }
    }

    pub fn push(&mut self, line: String) {
        self.lines.push(line);
    }

    pub fn push_sample(
        &mut self,
        meta: &SampleMeta,
        entries: &[RawFrameInfo],
        check: &SampleCheck,
    ) {
        let latest = match meta.latest {
            Latest::Frame(n) => format!("frame {n}"),
            Latest::NotYet => "not_yet(0x1672)".to_owned(),
            Latest::Error(code) => format!("error 0x{code:x}"),
        };
        self.lines.push(format!(
            "sample {} poll_tick={} query_cost_ticks={} latest={latest} count={} presented={} enqueued={} issues={:?} notes={:?}",
            meta.index, meta.poll_tick, meta.cost_ticks, meta.count, check.presented, check.enqueued, check.issues, check.notes
        ));
        for (n, entry) in entries.iter().take(check.count).enumerate() {
            self.lines.push(format!(
                "  entry {n} status={} frame={} enqueue={} present={} vblank={}",
                entry.status,
                entry.frame_number,
                entry.enqueue_tick,
                entry.present_tick,
                entry.vblank_tick
            ));
        }
        if !check.presented_deltas.is_empty() {
            let micros: Vec<i64> = check
                .presented_deltas
                .iter()
                .map(|&d| ticks_to_micros(d))
                .collect();
            self.lines.push(format!(
                "  presented_deltas_ticks={:?} presented_deltas_us={micros:?}",
                check.presented_deltas
            ));
        }
        if let Some(newest) = check.newest_present {
            self.lines.push(format!(
                "  now_minus_newest_present_ticks={}",
                (meta.poll_tick as i64).saturating_sub(newest)
            ));
        }
    }

    pub fn push_summary(&mut self, state: &RunState, outcome: Outcome) {
        self.lines.push(format!(
            "attempts={} ok={} empty={} invalid={} errors={} not_yet={} last_error=0x{:x} capacity_query={:?} layer_seen={}",
            state.attempts, state.ok, state.empty, state.invalid, state.errors, state.not_yet,
            state.last_error, state.capacity_query, state.layer_seen
        ));
        match median(&state.deltas) {
            Some(m) => self.lines.push(format!(
                "median_presented_delta_ticks={m} median_presented_delta_us={} expected_60hz_ticks=320000",
                ticks_to_micros(m)
            )),
            None => self.lines.push("median_presented_delta_ticks=none".to_owned()),
        }
        self.lines.push(format!("outcome={}", outcome.label()));
        self.lines.push(
            "meaning=compositor frame bookkeeping only; not scanout, not display or input latency"
                .to_owned(),
        );
    }

    pub fn render(&self) -> String {
        let mut text = self.lines.join("\n");
        text.push('\n');
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn presented(frame: u64, present: i64) -> RawFrameInfo {
        RawFrameInfo {
            status: STATUS_PRESENTED,
            frame_number: frame,
            enqueue_tick: present - 100_000,
            present_tick: present,
            vblank_tick: present,
        }
    }

    fn steady(now: u64) -> [RawFrameInfo; CAPACITY] {
        let mut entries = [RawFrameInfo::default(); CAPACITY];
        for (n, entry) in entries.iter_mut().enumerate() {
            *entry = presented(100 + n as u64, now as i64 - 2_560_000 + n as i64 * 320_000);
        }
        entries
    }

    #[test]
    fn layout_and_result_classification() {
        assert_eq!(core::mem::size_of::<RawFrameInfo>(), 40);
        assert_eq!(classify_latest(0, 77), Latest::Frame(77));
        assert_eq!(classify_latest(0x1672, 0), Latest::NotYet);
        assert_eq!(classify_latest(0x272, 0), Latest::Error(0x272));
        assert_eq!(ticks_to_micros(320_000), 16_666);
        assert_eq!(median(&[3, 1, 2]), Some(2));
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn steady_history_validates_with_sixty_hertz_deltas() {
        let now = 10_000_000_000;
        let check = check_sample(&steady(now), 8, now, Some(107));
        assert!(check.is_valid(), "{:?}", check.issues);
        assert_eq!((check.count, check.presented, check.enqueued), (8, 8, 0));
        assert_eq!(check.presented_deltas, vec![320_000; 7]);
        assert_eq!(check.frame_span, Some((100, 107)));
        assert!(check.notes.is_empty());
    }

    #[test]
    fn invalid_fields_are_reported_not_trusted() {
        let now = 10_000_000_000;
        let mut entries = steady(now);
        entries[2].status = 3;
        entries[4].frame_number = 1;
        entries[5].present_tick = entries[5].enqueue_tick - 1;
        entries[6].status = STATUS_ENQUEUED;
        let check = check_sample(&entries, 8, now, Some(103));
        for issue in [
            "status_invalid",
            "frame_order",
            "present_before_enqueue",
            "enqueued_with_present",
            "frame_beyond_latest",
        ] {
            assert!(
                check.issues.contains(&issue),
                "missing {issue}: {:?}",
                check.issues
            );
        }
        assert!(check_sample(&entries, 9, now, None)
            .issues
            .contains(&"count_out_of_range"));
        assert!(check_sample(&entries, -1, now, None)
            .issues
            .contains(&"count_out_of_range"));
        let mut far = steady(now);
        far[0].enqueue_tick = 0;
        assert!(check_sample(&far, 8, now, None)
            .issues
            .contains(&"enqueue_tick_implausible"));
        let empty = check_sample(&entries, 0, now, Some(5));
        assert!(empty.is_valid() && empty.notes.contains(&"empty"));
        let mut extreme = steady(now);
        extreme[0].enqueue_tick = i64::MIN;
        extreme[0].present_tick = i64::MIN;
        extreme[1].present_tick = i64::MAX;
        extreme[7].frame_number = u64::MAX;
        let check = check_sample(&extreme, 8, now, Some(u64::MAX));
        assert!(!check.is_valid());
    }

    #[test]
    fn outcomes_distinguish_unsupported_empty_and_ok() {
        let mut state = RunState::default();
        assert_eq!(state.outcome(false), Outcome::Pending);
        assert_eq!(state.outcome(true), Outcome::LayerNotSeen);
        state.layer_seen = true;
        state.capacity_query = Some(8);
        for _ in 0..3 {
            state.record(Observation::Error(0x272), &[]);
        }
        assert_eq!(state.outcome(false), Outcome::Unsupported);

        let mut empty = RunState {
            layer_seen: true,
            capacity_query: Some(8),
            ..RunState::default()
        };
        for _ in 0..10 {
            empty.record(Observation::Empty, &[]);
        }
        assert_eq!(empty.outcome(false), Outcome::EmptyHistory);

        let mut ok = RunState {
            layer_seen: true,
            capacity_query: Some(8),
            ..RunState::default()
        };
        ok.record(Observation::Error(0x272), &[]);
        for _ in 0..9 {
            ok.record(Observation::Ok, &[320_000, 320_000]);
        }
        assert_eq!(ok.outcome(false), Outcome::Pending);
        ok.record(Observation::Ok, &[320_000]);
        assert_eq!(ok.outcome(false), Outcome::Ok);
        assert_eq!(median(&ok.deltas), Some(320_000));

        let mut not_yet = RunState {
            layer_seen: true,
            capacity_query: Some(8),
            ..RunState::default()
        };
        for _ in 0..10 {
            not_yet.record(Observation::NotYet, &[]);
        }
        assert_eq!(not_yet.outcome(false), Outcome::NoFramesYet);

        let wrong_capacity = RunState {
            layer_seen: true,
            capacity_query: Some(16),
            ..RunState::default()
        };
        assert_eq!(wrong_capacity.outcome(false), Outcome::Unsupported);

        let mut invalid = RunState {
            layer_seen: true,
            capacity_query: Some(8),
            ..RunState::default()
        };
        for _ in 0..3 {
            invalid.record(Observation::Invalid, &[]);
        }
        assert_eq!(invalid.outcome(false), Outcome::Unsupported);
    }

    #[test]
    fn report_lists_samples_and_summary() {
        let now = 10_000_000_000;
        let entries = steady(now);
        let check = check_sample(&entries, 8, now, Some(107));
        let mut report = Report::new(&[("metric", "compositor_frame_history".to_owned())]);
        let meta = SampleMeta {
            index: 1,
            poll_tick: now,
            cost_ticks: 1200,
            latest: Latest::Frame(107),
            count: 8,
        };
        report.push_sample(&meta, &entries, &check);
        let mut state = RunState {
            layer_seen: true,
            capacity_query: Some(8),
            ..RunState::default()
        };
        state.record(Observation::Ok, &check.presented_deltas);
        report.push_summary(&state, state.outcome(true));
        let text = report.render();
        assert!(text.starts_with("metric=compositor_frame_history\n"));
        assert!(text.contains("sample 1 poll_tick=10000000000 query_cost_ticks=1200 latest=frame 107 count=8 presented=8"));
        assert!(text.contains("  entry 7 status=2 frame=107"));
        assert!(
            text.contains("median_presented_delta_ticks=320000 median_presented_delta_us=16666")
        );
        assert!(text.contains("outcome=ok\n"));
        assert!(text.ends_with("not display or input latency\n"));
    }
}
