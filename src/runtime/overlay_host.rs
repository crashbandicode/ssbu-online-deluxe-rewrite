//! ImGui host. This NRO still embeds imgui-smash and uses it when no standalone
//! `libimgui_smash.nro` is loaded. When that plugin is loaded, Skyline has
//! already `LoadModule`'d every plugin before any `main` (`PluginManager.cpp`),
//! and the backend creates its context on the first NVN init, after every
//! `main`. Registering with the standalone during our `main` is therefore
//! before `InitImGui` regardless of which `main` runs first.
//!
//! The standalone path does not call this NRO's `imgui_smash_init` and does
//! not install this NRO's NVN renderer. Draw callbacks and `igGetMainViewport`
//! run only after `igSetCurrentContext` on this NRO's cimgui, using the
//! context pointer the standalone passes in. `ig*` are direct links into this
//! NRO's compiled cimgui. They are not dynamic exports of this NRO.
//! `LookupSymbol` is not used for them: a standalone copy can export the same
//! names, and the first hit would be that other module.
//!
//! An export counts only when its text mapping begins with that NRO's own
//! header and the address falls in the header's text segment. The version tag
//! is read only from that same header's ro segment. Setup, draw, and get must
//! share that image base. A symbol that cannot be tied to one header is a
//! refusal, not a nearby-memory search. An unknown combination installs no
//! ImGui backend and does not call `ig*`. NVN observations still chain so the
//! rewrite's window setup keeps running.
//!
//! The bootstrap callback is exported as `deluxe_rewrite_nvn_bootstrap_hook`.
//! Its address is installed only when `loaded_nro` ties it to this NRO.
//! A different module is rejected before `install_hook`. `-Bsymbolic-functions`
//! binds this NRO's own function references locally; a later plugin can still
//! resolve an export that keeps the old shared name, so the prefix stays.
//!
//! C ABI is transcribed from `include/imgui_smash.h` and the standalone
//! `src/lib.rs` (`imgui_setup_context_export`, `imgui_smash_add_on_draw_frame_wrapper`).
//! `imgui_get_context_export` is absent from the Elite Legal Stages NRO and is
//! optional. `IM_ASSERT` in this backend build is `SMASH_ASSERT` and aborts.
use super::{logging, memory, record, Code};
use crate::imgui_host::{
    owns_replacement, parse_nro_header, ro_contains_tag, select_imgui_host, text_contains,
    ImguiHostChoice, NroSegments,
};
use skyline::libc::{c_char, c_void};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};

type InitFunc = unsafe extern "C" fn();
type ProcDrawFunc = unsafe extern "C" fn();
type LoggerFunc = unsafe extern "C" fn(*const c_char, usize);
type SetupExport = unsafe extern "C" fn(unsafe extern "C" fn(*mut u64));
type DrawWrapper = unsafe extern "C" fn(*const c_void);
type GetContextExport = unsafe extern "C" fn() -> *mut u64;

#[repr(C)]
struct ViewportHead {
    _id: u32,
    _flags: i32,
    _pos: [f32; 2],
    size: [f32; 2],
}

extern "C" {
    fn imgui_smash_init(post_init: Option<InitFunc>, render: Option<ProcDrawFunc>);
    fn imgui_smash_bootstrap_hook(name: *const c_char, original: *const c_void) -> *const c_void;
    fn imgui_smash_add_on_draw_frame(render: ProcDrawFunc);
    fn imgui_smash_context_setup(callback: InitFunc);
    fn imgui_smash_set_logger(callback: LoggerFunc);
    fn igGetMainViewport() -> *const ViewportHead;
    fn nvnBootstrapLoader(name: *const c_char) -> *const c_void;
}

const SETUP_EXPORT: &[u8] = b"imgui_setup_context_export\0";
const GET_CONTEXT_EXPORT: &[u8] = b"imgui_get_context_export\0";
const DRAW_WRAPPER: &[u8] = b"imgui_smash_add_on_draw_frame_wrapper\0";

const FALLBACK_DISPLAY: (f32, f32) = (1280.0, 720.0);
const SITE_BOOTSTRAP: u64 = 0x200;
const NRO_HEADER_SIZE: usize = 0x80;
const CHAIN_EMBEDDED: u8 = 0;
const CHAIN_EXTERNAL: u8 = 1;
const STATE_IDLE: u8 = 0;
const STATE_REFUSED: u8 = 1;
const STATE_EMBEDDED: u8 = 2;
const STATE_EXTERNAL: u8 = 3;
static STATE: AtomicU8 = AtomicU8::new(STATE_IDLE);
static CHAIN: AtomicU8 = AtomicU8::new(CHAIN_EMBEDDED);
static CONTEXT_READY: AtomicBool = AtomicBool::new(false);
static BACKEND_READY: AtomicBool = AtomicBool::new(false);
static BOOTSTRAP_CALLS: AtomicU64 = AtomicU64::new(0);
static LOG_COUNT: AtomicU64 = AtomicU64::new(0);
static RESIZE_COUNT: AtomicU64 = AtomicU64::new(0);
static LOGGER_GUARD: AtomicBool = AtomicBool::new(false);
static DRAW: AtomicUsize = AtomicUsize::new(0);
static EXTERNAL_CONTEXT: AtomicUsize = AtomicUsize::new(0);
static GET_CONTEXT: AtomicUsize = AtomicUsize::new(0);

#[skyline::hook(replace = nvnBootstrapLoader)]
unsafe fn deluxe_rewrite_nvn_bootstrap_hook(name: *const c_char) -> *const c_void {
    BOOTSTRAP_CALLS.fetch_add(1, Ordering::Relaxed);
    let chained = if CHAIN.load(Ordering::Acquire) == CHAIN_EXTERNAL {
        original!()(name)
    } else {
        imgui_smash_bootstrap_hook(name, original!() as *const c_void)
    };
    super::nvn::bootstrap(name, chained)
}

unsafe extern "C" fn on_context_setup() {
    CONTEXT_READY.store(true, Ordering::Release);
}

unsafe extern "C" fn on_backend_ready() {
    if !BACKEND_READY.swap(true, Ordering::AcqRel) {
        record(
            Code::OverlayBackendReady,
            SITE_BOOTSTRAP + 1,
            BOOTSTRAP_CALLS.load(Ordering::Relaxed),
        );
    }
}

unsafe extern "C" fn accept_external_context(ctx: *mut u64) {
    if ctx.is_null() || (ctx as usize) & 7 != 0 {
        record(Code::HookRejected, SITE_BOOTSTRAP, 5);
        return;
    }
    EXTERNAL_CONTEXT.store(ctx as usize, Ordering::Release);
    CONTEXT_READY.store(true, Ordering::Release);
}

unsafe extern "C" fn external_draw() {
    let mut ctx = EXTERNAL_CONTEXT.load(Ordering::Acquire);
    if ctx == 0 {
        let getter = GET_CONTEXT.load(Ordering::Acquire);
        if getter != 0 {
            let get: GetContextExport = std::mem::transmute(getter);
            let ptr = get();
            if !ptr.is_null() && (ptr as usize) & 7 == 0 {
                ctx = ptr as usize;
                EXTERNAL_CONTEXT.store(ctx, Ordering::Release);
                CONTEXT_READY.store(true, Ordering::Release);
            }
        }
    }
    if ctx == 0 {
        return;
    }
    super::imgui::igSetCurrentContext(ctx as *mut c_void);
    if super::imgui::igGetCurrentContext() != ctx as *mut c_void {
        record(Code::HookRejected, SITE_BOOTSTRAP, 6);
        return;
    }
    if !BACKEND_READY.swap(true, Ordering::AcqRel) {
        record(
            Code::OverlayBackendReady,
            SITE_BOOTSTRAP + 2,
            BOOTSTRAP_CALLS.load(Ordering::Relaxed),
        );
    }
    let draw = DRAW.load(Ordering::Acquire);
    if draw != 0 {
        let draw: ProcDrawFunc = std::mem::transmute(draw);
        draw();
    }
}

unsafe extern "C" fn backend_logger(message: *const c_char, len: usize) {
    if LOGGER_GUARD
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    observe_backend_log(message, len);
    LOGGER_GUARD.store(false, Ordering::Release);
}

unsafe fn observe_backend_log(message: *const c_char, len: usize) {
    let resizing = if message.is_null() || len == 0 {
        false
    } else {
        let n = len.min(512);
        let bytes = std::slice::from_raw_parts(message.cast::<u8>(), n);
        crate::render_trace::contains_resizing(bytes)
    };
    if resizing && embedded() {
        let count = RESIZE_COUNT
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        let tick = super::api_trace::start();
        if tick != 0 {
            super::api_trace::TRACE.observe(crate::render_trace::resize_event(tick, count));
        }
    }
    let n = LOG_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    if !(n <= 8 || n.is_multiple_of(256)) || message.is_null() || len == 0 {
        return;
    }
    let len = len.min(512);
    let bytes = std::slice::from_raw_parts(message, len);
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..end]);
    skyline::println!("[Deluxe/imgui #{}] {}", n, text.trim_end());
}

pub(super) fn embedded() -> bool {
    STATE.load(Ordering::Acquire) == STATE_EMBEDDED
}

struct LoadedNro {
    base: usize,
    segments: NroSegments,
}

/// Skyline maps an NRO at its file base. The Elite header has `text_offset == 0`,
/// so the text mapping starts at that header. A symbol whose mapping does not
/// begin with `NRO0`, or whose text offset is not zero, is not given a module.
fn loaded_nro(symbol: usize) -> Option<LoadedNro> {
    let (map_base, map_size) = unsafe { memory::region(symbol) }?;
    if map_size < NRO_HEADER_SIZE
        || !unsafe { memory::accessible(map_base, NRO_HEADER_SIZE, false) }
    {
        return None;
    }
    let mut header = [0u8; NRO_HEADER_SIZE];
    unsafe {
        header.copy_from_slice(std::slice::from_raw_parts(
            map_base as *const u8,
            NRO_HEADER_SIZE,
        ));
    }
    let segments = parse_nro_header(&header)?;
    if segments.text_offset != 0 || segments.text_size as usize > map_size {
        return None;
    }
    let offset = symbol.checked_sub(map_base)?;
    if !text_contains(&segments, offset) {
        return None;
    }
    Some(LoadedNro {
        base: map_base,
        segments,
    })
}

fn ro_tag_present(image: &LoadedNro) -> bool {
    let Some(start) = image.base.checked_add(image.segments.ro_offset as usize) else {
        return false;
    };
    let size = image.segments.ro_size as usize;
    if !unsafe { memory::accessible(start, size, false) } {
        return false;
    }
    let ro = unsafe { std::slice::from_raw_parts(start as *const u8, size) };
    ro_contains_tag(ro)
}

enum Export {
    Absent,
    Bound(usize, LoadedNro),
    Unbound,
}

fn locate_export(name: &[u8]) -> Export {
    let mut address = 0usize;
    unsafe {
        skyline::nn::ro::LookupSymbol(&mut address, name.as_ptr());
    }
    if address == 0 {
        return Export::Absent;
    }
    let Some(image) = loaded_nro(address) else {
        return Export::Unbound;
    };
    if loaded_nro(locate_export as *const () as usize).is_some_and(|own| own.base == image.base) {
        return Export::Absent;
    }
    Export::Bound(address, image)
}

fn refuse(phase: &str, detail: u64) -> bool {
    STATE.store(STATE_REFUSED, Ordering::Release);
    record(Code::HookRejected, SITE_BOOTSTRAP, detail);
    logging::startup_phase(phase);
    skyline::println!("[Deluxe] imgui host refused ({phase}); menu callbacks not installed");
    false
}

enum BootstrapFailure {
    EntryUnreadable,
    ForeignReplacement,
}

fn log_bootstrap_binding(
    replacement: usize,
    own_module: Option<usize>,
    replacement_module: Option<usize>,
) {
    let chain = if CHAIN.load(Ordering::Acquire) == CHAIN_EXTERNAL {
        "external"
    } else {
        "embedded"
    };
    let own_base = own_module.unwrap_or(0);
    let replacement_base = replacement_module.unwrap_or(0);
    logging::startup_phase(&format!(
        "nvn_bootstrap chain={chain} own_module={own_base:#x} replacement={replacement:#x} replacement_module={replacement_base:#x}"
    ));
    skyline::println!(
        "[Deluxe] nvn bootstrap chain={chain} own_module={own_base:#x} replacement={replacement:#x} replacement_module={replacement_base:#x}"
    );
}

/// Installs this NRO's bootstrap callback only when its address is inside this NRO.
/// A preempted export is recorded and left uninstalled.
fn hook_bootstrap() -> Result<(), BootstrapFailure> {
    let replacement = deluxe_rewrite_nvn_bootstrap_hook as *const () as usize;
    let own_module = loaded_nro(hook_bootstrap as *const () as usize).map(|image| image.base);
    let replacement_module = loaded_nro(replacement).map(|image| image.base);
    log_bootstrap_binding(replacement, own_module, replacement_module);
    if !owns_replacement(own_module, replacement_module) {
        record(Code::HookRejected, SITE_BOOTSTRAP, 9);
        logging::startup_phase("overlay_host_foreign_bootstrap");
        skyline::println!("[Deluxe] nvn bootstrap replacement is not this NRO; hook not installed");
        return Err(BootstrapFailure::ForeignReplacement);
    }
    let entry = nvnBootstrapLoader as *const () as usize;
    if entry == 0 || !unsafe { memory::accessible(entry, 4, false) } {
        return Err(BootstrapFailure::EntryUnreadable);
    }
    let word = unsafe { *(entry as *const u32) };
    record(Code::HookEntryWord, SITE_BOOTSTRAP, word as u64);
    logging::install_checkpoint(Code::HookEntryWord, entry as u64, word as u64);
    let already_far_hooked = word == 0x5800_0051;
    let already_branch = word & 0xfc00_0000 == 0x1400_0000;
    if already_far_hooked || already_branch {
        record(Code::Compatibility, 4, word as u64);
    }
    record(Code::HookInstalling, SITE_BOOTSTRAP, entry as u64);
    skyline::install_hook!(deluxe_rewrite_nvn_bootstrap_hook);
    record(Code::HookInstalled, SITE_BOOTSTRAP, entry as u64);
    Ok(())
}

fn fail_bootstrap(error: BootstrapFailure) -> bool {
    match error {
        BootstrapFailure::EntryUnreadable => refuse("overlay_host_entry_unreadable", 2),
        BootstrapFailure::ForeignReplacement => {
            STATE.store(STATE_REFUSED, Ordering::Release);
            skyline::println!(
                "[Deluxe] imgui host refused (overlay_host_foreign_bootstrap); menu callbacks not installed"
            );
            false
        }
    }
}

pub(super) fn install(draw: unsafe extern "C" fn()) -> bool {
    if STATE
        .compare_exchange(
            STATE_IDLE,
            STATE_EMBEDDED,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_err()
    {
        let state = STATE.load(Ordering::Acquire);
        return state == STATE_EMBEDDED || state == STATE_EXTERNAL;
    }
    logging::startup_phase("overlay_host_begin");

    let setup = locate_export(SETUP_EXPORT);
    let draw_wrapper = locate_export(DRAW_WRAPPER);
    let get_context = locate_export(GET_CONTEXT_EXPORT);
    let (setup_addr, draw_addr, get_addr, choice) = match (&setup, &draw_wrapper, &get_context) {
        (Export::Absent, Export::Absent, Export::Absent) => (
            None,
            None,
            None,
            select_imgui_host(false, false, false, false),
        ),
        (
            Export::Bound(setup_addr, setup_image),
            Export::Bound(draw_addr, draw_image),
            get_context,
        ) if setup_image.base == draw_image.base
            && match get_context {
                Export::Absent => true,
                Export::Bound(_, get_image) => get_image.base == setup_image.base,
                Export::Unbound => false,
            } =>
        {
            let get_addr = match get_context {
                Export::Bound(addr, _) => Some(*addr),
                _ => None,
            };
            (
                Some(*setup_addr),
                Some(*draw_addr),
                get_addr,
                select_imgui_host(true, true, get_addr.is_some(), ro_tag_present(setup_image)),
            )
        }
        _ => (
            None,
            None,
            None,
            select_imgui_host(true, false, false, false),
        ),
    };

    match choice {
        ImguiHostChoice::External => {
            CHAIN.store(CHAIN_EXTERNAL, Ordering::Release);
            // Selected before setup. `ready` is still false; NVN init is after main.
            STATE.store(STATE_EXTERNAL, Ordering::Release);
            if let Err(error) = hook_bootstrap() {
                return fail_bootstrap(error);
            }
            logging::startup_phase("overlay_host_external_chain");
            let (Some(setup_addr), Some(draw_addr)) = (setup_addr, draw_addr) else {
                return refuse("overlay_host_refused_unknown_abi", 8);
            };
            DRAW.store(draw as usize, Ordering::Release);
            if let Some(getter) = get_addr {
                GET_CONTEXT.store(getter, Ordering::Release);
            }
            unsafe {
                let setup: SetupExport = std::mem::transmute(setup_addr);
                setup(accept_external_context);
                let add_draw: DrawWrapper = std::mem::transmute(draw_addr);
                add_draw(external_draw as *const c_void);
            }
            record(Code::Compatibility, 5, 19263);
            logging::startup_phase("overlay_host_external_ready");
            skyline::println!(
                "[Deluxe] standalone imgui-smash 1.92.7 WIP (19263); embedded init skipped"
            );
            true
        }
        ImguiHostChoice::Refuse => {
            CHAIN.store(CHAIN_EXTERNAL, Ordering::Release);
            let _ = hook_bootstrap();
            skyline::println!(
                "[Deluxe] standalone imgui-smash ABI is not 1.92.7 WIP (19263); embedded init skipped"
            );
            refuse("overlay_host_refused_unknown_abi", 8)
        }
        ImguiHostChoice::Embedded => {
            CHAIN.store(CHAIN_EMBEDDED, Ordering::Release);
            if let Err(error) = hook_bootstrap() {
                return fail_bootstrap(error);
            }
            logging::startup_phase("overlay_host_bootstrap_hooked");
            unsafe {
                imgui_smash_set_logger(backend_logger);
                imgui_smash_context_setup(on_context_setup);
                imgui_smash_init(Some(on_backend_ready), None);
                imgui_smash_add_on_draw_frame(draw);
            }
            logging::startup_phase("overlay_host_ready");
            true
        }
    }
}

pub(super) fn ready() -> bool {
    BACKEND_READY.load(Ordering::Acquire)
}

pub(super) fn bootstrap_calls() -> u64 {
    BOOTSTRAP_CALLS.load(Ordering::Relaxed)
}

pub(super) unsafe fn display_size() -> (f32, f32) {
    if !CONTEXT_READY.load(Ordering::Acquire) || !BACKEND_READY.load(Ordering::Acquire) {
        return FALLBACK_DISPLAY;
    }
    let viewport = igGetMainViewport();
    if viewport.is_null() {
        return FALLBACK_DISPLAY;
    }
    let [width, height] = (*viewport).size;
    let plausible = |v: f32| v.is_finite() && (64.0..=8192.0).contains(&v);
    if plausible(width) && plausible(height) {
        (width, height)
    } else {
        FALLBACK_DISPLAY
    }
}
