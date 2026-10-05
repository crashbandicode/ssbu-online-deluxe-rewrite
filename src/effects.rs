//! Initial, explicit character-FX scope. No wildcard or global hit-effect deletion.
use crate::config::FxMode;
use crate::effect_context::{self, Context};

pub const fn hash40(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut crc = 0xffff_ffffu32;
    let mut index = 0;
    while index < bytes.len() {
        crc ^= bytes[index] as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = (crc >> 1) ^ if crc & 1 != 0 { 0xedb8_8320 } else { 0 };
            bit += 1;
        }
        index += 1;
    }
    ((bytes.len() as u64) << 32) | (!crc as u64)
}

/// Same named null resource used by the peer mod; not the numeric hash zero.
/// The original request still executes, preserving the engine's request path.
pub const NULL_EFFECT: u64 = hash40("null");

pub const TARGET_NAMES: &[&str] = &[
    // Explosive Flame: peer May-14 source, corroborated by vanilla Explode ACMD.
    "palutena_bomb",
    "palutena_bomb_appear",
    "palutena_bomb_finish",
    // Shadow Flare: vanilla fighter/flare1/flare2 ACMD and local ef_edge.eff.
    "edge_shadowflare_start",
    "edge_shadowflare_hold",
    "edge_shadowflare_aura",
    "edge_shadowflare_beam",
    "edge_shadowflare_beam_hit",
    "edge_shadowflare_bullet",
    "edge_shadowflare_bullet_attack",
    "edge_shadowflare_bullet_hit",
    "edge_shadowflare_bullet_end",
    "P_ChromVolcanoA",
    "P_ChromVolcanoB",
    "P_ChromVolcanoC",
    "P_ChromVolcanoD",
    "P_ChromVolcanoE",
    "P_ChromVolcanoF",
    "P_ChromVolcanoHold",
    "P_ChromVolcanoHold1",
    "P_ChromVolcanoHold2",
    "P_ChromVolcanoParts1",
    "P_ChromVolcanoParts1b",
    "P_ChromVolcanoParts2",
    "P_ChromVolcanoParts3",
    "P_ChromVolcanoSpark",
    "P_EdgeFire2Burst",
    "P_EdgeFire2Burst2",
    "P_EdgeFire2Smoke",
    "P_EdgeFire3Burst",
    "P_EdgeFire3Burst2",
    "P_EdgeFire3Smoke",
    "P_IkeVolcano",
    "P_IkeVolcanoAdd",
    "P_IkeVolcanoAdd2",
    "P_IkeVolcanoAdd3",
    "P_IkeVolcanoAdd4",
    "P_IkeVolcanoFlash",
    "P_IkeVolcanoFlash2",
    "P_IkeVolcanoFlash2Ground",
    "P_IkeVolcanoFlash3",
    "P_IkeVolcanoFlash3Ground",
    "P_IkeVolcanoFlashGround",
    "P_IkeVolcanoGround",
    "P_IkeVolcanoHold",
    "P_IkeVolcanoMax",
    "P_IkeVolcanoMiddle",
    "P_RoyEruption",
    "P_RoyEruptionBombMain",
    "P_RoyEruptionBombStart",
    "P_RoyEruptionBombStart_High",
    "P_RoyEruptionBombStart_Max",
    "P_RoyEruptionBombStart_Middle",
    "P_RoyEruptionHold",
    "P_RoyEruptionMax",
    "P_RoyEruptionMiddle",
    "captain_fp_body_l",
    "captain_fp_body_r",
    "captain_fp_flash",
    "captain_fp_hit_l",
    "captain_fp_hit_r",
    "captain_fp_hold",
    "captain_fp_wing_l",
    "captain_fp_wing_r",
    "chrom_volcano_a",
    "chrom_volcano_b",
    "chrom_volcano_c",
    "chrom_volcano_d",
    "chrom_volcano_e",
    "chrom_volcano_f",
    "chrom_volcano_hold",
    "chrom_volcano_hold_1",
    "chrom_volcano_hold_2",
    "chrom_volcano_parts_1",
    "chrom_volcano_parts_1b",
    "chrom_volcano_parts_2",
    "chrom_volcano_parts_3",
    "chrom_volcano_spark",
    "edge_fire2_burst",
    "edge_fire2_burst2",
    "edge_fire2_smoke",
    "edge_fire3_burst",
    "edge_fire3_burst2",
    "edge_fire3_smoke",
    "ike_volcano",
    "ike_volcano_add",
    "ike_volcano_add1",
    "ike_volcano_add2",
    "ike_volcano_add3",
    "ike_volcano_add4",
    "ike_volcano_bomb",
    "ike_volcano_fire",
    "ike_volcano_flash",
    "ike_volcano_flash2",
    "ike_volcano_flash3",
    "ike_volcano_ground",
    "ike_volcano_hold",
    "ike_volcano_max",
    "master_axe_aura",
    "master_axe_hold",
    "master_axe_hold2",
    "master_axe_hold_end",
    "master_axe_slash_edge",
    "master_axe_slash_particle",
    "master_axeflare1",
    "master_axeflare2",
    "master_axeflare3",
    "master_axeflare_sp1",
    "master_axeflare_sp1_end",
    "master_axeflare_sp2",
    "master_axeflare_sp2_end",
    "master_axeflare_sp3",
    "master_axeflare_sp3_end",
    "roy_eruption",
    "roy_eruption_bomb_main",
    "roy_eruption_bomb_start",
    "roy_eruption_bomb_start_high",
    "roy_eruption_bomb_start_max",
    "roy_eruption_bomb_start_middle",
    "roy_eruption_hold",
    "roy_eruption_max",
    "roy_eruption_middle",
    "roy_erupution_hold",
];

const TARGET_HASHES: &[u64] = &[
    hash40("palutena_bomb"),
    hash40("palutena_bomb_appear"),
    hash40("palutena_bomb_finish"),
    hash40("edge_shadowflare_start"),
    hash40("edge_shadowflare_hold"),
    hash40("edge_shadowflare_aura"),
    hash40("edge_shadowflare_beam"),
    hash40("edge_shadowflare_beam_hit"),
    hash40("edge_shadowflare_bullet"),
    hash40("edge_shadowflare_bullet_attack"),
    hash40("edge_shadowflare_bullet_hit"),
    hash40("edge_shadowflare_bullet_end"),
    hash40("P_ChromVolcanoA"),
    hash40("P_ChromVolcanoB"),
    hash40("P_ChromVolcanoC"),
    hash40("P_ChromVolcanoD"),
    hash40("P_ChromVolcanoE"),
    hash40("P_ChromVolcanoF"),
    hash40("P_ChromVolcanoHold"),
    hash40("P_ChromVolcanoHold1"),
    hash40("P_ChromVolcanoHold2"),
    hash40("P_ChromVolcanoParts1"),
    hash40("P_ChromVolcanoParts1b"),
    hash40("P_ChromVolcanoParts2"),
    hash40("P_ChromVolcanoParts3"),
    hash40("P_ChromVolcanoSpark"),
    hash40("P_EdgeFire2Burst"),
    hash40("P_EdgeFire2Burst2"),
    hash40("P_EdgeFire2Smoke"),
    hash40("P_EdgeFire3Burst"),
    hash40("P_EdgeFire3Burst2"),
    hash40("P_EdgeFire3Smoke"),
    hash40("P_IkeVolcano"),
    hash40("P_IkeVolcanoAdd"),
    hash40("P_IkeVolcanoAdd2"),
    hash40("P_IkeVolcanoAdd3"),
    hash40("P_IkeVolcanoAdd4"),
    hash40("P_IkeVolcanoFlash"),
    hash40("P_IkeVolcanoFlash2"),
    hash40("P_IkeVolcanoFlash2Ground"),
    hash40("P_IkeVolcanoFlash3"),
    hash40("P_IkeVolcanoFlash3Ground"),
    hash40("P_IkeVolcanoFlashGround"),
    hash40("P_IkeVolcanoGround"),
    hash40("P_IkeVolcanoHold"),
    hash40("P_IkeVolcanoMax"),
    hash40("P_IkeVolcanoMiddle"),
    hash40("P_RoyEruption"),
    hash40("P_RoyEruptionBombMain"),
    hash40("P_RoyEruptionBombStart"),
    hash40("P_RoyEruptionBombStart_High"),
    hash40("P_RoyEruptionBombStart_Max"),
    hash40("P_RoyEruptionBombStart_Middle"),
    hash40("P_RoyEruptionHold"),
    hash40("P_RoyEruptionMax"),
    hash40("P_RoyEruptionMiddle"),
    hash40("captain_fp_body_l"),
    hash40("captain_fp_body_r"),
    hash40("captain_fp_flash"),
    hash40("captain_fp_hit_l"),
    hash40("captain_fp_hit_r"),
    hash40("captain_fp_hold"),
    hash40("captain_fp_wing_l"),
    hash40("captain_fp_wing_r"),
    hash40("chrom_volcano_a"),
    hash40("chrom_volcano_b"),
    hash40("chrom_volcano_c"),
    hash40("chrom_volcano_d"),
    hash40("chrom_volcano_e"),
    hash40("chrom_volcano_f"),
    hash40("chrom_volcano_hold"),
    hash40("chrom_volcano_hold_1"),
    hash40("chrom_volcano_hold_2"),
    hash40("chrom_volcano_parts_1"),
    hash40("chrom_volcano_parts_1b"),
    hash40("chrom_volcano_parts_2"),
    hash40("chrom_volcano_parts_3"),
    hash40("chrom_volcano_spark"),
    hash40("edge_fire2_burst"),
    hash40("edge_fire2_burst2"),
    hash40("edge_fire2_smoke"),
    hash40("edge_fire3_burst"),
    hash40("edge_fire3_burst2"),
    hash40("edge_fire3_smoke"),
    hash40("ike_volcano"),
    hash40("ike_volcano_add"),
    hash40("ike_volcano_add1"),
    hash40("ike_volcano_add2"),
    hash40("ike_volcano_add3"),
    hash40("ike_volcano_add4"),
    hash40("ike_volcano_bomb"),
    hash40("ike_volcano_fire"),
    hash40("ike_volcano_flash"),
    hash40("ike_volcano_flash2"),
    hash40("ike_volcano_flash3"),
    hash40("ike_volcano_ground"),
    hash40("ike_volcano_hold"),
    hash40("ike_volcano_max"),
    hash40("master_axe_aura"),
    hash40("master_axe_hold"),
    hash40("master_axe_hold2"),
    hash40("master_axe_hold_end"),
    hash40("master_axe_slash_edge"),
    hash40("master_axe_slash_particle"),
    hash40("master_axeflare1"),
    hash40("master_axeflare2"),
    hash40("master_axeflare3"),
    hash40("master_axeflare_sp1"),
    hash40("master_axeflare_sp1_end"),
    hash40("master_axeflare_sp2"),
    hash40("master_axeflare_sp2_end"),
    hash40("master_axeflare_sp3"),
    hash40("master_axeflare_sp3_end"),
    hash40("roy_eruption"),
    hash40("roy_eruption_bomb_main"),
    hash40("roy_eruption_bomb_start"),
    hash40("roy_eruption_bomb_start_high"),
    hash40("roy_eruption_bomb_start_max"),
    hash40("roy_eruption_bomb_start_middle"),
    hash40("roy_eruption_hold"),
    hash40("roy_eruption_max"),
    hash40("roy_eruption_middle"),
    hash40("roy_erupution_hold"),
];

pub fn rewrite(mode: FxMode, hash: u64) -> u64 {
    if mode == FxMode::Off
        && (TARGET_HASHES.contains(&hash) || crate::effects_catalog::global(hash))
    {
        NULL_EFFECT
    } else {
        hash
    }
}

pub fn needs_context(hash: u64) -> bool {
    crate::effects_catalog::needs_context(hash)
}

pub fn rewrite_with_context(mode: FxMode, hash: u64, context: Option<Context>) -> u64 {
    let global = rewrite(mode, hash);
    if global != hash || mode != FxMode::Off {
        return global;
    }
    if context
        .and_then(effect_context::profile)
        .is_some_and(|p| crate::effects_catalog::scoped(p, hash))
    {
        NULL_EFFECT
    } else {
        hash
    }
}

/// Build sorted lookup tables at compile time; no runtime hashing/allocation.
pub(crate) const fn sorted_hashes<const N: usize>(names: &[&str]) -> [u64; N] {
    assert!(names.len() == N);
    let mut values = [0; N];
    let mut i = 0;
    while i < N {
        let hash = hash40(names[i]);
        let mut j = i;
        while j > 0 && values[j - 1] > hash {
            values[j] = values[j - 1];
            j -= 1;
        }
        values[j] = hash;
        i += 1;
    }
    values
}

pub fn suppress_screen(mode: FxMode, hash: u64) -> bool {
    mode == FxMode::Off && hash == hash40("edge_fire3_screen2")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn on_and_reserved_reduced_preserve_every_request() {
        for &hash in TARGET_HASHES
            .iter()
            .chain([0, u64::MAX, hash40("sys_hit_normal")].iter())
        {
            assert_eq!(rewrite(FxMode::On, hash), hash);
            assert_eq!(rewrite(FxMode::Reduced, hash), hash);
        }
        assert!(!suppress_screen(FxMode::On, hash40("edge_fire3_screen2")));
    }
    #[test]
    fn off_is_scoped_and_does_not_erase_general_feedback() {
        assert_eq!(rewrite(FxMode::Off, hash40("ike_volcano")), NULL_EFFECT);
        for name in [
            "sys_hit_normal",
            "sys_shield",
            "sys_damage_fire",
            "sys_score_aura",
            "bg_finishhit",
        ] {
            assert_eq!(rewrite(FxMode::Off, hash40(name)), hash40(name));
            assert!(!suppress_screen(FxMode::Off, hash40(name)));
        }
        assert!(suppress_screen(FxMode::Off, hash40("edge_fire3_screen2")));
    }
    #[test]
    fn hash_format_matches_crc32_and_name_table() {
        // CRC-32/ISO-HDLC standard check string.
        assert_eq!(hash40("123456789"), 0x09cb_f43926);
        for name in TARGET_NAMES {
            assert!(TARGET_HASHES.contains(&hash40(name)));
        }
        assert_eq!(TARGET_NAMES.len(), TARGET_HASHES.len());
    }
}
