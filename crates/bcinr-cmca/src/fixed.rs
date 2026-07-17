//! Q16.16 Fixed-Point Arithmetic Substrate
//! Branchless Contract
//! This module provides a deterministic, branchless, allocation-free Q16.16 fixed-point representation
//! for bounded computational substrates under Radon Law ($CC=1$).
//!
//! # Mathematical Layout
//! Real numbers are separated into strict domains:
//! - `NonNegativeFixed`: mathematical values in $[0.0, 65535.999984741]$, wrapping `u32`.
//! - `SignedFixed`: mathematical values in $[-32768.0, 32767.999984741]$, wrapping `i32`.
//! - `CanonicalMask`: exactly `0` or `u32::MAX`, used for branchless selection.

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalMask(pub u32);

impl CanonicalMask {
    pub const TRUE: Self = Self(u32::MAX);
    pub const FALSE: Self = Self(0);

    #[inline(always)]
    pub const fn select_u32(self, a: u32, b: u32) -> u32 {
        (a & self.0) | (b & !self.0)
    }

    #[inline(always)]
    pub const fn select_i32(self, a: i32, b: i32) -> i32 {
        (a & self.0 as i32) | (b & !(self.0 as i32))
    }
}

#[inline(always)]
pub const fn const_lt_u32(a: u32, b: u32) -> CanonicalMask {
    let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
    CanonicalMask(0u32.wrapping_sub(diff))
}

#[inline(always)]
pub const fn const_eq_u32(a: u32, b: u32) -> CanonicalMask {
    let x = a ^ b;
    let nonzero = (x | x.wrapping_neg()) >> 31;
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

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonNegativeFixed(pub u32);

impl core::fmt::Debug for NonNegativeFixed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let int_part = self.0 >> 16;
        let frac_part = (((self.0 & 0xFFFF) as u64 * 100_000) / 65536) as u32;
        write!(f, "{}.{:05}", int_part, frac_part)
    }
}

impl core::fmt::Display for NonNegativeFixed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let int_part = self.0 >> 16;
        let frac_part = (((self.0 & 0xFFFF) as u64 * 100_000) / 65536) as u32;
        write!(f, "{}.{:05}", int_part, frac_part)
    }
}

impl NonNegativeFixed {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(65536);
    pub const MAX: Self = Self(u32::MAX);

    #[inline(always)]
    pub const fn from_bits(bits: u32) -> Self { Self(bits) }

    #[inline(always)]
    pub const fn to_bits(self) -> u32 { self.0 }

    #[inline(always)]
    pub const fn from_num(num: u32) -> Self { Self(num.wrapping_shl(16)) }

    #[inline(always)]
    pub const fn to_num(self) -> u32 { self.0 >> 16 }

    #[inline(always)]
    pub const fn saturating_add(self, other: Self) -> Self {
        let sum = self.0.wrapping_add(other.0);
        let overflow = const_lt_u32(sum, self.0);
        Self(overflow.select_u32(u32::MAX, sum))
    }

    #[inline(always)]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let underflow = const_lt_u32(self.0, other.0);
        Self(underflow.select_u32(0, self.0.wrapping_sub(other.0)))
    }

    #[inline(always)]
    pub const fn saturating_mul(self, other: Self) -> Self {
        let prod = (self.0 as u64).wrapping_mul(other.0 as u64);
        let res_u64 = prod >> 16;
        let high = (res_u64 >> 32) as u32;
        let overflow = (high | high.wrapping_neg()) >> 31;
        let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow));
        Self(overflow_mask.select_u32(u32::MAX, res_u64 as u32))
    }

    #[inline(always)]
    pub const fn saturating_div(self, other: Self) -> Self {
        let den_is_zero = const_eq_u32(other.0, 0);
        let d = den_is_zero.select_u32(1, other.0);
        
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
        
        let n = self.0 as u128;
        let q = (n.wrapping_mul(x3 as u128)) >> (78 - lz);
        
        let rem = ((self.0 as u64) << 16).wrapping_sub((q as u64).wrapping_mul(d as u64)) as i64;
        
        let is_lt = ((rem >> 63) & 1) as u64;
        let diff = rem.wrapping_sub(d as i64);
        let is_ge = (((!diff) >> 63) & 1) as u64;
        
        let q_corrected = (q as u64).wrapping_add(is_ge).wrapping_sub(is_lt);
        
        let overflow_1 = const_lt_u32(u32::MAX, (q_corrected >> 32) as u32).0;
        let overflow_2 = 0u32.wrapping_sub((q_corrected > u32::MAX as u64) as u32);
        let overflow = CanonicalMask(overflow_1 | overflow_2);
        
        let saturate = CanonicalMask(overflow.0 | den_is_zero.0);
        Self(saturate.select_u32(u32::MAX, q_corrected as u32))
    }

    #[inline(always)]
    pub fn log2(self) -> SignedFixed {
        let val = self.0 as u64;
        let lz = val.leading_zeros(); 
        let nz = ((val | val.wrapping_neg()) >> 63) & 1; 
        let ip = 63u64.wrapping_sub(lz as u64) & nz.wrapping_neg(); 
        
        let mantissa = val.wrapping_shl(lz.wrapping_add(1));
        let f = (mantissa >> (64 - 16)) as u32; 
        
        let diff = 65536 - f;
        let correction = (f * diff) >> 16;
        let corrected_frac = f + ((correction * 29013) >> 16);
        
        let res = (ip << 16).wrapping_add(corrected_frac as u64);
        
        let is_zero = const_eq_u32(self.0, 0);
        let computed = (res as u32).wrapping_sub(16 << 16) as i32;
        SignedFixed(is_zero.select_i32(-1048576, computed))
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedFixed(pub i32);

impl core::fmt::Debug for SignedFixed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let abs_val = self.0.unsigned_abs();
        let int_part = abs_val >> 16;
        let frac_part = (((abs_val & 0xFFFF) as u64 * 100_000) / 65536) as u32;
        if self.0 < 0 {
            write!(f, "-{}.{:05}", int_part, frac_part)
        } else {
            write!(f, "{}.{:05}", int_part, frac_part)
        }
    }
}

impl core::fmt::Display for SignedFixed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let abs_val = self.0.unsigned_abs();
        let int_part = abs_val >> 16;
        let frac_part = (((abs_val & 0xFFFF) as u64 * 100_000) / 65536) as u32;
        if self.0 < 0 {
            write!(f, "-{}.{:05}", int_part, frac_part)
        } else {
            write!(f, "{}.{:05}", int_part, frac_part)
        }
    }
}

impl SignedFixed {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(65536);
    pub const MAX: Self = Self(i32::MAX);
    pub const MIN: Self = Self(i32::MIN);

    #[inline(always)]
    pub const fn from_bits(bits: i32) -> Self { Self(bits) }

    #[inline(always)]
    pub const fn to_bits(self) -> i32 { self.0 }

    #[inline(always)]
    pub const fn from_num(num: i32) -> Self { Self(num.wrapping_shl(16)) }

    #[inline(always)]
    pub const fn to_num(self) -> i32 { self.0 >> 16 }

    #[inline(always)]
    pub const fn saturating_add(self, other: Self) -> Self {
        let (sum, overflow) = self.0.overflowing_add(other.0);
        let is_neg = const_lt_i32(self.0, 0);
        let sat_val = is_neg.select_i32(i32::MIN, i32::MAX);
        let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow as u32));
        Self(overflow_mask.select_i32(sat_val, sum))
    }

    #[inline(always)]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let (diff, overflow) = self.0.overflowing_sub(other.0);
        let is_neg = const_lt_i32(self.0, 0);
        let sat_val = is_neg.select_i32(i32::MIN, i32::MAX);
        let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow as u32));
        Self(overflow_mask.select_i32(sat_val, diff))
    }
    
    #[inline(always)]
    pub const fn saturating_mul(self, other: Self) -> Self {
        let prod = (self.0 as i64).wrapping_mul(other.0 as i64);
        let res_i64 = prod >> 16;
        
        let overflow_max = CanonicalMask(0u32.wrapping_sub((res_i64 > i32::MAX as i64) as u32));
        let overflow_min = CanonicalMask(0u32.wrapping_sub((res_i64 < i32::MIN as i64) as u32));
        
        let mut res = overflow_min.select_i32(i32::MIN, res_i64 as i32);
        res = overflow_max.select_i32(i32::MAX, res);
        Self(res)
    }

    #[inline(always)]
    pub fn exp2(self) -> NonNegativeFixed {
        let x = self.0;
        let ip = x >> 16;
        let fp = x.wrapping_sub(ip.wrapping_shl(16));
        
        let y = fp as u32;
        let res1 = (y.wrapping_mul(630)) >> 16;
        let res2 = (y.wrapping_mul(3637u32.wrapping_add(res1))) >> 16;
        let res3 = (y.wrapping_mul(15763u32.wrapping_add(res2))) >> 16;
        let res4 = (y.wrapping_mul(45506u32.wrapping_add(res3))) >> 16;
        let frac_part = 65536u32.wrapping_add(res4);
        
        let is_overflow = CanonicalMask(0u32.wrapping_sub(((((ip.wrapping_sub(16)) >> 31) ^ 1) & 1) as u32));
        let is_underflow = CanonicalMask(0u32.wrapping_sub((((((-17i32).wrapping_sub(ip)) >> 31) ^ 1) & 1) as u32));
        
        let shl = (ip & 31) as u32;
        let shr = ((ip.wrapping_neg()) & 31) as u32;
        
        let val_shl = frac_part.wrapping_shl(shl);
        let val_shr = frac_part.wrapping_shr(shr);
        
        let ip_neg = CanonicalMask(0u32.wrapping_sub(((ip >> 31) & 1) as u32));
        let val_shifted = ip_neg.select_u32(val_shr, val_shl);
        
        let res = is_overflow.select_u32(u32::MAX,
                    is_underflow.select_u32(0, val_shifted));
        
        NonNegativeFixed(res)
    }

    #[inline(always)]
    pub fn exp(self) -> NonNegativeFixed {
        let x = self.0;
        let z = (((x as i64).wrapping_mul(94548)) >> 16) as i32;
        Self(z).exp2()
    }
}

// Aliases for transition to avoid immediate massive breakages, but we will remove `Fixed` entirely to strictly enforce domains.
// To ensure compile breaks everywhere and force a full audit, `Fixed` is not aliased.

impl core::ops::Add for NonNegativeFixed { type Output = Self; #[inline(always)] fn add(self, other: Self) -> Self { self.saturating_add(other) } }
impl core::ops::Sub for NonNegativeFixed { type Output = Self; #[inline(always)] fn sub(self, other: Self) -> Self { self.saturating_sub(other) } }
impl core::ops::Mul for NonNegativeFixed { type Output = Self; #[inline(always)] fn mul(self, other: Self) -> Self { self.saturating_mul(other) } }
impl core::ops::Div for NonNegativeFixed { type Output = Self; #[inline(always)] fn div(self, other: Self) -> Self { self.saturating_div(other) } }
impl core::ops::AddAssign for NonNegativeFixed { #[inline(always)] fn add_assign(&mut self, other: Self) { *self = *self + other; } }
impl core::ops::SubAssign for NonNegativeFixed { #[inline(always)] fn sub_assign(&mut self, other: Self) { *self = *self - other; } }
impl core::ops::MulAssign for NonNegativeFixed { #[inline(always)] fn mul_assign(&mut self, other: Self) { *self = *self * other; } }
impl core::ops::DivAssign for NonNegativeFixed { #[inline(always)] fn div_assign(&mut self, other: Self) { *self = *self / other; } }

impl core::ops::Add for SignedFixed { type Output = Self; #[inline(always)] fn add(self, other: Self) -> Self { self.saturating_add(other) } }
impl core::ops::Sub for SignedFixed { type Output = Self; #[inline(always)] fn sub(self, other: Self) -> Self { self.saturating_sub(other) } }
impl core::ops::Mul for SignedFixed { type Output = Self; #[inline(always)] fn mul(self, other: Self) -> Self { self.saturating_mul(other) } }
impl core::ops::AddAssign for SignedFixed { #[inline(always)] fn add_assign(&mut self, other: Self) { *self = *self + other; } }
impl core::ops::SubAssign for SignedFixed { #[inline(always)] fn sub_assign(&mut self, other: Self) { *self = *self - other; } }
impl core::ops::MulAssign for SignedFixed { #[inline(always)] fn mul_assign(&mut self, other: Self) { *self = *self * other; } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_conversions() {
        assert_eq!(NonNegativeFixed::from_num(5).to_num(), 5);
        assert_eq!(NonNegativeFixed::from_num(0).to_num(), 0);
        assert_eq!(NonNegativeFixed::ONE.to_bits(), 65536);
        assert_eq!(SignedFixed::from_num(-5).to_num(), -5);
    }

    #[test]
    fn test_nn_fixed_add() {
        let a = NonNegativeFixed::from_num(10);
        let b = NonNegativeFixed::from_num(20);
        assert_eq!((a + b).to_num(), 30);
        let max_val = NonNegativeFixed::MAX;
        assert_eq!((max_val + NonNegativeFixed::ONE).to_bits(), u32::MAX);
    }

    #[test]
    fn test_nn_fixed_sub() {
        let a = NonNegativeFixed::from_num(30);
        let b = NonNegativeFixed::from_num(10);
        assert_eq!((a - b).to_num(), 20);
        assert_eq!((b - a).to_bits(), 0); // floors at 0
    }

    #[test]
    fn test_nn_fixed_mul() {
        let a = NonNegativeFixed::from_bits(98304); 
        let b = NonNegativeFixed::from_num(2);
        assert_eq!((a * b).to_num(), 3);
        let max_val = NonNegativeFixed::MAX;
        assert_eq!((max_val * NonNegativeFixed::from_num(2)).to_bits(), u32::MAX);
    }

    #[test]
    fn test_nn_fixed_div() {
        let a = NonNegativeFixed::from_num(3);
        let b = NonNegativeFixed::from_bits(98304); 
        assert_eq!((a / b).to_num(), 2);
        assert_eq!((a / NonNegativeFixed::ZERO).to_bits(), u32::MAX);
    }

    #[test]
    fn test_fixed_log2_exp2_exp() {
        assert_eq!(NonNegativeFixed::ONE.log2().to_bits(), 0);
        assert_eq!(NonNegativeFixed::from_num(2).log2(), SignedFixed::ONE);
        assert_eq!(NonNegativeFixed::from_num(4).log2(), SignedFixed::from_num(2));

        assert_eq!(SignedFixed::ZERO.exp2(), NonNegativeFixed::ONE);
        assert_eq!(SignedFixed::ONE.exp2(), NonNegativeFixed::from_num(2));
        assert_eq!(SignedFixed(-65536).exp2().to_bits(), 32768); // 2^-1 = 0.5
        
        assert_eq!(SignedFixed::ZERO.exp(), NonNegativeFixed::ONE);
    }
}
