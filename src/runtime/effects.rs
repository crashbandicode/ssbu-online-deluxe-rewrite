//! Narrow C ABI surface transcribed from skyline-smash bed642e/src/cpp.rs.
//! Opaque pointers are forwarded untouched. Eight request hooks chain through
//! original!(), including Training Modpack's shared EffectModule hooks.
//! `kill_kind` and `detach_kind` return `u64`. Each calls the clone first, then
//! the original hash, and returns only that original call. ACMD
//! `EFFECT_OFF_KIND` and `EFFECT_DETACH_KIND` are void. They call the same
//! lua_bind entry for the clone, then the original command, and do not rewrite
//! the Lua stack. A thread-local key of accessor, original hash, and
//! kill-or-detach skips only that duplicate clone call. FX mode is ignored.
#![allow(clippy::too_many_arguments)] // Fixed native C++ ABI.
use super::{record, Code};
use crate::effects;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[repr(C)]
struct OpaqueAccessor {
    _opaque: [u8; 0],
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Hash40 {
    hash: u64,
}
#[repr(C)]
struct Vector3f {
    x: f32,
    y: f32,
    z: f32,
}

static CRITICAL_MARKER: AtomicBool = AtomicBool::new(false);
static CRITICAL_SEEN: AtomicU64 = AtomicU64::new(0);

/// Observe an allowlisted Special Zoom background before any On return.
/// The marker file is not read here. Suppression does not touch screen counters.
fn note_critical(hash: u64, path: u64, mode: crate::config::FxMode, scope: bool) -> bool {
    if !crate::critical_probe::is_listed(hash) {
        return false;
    }
    let suppress =
        crate::critical_probe::suppress(CRITICAL_MARKER.load(Ordering::Acquire), mode, scope, hash);
    let ordinal = CRITICAL_SEEN.fetch_add(1, Ordering::Relaxed) + 1;
    if crate::critical_probe::should_log(ordinal, super::api_trace::start() != 0) {
        record(
            Code::FxCriticalRequest,
            hash,
            crate::critical_probe::detail(path, suppress),
        );
    }
    suppress
}

fn latch_critical_probe() {
    let present = std::path::Path::new(super::DATA_DIR)
        .join(crate::critical_probe::MARKER_FILE)
        .exists();
    CRITICAL_MARKER.store(present, Ordering::Release);
    record(Code::CriticalProbe, u64::from(present), 0);
}

fn log_intent(intent: crate::effects_reduced::FxIntent, hash: u64, total: u64) {
    use crate::effects_reduced::FxIntent;
    // Existing finite CAPTURE_TIMING sessions also capture each FX decision.
    // Ordinary play retains sampled logging; the callback never writes files.
    if total == 0 || !(total == 1 || total.is_multiple_of(128) || super::api_trace::start() != 0) {
        return;
    }
    let code = match intent {
        FxIntent::Suppress => Code::FxSuppressed,
        FxIntent::ReducedCull => Code::FxReducedCull,
        FxIntent::ReducedKeep => Code::FxReducedKeep,
        FxIntent::ReducedRestore => Code::FxReducedRestore,
        FxIntent::ReducedPreserve => Code::FxReducedPreserve,
        FxIntent::Passthrough => return,
    };
    record(code, hash, total);
}

unsafe fn policy_hash(accessor: *mut OpaqueAccessor, hash: u64) -> u64 {
    use crate::config::FxMode;
    let mode = super::fx_mode();
    let scope = super::fx_scope_allows();
    if note_critical(hash, crate::critical_probe::PATH_ORDINARY, mode, scope) {
        return effects::NULL_EFFECT;
    }
    if mode == FxMode::On || (mode == FxMode::Off && !scope) {
        return hash;
    }
    let context = if (mode == FxMode::Reduced
        && (crate::effects_reduced::reduced_named(hash) || crate::eff_variant::wants_context(hash)))
        || (mode == FxMode::Off
            && effects::rewrite(FxMode::Off, hash) == hash
            && effects::needs_context(hash))
    {
        super::effect_context::from_accessor(accessor.cast())
    } else {
        None
    };
    let intent = crate::effects_reduced::classify(mode, scope, hash, context);
    let total = crate::effects_reduced::observe(intent, false);
    log_intent(intent, hash, total);
    let requested = crate::effects_reduced::request_hash(intent, hash);
    let mapped = crate::eff_variant::forward(mode, scope, requested, context);
    if mapped != requested {
        let swaps = crate::eff_variant::observe_swap(hash);
        if swaps == 1 || swaps.is_multiple_of(128) || super::api_trace::start() != 0 {
            record(
                Code::FxVariantUse,
                crate::eff_variant::source_file(hash),
                hash,
            );
        }
    }
    mapped
}

fn skip_screen(hash: u64) -> bool {
    let mode = super::fx_mode();
    let scope = super::fx_scope_allows();
    let intent = crate::effects_reduced::classify_screen(mode, scope, hash);
    let total = crate::effects_reduced::observe(intent, true);
    log_intent(intent, hash, total);
    crate::effects_reduced::skips_screen(intent)
}

mod abi {
    use super::{Hash40, OpaqueAccessor, Vector3f};
    extern "C" {
        #[link_name = "\u{1}_ZN3app8lua_bind22EffectModule__req_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40ERKNS3_8Vector3fES7_fjibi"]
        pub(super) fn req(
            module_accessor: *mut OpaqueAccessor,
            effect: Hash40,
            pos: *const Vector3f,
            rot: *const Vector3f,
            size: f32,
            arg6: u32,
            arg7: i32,
            arg8: bool,
            arg9: i32,
        ) -> u64;
        #[link_name = "\u{1}_ZN3app8lua_bind25EffectModule__req_2d_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40ERKNS3_8Vector3fES7_fj"]
        pub(super) fn req_2d(
            module_accessor: *mut OpaqueAccessor,
            effect: Hash40,
            pos: *const Vector3f,
            rot: *const Vector3f,
            size: f32,
            arg6: u32,
        ) -> u64;
        #[link_name = "\u{1}_ZN3app8lua_bind31EffectModule__req_on_joint_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40ES4_RKNS3_8Vector3fES7_fS7_S7_bjii"]
        pub(super) fn req_on_joint(
            module_accessor: *mut OpaqueAccessor,
            effect: Hash40,
            bone: Hash40,
            pos: *const Vector3f,
            rot: *const Vector3f,
            size: f32,
            arg7: *const Vector3f,
            arg8: *const Vector3f,
            arg9: bool,
            arg10: u32,
            arg11: i32,
            arg12: i32,
        ) -> u64;
        #[link_name = "\u{1}_ZN3app8lua_bind29EffectModule__req_follow_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40ES4_RKNS3_8Vector3fES7_fbjiiiibb"]
        pub(super) fn req_follow(
            module_accessor: *mut OpaqueAccessor,
            effect: Hash40,
            bone: Hash40,
            pos: *const Vector3f,
            rot: *const Vector3f,
            size: f32,
            arg7: bool,
            arg8: u32,
            arg9: i32,
            arg10: i32,
            arg11: i32,
            arg12: i32,
            arg13: bool,
            arg14: bool,
        ) -> u64;
        #[link_name = "\u{1}_ZN3app8lua_bind32EffectModule__req_continual_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40ES4_fji"]
        pub(super) fn req_continual(
            module_accessor: *mut OpaqueAccessor,
            effect: Hash40,
            arg3: Hash40,
            size: f32,
            arg5: u32,
            arg6: i32,
        ) -> u64;
        #[link_name = "\u{1}_ZN3app8lua_bind29EffectModule__req_common_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Ef"]
        pub(super) fn req_common(
            module_accessor: *mut OpaqueAccessor,
            arg2: Hash40,
            arg3: f32,
        ) -> u64;
        #[link_name = "\u{1}_ZN3app8lua_bind29EffectModule__req_screen_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Ebbb"]
        pub(super) fn req_screen(
            module_accessor: *mut OpaqueAccessor,
            arg2: Hash40,
            arg3: bool,
            arg4: bool,
            arg5: bool,
        ) -> u64;
        #[link_name = "\u{1}_ZN3app8lua_bind36EffectModule__req_screen_system_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Ebb"]
        pub(super) fn req_screen_system(
            module_accessor: *mut OpaqueAccessor,
            arg2: Hash40,
            arg3: bool,
            arg4: bool,
        ) -> u64;
        #[link_name = "\u{1}_ZN3app8lua_bind28EffectModule__kill_kind_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Ebb"]
        pub(super) fn kill_kind(
            module_accessor: *mut OpaqueAccessor,
            effect: Hash40,
            arg3: bool,
            arg4: bool,
        ) -> u64;
        #[link_name = "\u{1}_ZN3app8lua_bind30EffectModule__detach_kind_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Ei"]
        pub(super) fn detach_kind(
            module_accessor: *mut OpaqueAccessor,
            effect: Hash40,
            arg3: i32,
        ) -> u64;
    }
}

pub(super) fn log_variant_cleanup(original: u64, alias: u64) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNT: AtomicU64 = AtomicU64::new(0);
    let count = COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    if count == 1 || count.is_multiple_of(128) {
        record(Code::FxVariantCleanup, original, alias);
    }
}

pub(super) fn log_cleanup_route(layer: u64, nested: bool) {
    use std::sync::atomic::AtomicU64;
    static COUNTS: [AtomicU64; crate::eff_cleanup::ROUTE_SLOTS] = [
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
    ];
    if crate::eff_cleanup::take_route_sample(&COUNTS, layer, nested) {
        record(Code::FxCleanupRoute, layer, u64::from(nested));
    }
}

pub(super) unsafe fn invoke_kill_kind(
    accessor: *mut std::ffi::c_void,
    hash: u64,
    arg3: bool,
    arg4: bool,
) -> u64 {
    abi::kill_kind(accessor.cast(), Hash40 { hash }, arg3, arg4)
}

pub(super) unsafe fn invoke_detach_kind(
    accessor: *mut std::ffi::c_void,
    hash: u64,
    arg3: i32,
) -> u64 {
    abi::detach_kind(accessor.cast(), Hash40 { hash }, arg3)
}

#[skyline::hook(replace = abi::req)]
unsafe fn deluxe_rewrite_filter_req(
    module_accessor: *mut OpaqueAccessor,
    effect: Hash40,
    pos: *const Vector3f,
    rot: *const Vector3f,
    size: f32,
    arg6: u32,
    arg7: i32,
    arg8: bool,
    arg9: i32,
) -> u64 {
    let effect = Hash40 {
        hash: policy_hash(module_accessor, effect.hash),
    };
    call_original!(
        module_accessor,
        effect,
        pos,
        rot,
        size,
        arg6,
        arg7,
        arg8,
        arg9
    )
}

#[skyline::hook(replace = abi::req_2d)]
unsafe fn deluxe_rewrite_filter_req_2d(
    module_accessor: *mut OpaqueAccessor,
    effect: Hash40,
    pos: *const Vector3f,
    rot: *const Vector3f,
    size: f32,
    arg6: u32,
) -> u64 {
    let effect = Hash40 {
        hash: policy_hash(module_accessor, effect.hash),
    };
    call_original!(module_accessor, effect, pos, rot, size, arg6)
}

#[skyline::hook(replace = abi::req_on_joint)]
unsafe fn deluxe_rewrite_filter_req_on_joint(
    module_accessor: *mut OpaqueAccessor,
    effect: Hash40,
    bone: Hash40,
    pos: *const Vector3f,
    rot: *const Vector3f,
    size: f32,
    arg7: *const Vector3f,
    arg8: *const Vector3f,
    arg9: bool,
    arg10: u32,
    arg11: i32,
    arg12: i32,
) -> u64 {
    let effect = Hash40 {
        hash: policy_hash(module_accessor, effect.hash),
    };
    call_original!(
        module_accessor,
        effect,
        bone,
        pos,
        rot,
        size,
        arg7,
        arg8,
        arg9,
        arg10,
        arg11,
        arg12
    )
}

#[skyline::hook(replace = abi::req_follow)]
unsafe fn deluxe_rewrite_filter_req_follow(
    module_accessor: *mut OpaqueAccessor,
    effect: Hash40,
    bone: Hash40,
    pos: *const Vector3f,
    rot: *const Vector3f,
    size: f32,
    arg7: bool,
    arg8: u32,
    arg9: i32,
    arg10: i32,
    arg11: i32,
    arg12: i32,
    arg13: bool,
    arg14: bool,
) -> u64 {
    let effect = Hash40 {
        hash: policy_hash(module_accessor, effect.hash),
    };
    call_original!(
        module_accessor,
        effect,
        bone,
        pos,
        rot,
        size,
        arg7,
        arg8,
        arg9,
        arg10,
        arg11,
        arg12,
        arg13,
        arg14
    )
}

#[skyline::hook(replace = abi::req_continual)]
unsafe fn deluxe_rewrite_filter_req_continual(
    module_accessor: *mut OpaqueAccessor,
    effect: Hash40,
    arg3: Hash40,
    size: f32,
    arg5: u32,
    arg6: i32,
) -> u64 {
    let effect = Hash40 {
        hash: policy_hash(module_accessor, effect.hash),
    };
    call_original!(module_accessor, effect, arg3, size, arg5, arg6)
}

#[skyline::hook(replace = abi::req_common)]
unsafe fn deluxe_rewrite_filter_req_common(
    module_accessor: *mut OpaqueAccessor,
    arg2: Hash40,
    arg3: f32,
) -> u64 {
    let arg2 = Hash40 {
        hash: policy_hash(module_accessor, arg2.hash),
    };
    call_original!(module_accessor, arg2, arg3)
}

#[skyline::hook(replace = abi::req_screen)]
unsafe fn deluxe_rewrite_filter_req_screen(
    module_accessor: *mut OpaqueAccessor,
    arg2: Hash40,
    arg3: bool,
    arg4: bool,
    arg5: bool,
) -> u64 {
    let mode = super::fx_mode();
    let scope = super::fx_scope_allows();
    if note_critical(arg2.hash, crate::critical_probe::PATH_SCREEN, mode, scope) {
        return 0;
    }
    if skip_screen(arg2.hash) {
        return 0;
    }
    call_original!(module_accessor, arg2, arg3, arg4, arg5)
}

#[skyline::hook(replace = abi::req_screen_system)]
unsafe fn deluxe_rewrite_filter_req_screen_system(
    module_accessor: *mut OpaqueAccessor,
    arg2: Hash40,
    arg3: bool,
    arg4: bool,
) -> u64 {
    let mode = super::fx_mode();
    let scope = super::fx_scope_allows();
    if note_critical(arg2.hash, crate::critical_probe::PATH_SYSTEM, mode, scope) {
        return 0;
    }
    if skip_screen(arg2.hash) {
        return 0;
    }
    call_original!(module_accessor, arg2, arg3, arg4)
}

#[skyline::hook(replace = abi::kill_kind)]
unsafe fn deluxe_cleanup_kill_kind(
    module_accessor: *mut OpaqueAccessor,
    effect: Hash40,
    arg3: bool,
    arg4: bool,
) -> u64 {
    let accessor = module_accessor as usize;
    if let Some(alias) = crate::eff_variant::cleanup_alias(effect.hash) {
        if crate::eff_cleanup::delivered_matches(
            accessor,
            effect.hash,
            crate::eff_cleanup::CLEANUP_KILL,
        ) {
            log_cleanup_route(1, true);
        } else if let Some(_guard) = crate::eff_cleanup::begin_alias(
            accessor,
            effect.hash,
            crate::eff_cleanup::CLEANUP_KILL,
            Some(alias),
        ) {
            let _ignored = call_original!(module_accessor, Hash40 { hash: alias }, arg3, arg4);
            log_variant_cleanup(effect.hash, alias);
            log_cleanup_route(1, false);
            return call_original!(module_accessor, effect, arg3, arg4);
        }
    }
    call_original!(module_accessor, effect, arg3, arg4)
}

#[skyline::hook(replace = abi::detach_kind)]
unsafe fn deluxe_cleanup_detach_kind(
    module_accessor: *mut OpaqueAccessor,
    effect: Hash40,
    arg3: i32,
) -> u64 {
    let accessor = module_accessor as usize;
    if let Some(alias) = crate::eff_variant::cleanup_alias(effect.hash) {
        if crate::eff_cleanup::delivered_matches(
            accessor,
            effect.hash,
            crate::eff_cleanup::CLEANUP_DETACH,
        ) {
            log_cleanup_route(2, true);
        } else if let Some(_guard) = crate::eff_cleanup::begin_alias(
            accessor,
            effect.hash,
            crate::eff_cleanup::CLEANUP_DETACH,
            Some(alias),
        ) {
            let _ignored = call_original!(module_accessor, Hash40 { hash: alias }, arg3);
            log_variant_cleanup(effect.hash, alias);
            log_cleanup_route(2, false);
            return call_original!(module_accessor, effect, arg3);
        }
    }
    call_original!(module_accessor, effect, arg3)
}

pub(super) fn install() {
    latch_critical_probe();
    super::variant::install();
    super::online::install();
    super::acmd::install();
    macro_rules! install {
        ($id:expr, $target:path, $hook:ident) => {
            if log_entry($id, $target as *const ()) {
                skyline::install_hook!($hook);
                record(Code::HookInstalled, $id, $target as *const () as u64);
            }
        };
    }
    install!(256, abi::req, deluxe_rewrite_filter_req);
    install!(257, abi::req_2d, deluxe_rewrite_filter_req_2d);
    install!(258, abi::req_on_joint, deluxe_rewrite_filter_req_on_joint);
    install!(259, abi::req_follow, deluxe_rewrite_filter_req_follow);
    install!(260, abi::req_continual, deluxe_rewrite_filter_req_continual);
    install!(261, abi::req_common, deluxe_rewrite_filter_req_common);
    install!(262, abi::req_screen, deluxe_rewrite_filter_req_screen);
    install!(
        263,
        abi::req_screen_system,
        deluxe_rewrite_filter_req_screen_system
    );
    install!(264, abi::kill_kind, deluxe_cleanup_kill_kind);
    install!(265, abi::detach_kind, deluxe_cleanup_detach_kind);
}

fn log_entry(id: u64, address: *const ()) -> bool {
    if !unsafe { super::memory::accessible(address as usize, 4, false) } {
        record(Code::HookRejected, id, address as u64);
        return false;
    }
    record(Code::HookInstalling, id, address as u64);
    let word = unsafe { *(address as *const u32) };
    record(Code::HookEntryWord, id, word as u64);
    super::logging::install_checkpoint(Code::HookEntryWord, id, word as u64);
    true
}
