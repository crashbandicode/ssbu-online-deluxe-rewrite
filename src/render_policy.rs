//! Live Vanilla, LessLag, LLUltra, and LLDoubles on one atomic word.
//!
//! A count change is a buffer swap: the opening present only forces the wait.
//! A later present may call the setter at the canonical boundary, then three
//! original flushes drain, the marker is accepted, and a fresh readback commits.
//! accept the marker, then commit from a fresh readback. A policy change is
//! the E1 index swap and does not run while that buffer swap is open.
//! Effective policy is the committed profile during a buffer swap, and the
//! frozen in-flight profile during an index swap. The modulus is
//! `min(requested count, known active)` so a 2→3 request cannot select index 2
//! early. The latest saved mode arrives with each present and does not replace
//! the target already being drained.

use crate::config::RenderMode;
use std::sync::atomic::{AtomicU64, Ordering};

/// Presents spent waiting for a setter boundary, or for an index drain marker.
/// A buffer drain that already called the setter does not stop. The eighth
/// marker-less present emits one code-46 kind-12 warning and stays in `Drain`.
/// Code 48 is unused. A bad readback after the marker still stops.
pub const DRAIN_TIMEOUT: u8 = 8;

pub const TRACE_TRANSITION: u16 = 46;
pub const TRACE_REFUSED: u16 = 47;

const KIND_STARTED: u64 = 1;
const KIND_COMMITTED: u64 = 3;
const KIND_SETTLED: u64 = 4;
const KIND_CLEARED: u64 = 5;
const KIND_EXPIRED_CANCEL: u64 = 6;
const KIND_READY: u64 = 7;
const KIND_BUFFER_STARTED: u64 = 8;
const KIND_BUFFER_SET: u64 = 9;
const KIND_BUFFER_READY: u64 = 10;
const KIND_BUFFER_COMMITTED: u64 = 11;
/// One warning that the post-setter drain marker has not arrived. Not a refusal.
const KIND_DRAIN_STALLED: u64 = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Profile {
    Ultra = 0,
    LessLag = 1,
    Vanilla = 2,
    /// 3 textures, OneBehind (+1), LessLag flush shape. Not `DoublesCore`.
    Doubles = 3,
}

impl Profile {
    const fn textures(self) -> u8 {
        match self {
            Self::Vanilla | Self::Doubles => 3,
            Self::Ultra | Self::LessLag => 2,
        }
    }

    const fn addend(self) -> u32 {
        match self {
            Self::Ultra => 0,
            Self::LessLag | Self::Doubles => 1,
            Self::Vanilla => 2,
        }
    }

    const fn render_opts(self) -> bool {
        !matches!(self, Self::Vanilla)
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Ultra => "ultra",
            Self::LessLag => "lesslag",
            Self::Vanilla => "vanilla",
            Self::Doubles => "lldoubles",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IndexPhase {
    Idle = 0,
    WaitMarker = 1,
    ReadyToCommit = 2,
    Invalid = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BufferPhase {
    Idle = 0,
    AwaitSetter = 1,
    SetterClaimed = 2,
    Drain = 3,
    Readback = 4,
    Invalid = 7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Machine {
    live: bool,
    committed: Profile,
    /// Frozen destination. Buffer swaps and index swaps are not open together.
    requested: Profile,
    index_phase: IndexPhase,
    buffer_phase: BufferPhase,
    index_marker: bool,
    buffer_marker: bool,
    /// Last count confirmed by a valid getter. 2 or 3 while live.
    known_active: u8,
    /// Count the open buffer swap is allowed to apply. Equals `known_active` while idle.
    count_target: u8,
    failed: bool,
    failed_target: Option<Profile>,
    waited: u8,
    generation: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Decision {
    pub live: bool,
    pub effective: Profile,
    pub committed: Profile,
    pub force_wait: bool,
    pub suppress_late: bool,
    pub early: bool,
    pub drain: bool,
    pub swapping: bool,
    pub failed: bool,
    /// Sequence-group optimizations. False for an effective Vanilla profile.
    pub render_opts: bool,
    /// `Some(2)` forces the safe modulus. `None` leaves the game's `% 3`.
    modulus: Option<u32>,
    addend: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlushPlan {
    pub calls: u32,
    pub lesslag_fields: bool,
    pub arm_drain: bool,
    pub generation: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentInput {
    pub saved: RenderMode,
    pub presented_index: i32,
    pub outstanding: i64,
    pub active: i32,
    pub allocated: i32,
    pub fresh: bool,
    /// The native setter pointer is resolved. A missing setter refuses before any claim.
    pub setter_ready: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitError {
    NotReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefuseReason {
    Timeout = 1,
    Invalid = 2,
    /// Boundary reached, but the getter did not match the known count.
    Unresolved = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepEvent {
    None,
    Waiting,
    Started {
        from: Profile,
        to: Profile,
    },
    Ready {
        from: Profile,
        to: Profile,
    },
    Committed {
        from: Profile,
        to: Profile,
    },
    Settled {
        policy: Profile,
    },
    Cleared {
        policy: Profile,
    },
    ExpiredCancel {
        policy: Profile,
    },
    Refused {
        reason: RefuseReason,
        retained: Profile,
        abandoned: Profile,
    },
    BufferStarted {
        from_count: u8,
        to_count: u8,
    },
    /// The present hook calls the setter, then `setter_returned`.
    SetTextures {
        count: i32,
        generation: u8,
    },
    BufferReady {
        count: u8,
    },
    BufferCommitted {
        count: u8,
    },
    /// The setter already ran and the drain marker is still absent at the warning bound.
    /// The machine stays in `Drain`. This is not a refusal and not a second setter.
    DrainStalled {
        known: u8,
        target: u8,
    },
    /// Fresh count after the marker is unusable. The runtime panics.
    CountStop {
        seen: i32,
        expected: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyLabels<'a> {
    pub committed: &'a str,
    pub effective: &'a str,
    pub requested: &'a str,
    pub swapping: bool,
    pub failed: bool,
    pub force_wait: bool,
    pub known_active: i32,
    pub count_target: i32,
    pub buffer: &'a str,
    pub index: &'a str,
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    word: u64,
    machine: Machine,
}

impl Machine {
    pub const fn passthrough() -> Self {
        Self {
            live: false,
            committed: Profile::Vanilla,
            requested: Profile::Vanilla,
            index_phase: IndexPhase::Idle,
            buffer_phase: BufferPhase::Idle,
            index_marker: false,
            buffer_marker: false,
            known_active: 0,
            count_target: 0,
            failed: false,
            failed_target: None,
            waited: 0,
            generation: 0,
        }
    }

    pub const fn confirmed(policy: Profile) -> Self {
        let count = policy.textures();
        Self {
            live: true,
            committed: policy,
            requested: policy,
            index_phase: IndexPhase::Idle,
            buffer_phase: BufferPhase::Idle,
            index_marker: false,
            buffer_marker: false,
            known_active: count,
            count_target: count,
            failed: false,
            failed_target: None,
            waited: 0,
            generation: 0,
        }
    }

    pub fn decision(self) -> Decision {
        let buffer = self.live && !matches!(self.buffer_phase, BufferPhase::Idle);
        let index = self.live
            && matches!(
                self.index_phase,
                IndexPhase::WaitMarker | IndexPhase::ReadyToCommit
            );
        let effective = if buffer || !index {
            self.committed
        } else {
            self.requested
        };
        let count_request = if buffer {
            self.count_target
        } else {
            self.known_active
        };
        let safe = if count_request < self.known_active {
            count_request
        } else {
            self.known_active
        };
        Decision {
            live: self.live,
            effective,
            committed: self.committed,
            force_wait: buffer || index,
            suppress_late: self.live && !index && matches!(effective, Profile::Ultra),
            early: self.live && matches!(effective, Profile::Ultra),
            drain: self.live
                && ((self.buffer_phase == BufferPhase::Drain && !self.buffer_marker)
                    || (self.index_phase == IndexPhase::WaitMarker
                        && !self.index_marker
                        && self.buffer_phase == BufferPhase::Idle)),
            swapping: buffer || index,
            failed: self.failed && !buffer && !index,
            render_opts: self.live && effective.render_opts(),
            modulus: if self.live && safe == 2 {
                Some(2)
            } else {
                None
            },
            addend: effective.addend(),
        }
    }
}

impl Snapshot {
    pub fn machine(self) -> Machine {
        self.machine
    }

    /// True when this present's getter must be current, not the 60-frame cache.
    pub fn needs_fresh(self, _saved: RenderMode, index: i32, outstanding: i64) -> bool {
        if !self.machine.live || index != 1 || outstanding != 0 {
            return self.machine.live && self.machine.buffer_phase == BufferPhase::Readback;
        }
        matches!(
            self.machine.buffer_phase,
            BufferPhase::AwaitSetter | BufferPhase::Readback
        )
    }
}

/// Tracked-window acquisitions still outstanding after one present returns.
/// A real zero stays zero; an unmatched present becomes negative.
pub const fn outstanding_after_present(before: i64) -> i64 {
    before.wrapping_sub(1)
}

pub fn index_value(input: u32, decision: Decision, armed: RenderMode) -> u32 {
    if decision.live {
        return match decision.modulus {
            Some(modulus) => input.wrapping_add(decision.addend) % modulus,
            None => input.wrapping_add(decision.addend),
        };
    }
    if armed == RenderMode::Vanilla {
        input.wrapping_add(2)
    } else {
        input.wrapping_add(u32::from(!armed.immediate())) % armed.textures() as u32
    }
}

pub fn skip_wait(decision: Decision, armed: RenderMode) -> bool {
    if decision.live {
        !decision.force_wait && decision.effective != Profile::Vanilla
    } else {
        armed != RenderMode::Vanilla
    }
}

/// Present the acquired image at the end of the flush that submitted it,
/// instead of where the game presents it: after the next frame's whole CPU
/// update. On the console character select that update is 13.7 ms (16.3 ms
/// with the hand moving), so a two-image ring misses the vsync and the next
/// acquire waits for the one after it (the exact 33 ms frames in the 0.3.7
/// and 2026-10-14 captures). Every render-opts profile, committed and idle:
/// LessLag since 0.4.0; LLUltra (the other two-image ring, which had kept the
/// game's late present and therefore the same stall) and LLDoubles (three
/// images, the earlier present is a latency gain) since 0.5.2. Never during a
/// swap, a drain, a forced wait or after a refusal. Vanilla keeps the game
/// order: its third image already absorbs a late present and it has no
/// render-opts frame shape to present against.
pub fn present_at_flush(decision: Decision) -> bool {
    decision.live
        && decision.effective == decision.committed
        && matches!(
            decision.committed,
            Profile::LessLag | Profile::Ultra | Profile::Doubles
        )
        && !decision.swapping
        && !decision.force_wait
        && !decision.failed
        && !decision.drain
}

/// Startup `PROBE_RENDER_POLL` gate. `probe` selects the original wait.
/// `skip` is left as `skip_wait` computed it. Not a profile change.
pub fn render_poll_calls_original(skip: bool, probe: bool) -> bool {
    probe || !skip
}

pub fn run_early(decision: Decision, armed: RenderMode) -> bool {
    if decision.live {
        decision.early
    } else {
        armed.immediate()
    }
}

pub fn skip_late(decision: Decision, armed: RenderMode) -> bool {
    if decision.live {
        decision.suppress_late
    } else {
        armed.immediate()
    }
}

pub fn plan_flush(machine: Machine, armed: RenderMode) -> FlushPlan {
    if !machine.live {
        let lesslag_fields = armed != RenderMode::Vanilla && !armed.immediate();
        let calls = if armed.immediate() { 3 } else { 1 };
        return FlushPlan {
            calls,
            lesslag_fields,
            arm_drain: false,
            generation: machine.generation,
        };
    }
    let decision = machine.decision();
    if decision.drain {
        return FlushPlan {
            calls: 3,
            lesslag_fields: false,
            arm_drain: true,
            generation: machine.generation,
        };
    }
    let (calls, lesslag_fields) = match decision.effective {
        Profile::Ultra => (3, false),
        Profile::LessLag | Profile::Doubles => (1, true),
        Profile::Vanilla => (1, false),
    };
    FlushPlan {
        calls,
        lesslag_fields,
        arm_drain: false,
        generation: machine.generation,
    }
}

pub fn complete_drain(machine: Machine, generation: u8) -> Option<Machine> {
    if !machine.live || machine.generation != generation {
        return None;
    }
    if machine.buffer_phase == BufferPhase::Drain && !machine.buffer_marker {
        let mut next = machine;
        next.buffer_marker = true;
        return Some(next);
    }
    if machine.buffer_phase == BufferPhase::Idle
        && machine.index_phase == IndexPhase::WaitMarker
        && !machine.index_marker
    {
        let mut next = machine;
        next.index_marker = true;
        return Some(next);
    }
    None
}

pub fn commit_index(machine: Machine) -> Result<Machine, CommitError> {
    if !machine.live || machine.index_phase != IndexPhase::ReadyToCommit {
        return Err(CommitError::NotReady);
    }
    let mut done = machine;
    done.committed = machine.requested;
    done.requested = machine.requested;
    done.index_phase = IndexPhase::Idle;
    done.index_marker = false;
    done.waited = 0;
    done.failed = false;
    done.failed_target = None;
    Ok(done)
}

pub fn step(machine: Machine, input: PresentInput) -> (Machine, StepEvent) {
    if !machine.live {
        return (machine, StepEvent::None);
    }
    if machine.index_phase == IndexPhase::Invalid || machine.buffer_phase == BufferPhase::Invalid {
        return refuse_invalid(machine);
    }
    if machine.buffer_phase != BufferPhase::Idle {
        return step_buffer(machine, input);
    }
    if machine.index_phase != IndexPhase::Idle {
        return step_index(machine);
    }
    step_idle(machine, input)
}

fn step_idle(machine: Machine, input: PresentInput) -> (Machine, StepEvent) {
    let Some(want) = saved_profile(input.saved) else {
        return (machine, StepEvent::None);
    };
    if machine.failed && want == machine.committed && want.textures() == machine.known_active {
        let mut next = machine;
        next.failed = false;
        next.failed_target = None;
        return (
            next,
            StepEvent::Cleared {
                policy: machine.committed,
            },
        );
    }
    if machine.failed && machine.failed_target == Some(want) {
        return (machine, StepEvent::None);
    }
    if want.textures() != machine.known_active {
        let started = begin_buffer(machine, want);
        return (
            started,
            StepEvent::BufferStarted {
                from_count: machine.known_active,
                to_count: started.count_target,
            },
        );
    }
    if want != machine.committed {
        return begin_index(machine, want);
    }
    (machine, StepEvent::None)
}

fn begin_buffer(machine: Machine, want: Profile) -> Machine {
    let mut next = machine;
    next.failed = false;
    next.failed_target = None;
    next.requested = want;
    next.count_target = want.textures();
    next.buffer_phase = BufferPhase::AwaitSetter;
    next.buffer_marker = false;
    next.index_phase = IndexPhase::Idle;
    next.index_marker = false;
    next.waited = 0;
    next.generation = machine.generation.wrapping_add(1);
    next
}

fn begin_index(machine: Machine, want: Profile) -> (Machine, StepEvent) {
    let mut next = machine;
    next.failed = false;
    next.failed_target = None;
    next.requested = want;
    next.index_phase = IndexPhase::WaitMarker;
    next.index_marker = false;
    next.waited = 0;
    next.generation = machine.generation.wrapping_add(1);
    (
        next,
        StepEvent::Started {
            from: machine.committed,
            to: want,
        },
    )
}

fn step_buffer(machine: Machine, input: PresentInput) -> (Machine, StepEvent) {
    match machine.buffer_phase {
        BufferPhase::AwaitSetter => step_await(machine, input),
        BufferPhase::SetterClaimed => (
            machine,
            StepEvent::CountStop {
                seen: input.active,
                expected: i32::from(machine.count_target),
            },
        ),
        BufferPhase::Drain => step_drain(machine),
        BufferPhase::Readback => step_readback(machine, input),
        BufferPhase::Idle | BufferPhase::Invalid => refuse_invalid(machine),
    }
}

fn boundary_ready(input: PresentInput) -> bool {
    input.presented_index == 1 && input.outstanding == 0 && input.fresh
}

fn step_await(machine: Machine, input: PresentInput) -> (Machine, StepEvent) {
    if boundary_ready(input) {
        if !matches!(input.active, 2 | 3) {
            return refuse_buffer(machine, RefuseReason::Invalid);
        }
        if !input.setter_ready
            || input.allocated != 3
            || input.active != i32::from(machine.known_active)
        {
            return refuse_buffer(machine, RefuseReason::Unresolved);
        }
        let mut next = machine;
        next.buffer_phase = BufferPhase::SetterClaimed;
        return (
            next,
            StepEvent::SetTextures {
                count: i32::from(machine.count_target),
                generation: machine.generation,
            },
        );
    }
    let mut next = machine;
    next.waited = next.waited.saturating_add(1);
    if next.waited >= DRAIN_TIMEOUT {
        return refuse_buffer(machine, RefuseReason::Timeout);
    }
    (next, StepEvent::Waiting)
}

fn step_drain(machine: Machine) -> (Machine, StepEvent) {
    if machine.buffer_marker {
        let mut next = machine;
        next.buffer_phase = BufferPhase::Readback;
        next.waited = 0;
        return (
            next,
            StepEvent::BufferReady {
                count: machine.count_target,
            },
        );
    }
    if machine.waited >= DRAIN_TIMEOUT {
        return (machine, StepEvent::Waiting);
    }
    let mut next = machine;
    next.waited = next.waited.saturating_add(1);
    if next.waited >= DRAIN_TIMEOUT {
        return (
            next,
            StepEvent::DrainStalled {
                known: machine.known_active,
                target: machine.count_target,
            },
        );
    }
    (next, StepEvent::Waiting)
}

fn step_readback(machine: Machine, input: PresentInput) -> (Machine, StepEvent) {
    let expected = i32::from(machine.count_target);
    if !input.fresh {
        return (machine, StepEvent::Waiting);
    }
    if !matches!(input.active, 2 | 3) {
        return (
            machine,
            StepEvent::CountStop {
                seen: input.active,
                expected,
            },
        );
    }
    if input.active == expected {
        let mut done = machine;
        done.known_active = machine.count_target;
        done.count_target = machine.count_target;
        done.buffer_phase = BufferPhase::Idle;
        done.buffer_marker = false;
        done.requested = machine.committed;
        done.waited = 0;
        done.failed = false;
        done.failed_target = None;
        return (
            done,
            StepEvent::BufferCommitted {
                count: machine.count_target,
            },
        );
    }
    let mut next = machine;
    next.waited = next.waited.saturating_add(1);
    if next.waited >= DRAIN_TIMEOUT {
        return (
            machine,
            StepEvent::CountStop {
                seen: input.active,
                expected,
            },
        );
    }
    (next, StepEvent::Waiting)
}

fn refuse_buffer(machine: Machine, reason: RefuseReason) -> (Machine, StepEvent) {
    let abandoned = machine.requested;
    let retained = machine.committed;
    let mut next = machine;
    next.buffer_phase = BufferPhase::Idle;
    next.buffer_marker = false;
    next.requested = retained;
    next.count_target = machine.known_active;
    next.waited = 0;
    next.failed = true;
    next.failed_target = Some(abandoned);
    (
        next,
        StepEvent::Refused {
            reason,
            retained,
            abandoned,
        },
    )
}

fn step_index(machine: Machine) -> (Machine, StepEvent) {
    if machine.index_marker {
        if machine.index_phase == IndexPhase::WaitMarker {
            let mut next = machine;
            next.index_phase = IndexPhase::ReadyToCommit;
            return (
                next,
                StepEvent::Ready {
                    from: machine.committed,
                    to: machine.requested,
                },
            );
        }
        return commit_ready(machine);
    }
    if machine.index_phase == IndexPhase::ReadyToCommit {
        return commit_ready(machine);
    }
    let mut next = machine;
    next.waited = next.waited.saturating_add(1);
    if next.waited >= DRAIN_TIMEOUT {
        let abandoned = next.requested;
        let retained = next.committed;
        next.requested = retained;
        next.index_phase = IndexPhase::Idle;
        next.index_marker = false;
        next.waited = 0;
        if abandoned == retained {
            next.failed = false;
            next.failed_target = None;
            return (next, StepEvent::ExpiredCancel { policy: retained });
        }
        next.failed = true;
        next.failed_target = Some(abandoned);
        return (
            next,
            StepEvent::Refused {
                reason: RefuseReason::Timeout,
                retained,
                abandoned,
            },
        );
    }
    (next, StepEvent::Waiting)
}

fn commit_ready(machine: Machine) -> (Machine, StepEvent) {
    let from = machine.committed;
    let to = machine.requested;
    let Ok(done) = commit_index(machine) else {
        return refuse_invalid(machine);
    };
    let event = if from == to {
        StepEvent::Settled { policy: to }
    } else {
        StepEvent::Committed { from, to }
    };
    (done, event)
}

fn refuse_invalid(machine: Machine) -> (Machine, StepEvent) {
    let retained = machine.committed;
    let abandoned = machine.requested;
    let mut next = machine;
    next.index_phase = IndexPhase::Idle;
    next.buffer_phase = BufferPhase::Idle;
    next.index_marker = false;
    next.buffer_marker = false;
    next.requested = retained;
    next.count_target = machine.known_active;
    next.waited = 0;
    next.failed = true;
    next.failed_target = None;
    (
        next,
        StepEvent::Refused {
            reason: RefuseReason::Invalid,
            retained,
            abandoned,
        },
    )
}

fn saved_profile(mode: RenderMode) -> Option<Profile> {
    match mode {
        RenderMode::Vanilla => Some(Profile::Vanilla),
        RenderMode::LessLag => Some(Profile::LessLag),
        RenderMode::LessLagUltra => Some(Profile::Ultra),
        RenderMode::LessLagDoubles => Some(Profile::Doubles),
        RenderMode::LessLagCore | RenderMode::DoublesCore => None,
    }
}

pub fn diagnostic(event: &StepEvent) -> Option<(u16, u64, u64)> {
    let pack = |kind: u64, from: Profile, to: Profile| {
        kind | (pack_profile(from) << 8) | (pack_profile(to) << 16)
    };
    match *event {
        StepEvent::Started { from, to } => {
            Some((TRACE_TRANSITION, pack(KIND_STARTED, from, to), 0))
        }
        StepEvent::Ready { from, to } => Some((TRACE_TRANSITION, pack(KIND_READY, from, to), 0)),
        StepEvent::Committed { from, to } => {
            Some((TRACE_TRANSITION, pack(KIND_COMMITTED, from, to), 0))
        }
        StepEvent::Settled { policy } => {
            Some((TRACE_TRANSITION, pack(KIND_SETTLED, policy, policy), 0))
        }
        StepEvent::Cleared { policy } => {
            Some((TRACE_TRANSITION, pack(KIND_CLEARED, policy, policy), 0))
        }
        StepEvent::ExpiredCancel { policy } => Some((
            TRACE_TRANSITION,
            pack(KIND_EXPIRED_CANCEL, policy, policy),
            0,
        )),
        StepEvent::BufferStarted {
            from_count,
            to_count,
        } => Some((
            TRACE_TRANSITION,
            KIND_BUFFER_STARTED,
            u64::from(from_count) | (u64::from(to_count) << 8),
        )),
        StepEvent::SetTextures { count, generation } => Some((
            TRACE_TRANSITION,
            KIND_BUFFER_SET | (u64::from(generation) << 8),
            count as u64,
        )),
        StepEvent::BufferReady { count } => {
            Some((TRACE_TRANSITION, KIND_BUFFER_READY, u64::from(count)))
        }
        StepEvent::BufferCommitted { count } => {
            Some((TRACE_TRANSITION, KIND_BUFFER_COMMITTED, u64::from(count)))
        }
        StepEvent::DrainStalled { known, target } => Some((
            TRACE_TRANSITION,
            KIND_DRAIN_STALLED,
            u64::from(known) | (u64::from(target) << 8),
        )),
        StepEvent::Refused {
            reason,
            retained,
            abandoned,
        } => Some((
            TRACE_REFUSED,
            (reason as u64) | (pack_profile(retained) << 8) | (pack_profile(abandoned) << 16),
            0,
        )),
        StepEvent::CountStop { seen, expected } => Some((
            TRACE_REFUSED,
            3,
            (seen as u32 as u64) << 32 | (expected as u32 as u64),
        )),
        StepEvent::None | StepEvent::Waiting => None,
    }
}

pub fn profile_status(
    saved: RenderMode,
    armed: RenderMode,
    textures: i32,
    machine: Machine,
    applied: bool,
) -> &'static str {
    if !applied {
        return "";
    }
    if active_now(saved, textures, machine) {
        return "Active";
    }
    if machine.live && machine.failed && saved_profile(saved) != Some(machine.committed) {
        return "Refused";
    }
    if machine.live && machine.decision().swapping && saved_profile(saved).is_some() {
        return "Swap";
    }
    if awaiting_index(saved, machine) {
        return "Swap";
    }
    if !machine.live && saved == armed && saved == RenderMode::Vanilla {
        return "Vanilla";
    }
    if saved_profile(saved).is_none() && saved != RenderMode::Vanilla {
        return "Restart";
    }
    "Restart"
}

pub fn engine_status(
    armed: RenderMode,
    saved: RenderMode,
    textures: i32,
    machine: Machine,
) -> &'static str {
    if machine.live && machine.failed && saved_profile(saved) != Some(machine.committed) {
        return "Switch refused";
    }
    if machine.live
        && saved_profile(saved).is_some()
        && (machine.decision().swapping || awaiting_index(saved, machine))
    {
        return "Switching";
    }
    if active_now(saved, textures, machine) {
        return "Active";
    }
    if !machine.live && armed == RenderMode::Vanilla && saved == RenderMode::Vanilla {
        return "Vanilla";
    }
    if !machine.live && !armed.render_opts() && armed != RenderMode::Vanilla {
        return "Core only; restart";
    }
    "Restart to apply"
}

/// Count is already the saved profile's count, and the index swap has not opened.
fn awaiting_index(saved: RenderMode, machine: Machine) -> bool {
    if !machine.live
        || machine.failed
        || machine.buffer_phase != BufferPhase::Idle
        || machine.index_phase != IndexPhase::Idle
    {
        return false;
    }
    match saved_profile(saved) {
        Some(want) => want != machine.committed && want.textures() == machine.known_active,
        None => false,
    }
}

fn active_now(saved: RenderMode, textures: i32, machine: Machine) -> bool {
    let Some(want) = saved_profile(saved) else {
        return false;
    };
    machine.live
        && !machine.failed
        && machine.buffer_phase == BufferPhase::Idle
        && machine.index_phase == IndexPhase::Idle
        && want == machine.committed
        && textures == i32::from(machine.known_active)
        && machine.known_active == want.textures()
        && matches!(textures, 2 | 3)
}

pub fn labels(machine: Machine) -> PolicyLabels<'static> {
    if !machine.live {
        return PolicyLabels {
            committed: "passthrough",
            effective: "passthrough",
            requested: "passthrough",
            swapping: false,
            failed: false,
            force_wait: false,
            known_active: -1,
            count_target: -1,
            buffer: "idle",
            index: "idle",
        };
    }
    let decision = machine.decision();
    PolicyLabels {
        committed: machine.committed.key(),
        effective: decision.effective.key(),
        requested: machine.requested.key(),
        swapping: decision.swapping,
        failed: decision.failed,
        force_wait: decision.force_wait,
        known_active: i32::from(machine.known_active),
        count_target: i32::from(machine.count_target),
        buffer: buffer_name(machine.buffer_phase),
        index: index_name(machine.index_phase),
    }
}

fn buffer_name(phase: BufferPhase) -> &'static str {
    match phase {
        BufferPhase::Idle => "idle",
        BufferPhase::AwaitSetter => "await",
        BufferPhase::SetterClaimed => "claimed",
        BufferPhase::Drain => "drain",
        BufferPhase::Readback => "readback",
        BufferPhase::Invalid => "invalid",
    }
}

fn index_name(phase: IndexPhase) -> &'static str {
    match phase {
        IndexPhase::Idle => "idle",
        IndexPhase::WaitMarker => "wait",
        IndexPhase::ReadyToCommit => "ready",
        IndexPhase::Invalid => "invalid",
    }
}

fn pack_profile(policy: Profile) -> u64 {
    match policy {
        Profile::Ultra => 0,
        Profile::LessLag => 1,
        Profile::Vanilla => 2,
        Profile::Doubles => 3,
    }
}

fn unpack_profile(bits: u64) -> Option<Profile> {
    match bits & 0b111 {
        0 => Some(Profile::Ultra),
        1 => Some(Profile::LessLag),
        2 => Some(Profile::Vanilla),
        3 => Some(Profile::Doubles),
        _ => None,
    }
}

/// 0 is "no failed target", not Ultra. 7 is the corrupt sentinel.
fn pack_failed_target(target: Option<Profile>) -> u64 {
    match target {
        None => 0,
        Some(Profile::Ultra) => 1,
        Some(Profile::LessLag) => 2,
        Some(Profile::Vanilla) => 3,
        Some(Profile::Doubles) => 4,
    }
}

fn unpack_failed_target(bits: u64) -> Result<Option<Profile>, ()> {
    match bits & 0b111 {
        0 => Ok(None),
        1 => Ok(Some(Profile::Ultra)),
        2 => Ok(Some(Profile::LessLag)),
        3 => Ok(Some(Profile::Vanilla)),
        4 => Ok(Some(Profile::Doubles)),
        _ => Err(()),
    }
}

fn encode(machine: Machine) -> u64 {
    let known_bit = u64::from(machine.known_active == 3);
    let target_bit = u64::from(machine.count_target == 3);
    u64::from(machine.live)
        | (pack_profile(machine.committed) << 1)
        | (pack_profile(machine.requested) << 4)
        | ((machine.index_phase as u64) << 7)
        | ((machine.buffer_phase as u64) << 9)
        | (u64::from(machine.index_marker) << 12)
        | (u64::from(machine.buffer_marker) << 13)
        | (known_bit << 14)
        | (target_bit << 15)
        | (u64::from(machine.failed) << 16)
        | (pack_failed_target(machine.failed_target) << 17)
        | (u64::from(machine.waited) << 20)
        | (u64::from(machine.generation) << 28)
}

fn decode(word: u64) -> Machine {
    let committed = unpack_profile(word >> 1);
    let requested = unpack_profile(word >> 4);
    let index_phase = match (word >> 7) & 0b11 {
        0 => IndexPhase::Idle,
        1 => IndexPhase::WaitMarker,
        2 => IndexPhase::ReadyToCommit,
        _ => IndexPhase::Invalid,
    };
    let buffer_phase = match (word >> 9) & 0b111 {
        0 => BufferPhase::Idle,
        1 => BufferPhase::AwaitSetter,
        2 => BufferPhase::SetterClaimed,
        3 => BufferPhase::Drain,
        4 => BufferPhase::Readback,
        _ => BufferPhase::Invalid,
    };
    let failed_target = unpack_failed_target(word >> 17);
    let known_active = if (word >> 14) & 1 == 1 { 3 } else { 2 };
    let count_target = if (word >> 15) & 1 == 1 { 3 } else { 2 };
    let corrupt = committed.is_none() || requested.is_none() || failed_target.is_err();
    Machine {
        live: word & 1 == 1,
        committed: committed.unwrap_or(Profile::Vanilla),
        requested: requested.unwrap_or(Profile::Vanilla),
        index_phase: if corrupt {
            IndexPhase::Invalid
        } else {
            index_phase
        },
        buffer_phase: if corrupt {
            BufferPhase::Invalid
        } else {
            buffer_phase
        },
        index_marker: (word >> 12) & 1 == 1,
        buffer_marker: (word >> 13) & 1 == 1,
        known_active,
        count_target,
        failed: (word >> 16) & 1 == 1,
        failed_target: failed_target.unwrap_or(None),
        waited: ((word >> 20) & 0xff) as u8,
        generation: ((word >> 28) & 0xff) as u8,
    }
}

fn apply_slot(slot: &AtomicU64, snap: Snapshot, input: PresentInput) -> StepEvent {
    let (next, event) = step(snap.machine, input);
    if next == snap.machine {
        return event;
    }
    if slot
        .compare_exchange(snap.word, encode(next), Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        event
    } else {
        StepEvent::None
    }
}

fn release_unclaimed_slot(slot: &AtomicU64, generation: u8) -> bool {
    let current = slot.load(Ordering::Acquire);
    let machine = decode(current);
    if machine.buffer_phase != BufferPhase::SetterClaimed || machine.generation != generation {
        return false;
    }
    let (next, _) = refuse_buffer(machine, RefuseReason::Unresolved);
    slot.compare_exchange(current, encode(next), Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

fn setter_returned_slot(slot: &AtomicU64, generation: u8) -> bool {
    let current = slot.load(Ordering::Acquire);
    let machine = decode(current);
    if machine.buffer_phase != BufferPhase::SetterClaimed || machine.generation != generation {
        return false;
    }
    let mut next = machine;
    next.buffer_phase = BufferPhase::Drain;
    next.buffer_marker = false;
    slot.compare_exchange(current, encode(next), Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

fn publish_drain_slot(slot: &AtomicU64, generation: u8) -> bool {
    let current = slot.load(Ordering::Acquire);
    let Some(next) = complete_drain(decode(current), generation) else {
        return false;
    };
    slot.compare_exchange(current, encode(next), Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

static PUBLISHED: AtomicU64 = AtomicU64::new(0);

#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn arm(_mode: RenderMode) {
    PUBLISHED.store(encode(Machine::passthrough()), Ordering::Release);
}

#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn confirm(mode: RenderMode, active: i32) {
    let Some(policy) = saved_profile(mode) else {
        return;
    };
    if i32::from(policy.textures()) != active {
        return;
    }
    PUBLISHED.store(encode(Machine::confirmed(policy)), Ordering::Release);
}

#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn load_machine() -> Machine {
    decode(PUBLISHED.load(Ordering::Acquire))
}

#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn load_decision() -> Decision {
    load_machine().decision()
}

#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn snapshot() -> Snapshot {
    let word = PUBLISHED.load(Ordering::Acquire);
    Snapshot {
        word,
        machine: decode(word),
    }
}

#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn apply(snap: Snapshot, input: PresentInput) -> StepEvent {
    apply_slot(&PUBLISHED, snap, input)
}

#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn setter_returned(generation: u8) -> bool {
    setter_returned_slot(&PUBLISHED, generation)
}

/// The setter pointer was gone after a claim. Puts the known count back. No setter call.
#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn release_unclaimed(generation: u8) -> bool {
    release_unclaimed_slot(&PUBLISHED, generation)
}

#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn observe_flush() -> Option<FlushPlan> {
    let machine = decode(PUBLISHED.load(Ordering::Acquire));
    if !machine.live {
        return None;
    }
    Some(plan_flush(machine, RenderMode::Vanilla))
}

#[cfg_attr(not(target_os = "switch"), allow(dead_code))]
pub fn publish_drain(generation: u8) -> bool {
    publish_drain_slot(&PUBLISHED, generation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_return_keeps_a_real_zero_and_an_unmatched_present() {
        assert_eq!(outstanding_after_present(1), 0);
        assert_eq!(outstanding_after_present(0), -1);
        assert_eq!(outstanding_after_present(2), 1);
    }

    fn present(saved: RenderMode, index: i32, outstanding: i64, active: i32) -> PresentInput {
        PresentInput {
            saved,
            presented_index: index,
            outstanding,
            active,
            allocated: 3,
            fresh: true,
            setter_ready: true,
        }
    }

    fn open_index(from: Profile, to: Profile) -> Machine {
        let saved = match to {
            Profile::Ultra => RenderMode::LessLagUltra,
            Profile::LessLag => RenderMode::LessLag,
            Profile::Vanilla => RenderMode::Vanilla,
            Profile::Doubles => RenderMode::LessLagDoubles,
        };
        let (machine, event) = step(
            Machine::confirmed(from),
            present(saved, 0, 0, i32::from(from.textures())),
        );
        assert_eq!(event, StepEvent::Started { from, to });
        assert!(commit_index(machine).is_err());
        assert!(machine.decision().force_wait);
        assert!(!machine.decision().suppress_late);
        machine
    }

    fn mark_index(machine: Machine) -> Machine {
        let plan = plan_flush(machine, RenderMode::Vanilla);
        assert!(plan.arm_drain);
        assert_eq!(plan.calls, 3);
        let marked = complete_drain(machine, plan.generation).expect("generation");
        assert!(complete_drain(marked, plan.generation).is_none());
        assert!(complete_drain(machine, plan.generation.wrapping_add(1)).is_none());
        marked
    }

    #[test]
    fn index_swap_waits_for_ready_and_keeps_a_queued_request() {
        let opened = open_index(Profile::LessLag, Profile::Ultra);
        let (waiting, event) = step(opened, present(RenderMode::LessLag, 0, 0, 2));
        assert_eq!(event, StepEvent::Waiting);
        assert_eq!(waiting.requested, Profile::Ultra);
        let marked = mark_index(waiting);
        let (ready, event) = step(marked, present(RenderMode::LessLag, 1, 0, 2));
        assert_eq!(
            event,
            StepEvent::Ready {
                from: Profile::LessLag,
                to: Profile::Ultra
            }
        );
        assert_eq!(ready.committed, Profile::LessLag);
        assert!(ready.decision().force_wait);
        assert!(!ready.decision().suppress_late);
        assert!(ready.decision().early);
        let (done, event) = step(ready, present(RenderMode::LessLag, 0, 0, 2));
        assert_eq!(
            event,
            StepEvent::Committed {
                from: Profile::LessLag,
                to: Profile::Ultra
            }
        );
        assert_eq!(done.committed, Profile::Ultra);
        let (again, event) = step(done, present(RenderMode::LessLag, 0, 0, 2));
        assert_eq!(
            event,
            StepEvent::Started {
                from: Profile::Ultra,
                to: Profile::LessLag
            }
        );
        assert!(commit_index(again).is_err());
        assert!(again.decision().drain);
    }

    #[test]
    fn vanilla_reaches_a_two_texture_ring_only_after_readback() {
        let start = Machine::confirmed(Profile::Vanilla);
        assert_eq!(start.known_active, 3);
        let (opened, event) = step(start, present(RenderMode::LessLagUltra, 0, 0, 3));
        assert_eq!(
            event,
            StepEvent::BufferStarted {
                from_count: 3,
                to_count: 2
            }
        );
        assert_eq!(opened.committed, Profile::Vanilla);
        assert_eq!(opened.requested, Profile::Ultra);
        assert_eq!(opened.index_phase, IndexPhase::Idle);
        assert!(opened.decision().force_wait);
        assert!(!opened.decision().render_opts);
        let (queued, _) = step(opened, present(RenderMode::LessLag, 0, 0, 3));
        assert_eq!(queued.requested, Profile::Ultra);
        assert_eq!(queued.count_target, 2);
        let (blocked, event) = step(opened, present(RenderMode::LessLagUltra, 1, 1, 3));
        assert_eq!(event, StepEvent::Waiting);
        assert_eq!(blocked.buffer_phase, BufferPhase::AwaitSetter);
        let (claimed, event) = step(opened, present(RenderMode::LessLag, 1, 0, 3));
        assert_eq!(
            event,
            StepEvent::SetTextures {
                count: 2,
                generation: opened.generation
            }
        );
        assert_eq!(claimed.known_active, 3);
        let slot = AtomicU64::new(encode(claimed));
        assert!(setter_returned_slot(&slot, opened.generation));
        assert!(!setter_returned_slot(&slot, opened.generation));
        let draining = decode(slot.load(Ordering::Acquire));
        assert_eq!(draining.buffer_phase, BufferPhase::Drain);
        let plan = plan_flush(draining, RenderMode::Vanilla);
        assert!(plan.arm_drain);
        assert!(!plan_flush(opened, RenderMode::Vanilla).arm_drain);
        let marked = complete_drain(draining, plan.generation).unwrap();
        let (seen, event) = step(marked, present(RenderMode::LessLag, 0, 0, 3));
        assert_eq!(event, StepEvent::BufferReady { count: 2 });
        assert_eq!(seen.known_active, 3);
        assert_eq!(seen.committed, Profile::Vanilla);
        let (counted, event) = step(seen, present(RenderMode::LessLag, 1, 0, 2));
        assert_eq!(event, StepEvent::BufferCommitted { count: 2 });
        assert_eq!(counted.known_active, 2);
        assert_eq!(counted.committed, Profile::Vanilla);
        assert_eq!(counted.index_phase, IndexPhase::Idle);
        let (index, event) = step(counted, present(RenderMode::LessLag, 0, 0, 2));
        assert_eq!(
            event,
            StepEvent::Started {
                from: Profile::Vanilla,
                to: Profile::LessLag
            }
        );
        assert!(commit_index(index).is_err());
    }

    #[test]
    fn getter_problems_refuse_before_the_setter_and_stop_after_it() {
        let start = Machine::confirmed(Profile::Vanilla);
        let (opened, _) = step(start, present(RenderMode::LessLag, 0, 0, 3));
        for active in [-1, 0, 1, 4] {
            let (kept, event) = step(opened, present(RenderMode::LessLag, 1, 0, active));
            assert!(matches!(
                event,
                StepEvent::Refused {
                    reason: RefuseReason::Invalid,
                    ..
                }
            ));
            assert_eq!(kept.known_active, 3);
            assert_eq!(kept.buffer_phase, BufferPhase::Idle);
            assert!(!matches!(event, StepEvent::SetTextures { .. }));
        }
        let mut unresolved = present(RenderMode::LessLag, 1, 0, 2);
        unresolved.fresh = true;
        let (kept, event) = step(opened, unresolved);
        assert!(matches!(
            event,
            StepEvent::Refused {
                reason: RefuseReason::Unresolved,
                ..
            }
        ));
        assert_eq!(kept.known_active, 3);
        let mut short = present(RenderMode::LessLag, 1, 0, 3);
        short.allocated = 2;
        let (kept, _) = step(opened, short);
        assert_eq!(kept.known_active, 3);
        assert_eq!(kept.buffer_phase, BufferPhase::Idle);

        let (claimed, _) = step(opened, present(RenderMode::LessLag, 1, 0, 3));
        let slot = AtomicU64::new(encode(claimed));
        assert!(setter_returned_slot(&slot, claimed.generation));
        let draining = decode(slot.load(Ordering::Acquire));
        let marked = complete_drain(draining, draining.generation).unwrap();
        let (ready, _) = step(marked, present(RenderMode::LessLag, 0, 0, 3));
        let (stopped, event) = step(ready, present(RenderMode::LessLag, 1, 0, 0));
        assert_eq!(
            event,
            StepEvent::CountStop {
                seen: 0,
                expected: 2
            }
        );
        assert_eq!(stopped.known_active, 3);
        assert_eq!(stopped.buffer_phase, BufferPhase::Readback);
        let mut old = ready;
        for _ in 0..(DRAIN_TIMEOUT - 1) {
            let (next, event) = step(old, present(RenderMode::LessLag, 1, 0, 3));
            assert_eq!(event, StepEvent::Waiting);
            old = next;
        }
        let (stopped, event) = step(old, present(RenderMode::LessLag, 1, 0, 3));
        assert_eq!(
            event,
            StepEvent::CountStop {
                seen: 3,
                expected: 2
            }
        );
        assert_eq!(stopped.known_active, 3);
    }

    #[test]
    fn the_opening_present_never_sets_even_when_index_one_is_already_valid() {
        let start = Machine::confirmed(Profile::Vanilla);
        let boundary = present(RenderMode::LessLag, 1, 0, 3);
        let (opened, event) = step(start, boundary);
        assert_eq!(
            event,
            StepEvent::BufferStarted {
                from_count: 3,
                to_count: 2
            }
        );
        assert_eq!(opened.buffer_phase, BufferPhase::AwaitSetter);
        assert!(opened.decision().force_wait);
        assert_ne!(opened.known_active, opened.count_target);
        let (claimed, event) = step(opened, boundary);
        assert_eq!(
            event,
            StepEvent::SetTextures {
                count: 2,
                generation: opened.generation
            }
        );
        assert_eq!(claimed.buffer_phase, BufferPhase::SetterClaimed);
        let slot = AtomicU64::new(encode(claimed));
        assert!(setter_returned_slot(&slot, opened.generation));
        let draining = decode(slot.load(Ordering::Acquire));
        let marked = complete_drain(draining, draining.generation).unwrap();
        let (seen, event) = step(marked, boundary);
        assert_eq!(event, StepEvent::BufferReady { count: 2 });
        assert_eq!(seen.known_active, 3);
        let (done, event) = step(seen, present(RenderMode::LessLag, 0, 0, 2));
        assert_eq!(event, StepEvent::BufferCommitted { count: 2 });
        assert_eq!(done.known_active, 2);
        assert_eq!(done.committed, Profile::Vanilla);
        assert_eq!(done.index_phase, IndexPhase::Idle);

        let (again, _) = step(start, present(RenderMode::LessLagUltra, 0, 0, 3));
        let mut missing = present(RenderMode::LessLagUltra, 1, 0, 3);
        missing.setter_ready = false;
        let (kept, event) = step(again, missing);
        assert_eq!(
            event,
            StepEvent::Refused {
                reason: RefuseReason::Unresolved,
                retained: Profile::Vanilla,
                abandoned: Profile::Ultra,
            }
        );
        assert_eq!(kept.known_active, 3);
        assert_eq!(kept.buffer_phase, BufferPhase::Idle);
        assert_eq!(kept.committed, Profile::Vanilla);
        let claimed_word = AtomicU64::new(encode(claimed));
        assert!(release_unclaimed_slot(&claimed_word, claimed.generation));
        let restored = decode(claimed_word.load(Ordering::Acquire));
        assert_eq!(restored.known_active, 3);
        assert_eq!(restored.buffer_phase, BufferPhase::Idle);
        assert_eq!(restored.committed, Profile::Vanilla);
    }

    #[test]
    fn setter_preconditions_time_out_without_a_claim() {
        let start = Machine::confirmed(Profile::LessLag);
        let (mut machine, event) = step(start, present(RenderMode::Vanilla, 0, 0, 2));
        assert_eq!(
            event,
            StepEvent::BufferStarted {
                from_count: 2,
                to_count: 3
            }
        );
        for _ in 0..(DRAIN_TIMEOUT - 1) {
            let (next, event) = step(machine, present(RenderMode::Vanilla, 0, 0, 2));
            assert_eq!(event, StepEvent::Waiting);
            machine = next;
        }
        let (kept, event) = step(machine, present(RenderMode::Vanilla, 0, 0, 2));
        assert!(matches!(
            event,
            StepEvent::Refused {
                reason: RefuseReason::Timeout,
                ..
            }
        ));
        assert_eq!(kept.known_active, 2);
        assert_eq!(kept.committed, Profile::LessLag);
        assert_eq!(kept.buffer_phase, BufferPhase::Idle);
    }

    #[test]
    fn safe_modulus_never_emits_index_two_before_the_ring_grows() {
        let start = Machine::confirmed(Profile::LessLag);
        let (toward_three, _) = step(start, present(RenderMode::Vanilla, 0, 0, 2));
        let decision = toward_three.decision();
        assert_eq!(decision.modulus, Some(2));
        for input in 0..32 {
            let index = index_value(input, decision, RenderMode::LessLag);
            assert!(index < 2, "{index}");
        }
        let back = Machine::confirmed(Profile::Vanilla);
        let (toward_two, _) = step(back, present(RenderMode::LessLagUltra, 0, 0, 3));
        let decision = toward_two.decision();
        for input in 0..32 {
            assert!(index_value(input, decision, RenderMode::Vanilla) < 2);
        }
        let steady = Machine::confirmed(Profile::Vanilla).decision();
        assert_eq!(index_value(5, steady, RenderMode::Vanilla), 7);
        assert_eq!(index_value(u32::MAX, steady, RenderMode::Vanilla), 1);
        assert!(steady.modulus.is_none());
        let less = Machine::confirmed(Profile::LessLag).decision();
        assert_eq!(index_value(4, less, RenderMode::LessLag), 1);
        assert_eq!(index_value(5, less, RenderMode::LessLag), 0);
        let ultra = Machine::confirmed(Profile::Ultra).decision();
        assert_eq!(index_value(4, ultra, RenderMode::LessLagUltra), 0);
        assert_eq!(index_value(5, ultra, RenderMode::LessLagUltra), 1);
    }

    #[test]
    fn steady_shapes_follow_the_effective_profile() {
        let vanilla = Machine::confirmed(Profile::Vanilla);
        let decision = vanilla.decision();
        assert!(!skip_wait(decision, RenderMode::Vanilla));
        assert!(!run_early(decision, RenderMode::Vanilla));
        assert!(!skip_late(decision, RenderMode::Vanilla));
        assert!(!decision.render_opts);
        let plan = plan_flush(vanilla, RenderMode::Vanilla);
        assert_eq!(
            (plan.calls, plan.lesslag_fields, plan.arm_drain),
            (1, false, false)
        );
        let less = Machine::confirmed(Profile::LessLag);
        let decision = less.decision();
        assert!(skip_wait(decision, RenderMode::Vanilla));
        assert!(!run_early(decision, RenderMode::Vanilla));
        assert!(!skip_late(decision, RenderMode::Vanilla));
        assert!(decision.render_opts);
        let plan = plan_flush(less, RenderMode::Vanilla);
        assert_eq!((plan.calls, plan.lesslag_fields), (1, true));
        let ultra = Machine::confirmed(Profile::Ultra);
        let decision = ultra.decision();
        assert!(run_early(decision, RenderMode::Vanilla));
        assert!(skip_late(decision, RenderMode::Vanilla));
        assert_eq!(plan_flush(ultra, RenderMode::Vanilla).calls, 3);
        let (toward_less, _) = step(vanilla, present(RenderMode::LessLag, 0, 0, 3));
        assert!(!toward_less.decision().render_opts);
        assert_eq!(toward_less.buffer_phase, BufferPhase::AwaitSetter);
    }

    #[test]
    fn committed_idle_render_opts_profiles_present_at_the_flush() {
        assert!(present_at_flush(
            Machine::confirmed(Profile::LessLag).decision()
        ));
        assert!(!present_at_flush(
            Machine::confirmed(Profile::Vanilla).decision()
        ));
        // 0.5.2: the other two-image ring and the three-image Doubles too.
        assert!(present_at_flush(
            Machine::confirmed(Profile::Ultra).decision()
        ));
        assert!(present_at_flush(
            Machine::confirmed(Profile::Doubles).decision()
        ));
        // A swap toward LessLag is not LessLag yet.
        let (toward_less, _) = step(
            Machine::confirmed(Profile::Vanilla),
            present(RenderMode::LessLag, 0, 0, 3),
        );
        assert!(!present_at_flush(toward_less.decision()));
        // Before the window count is confirmed nothing is live.
        let armed = Decision {
            live: false,
            ..Machine::confirmed(Profile::LessLag).decision()
        };
        assert!(!present_at_flush(armed));
    }

    #[test]
    fn active_requires_a_verified_count_and_the_cell_stays_short() {
        let vanilla = Machine::confirmed(Profile::Vanilla);
        assert_eq!(
            profile_status(RenderMode::Vanilla, RenderMode::Vanilla, 3, vanilla, true),
            "Active"
        );
        assert_eq!(
            engine_status(RenderMode::Vanilla, RenderMode::Vanilla, 3, vanilla),
            "Active"
        );
        for textures in [-1, 0, 1, 4] {
            assert_ne!(
                profile_status(
                    RenderMode::Vanilla,
                    RenderMode::Vanilla,
                    textures,
                    vanilla,
                    true
                ),
                "Active"
            );
            assert_ne!(
                engine_status(RenderMode::Vanilla, RenderMode::Vanilla, textures, vanilla),
                "Active"
            );
        }
        let less = Machine::confirmed(Profile::LessLag);
        assert_eq!(
            profile_status(RenderMode::LessLag, RenderMode::LessLag, 2, less, true),
            "Active"
        );
        assert_ne!(
            engine_status(RenderMode::LessLag, RenderMode::LessLag, 3, less),
            "Active"
        );
        assert_ne!(
            engine_status(RenderMode::Vanilla, RenderMode::Vanilla, 2, vanilla),
            "Active"
        );
        let (swap, _) = step(vanilla, present(RenderMode::LessLagUltra, 0, 0, 3));
        assert_eq!(
            profile_status(RenderMode::LessLagUltra, RenderMode::Vanilla, 3, swap, true),
            "Swap"
        );
        assert!(!awaiting_index(RenderMode::LessLagUltra, swap));
        assert_eq!(
            profile_status(
                RenderMode::LessLag,
                RenderMode::Vanilla,
                3,
                Machine::passthrough(),
                true
            ),
            "Restart"
        );
        let labels = labels(vanilla);
        assert_eq!(labels.committed, "vanilla");
        assert_eq!(labels.known_active, 3);
        assert_eq!(labels.buffer, "idle");
    }

    #[test]
    fn a_stalled_buffer_drain_waits_and_keeps_ultra_late_suppression() {
        let (opened, started) = step(
            Machine::confirmed(Profile::Ultra),
            present(RenderMode::Vanilla, 0, 0, 2),
        );
        assert!(matches!(started, StepEvent::BufferStarted { .. }));
        assert!(opened.decision().suppress_late);
        assert!(opened.decision().force_wait);
        assert_eq!(opened.decision().modulus, Some(2));
        let (claimed, set) = step(opened, present(RenderMode::Vanilla, 1, 0, 2));
        assert!(matches!(set, StepEvent::SetTextures { count: 3, .. }));
        let mut machine = claimed;
        machine.buffer_phase = BufferPhase::Drain;
        machine.buffer_marker = false;
        machine.waited = 0;
        let mut warnings = 0u32;
        for index in 0u32..60 {
            assert!(index_value(index, machine.decision(), RenderMode::Vanilla) < 2);
            let (next, event) = step(machine, present(RenderMode::Vanilla, index as i32, 0, 2));
            match event {
                StepEvent::DrainStalled {
                    known: 2,
                    target: 3,
                } => {
                    warnings += 1;
                    assert_eq!(
                        diagnostic(&event),
                        Some((TRACE_TRANSITION, 12, 2 | (3 << 8)))
                    );
                }
                StepEvent::Waiting => {}
                other => panic!("drain left its wait: {other:?}"),
            }
            machine = next;
            assert_eq!(machine.buffer_phase, BufferPhase::Drain);
            assert!(!machine.buffer_marker);
            assert!(machine.decision().force_wait);
            assert!(machine.decision().suppress_late);
            assert_eq!(machine.decision().modulus, Some(2));
            assert_eq!(machine.known_active, 2);
            assert_eq!(machine.committed, Profile::Ultra);
        }
        assert_eq!(warnings, 1);
        assert_eq!(machine.waited, DRAIN_TIMEOUT);

        let marked = complete_drain(machine, machine.generation).expect("marker");
        let (ready, ready_event) = step(marked, present(RenderMode::Vanilla, 1, 0, 2));
        assert!(matches!(ready_event, StepEvent::BufferReady { count: 3 }));
        let (committed, committed_event) = step(ready, present(RenderMode::Vanilla, 0, 0, 3));
        assert!(matches!(
            committed_event,
            StepEvent::BufferCommitted { count: 3 }
        ));
        assert_eq!(committed.buffer_phase, BufferPhase::Idle);
        assert_eq!(committed.index_phase, IndexPhase::Idle);
        assert_eq!(committed.committed, Profile::Ultra);
        assert_eq!(committed.known_active, 3);
        assert_eq!(
            profile_status(RenderMode::Vanilla, RenderMode::Vanilla, 3, committed, true),
            "Swap"
        );
        // The committed buffer is still awaiting its index swap, not a refusal.
        assert!(awaiting_index(RenderMode::Vanilla, committed));
        assert_eq!(
            engine_status(RenderMode::Vanilla, RenderMode::Vanilla, 3, committed),
            "Switching"
        );
        let (index, index_event) = step(committed, present(RenderMode::Vanilla, 0, 0, 3));
        assert!(matches!(index_event, StepEvent::Started { .. }));
        assert!(!index.decision().suppress_late);
    }

    #[test]
    fn a_corrupt_word_refuses_without_adopting_the_request() {
        let mut word = encode(Machine::confirmed(Profile::LessLag));
        // 3-bit fields reserve 7. The old 2-bit pattern `3 << 1` is now Doubles.
        word = (word & !(0b111 << 1)) | (7 << 1);
        let machine = decode(word);
        assert_eq!(machine.index_phase, IndexPhase::Invalid);
        assert_eq!(machine.buffer_phase, BufferPhase::Invalid);
        let (kept, event) = step(machine, present(RenderMode::LessLagUltra, 1, 0, 2));
        assert!(matches!(
            event,
            StepEvent::Refused {
                reason: RefuseReason::Invalid,
                ..
            }
        ));
        assert_eq!(kept.known_active, machine.known_active);
        assert!(!matches!(event, StepEvent::SetTextures { .. }));
    }

    fn corrupt_field(word: u64, shift: u32) -> Machine {
        decode((word & !(0b111 << shift)) | (7 << shift))
    }

    #[test]
    fn three_bit_profiles_round_trip_and_the_sentinel_stays_invalid() {
        for committed in [
            Profile::Ultra,
            Profile::LessLag,
            Profile::Vanilla,
            Profile::Doubles,
        ] {
            for requested in [
                Profile::Ultra,
                Profile::LessLag,
                Profile::Vanilla,
                Profile::Doubles,
            ] {
                let machine = Machine {
                    live: true,
                    committed,
                    requested,
                    index_phase: IndexPhase::WaitMarker,
                    buffer_phase: BufferPhase::Drain,
                    index_marker: true,
                    buffer_marker: false,
                    known_active: committed.textures(),
                    count_target: requested.textures(),
                    failed: true,
                    failed_target: Some(requested),
                    waited: 9,
                    generation: 200,
                };
                assert_eq!(decode(encode(machine)), machine);
            }
        }
        let base = encode(Machine::confirmed(Profile::Doubles));
        for shift in [1_u32, 4, 17] {
            let machine = corrupt_field(base, shift);
            assert_eq!(machine.index_phase, IndexPhase::Invalid);
            assert_eq!(machine.buffer_phase, BufferPhase::Invalid);
            let (_, event) = step(machine, present(RenderMode::LessLagDoubles, 1, 0, 3));
            assert!(matches!(
                event,
                StepEvent::Refused {
                    reason: RefuseReason::Invalid,
                    ..
                }
            ));
        }
        let unused = decode((base & !(0b111 << 1)) | (4 << 1));
        assert_eq!(unused.index_phase, IndexPhase::Invalid);
    }

    #[test]
    fn doubles_idle_uses_the_lesslag_flush_shape_on_a_three_ring() {
        let doubles = Machine::confirmed(Profile::Doubles);
        let decision = doubles.decision();
        assert!(!decision.force_wait);
        assert!(!decision.early);
        assert!(!decision.suppress_late);
        assert!(decision.render_opts);
        assert!(decision.modulus.is_none());
        assert!(skip_wait(decision, RenderMode::Vanilla));
        assert!(!render_poll_calls_original(true, false));
        assert!(render_poll_calls_original(true, true));
        assert!(render_poll_calls_original(false, false));
        assert!(!run_early(decision, RenderMode::Vanilla));
        assert!(!skip_late(decision, RenderMode::Vanilla));
        let plan = plan_flush(doubles, RenderMode::Vanilla);
        assert_eq!(
            (plan.calls, plan.lesslag_fields, plan.arm_drain),
            (1, true, false)
        );
        assert_eq!(index_value(5, decision, RenderMode::LessLagDoubles), 6);
        let (toward_two, event) = step(doubles, present(RenderMode::LessLag, 0, 0, 3));
        assert!(matches!(
            event,
            StepEvent::BufferStarted {
                from_count: 3,
                to_count: 2
            }
        ));
        assert_eq!(toward_two.decision().effective, Profile::Doubles);
        assert_eq!(toward_two.decision().modulus, Some(2));
        for input in 0..32 {
            assert!(index_value(input, toward_two.decision(), RenderMode::LessLagDoubles) < 2);
        }
        assert_eq!(
            profile_status(
                RenderMode::LessLagDoubles,
                RenderMode::LessLagDoubles,
                3,
                doubles,
                true
            ),
            "Active"
        );
        assert_eq!(labels(doubles).committed, "lldoubles");
    }

    fn commit_count(
        machine: Machine,
        saved: RenderMode,
        from_active: i32,
        to_count: i32,
    ) -> Machine {
        let (waiting, event) = step(machine, present(saved, 0, 0, from_active));
        assert_eq!(event, StepEvent::Waiting);
        let mut stale = present(saved, 1, 0, from_active);
        stale.fresh = false;
        let (still, event) = step(waiting, stale);
        assert_eq!(event, StepEvent::Waiting);
        let (claimed, event) = step(still, present(saved, 1, 0, from_active));
        assert_eq!(
            event,
            StepEvent::SetTextures {
                count: to_count,
                generation: machine.generation
            }
        );
        let slot = AtomicU64::new(encode(claimed));
        assert!(setter_returned_slot(&slot, claimed.generation));
        let draining = decode(slot.load(Ordering::Acquire));
        assert_eq!(draining.buffer_phase, BufferPhase::Drain);
        let plan = plan_flush(draining, RenderMode::Vanilla);
        assert!(plan.arm_drain);
        assert_eq!(plan.calls, 3);
        let marked = complete_drain(draining, plan.generation).unwrap();
        let (ready, event) = step(marked, present(saved, 0, 0, from_active));
        assert_eq!(
            event,
            StepEvent::BufferReady {
                count: to_count as u8
            }
        );
        let (counted, event) = step(ready, present(saved, 0, 0, to_count));
        assert_eq!(
            event,
            StepEvent::BufferCommitted {
                count: to_count as u8
            }
        );
        assert_eq!(counted.known_active, to_count as u8);
        assert_eq!(counted.buffer_phase, BufferPhase::Idle);
        assert_eq!(counted.index_phase, IndexPhase::Idle);
        counted
    }

    fn commit_index_swap(machine: Machine, saved: RenderMode, active: i32, to: Profile) -> Machine {
        assert_eq!(machine.buffer_phase, BufferPhase::Idle);
        let (opened, event) = step(machine, present(saved, 0, 0, active));
        assert_eq!(
            event,
            StepEvent::Started {
                from: machine.committed,
                to
            }
        );
        assert_eq!(opened.buffer_phase, BufferPhase::Idle);
        let plan = plan_flush(opened, RenderMode::Vanilla);
        assert!(plan.arm_drain);
        assert_eq!(plan.calls, 3);
        let marked = complete_drain(opened, plan.generation).unwrap();
        let (ready, event) = step(marked, present(saved, 0, 0, active));
        assert!(matches!(event, StepEvent::Ready { .. }));
        let (done, event) = step(ready, present(saved, 0, 0, active));
        assert_eq!(
            event,
            StepEvent::Committed {
                from: machine.committed,
                to
            }
        );
        assert_eq!(done.committed, to);
        assert_eq!(done.index_phase, IndexPhase::Idle);
        assert_eq!(done.buffer_phase, BufferPhase::Idle);
        done
    }

    #[test]
    fn vanilla_and_doubles_swap_index_only_and_two_three_rings_drain_both_ways() {
        let vanilla = Machine::confirmed(Profile::Vanilla);
        let (opened, event) = step(vanilla, present(RenderMode::LessLagDoubles, 0, 0, 3));
        assert!(matches!(event, StepEvent::Started { .. }));
        assert_eq!(opened.buffer_phase, BufferPhase::Idle);
        let doubles = commit_index_swap(vanilla, RenderMode::LessLagDoubles, 3, Profile::Doubles);
        assert_eq!(doubles.known_active, 3);
        let back = commit_index_swap(doubles, RenderMode::Vanilla, 3, Profile::Vanilla);
        assert_eq!(back.known_active, 3);

        let less = Machine::confirmed(Profile::LessLag);
        let (growing, event) = step(less, present(RenderMode::LessLagDoubles, 0, 0, 2));
        assert_eq!(
            event,
            StepEvent::BufferStarted {
                from_count: 2,
                to_count: 3
            }
        );
        assert_eq!(growing.decision().modulus, Some(2));
        assert_eq!(growing.index_phase, IndexPhase::Idle);
        let counted = commit_count(growing, RenderMode::LessLagDoubles, 2, 3);
        assert_eq!(counted.committed, Profile::LessLag);
        let doubles = commit_index_swap(counted, RenderMode::LessLagDoubles, 3, Profile::Doubles);
        let (shrinking, event) = step(doubles, present(RenderMode::LessLagUltra, 0, 0, 3));
        assert_eq!(
            event,
            StepEvent::BufferStarted {
                from_count: 3,
                to_count: 2
            }
        );
        assert_eq!(shrinking.decision().effective, Profile::Doubles);
        for input in 0..16 {
            assert!(index_value(input, shrinking.decision(), RenderMode::LessLagDoubles) < 2);
        }
        let counted = commit_count(shrinking, RenderMode::LessLagUltra, 3, 2);
        assert_eq!(counted.committed, Profile::Doubles);
        let ultra = commit_index_swap(counted, RenderMode::LessLagUltra, 2, Profile::Ultra);
        assert_eq!(ultra.known_active, 2);
        for input in 0..16 {
            assert!(index_value(input, ultra.decision(), RenderMode::LessLagUltra) < 2);
        }
    }
}
