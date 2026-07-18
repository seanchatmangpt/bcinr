/// Opaque, branchless-composable fault set for fixed-point numeric operations.
///
/// Replaces the historical `u32::MAX`-is-OK sentinel. The inner representation is
/// private; the only publicly constructible values are `EMPTY` and unions/masked
/// selections of previously-constructed `NumericFaultSet` values, which by
/// construction always land in the powerset of the named bits below.
///
/// Accumulation across a computation path is a join-semilattice under bitwise
/// union with `EMPTY` as the identity element (see
/// `.claude/rules/cmca/numeric-hot-path.md` Invariant 1) — never
/// first-error-wins, never last-error-wins.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct NumericFaultSet(u32);

impl NumericFaultSet {
    /// Identity element of the union semilattice: no fault present.
    pub const EMPTY: Self = Self(0);

    pub const OVERFLOW: Self = Self(1 << 0);
    pub const UNDERFLOW: Self = Self(1 << 1);
    pub const DIVIDE_BY_ZERO: Self = Self(1 << 2);
    pub const INVALID_DOMAIN: Self = Self(1 << 3);
    pub const INVALID_NORMALIZATION: Self = Self(1 << 4);
    pub const SUPPORT_MISMATCH: Self = Self(1 << 5);
    pub const SATURATION: Self = Self(1 << 6);
    pub const APPROX_ENVELOPE: Self = Self(1 << 7);
    pub const RANGE_VIOLATION: Self = Self(1 << 8);

    /// Branchless Contract: bitwise union — the only accumulation operator for fault
    /// sets. Never short-circuits to "first fault wins" or "last fault wins";
    /// `{P: true} union(a, b) {Q: result.bits() == a.bits() | b.bits()}`, total, no
    /// branches, no data-dependent control flow (numeric-hot-path.md Invariant 1).
    #[inline(always)]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[inline(always)]
    const fn from_bits_raw(bits: u32) -> Self {
        Self(bits)
    }
}

/// Branchless canonical boolean mask: all-zero or all-one bit pattern for a
/// given width. The inner value is private; the only publicly constructible
/// values are `TRUE`/`FALSE` and comparator outputs, all of which land in
/// `{0, u32::MAX}` (see Invariant 3 in numeric-hot-path.md).
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalMask(u32);

impl CanonicalMask {
    pub const TRUE: Self = Self(u32::MAX);
    pub const FALSE: Self = Self(0);

    /// Build a canonical mask from a single least-significant bit (0 or 1).
    /// Any nonzero LSB collapses to all-ones via wrapping negation, so the
    /// image remains `{0, u32::MAX}` regardless of the other bits of `lsb`.
    #[inline(always)]
    pub const fn from_lsb(lsb: u32) -> Self {
        Self(0u32.wrapping_sub(lsb & 1))
    }

    /// Branchless Contract: `{P: self in {TRUE, FALSE}} select_u32(a, b) {Q: result ==
    /// a if self == TRUE else b}`, computed by mask-AND/OR with no branch, no
    /// data-dependent jump.
    #[inline(always)]
    pub const fn select_u32(self, a: u32, b: u32) -> u32 {
        (a & self.0) | (b & !self.0)
    }

    #[inline(always)]
    pub const fn select_i32(self, a: i32, b: i32) -> i32 {
        (a & self.0 as i32) | (b & !(self.0 as i32))
    }

    /// Branchless Contract: selects between two fault sets using the same mask, so
    /// that `select(m, (v_a, f_a), (v_b, f_b)) = (select(m, v_a, v_b), select(m, f_a,
    /// f_b))` holds by construction (Invariant 2, numeric-hot-path.md). Total,
    /// branchless.
    #[inline(always)]
    pub const fn select_faults(self, a: NumericFaultSet, b: NumericFaultSet) -> NumericFaultSet {
        NumericFaultSet::from_bits_raw(self.select_u32(a.0, b.0))
    }

    #[inline(always)]
    const fn raw(self) -> u32 {
        self.0
    }
}

/// Branchless Contract: `{P: true} const_lt_u32(a, b) {Q: result == TRUE iff a < b,
/// else FALSE}`, computed via bit-parallel comparison, no branch, no
/// data-dependent jump (numeric-hot-path.md Invariant 3: two-point image).
#[inline(always)]
pub const fn const_lt_u32(a: u32, b: u32) -> CanonicalMask {
    let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
    CanonicalMask(0u32.wrapping_sub(diff))
}

/// Branchless Contract: `{P: true} const_eq_u32(a, b) {Q: result == TRUE iff a ==
/// b, else FALSE}`, computed via bit-parallel comparison, no branch, no
/// data-dependent jump (numeric-hot-path.md Invariant 3: two-point image).
#[inline(always)]
pub const fn const_eq_u32(a: u32, b: u32) -> CanonicalMask {
    let x = a ^ b;
    #[cfg(not(feature = "mutant_7"))]
    let nonzero = (x | x.wrapping_neg()) >> 31;
    #[cfg(feature = "mutant_7")]
    let nonzero = (!x & !x.wrapping_neg()) >> 31; // Mutated: sign inversion
    CanonicalMask(0u32.wrapping_sub(1u32.wrapping_sub(nonzero)))
}

#[inline(always)]
pub const fn const_lt_i32(a: i32, b: i32) -> CanonicalMask {
    let diff = (a as u32).wrapping_sub(b as u32);
    let a_sign = (a as u32) >> 31;
    let b_sign = (b as u32) >> 31;
    let diff_sign = diff >> 31;
    let res = (a_sign & (b_sign ^ 1)) | ((!(a_sign ^ b_sign)) & diff_sign);
    CanonicalMask(0u32.wrapping_sub(res))
}

#[inline(always)]
pub const fn const_eq_i32(a: i32, b: i32) -> CanonicalMask {
    const_eq_u32(a as u32, b as u32)
}

/// Non-negative Q16.16 fixed-point value with a sealed representation: the
/// magnitude and fault set travel together and can only be constructed
/// through the crate's own arithmetic (public callers use `from_value_bits`,
/// which always starts fault-free, or the accessors below).
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct NonNegativeFixed {
    val: u32,
    faults: NumericFaultSet,
}

impl NonNegativeFixed {
    pub const ZERO: Self = Self {
        val: 0,
        faults: NumericFaultSet::EMPTY,
    };
    pub const ONE: Self = Self {
        val: 65536,
        faults: NumericFaultSet::EMPTY,
    };
    pub const MAX: Self = Self {
        val: u32::MAX,
        faults: NumericFaultSet::EMPTY,
    };

    /// Construct a fault-free value from raw Q16.16 bits.
    #[inline(always)]
    pub const fn from_value_bits(bits: u32) -> Self {
        Self {
            val: bits,
            faults: NumericFaultSet::EMPTY,
        }
    }

    /// Crate-internal constructor letting arithmetic carry a fault set
    /// alongside the value it produced.
    #[inline(always)]
    pub(crate) const fn from_parts(val: u32, faults: NumericFaultSet) -> Self {
        Self { val, faults }
    }

    /// Branchless Contract: total accessor, `{P: true} value_bits(self) {Q: result ==
    /// self.val}`, no branch.
    #[inline(always)]
    pub const fn value_bits(self) -> u32 {
        self.val
    }

    /// Branchless Contract: total accessor, `{P: true} faults(self) {Q: result ==
    /// self.faults}`, no branch.
    #[inline(always)]
    pub const fn faults(self) -> NumericFaultSet {
        self.faults
    }

    #[inline(always)]
    pub const fn from_num(num: u32) -> Self {
        Self::from_value_bits(num.wrapping_shl(16))
    }

    #[inline(always)]
    pub const fn to_num(self) -> u32 {
        self.val >> 16
    }

    #[inline(always)]
    pub const fn saturating_add(self, other: Self) -> Self {
        let sum = self.val.wrapping_add(other.val);
        #[cfg(not(feature = "mutant_6"))]
        let overflow = const_lt_u32(sum, self.val);
        #[cfg(feature = "mutant_6")]
        let overflow = const_lt_u32(self.val, sum); // Mutated: inverted overflow condition
        let e = CanonicalMask::select_faults(
            overflow,
            NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
            NumericFaultSet::EMPTY,
        );
        Self {
            val: overflow.select_u32(u32::MAX, sum),
            faults: self.faults.union(other.faults).union(e),
        }
    }

    #[inline(always)]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let underflow = const_lt_u32(self.val, other.val);
        let e = CanonicalMask::select_faults(
            underflow,
            NumericFaultSet::UNDERFLOW,
            NumericFaultSet::EMPTY,
        );
        Self {
            val: underflow.select_u32(0, self.val.wrapping_sub(other.val)),
            faults: self.faults.union(other.faults).union(e),
        }
    }

    #[inline(always)]
    pub const fn saturating_mul(self, other: Self) -> Self {
        let prod = (self.val as u64).wrapping_mul(other.val as u64);
        let res_u64 = prod >> 16;
        let high = (res_u64 >> 32) as u32;
        let overflow = (high | high.wrapping_neg()) >> 31;
        let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow));
        let e = CanonicalMask::select_faults(
            overflow_mask,
            NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
            NumericFaultSet::EMPTY,
        );
        Self {
            val: overflow_mask.select_u32(u32::MAX, res_u64 as u32),
            faults: self.faults.union(other.faults).union(e),
        }
    }

    #[inline(always)]
    pub const fn saturating_div(self, other: Self) -> Self {
        let den_is_zero = const_eq_u32(other.val, 0);
        let d = den_is_zero.select_u32(1, other.val);

        let lz = d.leading_zeros();
        let d_norm = d << lz;

        let a_scale = 13021703673752174592u64;
        let b_coeff = 2021160080u64;
        let x0 = a_scale.wrapping_sub(b_coeff.wrapping_mul(d_norm as u64));

        let e0 = (1i128 << 94) - (d_norm as i128) * (x0 as i128);
        let x1 = ((x0 as i128) + (((x0 as i128) * (e0 >> 32)) >> 62)) as u64;

        let e1 = (1i128 << 94) - (d_norm as i128) * (x1 as i128);
        let x2 = ((x1 as i128) + (((x1 as i128) * (e1 >> 32)) >> 62)) as u64;

        let e2 = (1i128 << 94) - (d_norm as i128) * (x2 as i128);
        let x3 = ((x2 as i128) + (((x2 as i128) * (e2 >> 32)) >> 62)) as u64;

        let n = self.val as u128;
        let q_u128 = n.wrapping_mul(x3 as u128);
        let q_shifted_46 = (q_u128 >> 46) as u64;
        let q = q_shifted_46 >> (32 - lz);

        let rem = ((self.val as u64) << 16).wrapping_sub(q.wrapping_mul(d as u64)) as i64;

        let is_lt = ((rem >> 63) & 1) as u64;
        let diff = rem.wrapping_sub(d as i64);
        let is_ge = (((!diff) >> 63) & 1) as u64;

        let q_corrected = q.wrapping_add(is_ge).wrapping_sub(is_lt);

        let overflow_1 = const_lt_u32(u32::MAX, (q_corrected >> 32) as u32).raw();
        let overflow_2 = 0u32.wrapping_sub((q_corrected > u32::MAX as u64) as u32);
        let overflow = CanonicalMask(overflow_1 | overflow_2);

        let saturate = CanonicalMask(overflow.raw() | den_is_zero.raw());

        let e = CanonicalMask::select_faults(
            den_is_zero,
            NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
            CanonicalMask::select_faults(
                overflow,
                NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
                NumericFaultSet::EMPTY,
            ),
        );
        Self {
            val: saturate.select_u32(u32::MAX, q_corrected as u32),
            faults: self.faults.union(other.faults).union(e),
        }
    }

    #[inline(always)]
    pub fn log2(self) -> SignedFixed {
        let x = self.val as u64;
        let lz = x.leading_zeros();
        let nz = ((x | x.wrapping_neg()) >> 63) & 1;
        let ip = 63u64.wrapping_sub(lz as u64) & nz.wrapping_neg();

        let mantissa = x.wrapping_shl(lz.wrapping_add(1));
        let f = (mantissa >> (64 - 16)) as u32;

        let diff = 65536 - f;
        let correction = (f * diff) >> 16;
        let corrected_frac = f + ((correction * 29013) >> 16);

        let res = (ip << 16).wrapping_add(corrected_frac as u64);

        #[cfg(not(feature = "mutant_8"))]
        let is_zero = const_eq_u32(self.val, 0);
        #[cfg(feature = "mutant_8")]
        let is_zero = const_eq_u32(0, 0); // Mutated: always true
        let computed = (res as u32).wrapping_sub(16 << 16) as i32;
        let e = CanonicalMask::select_faults(
            is_zero,
            NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
            NumericFaultSet::EMPTY,
        );
        SignedFixed {
            val: is_zero.select_i32(-1048576, computed),
            faults: self.faults.union(e),
        }
    }
}

/// Signed Q16.16 fixed-point value with a sealed representation (see
/// `NonNegativeFixed` for the rationale).
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SignedFixed {
    val: i32,
    faults: NumericFaultSet,
}

impl SignedFixed {
    pub const ZERO: Self = Self {
        val: 0,
        faults: NumericFaultSet::EMPTY,
    };
    pub const ONE: Self = Self {
        val: 65536,
        faults: NumericFaultSet::EMPTY,
    };
    pub const MAX: Self = Self {
        val: i32::MAX,
        faults: NumericFaultSet::EMPTY,
    };
    pub const MIN: Self = Self {
        val: i32::MIN,
        faults: NumericFaultSet::EMPTY,
    };

    #[inline(always)]
    pub const fn from_value_bits(bits: i32) -> Self {
        Self {
            val: bits,
            faults: NumericFaultSet::EMPTY,
        }
    }

    #[inline(always)]
    pub(crate) const fn from_parts(val: i32, faults: NumericFaultSet) -> Self {
        Self { val, faults }
    }

    /// Branchless Contract: total accessor, `{P: true} value_bits(self) {Q: result ==
    /// self.val}`, no branch.
    #[inline(always)]
    pub const fn value_bits(self) -> i32 {
        self.val
    }

    /// Branchless Contract: total accessor, `{P: true} faults(self) {Q: result ==
    /// self.faults}`, no branch.
    #[inline(always)]
    pub const fn faults(self) -> NumericFaultSet {
        self.faults
    }

    #[inline(always)]
    pub const fn from_num(num: i32) -> Self {
        Self::from_value_bits(num.wrapping_shl(16))
    }

    #[inline(always)]
    pub const fn to_num(self) -> i32 {
        self.val >> 16
    }

    #[inline(always)]
    pub const fn saturating_add(self, other: Self) -> Self {
        let (sum, overflow) = self.val.overflowing_add(other.val);
        let is_neg = const_lt_i32(self.val, 0);
        let sat_val = is_neg.select_i32(i32::MIN, i32::MAX);
        let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow as u32));
        let e = CanonicalMask::select_faults(
            overflow_mask,
            NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
            NumericFaultSet::EMPTY,
        );
        Self {
            val: overflow_mask.select_i32(sat_val, sum),
            faults: self.faults.union(other.faults).union(e),
        }
    }

    #[inline(always)]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let (diff, overflow) = self.val.overflowing_sub(other.val);
        let is_neg = const_lt_i32(self.val, 0);
        let sat_val = is_neg.select_i32(i32::MIN, i32::MAX);
        let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow as u32));
        let e = CanonicalMask::select_faults(
            overflow_mask,
            NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
            NumericFaultSet::EMPTY,
        );
        Self {
            val: overflow_mask.select_i32(sat_val, diff),
            faults: self.faults.union(other.faults).union(e),
        }
    }

    #[inline(always)]
    pub const fn saturating_mul(self, other: Self) -> Self {
        let prod = (self.val as i64).wrapping_mul(other.val as i64);
        let res_i64 = prod >> 16;

        let overflow_max = CanonicalMask(0u32.wrapping_sub((res_i64 > i32::MAX as i64) as u32));
        let overflow_min = CanonicalMask(0u32.wrapping_sub((res_i64 < i32::MIN as i64) as u32));

        let mut res = overflow_min.select_i32(i32::MIN, res_i64 as i32);
        res = overflow_max.select_i32(i32::MAX, res);

        let is_overflow = overflow_max.raw() | overflow_min.raw();
        let e = CanonicalMask::select_faults(
            const_eq_u32(is_overflow, 0),
            NumericFaultSet::EMPTY,
            NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
        );
        Self {
            val: res,
            faults: self.faults.union(other.faults).union(e),
        }
    }

    #[inline(always)]
    pub fn exp2(self) -> NonNegativeFixed {
        let x = self.val;
        let ip = x >> 16;
        let fp = x.wrapping_sub(ip.wrapping_shl(16));

        let y = fp as u32;
        let res1 = (y.wrapping_mul(630)) >> 16;
        let res2 = (y.wrapping_mul(3637u32.wrapping_add(res1))) >> 16;
        let res3 = (y.wrapping_mul(15763u32.wrapping_add(res2))) >> 16;
        let res4 = (y.wrapping_mul(45506u32.wrapping_add(res3))) >> 16;
        let frac_part = 65536u32.wrapping_add(res4);

        let is_overflow =
            CanonicalMask(0u32.wrapping_sub(((((ip.wrapping_sub(16)) >> 31) ^ 1) & 1) as u32));
        let is_underflow = CanonicalMask(
            0u32.wrapping_sub((((((-17i32).wrapping_sub(ip)) >> 31) ^ 1) & 1) as u32),
        );

        let shl = (ip & 31) as u32;
        let shr = ((ip.wrapping_neg()) & 31) as u32;

        let val_shl = frac_part.wrapping_shl(shl);
        let val_shr = frac_part.wrapping_shr(shr);

        let ip_neg = CanonicalMask(0u32.wrapping_sub(((ip >> 31) & 1) as u32));
        let val_shifted = ip_neg.select_u32(val_shr, val_shl);

        let res = is_overflow.select_u32(u32::MAX, is_underflow.select_u32(0, val_shifted));
        let e = CanonicalMask::select_faults(
            const_eq_u32(is_overflow.raw() | is_underflow.raw(), 0),
            NumericFaultSet::EMPTY,
            NumericFaultSet::RANGE_VIOLATION,
        );
        NonNegativeFixed {
            val: res,
            faults: self.faults.union(e),
        }
    }

    #[inline(always)]
    pub fn exp(self) -> NonNegativeFixed {
        let x = self.val;
        let z = (((x as i64).wrapping_mul(94548)) >> 16) as i32;
        SignedFixed {
            val: z,
            faults: self.faults,
        }
        .exp2()
    }
}

impl core::ops::Add for NonNegativeFixed {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
}
impl core::ops::Sub for NonNegativeFixed {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}
impl core::ops::Mul for NonNegativeFixed {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}
impl core::ops::Div for NonNegativeFixed {
    type Output = Self;
    #[inline(always)]
    fn div(self, other: Self) -> Self {
        self.saturating_div(other)
    }
}
impl core::ops::AddAssign for NonNegativeFixed {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl core::ops::SubAssign for NonNegativeFixed {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
impl core::ops::MulAssign for NonNegativeFixed {
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}
impl core::ops::DivAssign for NonNegativeFixed {
    #[inline(always)]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl core::ops::Add for SignedFixed {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
}
impl core::ops::Sub for SignedFixed {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}
impl core::ops::Mul for SignedFixed {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}
impl core::ops::AddAssign for SignedFixed {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl core::ops::SubAssign for SignedFixed {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
impl core::ops::MulAssign for SignedFixed {
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_accumulates_both_operands_distinct_faults() {
        let a = NonNegativeFixed::from_parts(1, NumericFaultSet::UNDERFLOW);
        let b = NonNegativeFixed::from_parts(2, NumericFaultSet::DIVIDE_BY_ZERO);
        let combined = a.faults().union(b.faults());
        assert!(!combined.is_empty());
        assert_eq!(
            combined.bits(),
            NumericFaultSet::UNDERFLOW.bits() | NumericFaultSet::DIVIDE_BY_ZERO.bits()
        );
        // Neither original bit is lost (no first/last-wins collapse).
        assert_ne!(combined, NumericFaultSet::UNDERFLOW);
        assert_ne!(combined, NumericFaultSet::DIVIDE_BY_ZERO);
    }

    #[test]
    fn select_preserves_selected_fault_and_drops_unselected() {
        let clean = NonNegativeFixed::from_parts(10, NumericFaultSet::EMPTY);
        let faulted = NonNegativeFixed::from_parts(20, NumericFaultSet::RANGE_VIOLATION);

        // Select `clean` (mask FALSE selects `b`): the unselected faulted
        // alternative's fault must not leak into the result.
        let mask_pick_b = CanonicalMask::FALSE;
        let picked_value = mask_pick_b.select_u32(faulted.value_bits(), clean.value_bits());
        let picked_faults = mask_pick_b.select_faults(faulted.faults(), clean.faults());
        assert_eq!(picked_value, clean.value_bits());
        assert!(
            picked_faults.is_empty(),
            "unselected fault leaked (contamination)"
        );

        // Select `faulted` (mask TRUE selects `a`): its fault must survive
        // the selection, not be erased.
        let mask_pick_a = CanonicalMask::TRUE;
        let picked_value2 = mask_pick_a.select_u32(faulted.value_bits(), clean.value_bits());
        let picked_faults2 = mask_pick_a.select_faults(faulted.faults(), clean.faults());
        assert_eq!(picked_value2, faulted.value_bits());
        assert_eq!(
            picked_faults2,
            NumericFaultSet::RANGE_VIOLATION,
            "selected fault was erased"
        );
    }

    #[test]
    fn mask_public_image_is_exactly_zero_or_all_ones() {
        fn check(m: CanonicalMask) {
            assert!(m.raw() == 0 || m.raw() == u32::MAX);
        }
        check(CanonicalMask::TRUE);
        check(CanonicalMask::FALSE);
        for lsb in 0u32..=1 {
            check(CanonicalMask::from_lsb(lsb));
        }
        for a in [0u32, 1, 2, u32::MAX / 2, u32::MAX - 1, u32::MAX] {
            for b in [0u32, 1, 2, u32::MAX / 2, u32::MAX - 1, u32::MAX] {
                check(const_lt_u32(a, b));
                check(const_eq_u32(a, b));
            }
        }
        for a in [i32::MIN, -1, 0, 1, i32::MAX] {
            for b in [i32::MIN, -1, 0, 1, i32::MAX] {
                check(const_lt_i32(a, b));
                check(const_eq_i32(a, b));
            }
        }
        // select_faults composed from any two masks still yields a mask-selected
        // fault set; verify the mask driving it stays in {0, all-ones}.
        let combined = CanonicalMask::from_lsb(1);
        check(combined);
    }

    // Skipped under `mutant_7`: that feature flips the sign of `const_eq_u32`'s
    // nonzero test (src/fixed.rs `const_eq_u32`), which `saturating_div` uses to
    // detect a zero denominator. Under the mutation, `den_is_zero` for a genuine
    // zero denominator resolves to `CanonicalMask::FALSE`, so `d` is selected as
    // `other.val` (0) instead of the fallback `1`, and the unconditional
    // `d << lz` normalization step panics on shift-overflow before this test's
    // own assertions can run. This is `mutant_7`'s own dedicated oracle
    // (`kill_mutant_7_saturating_div_false_zero` in
    // tests/hostile_mutants.rs) doing its job on shared production code — not a
    // weakening of what this baseline test asserts under the default build.
    #[cfg(not(feature = "mutant_7"))]
    #[test]
    fn saturating_div_by_zero_reports_divide_by_zero_and_invalid_domain() {
        let a = NonNegativeFixed::from_num(5);
        let z = NonNegativeFixed::ZERO;
        let r = a.saturating_div(z);
        assert!(!r.faults().is_empty());
        assert_eq!(
            r.faults().bits()
                & (NumericFaultSet::DIVIDE_BY_ZERO.bits() | NumericFaultSet::INVALID_DOMAIN.bits()),
            NumericFaultSet::DIVIDE_BY_ZERO.bits() | NumericFaultSet::INVALID_DOMAIN.bits()
        );
    }

    // Skipped under `mutant_6`: that feature inverts the overflow comparison in
    // `saturating_add` (src/fixed.rs, `const_lt_u32(self.val, sum)` instead of
    // `const_lt_u32(sum, self.val)`), so `MAX.saturating_add(ONE)` no longer
    // reports OVERFLOW|SATURATION. This is `mutant_6`'s own dedicated oracle
    // (`kill_mutant_6_saturating_add_false_overflow` in
    // tests/hostile_mutants.rs) doing its job on shared production code — not a
    // weakening of what this baseline test asserts under the default build.
    #[cfg(not(feature = "mutant_6"))]
    #[test]
    fn saturating_add_overflow_reports_overflow_and_saturation() {
        let r = NonNegativeFixed::MAX.saturating_add(NonNegativeFixed::ONE);
        assert_eq!(
            r.faults().bits(),
            NumericFaultSet::OVERFLOW.bits() | NumericFaultSet::SATURATION.bits()
        );
    }

    #[test]
    fn from_value_bits_is_fault_free() {
        assert!(NonNegativeFixed::from_value_bits(42).faults().is_empty());
        assert!(SignedFixed::from_value_bits(-42).faults().is_empty());
    }
}
