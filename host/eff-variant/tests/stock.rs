use std::fs;
use std::io::Cursor;

use eff_variant::{append_reduced_sibling, SiblingRequest};
use effect_library::NamcoEffectFile;

const STOCK: &str = "/tmp/eff-probe/out/effect__fighter__palutena__ef_palutena.eff";
const STOCK_SHA: &str = "ab2274dbafaac8f9f82b5666299111c25f0acabab449faa39500223eee60d9a5";

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn names_of(set: &effect_library::EmitterSet) -> Vec<String> {
    set.emitters
        .iter()
        .map(|emitter| emitter.data.display_name())
        .collect()
}

#[test]
#[ignore = "requires locally extracted stock EFF; no game assets are distributed"]
fn stock_palutena_sibling_preserves_original_resources() {
    let input = fs::read(STOCK).expect("read stock");
    assert_eq!(sha256(&input), STOCK_SHA);

    let appended = append_reduced_sibling(
        &input,
        &SiblingRequest {
            source_handle: "PALUTENA_BACKLIGHT2",
            source_set: "P_PalutenaBackLight2",
            new_handle: "PALUTENA_BACKLIGHT2_REDUCED",
            new_set: "P_PalutenaBackLight2Reduced",
            keep_emitters: &["backlight2_alpha", "backlight2_add"],
        },
    )
    .expect("append");
    assert!(appended.experimental_external_link);
    assert!(appended.bytes.len() <= 13 * 1024 * 1024);
    let output = &appended.bytes;

    let mut allowed = vec![
        appended.esta_count_at,
        appended.esta_count_at + 1,
        appended.last_link_at,
        appended.last_link_at + 1,
        appended.last_link_at + 2,
        appended.last_link_at + 3,
        appended.resource_at + 28,
        appended.resource_at + 29,
        appended.resource_at + 30,
        appended.resource_at + 31,
    ];
    allowed.sort_unstable();
    for offset in appended.resource_at..input.len() {
        if input[offset] != output[offset] {
            assert!(
                allowed.binary_search(&offset).is_ok(),
                "resource byte changed at {offset:#x}"
            );
        }
    }
    for range in &appended.source_emitter_payloads {
        assert_eq!(&input[range.clone()], &output[range.clone()]);
    }
    assert_eq!(output.len(), 12_314_816);
    assert_eq!(appended.copied_payloads, vec![0x00BB_D300, 0x00BB_DE00]);
    assert_eq!(appended.copied_payloads.len(), 2);
    assert!(output.len().is_multiple_of(16));
    for (payload_at, range) in appended
        .copied_payloads
        .iter()
        .zip(&appended.source_emitter_payloads)
    {
        assert!(payload_at.is_multiple_of(256), "{payload_at:#x}");
        let len = range.end - range.start;
        assert_eq!(
            &output[*payload_at..*payload_at + len],
            &input[range.clone()]
        );
    }

    let parsed = eff_lib::EffFile::read(&mut Cursor::new(output.as_slice())).expect("eff_lib");
    assert_eq!(parsed.effect_handles.len(), 68);
    assert_eq!(
        parsed.effect_handle_names[11].to_string().unwrap(),
        "PALUTENA_BACKLIGHT2"
    );
    assert_eq!(parsed.effect_handles[11].emitter_set_handle, 12);
    assert_eq!(
        parsed.effect_handle_names[67].to_string().unwrap(),
        "PALUTENA_BACKLIGHT2_REDUCED"
    );
    assert_eq!(parsed.effect_handles[67].emitter_set_handle, 68);

    let original = NamcoEffectFile::load(&input).expect("oracle original");
    let variant = NamcoEffectFile::load(output).expect("oracle variant");
    let original_ptcl = original.ptcl_file.as_ref().expect("original ptcl");
    let variant_ptcl = variant.ptcl_file.as_ref().expect("variant ptcl");
    let original_set = original_ptcl
        .emitter_list
        .emitter_sets
        .iter()
        .find(|set| set.name == "P_PalutenaBackLight2")
        .expect("original set");
    let variant_parent = variant_ptcl
        .emitter_list
        .emitter_sets
        .iter()
        .find(|set| set.name == "P_PalutenaBackLight2")
        .expect("parent set");
    let reduced = variant_ptcl
        .emitter_list
        .emitter_sets
        .iter()
        .find(|set| set.name == "P_PalutenaBackLight2Reduced")
        .expect("reduced set");
    assert_eq!(names_of(original_set), names_of(variant_parent));
    assert_eq!(
        names_of(reduced),
        vec!["backlight2_alpha".to_string(), "backlight2_add".to_string()]
    );
    assert_eq!(variant.entry_names.len(), 68);
    assert_eq!(variant.entries[67].emitter_set_id, 68);

    let original_texture = original_ptcl.texture_info.as_ref().expect("texture");
    let variant_texture = variant_ptcl.texture_info.as_ref().expect("variant texture");
    assert_eq!(
        original_texture.section_offset,
        variant_texture.section_offset
    );
    assert_eq!(original_texture.binary_data, variant_texture.binary_data);
    assert_eq!(
        original_texture.descriptors.len(),
        variant_texture.descriptors.len()
    );
    for (left, right) in original_texture
        .descriptors
        .iter()
        .zip(&variant_texture.descriptors)
    {
        assert_eq!(left.id, right.id);
        assert_eq!(left.name, right.name);
    }
    assert_eq!(
        original_ptcl
            .primitive_info
            .as_ref()
            .map(|info| info.binary_data.clone()),
        variant_ptcl
            .primitive_info
            .as_ref()
            .map(|info| info.binary_data.clone())
    );
}
