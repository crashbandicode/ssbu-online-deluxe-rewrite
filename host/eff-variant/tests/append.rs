use eff_variant::{
    append_reduced_sibling, append_reduced_sibling_in_place, append_reduced_siblings_in_place,
    SiblingRequest, VariantError,
};

fn header(
    magic: &[u8; 4],
    size: u32,
    child: u32,
    next: u32,
    attr: u32,
    binary: u32,
    count: u16,
) -> [u8; 32] {
    let mut raw = [0u8; 32];
    raw[0..4].copy_from_slice(magic);
    raw[4..8].copy_from_slice(&size.to_le_bytes());
    raw[8..12].copy_from_slice(&child.to_le_bytes());
    raw[12..16].copy_from_slice(&next.to_le_bytes());
    raw[16..20].copy_from_slice(&attr.to_le_bytes());
    raw[20..24].copy_from_slice(&binary.to_le_bytes());
    raw[28..30].copy_from_slice(&count.to_le_bytes());
    raw
}

fn emitter(name: &str, payload: u8) -> Vec<u8> {
    let size = 0x90u32;
    let mut node = header(b"EMTR", size, 0xffff_ffff, size, 0xffff_ffff, 0x40, 0).to_vec();
    node.resize(size as usize, payload);
    node[0x50..0x50 + 64].fill(0);
    node[0x50..0x50 + name.len()].copy_from_slice(name.as_bytes());
    node
}

fn tiny_file() -> Vec<u8> {
    let alpha = emitter("alpha", 0x11);
    let beta = emitter("beta", 0x22);
    let mut name_block = vec![0u8; 104];
    name_block[16..20].copy_from_slice(b"SET1");
    name_block[80..84].copy_from_slice(&2u32.to_le_bytes());
    let eset_size = (32 + name_block.len() + alpha.len() + beta.len()) as u32;
    let mut eset = header(b"ESET", eset_size, 0x88, 0xffff_ffff, 0xffff_ffff, 0x20, 2).to_vec();
    eset.extend_from_slice(&name_block);
    eset.extend_from_slice(&alpha);
    let beta_at = eset.len();
    eset.extend_from_slice(&beta);
    eset[beta_at + 12..beta_at + 16].copy_from_slice(&0xffff_ffffu32.to_le_bytes());

    let esta_size = 0x20 + eset.len() as u32;
    let mut esta = header(b"ESTA", esta_size, 0x20, esta_size, 0xffff_ffff, 0, 1).to_vec();
    esta.extend_from_slice(&eset);
    let mut texture = header(b"GRTF", 0x30, 0xffff_ffff, 0xffff_ffff, 0xffff_ffff, 0, 0).to_vec();
    texture.extend_from_slice(&[0xA5; 16]);

    let mut vfx = vec![0u8; 0x20];
    vfx[0..4].copy_from_slice(b"VFXB");
    vfx[10..12].copy_from_slice(&22u16.to_le_bytes());
    vfx[12..14].copy_from_slice(&0xfeffu16.to_le_bytes());
    vfx[22..24].copy_from_slice(&0x20u16.to_le_bytes());
    vfx.extend_from_slice(&esta);
    vfx.extend_from_slice(&texture);
    vfx.extend_from_slice(&[0xAB; 8]);
    let file_size = vfx.len() as u32;
    vfx[28..32].copy_from_slice(&file_size.to_le_bytes());

    let mut file = Vec::new();
    file.extend_from_slice(b"EFFN");
    file.extend_from_slice(&0x0002_0000u32.to_le_bytes());
    file.extend_from_slice(&1i16.to_le_bytes());
    file.extend_from_slice(&0i16.to_le_bytes());
    file.extend_from_slice(&0i16.to_le_bytes());
    file.extend_from_slice(&1i16.to_le_bytes());
    let mut handle = [0u8; 16];
    handle[0..4].copy_from_slice(&0x100u32.to_le_bytes());
    handle[4..8].copy_from_slice(&1i32.to_le_bytes());
    file.extend_from_slice(&handle);
    file.extend_from_slice(b"ONLY\0");
    file.resize(0x1000, 0);
    file.extend_from_slice(&vfx);
    assert_eq!(file.len() % 16, 0);
    file
}

fn alias_file() -> Vec<u8> {
    let stock = tiny_file();
    let mut file = Vec::new();
    file.extend_from_slice(b"EFFN");
    file.extend_from_slice(&0x0002_0000u32.to_le_bytes());
    file.extend_from_slice(&3i16.to_le_bytes());
    file.extend_from_slice(&0i16.to_le_bytes());
    file.extend_from_slice(&0i16.to_le_bytes());
    file.extend_from_slice(&1i16.to_le_bytes());
    let mut source = [0u8; 16];
    source.copy_from_slice(&stock[16..32]);
    let mut alias = source;
    alias[0] = 0x11;
    let mut none = source;
    none[0] = 0x22;
    none[4..8].copy_from_slice(&0i32.to_le_bytes());
    file.extend_from_slice(&source);
    file.extend_from_slice(&alias);
    file.extend_from_slice(&none);
    file.extend_from_slice(b"ONLY\0ALIAS\0NOSET\0");
    file.resize(0x1000, 0);
    file.extend_from_slice(&stock[0x1000..]);
    file
}

fn request<'a>(keep: &'a [&'a str]) -> SiblingRequest<'a> {
    SiblingRequest {
        source_handle: "ONLY",
        source_set: "SET1",
        new_handle: "ONLY_REDUCED",
        new_set: "P_SetReduced",
        keep_emitters: keep,
    }
}

#[test]
fn synthetic_append_keeps_original_bytes_except_documented_fields() {
    let input = tiny_file();
    let appended = append_reduced_sibling(&input, &request(&["alpha"])).expect("append");
    let output = &appended.bytes;
    assert!(output.len() > input.len());
    assert_eq!(
        &output[appended.appended_at..appended.appended_at + 4],
        b"ESET"
    );

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
                "unexpected change at {offset}"
            );
        }
    }
    assert_eq!(&input[16..32], &output[16..32]);
    let texture = input
        .windows(4)
        .position(|window| window == b"GRTF")
        .unwrap();
    assert_eq!(
        &input[texture..texture + 0x30],
        &output[texture..texture + 0x30]
    );
    assert_eq!(&output[input.len() - 8..input.len()], &[0xAB; 8]);
    assert!(output.len().is_multiple_of(16));
    assert_eq!(appended.copied_payloads.len(), 1);
    assert!(appended.copied_payloads[0].is_multiple_of(256));

    let parsed =
        eff_lib::EffFile::read(&mut std::io::Cursor::new(output.as_slice())).expect("eff_lib");
    assert_eq!(parsed.effect_handles.len(), 2);
    assert_eq!(parsed.effect_handle_names[0].to_string().unwrap(), "ONLY");
    assert_eq!(
        parsed.effect_handle_names[1].to_string().unwrap(),
        "ONLY_REDUCED"
    );
    assert_eq!(parsed.effect_handles[1].emitter_set_handle, 2);
    let resource = parsed.resource_data.expect("resource");
    assert_eq!(resource.len(), output.len() - 0x1000);
}

#[test]
fn missing_emitter_is_rejected() {
    let error = append_reduced_sibling(&tiny_file(), &request(&["missing"])).unwrap_err();
    assert_eq!(error, VariantError::Corrupt("kept emitter missing"));
}

#[test]
fn bad_magic_and_version_are_rejected() {
    let mut input = tiny_file();
    input[0] = b'X';
    assert_eq!(
        append_reduced_sibling(&input, &request(&["alpha"])).unwrap_err(),
        VariantError::BadMagic
    );
    let mut input = tiny_file();
    input[0x100A] = 21;
    assert_eq!(
        append_reduced_sibling(&input, &request(&["alpha"])).unwrap_err(),
        VariantError::BadVersion
    );
}

#[test]
fn truncated_input_is_rejected() {
    let error = append_reduced_sibling(b"EF", &request(&["alpha"])).unwrap_err();
    assert_eq!(error, VariantError::Truncated);
}

#[test]
fn copied_emitter_keeps_relative_child_and_attr() {
    let mut input = tiny_file();
    let at = input
        .windows(4)
        .position(|window| window == b"EMTR")
        .unwrap();
    input[at + 8..at + 12].copy_from_slice(&0x20u32.to_le_bytes());
    input[at + 16..at + 20].copy_from_slice(&0x30u32.to_le_bytes());
    let appended = append_reduced_sibling(&input, &request(&["alpha"])).expect("append");
    assert!(appended.experimental_external_link);
    let emitter_at = appended.copied_emitters[0];
    let copied = &appended.bytes[emitter_at..emitter_at + 0x90];
    let source = &input[at..at + 0x90];
    assert_eq!(&copied[8..12], &0x20u32.to_le_bytes());
    assert_eq!(&copied[16..20], &0x30u32.to_le_bytes());
    assert_eq!(&copied[..12], &source[..12]);
    assert_eq!(&copied[16..], &source[16..]);
}

#[test]
fn both_kept_payloads_are_256_aligned_and_match() {
    let input = tiny_file();
    let appended = append_reduced_sibling(&input, &request(&["alpha", "beta"])).expect("append");
    assert_eq!(appended.copied_payloads.len(), 2);
    assert!(appended.bytes.len().is_multiple_of(16));
    for (payload_at, range) in appended
        .copied_payloads
        .iter()
        .zip(&appended.source_emitter_payloads)
    {
        assert!(payload_at.is_multiple_of(256), "{payload_at:#x}");
        let len = range.end - range.start;
        assert_eq!(
            &appended.bytes[*payload_at..*payload_at + len],
            &input[range.clone()]
        );
    }
    let mut staged = vec![0u8; appended.bytes.len()];
    staged[..input.len()].copy_from_slice(&input);
    let new_len =
        append_reduced_sibling_in_place(&mut staged, input.len(), &request(&["alpha", "beta"]))
            .expect("in place");
    assert_eq!(new_len, appended.bytes.len());
    assert_eq!(&staged[..new_len], appended.bytes.as_slice());
    let mut tight = input.clone();
    let tight_len = tight.len();
    assert_eq!(
        append_reduced_sibling_in_place(&mut tight, tight_len, &request(&["alpha"])).unwrap_err(),
        VariantError::Overflow
    );
    assert_eq!(tight, input);
}

#[test]
fn in_place_chain_matches_one_file_per_sibling_without_a_second_copy() {
    let input = tiny_file();
    let first = request(&["alpha"]);
    let source_handle = first.source_handle;
    let source_set = first.source_set;
    let second = SiblingRequest {
        source_handle,
        source_set,
        new_handle: "ONLY_B",
        new_set: "P_SetB",
        keep_emitters: &["beta"],
    };
    let once = append_reduced_sibling(&input, &first).expect("first");
    let twice = append_reduced_sibling(&once.bytes, &second).expect("second");
    let mut buffer = vec![0u8; twice.bytes.len()];
    buffer[..input.len()].copy_from_slice(&input);
    let new_len = append_reduced_siblings_in_place(&mut buffer, input.len(), &[first, second])
        .expect("chain");
    assert_eq!(new_len, twice.bytes.len());
    assert_eq!(&buffer[..new_len], twice.bytes.as_slice());
    let mut broken = buffer.clone();
    let missing = SiblingRequest {
        source_handle,
        source_set,
        new_handle: "ONLY_C",
        new_set: "P_SetC",
        keep_emitters: &["missing"],
    };
    assert_eq!(
        append_reduced_siblings_in_place(&mut broken, new_len, &[missing]).unwrap_err(),
        VariantError::Corrupt("kept emitter missing")
    );
}

#[test]
fn handle_count_is_independent_of_the_emitter_set_index() {
    let input = alias_file();
    let appended = append_reduced_sibling(&input, &request(&["alpha"])).expect("append");
    let output = &appended.bytes;
    assert_eq!(i16::from_le_bytes(output[8..10].try_into().unwrap()), 4);
    assert_eq!(
        u16::from_le_bytes(
            output[appended.esta_count_at..appended.esta_count_at + 2]
                .try_into()
                .unwrap()
        ),
        2
    );
    assert_eq!(&output[16..64], &input[16..64]);
    assert_eq!(&output[68..72], &2i32.to_le_bytes());
    let parsed =
        eff_lib::EffFile::read(&mut std::io::Cursor::new(output.as_slice())).expect("eff_lib");
    assert_eq!(parsed.effect_handles.len(), 4);
    assert_eq!(parsed.effect_handles[0].emitter_set_handle, 1);
    assert_eq!(parsed.effect_handles[1].emitter_set_handle, 1);
    assert_eq!(parsed.effect_handles[2].emitter_set_handle, 0);
    assert_eq!(parsed.effect_handles[3].emitter_set_handle, 2);
    assert_eq!(parsed.effect_handle_names[1].to_string().unwrap(), "ALIAS");
    assert_eq!(
        parsed.effect_handle_names[3].to_string().unwrap(),
        "ONLY_REDUCED"
    );
}

#[test]
fn a_handle_pointing_past_the_set_list_is_rejected() {
    let mut input = tiny_file();
    input[20..24].copy_from_slice(&5i32.to_le_bytes());
    assert_eq!(
        append_reduced_sibling(&input, &request(&["alpha"])).unwrap_err(),
        VariantError::Corrupt("handle set index out of range")
    );
}

#[test]
fn nested_link_past_the_emitter_is_rejected() {
    let mut input = tiny_file();
    let at = input
        .windows(4)
        .position(|window| window == b"EMTR")
        .unwrap();
    input[at + 8..at + 12].copy_from_slice(&0x90u32.to_le_bytes());
    let error = append_reduced_sibling(&input, &request(&["alpha"])).unwrap_err();
    assert_eq!(error, VariantError::Corrupt("nested link outside emitter"));
}
