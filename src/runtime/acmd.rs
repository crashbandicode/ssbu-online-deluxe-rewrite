//! ACMD has its own request path; lua_bind::EffectModule hooks do not cover it.
//! `EFFECT_OFF_KIND` and `EFFECT_DETACH_KIND` are void in skyline-smash
//! bed642e. Their hooks read the hash and flags, call the lua_bind cleanup for
//! the clone, then call the original command. They do not rewrite the Lua
//! stack and do not invent a return value.
//! Mirrors the supplied FX mod's stack bridge, with bounded primitive storage,
//! no per-frame allocation and an unchanged original call after substitution.
use super::{fx_mode, fx_scope_allows, record, Code};
use crate::{acmd::Value, config::FxMode, effects_reduced::FxIntent};
use std::sync::atomic::{AtomicU64, Ordering};

#[repr(C)]
struct Agent {
    // skyline-smash bed642e L2CAgent (cpp.rs), 0x48 bytes, alignment 8.
    storage: [u64; 9],
}

extern "C" {
    #[link_name = "\u{1}_ZN3lib8L2CAgentC2EP9lua_State"]
    fn agent_construct(agent: *mut Agent, lua_state: u64) -> *mut Agent;
    #[link_name = "\u{1}_ZN3lib8L2CAgent13pop_lua_stackEi"]
    fn stack_value(agent: *mut Agent, index: i32) -> Value;
    #[link_name = "\u{1}_ZN3lib8L2CAgent14push_lua_stackERKNS_8L2CValueE"]
    fn stack_push(agent: *mut Agent, value: *const Value) -> u64;
    #[link_name = "\u{1}_ZN3lib8L2CAgent15clear_lua_stackEv"]
    fn stack_clear(agent: *mut Agent) -> u64;
}

static REJECTED: AtomicU64 = AtomicU64::new(0);

fn reject(path: u64, reason: u64) {
    let count = REJECTED.fetch_add(1, Ordering::Relaxed);
    if count < 8 {
        record(Code::AcmdStackRejected, path, reason);
    }
}

unsafe fn filter(lua_state: u64, path: u64, calls: &AtomicU64, hash_count: usize) {
    let mode = fx_mode();
    let scope = fx_scope_allows();
    if lua_state == 0 || mode == FxMode::On || (mode == FxMode::Off && !scope) {
        return;
    }
    let count = calls.fetch_add(1, Ordering::Relaxed) + 1;
    let mut agent = Agent { storage: [0; 9] };
    agent_construct(&mut agent, lua_state);
    let mut args = [Value::VOID; 32];
    for (index, arg) in args.iter_mut().take(hash_count).enumerate() {
        *arg = stack_value(&mut agent, (index + 1) as i32);
        if !matches!(arg.kind, 2 | 7) {
            reject(path, arg.kind as u64);
            return;
        }
    }
    if count == 1 || count.is_multiple_of(256) {
        record(Code::AcmdEffectSeen, path, args[0].raw);
    }
    let context = if args[..hash_count].iter().any(|v| match mode {
        FxMode::Off => {
            crate::effects::rewrite(FxMode::Off, v.raw) == v.raw
                && crate::effects::needs_context(v.raw)
        }
        FxMode::Reduced => {
            crate::effects_reduced::reduced_named(v.raw) || crate::eff_variant::wants_context(v.raw)
        }
        FxMode::On => false,
    }) {
        super::effect_context::from_lua(lua_state)
    } else {
        None
    };
    let mut intents = [FxIntent::Passthrough; 2];
    let mut forwarded = [0u64; 2];
    let mut variant = [false; 2];
    let mut changed = false;
    for index in 0..hash_count {
        intents[index] = crate::effects_reduced::classify(mode, scope, args[index].raw, context);
        let requested = crate::effects_reduced::request_hash(intents[index], args[index].raw);
        let next = crate::eff_variant::forward(mode, scope, requested, context);
        forwarded[index] = next;
        variant[index] = next != requested;
        if next != args[index].raw {
            changed = true;
        }
    }
    // Reads don't clear Lua. A rejected tail leaves the original request untouched.
    let mut len = hash_count;
    if changed && scope {
        loop {
            let value = stack_value(&mut agent, (len + 1) as i32);
            if value.kind == 0 {
                break;
            }
            if len == args.len() || !value.is_primitive() {
                reject(path, value.kind as u64);
                return;
            }
            args[len] = value;
            len += 1;
        }
    }
    for index in 0..hash_count {
        if variant[index] {
            let swaps = crate::eff_variant::observe_swap(args[index].raw);
            if swaps == 1 || swaps.is_multiple_of(128) || super::api_trace::start() != 0 {
                record(
                    Code::FxVariantUse,
                    crate::eff_variant::source_file(args[index].raw),
                    args[index].raw,
                );
            }
        }
    }
    let originals = [args[0].raw, args[1].raw];
    for index in 0..hash_count {
        let intent = intents[index];
        let next = forwarded[index];
        let code = match intent {
            FxIntent::Passthrough => {
                if changed && scope {
                    args[index].raw = next;
                }
                continue;
            }
            FxIntent::Suppress => Code::AcmdEffectSuppressed,
            FxIntent::ReducedCull => Code::FxReducedCull,
            FxIntent::ReducedKeep => Code::FxReducedKeep,
            FxIntent::ReducedRestore => Code::FxReducedRestore,
            FxIntent::ReducedPreserve => Code::FxReducedPreserve,
        };
        let total = crate::effects_reduced::observe(intent, false);
        let sample = match intent {
            FxIntent::Suppress => total <= 32 || total.is_multiple_of(128),
            _ => total == 1 || total.is_multiple_of(128),
        };
        if sample || super::api_trace::start() != 0 {
            record(code, originals[index], (path << 32) | total);
        }
        if changed && scope {
            args[index].raw = next;
        }
    }
    if changed && scope {
        stack_clear(&mut agent);
        for value in &args[..len] {
            stack_push(&mut agent, value);
        }
    }
}

macro_rules! effect_hook {
    ($target:ident, $hook:ident, $symbol:literal, $id:expr) => {
        effect_hook!($target, $hook, $symbol, $id, 1);
    };
    ($target:ident, $hook:ident, $symbol:literal, $id:expr, $arity:expr) => {
        extern "C" {
            #[link_name = $symbol]
            fn $target(lua_state: u64);
        }
        #[skyline::hook(replace = $target)]
        unsafe fn $hook(lua_state: u64) {
            static CALLS: AtomicU64 = AtomicU64::new(0);
            filter(lua_state, $id, &CALLS, $arity);
            call_original!(lua_state);
        }
    };
}

// Verified vanilla Palutena Explode and Edge Burst/Flare scripts use EFFECT
// and EFFECT_FOLLOW. Alpha/no-stop/position variants cover the existing policy.
effect_hook!(
    effect,
    deluxe_rewrite_filter_effect,
    "\u{1}_ZN3app10sv_animcmd6EFFECTEP9lua_State",
    0x110
);
effect_hook!(
    follow,
    deluxe_rewrite_filter_follow,
    "\u{1}_ZN3app10sv_animcmd13EFFECT_FOLLOWEP9lua_State",
    0x111
);
effect_hook!(
    alpha,
    deluxe_rewrite_filter_alpha,
    "\u{1}_ZN3app10sv_animcmd19EFFECT_FOLLOW_ALPHAEP9lua_State",
    0x112
);
effect_hook!(
    no_stop,
    deluxe_rewrite_filter_no_stop,
    "\u{1}_ZN3app10sv_animcmd21EFFECT_FOLLOW_NO_STOPEP9lua_State",
    0x113
);
effect_hook!(
    pos,
    deluxe_rewrite_filter_pos,
    "\u{1}_ZN3app10sv_animcmd14EFFECT_FLW_POSEP9lua_State",
    0x114
);
effect_hook!(
    pos_no_stop,
    deluxe_rewrite_filter_pos_no_stop,
    "\u{1}_ZN3app10sv_animcmd22EFFECT_FLW_POS_NO_STOPEP9lua_State",
    0x115
);

effect_hook!(
    landing,
    deluxe_rewrite_filter_landing,
    "\u{1}_ZN3app10sv_animcmd14LANDING_EFFECTEP9lua_State",
    0x116
);
effect_hook!(
    flip,
    deluxe_rewrite_filter_flip,
    "\u{1}_ZN3app10sv_animcmd18EFFECT_FOLLOW_FLIPEP9lua_State",
    0x117,
    2
);
effect_hook!(
    flip_alpha,
    deluxe_rewrite_filter_flip_alpha,
    "\u{1}_ZN3app10sv_animcmd24EFFECT_FOLLOW_FLIP_ALPHAEP9lua_State",
    0x118,
    2
);
effect_hook!(
    flip_no_stop,
    deluxe_rewrite_filter_flip_no_stop,
    "\u{1}_ZN3app10sv_animcmd26EFFECT_FOLLOW_NO_STOP_FLIPEP9lua_State",
    0x119,
    2
);

extern "C" {
    #[link_name = "\u{1}_ZN3app10sv_animcmd15EFFECT_OFF_KINDEP9lua_State"]
    fn effect_off_kind(lua_state: u64);
    #[link_name = "\u{1}_ZN3app10sv_animcmd18EFFECT_DETACH_KINDEP9lua_State"]
    fn effect_detach_kind(lua_state: u64);
}

struct OffFlags {
    hash: u64,
    second: bool,
    third: bool,
}

struct DetachFlags {
    hash: u64,
    arg: i32,
}

unsafe fn read_off_flags(lua_state: u64) -> Option<OffFlags> {
    if lua_state == 0 {
        return None;
    }
    let mut agent = Agent { storage: [0; 9] };
    agent_construct(&mut agent, lua_state);
    let hash = stack_value(&mut agent, 1);
    let second = stack_value(&mut agent, 2);
    let third = stack_value(&mut agent, 3);
    if !matches!(hash.kind, 2 | 7) || second.kind != 1 || third.kind != 1 {
        return None;
    }
    Some(OffFlags {
        hash: hash.raw,
        second: second.raw != 0,
        third: third.raw != 0,
    })
}

unsafe fn read_detach_flags(lua_state: u64) -> Option<DetachFlags> {
    if lua_state == 0 {
        return None;
    }
    let mut agent = Agent { storage: [0; 9] };
    agent_construct(&mut agent, lua_state);
    let hash = stack_value(&mut agent, 1);
    let arg = stack_value(&mut agent, 2);
    if !matches!(hash.kind, 2 | 7) || arg.kind != 2 {
        return None;
    }
    Some(DetachFlags {
        hash: hash.raw,
        arg: arg.raw as i32,
    })
}

#[skyline::hook(replace = effect_off_kind)]
unsafe fn deluxe_acmd_cleanup_off_kind(lua_state: u64) {
    if let Some(saved) = read_off_flags(lua_state) {
        if let Some(alias) = crate::eff_variant::cleanup_alias(saved.hash) {
            let accessor = super::effect_context::module_accessor(lua_state);
            if let Some(_guard) = crate::eff_cleanup::begin_alias(
                accessor as usize,
                saved.hash,
                crate::eff_cleanup::CLEANUP_KILL,
                Some(alias),
            ) {
                let _ignored =
                    super::effects::invoke_kill_kind(accessor, alias, saved.second, saved.third);
                super::effects::log_variant_cleanup(saved.hash, alias);
                super::effects::log_cleanup_route(3, false);
                call_original!(lua_state);
                return;
            }
        }
    }
    call_original!(lua_state);
}

#[skyline::hook(replace = effect_detach_kind)]
unsafe fn deluxe_acmd_cleanup_detach_kind(lua_state: u64) {
    if let Some(saved) = read_detach_flags(lua_state) {
        if let Some(alias) = crate::eff_variant::cleanup_alias(saved.hash) {
            let accessor = super::effect_context::module_accessor(lua_state);
            if let Some(_guard) = crate::eff_cleanup::begin_alias(
                accessor as usize,
                saved.hash,
                crate::eff_cleanup::CLEANUP_DETACH,
                Some(alias),
            ) {
                let _ignored = super::effects::invoke_detach_kind(accessor, alias, saved.arg);
                super::effects::log_variant_cleanup(saved.hash, alias);
                super::effects::log_cleanup_route(4, false);
                call_original!(lua_state);
                return;
            }
        }
    }
    call_original!(lua_state);
}

pub(super) fn install() {
    macro_rules! install {
        ($target:ident, $hook:ident, $id:expr) => {
            let address = $target as *const () as u64;
            if !unsafe { super::memory::accessible(address as usize, 4, false) } {
                record(Code::HookRejected, $id, address);
            } else {
                record(Code::HookInstalling, $id, address);
                let word = unsafe { *(address as *const u32) } as u64;
                record(Code::HookEntryWord, $id, word);
                super::logging::install_checkpoint(Code::HookEntryWord, $id, word);
                skyline::install_hook!($hook);
                record(Code::HookInstalled, $id, address);
            }
        };
    }
    install!(effect, deluxe_rewrite_filter_effect, 0x110);
    install!(follow, deluxe_rewrite_filter_follow, 0x111);
    install!(alpha, deluxe_rewrite_filter_alpha, 0x112);
    install!(no_stop, deluxe_rewrite_filter_no_stop, 0x113);
    install!(pos, deluxe_rewrite_filter_pos, 0x114);
    install!(pos_no_stop, deluxe_rewrite_filter_pos_no_stop, 0x115);
    install!(landing, deluxe_rewrite_filter_landing, 0x116);
    install!(flip, deluxe_rewrite_filter_flip, 0x117);
    install!(flip_alpha, deluxe_rewrite_filter_flip_alpha, 0x118);
    install!(flip_no_stop, deluxe_rewrite_filter_flip_no_stop, 0x119);
    install!(effect_off_kind, deluxe_acmd_cleanup_off_kind, 0x11a);
    install!(effect_detach_kind, deluxe_acmd_cleanup_detach_kind, 0x11b);
}
