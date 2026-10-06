//! Narrow C ABI surface transcribed from skyline-smash bed642e/src/cpp.rs.
//! Opaque pointers are forwarded untouched. Eight native function hooks chain
//! through original!(), including Training Modpack's shared EffectModule hooks.
#![allow(clippy::too_many_arguments)] // Fixed native C++ ABI.
use super::{effects_off, record, Code};
use crate::{config::FxMode, effects};
use std::sync::atomic::{AtomicU64, Ordering};

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

static SUPPRESSED: AtomicU64 = AtomicU64::new(0);
fn note(hash: u64) {
    let count = SUPPRESSED.fetch_add(1, Ordering::Relaxed) + 1;
    if count == 1 || count.is_multiple_of(128) {
        record(Code::FxSuppressed, hash, count);
    }
}

// Resolve actor context only for scoped names, before forwarding the original
// request. On takes the existing fast path without any actor reads.
unsafe fn rewrite(accessor: *mut OpaqueAccessor, hash: u64) -> u64 {
    let global = effects::rewrite(FxMode::Off, hash);
    if global != hash || !effects::needs_context(hash) {
        return global;
    }
    effects::rewrite_with_context(
        FxMode::Off,
        hash,
        super::effect_context::from_accessor(accessor.cast()),
    )
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
    }
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
    let effect = if effects_off() {
        let hash = rewrite(module_accessor, effect.hash);
        if hash != effect.hash {
            note(effect.hash);
        }
        Hash40 { hash }
    } else {
        effect
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
    let effect = if effects_off() {
        let hash = rewrite(module_accessor, effect.hash);
        if hash != effect.hash {
            note(effect.hash);
        }
        Hash40 { hash }
    } else {
        effect
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
    let effect = if effects_off() {
        let hash = rewrite(module_accessor, effect.hash);
        if hash != effect.hash {
            note(effect.hash);
        }
        Hash40 { hash }
    } else {
        effect
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
    let effect = if effects_off() {
        let hash = rewrite(module_accessor, effect.hash);
        if hash != effect.hash {
            note(effect.hash);
        }
        Hash40 { hash }
    } else {
        effect
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
    let effect = if effects_off() {
        let hash = rewrite(module_accessor, effect.hash);
        if hash != effect.hash {
            note(effect.hash);
        }
        Hash40 { hash }
    } else {
        effect
    };
    call_original!(module_accessor, effect, arg3, size, arg5, arg6)
}

#[skyline::hook(replace = abi::req_common)]
unsafe fn deluxe_rewrite_filter_req_common(module_accessor: *mut OpaqueAccessor, arg2: Hash40, arg3: f32) -> u64 {
    let arg2 = if effects_off() {
        let hash = rewrite(module_accessor, arg2.hash);
        if hash != arg2.hash {
            note(arg2.hash);
        }
        Hash40 { hash }
    } else {
        arg2
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
    if effects_off() && effects::suppress_screen(FxMode::Off, arg2.hash) {
        note(arg2.hash);
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
    if effects_off() && effects::suppress_screen(FxMode::Off, arg2.hash) {
        note(arg2.hash);
        return 0;
    }
    call_original!(module_accessor, arg2, arg3, arg4)
}

pub(super) fn install() {
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
    install!(263, abi::req_screen_system, deluxe_rewrite_filter_req_screen_system);
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
