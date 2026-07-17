//! Q16.16 Fixed-Point Arithmetic Substrate
//!
//! This module provides a deterministic, branchless, allocation-free Q16.16 fixed-point representation
//! for bounded computational substrates under Radon Law ($CC=1$).
//!
//! # Mathematical Layout
//! A real number $r$ is represented as a 32-bit unsigned integer $x$ such that:
//!
//! $$r = x \times 2^{-16} = \frac{x}{65536}$$
//!
//! Consequently:
//! - The minimum representable value (resolution) is $\epsilon = 2^{-16} \approx 0.000015258$.
//! - The maximum representable value is $65535.999984741$ (raw bits `u32::MAX`).
//! - Operations saturate rather than wrap (except where explicitly noted).
//!
//! # Runtime Laws (Radon Law)
//! All operations in this module strictly enforce:
//! - **Cyclomatic Complexity ($CC = 1$)**: No data-dependent branches, loops, or early returns.
//! - **Zero Allocation**: $0$ heap allocations.
//! - **Constant-Time Mechanics**: The instruction flow is independent of input values.

/// Q16.16 Fixed-Point representation wrapping a `u32` under Radon Law (CC=1).
///
/// The value represents a real number scaled by 2^16 (65536).
/// It provides branchless addition, subtraction, multiplication, and division.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixed(pub u32);

impl core::fmt::Debug for Fixed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let int_part = self.0 >> 16;
        let frac_part = (((self.0 & 0xFFFF) as u64 * 100_000) / 65536) as u32;
        write!(f, "{}.{:05}", int_part, frac_part)
    }
}

impl core::fmt::Display for Fixed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let int_part = self.0 >> 16;
        let frac_part = (((self.0 & 0xFFFF) as u64 * 100_000) / 65536) as u32;
        write!(f, "{}.{:05}", int_part, frac_part)
    }
}

#[inline(always)]
const fn const_select_u32(condition: u32, a: u32, b: u32) -> u32 {
    let cond = (condition | condition.wrapping_neg()) >> 31;
    let mask = 0u32.wrapping_sub(cond);
    (a & mask) | (b & !mask)
}

#[inline(always)]
const fn const_lt_u32(a: u32, b: u32) -> u32 {
    ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1
}

#[inline(always)]
const fn const_eq_u32(a: u32, b: u32) -> u32 {
    let x = a ^ b;
    let nonzero = (x | x.wrapping_neg()) >> 31;
    1u32.wrapping_sub(nonzero)
}

impl Fixed {
    /// Zero representation ($0.0$)
    pub const ZERO: Self = Self(0);

    /// One representation ($1.0$)
    pub const ONE: Self = Self(65536);

    /// Maximum representation ($65535.99998$)
    pub const MAX: Self = Self(u32::MAX);

    /// Creates a `Fixed` value from its raw 32-bit representation.
    ///
    /// # Preconditions
    /// None. Any 32-bit unsigned value is a valid Q16.16 representation.
    ///
    /// # Postconditions
    /// Returns a `Fixed` value wrapping the exact raw bits provided.
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// let val = Fixed::from_bits(65536); // represents 1.0
    /// assert_eq!(val, Fixed::ONE);
    /// ```
    #[inline(always)]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Retrieves the underlying raw 32-bit representation.
    ///
    /// # Preconditions
    /// None.
    ///
    /// # Postconditions
    /// Returns the raw `u32` value.
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// assert_eq!(Fixed::ONE.to_bits(), 65536);
    /// ```
    #[inline(always)]
    pub const fn to_bits(self) -> u32 {
        self.0
    }

    /// Creates a `Fixed` value from a 32-bit integer.
    ///
    /// # Preconditions
    /// - The integer `num` should be in the range $[0, 65535]$ to prevent wrapping.
    /// - Input values $\ge 65536$ will wrap due to `wrapping_shl(16)`.
    ///
    /// # Postconditions
    /// Returns a `Fixed` value representing the integer value scaled by $2^{16}$.
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// let val = Fixed::from_num(5);
    /// assert_eq!(val.to_bits(), 5 * 65536);
    /// ```
    #[inline(always)]
    pub const fn from_num(num: u32) -> Self {
        Self(num.wrapping_shl(16))
    }

    /// Converts a `Fixed` value to its truncated integer part.
    ///
    /// # Preconditions
    /// None.
    ///
    /// # Postconditions
    /// Returns the integer part of the fixed-point value, discarding any fractional bits.
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// let val = Fixed::from_bits(98304); // represents 1.5
    /// assert_eq!(val.to_num(), 1);
    /// ```
    #[inline(always)]
    pub const fn to_num(self) -> u32 {
        self.0 >> 16
    }

    /// Saturating addition without branching ($CC=1$).
    ///
    /// # Mathematical Definition
    /// Let $x$ and $y$ be Q16.16 fixed-point values.
    ///
    /// $$z = \min(x + y, \text{MAX})$$
    ///
    /// # Preconditions
    /// None.
    ///
    /// # Postconditions
    /// - If the mathematical sum $x + y \le 65535.999984741$, the returned value is exactly $x + y$.
    /// - Otherwise, returns [`Fixed::MAX`].
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// let a = Fixed::from_num(10);
    /// let b = Fixed::from_num(20);
    /// assert_eq!(a.saturating_add(b), Fixed::from_num(30));
    ///
    /// // Overflow saturation
    /// let max_val = Fixed::MAX;
    /// assert_eq!(max_val.saturating_add(Fixed::ONE), Fixed::MAX);
    /// ```
    #[inline(always)]
    pub const fn saturating_add(self, other: Self) -> Self {
        let sum = self.0.wrapping_add(other.0);
        let overflow = const_lt_u32(sum, self.0);
        Self(const_select_u32(overflow, u32::MAX, sum))
    }

    /// Saturating subtraction without branching ($CC=1$).
    ///
    /// # Mathematical Definition
    /// Let $x$ and $y$ be Q16.16 fixed-point values.
    ///
    /// $$z = \max(x - y, 0)$$
    ///
    /// # Preconditions
    /// None.
    ///
    /// # Postconditions
    /// - If $x \ge y$, the returned value is exactly $x - y$.
    /// - Otherwise, returns [`Fixed::ZERO`].
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// let a = Fixed::from_num(30);
    /// let b = Fixed::from_num(10);
    /// assert_eq!(a.saturating_sub(b), Fixed::from_num(20));
    ///
    /// // Underflow saturation
    /// assert_eq!(b.saturating_sub(a), Fixed::ZERO);
    /// ```
    #[inline(always)]
    pub const fn saturating_sub(self, other: Self) -> Self {
        let underflow = const_lt_u32(self.0, other.0);
        Self(const_select_u32(underflow, 0, self.0.wrapping_sub(other.0)))
    }

    /// Saturating multiplication without branching ($CC=1$).
    ///
    /// # Mathematical Definition
    /// Let $x$ and $y$ be Q16.16 fixed-point values.
    ///
    /// $$z = \min(x \times y, \text{MAX})$$
    ///
    /// # Preconditions
    /// None.
    ///
    /// # Postconditions
    /// - Under fixed-point scaling, the product of $x$ and $y$ is computed with 64-bit precision, scaled back, and saturated.
    /// - If the mathematical product $x \times y \le 65535.999984741$, the returned value is exactly $x \times y$.
    /// - Otherwise, returns [`Fixed::MAX`].
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// let a = Fixed::from_bits(98304); // 1.5
    /// let b = Fixed::from_num(2);      // 2.0
    /// assert_eq!(a.saturating_mul(b), Fixed::from_num(3));
    ///
    /// // Overflow saturation
    /// assert_eq!(Fixed::MAX.saturating_mul(Fixed::from_num(2)), Fixed::MAX);
    /// ```
    #[inline(always)]
    pub const fn saturating_mul(self, other: Self) -> Self {
        let prod = (self.0 as u64).wrapping_mul(other.0 as u64);
        let res_u64 = prod >> 16;
        let high = (res_u64 >> 32) as u32;
        let overflow = (high | high.wrapping_neg()) >> 31;
        Self(const_select_u32(overflow, u32::MAX, res_u64 as u32))
    }

    /// Saturating division without branching ($CC=1$) using Newton-Raphson reciprocal approximation.
    ///
    /// # Mathematical Definition
    /// Let $n$ be the numerator (`self`) and $d$ be the denominator (`other`).
    ///
    /// $$q = \min\left(\frac{n}{d}, \text{MAX}\right)$$
    ///
    /// # Algorithm
    /// The division is computed in constant time without conditional branching:
    /// 1. Denominator normalization via leading zeros count.
    /// 2. Initial linear reciprocal guess:
    ///    $$X_0 = A_{\text{scale}} - B_{\text{coeff}} \times d_{\text{norm}}$$
    /// 3. Three iterations of Newton-Raphson correction:
    ///    $$X_{k+1} = X_k \left(2 - d_{\text{norm}} \times X_k\right)$$
    /// 4. Quotient reconstruction and remainder-based rounding correction.
    ///
    /// # Preconditions
    /// None. If the denominator is zero, the division saturates to [`Fixed::MAX`].
    ///
    /// # Postconditions
    /// - If $d = 0$, returns [`Fixed::MAX`].
    /// - If $n / d > 65535.999984741$, returns [`Fixed::MAX`].
    /// - Otherwise, returns $n / d$ with 1-bit accuracy (with remainder correction).
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// let a = Fixed::from_num(3);
    /// let b = Fixed::from_bits(98304); // 1.5
    /// assert_eq!(a.saturating_div(b), Fixed::from_num(2));
    ///
    /// // Division by zero saturation
    /// assert_eq!(a.saturating_div(Fixed::ZERO), Fixed::MAX);
    ///
    /// // Overflow saturation
    /// assert_eq!(Fixed::MAX.saturating_div(Fixed::from_bits(32768)), Fixed::MAX); // divide by 0.5
    /// ```
    #[inline(always)]
    pub const fn saturating_div(self, other: Self) -> Self {
        let den_is_zero = const_eq_u32(other.0, 0);
        // Safety divisor to prevent clz(0) undefined behavior
        let d = const_select_u32(den_is_zero, 1, other.0);
        
        let lz = d.leading_zeros();
        let d_norm = d << lz;
        
        // Initial linear guess (64-bit Q2.62): X_0 = A_scale - B_coeff * d_norm
        // A_scale = 3031742610 << 32 = 13021703673752174592
        // B_coeff = 2021160080
        let a_scale = 13021703673752174592u64;
        let b_coeff = 2021160080u64;
        let x0 = a_scale.wrapping_sub(b_coeff.wrapping_mul(d_norm as u64));
        
        // Iteration 1: e0 = 2^94 - d_norm * x0 (signed i128)
        let e0 = (1i128 << 94) - (d_norm as i128) * (x0 as i128);
        let x1 = ((x0 as i128) + (((x0 as i128) * (e0 >> 32)) >> 62)) as u64;
        
        // Iteration 2: e1 = 2^94 - d_norm * x1
        let e1 = (1i128 << 94) - (d_norm as i128) * (x1 as i128);
        let x2 = ((x1 as i128) + (((x1 as i128) * (e1 >> 32)) >> 62)) as u64;
        
        // Iteration 3: e2 = 2^94 - d_norm * x2
        let e2 = (1i128 << 94) - (d_norm as i128) * (x2 as i128);
        let x3 = ((x2 as i128) + (((x2 as i128) * (e2 >> 32)) >> 62)) as u64;
        
        // Compute uncorrected quotient: q = (n * x3) >> (78 - lz)
        let n = self.0 as u128;
        let q = (n.wrapping_mul(x3 as u128)) >> (78 - lz);
        
        // Remainder: rem = (n << 16) - q * d
        let rem = ((self.0 as u64) << 16).wrapping_sub((q as u64).wrapping_mul(d as u64)) as i64;
        
        // Branchless correction using sign bit extraction
        let is_lt = ((rem >> 63) & 1) as u64;
        let diff = rem.wrapping_sub(d as i64);
        let is_ge = (((!diff) >> 63) & 1) as u64;
        
        let q_corrected = (q as u64).wrapping_add(is_ge).wrapping_sub(is_lt);
        
        // Saturate check
        let overflow = const_lt_u32(u32::MAX, (q_corrected >> 32) as u32) | (q_corrected > u32::MAX as u64) as u32;
        let saturate = overflow | den_is_zero;
        
        Self(const_select_u32(saturate, u32::MAX, q_corrected as u32))
    }

    /// Branchless Q16.16 binary logarithm ($CC=1$).
    ///
    /// # Mathematical Definition
    /// Let $x$ be the input Q16.16 value.
    ///
    /// $$y = \log_2(x)$$
    ///
    /// # Preconditions
    /// - Mathematical logarithm is undefined for $x \le 0$.
    /// - In this implementation, $x = 0$ is handled safely without branching/panicking, returning a value representing $-16.0$.
    ///
    /// # Postconditions
    /// - For $x > 0$, returns $\log_2(x)$ as a signed Q16.16 value wrapped in [`Fixed`].
    /// - For $x = 0$, returns a raw value representing $-16.0$ (due to underflow handling).
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// // log2(1.0) = 0.0
    /// assert_eq!(Fixed::ONE.log2().to_bits(), 0);
    ///
    /// // log2(2.0) = 1.0
    /// assert_eq!(Fixed::from_num(2).log2(), Fixed::ONE);
    ///
    /// // log2(4.0) = 2.0
    /// assert_eq!(Fixed::from_num(4).log2(), Fixed::from_num(2));
    /// ```
    #[inline(always)]
    pub fn log2(self) -> Self {
        let val = self.0 as u64;
        let lz = val.leading_zeros(); // 64 when val == 0
        let nz = ((val | val.wrapping_neg()) >> 63) & 1; // 1 iff val != 0
        let ip = 63u64.wrapping_sub(lz as u64) & nz.wrapping_neg(); // 0 when val == 0
        
        let mantissa = val.wrapping_shl(lz.wrapping_add(1));
        let f = (mantissa >> (64 - 16)) as u32; // Q16.16 fraction f
        
        let diff = 65536 - f;
        let correction = (f * diff) >> 16;
        let corrected_frac = f + ((correction * 29013) >> 16);
        
        let res = (ip << 16).wrapping_add(corrected_frac as u64);
        Self((res as u32).wrapping_sub(16 << 16))
    }

    /// Branchless Q16.16 base-2 exponential ($CC=1$) supporting both positive and negative exponents.
    ///
    /// # Mathematical Definition
    /// Let $x$ be the input Q16.16 value.
    ///
    /// $$y = \min\left(2^x, \text{MAX}\right)$$
    ///
    /// # Preconditions
    /// None.
    ///
    /// # Postconditions
    /// - If the input represents $x \ge 16.0$, the result saturates to [`Fixed::MAX`].
    /// - If the input represents $x \le -17.0$, the result saturates to [`Fixed::ZERO`].
    /// - Otherwise, returns $2^x$ using a polynomial approximation for the fractional part.
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// // exp2(0.0) = 1.0
    /// assert_eq!(Fixed::ZERO.exp2(), Fixed::ONE);
    ///
    /// // exp2(1.0) = 2.0
    /// assert_eq!(Fixed::ONE.exp2(), Fixed::from_num(2));
    ///
    /// // exp2(-1.0) = 0.5
    /// assert_eq!(Fixed(4294901760).exp2().to_bits(), 32768);
    /// ```
    #[inline(always)]
    pub fn exp2(self) -> Self {
        let x = self.0 as i32;
        let ip = x >> 16;
        let fp = x.wrapping_sub(ip.wrapping_shl(16)); // fp in [0, 65535]
        
        let y = fp as u32;
        let res1 = (y.wrapping_mul(630)) >> 16;
        let res2 = (y.wrapping_mul(3637u32.wrapping_add(res1))) >> 16;
        let res3 = (y.wrapping_mul(15763u32.wrapping_add(res2))) >> 16;
        let res4 = (y.wrapping_mul(45506u32.wrapping_add(res3))) >> 16;
        let frac_part = 65536u32.wrapping_add(res4);
        
        let is_overflow = (((ip.wrapping_sub(16)) >> 31) ^ 1) & 1;
        let is_underflow = ((((-17i32).wrapping_sub(ip)) >> 31) ^ 1) & 1;
        
        let shl = ip & 31;
        let shr = (ip.wrapping_neg()) & 31;
        
        let val_shl = frac_part.wrapping_shl(shl as u32);
        let val_shr = frac_part.wrapping_shr(shr as u32);
        
        let ip_neg = ((ip >> 31) & 1) as u32;
        let val_shifted = const_select_u32(ip_neg, val_shr, val_shl);
        
        let res = const_select_u32(is_overflow as u32, u32::MAX,
                    const_select_u32(is_underflow as u32, 0, val_shifted));
        
        Self(res)
    }

    /// Branchless Q16.16 natural exponential ($CC=1$).
    ///
    /// # Mathematical Definition
    /// Let $x$ be the input Q16.16 value.
    ///
    /// $$y = e^x = 2^{x \log_2(e)}$$
    ///
    /// # Algorithm
    /// The natural exponential is computed by scaling the exponent by $\log_2(e) \approx 1.44269504$:
    ///
    /// $$y = 2^{x \times 1.44269504}$$
    ///
    /// # Preconditions
    /// None.
    ///
    /// # Postconditions
    /// - If $x \ge 11.09035$, the result saturates to [`Fixed::MAX`].
    /// - If $x \le -11.7835$, the result saturates to [`Fixed::ZERO`].
    /// - Otherwise, returns $e^x$.
    ///
    /// # Examples
    /// ```
    /// use bcinr_cmca::fixed::Fixed;
    ///
    /// // exp(0.0) = 1.0
    /// assert_eq!(Fixed::ZERO.exp(), Fixed::ONE);
    /// ```
    #[inline(always)]
    pub fn exp(self) -> Self {
        let x = self.0 as i32;
        // log2(e) ≈ 1.44269504, Q16.16 = 94548
        let z = (((x as i64).wrapping_mul(94548)) >> 16) as i32;
        Self(z as u32).exp2()
    }
}

impl core::ops::Add for Fixed {
    type Output = Self;
    #[inline(always)]
    fn add(self, other: Self) -> Self {
        self.saturating_add(other)
    }
}

impl core::ops::Sub for Fixed {
    type Output = Self;
    #[inline(always)]
    fn sub(self, other: Self) -> Self {
        self.saturating_sub(other)
    }
}

impl core::ops::Mul for Fixed {
    type Output = Self;
    #[inline(always)]
    fn mul(self, other: Self) -> Self {
        self.saturating_mul(other)
    }
}

impl core::ops::Div for Fixed {
    type Output = Self;
    #[inline(always)]
    fn div(self, other: Self) -> Self {
        self.saturating_div(other)
    }
}

impl core::ops::AddAssign for Fixed {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl core::ops::SubAssign for Fixed {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl core::ops::MulAssign for Fixed {
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl core::ops::DivAssign for Fixed {
    #[inline(always)]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_conversions() {
        assert_eq!(Fixed::from_num(5).to_num(), 5);
        assert_eq!(Fixed::from_num(0).to_num(), 0);
        assert_eq!(Fixed::ONE.to_bits(), 65536);
    }

    #[test]
    fn test_fixed_add() {
        let a = Fixed::from_num(10);
        let b = Fixed::from_num(20);
        assert_eq!((a + b).to_num(), 30);

        // Saturating
        let max_val = Fixed::MAX;
        assert_eq!((max_val + Fixed::ONE).to_bits(), u32::MAX);
    }

    #[test]
    fn test_fixed_sub() {
        let a = Fixed::from_num(30);
        let b = Fixed::from_num(10);
        assert_eq!((a - b).to_num(), 20);

        // Saturating
        assert_eq!((b - a).to_bits(), 0);
    }

    #[test]
    fn test_fixed_mul() {
        // 1.5 * 2.0 = 3.0
        let a = Fixed::from_bits(98304); // 1.5
        let b = Fixed::from_num(2);
        assert_eq!((a * b).to_bits(), 196608); // 3.0
        assert_eq!((a * b).to_num(), 3);

        // Saturating
        let max_val = Fixed::MAX;
        assert_eq!((max_val * Fixed::from_num(2)).to_bits(), u32::MAX);
    }

    #[test]
    fn test_fixed_div() {
        // 3.0 / 1.5 = 2.0
        let a = Fixed::from_num(3);
        let b = Fixed::from_bits(98304); // 1.5
        assert_eq!((a / b).to_num(), 2);

        // Div by zero
        assert_eq!((a / Fixed::ZERO).to_bits(), u32::MAX);

        // Saturating
        let max_val = Fixed::MAX;
        assert_eq!((max_val / Fixed::from_bits(32768)).to_bits(), u32::MAX); // divide by 0.5
    }

    #[test]
    fn test_fixed_log2_exp2_exp() {
        // log2(1.0) = 0.0
        assert_eq!(Fixed::ONE.log2().to_bits(), 0);
        // log2(2.0) = 1.0
        assert_eq!(Fixed::from_num(2).log2().to_bits(), 65536);
        // log2(4.0) = 2.0
        assert_eq!(Fixed::from_num(4).log2().to_bits(), 131072);

        // exp2(0.0) = 1.0
        assert_eq!(Fixed::ZERO.exp2().to_bits(), 65536);
        // exp2(1.0) = 2.0
        assert_eq!(Fixed::ONE.exp2().to_bits(), 131072);
        // exp2(-1.0) = 0.5
        assert_eq!(Fixed(4294901760).exp2().to_bits(), 32768);

        // exp(0) = 1.0
        assert_eq!(Fixed::ZERO.exp().to_bits(), 65536);
    }
}
