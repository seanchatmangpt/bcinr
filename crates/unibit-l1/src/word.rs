//! Word-level truth algebra.
//!
//! Generated from ontology.

#[repr(transparent)]
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct UCell(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct UMask(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct UDenial(pub u64);

#[inline(always)]
pub fn bool_mask(flag: bool) -> u64 {
    (flag as u64).wrapping_neg()
}

#[inline(always)]
pub fn admit3(
    state: UCell,
    prereq: UMask,
    law: UMask,
    capability: UMask,
) -> UDenial {
    UDenial(
        ((state.0 & prereq.0) ^ prereq.0)
            | ((state.0 & law.0) ^ law.0)
            | ((state.0 & capability.0) ^ capability.0),
    )
}

#[inline(always)]
pub fn commit_masked(
    old: UCell,
    consume: UMask,
    produce: UMask,
    deny: UDenial,
) -> (UCell, UMask) {
    let candidate = (old.0 & !consume.0) | produce.0;
    let admitted = bool_mask(deny.0 == 0);
    let next = (candidate & admitted) | (old.0 & !admitted);

    (UCell(next), UMask(old.0 ^ next))
}