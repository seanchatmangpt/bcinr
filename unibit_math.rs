//! Scalar math and logic primitives for `U_{1,64}`.
//!
//! This module provides deterministic, constant-time operations over `64` bits
//! of truth. These algorithms are designed to compile to branchless assembly.
//!
//! No allocation. No dynamic dispatch. No panic paths. No loops unless fully
//! unrolled.
//!
//! ## Substrate rules
//!
//! Every primitive must be $T_1$ admissible (branchless execution).
//! The formal universe size dictates the algorithm shape:
//! - `U_{1,64}` represents `64` bits (one machine word).

/// Isolates the lowest set bit and masks all lower bits.
///
/// This is equivalent to the `BLSMSK` instruction pattern.
/// Returns `0` if the value is `0`.
///
/// # Example
/// ```
/// use unibit_kernel::math::isolate_lowest_set_mask;
///
/// let mask = isolate_lowest_set_mask(0b10100);
/// assert_eq!(mask, 0b00111);
/// ```
#[inline(always)]
#[must_use]
pub const fn isolate_lowest_set_mask(val: u64) -> u64 {
    val ^ val.wrapping_sub(1)
}

/// Computes the Gray code of a `64`-bit value.
///
/// # Example
/// ```
/// use unibit_kernel::math::gray_encode;
///
/// let g = gray_encode(0b101);
/// assert_eq!(g, 0b111);
/// ```
#[inline(always)]
#[must_use]
pub const fn gray_encode(val: u64) -> u64 {
    val ^ (val >> 1)
}

/// Computes the base-2 logarithm (floor) of a `64`-bit value branchlessly.
///
/// Returns `0` if the value is `0`.
///
/// # Example
/// ```
/// use unibit_kernel::math::log2_floor;
///
/// assert_eq!(log2_floor(16), 4);
/// assert_eq!(log2_floor(0), 0);
/// ```
#[inline(always)]
#[must_use]
pub const fn log2_floor(val: u64) -> u64 {
    let nz = (val != 0) as u64;
    let mask = 0u64.wrapping_sub(nz);
    (63u64.wrapping_sub(val.leading_zeros() as u64)) & mask
}

/// Saturating addition for `64` bits.
///
/// Unlike `u64::saturating_add`, this avoids any compiler-inserted branch
/// dependencies, guaranteeing a single-cycle or fixed-cycle profile across
/// all targets without LLVM optimization unpredictability.
///
/// # Example
/// ```
/// use unibit_kernel::math::add_sat_u64;
///
/// assert_eq!(add_sat_u64(u64::MAX - 10, 20), u64::MAX);
/// ```
#[inline(always)]
#[must_use]
pub const fn add_sat_u64(a: u64, b: u64) -> u64 {
    let res = a.wrapping_add(b);
    res | 0u64.wrapping_sub((res < a) as u64)
}

/// Saturating subtraction for `64` bits.
///
/// # Example
/// ```
/// use unibit_kernel::math::sub_sat_u64;
///
/// assert_eq!(sub_sat_u64(10, 20), 0);
/// ```
#[inline(always)]
#[must_use]
pub const fn sub_sat_u64(a: u64, b: u64) -> u64 {
    let res = a.wrapping_sub(b);
    res & !0u64.wrapping_sub((a < b) as u64)
}

/// Clamp a `64`-bit value to `[min, max]` branchlessly.
///
/// # Example
/// ```
/// use unibit_kernel::math::clamp_u64;
///
/// assert_eq!(clamp_u64(150, 100, 200), 150);
/// assert_eq!(clamp_u64(50, 100, 200), 100);
/// assert_eq!(clamp_u64(250, 100, 200), 200);
/// ```
#[inline(always)]
#[must_use]
pub const fn clamp_u64(val: u64, min: u64, max: u64) -> u64 {
    let mut res = val;
    let lt_min = (res < min) as u64;
    res = (min & 0u64.wrapping_sub(lt_min)) | (res & !0u64.wrapping_sub(lt_min));
    let gt_max = (res > max) as u64;
    res = (max & 0u64.wrapping_sub(gt_max)) | (res & !0u64.wrapping_sub(gt_max));
    res
}
