//! ACMD has its own request path; lua_bind::EffectModule hooks do not cover it.
//! Mirrors the supplied FX mod's stack bridge, with bounded primitive storage,
//! no per-frame allocation and an unchanged original call after substitution.
use super::{effects_off, record, Code};
use crate::{acmd::Value, config::FxMode};
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

static SUPPRESSED: AtomicU64 = AtomicU64::new(0);
static REJECTED: AtomicU64 = AtomicU64::new(0);

fn reject(path: u64, reason: u64) {
    let count = REJECTED.fetch_add(1, Ordering::Relaxed);
    if count < 8 {
        record(Code::AcmdStackRejected, path, reason);
    }
}

unsafe fn filter(lua_state: u64, path: u64, calls: &AtomicU64, hash_count: usize) {
    if !effects_off() || lua_state == 0 {
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
    if count == 1 || count % 256 == 0 {
        record(Code::AcmdEffectSeen, path, args[0].raw);
    }
    let context = if args[..hash_count].iter().any(|v| {
        crate::effects::rewrite(FxMode::Off, v.raw) == v.raw && crate::effects::needs_context(v.raw)
    }) {
        super::effect_context::from_lua(lua_state)
    } else {
        None
    };
    if crate::acmd::replacements(FxMode::Off, &args[..hash_count], hash_count, context)
        .iter()
        .all(Option::is_none)
    {
        return;
    }
    // Reads don't clear Lua. Unknown/oversized commands pass through intact.
    let mut len = hash_count;
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
    let replacements = crate::acmd::replacements(FxMode::Off, &args[..len], hash_count, context);
    let old = [args[0].raw, args[1].raw];
    for (index, hash) in replacements.iter().enumerate() {
        if let Some(hash) = hash {
            args[index].raw = *hash;
        }
    }
    stack_clear(&mut agent);
    for value in &args[..len] {
        stack_push(&mut agent, value);
    }
    for index in 0..hash_count {
        if replacements[index].is_some() {
            let count = SUPPRESSED.fetch_add(1, Ordering::Relaxed) + 1;
            if count <= 32 || count % 128 == 0 {
                record(Code::AcmdEffectSuppressed, old[index], (path << 32) | count);
            }
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
    filter_effect,
    "\u{1}_ZN3app10sv_animcmd6EFFECTEP9lua_State",
    0x110
);
effect_hook!(
    follow,
    filter_follow,
    "\u{1}_ZN3app10sv_animcmd13EFFECT_FOLLOWEP9lua_State",
    0x111
);
effect_hook!(
    alpha,
    filter_alpha,
    "\u{1}_ZN3app10sv_animcmd19EFFECT_FOLLOW_ALPHAEP9lua_State",
    0x112
);
effect_hook!(
    no_stop,
    filter_no_stop,
    "\u{1}_ZN3app10sv_animcmd21EFFECT_FOLLOW_NO_STOPEP9lua_State",
    0x113
);
effect_hook!(
    pos,
    filter_pos,
    "\u{1}_ZN3app10sv_animcmd14EFFECT_FLW_POSEP9lua_State",
    0x114
);
effect_hook!(
    pos_no_stop,
    filter_pos_no_stop,
    "\u{1}_ZN3app10sv_animcmd22EFFECT_FLW_POS_NO_STOPEP9lua_State",
    0x115
);

effect_hook!(
    landing,
    filter_landing,
    "\u{1}_ZN3app10sv_animcmd14LANDING_EFFECTEP9lua_State",
    0x116
);
effect_hook!(
    flip,
    filter_flip,
    "\u{1}_ZN3app10sv_animcmd18EFFECT_FOLLOW_FLIPEP9lua_State",
    0x117,
    2
);
effect_hook!(
    flip_alpha,
    filter_flip_alpha,
    "\u{1}_ZN3app10sv_animcmd24EFFECT_FOLLOW_FLIP_ALPHAEP9lua_State",
    0x118,
    2
);
effect_hook!(
    flip_no_stop,
    filter_flip_no_stop,
    "\u{1}_ZN3app10sv_animcmd26EFFECT_FOLLOW_NO_STOP_FLIPEP9lua_State",
    0x119,
    2
);

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
    install!(effect, filter_effect, 0x110);
    install!(follow, filter_follow, 0x111);
    install!(alpha, filter_alpha, 0x112);
    install!(no_stop, filter_no_stop, 0x113);
    install!(pos, filter_pos, 0x114);
    install!(pos_no_stop, filter_pos_no_stop, 0x115);
    install!(landing, filter_landing, 0x116);
    install!(flip, filter_flip, 0x117);
    install!(flip_alpha, filter_flip_alpha, 0x118);
    install!(flip_no_stop, filter_flip_no_stop, 0x119);
}
