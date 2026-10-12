//! Policy for recognizing a pre-existing AArch64 absolute jump at a hook entry.
//!
//! The four words describe `ldr x17, #8; br x17; .quad destination`. The
//! literal payload is data, so callers must chain to its decoded address rather
//! than treating it as instructions to relocate.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryPlan {
    Relocate,
    Chain(u64),
    Reject,
}

const LDR_X17_LITERAL_8: u32 = 0x5800_0051;
const BR_X17: u32 = 0xd61f_0220;
const TARGET_STUB_LEN: u64 = 16;

/// Decide how to handle the entry words before installing a far hook.
///
/// A matching `ldr` must be followed by `br x17`; any other second instruction
/// makes the apparent stub malformed. A valid destination must be nonzero,
/// four-byte aligned, and outside the 16-byte entry being overwritten. It may
/// not point back to the target or to the replacement handler.
pub fn plan_far_chain(words: [u32; 4], target: u64, replacement: u64) -> EntryPlan {
    if words[0] != LDR_X17_LITERAL_8 {
        return EntryPlan::Relocate;
    }
    if words[1] != BR_X17 {
        return EntryPlan::Reject;
    }

    if target & 7 != 0 {
        return EntryPlan::Reject;
    }
    let Some(target_end) = target.checked_add(TARGET_STUB_LEN) else {
        return EntryPlan::Reject;
    };

    let destination = words[2] as u64 | ((words[3] as u64) << 32);
    if destination == 0 || destination & 3 != 0 {
        return EntryPlan::Reject;
    }
    if destination == target || destination == replacement {
        return EntryPlan::Reject;
    }
    if (target..target_end).contains(&destination) {
        return EntryPlan::Reject;
    }

    EntryPlan::Chain(destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn far_stub(destination: u64) -> [u32; 4] {
        [
            LDR_X17_LITERAL_8,
            BR_X17,
            destination as u32,
            (destination >> 32) as u32,
        ]
    }

    const TARGET: u64 = 0x0000_7100_0000_0000;
    const REPLACEMENT: u64 = 0x0000_7100_0000_1000;

    #[test]
    fn crash_reported_low_word_chains_for_each_high_word() {
        for high in [0, 1, 0x1234_5678, u32::MAX] {
            let destination = ((high as u64) << 32) | 0x98bc_e454;
            assert_eq!(
                plan_far_chain(far_stub(destination), TARGET, REPLACEMENT),
                EntryPlan::Chain(destination)
            );
        }
    }

    #[test]
    fn malformed_far_stub_header_is_rejected() {
        let mut words = far_stub(0x0000_7100_0000_2000);
        words[1] = 0xd65f_03c0;
        assert_eq!(
            plan_far_chain(words, TARGET, REPLACEMENT),
            EntryPlan::Reject
        );
    }

    #[test]
    fn zero_and_unaligned_destinations_are_rejected() {
        assert_eq!(
            plan_far_chain(far_stub(0), TARGET, REPLACEMENT),
            EntryPlan::Reject
        );
        assert_eq!(
            plan_far_chain(far_stub(0x0000_7100_0000_2002), TARGET, REPLACEMENT),
            EntryPlan::Reject
        );
    }

    #[test]
    fn self_replacement_and_all_words_of_the_entry_stub_are_rejected() {
        assert_eq!(
            plan_far_chain(far_stub(TARGET), TARGET, REPLACEMENT),
            EntryPlan::Reject
        );
        assert_eq!(
            plan_far_chain(far_stub(REPLACEMENT), TARGET, REPLACEMENT),
            EntryPlan::Reject
        );
        for destination in [TARGET + 4, TARGET + 8, TARGET + 12] {
            assert_eq!(
                plan_far_chain(far_stub(destination), TARGET, REPLACEMENT),
                EntryPlan::Reject
            );
        }
        assert_eq!(
            plan_far_chain(far_stub(TARGET + TARGET_STUB_LEN), TARGET, REPLACEMENT),
            EntryPlan::Chain(TARGET + TARGET_STUB_LEN)
        );
    }

    #[test]
    fn invalid_target_alignment_and_end_overflow_are_rejected() {
        assert_eq!(
            plan_far_chain(far_stub(0x0000_7100_0000_2000), TARGET + 4, REPLACEMENT),
            EntryPlan::Reject
        );
        assert_eq!(
            plan_far_chain(far_stub(0x0000_7100_0000_2000), u64::MAX - 7, REPLACEMENT),
            EntryPlan::Reject
        );
    }

    #[test]
    fn stock_prologue_uses_skyline_relocation() {
        let stock_prologue = [0xa9bf_7bfd, 0x9100_03fd, 0xaa00_03f3, 0xd503_201f];
        assert_eq!(
            plan_far_chain(stock_prologue, TARGET, REPLACEMENT),
            EntryPlan::Relocate
        );
    }
}
