//! Switch/Eden side of the stand-in: register `hoc:clk` through sm, then a
//! single-threaded server loop modelled on hoc-clk's `ipc/ipc_server.c`
//! (wait on the port and every session, accept, receive, reply). Raw SVCs on
//! this thread's TLS message buffer; every state change is logged to
//! `sd:/ultimate/hoc_standin/standin.log` as integration evidence.

use crate::protocol::{self, Event, Message, Server, BUFFER, KIND_CLOSE};
use std::io::Write;

type Handle = u32;

const STACK_BYTES: usize = 64 * 1024;
const LOG_DIR: &str = "sd:/ultimate/hoc_standin";
const LOG_PATH: &str = "sd:/ultimate/hoc_standin/standin.log";
/// `KERNELRESULT(TimedOut)`: a reply-only `svcReplyAndReceive` returns it.
const TIMED_OUT: u32 = 0xEA01;
/// The port plus sessions; Horizon allows 64 wait objects.
const MAX_HANDLES: usize = 64;
const SM_REGISTER_CLIENT: u32 = 0;
const SM_REGISTER_SERVICE: u32 = 2;

fn tick() -> u64 {
    unsafe { skyline::nn::os::GetSystemTick() as u64 }
}

fn log(line: &str) {
    skyline::println!("[hoc-standin] {line}");
    let _ = std::fs::create_dir_all(LOG_DIR);
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(LOG_PATH) {
        let _ = writeln!(file, "{} {line}", tick());
    }
}

unsafe fn tls() -> *mut u8 {
    let tls: usize;
    core::arch::asm!("mrs {}, tpidrro_el0", out(reg) tls, options(nomem, nostack));
    tls as *mut u8
}

fn load(msg: &mut Message) {
    unsafe { core::ptr::copy_nonoverlapping(tls(), msg.as_mut_ptr(), BUFFER) };
}

fn store(msg: &Message) {
    unsafe { core::ptr::copy_nonoverlapping(msg.as_ptr(), tls(), BUFFER) };
}

unsafe fn connect_named_port(name: &[u8; 4]) -> (u32, Handle) {
    let result: u64;
    let handle: u64;
    core::arch::asm!("svc 0x1f", lateout("x0") result, inout("x1") name.as_ptr() as u64 => handle,
        clobber_abi("C"), options(nostack));
    (result as u32, handle as Handle)
}

unsafe fn send_sync_request(handle: Handle) -> u32 {
    let result: u64;
    core::arch::asm!("svc 0x21", inout("x0") u64::from(handle) => result, clobber_abi("C"), options(nostack));
    result as u32
}

unsafe fn wait_synchronization(handles: &[Handle]) -> (u32, usize) {
    let result: u64;
    let index: u64;
    core::arch::asm!("svc 0x18", lateout("x0") result, inout("x1") handles.as_ptr() as u64 => index,
        in("x2") handles.len() as u64, in("x3") u64::MAX, clobber_abi("C"), options(nostack));
    (result as u32, index as u32 as usize)
}

unsafe fn accept_session(port: Handle) -> (u32, Handle) {
    let result: u64;
    let session: u64;
    core::arch::asm!("svc 0x41", lateout("x0") result, inout("x1") u64::from(port) => session,
        clobber_abi("C"), options(nostack));
    (result as u32, session as Handle)
}

/// Receive on `handles` (count may be 0) after replying to `reply_target` (0 none).
unsafe fn reply_and_receive(handles: &[Handle], reply_target: Handle, timeout: u64) -> u32 {
    let result: u64;
    core::arch::asm!("svc 0x43", lateout("x0") result, inout("x1") handles.as_ptr() as u64 => _,
        in("x2") handles.len() as u64, in("x3") u64::from(reply_target), in("x4") timeout,
        clobber_abi("C"), options(nostack));
    result as u32
}

unsafe fn close_handle(handle: Handle) {
    core::arch::asm!("svc 0x16", inout("x0") u64::from(handle) => _, clobber_abi("C"), options(nostack));
}

fn sm_call(sm: Handle, command: u32, send_pid: bool, payload: &[u8]) -> Result<Option<Handle>, u32> {
    let mut msg = [0; BUFFER];
    protocol::request(&mut msg, command, send_pid, payload);
    store(&msg);
    let rc = unsafe { send_sync_request(sm) };
    if rc != 0 {
        return Err(rc);
    }
    load(&mut msg);
    match protocol::decode_reply(&msg) {
        Ok((0, moved)) => Ok(moved),
        Ok((result, _)) => Err(result),
        Err(_) => Err(0xFFFF_FFFF),
    }
}

/// `RegisterClient`, then `RegisterService("hoc:clk", is_light=false, 8 sessions)`.
/// The sm session stays open for the life of the process.
fn register() -> Result<(Handle, Handle), u32> {
    let (rc, sm) = unsafe { connect_named_port(b"sm:\0") };
    if rc != 0 {
        return Err(rc);
    }
    sm_call(sm, SM_REGISTER_CLIENT, true, &0u64.to_le_bytes())?;
    let mut args = [0u8; 16];
    args[..8].copy_from_slice(b"hoc:clk\0");
    args[12..16].copy_from_slice(&8u32.to_le_bytes());
    let port = sm_call(sm, SM_REGISTER_SERVICE, false, &args)?.ok_or(0xFFFF_FFFEu32)?;
    Ok((sm, port))
}

/// Integration evidence: the raw words sm returns for the TIPC
/// `AtmosphereHasService(name)` query Deluxe's client makes (command 65100,
/// message type 65116, the name as the only data).
fn probe_has_service(sm: Handle, name: &[u8; 8]) {
    let mut msg = [0u8; BUFFER];
    msg[0..4].copy_from_slice(&(65100u32 + 16).to_le_bytes());
    msg[4..8].copy_from_slice(&2u32.to_le_bytes());
    msg[8..16].copy_from_slice(name);
    store(&msg);
    let rc = unsafe { send_sync_request(sm) };
    load(&mut msg);
    let words: Vec<String> = (0..8)
        .map(|i| format!("{:08x}", u32::from_le_bytes([msg[i * 4], msg[i * 4 + 1], msg[i * 4 + 2], msg[i * 4 + 3]])))
        .collect();
    let shown = String::from_utf8_lossy(name).trim_end_matches('\0').to_owned();
    log(&format!("probe HasService({shown}) rc={rc:#x} words={}", words.join(" ")));
}

/// One session became signalled: receive its message, answer it. Returns false
/// when the session is gone (closed by the client or a failed receive/reply).
fn serve(session: Handle, server: &mut Server, seen: &mut [bool; 104]) -> bool {
    let rc = unsafe { reply_and_receive(&[session], 0, u64::MAX) };
    if rc != 0 {
        return false;
    }
    let mut received = [0; BUFFER];
    load(&mut received);
    let mut out = [0; BUFFER];
    let keep = match protocol::parse(&received) {
        // CMIF Close: acknowledge, then drop the session.
        Ok(request) if request.kind == KIND_CLOSE => {
            protocol::reply(&mut out, 0, &[]);
            false
        }
        Ok(request) => {
            let command = request.command as usize;
            if command < seen.len() && !seen[command] {
                seen[command] = true;
                log(&format!("first cmd {command} pid={:?}", request.pid));
            }
            let (result, payload, event) = server.handle(&request, tick());
            if let Some(event) = event {
                let name = match event {
                    Event::Armed => "armed",
                    Event::ApplyOn => "apply on",
                    Event::ApplyOff => "apply off",
                    Event::Disarmed => "disarmed",
                };
                log(&format!("{name} (pid={:?})", request.pid));
            }
            protocol::reply(&mut out, result, &payload);
            true
        }
        Err(error) => {
            log(&format!("malformed request {error:?}"));
            protocol::reply(&mut out, protocol::RESULT_GENERIC, &[]);
            true
        }
    };
    store(&out);
    let rc = unsafe { reply_and_receive(&[], session, 0) };
    keep && (rc == 0 || rc == TIMED_OUT)
}

fn run() {
    let (sm, port) = match register() {
        Ok(handles) => handles,
        Err(code) => {
            log(&format!("register hoc:clk failed {code:#x}"));
            return;
        }
    };
    log("registered hoc:clk (protocol v1 stand-in, no clock control)");
    probe_has_service(sm, b"hoc:clk\0");
    probe_has_service(sm, b"nothere\0");
    probe_has_service(sm, b"fsp-srv\0");
    // The same query over CMIF (Eden also lists 65100 in its CMIF table).
    let mut msg = [0u8; BUFFER];
    protocol::request(&mut msg, 65100, false, b"hoc:clk\0");
    store(&msg);
    let rc = unsafe { send_sync_request(sm) };
    load(&mut msg);
    let words: Vec<String> = (0..12)
        .map(|i| format!("{:08x}", u32::from_le_bytes([msg[i * 4], msg[i * 4 + 1], msg[i * 4 + 2], msg[i * 4 + 3]])))
        .collect();
    log(&format!("probe CMIF HasService(hoc:clk) rc={rc:#x} words={}", words.join(" ")));
    let mut handles: Vec<Handle> = vec![port];
    let mut server = Server::default();
    let mut seen = [false; 104];
    loop {
        let (rc, index) = unsafe { wait_synchronization(&handles) };
        if rc != 0 || index >= handles.len() {
            log(&format!("wait failed {rc:#x}"));
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }
        if index == 0 {
            let (rc, session) = unsafe { accept_session(port) };
            if rc == 0 && handles.len() < MAX_HANDLES {
                handles.push(session);
                log(&format!("session accepted ({} open)", handles.len() - 1));
            } else if rc == 0 {
                unsafe { close_handle(session) };
            }
        } else if !serve(handles[index], &mut server, &mut seen) {
            unsafe { close_handle(handles[index]) };
            handles.remove(index);
            log(&format!("session closed ({} open)", handles.len() - 1));
        }
    }
}

pub fn start() {
    if let Err(error) = std::thread::Builder::new()
        .name("hoc-standin".into())
        .stack_size(STACK_BYTES)
        .spawn(run)
    {
        skyline::println!("[hoc-standin] thread failed: {error}");
    }
}
