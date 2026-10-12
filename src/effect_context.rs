//! Object-category and move qualification for the peer's 19 character profiles.
//! Constants verified against skyline-smash bed642e lua_const.rs (Smash 13.0.5).
use crate::effects::hash40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Profile {
    KazuyaFSmash,
    FalconPunch,
    NessUpB,
    LucasUpB,
    IkeNeutralB,
    ChromNeutralB,
    RoyNeutralB,
    PalutenaRapidJab,
    PalutenaNeutralB,
    PalutenaUpB,
    PalutenaSideB,
    PalutenaUpSmash,
    PalutenaTilts,
    PalutenaWingSmash,
    PalutenaUpAir,
    EdgeFSmash,
    EdgePurpleDash,
    EdgeNeutralB,
    EdgeDownB,
    EdgeSideB,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub category: u8,
    pub kind: i32,
    pub motion: u64,
    pub status: i32,
}

struct Rule {
    profile: Profile,
    kind: i32,
    motions: &'static [u64],
    statuses: &'static [i32],
    require_both: bool,
}

const RULES: &[Rule] = &[
    Rule {
        profile: Profile::KazuyaFSmash,
        kind: 0x5c, // FIGHTER_KIND_DEMON
        motions: &[
            hash40("attack_s4"),
            hash40("attack_s4_s"),
            hash40("attack_s4_hold"),
            hash40("attack_s4_start"),
        ],
        statuses: &[0x2d, 0x2e, 0x2f],
        require_both: false,
    },
    Rule {
        profile: Profile::FalconPunch,
        kind: 0xb, // FIGHTER_KIND_CAPTAIN
        motions: &[
            hash40("special_n"),
            hash40("special_air_n"),
            hash40("special_n_turn"),
            hash40("special_air_n_turn"),
        ],
        statuses: &[0x1dc, 0x1e1],
        require_both: false,
    },
    Rule {
        profile: Profile::NessUpB,
        kind: 0xa, // FIGHTER_KIND_NESS
        motions: &[
            hash40("special_hi"),
            hash40("special_air_hi"),
            hash40("special_hi_hold"),
            hash40("special_air_hi_hold"),
            hash40("special_hi_attack"),
            hash40("special_air_hi_attack"),
            hash40("special_hi_end"),
            hash40("special_air_hi_end"),
        ],
        statuses: &[0x1de],
        require_both: false,
    },
    Rule {
        profile: Profile::LucasUpB,
        kind: 0x28, // FIGHTER_KIND_LUCAS
        motions: &[
            hash40("special_hi"),
            hash40("special_air_hi"),
            hash40("special_hi_hold"),
            hash40("special_air_hi_hold"),
            hash40("special_hi_attack"),
            hash40("special_air_hi_attack"),
            hash40("special_hi_end"),
            hash40("special_air_hi_end"),
        ],
        statuses: &[0x1de, 0x1e4, 0x1e5, 0x1e6, 0x1e7, 0x1e8],
        require_both: false,
    },
    Rule {
        profile: Profile::IkeNeutralB,
        kind: 0x23, // FIGHTER_KIND_IKE
        motions: &[
            hash40("special_n"),
            hash40("special_air_n"),
            hash40("special_n_start"),
            hash40("special_air_n_start"),
            hash40("special_n_loop"),
            hash40("special_air_n_loop"),
            hash40("special_n_hold"),
            hash40("special_air_n_hold"),
            hash40("special_n_end"),
            hash40("special_air_n_end"),
            hash40("special_n_end_max"),
            hash40("special_air_n_end_max"),
        ],
        statuses: &[0x1dc],
        require_both: false,
    },
    Rule {
        profile: Profile::ChromNeutralB,
        kind: 0x1b, // FIGHTER_KIND_CHROM
        motions: &[
            hash40("special_n"),
            hash40("special_air_n"),
            hash40("special_n_start"),
            hash40("special_air_n_start"),
            hash40("special_n_loop"),
            hash40("special_air_n_loop"),
            hash40("special_n_hold"),
            hash40("special_air_n_hold"),
            hash40("special_n_end"),
            hash40("special_air_n_end"),
            hash40("special_n_end_max"),
            hash40("special_air_n_end_max"),
        ],
        statuses: &[0x1dc],
        require_both: false,
    },
    Rule {
        profile: Profile::RoyNeutralB,
        kind: 0x1a, // FIGHTER_KIND_ROY
        motions: &[
            hash40("special_n"),
            hash40("special_air_n"),
            hash40("special_n_start"),
            hash40("special_air_n_start"),
            hash40("special_n_loop"),
            hash40("special_air_n_loop"),
            hash40("special_n_hold"),
            hash40("special_air_n_hold"),
            hash40("special_n_end"),
            hash40("special_air_n_end"),
            hash40("special_n_end_max"),
            hash40("special_air_n_end_max"),
        ],
        statuses: &[0x1dc],
        require_both: false,
    },
    Rule {
        profile: Profile::PalutenaRapidJab,
        kind: 0x36, // FIGHTER_KIND_PALUTENA
        motions: &[
            hash40("attack_100"),
            hash40("attack_100_start"),
            hash40("attack_100_end"),
        ],
        statuses: &[0x28],
        require_both: false,
    },
    Rule {
        profile: Profile::PalutenaNeutralB,
        kind: 0x36, // FIGHTER_KIND_PALUTENA
        motions: &[
            hash40("special_n"),
            hash40("special_air_n"),
            hash40("special_n_start"),
            hash40("special_air_n_start"),
            hash40("special_n_shoot"),
            hash40("special_air_n_shoot"),
            hash40("special_n_end"),
            hash40("special_air_n_end"),
        ],
        statuses: &[0x1dc],
        require_both: false,
    },
    Rule {
        profile: Profile::PalutenaUpB,
        kind: 0x36, // FIGHTER_KIND_PALUTENA
        motions: &[
            hash40("special_hi"),
            hash40("special_air_hi"),
            hash40("special_hi_start"),
            hash40("special_air_hi_start"),
            hash40("special_hi_end"),
            hash40("special_air_hi_end"),
        ],
        statuses: &[0x1de],
        require_both: false,
    },
    Rule {
        profile: Profile::PalutenaSideB,
        kind: 0x36, // FIGHTER_KIND_PALUTENA
        motions: &[
            hash40("special_s"),
            hash40("special_air_s"),
            hash40("special_s_start"),
            hash40("special_air_s_start"),
            hash40("special_s_shoot"),
            hash40("special_air_s_shoot"),
            hash40("special_s_end"),
            hash40("special_air_s_end"),
        ],
        statuses: &[0x1dd],
        require_both: false,
    },
    Rule {
        profile: Profile::PalutenaUpSmash,
        kind: 0x36, // FIGHTER_KIND_PALUTENA
        motions: &[
            hash40("attack_hi4"),
            hash40("attack_hi4_hold"),
            hash40("attack_hi4_start"),
        ],
        statuses: &[0x35, 0x34, 0x33],
        require_both: false,
    },
    Rule {
        profile: Profile::PalutenaTilts,
        kind: 0x36, // FIGHTER_KIND_PALUTENA
        motions: &[
            hash40("attack_s3"),
            hash40("attack_s3_s"),
            hash40("attack_s3_hi"),
            hash40("attack_s3_lw"),
            hash40("attack_hi3"),
            hash40("attack_lw3"),
        ],
        statuses: &[0x2a, 0x2b, 0x2c],
        require_both: false,
    },
    Rule {
        profile: Profile::PalutenaWingSmash,
        kind: 0x36, // FIGHTER_KIND_PALUTENA
        motions: &[
            hash40("attack_s4"),
            hash40("attack_s4_s"),
            hash40("attack_s4_hi"),
            hash40("attack_s4_lw"),
            hash40("attack_lw4"),
            hash40("attack_s4_hold"),
            hash40("attack_lw4_hold"),
            hash40("attack_s4_start"),
            hash40("attack_lw4_start"),
        ],
        statuses: &[0x2f, 0x2e, 0x2d, 0x32, 0x31, 0x30],
        require_both: false,
    },
    Rule {
        profile: Profile::PalutenaUpAir,
        kind: 0x36, // FIGHTER_KIND_PALUTENA
        motions: &[hash40("attack_air_hi")],
        statuses: &[0x36],
        require_both: true,
    },
    Rule {
        profile: Profile::EdgeFSmash,
        kind: 0x59, // FIGHTER_KIND_EDGE
        motions: &[
            hash40("attack_s4"),
            hash40("attack_s4_s"),
            hash40("attack_s4_hold"),
            hash40("attack_s4_start"),
            hash40("attack_hi4"),
            hash40("attack_hi4_start"),
            hash40("attack_hi4_hold"),
            hash40("attack_lw4"),
            hash40("attack_lw4_start"),
            hash40("attack_lw4_hold"),
        ],
        // bed642e: S4 0x2f; HI4/START/HOLD 0x35/0x33/0x34; LW4/START/HOLD 0x32/0x30/0x31.
        statuses: &[0x2f, 0x35, 0x33, 0x34, 0x32, 0x30, 0x31],
        require_both: false,
    },
    Rule {
        profile: Profile::EdgePurpleDash,
        kind: 0x59, // FIGHTER_KIND_EDGE
        motions: &[hash40("attack_dash"), hash40("attack_dash_hit")],
        statuses: &[0x29],
        require_both: false,
    },
    Rule {
        profile: Profile::EdgeNeutralB,
        kind: 0x59, // FIGHTER_KIND_EDGE
        motions: &[hash40("special_n_shoot"), hash40("special_air_n_shoot")],
        statuses: &[0x1e5],
        require_both: false,
    },
    Rule {
        profile: Profile::EdgeDownB,
        kind: 0x59, // FIGHTER_KIND_EDGE
        motions: &[
            hash40("special_lw"),
            hash40("special_air_lw"),
            hash40("special_lw_hit"),
            hash40("special_air_lw_hit"),
            hash40("special_lw_end"),
            hash40("special_air_lw_end"),
        ],
        statuses: &[0x1df, 0x1ee, 0x1ef],
        require_both: false,
    },
    Rule {
        profile: Profile::EdgeSideB,
        kind: 0x59, // FIGHTER_KIND_EDGE
        motions: &[
            hash40("special_s"),
            hash40("special_air_s"),
            hash40("special_s_hold"),
            hash40("special_air_s_hold"),
        ],
        statuses: &[0x1dd],
        require_both: false,
    },
];

pub fn tracked(category: u8, kind: i32) -> bool {
    match category {
        0 => RULES.iter().any(|r| r.kind == kind),
        // skyline-smash bed642e lua_const.rs: FALCONPUNCH 0x58, PK thunder 0x61/0x163,
        // explosive flame 0x110 and reserve 0x111, autoaim bullet 0x112, Edge fire/flash 0x246/0x247.
        1 => matches!(
            kind,
            0x58 | 0x61 | 0x110 | 0x111 | 0x112 | 0x163 | 0x246 | 0x247 | 0x248 | 0x249
        ),
        _ => false,
    }
}

pub fn profile(context: Context) -> Option<Profile> {
    // Fighter and weapon IDs overlap. Never interpret a weapon kind as a fighter.
    if context.category == 1 {
        return match context.kind {
            0x58 => Some(Profile::FalconPunch),
            0x61 => Some(Profile::NessUpB),
            0x163 => Some(Profile::LucasUpB),
            0x110 | 0x111 => Some(Profile::PalutenaSideB),
            0x112 => Some(Profile::PalutenaNeutralB),
            // BURST_S/M/L 0x3/0x4/0x5. FLY_S/M/L 0x0/0x1/0x2 stay excluded.
            0x246 if matches!(context.status, 3..=5) => Some(Profile::EdgeNeutralB),
            0x247 => Some(Profile::EdgeDownB),
            // skyline-smash bed642e lua_const.rs, cited in docs/FX.md: flare1, flare2.
            0x248 | 0x249 => Some(Profile::EdgeSideB),
            _ => None,
        };
    }
    if context.category != 0 {
        return None;
    }
    RULES.iter().find_map(|rule| {
        if context.kind != rule.kind {
            return None;
        }
        let motion = rule.motions.contains(&context.motion);
        let status = rule.statuses.contains(&context.status);
        let matches = if rule.require_both {
            motion && status
        } else {
            motion || status
        };
        matches.then_some(rule.profile)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_profile_accepts_its_move_and_rejects_idle_and_other_categories() {
        for rule in RULES {
            let c = Context {
                category: 0,
                kind: rule.kind,
                motion: rule.motions[0],
                status: rule.statuses[0],
            };
            assert_eq!(profile(c), Some(rule.profile));
            assert_eq!(profile(Context { category: 1, ..c }), None);
            assert_eq!(profile(Context { category: 2, ..c }), None);
            assert_eq!(
                profile(Context {
                    motion: hash40("wait"),
                    status: 0,
                    ..c
                }),
                None
            );
        }
    }

    #[test]
    fn weapons_need_their_own_kind_and_burst_status() {
        let c = Context {
            category: 1,
            kind: 0x246,
            motion: 0,
            status: 4,
        };
        assert_eq!(profile(c), Some(Profile::EdgeNeutralB));
        assert_eq!(
            profile(Context { status: 3, ..c }),
            Some(Profile::EdgeNeutralB)
        );
        assert_eq!(
            profile(Context { status: 5, ..c }),
            Some(Profile::EdgeNeutralB)
        );
        assert_eq!(profile(Context { status: 0, ..c }), None);
        assert_eq!(profile(Context { status: 1, ..c }), None);
        assert_eq!(profile(Context { status: 2, ..c }), None);
        assert_eq!(
            profile(Context {
                category: 0,
                status: 3,
                ..c
            }),
            None
        );
        assert_eq!(
            profile(Context {
                kind: 0x245,
                status: 3,
                ..c
            }),
            None
        );
        assert_ne!(
            profile(Context {
                kind: 0x247,
                status: 3,
                ..c
            }),
            Some(Profile::EdgeNeutralB)
        );
        assert_eq!(
            profile(Context {
                kind: 0x110,
                status: 0,
                ..c
            }),
            Some(Profile::PalutenaSideB)
        );
        assert_eq!(
            profile(Context {
                kind: 0x58,
                status: 0,
                ..c
            }),
            Some(Profile::FalconPunch)
        );
        assert_eq!(
            profile(Context {
                kind: 0x112,
                status: 0,
                ..c
            }),
            Some(Profile::PalutenaNeutralB)
        );
        // Palutena's aerial status alone must not classify every aerial as up-air.
        let c = Context {
            category: 0,
            kind: 0x36,
            motion: hash40("attack_air_f"),
            status: 0x36,
        };
        assert_eq!(profile(c), None);
        assert_eq!(
            profile(Context {
                motion: hash40("attack_air_hi"),
                ..c
            }),
            Some(Profile::PalutenaUpAir)
        );
        assert_eq!(
            profile(Context {
                category: 1,
                kind: 0x248,
                motion: 0,
                status: 0,
            }),
            Some(Profile::EdgeSideB)
        );
        assert_eq!(
            profile(Context {
                category: 1,
                kind: 0x249,
                motion: 0,
                status: 0,
            }),
            Some(Profile::EdgeSideB)
        );
        assert_eq!(
            profile(Context {
                category: 0,
                kind: 0x59,
                motion: hash40("special_s_hold"),
                status: 0,
            }),
            Some(Profile::EdgeSideB)
        );
        assert_eq!(
            profile(Context {
                category: 0,
                kind: 0x59,
                motion: hash40("attack_s4"),
                status: 0x2f,
            }),
            Some(Profile::EdgeFSmash)
        );
    }

    #[test]
    fn edge_up_and_down_smash_use_fsmash_profile() {
        let edge = |motion: &str, status: i32| Context {
            category: 0,
            kind: 0x59,
            motion: hash40(motion),
            status,
        };
        for (motion, status) in [
            ("attack_hi4", 0x35),
            ("attack_hi4_start", 0x33),
            ("attack_hi4_hold", 0x34),
            ("attack_lw4", 0x32),
            ("attack_lw4_start", 0x30),
            ("attack_lw4_hold", 0x31),
        ] {
            assert_eq!(profile(edge(motion, status)), Some(Profile::EdgeFSmash));
            assert_eq!(profile(edge(motion, 0)), Some(Profile::EdgeFSmash));
            assert_eq!(
                profile(Context {
                    motion: hash40("wait"),
                    ..edge(motion, status)
                }),
                Some(Profile::EdgeFSmash)
            );
        }
        for motion in [
            "attack_hi3",
            "attack_lw3",
            "attack_s3",
            "attack_air_hi",
            "attack_air_lw",
            "attack_air_f",
            "attack_air_n",
            "attack_air_b",
        ] {
            assert_eq!(profile(edge(motion, 0)), None);
        }
        for status in [0x2a, 0x2b, 0x2c, 0x36] {
            assert_eq!(
                profile(Context {
                    motion: hash40("wait"),
                    ..edge("wait", status)
                }),
                None
            );
        }
        assert_eq!(profile(edge("attack_air_hi", 0x36)), None);
        assert_eq!(
            profile(Context {
                kind: 0x23,
                ..edge("attack_hi4", 0x35)
            }),
            None
        );
        assert_eq!(
            profile(Context {
                kind: 0x23,
                ..edge("attack_lw4_hold", 0x31)
            }),
            None
        );
        assert_eq!(
            profile(Context {
                kind: 0x36,
                ..edge("attack_hi4", 0x35)
            }),
            Some(Profile::PalutenaUpSmash)
        );
        assert_eq!(
            profile(Context {
                kind: 0x36,
                ..edge("attack_lw4_hold", 0x31)
            }),
            Some(Profile::PalutenaWingSmash)
        );
        assert_eq!(
            profile(Context {
                category: 1,
                ..edge("attack_hi4", 0x35)
            }),
            None
        );
    }
}
