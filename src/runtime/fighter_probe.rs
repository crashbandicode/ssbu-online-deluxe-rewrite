//! One `LoadModuleInternal` / `UnloadModule` chain for `common`.
//!
//! The sixteen-word dump stays behind `PROBE_FIGHTER_MODULE`. The same load
//! calls the logical Main installer. The export is recorded and never hooked.

use super::hang_trace::{self, Hook};
use super::{fighter_main, logging, memory, record, Code, DATA_DIR};
use crate::fighter_probe::{self, LoadView, LOAD_MODULE_INTERNAL, SITE_OFFSET, WORD_COUNT};
use crate::hook_chain::{plan_far_chain, EntryPlan};
use skyline::hooks::A64HookFunction;
use skyline::nn::ro::Module;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Mutex;

const FLAG: &str = "PROBE_FIGHTER_MODULE";
const STATUS_FILE: &str = "fighter-module.txt";

static DUMP: AtomicBool = AtomicBool::new(false);
static LOAD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static UNLOAD_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static LAST_WRITTEN: AtomicU32 = AtomicU32::new(0);
static STATE: Mutex<fighter_probe::Observation> = Mutex::new(fighter_probe::Observation::idle());

type LoadFn = unsafe extern "C" fn(*mut Module, *const u8, *mut u8, usize, i32, bool) -> u32;
type UnloadFn = unsafe extern "C" fn(*mut Module) -> u32;

pub(super) fn install() {
    fighter_main::configure();
    let dump = Path::new(DATA_DIR).join(FLAG).exists();
    DUMP.store(dump, Ordering::Release);
    if dump {
        logging::startup_phase("fighter_module_probe_begin");
    } else {
        logging::startup_phase("fighter_module_probe_off");
    }
    logging::startup_phase("fighter_module_hooks_begin");
    let load = lookup(LOAD_MODULE_INTERNAL);
    let unload = skyline::nn::ro::UnloadModule as *const () as usize;
    if !hookable(load) || !hookable(unload) {
        record(Code::HookRejected, 1, load as u64);
        record(Code::HookRejected, 2, unload as u64);
        logging::startup_phase("fighter_module_probe_unresolved");
        return;
    }
    let unload_linked = hook(unload, on_unload as *const (), &UNLOAD_ORIGINAL, 2);
    if !unload_linked || UNLOAD_ORIGINAL.load(Ordering::Acquire) == 0 {
        logging::startup_phase("fighter_module_probe_unload_failed");
        return;
    }
    let load_linked = hook(load, on_load as *const (), &LOAD_ORIGINAL, 1);
    if !load_linked || LOAD_ORIGINAL.load(Ordering::Acquire) == 0 {
        logging::startup_phase("fighter_module_probe_load_failed");
        return;
    }
    logging::startup_phase("fighter_module_probe_ready");
}

pub(super) fn poll() {
    fighter_main::poll();
    if !DUMP.load(Ordering::Acquire) {
        return;
    }
    let observation = {
        let guard = STATE.lock().unwrap_or_else(|err| err.into_inner());
        *guard
    };
    if observation.generation == 0 || observation.generation == LAST_WRITTEN.load(Ordering::Acquire)
    {
        return;
    }
    let text = fighter_probe::status_text(
        crate::build_info::BUILD_ID,
        &observation,
        fighter_main::view(),
    );
    if let Ok(mut file) = File::create(Path::new(DATA_DIR).join(STATUS_FILE)) {
        if file.write_all(text.as_bytes()).is_ok() && file.sync_all().is_ok() {
            LAST_WRITTEN.store(observation.generation, Ordering::Release);
        }
    }
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

fn hookable(address: usize) -> bool {
    address != 0 && address & 3 == 0 && unsafe { memory::executable(address, 20) }
}

fn hook(target: usize, replacement: *const (), original: &AtomicUsize, site: u64) -> bool {
    if !hookable(target) || !unsafe { memory::executable(replacement as usize, 4) } {
        record(Code::HookRejected, site, 0);
        return false;
    }
    let before = entry_words(target);
    let plan = plan_far_chain(before, target as u64, replacement as u64);
    let handler_region = match plan {
        EntryPlan::Chain(handler) => unsafe { memory::region(handler as usize) },
        _ => None,
    };
    // Ring events may not flush before a native exception. Persist the full stub
    // and prior-handler mapping before entering Skyline's hook engine.
    logging::startup_phase(&format!(
        "fighter_module_hook_before site={site} target={target:#x} replacement={:#x} words={before:08x?} plan={plan:?} handler_region={handler_region:x?}",
        replacement as usize,
    ));
    record(Code::HookEntryWord, site, before[0] as u64);
    if plan == EntryPlan::Reject {
        record(Code::HookRejected, site, 2);
        return false;
    }
    record(Code::HookInstalling, site, target as u64);
    match plan {
        EntryPlan::Chain(handler) => {
            if !unsafe { memory::executable(handler as usize, 4) } {
                record(Code::HookRejected, site, handler);
                return false;
            }
            // The existing LDR/BR stub already supplies the previous callable.
            // Asking Skyline for a trampoline would decode its address payload
            // as instructions (the captured 0x98bce454 LDRSW crash). A null result
            // is the supported no-relocation path. Publish before patching so
            // the replacement always has a valid previous handler to call.
            original.store(handler as usize, Ordering::Release);
            unsafe {
                A64HookFunction(
                    target as *const skyline::libc::c_void,
                    replacement as *const skyline::libc::c_void,
                    std::ptr::null_mut(),
                );
            }
        }
        EntryPlan::Relocate => {
            let mut trampoline: *mut skyline::libc::c_void = std::ptr::null_mut();
            unsafe {
                A64HookFunction(
                    target as *const skyline::libc::c_void,
                    replacement as *const skyline::libc::c_void,
                    &mut trampoline,
                );
            }
            if trampoline.is_null() {
                record(Code::HookRejected, site, 1);
                return false;
            }
            original.store(trampoline as usize, Ordering::Release);
        }
        EntryPlan::Reject => unreachable!(),
    }
    let after = entry_words(target);
    let installed = if after[0] == 0x5800_0051 && after[1] == 0xd61f_0220 {
        u64::from(after[2]) | (u64::from(after[3]) << 32) == replacement as u64
    } else if after[0] & 0xfc00_0000 == 0x1400_0000 {
        let displacement = ((after[0] << 6) as i32) >> 4;
        target.checked_add_signed(displacement as isize) == Some(replacement as usize)
    } else {
        false
    };
    logging::startup_phase(&format!(
        "fighter_module_hook_after site={site} installed={installed} words={after:08x?} previous={:#x}",
        original.load(Ordering::Acquire),
    ));
    record(
        if installed {
            Code::HookInstalled
        } else {
            Code::HookRejected
        },
        site,
        after[0] as u64,
    );
    installed
}

fn entry_words(target: usize) -> [u32; 4] {
    // Callers have checked the complete readable executable mapping.
    std::array::from_fn(|index| unsafe {
        std::ptr::read_unaligned((target as *const u32).add(index))
    })
}

#[allow(clippy::too_many_arguments)]
unsafe extern "C" fn on_load(
    module: *mut Module,
    image: *const u8,
    buffer: *mut u8,
    buffer_size: usize,
    flag: i32,
    check: bool,
) -> u32 {
    let trampoline = LOAD_ORIGINAL.load(Ordering::Acquire);
    if trampoline == 0 {
        return u32::MAX;
    }
    let original: LoadFn = std::mem::transmute(trampoline);
    hang_trace::enter(Hook::FighterLoad);
    let result = original(module, image, buffer, buffer_size, flag, check);
    if result == 0 {
        observe_load(module);
    }
    hang_trace::exit(Hook::FighterLoad);
    result
}

unsafe extern "C" fn on_unload(module: *mut Module) -> u32 {
    let address = module as u64;
    // Every unload, tracked or not, marks the frame for the render-sequence
    // teardown guard before the module goes away.
    super::render_sequence::note_module_unload();
    let tracked = address != 0 && address == snapshot().module;
    let trampoline = UNLOAD_ORIGINAL.load(Ordering::Acquire);
    if trampoline == 0 {
        return u32::MAX;
    }
    let original: UnloadFn = std::mem::transmute(trampoline);
    hang_trace::enter(Hook::FighterUnload);
    let result = original(module);
    if tracked {
        let current = snapshot();
        let next = fighter_probe::note_unload(current, address, result);
        if next.generation != current.generation {
            publish(next, DUMP.load(Ordering::Acquire));
            fighter_main::on_common_unloaded(address);
        }
    }
    hang_trace::exit(Hook::FighterUnload);
    result
}

fn observe_load(module: *mut Module) {
    let Some(name) = read_name(module) else {
        return;
    };
    if !fighter_probe::name_is_common(&name) {
        return;
    }
    let base = read_base(module);
    let (symbol, symbol_words) = lookup_call_line();
    let site_words = base.and_then(read_site);
    let current = snapshot();
    let next = fighter_probe::note_load(
        current,
        LoadView {
            module: module as u64,
            result: 0,
            name: &name,
            base,
            symbol,
            symbol_words: symbol_words.as_ref().map(|words| words.as_slice()),
            site_words: site_words.as_ref().map(|words| words.as_slice()),
        },
    );
    if next.generation != current.generation {
        publish(next, DUMP.load(Ordering::Acquire));
        fighter_main::consider(&next);
    }
}

fn lookup_call_line() -> (Option<u64>, Option<[u32; WORD_COUNT]>) {
    let address = lookup(fighter_probe::CALL_LINE_SYMBOL);
    if address == 0 {
        return (None, None);
    }
    let symbol = address as u64;
    (Some(symbol), read_words(symbol))
}

fn read_site(base: u64) -> Option<[u32; WORD_COUNT]> {
    read_words(base.checked_add(SITE_OFFSET)?)
}

fn read_words(address: u64) -> Option<[u32; WORD_COUNT]> {
    let start = usize::try_from(address).ok()?;
    let bytes = WORD_COUNT.checked_mul(4)?;
    if start.checked_add(bytes).is_none() || !unsafe { memory::accessible(start, bytes, false) } {
        return None;
    }
    let mut words = [0; WORD_COUNT];
    for (index, word) in words.iter_mut().enumerate() {
        let ptr = start.wrapping_add(index * 4) as *const u32;
        *word = unsafe { std::ptr::read_unaligned(ptr) };
    }
    Some(words)
}

fn read_name(module: *mut Module) -> Option<[u8; 7]> {
    if module.is_null() {
        return None;
    }
    let name = unsafe { std::ptr::addr_of!((*module).Name) } as *const u8;
    if !unsafe { memory::accessible(name as usize, 7, false) } {
        return None;
    }
    let mut bytes = [0; 7];
    unsafe { std::ptr::copy_nonoverlapping(name, bytes.as_mut_ptr(), 7) };
    Some(bytes)
}

fn read_base(module: *mut Module) -> Option<u64> {
    if module.is_null() {
        return None;
    }
    let object_slot = unsafe { std::ptr::addr_of!((*module).ModuleObject) };
    if !unsafe { memory::accessible(object_slot as usize, std::mem::size_of::<*mut u8>(), false) } {
        return None;
    }
    let object = unsafe { *object_slot };
    if object.is_null() {
        return None;
    }
    let base_slot = unsafe { std::ptr::addr_of!((*object).module_base) };
    if !unsafe { memory::accessible(base_slot as usize, 8, false) } {
        return None;
    }
    let base = unsafe { *base_slot };
    if base == 0 {
        None
    } else {
        Some(base)
    }
}

fn snapshot() -> fighter_probe::Observation {
    let guard = STATE.lock().unwrap_or_else(|err| err.into_inner());
    *guard
}

fn publish(next: fighter_probe::Observation, record_events: bool) {
    {
        let mut guard = STATE.lock().unwrap_or_else(|err| err.into_inner());
        *guard = next;
    }
    if !record_events {
        return;
    }
    let (len, items) = fighter_probe::events(&next);
    for event in items.into_iter().take(len) {
        record(event.code, event.a, event.b);
    }
}
