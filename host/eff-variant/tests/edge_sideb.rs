use std::fs;
use std::io::Cursor;

use eff_variant::{append_reduced_sibling, SiblingRequest};
use effect_library::NamcoEffectFile;
use ssbu_online_deluxe_rewrite::eff_variant::{EDGE_MAX_BYTES, EDGE_REQUESTS};

const EDGE: &str = "/tmp/eff-probe/out/effect__fighter__edge__ef_edge.eff";
const CHROM: &str = "/tmp/eff-probe/out/effect__fighter__chrom__ef_chrom.eff";

fn names_of(set: &effect_library::EmitterSet) -> Vec<String> {
    set.emitters
        .iter()
        .map(|emitter| emitter.data.display_name())
        .collect()
}

#[test]
#[ignore = "requires locally extracted stock EFF; no game assets are distributed"]
fn stock_edge_aliases_keep_their_set_index_when_a_sibling_is_appended() {
    let original = fs::read(EDGE).expect("edge");
    let mut current = original.clone();
    for request in EDGE_REQUESTS {
        current = append_reduced_sibling(&current, request)
            .unwrap_or_else(|error| panic!("{}: {error:?}", request.new_handle))
            .bytes;
    }
    eprintln!("EDGE_REDUCED_LEN={}", current.len());
    assert!(current.len() <= EDGE_MAX_BYTES, "{}", current.len());
    assert_eq!(i16::from_le_bytes(current[8..10].try_into().unwrap()), 147);
    let parsed = eff_lib::EffFile::read(&mut Cursor::new(current.as_slice())).expect("eff_lib");
    assert_eq!(parsed.effect_handles.len(), 147);
    let original_handles =
        eff_lib::EffFile::read(&mut Cursor::new(original.as_slice())).expect("original");
    assert_eq!(original_handles.effect_handles.len(), 139);
    assert_eq!(
        &parsed.effect_handles[..139],
        original_handles.effect_handles.as_slice()
    );
    let shared = original_handles
        .effect_handles
        .iter()
        .filter(|handle| handle.emitter_set_handle == 110)
        .count();
    assert_eq!(shared, 6);
    assert_eq!(parsed.effect_handles[139].emitter_set_handle, 134);
    assert_eq!(parsed.effect_handles[146].emitter_set_handle, 141);
    assert_eq!(
        parsed.effect_handle_names[146].to_string().unwrap(),
        "EDGE_SHADOWFLARE_BULLET_HIT_REDUCED"
    );

    let variant = NamcoEffectFile::load(&current).expect("oracle");
    let sets = &variant
        .ptcl_file
        .as_ref()
        .expect("ptcl")
        .emitter_list
        .emitter_sets;
    assert_eq!(sets.len(), 141);
    let reduced = sets
        .iter()
        .find(|set| set.name == "P_EdgeShadowFlareBulletReduced")
        .expect("bullet sibling");
    assert_eq!(
        names_of(reduced),
        vec!["sphereBase".to_string(), "flare1".to_string()]
    );
    let aura = sets
        .iter()
        .find(|set| set.name == "P_EdgeShadowFlareAura")
        .expect("aura");
    let aura_emitter = &aura.emitters[0];
    assert!(aura_emitter.children.is_empty());
    eprintln!(
        "AURA rate={} rate_random={} life={} life_random={} infinite={} children={}",
        aura_emitter.data.emission.rate,
        aura_emitter.data.emission.rate_random,
        aura_emitter.data.particle_data.life,
        aura_emitter.data.particle_data.life_random,
        aura_emitter.data.particle_data.infinite_life,
        aura_emitter.children.len()
    );
    assert_eq!(names_of(aura), vec!["aura1_bk1".to_string()]);
    assert!(sets
        .iter()
        .all(|set| set.name != "P_EdgeShadowFlareAuraReduced"));
}

#[test]
#[ignore = "requires locally extracted stock EFF; no game assets are distributed"]
fn stock_chrom_zero_set_handles_stay_zero() {
    let original = fs::read(CHROM).expect("chrom");
    let appended = append_reduced_sibling(
        &original,
        &SiblingRequest {
            source_handle: "CHROM_MC_1",
            source_set: "P_ChromMC1",
            new_handle: "CHROM_MC_1_REDUCED",
            new_set: "P_ChromMC1Reduced",
            keep_emitters: &["trace1"],
        },
    )
    .expect("chrom append");
    let parsed = eff_lib::EffFile::read(&mut Cursor::new(appended.bytes.as_slice())).expect("eff");
    let before = eff_lib::EffFile::read(&mut Cursor::new(original.as_slice())).expect("before");
    assert_eq!(parsed.effect_handles.len(), 64);
    assert_eq!(
        &parsed.effect_handles[..63],
        before.effect_handles.as_slice()
    );
    assert_eq!(
        before
            .effect_handles
            .iter()
            .filter(|handle| handle.emitter_set_handle == 0)
            .count(),
        3
    );
    assert_eq!(parsed.effect_handles[63].emitter_set_handle, 61);
    let variant = NamcoEffectFile::load(&appended.bytes).expect("oracle");
    let reduced = variant
        .ptcl_file
        .as_ref()
        .expect("ptcl")
        .emitter_list
        .emitter_sets
        .iter()
        .find(|set| set.name == "P_ChromMC1Reduced")
        .expect("sibling");
    assert_eq!(names_of(reduced), vec!["trace1".to_string()]);
    let parent = variant
        .ptcl_file
        .as_ref()
        .expect("ptcl")
        .emitter_list
        .emitter_sets
        .iter()
        .find(|set| set.name == "P_ChromMC1")
        .expect("parent");
    let trace = &parent.emitters[0];
    assert!(trace.children.is_empty());
    eprintln!(
        "CHROM_TRACE rate={} rate_random={} life={} life_random={} infinite={} children={}",
        trace.data.emission.rate,
        trace.data.emission.rate_random,
        trace.data.particle_data.life,
        trace.data.particle_data.life_random,
        trace.data.particle_data.infinite_life,
        trace.children.len()
    );
}
