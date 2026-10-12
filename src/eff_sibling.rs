//! Byte-preserving EFF sibling append. Dependency-free so it can ship in the NRO.
//!
//! The particle blob stays at its original alignment. Texture and primitive
//! sections are not rewritten. A new handle and a new emitter set are added by
//! copying selected EMTR nodes and patching documented counts and link fields.
//!
//! The new emitter set is linked from the last ESET and appended past the
//! original `ESTA.size`. Host parsers follow that link. The game has not been
//! shown to, so [`AppendedSibling::experimental_external_link`] stays set.
//! Copied EMTR child and attribute offsets stay relative to the copied node.
//! Only `next` is rewritten, and it may skip zero padding. Each copied payload
//! is placed so `(node_at + binary) % 256 == 0`, matching the stock file.
//! The finished file stays 16-byte aligned. A link that is not `NONE` and not
//! strictly inside the node is rejected. Malformed input and allocation
//! failure return [`VariantError`] instead of panicking.

use std::ops::Range;

const EFF_VERSION: u32 = 0x0002_0000;
const VFX_VERSION: u16 = 22;
const NONE: u32 = 0xffff_ffff;
const NODE_HEADER: usize = 32;
const ESET_NAME_BLOCK_MIN: usize = 104;
const EMITTER_NAME_BIAS: usize = 16;
const FIXED_NAME: usize = 64;
const PAYLOAD_ALIGN: usize = 256;
/// Version-22 `EmitterData` offset of `ParticleData.life`.
/// effect_library 1.1.4 reads and writes the same prefix: 80-byte header
/// and name, 1792-byte `EmitterStatic`, 136-byte `EmitterInfo`, 24-byte
/// `EmitterInheritance`, 72-byte `Emission`, 96-byte `EmitterShapeInfo`,
/// 16-byte `EmitterRenderState`, then 16 flag bytes. `Emission.rate` is
/// the f32 at [`TRACE_RATE_OFFSET`].
pub const TRACE_LIFE_OFFSET: usize = 2232;
/// Version-22 `EmitterData` offset of `Emission.rate`.
pub const TRACE_RATE_OFFSET: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiblingRequest<'a> {
    pub source_handle: &'a str,
    pub source_set: &'a str,
    pub new_handle: &'a str,
    pub new_set: &'a str,
    pub keep_emitters: &'a [&'a str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendedSibling {
    pub bytes: Vec<u8>,
    pub resource_at: usize,
    pub esta_count_at: usize,
    pub last_link_at: usize,
    pub appended_at: usize,
    pub source_emitter_payloads: Vec<Range<usize>>,
    /// File offset of each copied EMTR header.
    pub copied_emitters: Vec<usize>,
    /// File offset of each copied payload (`emitter + binary`).
    pub copied_payloads: Vec<usize>,
    /// True when the appended set begins at or past the original ESTA end.
    /// The link is still written. Runtime use stays experimental.
    pub experimental_external_link: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantError {
    Truncated,
    BadMagic,
    BadVersion,
    Unsupported(&'static str),
    Corrupt(&'static str),
    Overflow,
    OutOfMemory,
}

#[derive(Clone, Copy)]
struct Node {
    at: usize,
    size: u32,
    child: u32,
    next: u32,
    attr: u32,
    binary: u32,
    pad: u32,
    count: u16,
    unknown: u16,
}

pub fn append_reduced_sibling(
    input: &[u8],
    request: &SiblingRequest<'_>,
) -> Result<AppendedSibling, VariantError> {
    let prepared = plan_sibling(input, request)?;
    let output_len = input
        .len()
        .checked_add(prepared.body.len())
        .ok_or(VariantError::Overflow)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(output_len)
        .map_err(|_| VariantError::OutOfMemory)?;
    bytes.resize(output_len, 0);
    bytes[..input.len()].copy_from_slice(input);
    apply_in_place(&mut bytes, input.len(), &prepared, request)?;
    Ok(AppendedSibling {
        bytes,
        resource_at: prepared.resource_at,
        esta_count_at: prepared.esta_count_at,
        last_link_at: prepared.last_link_at,
        appended_at: prepared.appended_at,
        source_emitter_payloads: prepared.source_emitter_payloads,
        copied_emitters: prepared.copied_emitters,
        copied_payloads: prepared.copied_payloads,
        experimental_external_link: prepared.experimental_external_link,
    })
}

fn plan_sibling(input: &[u8], request: &SiblingRequest<'_>) -> Result<Prepared, VariantError> {
    if request.keep_emitters.is_empty()
        || request.new_handle.is_empty()
        || request.new_set.is_empty()
        || request.new_handle.as_bytes().contains(&0)
        || request.new_set.len() >= FIXED_NAME
    {
        return Err(VariantError::Unsupported("empty or overlong name"));
    }
    let header = read_eff_header(input)?;
    if header.models != 0 || header.groups != 0 {
        return Err(VariantError::Unsupported("model or group table"));
    }
    if header.factor <= 0 {
        return Err(VariantError::Unsupported("resource alignment"));
    }
    let resource_at = (header.factor as usize)
        .checked_mul(0x1000)
        .ok_or(VariantError::Overflow)?;
    if resource_at >= input.len() || input.len() - resource_at < NODE_HEADER {
        return Err(VariantError::Truncated);
    }
    let (handles, names, names_end) = read_names(input, &header, resource_at)?;
    if handles.len() != names.len() {
        return Err(VariantError::Corrupt("handle and name counts differ"));
    }
    if !input[names_end..resource_at].iter().all(|byte| *byte == 0) {
        return Err(VariantError::Unsupported("header slack is not empty"));
    }
    let source_index = names
        .iter()
        .position(|name| name == request.source_handle)
        .ok_or(VariantError::Corrupt("source handle missing"))?;
    if names.iter().any(|name| name == request.new_handle) {
        return Err(VariantError::Corrupt("variant handle already exists"));
    }
    let added_name = request.new_handle.len() + 1;
    let names_fit = names_end
        .checked_add(NODE_HEADER / 2)
        .and_then(|at| at.checked_add(added_name))
        .ok_or(VariantError::Overflow)?;
    if names_fit > resource_at {
        return Err(VariantError::Unsupported(
            "header slack cannot hold the new name",
        ));
    }

    let vfx = read_vfx(input, resource_at)?;
    let vfx_end = resource_at
        .checked_add(vfx.file_size as usize)
        .ok_or(VariantError::Overflow)?;
    if vfx_end != input.len() {
        return Err(VariantError::Corrupt("vfx size does not end the file"));
    }
    let top = walk_chain(input, vfx_end, resource_at + vfx.block_offset, 16)?;
    let esta = top.first().ok_or(VariantError::Corrupt("no sections"))?;
    if magic_of(input, esta.at)? != *b"ESTA" {
        return Err(VariantError::Corrupt("first section is not ESTA"));
    }
    // eff_lib::EffectHandle::emitter_set_handle is a positive 1-based index
    // into the ESTA list. The EFF handle count is a different field. Several
    // handles may share one index. Zero means the handle has no emitter set.
    let sets = walk_links(
        input,
        vfx_end,
        offset(esta, esta.child)?,
        esta.count as usize,
        Some(esta),
    )?;
    if sets.len() != esta.count as usize {
        return Err(VariantError::Corrupt("emitter set walk ended early"));
    }
    for handle in &handles {
        let set_id = le_i32(&handle[4..8])?;
        if set_id < 0 || set_id as usize > sets.len() {
            return Err(VariantError::Corrupt("handle set index out of range"));
        }
    }
    let mut source_set_index = None;
    for (index, node) in sets.iter().enumerate() {
        if fixed_name(input, node, EMITTER_NAME_BIAS)? == request.source_set {
            source_set_index = Some(index);
            break;
        }
    }
    let source_set_index =
        source_set_index.ok_or(VariantError::Corrupt("source emitter set missing"))?;
    let source = &sets[source_set_index];
    if magic_of(input, source.at)? != *b"ESET" {
        return Err(VariantError::Corrupt("source set magic"));
    }
    if source.attr != NONE {
        return Err(VariantError::Unsupported("emitter set attribute"));
    }
    let handle_set = le_i32(&handles[source_index][4..8])?;
    if handle_set != source_set_index as i32 + 1 {
        return Err(VariantError::Corrupt(
            "handle does not point at the named set",
        ));
    }
    let name_block_len = (source.child as usize)
        .checked_sub(source.binary as usize)
        .ok_or(VariantError::Corrupt("set name block"))?;
    if name_block_len < ESET_NAME_BLOCK_MIN || source.binary > source.size {
        return Err(VariantError::Corrupt("set name block"));
    }
    let emitters = walk_links(
        input,
        vfx_end,
        offset(source, source.child)?,
        source.count as usize,
        Some(source),
    )?;
    let mut kept = Vec::new();
    kept.try_reserve_exact(request.keep_emitters.len())
        .map_err(|_| VariantError::OutOfMemory)?;
    let mut cursor = 0usize;
    for wanted in request.keep_emitters {
        let mut found = None;
        for (offset, node) in emitters.iter().enumerate().skip(cursor) {
            if fixed_name(input, node, EMITTER_NAME_BIAS)? == *wanted {
                found = Some(offset);
                break;
            }
        }
        let Some(index) = found else {
            return Err(VariantError::Corrupt("kept emitter missing"));
        };
        let node = &emitters[index];
        if magic_of(input, node.at)? != *b"EMTR" {
            return Err(VariantError::Corrupt("kept node is not EMTR"));
        }
        let name_end = (node.binary as usize)
            .checked_add(EMITTER_NAME_BIAS + FIXED_NAME)
            .ok_or(VariantError::Overflow)?;
        if name_end > node.size as usize {
            return Err(VariantError::Corrupt("emitter name outside node"));
        }
        if !link_inside_node(node.child, node.size) || !link_inside_node(node.attr, node.size) {
            return Err(VariantError::Corrupt("nested link outside emitter"));
        }
        kept.push(*node);
        cursor = index + 1;
    }
    let payloads = kept
        .iter()
        .map(|node| {
            let start = node.at + node.binary as usize;
            start..node.at + node.size as usize
        })
        .collect();

    let name_block_at = source.at + source.binary as usize;
    let name_block_end = name_block_at + name_block_len;
    if name_block_end > source.at + source.size as usize {
        return Err(VariantError::Corrupt("set name block"));
    }
    let appended_at = input.len();
    let esta_end = esta
        .at
        .checked_add(esta.size as usize)
        .ok_or(VariantError::Overflow)?;
    let experimental_external_link = appended_at >= esta_end;
    if !appended_at.is_multiple_of(16) {
        return Err(VariantError::Unsupported("file end is not 16-byte aligned"));
    }
    let last = sets
        .last()
        .ok_or(VariantError::Corrupt("no emitter sets"))?;
    if last.next != NONE {
        return Err(VariantError::Corrupt("emitter set list has no end"));
    }
    let link = u32::try_from(
        appended_at
            .checked_sub(last.at)
            .ok_or(VariantError::Overflow)?,
    )
    .map_err(|_| VariantError::Overflow)?;
    if link == NONE {
        return Err(VariantError::Overflow);
    }
    let nested: u32 = kept.iter().map(|node| u32::from(node.count)).sum();
    let direct = u32::try_from(kept.len()).map_err(|_| VariantError::Overflow)?;
    let body = aligned_set(SetSource {
        input,
        kept: &kept,
        appended_at,
        name_block_at,
        name_block_len,
        new_set: request.new_set,
        emitter_count: direct + nested,
        pad: source.pad,
        unknown: source.unknown,
    })?;
    let new_file_size = (vfx.file_size as usize)
        .checked_add(body.bytes.len())
        .ok_or(VariantError::Overflow)?;
    let new_file_size = u32::try_from(new_file_size).map_err(|_| VariantError::Overflow)?;
    let new_handle_count = u16::try_from(handles.len())
        .ok()
        .and_then(|count| count.checked_add(1))
        .ok_or(VariantError::Overflow)?;
    if i16::try_from(new_handle_count).is_err() {
        return Err(VariantError::Overflow);
    }
    let new_set_count = esta.count.checked_add(1).ok_or(VariantError::Overflow)?;
    let handle_end = 16 + handles.len() * 16;
    let mut new_handle = handles[source_index];
    let new_set_id = i32::try_from(sets.len())
        .ok()
        .and_then(|count| count.checked_add(1))
        .ok_or(VariantError::Overflow)?;
    new_handle[4..8].copy_from_slice(&new_set_id.to_le_bytes());
    let prepared = Prepared {
        body: body.bytes,
        copied_emitters: body.emitters,
        copied_payloads: body.payloads,
        new_handle_count,
        new_set_count,
        new_handle,
        handle_end,
        names_end,
        resource_at,
        esta_count_at: esta.at + 28,
        last_link_at: last.at + 12,
        link,
        new_file_size,
        appended_at,
        experimental_external_link,
        source_emitter_payloads: payloads,
    };
    Ok(prepared)
}

/// Writes the sibling into `buffer[..input_len]` and the bytes that follow it.
/// The only allocation is the small appended set. A full-file copy is not made.
/// `buffer` is unchanged when this returns an error before the write; a write
/// that then fails still returns the error so the caller can discard it.
pub fn append_reduced_sibling_in_place(
    buffer: &mut [u8],
    input_len: usize,
    request: &SiblingRequest<'_>,
) -> Result<usize, VariantError> {
    let input = buffer.get(..input_len).ok_or(VariantError::Truncated)?;
    let prepared = plan_sibling(input, request)?;
    apply_in_place(buffer, input_len, &prepared, request)
}

/// Appends every request in order inside `buffer`. `input_len` advances after
/// each sibling. The only heap use is the small set body inside each plan.
/// A later error leaves earlier siblings in the buffer. The caller must not
/// publish that buffer.
pub fn append_reduced_siblings_in_place(
    buffer: &mut [u8],
    mut input_len: usize,
    requests: &[SiblingRequest<'_>],
) -> Result<usize, VariantError> {
    for request in requests {
        let new_len = append_reduced_sibling_in_place(buffer, input_len, request)?;
        if new_len < input_len || new_len > buffer.len() {
            return Err(VariantError::Overflow);
        }
        input_len = new_len;
    }
    Ok(input_len)
}

/// Copies one emitter, then changes only `ParticleData.life` from 3 to 2
/// on that copy. The source payload is not written. A failed guard returns
/// before [`apply_in_place`], so the caller's buffer stays unchanged.
pub fn append_trace_life_sibling_in_place(
    buffer: &mut [u8],
    input_len: usize,
    request: &SiblingRequest<'_>,
) -> Result<usize, VariantError> {
    let input = buffer.get(..input_len).ok_or(VariantError::Truncated)?;
    let mut prepared = plan_sibling(input, request)?;
    patch_trace_life(&mut prepared)?;
    apply_in_place(buffer, input_len, &prepared, request)
}

fn patch_trace_life(prepared: &mut Prepared) -> Result<(), VariantError> {
    if prepared.copied_emitters.len() != 1 || prepared.copied_payloads.len() != 1 {
        return Err(VariantError::Unsupported("trace life emitter count"));
    }
    let emitter_rel = prepared.copied_emitters[0]
        .checked_sub(prepared.appended_at)
        .ok_or(VariantError::Overflow)?;
    let payload_rel = prepared.copied_payloads[0]
        .checked_sub(prepared.appended_at)
        .ok_or(VariantError::Overflow)?;
    let header = prepared
        .body
        .get(emitter_rel..emitter_rel + NODE_HEADER)
        .ok_or(VariantError::Truncated)?;
    if &header[0..4] != b"EMTR" {
        return Err(VariantError::Corrupt("trace node is not EMTR"));
    }
    let size = u32::from_le_bytes(array4(&header[4..8])?);
    let child = u32::from_le_bytes(array4(&header[8..12])?);
    let binary = u32::from_le_bytes(array4(&header[20..24])?);
    let count = u16::from_le_bytes(array2(&header[28..30])?);
    if child != NONE || count != 0 {
        return Err(VariantError::Unsupported("trace life children"));
    }
    if payload_rel != emitter_rel + binary as usize {
        return Err(VariantError::Corrupt("trace payload placement"));
    }
    let payload_len = (size as usize)
        .checked_sub(binary as usize)
        .ok_or(VariantError::Overflow)?;
    let end = payload_rel
        .checked_add(payload_len)
        .ok_or(VariantError::Overflow)?;
    let payload = prepared
        .body
        .get_mut(payload_rel..end)
        .ok_or(VariantError::Truncated)?;
    if payload.len() < TRACE_LIFE_OFFSET + 8 {
        return Err(VariantError::Unsupported("trace life payload"));
    }
    let rate = f32::from_le_bytes(array4(&payload[TRACE_RATE_OFFSET..TRACE_RATE_OFFSET + 4])?);
    if rate != 1.0 {
        return Err(VariantError::Unsupported("trace life rate"));
    }
    if payload[TRACE_LIFE_OFFSET - 16] != 0 {
        return Err(VariantError::Unsupported("trace life infinite"));
    }
    let life = i32::from_le_bytes(array4(&payload[TRACE_LIFE_OFFSET..TRACE_LIFE_OFFSET + 4])?);
    let life_random = i32::from_le_bytes(array4(
        &payload[TRACE_LIFE_OFFSET + 4..TRACE_LIFE_OFFSET + 8],
    )?);
    if life != 3 || life_random != 0 {
        return Err(VariantError::Unsupported("trace life scalar"));
    }
    payload[TRACE_LIFE_OFFSET..TRACE_LIFE_OFFSET + 4].copy_from_slice(&2i32.to_le_bytes());
    Ok(())
}

fn array4(bytes: &[u8]) -> Result<[u8; 4], VariantError> {
    bytes.try_into().map_err(|_| VariantError::Truncated)
}

fn array2(bytes: &[u8]) -> Result<[u8; 2], VariantError> {
    bytes.try_into().map_err(|_| VariantError::Truncated)
}

struct Prepared {
    body: Vec<u8>,
    copied_emitters: Vec<usize>,
    copied_payloads: Vec<usize>,
    new_handle_count: u16,
    new_set_count: u16,
    new_handle: [u8; 16],
    handle_end: usize,
    names_end: usize,
    resource_at: usize,
    esta_count_at: usize,
    last_link_at: usize,
    link: u32,
    new_file_size: u32,
    appended_at: usize,
    experimental_external_link: bool,
    source_emitter_payloads: Vec<Range<usize>>,
}

struct AlignedSet {
    bytes: Vec<u8>,
    emitters: Vec<usize>,
    payloads: Vec<usize>,
}

fn apply_in_place(
    buffer: &mut [u8],
    input_len: usize,
    prepared: &Prepared,
    request: &SiblingRequest<'_>,
) -> Result<usize, VariantError> {
    if input_len != prepared.appended_at {
        return Err(VariantError::Corrupt("input length changed"));
    }
    let output_len = input_len
        .checked_add(prepared.body.len())
        .ok_or(VariantError::Overflow)?;
    if buffer.len() < output_len {
        return Err(VariantError::Overflow);
    }
    let name = request.new_handle.as_bytes();
    let name_at = prepared
        .names_end
        .checked_add(16)
        .ok_or(VariantError::Overflow)?;
    let name_end = name_at
        .checked_add(name.len() + 1)
        .ok_or(VariantError::Overflow)?;
    if name_end > prepared.resource_at || prepared.handle_end + 16 > prepared.resource_at {
        return Err(VariantError::Overflow);
    }
    buffer.copy_within(
        prepared.handle_end..prepared.names_end,
        prepared.handle_end + 16,
    );
    buffer[prepared.handle_end..prepared.handle_end + 16].copy_from_slice(&prepared.new_handle);
    buffer[name_at..name_at + name.len()].copy_from_slice(name);
    buffer[name_at + name.len()] = 0;
    buffer[8..10].copy_from_slice(&prepared.new_handle_count.to_le_bytes());
    buffer[input_len..output_len].copy_from_slice(&prepared.body);
    let count_at = prepared.esta_count_at;
    buffer[count_at..count_at + 2].copy_from_slice(&prepared.new_set_count.to_le_bytes());
    let link_at = prepared.last_link_at;
    buffer[link_at..link_at + 4].copy_from_slice(&prepared.link.to_le_bytes());
    let size_at = prepared.resource_at + 28;
    buffer[size_at..size_at + 4].copy_from_slice(&prepared.new_file_size.to_le_bytes());
    Ok(output_len)
}

fn payload_pad(cursor: usize, binary: u32) -> Result<usize, VariantError> {
    let origin = cursor
        .checked_add(binary as usize)
        .ok_or(VariantError::Overflow)?;
    Ok((PAYLOAD_ALIGN - (origin % PAYLOAD_ALIGN)) % PAYLOAD_ALIGN)
}

struct SetSource<'a> {
    input: &'a [u8],
    kept: &'a [Node],
    appended_at: usize,
    name_block_at: usize,
    name_block_len: usize,
    new_set: &'a str,
    emitter_count: u32,
    pad: u32,
    unknown: u16,
}

fn aligned_set(source: SetSource<'_>) -> Result<AlignedSet, VariantError> {
    let SetSource {
        input,
        kept,
        appended_at,
        name_block_at,
        name_block_len,
        new_set,
        emitter_count,
        pad,
        unknown,
    } = source;
    if kept.is_empty() || name_block_len < ESET_NAME_BLOCK_MIN {
        return Err(VariantError::Corrupt("set name block"));
    }
    let mut file_cursor = appended_at
        .checked_add(NODE_HEADER + name_block_len)
        .ok_or(VariantError::Overflow)?;
    let mut pads = Vec::new();
    pads.try_reserve_exact(kept.len())
        .map_err(|_| VariantError::OutOfMemory)?;
    for node in kept {
        let gap = payload_pad(file_cursor, node.binary)?;
        pads.push(gap);
        file_cursor = file_cursor
            .checked_add(gap)
            .and_then(|at| at.checked_add(node.size as usize))
            .ok_or(VariantError::Overflow)?;
    }
    let tail = (16 - (file_cursor % 16)) % 16;
    let body_end = file_cursor
        .checked_add(tail)
        .ok_or(VariantError::Overflow)?;
    let body_len = body_end
        .checked_sub(appended_at)
        .ok_or(VariantError::Overflow)?;
    if !body_len.is_multiple_of(16) {
        return Err(VariantError::Corrupt("constructed set size"));
    }
    let body_size = u32::try_from(body_len).map_err(|_| VariantError::Overflow)?;
    let child_at = NODE_HEADER
        .checked_add(name_block_len)
        .and_then(|at| at.checked_add(pads[0]))
        .ok_or(VariantError::Overflow)?;
    let child = u32::try_from(child_at).map_err(|_| VariantError::Overflow)?;
    if child == NONE {
        return Err(VariantError::Overflow);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(body_len)
        .map_err(|_| VariantError::OutOfMemory)?;
    bytes.resize(body_len, 0);
    bytes[0..4].copy_from_slice(b"ESET");
    bytes[4..8].copy_from_slice(&body_size.to_le_bytes());
    bytes[8..12].copy_from_slice(&child.to_le_bytes());
    bytes[12..16].copy_from_slice(&NONE.to_le_bytes());
    bytes[16..20].copy_from_slice(&NONE.to_le_bytes());
    bytes[20..24].copy_from_slice(&(NODE_HEADER as u32).to_le_bytes());
    bytes[24..28].copy_from_slice(&pad.to_le_bytes());
    bytes[28..30].copy_from_slice(&(kept.len() as u16).to_le_bytes());
    bytes[30..32].copy_from_slice(&unknown.to_le_bytes());
    let name_src = input
        .get(name_block_at..name_block_at + name_block_len)
        .ok_or(VariantError::Truncated)?;
    bytes[NODE_HEADER..NODE_HEADER + name_block_len].copy_from_slice(name_src);
    bytes[NODE_HEADER + 16..NODE_HEADER + 16 + FIXED_NAME].fill(0);
    bytes[NODE_HEADER + 16..NODE_HEADER + 16 + new_set.len()].copy_from_slice(new_set.as_bytes());
    bytes[NODE_HEADER + 80..NODE_HEADER + 84].copy_from_slice(&emitter_count.to_le_bytes());

    let mut emitters = Vec::new();
    let mut payloads = Vec::new();
    emitters
        .try_reserve_exact(kept.len())
        .map_err(|_| VariantError::OutOfMemory)?;
    payloads
        .try_reserve_exact(kept.len())
        .map_err(|_| VariantError::OutOfMemory)?;
    let mut rel = NODE_HEADER + name_block_len;
    for (index, node) in kept.iter().enumerate() {
        rel = rel.checked_add(pads[index]).ok_or(VariantError::Overflow)?;
        let node_len = node.size as usize;
        let src_end = node
            .at
            .checked_add(node_len)
            .ok_or(VariantError::Overflow)?;
        let src = input.get(node.at..src_end).ok_or(VariantError::Truncated)?;
        let dst = bytes
            .get_mut(rel..rel + node_len)
            .ok_or(VariantError::Corrupt("constructed set size"))?;
        dst.copy_from_slice(src);
        let next = if index + 1 == kept.len() {
            NONE
        } else {
            u32::try_from(node_len + pads[index + 1]).map_err(|_| VariantError::Overflow)?
        };
        dst[12..16].copy_from_slice(&next.to_le_bytes());
        let emitter_at = appended_at + rel;
        let payload_at = emitter_at
            .checked_add(node.binary as usize)
            .ok_or(VariantError::Overflow)?;
        if !payload_at.is_multiple_of(PAYLOAD_ALIGN) {
            return Err(VariantError::Corrupt("payload alignment"));
        }
        emitters.push(emitter_at);
        payloads.push(payload_at);
        rel = rel.checked_add(node_len).ok_or(VariantError::Overflow)?;
    }
    if rel.checked_add(tail) != Some(body_len) {
        return Err(VariantError::Corrupt("constructed set size"));
    }
    Ok(AlignedSet {
        bytes,
        emitters,
        payloads,
    })
}

struct EffHeader {
    handles: i16,
    models: i16,
    groups: i16,
    factor: i16,
}

struct VfxHeader {
    file_size: u32,
    block_offset: usize,
}

fn read_eff_header(input: &[u8]) -> Result<EffHeader, VariantError> {
    if input.len() < 16 || &input[0..4] != b"EFFN" {
        return Err(if input.len() < 4 {
            VariantError::Truncated
        } else {
            VariantError::BadMagic
        });
    }
    let version = le_u32(&input[4..8])?;
    if version != EFF_VERSION {
        return Err(VariantError::BadVersion);
    }
    let handles = le_i16(&input[8..10])?;
    let models = le_i16(&input[10..12])?;
    let groups = le_i16(&input[12..14])?;
    let factor = le_i16(&input[14..16])?;
    if handles <= 0 {
        return Err(VariantError::Corrupt("handle count"));
    }
    Ok(EffHeader {
        handles,
        models,
        groups,
        factor,
    })
}

type HandleTable = Vec<[u8; 16]>;

fn read_names(
    input: &[u8],
    header: &EffHeader,
    resource_at: usize,
) -> Result<(HandleTable, Vec<String>, usize), VariantError> {
    let count = header.handles as usize;
    let handle_end = 16usize
        .checked_add(count.checked_mul(16).ok_or(VariantError::Overflow)?)
        .ok_or(VariantError::Overflow)?;
    if handle_end > resource_at {
        return Err(VariantError::Truncated);
    }
    let mut handles = Vec::with_capacity(count);
    for index in 0..count {
        let at = 16 + index * 16;
        handles.push(array16(
            input.get(at..at + 16).ok_or(VariantError::Truncated)?,
        )?);
    }
    let mut names = Vec::with_capacity(count);
    let mut cursor = handle_end;
    for _ in 0..count {
        let end = input[cursor..resource_at]
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(VariantError::Corrupt("unterminated handle name"))?;
        let name = std::str::from_utf8(&input[cursor..cursor + end])
            .map_err(|_| VariantError::Corrupt("handle name is not utf-8"))?
            .to_string();
        names.push(name);
        cursor = cursor + end + 1;
    }
    Ok((handles, names, cursor))
}

fn read_vfx(input: &[u8], resource_at: usize) -> Result<VfxHeader, VariantError> {
    let header = input
        .get(resource_at..resource_at + NODE_HEADER)
        .ok_or(VariantError::Truncated)?;
    if &header[0..4] != b"VFXB" {
        return Err(VariantError::BadMagic);
    }
    let version = le_u16(&header[10..12])?;
    if version != VFX_VERSION {
        return Err(VariantError::BadVersion);
    }
    let order = le_u16(&header[12..14])?;
    if order != 0xfeff {
        return Err(VariantError::Unsupported("byte order"));
    }
    let block = le_u16(&header[22..24])? as usize;
    let file_size = le_u32(&header[28..32])?;
    if block < NODE_HEADER || block as u32 >= file_size {
        return Err(VariantError::Corrupt("vfx block offset"));
    }
    Ok(VfxHeader {
        file_size,
        block_offset: block,
    })
}

fn walk_links(
    input: &[u8],
    vfx_end: usize,
    start: usize,
    expected: usize,
    parent: Option<&Node>,
) -> Result<Vec<Node>, VariantError> {
    if expected == 0 {
        return Err(VariantError::Corrupt("empty link list"));
    }
    let mut nodes = Vec::with_capacity(expected);
    let mut at = start;
    for index in 0..expected {
        let node = read_node(input, vfx_end, at)?;
        if let Some(parent) = parent {
            contain_child(input, parent, &node, vfx_end)?;
        }
        let finished = node.next == NONE;
        let next_rel = node.next;
        let node_at = node.at;
        nodes.push(node);
        if finished {
            if index + 1 != expected {
                return Err(VariantError::Corrupt("link list ended early"));
            }
            break;
        }
        if index + 1 == expected {
            return Err(VariantError::Corrupt("link list longer than count"));
        }
        at = node_at
            .checked_add(next_rel as usize)
            .ok_or(VariantError::Overflow)?;
    }
    Ok(nodes)
}

fn read_node(input: &[u8], vfx_end: usize, at: usize) -> Result<Node, VariantError> {
    let raw = input
        .get(at..at + NODE_HEADER)
        .ok_or(VariantError::Truncated)?;
    let size = le_u32(&raw[4..8])?;
    let node = Node {
        at,
        size,
        child: le_u32(&raw[8..12])?,
        next: le_u32(&raw[12..16])?,
        attr: le_u32(&raw[16..20])?,
        binary: le_u32(&raw[20..24])?,
        pad: le_u32(&raw[24..28])?,
        count: le_u16(&raw[28..30])?,
        unknown: le_u16(&raw[30..32])?,
    };
    if size != 0 {
        let end = at
            .checked_add(size as usize)
            .ok_or(VariantError::Overflow)?;
        if end > vfx_end {
            return Err(VariantError::Corrupt("node exceeds vfx"));
        }
    }
    Ok(node)
}

fn contain_child(
    input: &[u8],
    parent: &Node,
    node: &Node,
    vfx_end: usize,
) -> Result<(), VariantError> {
    let parent_end = parent
        .at
        .checked_add(parent.size as usize)
        .ok_or(VariantError::Overflow)?;
    let node_end = node
        .at
        .checked_add(node.size as usize)
        .ok_or(VariantError::Overflow)?;
    if parent.size == 0 || node.at < parent.at || node_end > vfx_end {
        return Err(VariantError::Corrupt("child outside parent"));
    }
    if node_end <= parent_end {
        return Ok(());
    }
    // A reduced sibling is linked from the last ESET and stored past ESTA.size.
    // Later siblings have to walk that tail. Emitters stay inside their set.
    if magic_of(input, parent.at)? == *b"ESTA" && node.at >= parent_end {
        return Ok(());
    }
    Err(VariantError::Corrupt("child outside parent"))
}

fn offset(node: &Node, relative: u32) -> Result<usize, VariantError> {
    if relative == NONE {
        return Err(VariantError::Corrupt("missing child offset"));
    }
    node.at
        .checked_add(relative as usize)
        .ok_or(VariantError::Overflow)
}

fn magic_of(input: &[u8], at: usize) -> Result<[u8; 4], VariantError> {
    input
        .get(at..at + 4)
        .ok_or(VariantError::Truncated)?
        .try_into()
        .map_err(|_| VariantError::Truncated)
}

fn fixed_name(input: &[u8], node: &Node, bias: usize) -> Result<String, VariantError> {
    let start = node
        .at
        .checked_add(node.binary as usize)
        .and_then(|at| at.checked_add(bias))
        .ok_or(VariantError::Overflow)?;
    let end = start
        .checked_add(FIXED_NAME)
        .ok_or(VariantError::Overflow)?;
    let raw = input.get(start..end).ok_or(VariantError::Truncated)?;
    let text = match raw.iter().position(|byte| *byte == 0) {
        Some(end) => &raw[..end],
        None => raw,
    };
    std::str::from_utf8(text)
        .map(|value| value.to_string())
        .map_err(|_| VariantError::Corrupt("fixed name is not utf-8"))
}

fn link_inside_node(relative: u32, size: u32) -> bool {
    relative == NONE || ((relative as usize) >= NODE_HEADER && relative < size)
}

fn le_u32(bytes: &[u8]) -> Result<u32, VariantError> {
    let mut raw = [0u8; 4];
    raw.copy_from_slice(bytes.get(..4).ok_or(VariantError::Truncated)?);
    Ok(u32::from_le_bytes(raw))
}

fn le_u16(bytes: &[u8]) -> Result<u16, VariantError> {
    let mut raw = [0u8; 2];
    raw.copy_from_slice(bytes.get(..2).ok_or(VariantError::Truncated)?);
    Ok(u16::from_le_bytes(raw))
}

fn le_i16(bytes: &[u8]) -> Result<i16, VariantError> {
    let mut raw = [0u8; 2];
    raw.copy_from_slice(bytes.get(..2).ok_or(VariantError::Truncated)?);
    Ok(i16::from_le_bytes(raw))
}

fn le_i32(bytes: &[u8]) -> Result<i32, VariantError> {
    let mut raw = [0u8; 4];
    raw.copy_from_slice(bytes.get(..4).ok_or(VariantError::Truncated)?);
    Ok(i32::from_le_bytes(raw))
}

fn array16(bytes: &[u8]) -> Result<[u8; 16], VariantError> {
    let mut raw = [0u8; 16];
    raw.copy_from_slice(bytes.get(..16).ok_or(VariantError::Truncated)?);
    Ok(raw)
}

fn walk_chain(
    input: &[u8],
    vfx_end: usize,
    start: usize,
    max_nodes: usize,
) -> Result<Vec<Node>, VariantError> {
    let mut nodes = Vec::new();
    let mut at = start;
    for _ in 0..max_nodes {
        let node = read_node(input, vfx_end, at)?;
        let next = node.next;
        nodes.push(node);
        if next == NONE {
            return Ok(nodes);
        }
        at = node
            .at
            .checked_add(next as usize)
            .ok_or(VariantError::Overflow)?;
        if at >= vfx_end {
            return Err(VariantError::Corrupt("section link leaves the vfx"));
        }
    }
    Err(VariantError::Corrupt("too many sections"))
}
