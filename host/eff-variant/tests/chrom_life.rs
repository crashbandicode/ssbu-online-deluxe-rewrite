use std::io::Cursor;

use eff_variant::{
    append_trace_life_sibling_in_place, SiblingRequest, VariantError, TRACE_LIFE_OFFSET,
    TRACE_RATE_OFFSET,
};
use effect_library::EmitterData;

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

fn version22_emitter(life: i32) -> Vec<u8> {
    let raw = vec![0u8; 65_536];
    let mut cursor = Cursor::new(raw.as_slice());
    let mut data = EmitterData::read(&mut cursor, 22).expect("version 22 emitter");
    let consumed = cursor.position() as usize;
    data.name = Some("trace1".to_owned());
    data.emission.rate = 1.0;
    data.particle_data.life = life;
    data.particle_data.life_random = 0;
    data.particle_data.infinite_life = false;
    let written = data.write(22).expect("write emitter");
    assert_eq!(written.len(), consumed, "reader and writer disagree");
    written
}

fn file_with(payload: &[u8], version: u16) -> (Vec<u8>, std::ops::Range<usize>) {
    let binary = 0x40u32;
    let size = binary + payload.len() as u32;
    let mut node = header(
        b"EMTR",
        size,
        0xffff_ffff,
        0xffff_ffff,
        0xffff_ffff,
        binary,
        0,
    )
    .to_vec();
    node.resize(binary as usize, 0);
    node.extend_from_slice(payload);
    let mut name_block = vec![0u8; 104];
    // `P_ChromMC1` is 10 bytes, so the name occupies 16..26.
    name_block[16..26].copy_from_slice(b"P_ChromMC1");
    name_block[80..84].copy_from_slice(&1u32.to_le_bytes());
    let eset_size = (32 + name_block.len() + node.len()) as u32;
    let mut eset = header(b"ESET", eset_size, 0x88, 0xffff_ffff, 0xffff_ffff, 0x20, 1).to_vec();
    eset.extend_from_slice(&name_block);
    let payload_at_in_eset = eset.len() + binary as usize;
    eset.extend_from_slice(&node);
    let esta_size = 0x20 + eset.len() as u32;
    let mut esta = header(b"ESTA", esta_size, 0x20, esta_size, 0xffff_ffff, 0, 1).to_vec();
    esta.extend_from_slice(&eset);
    let mut texture = header(b"GRTF", 0x30, 0xffff_ffff, 0xffff_ffff, 0xffff_ffff, 0, 0).to_vec();
    texture.extend_from_slice(&[0xA5; 16]);
    let mut vfx = vec![0u8; 0x20];
    vfx[0..4].copy_from_slice(b"VFXB");
    vfx[10..12].copy_from_slice(&version.to_le_bytes());
    vfx[12..14].copy_from_slice(&0xfeffu16.to_le_bytes());
    vfx[22..24].copy_from_slice(&0x20u16.to_le_bytes());
    vfx.extend_from_slice(&esta);
    vfx.extend_from_slice(&texture);
    while vfx.len() % 16 != 0 {
        vfx.push(0);
    }
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
    handle[4..8].copy_from_slice(&1i32.to_le_bytes());
    file.extend_from_slice(&handle);
    file.extend_from_slice(b"CHROM_MC_1\0");
    file.resize(0x1000, 0);
    let payload_at = 0x1000 + 0x40 + payload_at_in_eset;
    file.extend_from_slice(&vfx);
    (file, payload_at..payload_at + payload.len())
}

fn request() -> SiblingRequest<'static> {
    SiblingRequest {
        source_handle: "CHROM_MC_1",
        source_set: "P_ChromMC1",
        new_handle: "CHROM_MC_1_REDUCED",
        new_set: "P_ChromMC1Reduced",
        keep_emitters: &["trace1"],
    }
}

#[test]
fn library_writer_puts_life_at_the_version22_offset() {
    let three = version22_emitter(3);
    let two = version22_emitter(2);
    assert_eq!(three.len(), two.len());
    let mut diffs = Vec::new();
    for (index, (left, right)) in three.iter().zip(two.iter()).enumerate() {
        if left != right {
            diffs.push(index);
        }
    }
    assert!(diffs
        .iter()
        .all(|index| (TRACE_LIFE_OFFSET..TRACE_LIFE_OFFSET + 4).contains(index)));
    assert!(!diffs.is_empty());
    assert_eq!(
        &three[TRACE_LIFE_OFFSET..TRACE_LIFE_OFFSET + 4],
        &3i32.to_le_bytes()
    );
    assert_eq!(
        &two[TRACE_LIFE_OFFSET..TRACE_LIFE_OFFSET + 4],
        &2i32.to_le_bytes()
    );
    assert_eq!(
        f32::from_le_bytes(
            three[TRACE_RATE_OFFSET..TRACE_RATE_OFFSET + 4]
                .try_into()
                .unwrap()
        ),
        1.0
    );
}

#[test]
fn copied_sibling_changes_only_life_and_leaves_the_source() {
    let payload = version22_emitter(3);
    let (input, range) = file_with(&payload, 22);
    assert_eq!(&input[range.clone()], &payload);
    let mut buffer = input.clone();
    buffer.resize(input.len() + 2 * 1024 * 1024, 0);
    let new_len = append_trace_life_sibling_in_place(&mut buffer, input.len(), &request())
        .expect("life patch");
    assert!(new_len > input.len());
    assert_eq!(&buffer[range.clone()], &payload);
    let copied = copied_payload(&buffer[input.len()..new_len]);
    let mut expected = payload.clone();
    expected[TRACE_LIFE_OFFSET..TRACE_LIFE_OFFSET + 4].copy_from_slice(&2i32.to_le_bytes());
    assert_eq!(copied, expected);
    let source = EmitterData::read(&mut Cursor::new(&payload), 22).expect("source parser");
    let sibling =
        EmitterData::read(&mut Cursor::new(copied.as_slice()), 22).expect("sibling parser");
    assert_eq!(source.particle_data.life, 3);
    assert_eq!(sibling.particle_data.life, 2);
    assert_eq!(source.emission.rate, 1.0);
    assert_eq!(sibling.emission.rate, 1.0);
    assert!(sibling.particle_data.life > 0);
}

#[test]
fn unexpected_life_or_version_is_rejected_before_publication() {
    let mut payload = version22_emitter(3);
    payload[TRACE_LIFE_OFFSET..TRACE_LIFE_OFFSET + 4].copy_from_slice(&4i32.to_le_bytes());
    let (input, _) = file_with(&payload, 22);
    let mut buffer = input.clone();
    buffer.resize(input.len() + 1024 * 1024, 0);
    let error = append_trace_life_sibling_in_place(&mut buffer, input.len(), &request());
    assert_eq!(error, Err(VariantError::Unsupported("trace life scalar")));
    assert_eq!(&buffer[..input.len()], &input);

    let payload = version22_emitter(3);
    let (input, _) = file_with(&payload, 21);
    let mut buffer = input.clone();
    buffer.resize(input.len() + 1024 * 1024, 0);
    let error = append_trace_life_sibling_in_place(&mut buffer, input.len(), &request());
    assert_eq!(error, Err(VariantError::BadVersion));
    assert_eq!(&buffer[..input.len()], &input);
}

fn copied_payload(appended: &[u8]) -> Vec<u8> {
    let at = appended
        .windows(4)
        .position(|window| window == b"EMTR")
        .expect("copied emitter");
    let size = u32::from_le_bytes(appended[at + 4..at + 8].try_into().unwrap()) as usize;
    let binary = u32::from_le_bytes(appended[at + 20..at + 24].try_into().unwrap()) as usize;
    appended[at + binary..at + size].to_vec()
}

#[test]
#[ignore = "requires locally extracted stock Chrom EFF; no game assets are distributed"]
fn stock_chrom_changes_only_the_copied_life_field() {
    use sha2::{Digest, Sha256};
    let input = std::fs::read("/tmp/eff-probe/out/effect__fighter__chrom__ef_chrom.eff")
        .expect("extract the stock Chrom EFF before running this acceptance test");
    assert_eq!(
        format!("{:x}", Sha256::digest(&input)),
        "46a099edac555549e1f0b5272320c5c2b09b3cbfb429898456ea6e69a6997f98"
    );
    let untouched_copy = eff_variant::append_reduced_sibling(&input, &request()).unwrap();
    assert_eq!(untouched_copy.copied_payloads.len(), 1);
    let life_at = untouched_copy.copied_payloads[0] + TRACE_LIFE_OFFSET;
    let mut expected = untouched_copy.bytes;
    assert_eq!(&expected[life_at..life_at + 4], &3i32.to_le_bytes());
    expected[life_at..life_at + 4].copy_from_slice(&2i32.to_le_bytes());

    let mut buffer = input.clone();
    buffer.resize(ssbu_online_deluxe_rewrite::eff_variant::CHROM_MAX_BYTES, 0);
    let len = append_trace_life_sibling_in_place(&mut buffer, input.len(), &request()).unwrap();
    assert_eq!(&buffer[..len], &expected);
    for range in untouched_copy.source_emitter_payloads {
        assert_eq!(&buffer[range.clone()], &input[range]);
    }
    let parsed = effect_library::NamcoEffectFile::load(&buffer[..len]).unwrap();
    let sets = &parsed.ptcl_file.as_ref().unwrap().emitter_list.emitter_sets;
    for (name, life) in [("P_ChromMC1", 3), ("P_ChromMC1Reduced", 2)] {
        let set = sets.iter().find(|set| set.name == name).unwrap();
        assert_eq!(set.emitters.len(), 1);
        assert_eq!(set.emitters[0].data.particle_data.life, life);
        assert_eq!(set.emitters[0].data.emission.rate, 1.0);
        assert!(set.emitters[0].children.is_empty());
    }
}
