#![cfg_attr(not(target_os = "switch"), allow(dead_code))]

//! Pure choice between the embedded ImGui backend and a standalone
//! `libimgui_smash.nro`. Symbol lookup stays in the Switch host. This module
//! parses an NRO header's text and ro extents and decides what those facts mean.
//!
//! `imgui_get_context_export` is optional. The standalone NRO validated for
//! Elite Legal Stages exports the setup and draw-wrapper symbols and the
//! version tag below, and does not export `imgui_get_context_export`.

/// Bytes of `IMGUI_VERSION` plus `IMGUI_VERSION_NUM` as printed by the pinned
/// cimgui (`1.92.7 WIP` / 19263). The standalone NRO and `libimgui_smash.a`
/// both contain this exact sequence.
pub const COMPATIBLE_IMGUI_VERSION: &[u8] = b"1.92.7 WIP (19263)";

/// File extents from Skyline's `nn::ro::NroHeader` (`include/nn/ro.h`).
/// `text_offset == 0` means the text mapping begins at the header, which is
/// the layout of the Elite Legal Stages NRO. A non-zero text offset is parsed
/// but cannot be tied to a mapping base without another anchor, so the host
/// refuses it instead of scanning nearby mappings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NroSegments {
    pub size: u32,
    pub text_offset: u32,
    pub text_size: u32,
    pub ro_offset: u32,
    pub ro_size: u32,
}

pub fn parse_nro_header(header: &[u8]) -> Option<NroSegments> {
    if header.len() < 0x80 || header[0x10..0x14] != *b"NRO0" {
        return None;
    }
    let u32_at = |offset: usize| {
        let bytes: [u8; 4] = header[offset..offset + 4].try_into().ok()?;
        Some(u32::from_le_bytes(bytes))
    };
    let size = u32_at(0x18)?;
    let text_offset = u32_at(0x20)?;
    let text_size = u32_at(0x24)?;
    let ro_offset = u32_at(0x28)?;
    let ro_size = u32_at(0x2c)?;
    let rw_offset = u32_at(0x30)?;
    let rw_size = u32_at(0x34)?;
    let inside = |offset: u32, len: u32| {
        len > 0 && offset.checked_add(len).is_some_and(|end| end <= size)
    };
    if size < 0x80 || !inside(text_offset, text_size) || !inside(ro_offset, ro_size) {
        return None;
    }
    if rw_size != 0 && !inside(rw_offset, rw_size) {
        return None;
    }
    Some(NroSegments {
        size,
        text_offset,
        text_size,
        ro_offset,
        ro_size,
    })
}

pub fn text_contains(segments: &NroSegments, offset_from_base: usize) -> bool {
    let Ok(offset) = u32::try_from(offset_from_base) else {
        return false;
    };
    offset >= segments.text_offset
        && offset.saturating_sub(segments.text_offset) < segments.text_size
}

pub fn ro_contains_tag(ro: &[u8]) -> bool {
    ro.len() >= COMPATIBLE_IMGUI_VERSION.len()
        && ro
            .windows(COMPATIBLE_IMGUI_VERSION.len())
            .any(|window| window == COMPATIBLE_IMGUI_VERSION)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImguiHostChoice {
    /// No standalone host exports. This NRO owns init and the NVN backend.
    Embedded,
    /// Standalone host is present and matches the pinned ImGui layout.
    /// Do not call this NRO's `imgui_smash_init` or its bootstrap renderer.
    External,
    /// Some standalone export is present, but the pair or the version tag is
    /// not the known ABI. Do not init either backend and do not call `ig*`.
    Refuse,
}

/// True only when both addresses resolved and name the same NRO base.
/// A missing base is not ownership: the replacement must be tied to this module.
pub fn owns_replacement(own_module: Option<usize>, replacement_module: Option<usize>) -> bool {
    matches!(
        (own_module, replacement_module),
        (Some(own), Some(replacement)) if own == replacement
    )
}

pub fn select_imgui_host(
    setup_export: bool,
    draw_wrapper: bool,
    get_context_export: bool,
    version_matches: bool,
) -> ImguiHostChoice {
    let standalone_present = setup_export || draw_wrapper || get_context_export;
    if !standalone_present {
        ImguiHostChoice::Embedded
    } else if setup_export && draw_wrapper && version_matches {
        ImguiHostChoice::External
    } else {
        ImguiHostChoice::Refuse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_host_stays_embedded() {
        assert_eq!(
            select_imgui_host(false, false, false, false),
            ImguiHostChoice::Embedded
        );
        assert_eq!(
            select_imgui_host(false, false, false, true),
            ImguiHostChoice::Embedded
        );
    }

    #[test]
    fn known_nro_without_get_context_is_external() {
        assert_eq!(
            select_imgui_host(true, true, false, true),
            ImguiHostChoice::External
        );
    }

    #[test]
    fn optional_get_context_still_external_when_required_pair_matches() {
        assert_eq!(
            select_imgui_host(true, true, true, true),
            ImguiHostChoice::External
        );
    }

    fn sample_header(text_offset: u32) -> [u8; 0x80] {
        let mut header = [0u8; 0x80];
        header[0x10..0x14].copy_from_slice(b"NRO0");
        header[0x18..0x1c].copy_from_slice(&0x200u32.to_le_bytes());
        header[0x20..0x24].copy_from_slice(&text_offset.to_le_bytes());
        header[0x24..0x28].copy_from_slice(&0x100u32.to_le_bytes());
        header[0x28..0x2c].copy_from_slice(&0x100u32.to_le_bytes());
        header[0x2c..0x30].copy_from_slice(&0x40u32.to_le_bytes());
        header[0x30..0x34].copy_from_slice(&0x140u32.to_le_bytes());
        header[0x34..0x38].copy_from_slice(&0x10u32.to_le_bytes());
        header
    }

    #[test]
    fn header_extents_cover_text_and_ro_only() {
        let segments = parse_nro_header(&sample_header(0)).unwrap();
        assert!(text_contains(&segments, 0));
        assert!(text_contains(&segments, 0xff));
        assert!(!text_contains(&segments, 0x100));
        let mut ro = vec![0u8; segments.ro_size as usize];
        assert!(!ro_contains_tag(&ro));
        ro[..COMPATIBLE_IMGUI_VERSION.len()].copy_from_slice(COMPATIBLE_IMGUI_VERSION);
        assert!(ro_contains_tag(&ro));
        assert!(!ro_contains_tag(&COMPATIBLE_IMGUI_VERSION[..COMPATIBLE_IMGUI_VERSION.len() - 1]));
    }

    #[test]
    fn measured_elite_tag_is_inside_its_ro_segment() {
        let mut header = [0u8; 0x80];
        header[0x10..0x14].copy_from_slice(b"NRO0");
        for (offset, value) in [
            (0x18, 0x134000u32),
            (0x20, 0),
            (0x24, 0xe8000),
            (0x28, 0xe8000),
            (0x2c, 0x42000),
            (0x30, 0x12a000),
            (0x34, 0xa000),
        ] {
            header[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        let segments = parse_nro_header(&header).unwrap();
        let tag = 0xf6083usize;
        assert!(text_contains(&segments, 0x1000));
        assert!(!text_contains(&segments, tag));
        assert!(tag >= segments.ro_offset as usize);
        assert!(tag + COMPATIBLE_IMGUI_VERSION.len() <= (segments.ro_offset + segments.ro_size) as usize);
    }

    #[test]
    fn corrupt_extents_are_rejected() {
        assert!(parse_nro_header(&[0u8; 0x80]).is_none());
        let mut header = sample_header(0);
        header[0x2c..0x30].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse_nro_header(&header).is_none());
        assert!(parse_nro_header(&sample_header(0x40)).is_some());
    }

    #[test]
    fn replacement_must_share_this_nro_base() {
        assert!(owns_replacement(Some(0x1000), Some(0x1000)));
        assert!(!owns_replacement(Some(0x1000), Some(0x2000)));
        assert!(!owns_replacement(None, Some(0x1000)));
        assert!(!owns_replacement(Some(0x1000), None));
        assert!(!owns_replacement(None, None));
    }

    #[test]
    fn partial_or_unversioned_host_is_refused() {
        assert_eq!(
            select_imgui_host(true, false, false, true),
            ImguiHostChoice::Refuse
        );
        assert_eq!(
            select_imgui_host(false, true, false, true),
            ImguiHostChoice::Refuse
        );
        assert_eq!(
            select_imgui_host(false, false, true, true),
            ImguiHostChoice::Refuse
        );
        assert_eq!(
            select_imgui_host(true, true, false, false),
            ImguiHostChoice::Refuse
        );
        assert_eq!(
            select_imgui_host(true, true, true, false),
            ImguiHostChoice::Refuse
        );
    }
}
