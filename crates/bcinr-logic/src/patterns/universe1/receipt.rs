//! # U1Receipt — FNV-1a Rolling Receipt for Universe1_n
//!
//! Reuses FNV-1a 64-bit offset basis and prime from Universe64 so receipts
//! can be mixed across tiers without re-seeding.

use super::coord::U1Coord;
use crate::patterns::universe64::constants::{FNV1A_64_OFFSET_BASIS, FNV1A_64_PRIME};

/// Typed newtype for a rolling Universe1_n receipt hash.
///
/// ```
/// use bcinr_logic::patterns::universe1::receipt::{U1Receipt, new_u1_receipt};
/// let r = new_u1_receipt();
/// assert_ne!(r.value(), 0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct U1Receipt(u64);

impl U1Receipt {
    /// Construct from raw `u64`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::receipt::U1Receipt;
    /// assert_eq!(U1Receipt::from_raw(0xDEAD).value(), 0xDEAD);
    /// ```
    #[inline(always)]
    pub const fn from_raw(v: u64) -> Self {
        Self(v)
    }

    /// Unwrap to raw `u64`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::receipt::{U1Receipt, new_u1_receipt};
    /// let r = new_u1_receipt();
    /// let v = r.value();
    /// assert_eq!(U1Receipt::from_raw(v).value(), v);
    /// ```
    #[inline(always)]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Seed a fresh receipt using the FNV-1a 64-bit offset basis.
///
/// ```
/// use bcinr_logic::patterns::universe1::receipt::new_u1_receipt;
/// let r = new_u1_receipt();
/// assert_eq!(r.value(), 0xcbf29ce484222325);
/// ```
#[inline(always)]
pub const fn new_u1_receipt() -> U1Receipt {
    U1Receipt(FNV1A_64_OFFSET_BASIS)
}

/// Mix one transition event into a rolling receipt.
/// Order: `coord → sequence → fired_mask → delta_word`, each step:
/// `h ^= field; h = h.wrapping_mul(FNV1A_64_PRIME)`.
///
/// ```
/// use bcinr_logic::patterns::universe1::receipt::{new_u1_receipt, receipt_mix_u1_transition};
/// use bcinr_logic::patterns::universe1::coord::U1Coord;
///
/// let r0 = new_u1_receipt();
/// let c = U1Coord::new_const(1, 2, 3);
/// let r1 = receipt_mix_u1_transition(r0, c, 0, !0u64, 0xDEAD);
/// assert_ne!(r0.value(), r1.value());
///
/// // Determinism: same inputs produce same output.
/// let r1b = receipt_mix_u1_transition(r0, c, 0, !0u64, 0xDEAD);
/// assert_eq!(r1, r1b);
///
/// // Sensitivity: changing any field changes the hash.
/// let r2 = receipt_mix_u1_transition(r0, c, 1, !0u64, 0xDEAD);
/// assert_ne!(r1, r2);
/// ```
#[inline(always)]
pub const fn receipt_mix_u1_transition(
    receipt: U1Receipt,
    coord: U1Coord,
    sequence: u64,
    fired_mask: u64,
    delta_word: u64,
) -> U1Receipt {
    let mut h = receipt.0;
    h ^= coord.as_u16() as u64;
    h = h.wrapping_mul(FNV1A_64_PRIME);
    h ^= sequence;
    h = h.wrapping_mul(FNV1A_64_PRIME);
    h ^= fired_mask;
    h = h.wrapping_mul(FNV1A_64_PRIME);
    h ^= delta_word;
    h = h.wrapping_mul(FNV1A_64_PRIME);
    U1Receipt(h)
}
