//! ARCropolis load callbacks for the Reduced siblings.
//!
//! One callback per EFF path. Lookup only. No second NRO. Each callback loads
//! stock into the buffer ARCropolis allocated and appends that file's siblings
//! there. The only allocation is the small copied emitter set. `*out_len` and
//! the file's Ready state are written only after every sibling in that file
//! succeeds. A false return leaves `*out_len` untouched.
//!
//! Installed ARCropolis 4.1.1, `audittemp/arcropolis411-source/src/modfs/mod.rs`
//! `invoke_virtual` at lines 289–304, allocates `Vec::with_capacity` and passes
//! that uninitialized storage. It keeps the buffer only when the callback
//! returns true and the published length is at most `max_size`. Otherwise it
//! returns `None`. `read_by_hash` (line 242) and `read_into` then call
//! `read_base`. A partial sequence is dropped with that buffer. The load
//! initializes only `got` bytes, so each callback zeroes the tail through
//! `cap` before forming the slice. Stock bytes below `got` stay as loaded.

use super::{logging, record, Code};
use crate::eff_sibling::{
    append_reduced_sibling_in_place, append_reduced_siblings_in_place,
    append_trace_life_sibling_in_place, SiblingRequest, VariantError,
};
use crate::eff_variant::{
    self, CHROM_MAX_BYTES, CHROM_PATH_HASH, CHROM_REQUESTS, EDGE_MAX_BYTES, EDGE_PATH_HASH,
    EDGE_REQUESTS, INK_MAX_BYTES, INK_PATH_HASH, INK_REQUESTS, MAX_BYTES, PATH_HASH, REASON_SIZE,
    REASON_TRANSFORM,
};
use std::sync::atomic::{AtomicUsize, Ordering};

type CallbackFn = unsafe extern "C" fn(u64, *mut u8, usize, *mut usize) -> bool;
type RegisterFn = unsafe extern "C" fn(u64, usize, CallbackFn);
type LoadFn = unsafe extern "C" fn(u64, *mut u8, usize, *mut usize) -> bool;

static LOAD: AtomicUsize = AtomicUsize::new(0);

const REQUEST: SiblingRequest<'static> = SiblingRequest {
    source_handle: "PALUTENA_BACKLIGHT2",
    source_set: "P_PalutenaBackLight2",
    new_handle: "PALUTENA_BACKLIGHT2_REDUCED",
    new_set: "P_PalutenaBackLight2Reduced",
    keep_emitters: &["backlight2_alpha", "backlight2_add"],
};

fn lookup(symbol: &[u8]) -> usize {
    let mut address = 0usize;
    let result = unsafe { skyline::nn::ro::LookupSymbol(&mut address, symbol.as_ptr()) };
    if result == 0 {
        address
    } else {
        0
    }
}

pub(super) fn install() {
    let register = lookup(b"arcrop_register_callback\0");
    let load = lookup(b"arcrop_load_file\0");
    if register == 0 || load == 0 {
        if eff_variant::note_missing_api() {
            record(Code::FxVariantUnavailable, eff_variant::REASON_API, 0);
            logging::install_checkpoint(Code::FxVariantUnavailable, eff_variant::REASON_API, 0);
        }
        if eff_variant::note_edge_missing_api() {
            record(
                Code::FxVariantUnavailable,
                eff_variant::REASON_API,
                EDGE_PATH_HASH,
            );
            logging::install_checkpoint(
                Code::FxVariantUnavailable,
                eff_variant::REASON_API,
                EDGE_PATH_HASH,
            );
        }
        if eff_variant::note_ink_missing_api() {
            record(
                Code::FxVariantUnavailable,
                eff_variant::REASON_API,
                INK_PATH_HASH,
            );
            logging::install_checkpoint(
                Code::FxVariantUnavailable,
                eff_variant::REASON_API,
                INK_PATH_HASH,
            );
        }
        if eff_variant::note_chrom_missing_api() {
            record(
                Code::FxVariantUnavailable,
                eff_variant::REASON_API,
                CHROM_PATH_HASH,
            );
            logging::install_checkpoint(
                Code::FxVariantUnavailable,
                eff_variant::REASON_API,
                CHROM_PATH_HASH,
            );
        }
        return;
    }
    LOAD.store(load, Ordering::Release);
    let register: RegisterFn = unsafe { std::mem::transmute(register) };
    unsafe { register(PATH_HASH, MAX_BYTES, on_eff) };
    record(Code::FxVariantReady, 0, PATH_HASH);
    logging::install_checkpoint(Code::FxVariantReady, 0, PATH_HASH);
    unsafe { register(EDGE_PATH_HASH, EDGE_MAX_BYTES, on_edge) };
    record(Code::FxVariantReady, 0, EDGE_PATH_HASH);
    logging::install_checkpoint(Code::FxVariantReady, 0, EDGE_PATH_HASH);
    unsafe { register(INK_PATH_HASH, INK_MAX_BYTES, on_ink) };
    record(Code::FxVariantReady, 0, INK_PATH_HASH);
    logging::install_checkpoint(Code::FxVariantReady, 0, INK_PATH_HASH);
    unsafe { register(CHROM_PATH_HASH, CHROM_MAX_BYTES, on_chrom) };
    record(Code::FxVariantReady, 0, CHROM_PATH_HASH);
    logging::install_checkpoint(Code::FxVariantReady, 0, CHROM_PATH_HASH);
}

unsafe extern "C" fn on_eff(
    hash: u64,
    data: *mut u8,
    max_size: usize,
    out_len: *mut usize,
) -> bool {
    if hash != PATH_HASH || data.is_null() || out_len.is_null() || max_size == 0 {
        return false;
    }
    let load_addr = LOAD.load(Ordering::Acquire);
    if load_addr == 0 {
        finish_fail(eff_variant::REASON_API);
        return false;
    }
    let cap = max_size.min(MAX_BYTES);
    let mut got = 0usize;
    let load: LoadFn = unsafe { std::mem::transmute(load_addr) };
    let loaded = unsafe { load(hash, data, cap, &mut got) };
    if !loaded || got == 0 || got > cap {
        finish_fail(REASON_SIZE);
        return false;
    }
    let buffer = unsafe { prepared_buffer(data, cap, got) };
    match append_reduced_sibling_in_place(buffer, got, &REQUEST) {
        Ok(new_len) if new_len <= cap => {
            unsafe { *out_len = new_len };
            let sample = eff_variant::note_load_ok();
            if sample.log {
                record(Code::FxVariantReady, PATH_HASH, new_len as u64);
                logging::install_checkpoint(Code::FxVariantReady, PATH_HASH, new_len as u64);
            }
            true
        }
        Ok(_) | Err(VariantError::OutOfMemory | VariantError::Overflow) => {
            finish_fail(REASON_SIZE);
            false
        }
        Err(_) => {
            finish_fail(REASON_TRANSFORM);
            false
        }
    }
}

unsafe extern "C" fn on_edge(
    hash: u64,
    data: *mut u8,
    max_size: usize,
    out_len: *mut usize,
) -> bool {
    on_family(
        hash,
        data,
        max_size,
        out_len,
        &Family {
            path: EDGE_PATH_HASH,
            max_bytes: EDGE_MAX_BYTES,
            requests: EDGE_REQUESTS,
            fail: finish_edge_fail,
            ok: eff_variant::note_edge_load_ok,
        },
    )
}

unsafe extern "C" fn on_chrom(
    hash: u64,
    data: *mut u8,
    max_size: usize,
    out_len: *mut usize,
) -> bool {
    if hash != CHROM_PATH_HASH || data.is_null() || out_len.is_null() || max_size == 0 {
        return false;
    }
    let load_addr = LOAD.load(Ordering::Acquire);
    if load_addr == 0 {
        finish_chrom_fail(eff_variant::REASON_API);
        return false;
    }
    let cap = max_size.min(CHROM_MAX_BYTES);
    let mut got = 0usize;
    let load: LoadFn = unsafe { std::mem::transmute(load_addr) };
    let loaded = unsafe { load(hash, data, cap, &mut got) };
    if !loaded || got == 0 || got > cap {
        finish_chrom_fail(REASON_SIZE);
        return false;
    }
    let buffer = unsafe { prepared_buffer(data, cap, got) };
    match append_trace_life_sibling_in_place(buffer, got, &CHROM_REQUESTS[0]) {
        Ok(new_len) if new_len <= cap => {
            unsafe { *out_len = new_len };
            let sample = eff_variant::note_chrom_load_ok();
            if sample.log {
                record(Code::FxVariantReady, CHROM_PATH_HASH, new_len as u64);
                logging::install_checkpoint(Code::FxVariantReady, CHROM_PATH_HASH, new_len as u64);
            }
            true
        }
        Ok(_) | Err(VariantError::OutOfMemory | VariantError::Overflow) => {
            finish_chrom_fail(REASON_SIZE);
            false
        }
        Err(_) => {
            finish_chrom_fail(REASON_TRANSFORM);
            false
        }
    }
}

unsafe extern "C" fn on_ink(
    hash: u64,
    data: *mut u8,
    max_size: usize,
    out_len: *mut usize,
) -> bool {
    on_family(
        hash,
        data,
        max_size,
        out_len,
        &Family {
            path: INK_PATH_HASH,
            max_bytes: INK_MAX_BYTES,
            requests: INK_REQUESTS,
            fail: finish_ink_fail,
            ok: eff_variant::note_ink_load_ok,
        },
    )
}

struct Family {
    path: u64,
    max_bytes: usize,
    requests: &'static [SiblingRequest<'static>],
    fail: fn(u64),
    ok: fn() -> eff_variant::LoadSample,
}

unsafe fn on_family(
    hash: u64,
    data: *mut u8,
    max_size: usize,
    out_len: *mut usize,
    family: &Family,
) -> bool {
    let Family {
        path,
        max_bytes,
        requests,
        fail,
        ok,
    } = *family;
    if hash != path || data.is_null() || out_len.is_null() || max_size == 0 {
        return false;
    }
    let load_addr = LOAD.load(Ordering::Acquire);
    if load_addr == 0 {
        fail(eff_variant::REASON_API);
        return false;
    }
    let cap = max_size.min(max_bytes);
    let mut got = 0usize;
    let load: LoadFn = unsafe { std::mem::transmute(load_addr) };
    let loaded = unsafe { load(hash, data, cap, &mut got) };
    if !loaded || got == 0 || got > cap {
        fail(REASON_SIZE);
        return false;
    }
    let buffer = unsafe { prepared_buffer(data, cap, got) };
    match append_reduced_siblings_in_place(buffer, got, requests) {
        Ok(new_len) if new_len <= cap => {
            unsafe { *out_len = new_len };
            let sample = ok();
            if sample.log {
                record(Code::FxVariantReady, path, new_len as u64);
                logging::install_checkpoint(Code::FxVariantReady, path, new_len as u64);
            }
            true
        }
        Ok(_) | Err(VariantError::OutOfMemory | VariantError::Overflow) => {
            fail(REASON_SIZE);
            false
        }
        Err(_) => {
            fail(REASON_TRANSFORM);
            false
        }
    }
}

/// `cap` is `min(max_size, configured maximum)` and `got <= cap`.
/// ARCropolis 4.1.1 leaves `[got, cap)` uninitialized. Zero that tail only.
unsafe fn prepared_buffer<'a>(data: *mut u8, cap: usize, got: usize) -> &'a mut [u8] {
    let tail = cap - got;
    if tail != 0 {
        std::ptr::write_bytes(data.add(got), 0, tail);
    }
    std::slice::from_raw_parts_mut(data, cap)
}

fn finish_chrom_fail(reason: u64) {
    let sample = eff_variant::note_chrom_load_failed(reason);
    if sample.log && sample.count > 0 {
        record(Code::FxVariantUnavailable, reason, CHROM_PATH_HASH);
        logging::install_checkpoint(Code::FxVariantUnavailable, reason, CHROM_PATH_HASH);
    }
}

fn finish_ink_fail(reason: u64) {
    let sample = eff_variant::note_ink_load_failed(reason);
    if sample.log && sample.count > 0 {
        record(Code::FxVariantUnavailable, reason, INK_PATH_HASH);
        logging::install_checkpoint(Code::FxVariantUnavailable, reason, INK_PATH_HASH);
    }
}

fn finish_edge_fail(reason: u64) {
    let sample = eff_variant::note_edge_load_failed(reason);
    if sample.log && sample.count > 0 {
        record(Code::FxVariantUnavailable, reason, EDGE_PATH_HASH);
        logging::install_checkpoint(Code::FxVariantUnavailable, reason, EDGE_PATH_HASH);
    }
}

fn finish_fail(reason: u64) {
    let sample = eff_variant::note_load_failed(reason);
    if sample.log && sample.count > 0 {
        record(Code::FxVariantUnavailable, reason, PATH_HASH);
        logging::install_checkpoint(Code::FxVariantUnavailable, reason, PATH_HASH);
    }
}
