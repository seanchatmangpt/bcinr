use crate::allocator::StabilityRefusal;

const OK: u32 = u32::MAX;

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CanonicalMask {
    pub val: u32,
}

impl CanonicalMask {
    pub const TRUE: Self = Self { val: u32::MAX };
    pub const FALSE: Self = Self { val: 0 };

    #[inline(always)]
    pub const fn select_u32(self, a: u32, b: u32) -> u32 {
        (a & self.val) | (b & !self.val)
    }

    #[inline(always)]
    pub const fn select_i32(self, a: i32, b: i32) -> i32 {
        (a & self.val as i32) | (b & !(self.val as i32))
    }

    /// Select a complete fixed-point state. The error channel is never
    /// reconstructed or reset independently from the value channel.
    #[inline(always)]
    pub const fn select_nonnegative(
        self,
        a: NonNegativeFixed,
        b: NonNegativeFixed,
    ) -> NonNegativeFixed {
        NonNegativeFixed {
            val: self.select_u32(a.val, b.val),
            err: self.select_u32(a.err, b.err),
        }
    }

    /// Signed counterpart of [`Self::select_nonnegative`].
    #[inline(always)]
    pub const fn select_signed(self, a: SignedFixed, b: SignedFixed) -> SignedFixed {
        SignedFixed {
            val: self.select_i32(a.val, b.val),
            err: self.select_u32(a.err, b.err),
        }
    }
}

#[inline(always)]
pub const fn const_lt_u32(a: u32, b: u32) -> CanonicalMask {
    let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
    CanonicalMask {
        val: 0u32.wrapping_sub(diff),
    }
}

#[inline(always)]
pub const fn const_eq_u32(a: u32, b: u32) -> CanonicalMask {
    let x = a ^ b;
    #[cfg(not(feature = "mutant_7"))]
    let nonzero = (x | x.wrapping_neg()) >> 31;
    #[cfg(feature = "mutant_7")]
    let nonzero = (!x & !x.wrapping_neg()) >> 31;
    CanonicalMask {
        val: 0u32.wrapping_sub(1u32.wrapping_sub(nonzero)),
    }
}

#[inline(always)]
pub const fn const_lt_i32(a: i32, b: i32) -> CanonicalMask {
    let diff = (a as u32).wrapping_sub(b as u32);
    let a_sign = (a as u32) >> 31;
    let b_sign = (b as u32) >> 31;
    let diff_sign = diff >> 31;
    let result = (a_sign & (b_sign ^ 1)) | ((!(a_sign ^ b_sign)) & diff_sign);
    CanonicalMask {
        val: 0u32.wrapping_sub(result),
    }
}

#[inline(always)]
pub const fn const_eq_i32(a: i32, b: i32) -> CanonicalMask {
    const_eq_u32(a as u32, b as u32)
}

/// First-error reduction. `u32::MAX` is the unique success sentinel.
#[inline(always)]
pub const fn branchless_err_acc(first: u32, second: u32) -> u32 {
    const_eq_u32(first, OK).select_u32(second, first)
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct NonNegativeFixed {
    pub val: u32,
    pub err: u32,
}

impl NonNegativeFixed {
    pub const ZERO: Self = Self { val: 0, err: OK };
    pub const ONE: Self = Self {
        val: 65_536,
        err: OK,
    };
    pub const MAX: Self = Self {
        val: u32::MAX,
        err: OK,
    };

    #[inline(always)]
    pub const fn from_bits(bits: u32) -> Self {
        Self { val: bits, err: OK }
    }

    #[inline(always)]
    pub const fn from_bits_with_error(bits: u32, err: u32) -> Self {
        Self { val: bits, err }
    }

    #[inline(always)]
    pub const fn to_bits(self) -> u32 {
        self.val
    }

    #[inline(always)]
    pub const fn from_num(number: u32) -> Self {
        let overflow = CanonicalMask {
            val: 0u32.wrapping_sub((number > 65_535) as u32),
        };
        Self {
            val: overflow.select_u32(u32::MAX, number << 16),
            err: overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK),
        }
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
        let overflow = const_lt_u32(self.val, sum);
        let local = overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK);
        Self {
            val: overflow.select_u32(u32::MAX, sum),
            err: branchless_err_acc(self.err, branchless_err_acc(other.err, local)),
        }
    }

    #[inline(always)]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let underflow = const_lt_u32(self.val, other.val);
        let local = underflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK);
        Self {
            val: underflow.select_u32(0, self.val.wrapping_sub(other.val)),
            err: branchless_err_acc(self.err, branchless_err_acc(other.err, local)),
        }
    }

    #[inline(always)]
    pub const fn saturating_mul(self, other: Self) -> Self {
        let product = (self.val as u64) * (other.val as u64);
        let shifted = product >> 16;
        let overflow = CanonicalMask {
            val: 0u32.wrapping_sub((shifted > u32::MAX as u64) as u32),
        };
        let local = overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK);
        Self {
            val: overflow.select_u32(u32::MAX, shifted as u32),
            err: branchless_err_acc(self.err, branchless_err_acc(other.err, local)),
        }
    }

    #[inline(always)]
    pub const fn saturating_div(self, other: Self) -> Self {
        let zero = const_eq_u32(other.val, 0);
        let denominator = zero.select_u32(1, other.val) as u64;
        let quotient = ((self.val as u64) << 16) / denominator;
        let overflow = CanonicalMask {
            val: 0u32.wrapping_sub((quotient > u32::MAX as u64) as u32),
        };
        let local = zero.select_u32(
            StabilityRefusal::UnsupportedDomain as u32,
            overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK),
        );
        Self {
            val: CanonicalMask {
                val: zero.val | overflow.val,
            }
            .select_u32(u32::MAX, quotient as u32),
            err: branchless_err_acc(self.err, branchless_err_acc(other.err, local)),
        }
    }

    /// Deterministic Q16.16 binary logarithm. Sixteen fixed squaring steps
    /// manufacture the fractional bits; the approximation error is below one
    /// output ulp for every positive representable input.
    #[inline(always)]
    pub fn log2(self) -> SignedFixed {
        #[cfg(not(feature = "mutant_8"))]
        let zero = const_eq_u32(self.val, 0);
        #[cfg(feature = "mutant_8")]
        let zero = const_eq_u32(0, 0);
        let safe = zero.select_u32(1, self.val);
        let msb = 31i32 - safe.leading_zeros() as i32;
        let integer = msb - 16;
        let mut normalized = (safe as u64) << (31 - msb as u32);
        let mut fraction = 0u32;

        macro_rules! bit {
            ($position:expr) => {{
                normalized = (normalized * normalized) >> 31;
                let ge_two = ((normalized >> 32) != 0) as u32;
                normalized >>= ge_two;
                fraction |= ge_two << $position;
            }};
        }
        bit!(15);
        bit!(14);
        bit!(13);
        bit!(12);
        bit!(11);
        bit!(10);
        bit!(9);
        bit!(8);
        bit!(7);
        bit!(6);
        bit!(5);
        bit!(4);
        bit!(3);
        bit!(2);
        bit!(1);
        bit!(0);
        let _ = normalized;

        let computed = integer.wrapping_shl(16).wrapping_add(fraction as i32);
        let local = zero.select_u32(StabilityRefusal::UnsupportedDomain as u32, OK);
        SignedFixed {
            val: zero.select_i32(i32::MIN, computed),
            err: branchless_err_acc(self.err, local),
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SignedFixed {
    pub val: i32,
    pub err: u32,
}

impl SignedFixed {
    pub const ZERO: Self = Self { val: 0, err: OK };
    pub const ONE: Self = Self {
        val: 65_536,
        err: OK,
    };
    pub const MAX: Self = Self {
        val: i32::MAX,
        err: OK,
    };
    pub const MIN: Self = Self {
        val: i32::MIN,
        err: OK,
    };

    #[inline(always)]
    pub const fn from_bits(bits: i32) -> Self {
        Self { val: bits, err: OK }
    }

    #[inline(always)]
    pub const fn from_bits_with_error(bits: i32, err: u32) -> Self {
        Self { val: bits, err }
    }

    #[inline(always)]
    pub const fn to_bits(self) -> i32 {
        self.val
    }

    #[inline(always)]
    pub const fn from_num(number: i32) -> Self {
        let shifted = (number as i64) << 16;
        let overflow = CanonicalMask {
            val: 0u32.wrapping_sub((shifted > i32::MAX as i64 || shifted < i32::MIN as i64) as u32),
        };
        let saturation = const_lt_i32(number, 0).select_i32(i32::MIN, i32::MAX);
        Self {
            val: overflow.select_i32(saturation, shifted as i32),
            err: overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK),
        }
    }

    #[inline(always)]
    pub const fn to_num(self) -> i32 {
        self.val >> 16
    }

    #[inline(always)]
    pub const fn saturating_add(self, other: Self) -> Self {
        let (sum, did_overflow) = self.val.overflowing_add(other.val);
        let overflow = CanonicalMask {
            val: 0u32.wrapping_sub(did_overflow as u32),
        };
        let saturation = const_lt_i32(self.val, 0).select_i32(i32::MIN, i32::MAX);
        let local = overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK);
        Self {
            val: overflow.select_i32(saturation, sum),
            err: branchless_err_acc(self.err, branchless_err_acc(other.err, local)),
        }
    }

    #[inline(always)]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let (difference, did_overflow) = self.val.overflowing_sub(other.val);
        let overflow = CanonicalMask {
            val: 0u32.wrapping_sub(did_overflow as u32),
        };
        let saturation = const_lt_i32(self.val, 0).select_i32(i32::MIN, i32::MAX);
        let local = overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK);
        Self {
            val: overflow.select_i32(saturation, difference),
            err: branchless_err_acc(self.err, branchless_err_acc(other.err, local)),
        }
    }

    #[inline(always)]
    pub const fn saturating_mul(self, other: Self) -> Self {
        let product = (self.val as i64) * (other.val as i64);
        let shifted = product >> 16;
        let above = CanonicalMask {
            val: 0u32.wrapping_sub((shifted > i32::MAX as i64) as u32),
        };
        let below = CanonicalMask {
            val: 0u32.wrapping_sub((shifted < i32::MIN as i64) as u32),
        };
        let mut value = below.select_i32(i32::MIN, shifted as i32);
        value = above.select_i32(i32::MAX, value);
        let overflow = CanonicalMask {
            val: above.val | below.val,
        };
        let local = overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK);
        Self {
            val: value,
            err: branchless_err_acc(self.err, branchless_err_acc(other.err, local)),
        }
    }

    /// Deterministic Q16.16 `2^x`. The sixteen fractional multipliers are
    /// rounded Q32 constants. The implementation has no libm dependency and
    /// returns a typed range refusal outside `[-16, 16)`.
    #[inline(always)]
    pub fn exp2(self) -> NonNegativeFixed {
        const ROOTS: [u64; 16] = [
            6_074_001_000,
            5_107_605_667,
            4_683_695_048,
            4_485_121_744,
            4_389_014_833,
            4_341_736_423,
            4_318_288_544,
            4_306_612_134,
            4_300_785_774,
            4_297_875_550,
            4_296_421_177,
            4_295_694_175,
            4_295_330_720,
            4_295_149_004,
            4_295_058_149,
            4_295_012_722,
        ];

        let integer = self.val >> 16;
        let fraction = self.val.wrapping_sub(integer.wrapping_shl(16)) as u32;
        let mut value = 1u64 << 32;

        macro_rules! factor {
            ($index:expr, $bit:expr) => {{
                let multiplied = ((value as u128 * ROOTS[$index] as u128) >> 32) as u64;
                let mask = CanonicalMask {
                    val: 0u32.wrapping_sub(((fraction >> $bit) & 1) as u32),
                };
                value = (mask.select_u32(multiplied as u32, value as u32) as u64)
                    | ((mask.select_u32((multiplied >> 32) as u32, (value >> 32) as u32) as u64)
                        << 32);
            }};
        }
        factor!(0, 15);
        factor!(1, 14);
        factor!(2, 13);
        factor!(3, 12);
        factor!(4, 11);
        factor!(5, 10);
        factor!(6, 9);
        factor!(7, 8);
        factor!(8, 7);
        factor!(9, 6);
        factor!(10, 5);
        factor!(11, 4);
        factor!(12, 3);
        factor!(13, 2);
        factor!(14, 1);
        factor!(15, 0);

        let overflow = CanonicalMask {
            val: 0u32.wrapping_sub((integer >= 16) as u32),
        };
        let underflow = CanonicalMask {
            val: 0u32.wrapping_sub((integer < -16) as u32),
        };
        let positive = CanonicalMask {
            val: 0u32.wrapping_sub((integer >= 0) as u32),
        };
        let positive_value = (value << ((integer as u32) & 31)) >> 16;
        let negative_value = value >> ((16 + integer.wrapping_neg()) as u32 & 63);
        let finite = positive.select_u32(positive_value as u32, negative_value as u32);
        let range = CanonicalMask {
            val: overflow.val | underflow.val,
        };
        let local = range.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK);
        NonNegativeFixed {
            val: overflow.select_u32(u32::MAX, underflow.select_u32(0, finite)),
            err: branchless_err_acc(self.err, local),
        }
    }

    #[inline(always)]
    pub fn exp(self) -> NonNegativeFixed {
        let product = (self.val as i64) * 94_548i64;
        let shifted = product >> 16;
        let above = shifted > i32::MAX as i64;
        let below = shifted < i32::MIN as i64;
        let overflow = CanonicalMask {
            val: 0u32.wrapping_sub((above || below) as u32),
        };
        let saturation = const_lt_i32(self.val, 0).select_i32(i32::MIN, i32::MAX);
        SignedFixed {
            val: overflow.select_i32(saturation, shifted as i32),
            err: branchless_err_acc(
                self.err,
                overflow.select_u32(StabilityRefusal::NumericRangeExceeded as u32, OK),
            ),
        }
        .exp2()
    }
}

impl core::ops::Add for NonNegativeFixed {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self::Output {
        self.saturating_add(other)
    }
}
impl core::ops::Sub for NonNegativeFixed {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self::Output {
        self.saturating_sub(other)
    }
}
impl core::ops::Mul for NonNegativeFixed {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: Self) -> Self::Output {
        self.saturating_mul(other)
    }
}
impl core::ops::Div for NonNegativeFixed {
    type Output = Self;
    #[inline(always)]
    fn div(self, other: Self) -> Self::Output {
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
    fn add(self, other: Self) -> Self::Output {
        self.saturating_add(other)
    }
}
impl core::ops::Sub for SignedFixed {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self::Output {
        self.saturating_sub(other)
    }
}
impl core::ops::Mul for SignedFixed {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: Self) -> Self::Output {
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
    fn complete_selection_preserves_error() {
        let poisoned = NonNegativeFixed::from_bits_with_error(
            7,
            StabilityRefusal::NumericRangeExceeded as u32,
        );
        assert_eq!(
            CanonicalMask::TRUE.select_nonnegative(poisoned, NonNegativeFixed::ZERO),
            poisoned
        );
    }

    #[test]
    fn log2_and_exp2_are_near_inverse() {
        for raw in [6, 1_024, 32_768, 65_536, 131_072, 65_536_000] {
            let value = NonNegativeFixed::from_bits(raw);
            let round_trip = value.log2().exp2();
            assert_eq!(round_trip.err, OK);
            assert!(
                round_trip.val.abs_diff(raw) <= 4,
                "raw={raw}, got={}",
                round_trip.val
            );
        }
    }
}
