//! Inline logical Main at `common+0x3250`.
//!
//! Smashline calls its Main callbacks after the original `call_line_status`
//! returns. This hook is inline at the entry, so it reads `x0` before that
//! prologue and is one tick earlier. The `0x9ea0` export moves `x8` and is
//! never function-hooked. `COUNT_FIGHTER_TICKS` keeps the same callback in
//! count-only mode: aggregates only, no reports and no cut-in hook.

use super::hang_trace::{self, Hook};
use super::{logging, memory, nvn, record, Code, DATA_DIR};
use crate::fighter_probe::{
    self, ArmPlan, HookView, TickLog, AGENT_BATTLE_OBJECT, AGENT_MODULE_ACCESSOR, FIGHTER_CATEGORY,
    OBJECT_ID, OBJECT_KIND, OBJECT_MODULE_ACCESSOR, SITE_OFFSET,
};
use crate::perf_scaler::{self, Action, CUT_IN_OFFSET, ENTRY_ID_WORK};
use skyline::hooks::{getRegionAddress, A64HookFunction, A64InlineHook, InlineCtx, Region};
use skyline::libc::c_void;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};

const COUNT_FLAG: &str = "COUNT_FIGHTER_TICKS";
const TICK_FILE: &str = "fighter-ticks.txt";
const MANAGER_SYMBOL: &[u8] = b"_ZN3lib9SingletonIN3app14FighterManagerEE9instance_E\0";

static COUNT_ONLY: AtomicBool = AtomicBool::new(false);
static ARMED: AtomicBool = AtomicBool::new(false);
static ARMED_BASE: AtomicU64 = AtomicU64::new(0);
static CUT_IN_INSTALLED: AtomicBool = AtomicBool::new(false);
static CUT_IN_TRIED: AtomicBool = AtomicBool::new(false);
static TRACKED_MODULE: AtomicU64 = AtomicU64::new(0);
static CUT_IN_ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static LAST_WRITTEN: AtomicU32 = AtomicU32::new(0);
static MANAGER_SLOT: AtomicUsize = AtomicUsize::new(0);
/// Tick counts: the Main hook only tries it, the diagnostics thread copies
/// it. Its flags come from `ARMED`, `CUT_IN_INSTALLED` and `COUNT_ONLY` when
/// the file is written, and `FLAGS_GENERATION` counts their changes.
static TICKS: crate::hook_lock::TryLock<TickLog> = crate::hook_lock::TryLock::new(TickLog::idle());
static FLAGS_GENERATION: AtomicU32 = AtomicU32::new(0);
static LAST_FLAGS: AtomicU32 = AtomicU32::new(0);
/// The gate between ticks and resets, which no thread waits at. Fighter Main
/// and cut-in ticks run side by side: each touches its own entry's scaler
/// slot, and the resolution policy is a lock-free cell. A reset or retarget
/// clears every slot, so it runs alone. A tick that finds a reset running
/// skips (counted in `hook_lock_skipped`); a reset that finds a tick running
/// stays requested, and the next hook entry applies it.
static TICKS_RUNNING: AtomicU32 = AtomicU32::new(0);
static RESETTING: AtomicBool = AtomicBool::new(false);
static RESET_REQUESTED: AtomicBool = AtomicBool::new(false);

struct TickPass;

impl Drop for TickPass {
    fn drop(&mut self) {
        TICKS_RUNNING.fetch_sub(1, Ordering::Release);
    }
}

fn enter_tick() -> Option<TickPass> {
    TICKS_RUNNING.fetch_add(1, Ordering::SeqCst);
    if RESETTING.load(Ordering::SeqCst) {
        TICKS_RUNNING.fetch_sub(1, Ordering::Release);
        crate::hook_lock::note_skipped();
        return None;
    }
    Some(TickPass)
}

struct ResetPass;

impl Drop for ResetPass {
    fn drop(&mut self) {
        RESETTING.store(false, Ordering::Release);
    }
}

/// Never called while the caller holds a `TickPass`.
fn enter_reset() -> Option<ResetPass> {
    if RESETTING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
        .is_err()
    {
        return None;
    }
    if TICKS_RUNNING.load(Ordering::SeqCst) != 0 {
        RESETTING.store(false, Ordering::Release);
        return None;
    }
    Some(ResetPass)
}

extern "C" {
    #[link_name = "\u{1}_ZN3app8lua_bind24WorkModule__get_int_implEPNS_26BattleObjectModuleAccessorEi"]
    fn work_int(accessor: *mut c_void, kind: i32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind24SlowModule__is_slow_implEPNS_26BattleObjectModuleAccessorE"]
    fn is_slow(accessor: *mut c_void) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind30StatusModule__status_kind_implEPNS_26BattleObjectModuleAccessorE"]
    fn status_kind(accessor: *mut c_void) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind38FighterManager__get_fighter_entry_implEPNS_14FighterManagerENS_14FighterEntryIDE"]
    fn fighter_entry(manager: *mut c_void, entry: i32) -> u64;
}

type CutInFn = unsafe extern "C" fn(u64, *mut c_void, u64, u32, u64, u32);

struct Sample {
    entry: usize,
    kind: i32,
    status: i32,
    slow: bool,
}

pub(super) fn configure() {
    cache_manager_slot();
    let count_only = Path::new(DATA_DIR).join(COUNT_FLAG).exists();
    COUNT_ONLY.store(count_only, Ordering::Release);
    FLAGS_GENERATION.fetch_add(1, Ordering::AcqRel);
}

pub(super) fn view() -> HookView {
    HookView {
        callback_armed: ARMED.load(Ordering::Acquire),
        cut_in_installed: CUT_IN_INSTALLED.load(Ordering::Acquire),
        count_only: COUNT_ONLY.load(Ordering::Acquire),
    }
}

pub(super) fn consider(obs: &fighter_probe::Observation) {
    let previous = TRACKED_MODULE.swap(obs.module, Ordering::AcqRel);
    if previous != 0 && previous != obs.module {
        ARMED.store(false, Ordering::Release);
        ARMED_BASE.store(0, Ordering::Release);
        reset_reports();
    }
    if obs.state != fighter_probe::ObserveState::Observed {
        record(Code::FighterMain, 2, 6);
        return;
    }
    match fighter_probe::arm_plan(obs.offset, &obs.symbol_words, &obs.site_words) {
        ArmPlan::Install => install_site(obs.base),
        ArmPlan::Reject(reason) => {
            record(Code::FighterMain, 2, reason_code(reason));
            logging::startup_phase("fighter_main_rejected");
        }
    }
}

pub(super) fn on_common_unloaded(module: u64) {
    if module == 0 || TRACKED_MODULE.load(Ordering::Acquire) != module {
        return;
    }
    ARMED.store(false, Ordering::Release);
    ARMED_BASE.store(0, Ordering::Release);
    TRACKED_MODULE.store(0, Ordering::Release);
    reset_reports();
    FLAGS_GENERATION.fetch_add(1, Ordering::AcqRel);
}

/// Game threads (the module load and unload hooks): reset now.
pub(super) fn reset_reports() {
    RESET_REQUESTED.store(true, Ordering::Release);
    apply_requested_reset();
}

/// Any thread: the next fighter Main, cut-in or present hook resets. The scene
/// driver calls this, from hooks and from the diagnostics thread's stale check
/// alike.
pub(super) fn request_reset() {
    RESET_REQUESTED.store(true, Ordering::Release);
}

/// A requested reset, once, when no tick is running; otherwise it stays
/// requested for the next hook entry.
fn apply_requested_reset() {
    if !RESET_REQUESTED.load(Ordering::Acquire) {
        return;
    }
    let Some(_pass) = enter_reset() else {
        return;
    };
    if RESET_REQUESTED.swap(false, Ordering::AcqRel) {
        reset_alone();
    }
}

fn reset_alone() {
    // The scaler, then the resolution policy. Clear tokens before the scene
    // reset so a stale token cannot pop the next scene's report.
    perf_scaler::clear_global();
    super::adaptive_drs::reset();
    crate::resolution::reset_scene();
}

/// The present hook, every tracked present.
pub(super) fn sync_resolution() {
    apply_requested_reset();
    let decision = crate::render_policy::load_decision();
    let setting = crate::resolution::active_setting(
        decision.live,
        crate::resolution::hooks_ready(),
        decision.committed,
        super::platform(),
        super::ultra_resolution().level(),
    );
    if !crate::resolution::needs_retarget(setting.level, setting.dynamic) {
        return;
    }
    // With a tick running, the next present retargets.
    let Some(_pass) = enter_reset() else {
        return;
    };
    if !crate::resolution::needs_retarget(setting.level, setting.dynamic) {
        return;
    }
    perf_scaler::clear_global();
    super::adaptive_drs::reset();
    if crate::resolution::retarget(setting.level, setting.dynamic) {
        perf_scaler::global().set_base(setting.level);
    }
}

pub(super) fn poll() {
    let mut ticks = {
        let guard = TICKS.lock().unwrap_or_else(|err| err.into_inner());
        *guard
    };
    let flags = FLAGS_GENERATION.load(Ordering::Acquire);
    if (ticks.generation == 0 && flags == 0)
        || (ticks.generation == LAST_WRITTEN.load(Ordering::Acquire)
            && flags == LAST_FLAGS.load(Ordering::Acquire))
    {
        return;
    }
    ticks.main_armed = ARMED.load(Ordering::Acquire);
    ticks.cut_in_installed = CUT_IN_INSTALLED.load(Ordering::Acquire);
    ticks.count_only = COUNT_ONLY.load(Ordering::Acquire);
    let text = ticks.text(crate::build_info::BUILD_ID);
    if let Ok(mut file) = File::create(Path::new(DATA_DIR).join(TICK_FILE)) {
        if file.write_all(text.as_bytes()).is_ok() && file.sync_all().is_ok() {
            LAST_WRITTEN.store(ticks.generation, Ordering::Release);
            LAST_FLAGS.store(flags, Ordering::Release);
        }
    }
}

fn install_site(base: u64) {
    cache_manager_slot();
    if base == 0 {
        record(Code::FighterMain, 2, 6);
        return;
    }
    if ARMED.load(Ordering::Acquire) && ARMED_BASE.load(Ordering::Acquire) == base {
        return;
    }
    let Some(site) = base.checked_add(SITE_OFFSET) else {
        record(Code::FighterMain, 2, 6);
        return;
    };
    if !readable(site, 4) {
        record(Code::FighterMain, 2, 6);
        return;
    }
    unsafe {
        A64InlineHook(site as *const c_void, on_fighter_main as *const c_void);
    }
    let word = read_u32(site).unwrap_or(0);
    if !fighter_probe::site_already_hooked(word) {
        record(Code::FighterMain, 2, 7);
        logging::startup_phase("fighter_main_hook_failed");
        return;
    }
    ARMED_BASE.store(base, Ordering::Release);
    ARMED.store(true, Ordering::Release);
    FLAGS_GENERATION.fetch_add(1, Ordering::AcqRel);
    record(Code::FighterMain, 1, site);
    logging::startup_phase("fighter_main_armed");
    if !COUNT_ONLY.load(Ordering::Acquire) {
        install_cut_in();
    }
}

fn install_cut_in() {
    if CUT_IN_TRIED.swap(true, Ordering::AcqRel) {
        return;
    }
    let text = unsafe { getRegionAddress(Region::Text) as usize };
    let Some(site) = text.checked_add(CUT_IN_OFFSET) else {
        super::record_install(Code::HookRejected, CUT_IN_OFFSET as u64, 0);
        return;
    };
    let word = read_u32(site as u64).unwrap_or(0);
    if !perf_scaler::cut_in_install_allowed(true, word) {
        super::record_install(Code::HookRejected, CUT_IN_OFFSET as u64, word as u64);
        logging::startup_phase("fighter_cut_in_rejected");
        return;
    }
    super::record_install(Code::HookInstalling, CUT_IN_OFFSET as u64, word as u64);
    let mut trampoline: *mut c_void = std::ptr::null_mut();
    unsafe {
        A64HookFunction(
            site as *const c_void,
            on_cut_in as *const c_void,
            &mut trampoline,
        );
    }
    if trampoline.is_null() {
        super::record_install(Code::HookRejected, CUT_IN_OFFSET as u64, 1);
        return;
    }
    CUT_IN_ORIGINAL.store(trampoline as usize, Ordering::Release);
    let after = read_u32(site as u64).unwrap_or(0);
    if !fighter_probe::site_already_hooked(after) {
        super::record_install(Code::HookRejected, CUT_IN_OFFSET as u64, after as u64);
        return;
    }
    CUT_IN_INSTALLED.store(true, Ordering::Release);
    FLAGS_GENERATION.fetch_add(1, Ordering::AcqRel);
    super::record_install(Code::HookInstalled, CUT_IN_OFFSET as u64, after as u64);
    logging::startup_phase("fighter_cut_in_ready");
}

unsafe extern "C" fn on_fighter_main(ctx: &mut InlineCtx) {
    if !ARMED.load(Ordering::Acquire) {
        return;
    }
    hang_trace::enter(Hook::FighterMain);
    fighter_main_body(ctx.registers[0].x());
    hang_trace::exit(Hook::FighterMain);
}

unsafe fn fighter_main_body(this: u64) {
    let present = nvn::tracked_presents();
    match read_tick(this) {
        Ok(sample) => {
            // While the diagnostics thread copies the counts, this tick is
            // not counted (`hook_lock_skipped` counts it instead).
            if let Some(mut ticks) = crate::hook_lock::try_take(&TICKS) {
                ticks.note(
                    sample.entry,
                    sample.kind,
                    sample.status,
                    sample.slow,
                    present,
                );
            }
            if COUNT_ONLY.load(Ordering::Acquire) {
                return;
            }
            apply_requested_reset();
            let Some(_tick) = enter_tick() else {
                return;
            };
            if !ARMED.load(Ordering::Acquire) {
                return;
            }
            let (cut, (moved, which)) = perf_scaler::global().on_main(
                perf_scaler::MainTick {
                    entry_id: sample.entry as i32,
                    entry_present: true,
                    kind: sample.kind,
                    slow: sample.slow,
                    status: sample.status,
                },
                super::drs_moves(),
            );
            // Source 2 was the Sephiroth Gigaflare trigger (removed 2026-10-17).
            note_transition(sample.entry, 1, cut);
            note_transition(sample.entry, 3, moved);
            note_move_trigger(sample.entry, which, moved);
        }
        Err(reason) => {
            if let Some(mut ticks) = crate::hook_lock::try_take(&TICKS) {
                ticks.reject(reason, present);
            }
        }
    }
}

unsafe extern "C" fn on_cut_in(
    manager: u64,
    accessor: *mut c_void,
    transactor: u64,
    kind: u32,
    data: u64,
    priority: u32,
) {
    hang_trace::enter(Hook::CutIn);
    let trampoline = CUT_IN_ORIGINAL.load(Ordering::Acquire);
    if trampoline != 0 {
        let original: CutInFn = std::mem::transmute(trampoline);
        original(manager, accessor, transactor, kind, data, priority);
    }
    cut_in_body(accessor);
    hang_trace::exit(Hook::CutIn);
}

unsafe fn cut_in_body(accessor: *mut c_void) {
    if COUNT_ONLY.load(Ordering::Acquire) || !ARMED.load(Ordering::Acquire) {
        return;
    }
    let manager = fighter_manager();
    let Some(entry) = valid_entry(accessor, manager) else {
        return;
    };
    apply_requested_reset();
    let Some(_tick) = enter_tick() else {
        return;
    };
    if COUNT_ONLY.load(Ordering::Acquire) || !ARMED.load(Ordering::Acquire) {
        return;
    }
    let action = perf_scaler::global().cut_in_start(entry, true);
    note_transition(entry as usize, 1, action);
}

unsafe fn read_tick(this: u64) -> Result<Sample, u8> {
    if !readable(this, 0x48) {
        return Err(1);
    }
    let object = read_u64(this, AGENT_BATTLE_OBJECT).unwrap_or(0);
    if object == 0 {
        return Err(3);
    }
    if !readable(object, OBJECT_MODULE_ACCESSOR + 8) {
        return Err(4);
    }
    let id = read_u32(object.wrapping_add(OBJECT_ID as u64)).unwrap_or(u32::MAX);
    if fighter_probe::category_of(id) != FIGHTER_CATEGORY {
        return Err(5);
    }
    let kind = read_u32(object.wrapping_add(OBJECT_KIND as u64)).unwrap_or(u32::MAX) as i32;
    let Some(accessor) = read_u64(this, AGENT_MODULE_ACCESSOR) else {
        return Err(6);
    };
    let Some(object_accessor) = read_u64(object, OBJECT_MODULE_ACCESSOR) else {
        return Err(4);
    };
    if accessor == 0 || !readable(accessor, 0x10) {
        return Err(6);
    }
    if accessor != object_accessor {
        return Err(9);
    }
    let manager = fighter_manager();
    if manager.is_null() {
        return Err(8);
    }
    let Some(entry) = valid_entry(accessor as *mut c_void, manager) else {
        return Err(7);
    };
    let status = status_kind(accessor as *mut c_void);
    let slow = is_slow(accessor as *mut c_void);
    Ok(Sample {
        entry: entry as usize,
        kind,
        status,
        slow,
    })
}

unsafe fn valid_entry(accessor: *mut c_void, manager: *mut c_void) -> Option<i32> {
    if accessor.is_null()
        || manager.is_null()
        || !readable(accessor as u64, 0x10)
        || !readable(manager as u64, 8)
    {
        return None;
    }
    let entry = work_int(accessor, ENTRY_ID_WORK);
    if !(0..perf_scaler::ENTRIES as i32).contains(&entry) {
        return None;
    }
    if fighter_entry(manager, entry) == 0 {
        return None;
    }
    Some(entry)
}

fn cache_manager_slot() {
    if MANAGER_SLOT.load(Ordering::Acquire) != 0 {
        return;
    }
    let slot = lookup(MANAGER_SYMBOL);
    if slot != 0 && readable(slot as u64, 8) {
        MANAGER_SLOT.store(slot, Ordering::Release);
    }
}

fn fighter_manager() -> *mut c_void {
    let slot = MANAGER_SLOT.load(Ordering::Acquire);
    if !readable(slot as u64, 8) {
        return std::ptr::null_mut();
    }
    let manager = unsafe { std::ptr::read_unaligned(slot as *const *mut c_void) };
    if manager.is_null() || !readable(manager as u64, 8) {
        return std::ptr::null_mut();
    }
    manager
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

fn readable(address: u64, size: usize) -> bool {
    let Ok(start) = usize::try_from(address) else {
        return false;
    };
    unsafe { memory::accessible(start, size, false) }
}

fn read_u64(base: u64, offset: usize) -> Option<u64> {
    let address = base.checked_add(offset as u64)?;
    if !readable(address, 8) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(address as *const u64) })
}

fn read_u32(address: u64) -> Option<u32> {
    if !readable(address, 4) {
        return None;
    }
    Some(unsafe { std::ptr::read_unaligned(address as *const u32) })
}

/// Code 108 edge for the 0.5.2 move table: a = 1 | move << 8 | entry << 16,
/// b = 1 push / 2 pop | pushed level << 8.
fn note_move_trigger(entry: usize, which: Option<u8>, action: Action) {
    let (edge, Some(which)) = (
        match action {
            Action::Started => 1u64,
            Action::Ended => 2,
            _ => return,
        },
        which,
    ) else {
        return;
    };
    let level = crate::perf_scaler::intensive_level(perf_scaler::global().base_level()) as u64;
    record(
        Code::DrsTrigger,
        1 | (u64::from(which) << 8) | ((entry as u64) << 16),
        edge | (level << 8),
    );
}

fn note_transition(entry: usize, source: u64, action: Action) {
    let action_code = match action {
        Action::Started => 1u64,
        Action::Ended => 2,
        _ => return,
    };
    record(
        Code::FighterTransition,
        entry as u64 | (action_code << 8) | (source << 16),
        0,
    );
}

fn reason_code(reason: &str) -> u64 {
    match reason {
        "already_hooked" => 1,
        "export_offset" => 2,
        "export_words" => 3,
        "site_words" => 4,
        "short" => 5,
        "unmapped" => 6,
        "hook_failed" => 7,
        _ => 9,
    }
}
