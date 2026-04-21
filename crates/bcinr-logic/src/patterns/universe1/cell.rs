//! # U1_64 — Cell profile: U_{1,64} = 𝔹⁶⁴ — 64 one-bit truth atoms in one u64
//!
//! One `u64` holds 64 boolean places. All ops branchless, zero-alloc.
//! Target: T0 (≤ 8 ns) per primitive.

/// 64-bit cell — the atomic register unit of Universe1_n.
///
/// ```
/// use bcinr_logic::patterns::universe1::cell::U1_64;
/// let c = U1_64::new(0b1010);
/// assert_eq!(c.get(), 0b1010);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct U1_64(u64);

impl U1_64 {
    /// Construct from raw `u64`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// assert_eq!(U1_64::new(42).get(), 42);
    /// ```
    #[inline(always)]
    pub const fn new(v: u64) -> Self {
        Self(v)
    }

    /// Zero cell.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// assert_eq!(U1_64::zero().get(), 0);
    /// ```
    #[inline(always)]
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Unwrap to raw `u64`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// assert_eq!(U1_64::new(7).get(), 7);
    /// ```
    #[inline(always)]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Branchless Petri fire: returns `(new_cell, fired_mask)`.
    /// `M' = (M & !I) | O` when `(M & I) == I`, else `M` unchanged.
    /// `fired_mask` is `!0` when fired, `0` otherwise.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// // Input mask 0b11 must be present; output adds bit 2.
    /// let m = U1_64::new(0b0011);
    /// let (m2, fired) = m.fire(0b0011, 0b0100);
    /// assert_eq!(m2.get(), 0b0100);
    /// assert_eq!(fired, !0u64);
    ///
    /// // Not enabled — cell unchanged.
    /// let m = U1_64::new(0b0001);
    /// let (m2, fired) = m.fire(0b0011, 0b0100);
    /// assert_eq!(m2.get(), 0b0001);
    /// assert_eq!(fired, 0);
    /// ```
    #[inline(always)]
    pub const fn fire(self, input: u64, output: u64) -> (Self, u64) {
        let enabled = ((self.0 & input) == input) as u64;
        let fired = 0u64.wrapping_sub(enabled); // 0 or !0
        let candidate = (self.0 & !input) | output;
        let next = (candidate & fired) | (self.0 & !fired);
        (Self(next), fired)
    }

    /// Count set places (popcount).
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// assert_eq!(U1_64::new(0b1011).popcount(), 3);
    /// ```
    #[inline(always)]
    pub const fn popcount(self) -> u32 {
        self.0.count_ones()
    }

    /// Law admissibility: `true` iff all set bits are allowed by `law`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// let law = 0b1111;
    /// assert!(U1_64::new(0b1010).is_admissible(law));
    /// assert!(!U1_64::new(0b11110000).is_admissible(law));
    /// ```
    #[inline(always)]
    pub const fn is_admissible(self, law: u64) -> bool {
        (self.0 & !law) == 0
    }

    /// Law violation mask: set bits that violate `law`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// assert_eq!(U1_64::new(0b1111).law_violation(0b1010), 0b0101);
    /// ```
    #[inline(always)]
    pub const fn law_violation(self, law: u64) -> u64 {
        self.0 & !law
    }

    /// Hamming distance from `expected`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::cell::U1_64;
    /// let a = U1_64::new(0b1100);
    /// let b = U1_64::new(0b1010);
    /// assert_eq!(a.distance(b), 2);
    /// ```
    #[inline(always)]
    pub const fn distance(self, expected: Self) -> u32 {
        (self.0 ^ expected.0).count_ones()
    }
}
