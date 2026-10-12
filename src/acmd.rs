//! Primitive effect-command arguments. Layout follows skyline-smash bed642e.
use crate::config::FxMode;

/// The game returns L2CValue through x8. The extra storage makes this 24 bytes,
/// selecting that ABI; it is MaybeUninit because the game writes only 16 bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Value {
    pub kind: u32,
    pub metadata: u32,
    pub raw: u64,
    pub indirect_return_padding: std::mem::MaybeUninit<u64>,
}

impl Value {
    pub const VOID: Self = Self {
        kind: 0,
        metadata: 0,
        raw: 0,
        indirect_return_padding: std::mem::MaybeUninit::uninit(),
    };

    pub fn is_primitive(&self) -> bool {
        matches!(self.kind, 1 | 2 | 3 | 7)
    }
}

/// Ordinary commands carry a single effect hash at argument one.
/// Refuse nonprimitive arguments before the runtime clears the original stack.
pub fn replacement(mode: FxMode, args: &[Value]) -> Option<u64> {
    replacements(mode, args, 1, None)[0]
}

/// Flip commands take two effect names; ordinary commands take one. Never
/// rewrite their joint name, scale or other arguments.
pub fn replacements(
    mode: FxMode,
    args: &[Value],
    hash_count: usize,
    context: Option<crate::effect_context::Context>,
) -> [Option<u64>; 2] {
    let mut result = [None; 2];
    if !matches!(mode, FxMode::Off | FxMode::Reduced)
        || !(1..=2).contains(&hash_count)
        || args.len() < hash_count
        || !args.iter().all(Value::is_primitive)
    {
        return result;
    }
    for index in 0..hash_count {
        if matches!(args[index].kind, 2 | 7) {
            let intent = crate::effects_reduced::classify(mode, true, args[index].raw, context);
            let hash = crate::effects_reduced::request_hash(intent, args[index].raw);
            let hash = crate::eff_variant::forward(mode, true, hash, context);
            result[index] = (hash != args[index].raw).then_some(hash);
        }
    }
    result
}

const _: () = assert!(std::mem::size_of::<Value>() == 24);
const _: () = assert!(std::mem::align_of::<Value>() == 8);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects;

    #[test]
    fn vanilla_explosion_command_changes_only_the_effect_name() {
        // Vanilla Palutena Explode: EFFECT(bomb, top, xyz, rotation, scale,
        // six random offsets, true). A wrong 16-byte ABI or hash index fails here.
        assert_eq!(std::mem::offset_of!(Value, raw), 8);
        let mut args = [Value {
            kind: 2,
            ..Value::VOID
        }; 16];
        args[0] = Value {
            kind: 7,
            raw: effects::hash40("palutena_bomb"),
            ..Value::VOID
        };
        args[1] = Value {
            kind: 7,
            raw: effects::hash40("top"),
            ..Value::VOID
        };
        args[8] = Value {
            kind: 3,
            raw: 1.0f32.to_bits() as u64,
            ..Value::VOID
        };
        args[15] = Value {
            kind: 1,
            raw: 1,
            ..Value::VOID
        };
        let before: Vec<_> = args.iter().map(|v| (v.kind, v.metadata, v.raw)).collect();
        assert_eq!(replacement(FxMode::Off, &args), Some(effects::NULL_EFFECT));
        assert_eq!(replacement(FxMode::On, &args), None);
        assert_eq!(replacement(FxMode::Reduced, &args), None);
        let side = crate::effect_context::Context {
            category: 0,
            kind: 0x36,
            motion: effects::hash40("special_s"),
            status: 0x1dd,
        };
        args[0].raw = effects::hash40("palutena_explosiveflame");
        assert_eq!(
            replacement(FxMode::Reduced, &args),
            None,
            "the explosiveflame string is not a script request"
        );
        assert_eq!(
            replacements(FxMode::Reduced, &args, 1, Some(side)),
            [None, None]
        );
        for name in ["palutena_wand_light_trace", "palutena_bomb_appear"] {
            args[0].raw = effects::hash40(name);
            assert_eq!(
                replacements(FxMode::Reduced, &args, 1, None),
                [None, None],
                "{name} stays until the side-B move matches"
            );
            assert_eq!(
                replacements(FxMode::Reduced, &args, 1, Some(side)),
                [Some(effects::NULL_EFFECT), None],
                "{name}"
            );
        }
        args[0].raw = effects::hash40("palutena_bomb");
        assert_eq!(
            replacements(FxMode::Reduced, &args, 1, Some(side)),
            [None, None]
        );
        assert_eq!(
            before,
            args.iter()
                .map(|v| (v.kind, v.metadata, v.raw))
                .collect::<Vec<_>>()
        );
        args[0].raw = effects::hash40("sys_hit_normal");
        assert_eq!(replacement(FxMode::Off, &args), None);
        args[0].raw = effects::hash40("edge_shadowflare_bullet_hit");
        assert_eq!(replacement(FxMode::Off, &args), Some(effects::NULL_EFFECT));
        args[0].kind = 3; // Same bits in a numeric argument are not a hash.
        assert_eq!(replacement(FxMode::Off, &args), None);
        args[0].kind = 7;
        args[9].kind = 5; // Never rebuild tables or other owned Lua values.
        assert_eq!(replacement(FxMode::Off, &args), None);
        assert_eq!(replacement(FxMode::Off, &[]), None);
    }
    #[test]
    fn flip_filters_both_effect_names_independently_and_never_the_bone() {
        let value = |name| Value {
            kind: 7,
            raw: effects::hash40(name),
            ..Value::VOID
        };
        let targeted = value("palutena_bomb");
        let safe = value("sys_hit_normal");
        // Deliberately use a policy name as the third (joint) argument.
        let args = [targeted, targeted, targeted];
        let null = Some(effects::NULL_EFFECT);
        assert_eq!(replacements(FxMode::Off, &args, 2, None), [null, null]);
        assert_eq!(replacements(FxMode::On, &args, 2, None), [None, None]);
        assert_eq!(replacements(FxMode::Reduced, &args, 2, None), [None, None]);
        assert_eq!(
            replacements(FxMode::Off, &[safe, targeted, targeted], 2, None),
            [None, null]
        );
        assert_eq!(
            replacements(FxMode::Off, &[targeted, safe, targeted], 2, None),
            [null, None]
        );
        assert_eq!(
            replacements(FxMode::Off, &[safe, safe, targeted], 2, None),
            [None, None]
        );
        assert_eq!(
            replacements(FxMode::Off, &[targeted], 2, None),
            [None, None]
        );
        assert_eq!(replacements(FxMode::Off, &args, 3, None), [None, None]);
        let owned = Value {
            kind: 5,
            ..Value::VOID
        };
        assert_eq!(
            replacements(FxMode::Off, &[targeted, safe, owned], 2, None),
            [None, None]
        );
    }

    #[test]
    fn shared_flash_is_filtered_only_in_its_qualified_move() {
        use crate::effect_context::Context;
        let args = [Value {
            kind: 7,
            raw: effects::hash40("sys_smash_flash"),
            ..Value::VOID
        }];
        let smash = Context {
            category: 0,
            kind: 0x59,
            motion: effects::hash40("attack_s4"),
            status: 0x2f,
        };
        assert_eq!(
            replacements(FxMode::Off, &args, 1, Some(smash)),
            [Some(effects::NULL_EFFECT), None]
        );
        for context in [
            None,
            Some(Context { kind: 0, ..smash }),
            Some(Context {
                category: 1,
                ..smash
            }),
            Some(Context {
                motion: effects::hash40("wait"),
                status: 0,
                ..smash
            }),
        ] {
            assert_eq!(replacements(FxMode::Off, &args, 1, context), [None, None]);
        }
    }
}
