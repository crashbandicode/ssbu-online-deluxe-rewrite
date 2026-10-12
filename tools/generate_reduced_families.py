#!/usr/bin/env python3
"""Regenerate src/effects_reduced_family.rs from pinned script requests.

Names are Hash40 arguments of effect_* functions in WuBoytH/SSBU-Dumped-Scripts
commit 59004685238a3f6f93bae905d3a4e079701c40b4. Joints, textures, and sys_*
feedback are not listed. A name is included only when that commit requests it
on the cited move and a co-requested body or telegraph remains.

Pass --check PATH to require every keep/cull string to appear in an
effect-requests extract produced from `git show` of that commit. The extract
is not a repository file and must not contain Nintendo archives.
"""

from __future__ import annotations

import argparse
from pathlib import Path

PIN = "59004685238a3f6f93bae905d3a4e079701c40b4"

# (const prefix, Profile variant, keep, cull, source note)
PROFILES = [
    (
        "KAZUYAFSMASH",
        "KazuyaFSmash",
        ["demon_smash_line", "demon_devil_aura", "demon_devil_start"],
        ["demon_devil_start_elec", "demon_devil_sign_flash"],
        "lua2cpp_demon/demon/AttackS4.txt AttackHi4.txt AttackLw4.txt and AttackS4Charge.txt. Parent requests. Charge keeps start; start_elec and the sign flash are extra.",
    ),
    (
        "FALCONPUNCH",
        "FalconPunch",
        ["captain_fp_body_l", "captain_fp_body_r", "captain_fp_hold"],
        ["captain_fp_wing_l", "captain_fp_wing_r"],
        "Fighter lua2cpp_captain/captain/SpecialN.txt requests flash at frame 6 and hold at frame 26. Flash is the only parent until the hold starts, so it stays original. Weapon captain_falconpunch/SpecialN.txt requests body and wing in the same block. WEAPON_KIND_CAPTAIN_FALCONPUNCH is 0x58. Hit sparks stay original. Falcon Kick fk_* names are a different move.",
    ),
    (
        "NESSUPB",
        "NessUpB",
        ["ness_pkt_attack"],
        ["ness_pkt_bomb"],
        "lua2cpp_ness/ness/SpecialAirHi.txt. Parent requests. ness_pkt_bullet on ness_pkthunder/Move.txt is the projectile and is not listed. Hold stays original.",
    ),
    (
        "LUCASUPB",
        "LucasUpB",
        ["lucas_pkt_attack"],
        ["lucas_pkt_bomb"],
        "lua2cpp_lucas/lucas/SpecialAirHi.txt. Parent requests. lucas_pkt_bullet on lucas_pkthunder/Move.txt stays original.",
    ),
    (
        "IKENEUTRALB",
        "IkeNeutralB",
        ["ike_volcano", "ike_volcano_middle", "ike_volcano_hold"],
        ["ike_volcano_add", "ike_volcano_add2", "ike_volcano_add3", "ike_volcano_add4"],
        "lua2cpp_ike/ike/SpecialNEnd.txt has volcano without middle. SpecialNEndMdl.txt and SpecialNEndMax.txt request middle. SpecialNStart.txt requests hold. Max, ground, and flash names stay original.",
    ),
    (
        "CHROMNEUTRALB",
        "ChromNeutralB",
        ["chrom_volcano_a", "chrom_volcano_e", "chrom_volcano_c", "chrom_sword_purple"],
        ["chrom_volcano_hold_2", "chrom_volcano_spark"],
        "lua2cpp_chrom/chrom/SpecialNEnd.txt End2 End3 EndMax. Each charge level keeps its own body. hold_1 stays; hold_2 is the second hold layer.",
    ),
    (
        "ROYNEUTRALB",
        "RoyNeutralB",
        [
            "roy_eruption_bomb_main",
            "roy_eruption",
            "roy_eruption_middle",
            "roy_eruption_max",
            "roy_erupution_hold",
        ],
        [
            "roy_eruption_bomb_start",
            "roy_eruption_bomb_start_middle",
            "roy_eruption_bomb_start_high",
            "roy_eruption_bomb_start_max",
        ],
        "lua2cpp_roy/roy/SpecialNEnd.txt through EndMax. The hold string is the script typo roy_erupution_hold. bomb_main is on every level.",
    ),
    (
        "PALUTENARAPIDJAB",
        "PalutenaRapidJab",
        ["palutena_wand_spark", "palutena_wand_finish"],
        ["palutena_wand_light1"],
        "lua2cpp_palutena/palutena/Attack100.txt requests only wand_spark. Attack100End.txt adds light1 and finish. Spark stays so the loop is not blank.",
    ),
    (
        "PALUTENANEUTRALB",
        "PalutenaNeutralB",
        ["palutena_lockon", "palutena_bullet_shot"],
        ["palutena_wand_light_trace", "palutena_wand_light2"],
        "Fighter SpecialN.txt requests lock-on and wand lights. palutena_autoaimbullet/Shot.txt requests only palutena_bullet_shot. Weapon kind 0x112. The shot is kept.",
    ),
    (
        "PALUTENAUPB",
        "PalutenaUpB",
        ["palutena_teleport_start", "palutena_teleport_end"],
        ["palutena_backlight"],
        "SpecialHiStart.txt and SpecialHi.txt. Parent requests. palutena_gate is not requested here.",
    ),
    (
        "PALUTENASIDEB",
        "PalutenaSideB",
        ["palutena_bomb", "palutena_bomb_finish"],
        ["palutena_bomb_appear", "palutena_wand_light_trace", "palutena_wand_light2", "palutena_backlight"],
        "palutena_explosiveflame/Explode.txt requests bomb, bomb_appear, and bomb_finish. Fighter SpecialS.txt requests the wand lights. palutena_explosiveflame is not a request. Weapon kinds 0x110 and 0x111.",
    ),
    (
        "PALUTENAUPSMASH",
        "PalutenaUpSmash",
        ["palutena_pressure", "palutena_wand_light2"],
        ["palutena_backlight"],
        "AttackHi4.txt. palutena_meteor is not requested in this script.",
    ),
    (
        "PALUTENATILTS",
        "PalutenaTilts",
        ["palutena_wand_light4", "palutena_wand_trace_01"],
        ["palutena_wand_trace_08"],
        "AttackS3.txt AttackHi3.txt AttackLw3.txt request traces 01-08 plus light4. Only the last trace segment is culled.",
    ),
    (
        "PALUTENAWINGSMASH",
        "PalutenaWingSmash",
        ["palutena_feather"],
        ["palutena_backlight"],
        "AttackS4.txt and AttackLw4.txt. Smash traces stay original. palutena_feather00 is not the request on these scripts.",
    ),
    (
        "PALUTENAUPAIR",
        "PalutenaUpAir",
        [],
        [],
        "AttackAirHi.txt requests only protected palutena_backlight2. eff_variant supplies a lighter sibling retaining two emitters; this request-level cull table stays empty.",
    ),
    (
        "EDGEFSMASH",
        "EdgeFSmash",
        ["edge_sword", "edge_sword_flare", "edge_slash_light"],
        ["edge_sword_smash_flash", "edge_sword_light3"],
        "lua2cpp_edge/edge/AttackS4.txt AttackHi4.txt AttackLw4.txt. sys_smash_flash stays original. tex_edge_sword textures are not effect requests.",
    ),
    (
        "EDGEPURPLEDASH",
        "EdgePurpleDash",
        ["edge_attack_dash", "edge_attack_dash_hit"],
        ["edge_attack_dash_aura", "edge_attack_dash2"],
        "AttackDash.txt and AttackDashHit.txt. debris1, fire3, flash1, flash2, ring1, and sparkline1 are not requests in these scripts.",
    ),
    (
        "EDGENEUTRALB",
        "EdgeNeutralB",
        ["edge_fire3_burst2", "edge_fire2_burst2", "edge_fire1_burst"],
        ["edge_fire3_burst", "edge_fire2_burst", "edge_fire1_sign"],
        "edge_fire/BurstL.txt BurstM.txt BurstS.txt. BurstS has only sign plus burst, so the burst stays and the sign is the extra. Bullets and triggers stay original. edge_fire3_screen2 is the screen skip, not a body null.",
    ),
    (
        "EDGEDOWNB",
        "EdgeDownB",
        ["edge_senkou_shield", "edge_senkou_hold_attack"],
        ["edge_senkou_start", "edge_senkou_hold"],
        "Fighter SpecialLw.txt requests start and shield. Weapon edge_flash/Attack.txt requests hold_attack and hold. Kind 0x247. Success and break layers stay original.",
    ),
    (
        "EDGESIDEB",
        "EdgeSideB",
        [
            "edge_shadowflare_aura",
            "edge_shadowflare_beam",
            "edge_shadowflare_beam_hit",
            "edge_shadowflare_bullet",
            "edge_shadowflare_bullet_attack",
            "edge_shadowflare_bullet_end",
            "edge_shadowflare_bullet_hit",
            "edge_shadowflare_start",
            "edge_shadowflare_hold",
        ],
        [],
        "SpecialS and SpecialAirS request only edge_shadowflare_start, detached at frame 10. SpecialSHold and SpecialAirSHold request hold until frame 24, when aura replaces it. Those two early cues stay and use the lighter siblings. Aura, beam, beam_hit, bullet, bullet_attack, bullet_end, and bullet_hit stay.",
    ),
]

GLOBALS = [
    (
        "PYRA",
        ["eflame_sword_open", "eflame_attack_dash"],
        ["eflame_sword_firetrail", "eflame_sword_firetrail_end"],
        "lua2cpp_eflame sword scripts request firetrail together with sword_open and sword_fire. firetrail is never the only effect. Blaze-end projectiles are not listed.",
    ),
    (
        "MYTHRA",
        ["elight_sword_open"],
        ["elight_sword_light", "elight_sword_light_few"],
        "lua2cpp_elight sword scripts. The light layers are never the only request. Sword beams and buster projectiles stay original.",
    ),
    (
        "BYLETH",
        ["master_axeflare1", "master_axe_slash"],
        ["master_axeflare3", "master_axe_hold2"],
        "lua2cpp_master axe scripts request axeflare1 with axeflare3. SpecialLw.txt requests slash with hold and hold2. LandingAirLw.txt still keeps axeflare1.",
    ),
    (
        "INKLING",
        [
            "inkling_pablo_splash",
            "inkling_slosher_splash",
            "inkling_slosher_splash2",
        ],
        ["inkling_pablo_trace2", "inkling_slosher_trace"],
        "AttackS4.txt full-ink body is inkling_pablo_splash. inkling_pablo_trace2 stays the extra cull. inkling_pablo_trace stays original because the hdr decompile places that follow outside the ink check. AttackLw4.txt full ink requests inkling_slosher_splash plus inkling_slosher_trace. Splash stays. Trace is the extra cull. inkling_slosher_splash2 is only the no-ink else body, so it is kept. Roller parents are not in this table; eff_variant keeps their Splash emitter. Blaster and bomb projectiles stay original.",
    ),
    (
        "GANONDORF",
        ["ganon_majinken", "ganon_majinken_start", "ganon_sword_flare"],
        ["ganon_majinken_flash", "ganon_sword_trace"],
        "SpecialN.txt requests majinken with flash. AttackS4.txt requests sword_flare with sword_trace. sword_flare is the only effect on other scripts, so it is kept.",
    ),
    (
        "CHROMEXTRA",
        [],
        [],
        "chrom_mc_aura and its suffixes are not Hash40 requests at this pin. Each chrom_mc_* side-B branch requests one trace1 parent. eff_variant supplies a guarded first-trail chrom_mc_1 sibling with life 3 to 2; the other branches remain original. No parent is nulled here.",
    ),
]


def rust_list(names: list[str]) -> str:
    if not names:
        return "&[]"
    body = ",\n".join(f'    "{name}"' for name in names)
    return f"&[\n{body},\n]"


def emit_pair(prefix: str, keep: list[str], cull: list[str], note: str) -> str:
    return f"""// {note}
const {prefix}_KEEP_NAMES: &[&str] = {rust_list(keep)};

const {prefix}_CULL_NAMES: &[&str] = {rust_list(cull)};

const {prefix}_KEEP: [u64; {prefix}_KEEP_NAMES.len()] = sorted_hashes({prefix}_KEEP_NAMES);

const {prefix}_CULL: [u64; {prefix}_CULL_NAMES.len()] = sorted_hashes({prefix}_CULL_NAMES);
"""


def render() -> str:
    parts = [
        "// Generated by tools/generate_reduced_families.py. Do not edit by hand.",
        f"// Script pin: WuBoytH/SSBU-Dumped-Scripts {PIN}.",
        "// Every listed name is a parent Hash40 request from that pin, not an emitter child or kill_kind alias.",
        "",
        "use super::Role;",
        "",
        "use crate::{effect_context::Profile, effects::sorted_hashes};",
        "",
    ]
    for prefix, _profile, keep, cull, note in PROFILES:
        parts.append(emit_pair(prefix, keep, cull, note))
    for prefix, keep, cull, note in GLOBALS:
        parts.append(emit_pair(prefix, keep, cull, note))
    parts.append(
        """
fn contains(table: &[u64], hash: u64) -> bool {
    table.binary_search(&hash).is_ok()
}

fn pair(hash: u64, keep: &[u64], cull: &[u64]) -> Role {
    if contains(cull, hash) {
        Role::Cull
    } else if contains(keep, hash) {
        Role::Keep
    } else {
        Role::None
    }
}

pub(super) fn profile_role(profile: Profile, hash: u64) -> Role {
    match profile {
"""
    )
    for prefix, profile, _keep, _cull, _note in PROFILES:
        parts.append(
            f"        Profile::{profile} => pair(hash, &{prefix}_KEEP, &{prefix}_CULL),"
        )
    parts.append(
        """    }
}

pub(super) fn global_role(hash: u64) -> Role {
    let found = [
"""
    )
    for prefix, _keep, _cull, _note in GLOBALS:
        parts.append(f"        pair(hash, &{prefix}_KEEP, &{prefix}_CULL),")
    parts.append(
        """    ];
    for role in found {
        if role != Role::None {
            return role;
        }
    }
    Role::None
}

pub(super) fn family_named(hash: u64) -> bool {
    const PROFILES: [Profile; """ + str(len(PROFILES)) + """] = [
"""
    )
    for _prefix, profile, _keep, _cull, _note in PROFILES:
        parts.append(f"        Profile::{profile},")
    parts.append(
        """    ];
    PROFILES.iter().any(|profile| profile_role(*profile, hash) != Role::None) || global_role(hash) != Role::None
}
"""
    )
    return "\n".join(parts) + "\n"


def check(extract: Path) -> None:
    text = extract.read_text(encoding="utf-8", errors="replace")
    requested = set()
    for line in text.splitlines():
        name = line.strip()
        if name:
            requested.add(name)
    missing = []
    for prefix, _profile, keep, cull, _note in PROFILES:
        for name in keep + cull:
            if name not in requested:
                missing.append(f"{prefix} {name}")
    for prefix, keep, cull, _note in GLOBALS:
        for name in keep + cull:
            if name not in requested:
                missing.append(f"{prefix} {name}")
    if missing:
        raise SystemExit("names absent from extract:\\n" + "\\n".join(missing))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.check is not None:
        check(args.check)
    output = args.output
    if output is None:
        output = Path(__file__).resolve().parents[1] / "src" / "effects_reduced_family.rs"
    output.write_text(render(), encoding="utf-8")


if __name__ == "__main__":
    main()
