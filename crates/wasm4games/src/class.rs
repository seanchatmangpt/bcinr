//! Byte-class authority: bounded `u8` state alphabets and the admission status vocabulary.
//!
//! The whole crate operates on small, bounded `u8` domains so that kernels remain
//! branchless and SIMD-friendly. Joining and clamping lower onto [`bcinr_logic`]
//! primitives ([`bcinr_logic::mask::max_u32`], [`bcinr_logic::fix::clamp_u32`]) rather
//! than `if`/`match`.

use bcinr_logic::fix;
use bcinr_logic::mask;

/// The admission status vocabulary, as ordered `u8` codes.
///
/// Ordering is meaningful: it forms a coarse "admission lattice" so a worst-of join can
/// be computed with a branchless `max`. Terminal/abnormal codes (`REFUSED`, `RESIDUAL`)
/// sort above the normal lifecycle so they dominate a join.
pub mod status {
    /// Nothing is known yet.
    pub const UNKNOWN: u8 = 0;
    /// A precondition prevented progress.
    pub const BLOCKED: u8 = 1;
    /// Partially observed; not yet admissible.
    pub const PARTIAL: u8 = 2;
    /// Awaiting a downstream decision.
    pub const PENDING: u8 = 3;
    /// Admitted into bounded state.
    pub const ADMITTED: u8 = 4;
    /// Projected to an engine/host surface.
    pub const PROJECTED: u8 = 5;
    /// Sealed into a receipt chain.
    pub const RECEIPTED: u8 = 6;
    /// Refused by an admission rule.
    pub const REFUSED: u8 = 7;
    /// Residual diagnostic preserved after repair.
    pub const RESIDUAL: u8 = 8;
    /// Cardinality of the vocabulary (number of distinct codes).
    pub const COUNT: u8 = 9;
}

/// A bounded byte class: a `u8` drawn from a domain of known cardinality.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ByteClass(pub u8);

impl ByteClass {
    /// The raw `u8` code.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// Branchlessly clamp this class into the domain `[0, card)`.
    ///
    /// A `card` of `0` is treated as `1` (the domain `{0}`).
    #[inline]
    #[must_use]
    pub fn clamp(self, card: u8) -> ByteClass {
        let hi = card.saturating_sub(1) as u32;
        ByteClass(fix::clamp_u32(self.0 as u32, 0, hi) as u8)
    }
}

/// An admission [`status`] code as a typed value.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Status(pub u8);

impl Status {
    /// The raw `u8` code.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// Worst-of (max-code) lattice join, computed branchlessly.
    #[inline]
    #[must_use]
    pub fn join(self, other: Status) -> Status {
        Status(mask::max_u32(self.0 as u32, other.0 as u32) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_bounds() {
        assert_eq!(ByteClass(200).clamp(status::COUNT).raw(), status::COUNT - 1);
        assert_eq!(ByteClass(3).clamp(status::COUNT).raw(), 3);
        assert_eq!(ByteClass(5).clamp(0).raw(), 0);
    }

    #[test]
    fn join_is_worst_of() {
        assert_eq!(
            Status(status::ADMITTED).join(Status(status::REFUSED)).raw(),
            status::REFUSED
        );
        assert_eq!(
            Status(status::UNKNOWN).join(Status(status::ADMITTED)).raw(),
            status::ADMITTED
        );
    }
}
