use std::fs;
use std::io::Cursor;

use eff_variant::append_reduced_sibling;
use effect_library::NamcoEffectFile;
use ssbu_online_deluxe_rewrite::eff_variant::{INK_MAX_BYTES, INK_REQUESTS};

const INK: &str = "/tmp/eff-probe/out/effect__fighter__inkling__ef_inkling.eff";

#[test]
#[ignore = "requires locally extracted stock EFF; no game assets are distributed"]
fn stock_roller_siblings_keep_splash_and_drop_the_extra_emitters() {
    let original = fs::read(INK).expect("inkling");
    let mut current = original.clone();
    for request in INK_REQUESTS {
        current = append_reduced_sibling(&current, request)
            .unwrap_or_else(|error| panic!("{}: {error:?}", request.new_handle))
            .bytes;
    }
    eprintln!("INK_REDUCED_LEN={}", current.len());
    assert!(current.len() <= INK_MAX_BYTES, "{}", current.len());
    let parsed = eff_lib::EffFile::read(&mut Cursor::new(current.as_slice())).expect("eff");
    let before = eff_lib::EffFile::read(&mut Cursor::new(original.as_slice())).expect("before");
    assert_eq!(before.effect_handles.len(), 56);
    assert_eq!(parsed.effect_handles.len(), 59);
    assert_eq!(
        &parsed.effect_handles[..56],
        before.effect_handles.as_slice()
    );
    assert_eq!(parsed.effect_handles[56].emitter_set_handle, 57);
    assert_eq!(parsed.effect_handles[58].emitter_set_handle, 59);
    let variant = NamcoEffectFile::load(&current).expect("oracle");
    let sets = &variant.ptcl_file.expect("ptcl").emitter_list.emitter_sets;
    assert_eq!(sets.len(), 59);
    for name in [
        "P_InklingRollerSplashWalkReduced",
        "P_InklingRollerSplashRunReduced",
        "P_InklingRollerSplashAirReduced",
    ] {
        let set = sets.iter().find(|set| set.name == name).expect(name);
        let names: Vec<_> = set
            .emitters
            .iter()
            .map(|emitter| emitter.data.display_name())
            .collect();
        assert_eq!(names, vec!["Splash".to_string()], "{name}");
    }
}
