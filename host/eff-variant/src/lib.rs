//! Dev-only entry. The algorithm lives in the product so the NRO stays one crate.
pub use ssbu_online_deluxe_rewrite::eff_sibling::{
    append_reduced_sibling, append_reduced_sibling_in_place, append_reduced_siblings_in_place,
    append_trace_life_sibling_in_place, AppendedSibling, SiblingRequest, VariantError,
    TRACE_LIFE_OFFSET, TRACE_RATE_OFFSET,
};
