//! Portable checks around the CSS pointer walk.
pub fn banner_address(owner: usize, mut read: impl FnMut(usize) -> Option<usize>) -> Option<usize> {
    fn plausible(value: usize) -> Option<usize> {
        (value >= 0x10000 && value.is_multiple_of(8)).then_some(value)
    }
    let first = plausible(read(plausible(owner)?.checked_add(0xe58)?)?)?;
    // The second value IS the pane. Dereferencing again would read its vtable.
    plausible(read(first.checked_add(0x10)?)?)
}

/// txt1 identities in the user's local chara_select.layout.arc.
/// No game asset data is shipped. Restrict this to known title TextBoxes.
pub fn is_title_name(name: &[u8]) -> bool {
    matches!(
        name,
        b"txt_all_title_00" | b"txt_all_title_01" | b"txt_vip_title_00" | b"txt_vip_title_01"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chain_stops_at_pane_not_vtable_and_never_reads_null() {
        let mut visited = Vec::new();
        let result = banner_address(0x10000, |address| {
            visited.push(address);
            match address {
                0x10e58 => Some(0x20000),
                0x20010 => Some(0x30000),
                _ => panic!("extra dereference"),
            }
        });
        assert_eq!(result, Some(0x30000));
        assert_eq!(visited, [0x10e58, 0x20010]);
        assert_eq!(banner_address(0, |_| panic!("null dereference")), None);
        assert_eq!(banner_address(0x10000, |_| None), None);
        assert_eq!(banner_address(usize::MAX - 7, |_| panic!("overflow")), None);
    }
    #[test]
    fn only_title_textboxes_can_be_written() {
        for name in [
            "txt_all_title_00",
            "txt_all_title_01",
            "txt_vip_title_00",
            "txt_vip_title_01",
        ] {
            assert!(is_title_name(name.as_bytes()));
        }
        for name in ["RootPane", "set_txt_name_l", "txt_all_title_00_extra", ""] {
            assert!(!is_title_name(name.as_bytes()));
        }
    }
}
