//! Embedded imgui-smash host: this NRO owns the NVN bootstrap interposition and
//! the single ImGui context, so a global overlay works in every scene without a
//! `Layout::Draw` hook or any game-text patch.
//!
//! Mirrors the upstream host glue (`Coolsonickirby/imgui-smash` `src/lib.rs`
//! at commit `bb47b205ed0be2fb864269ee2d37fc886b81221e`) with three changes:
//! an external host is detected and refused instead of racing it, startup is
//! checkpointed to the SD card in a bounded number of steps, and the logger is
//! lossy and rate-limited instead of `to_str().unwrap()`.
//!
//! C ABI is transcribed from `include/imgui_smash.h` and `include/smash_types.h`
//! at that commit; the viewport head matches `src/cimgui.h` (`ImGuiViewport`).
//! Drawing bindings (`ig*`) belong to the UI module; the archive is already on
//! the link line via `build.rs`, so plain `extern "C"` declarations resolve.
//!
//! Important: in this backend build `IM_ASSERT` is `SMASH_ASSERT`, which aborts
//! the game. Any ImGui misuse inside the draw callback (Begin/End mismatch,
//! unbalanced Push/Pop, calling `ig*` outside a frame) is fatal, not a log line.
use super::{logging, memory, record, Code};
use skyline::libc::{c_char, c_void};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};

// smash_types.h
type InitFunc = unsafe extern "C" fn();
type ProcDrawFunc = unsafe extern "C" fn();
type LoggerFunc = unsafe extern "C" fn(*const c_char, usize);

/// First fields of `ImGuiViewport` (cimgui.h at the pinned commit):
/// `ImGuiID ID; ImGuiViewportFlags Flags; ImVec2 Pos; ImVec2 Size; ...`.
/// Only this prefix is read; later fields are version-sensitive.
#[repr(C)]
struct ViewportHead {
    _id: u32,
    _flags: i32,
    _pos: [f32; 2],
    size: [f32; 2],
}

extern "C" {
    // libimgui_smash.a (imgui_smash.cpp / cimgui.cpp)
    fn imgui_smash_init(post_init: Option<InitFunc>, render: Option<ProcDrawFunc>);
    fn imgui_smash_bootstrap_hook(name: *const c_char, original: *const c_void) -> *const c_void;
    fn imgui_smash_add_on_draw_frame(render: ProcDrawFunc);
    fn imgui_smash_context_setup(callback: InitFunc);
    fn imgui_smash_set_logger(callback: LoggerFunc);
    fn igGetMainViewport() -> *const ViewportHead;

    // Game import: the NVN loader entry the backend interposes.
    fn nvnBootstrapLoader(name: *const c_char) -> *const c_void;
}

/// Exports that only the standalone `libimgui_smash.nro` host defines. If any
/// resolves, a second bootstrap interposer would double-initialise ImGui.
const EXTERNAL_HOST_SYMBOLS: [&[u8]; 3] = [
    b"imgui_setup_context_export\0",
    b"imgui_get_context_export\0",
    b"imgui_smash_add_on_draw_frame_wrapper\0",
];

/// Backend viewport configured in `user_config/imgui_backend_config.h`.
const FALLBACK_DISPLAY: (f32, f32) = (1280.0, 720.0);

/// Diagnostic site id used in the `a` field of host records (hooks use text
/// offsets, effects use 0x100..; 0x200 is this host).
const SITE_BOOTSTRAP: u64 = 0x200;

const STATE_IDLE: u8 = 0;
const STATE_REFUSED: u8 = 1;
const STATE_INSTALLED: u8 = 2;

static STATE: AtomicU8 = AtomicU8::new(STATE_IDLE);
static CONTEXT_READY: AtomicBool = AtomicBool::new(false);
static BACKEND_READY: AtomicBool = AtomicBool::new(false);
static BOOTSTRAP_CALLS: AtomicU64 = AtomicU64::new(0);
static LOG_COUNT: AtomicU64 = AtomicU64::new(0);

#[skyline::hook(replace = nvnBootstrapLoader)]
unsafe fn nvn_bootstrap_hook(name: *const c_char) -> *const c_void {
    BOOTSTRAP_CALLS.fetch_add(1, Ordering::Relaxed);
    // The backend returns either its wrapper or the original pointer; the
    // trampoline is accepted by the header contract ("trampoline functions are
    // also accepted, provided they call the original").
    let next = imgui_smash_bootstrap_hook(name, original!() as *const c_void);
    super::nvn::bootstrap(name, next)
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

/// Lossy, bounded forwarder: first eight backend messages, then one in 256.
/// Never trusts the buffer to be valid UTF-8 or nul-terminated.
unsafe extern "C" fn backend_logger(message: *const c_char, len: usize) {
    let n = LOG_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    if !(n <= 8 || n % 256 == 0) {
        return;
    }
    if message.is_null() || len == 0 {
        return;
    }
    let len = len.min(512);
    let bytes = std::slice::from_raw_parts(message as *const u8, len);
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..end]);
    skyline::println!("[Deluxe/imgui #{}] {}", n, text.trim_end());
}

fn external_host_symbol() -> Option<&'static [u8]> {
    for symbol in EXTERNAL_HOST_SYMBOLS {
        let mut address = 0usize;
        unsafe {
            skyline::nn::ro::LookupSymbol(&mut address, symbol.as_ptr());
        }
        if address != 0 {
            return Some(symbol);
        }
    }
    None
}

fn refuse(phase: &str, detail: u64) -> bool {
    STATE.store(STATE_REFUSED, Ordering::Release);
    record(Code::HookRejected, SITE_BOOTSTRAP, detail);
    logging::startup_phase(phase);
    false
}

/// Installs the embedded ImGui host and registers `draw` as the per-frame
/// callback. Call once from plugin `main`, before the game initialises NVN
/// (plugin `main` runs inside `nn::ro::Initialize`, which is early enough; the
/// fork's bundle relied on the same timing).
///
/// `draw` runs on the thread that submits NVN commands, every frame, in every
/// scene, between `ImGui::NewFrame` and `ImGui::Render`. Keep it to UI state
/// and atomics; it is not a place for engine transitions.
///
/// Returns `false` without installing anything when an external ImGui host is
/// present, when the loader entry is not readable, or on a repeated call.
pub(super) fn install(draw: unsafe extern "C" fn()) -> bool {
    if STATE
        .compare_exchange(
            STATE_IDLE,
            STATE_INSTALLED,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_err()
    {
        return STATE.load(Ordering::Acquire) == STATE_INSTALLED;
    }
    logging::startup_phase("overlay_host_begin");

    if let Some(symbol) = external_host_symbol() {
        record(Code::Compatibility, 3, symbol[0] as u64);
        skyline::println!(
            "[Deluxe] external imgui-smash host detected ({}); embedded overlay host not installed",
            String::from_utf8_lossy(&symbol[..symbol.len() - 1])
        );
        return refuse("overlay_host_refused_external_host", 1);
    }

    let entry = nvnBootstrapLoader as *const () as usize;
    if entry == 0 || !unsafe { memory::accessible(entry, 4, false) } {
        return refuse("overlay_host_entry_unreadable", 2);
    }
    let word = unsafe { *(entry as *const u32) };
    record(Code::HookEntryWord, SITE_BOOTSTRAP, word as u64);
    logging::install_checkpoint(Code::HookEntryWord, entry as u64, word as u64);
    let already_far_hooked = word == 0x5800_0051;
    let already_branch = word & 0xfc00_0000 == 0x1400_0000;
    if already_far_hooked || already_branch {
        // Another interposer owns this entry (not an ImGui host, those were
        // excluded above). Chaining through it is the normal Skyline behaviour;
        // record it so a double-overlay symptom can be attributed.
        record(Code::Compatibility, 4, word as u64);
    }

    record(Code::HookInstalling, SITE_BOOTSTRAP, entry as u64);
    skyline::install_hook!(nvn_bootstrap_hook);
    record(Code::HookInstalled, SITE_BOOTSTRAP, entry as u64);
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

/// True once the backend reported full initialisation (first eligible command
/// buffer). Before that the draw callback is never invoked.
pub(super) fn ready() -> bool {
    BACKEND_READY.load(Ordering::Acquire)
}

/// Number of `nvnBootstrapLoader` calls seen through the hook; zero after
/// install means the hook never fired (installed too late or wrong symbol).
pub(super) fn bootstrap_calls() -> u64 {
    BOOTSTRAP_CALLS.load(Ordering::Relaxed)
}

/// Main-viewport size from ImGui. Valid only inside the draw callback (a frame
/// must be active and the context created); otherwise, or if the value is
/// implausible, returns the backend's configured 1280x720 so layout math never
/// sees NaN or zero.
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
