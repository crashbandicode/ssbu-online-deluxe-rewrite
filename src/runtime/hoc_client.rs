//! Game-aware Horizon-OC client (0.9.0). One small thread of its own owns
//! the only `hoc:clk` session this plugin opens; hooks, the renderer and the
//! diagnostics thread never wait on it. Every message is framed and every
//! decision made by [`crate::hoc_game_aware`]; this file only moves bytes
//! through the thread's TLS IPC buffer with raw SVCs, as `memory` does
//! (nnsdk 0.6.2 and skyline 0.6.0 have no sm bindings).
//!
//! The sm requests ride Smash's own `sm:` session, read from its SDK
//! ([`wire::sdk_sm`]): a console gives a game a single session of its own
//! and the SDK already holds it. The kernel serializes requests from several
//! threads on one session. The session is never registered, detached or
//! closed here.

use super::{record, Code};
use crate::hoc_game_aware::{
    self as wire, sdk_sm, Capability, Heartbeat, IpcBuffer, WireError, IPC_BUFFER_SIZE,
};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Duration;

const STACK_BYTES: usize = 64 * 1024;
const PERIOD: Duration = Duration::from_millis(500);

static CAPABILITY: AtomicU8 = AtomicU8::new(Capability::Unknown as u8);
static APPLYING: AtomicBool = AtomicBool::new(false);

/// The last handshake result, for the window and ZR+L3.
pub(super) fn capability() -> Capability {
    Capability::from_byte(CAPABILITY.load(Ordering::Acquire))
}

/// The fork's last answer: Smash's saved profile is being applied.
pub(super) fn applying() -> bool {
    APPLYING.load(Ordering::Acquire)
}

type Handle = u32;

#[derive(Clone, Copy, Debug)]
enum Failure {
    /// The SVC itself failed (for example a closed session).
    Kernel(u32),
    Wire(WireError),
    /// The service answered with this result.
    Result(u32),
    /// Smash's sm session could not be read.
    Route(sdk_sm::Route),
}

impl Failure {
    const fn detail(self) -> u64 {
        match self {
            Self::Kernel(code) | Self::Result(code) => code as u64,
            Self::Wire(WireError::Truncated) => 1,
            Self::Wire(WireError::BadMagic) => 2,
            Self::Route(route) => route.detail(),
        }
    }
}

unsafe fn ipc_buffer() -> *mut u8 {
    let tls: usize;
    core::arch::asm!("mrs {}, tpidrro_el0", out(reg) tls, options(nomem, nostack));
    tls as *mut u8
}

fn lookup(symbol: &[u8]) -> usize {
    let mut address = 0usize;
    let result = unsafe { skyline::nn::ro::LookupSymbol(&mut address, symbol.as_ptr()) };
    if result == 0 {
        address
    } else {
        0
    }
}

/// `len` bytes of SDK code at `address` hash to `fnv`.
fn pinned_code(address: usize, len: usize, fnv: u64) -> bool {
    address & 3 == 0
        && unsafe { super::memory::executable(address, len) }
        && sdk_sm::fnv1a64(unsafe { core::slice::from_raw_parts(address as *const u8, len) }) == fnv
}

fn read_u64(address: usize) -> Option<u64> {
    (address & 7 == 0 && unsafe { super::memory::accessible(address, 8, false) })
        .then(|| unsafe { core::ptr::read_volatile(address as *const u64) })
}

fn read_u32(address: usize) -> Option<u32> {
    (address & 3 == 0 && unsafe { super::memory::accessible(address, 4, false) })
        .then(|| unsafe { core::ptr::read_volatile(address as *const u32) })
}

/// Smash's registered `sm:` session, as `nn::sm::GetServiceHandle` reaches
/// it: the slot behind the pointer its `adrp`/`ldr` loads, the session object
/// in that slot (checked by its vtable), the handle `nn::sm::Initialize`
/// stored in the object. Only after both functions match the pinned SDK.
fn sdk_sm_session() -> Result<Handle, Failure> {
    use sdk_sm::Route;
    let get = lookup(sdk_sm::GET_SERVICE_HANDLE);
    let init = lookup(sdk_sm::INITIALIZE);
    if get == 0 || init == 0 {
        return Err(Failure::Route(Route::Symbol));
    }
    if !pinned_code(get, sdk_sm::GET_SERVICE_HANDLE_LEN, sdk_sm::GET_SERVICE_HANDLE_FNV)
        || !pinned_code(init, sdk_sm::INITIALIZE_LEN, sdk_sm::INITIALIZE_FNV)
    {
        return Err(Failure::Route(Route::Code));
    }
    let object = read_u64(get.wrapping_add(sdk_sm::SLOT_POINTER_FROM_GET))
        .and_then(|slot| read_u64(slot as usize))
        .filter(|object| *object != 0)
        .ok_or(Failure::Route(Route::Object))? as usize;
    let vtable = init.wrapping_add(sdk_sm::VTABLE_FROM_INITIALIZE) as u64;
    if read_u64(object) != Some(vtable) {
        return Err(Failure::Route(Route::Vtable));
    }
    read_u32(object.wrapping_add(sdk_sm::HANDLE_OFFSET))
        .filter(|handle| sdk_sm::plausible_handle(*handle))
        .ok_or(Failure::Route(Route::Handle))
}

/// `svcSendSyncRequest` on the message already in the TLS buffer.
unsafe fn send_sync_request(handle: Handle) -> u32 {
    let result: u64;
    core::arch::asm!(
        "svc 0x21",
        inout("x0") u64::from(handle) => result,
        clobber_abi("C"),
        options(nostack),
    );
    result as u32
}

unsafe fn close_handle(handle: Handle) {
    core::arch::asm!(
        "svc 0x16",
        inout("x0") u64::from(handle) => _,
        clobber_abi("C"),
        options(nostack),
    );
}

/// Copies `request` into the TLS IPC buffer, sends it and copies the reply out.
fn transact(handle: Handle, request: &IpcBuffer) -> Result<IpcBuffer, Failure> {
    let mut reply = [0; IPC_BUFFER_SIZE];
    unsafe {
        let tls = ipc_buffer();
        core::ptr::copy_nonoverlapping(request.as_ptr(), tls, IPC_BUFFER_SIZE);
        let result = send_sync_request(handle);
        if result != 0 {
            return Err(Failure::Kernel(result));
        }
        core::ptr::copy_nonoverlapping(tls, reply.as_mut_ptr(), IPC_BUFFER_SIZE);
    }
    Ok(reply)
}

/// A CMIF call whose service result must be success.
fn cmif(
    handle: Handle,
    command: u32,
    send_pid: bool,
    payload: &[u8],
) -> Result<wire::Reply, Failure> {
    let mut request = [0; IPC_BUFFER_SIZE];
    wire::encode_cmif(&mut request, command, send_pid, payload);
    let reply = wire::decode_cmif(&transact(handle, &request)?).map_err(Failure::Wire)?;
    if reply.result != 0 {
        return Err(Failure::Result(reply.result));
    }
    Ok(reply)
}

struct Client {
    program_id: u64,
    /// Smash's `sm:` session, borrowed (see the module note).
    sm: Option<Handle>,
    hoc: Option<Handle>,
    capability: Capability,
    retry_at: u64,
    /// The fork's arm for this process as last confirmed. `None` after every
    /// handshake: the arm is bound to the process, not the session, so a
    /// reconnect does not reveal it and the first pass asserts Arm or Disarm
    /// (both idempotent).
    armed: Option<bool>,
    heartbeat: Heartbeat,
    seq: u32,
    /// Last recorded `HocGate` state bits.
    gate: Option<u64>,
}

impl Client {
    fn new(program_id: u64) -> Self {
        Self {
            program_id,
            sm: None,
            hoc: None,
            capability: Capability::Unknown,
            retry_at: 0,
            armed: None,
            heartbeat: Heartbeat::default(),
            seq: 0,
            gate: None,
        }
    }

    fn set_capability(&mut self, capability: Capability, detail: u64) {
        if capability != self.capability {
            record(Code::HocCapability, capability as u64, detail);
        }
        self.capability = capability;
        CAPABILITY.store(capability as u8, Ordering::Release);
    }

    fn note_gate(&mut self, applying: bool, online: bool, detail: u64) {
        APPLYING.store(applying, Ordering::Release);
        let bits = u64::from(applying)
            | u64::from(self.armed == Some(true)) << 1
            | u64::from(super::hoc_oc() == wire::OcMode::Online) << 2
            | u64::from(online) << 3;
        if self.gate != Some(bits) {
            self.gate = Some(bits);
            record(Code::HocGate, bits, detail);
        }
    }

    fn sm(&mut self) -> Result<Handle, Failure> {
        if let Some(sm) = self.sm {
            return Ok(sm);
        }
        let sm = sdk_sm_session()?;
        self.sm = Some(sm);
        Ok(sm)
    }

    /// `HasService` first (never deferred; TIPC on a console, CMIF with
    /// `platform=emulator`, see `HasServiceFraming`), then the session, the
    /// API version and cmd 100.
    fn open(&mut self) -> Result<(Capability, u64), Failure> {
        let sm = self.sm()?;
        let emulator = super::settings().platform == crate::config::Platform::Emulator;
        let framing = wire::HasServiceFraming::for_platform(emulator);
        let mut request = [0; IPC_BUFFER_SIZE];
        wire::encode_has_service(&mut request, framing);
        let has = match transact(sm, &request) {
            Ok(reply) => match wire::decode_has_service(&reply, framing) {
                Ok(Ok(present)) => present,
                Ok(Err(result)) => return Err(Failure::Result(result)),
                // An unreadable reply leaves the session usable; keep it.
                Err(error) => return Err(Failure::Wire(error)),
            },
            Err(failure) => {
                // The SVC failed: forget the handle, never close Smash's
                // session, and read it again at the next handshake.
                self.sm = None;
                return Err(failure);
            }
        };
        if !has {
            return Ok((Capability::NoService, 0));
        }
        let handle = cmif(sm, wire::SM_GET_SERVICE_HANDLE, false, &wire::SERVICE)?
            .moved
            .ok_or(Failure::Wire(WireError::Truncated))?;
        self.hoc = Some(handle);
        let api = cmif(handle, wire::CMD_GET_API_VERSION, false, &[])?;
        let version = api
            .payload()
            .get(..4)
            .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .ok_or(Failure::Wire(WireError::Truncated))?;
        let mut request = [0; IPC_BUFFER_SIZE];
        wire::encode_cmif(&mut request, wire::CMD_INFO, false, &[]);
        let info = wire::decode_cmif(&transact(handle, &request)?).map_err(Failure::Wire)?;
        let capability = wire::classify(version, info.result, info.payload());
        let detail = match (capability, wire::decode_info(info.payload())) {
            (Capability::Present, Some(found)) => {
                u64::from(found.caps) << 16 | u64::from(found.version)
            }
            _ => u64::from(info.result),
        };
        Ok((capability, detail))
    }

    fn close_hoc(&mut self) {
        if let Some(handle) = self.hoc.take() {
            unsafe { close_handle(handle) };
        }
        self.armed = None;
        self.heartbeat.reset();
    }

    /// A call failed: drop the session and re-handshake after the retry
    /// delay. A fork that is still running keeps this process armed, so its
    /// lease runs out and the saved profile stays held back until the next
    /// handshake renews it; a restarted hoc-clk comes back unarmed and applies
    /// the profile as stock Horizon-OC does. Either way nothing is persisted.
    fn fail(&mut self, now: u64, failure: Failure, online: bool) {
        self.close_hoc();
        let next = if self.capability == Capability::Present {
            Capability::Unavailable
        } else {
            self.capability
        };
        self.set_capability(next, failure.detail());
        self.note_gate(false, online, 1 << 32 | failure.detail());
        self.retry_at = now.wrapping_add(wire::RETRY_TICKS);
    }

    fn handshake(&mut self, now: u64, online: bool) {
        match self.open() {
            Ok((capability, detail)) => {
                self.set_capability(capability, detail);
                if capability != Capability::Present {
                    self.close_hoc();
                    self.retry_at = now.wrapping_add(wire::retry_ticks(capability));
                }
            }
            Err(failure) => {
                self.close_hoc();
                let next = match self.capability {
                    Capability::Present | Capability::Unavailable => Capability::Unavailable,
                    _ => Capability::NoService,
                };
                self.set_capability(next, failure.detail());
                self.note_gate(false, online, 1 << 32 | failure.detail());
                self.retry_at = now.wrapping_add(wire::RETRY_TICKS);
            }
        }
    }

    fn step(&mut self, now: u64) {
        let mode = super::hoc_oc();
        let online = super::online_session::live_at(now);
        if self.hoc.is_none() {
            if now < self.retry_at {
                return;
            }
            self.handshake(now, online);
        }
        let Some(handle) = self.hoc else {
            return;
        };
        let want_armed = wire::want_armed(mode, self.capability);
        if self.armed != Some(want_armed) {
            let result = if want_armed {
                cmif(handle, wire::CMD_ARM, true, &wire::arm_args(self.program_id))
            } else {
                cmif(handle, wire::CMD_DISARM, true, &[])
            };
            if let Err(failure) = result {
                return self.fail(now, failure, online);
            }
            self.armed = Some(want_armed);
            self.heartbeat.reset();
            if !want_armed {
                self.note_gate(false, online, 0);
            }
        }
        if !want_armed {
            return;
        }
        let apply = wire::want_apply(mode, self.capability, online);
        if !self.heartbeat.due(apply, now) {
            return;
        }
        self.seq = self.seq.wrapping_add(1);
        let args = wire::set_apply_args(apply, wire::LEASE_MS, self.seq);
        match cmif(handle, wire::CMD_SET_APPLY, true, &args) {
            Ok(reply) => {
                self.heartbeat.sent(apply, now);
                let state = wire::decode_state(reply.payload()).unwrap_or_default();
                self.note_gate(state.applying, online, u64::from(state.generation));
            }
            Err(failure) => self.fail(now, failure, online),
        }
    }
}

fn run(program_id: u64) {
    let mut client = Client::new(program_id);
    loop {
        client.step(super::online_session::tick());
        std::thread::sleep(PERIOD);
    }
}

/// Starts the client thread. No hook is installed; `DISABLE_HOC` skips it.
pub(super) fn install() {
    let Some(program_id) = super::memory::program_id() else {
        skyline::println!("[Deluxe] Program id unavailable; Horizon-OC client not started");
        return;
    };
    if let Err(error) = std::thread::Builder::new()
        .name("deluxe-hoc".into())
        .stack_size(STACK_BYTES)
        .spawn(move || run(program_id))
    {
        skyline::println!("[Deluxe] Horizon-OC client thread failed: {error}");
    }
}
