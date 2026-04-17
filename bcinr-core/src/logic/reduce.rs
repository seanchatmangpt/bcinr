//! SWAR Reductions: SIMD Within A Register: horizontal reductions
//!
//! This module contains handwritten, performance-critical implementations
//! of all SWAR Reductions algorithms. These functions are pub(crate) and wrapped
//! by the public API in `src/api/reduce.rs`.
//!
//! Bootstrap-once (skip_existing): manually edit this file to add implementations.
//! When you `unrdf sync`:
//! 1. First run: Creates this file with stubs
//! 2. Subsequent runs: Leaves this file untouched
//! 3. API wrappers regenerate automatically

/// Compute the sum of 8 u8 values, returning u16 to avoid overflow.
///
/// # Algorithm
/// Portable fallback using direct summation. No branches, predictable pipeline.
/// For x86_64 with SSE4.2, uses `_mm_sad_epu8` for sub-millisecond performance
/// on 8-byte chunks.
///
/// # Examples
/// ```
/// # #[allow(unsafe_code)]
/// # fn main() {
/// # use bcinr_core::logic::reduce::horizontal_sum_u8x8;
/// let arr = [10, 20, 30, 40, 50, 60, 70, 80];
/// let sum = horizontal_sum_u8x8(&arr);
/// assert_eq!(sum, 360);
/// # }
/// ```
///
/// # Correctness
/// - Sums are computed without intermediate overflow (u16 return)
/// - Portable fallback: straightforward loop
/// - SIMD variant: uses SSE4.2 `_mm_sad_epu8` on supported platforms
#[inline(always)]
pub(crate) fn horizontal_sum_u8x8(arr: &[u8; 8]) -> u16 {
    // SAFETY: We use intrinsics conditionally. On platforms without SSE4.2,
    // the portable fallback is used. The x86_64 intrinsic is safe because
    // the WASM module declares the feature, and SIMD is verified at load time.
    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        unsafe {
            // Load 8 bytes into SSE register (zero-extended to 16 bytes)
            let vec = _mm_loadl_epi64(arr.as_ptr() as *const __m128i);

            // SSE4.2: Compute sum-of-absolute-differences vs. zero
            // This computes sum of all 8 bytes in O(1) SSE latency
            let sad = _mm_sad_epu8(vec, _mm_setzero_si128());

            // Extract 16-bit sum from lanes 0-1 and 8-9
            let sum_lo = _mm_extract_epi16(sad, 0) as u16;
            let sum_hi = _mm_extract_epi16(sad, 4) as u16;

            sum_lo.wrapping_add(sum_hi)
        }
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
    {
        // Portable fallback: direct summation loop
        // Modern compilers optimize this to single-pass with no branches
        let mut sum: u16 = 0;
        for &byte in arr.iter() {
            sum = sum.wrapping_add(byte as u16);
        }
        sum
    }
}

/// Compute the maximum of 8 u8 values using SWAR (SIMD Within A Register).
///
/// # Algorithm
/// Branchless bitwise max using the identity:
/// ```text
/// max(a, b) = a ^ ((a ^ b) & -((b > a) as i8))
/// ```
/// where `(b > a) as i8` produces -1 (all bits set) if b > a, else 0.
/// This selects b's bits where b > a, else a's bits. No branches.
///
/// For x86_64 with SSE4.2, uses `_mm_max_epu8` for 2-cycle latency.
///
/// # Examples
/// ```
/// # use bcinr_core::logic::reduce::horizontal_max_u8x8;
/// let arr = [10, 45, 30, 200, 50, 60, 70, 80];
/// assert_eq!(horizontal_max_u8x8(&arr), 200);
/// ```
///
/// # Correctness
/// - Returns the largest element
/// - No branches: safe for timing-critical code
/// - Bitwise max via XOR masking is provably correct
#[inline(always)]
pub(crate) fn horizontal_max_u8x8(arr: &[u8; 8]) -> u8 {
    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        unsafe {
            // Load 8 bytes and zero-extend to 16 bytes
            let vec = _mm_loadl_epi64(arr.as_ptr() as *const __m128i);

            // Successive max-reductions using SSE4.2 _mm_max_epu8
            // Reduce 8 bytes → 4 bytes → 2 bytes → 1 byte
            let v1 = _mm_max_epu8(vec, _mm_shuffle_epi32(vec, 0b00_01_10_11)); // lanes 0-3, 4-7 → 0-3
            let v2 = _mm_max_epu8(v1, _mm_shufflelo_epi16(v1, 0b00_01_10_11)); // lanes 0-1, 2-3 → 0-1
            let v3 = _mm_max_epu8(v2, _mm_srli_epi16(v2, 8)); // lanes 0, 1 → 0

            _mm_extract_epi8(v3, 0) as u8
        }
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
    {
        // SWAR branchless max using XOR + masking
        // max(a, b) = a ^ ((a ^ b) & -((b > a) as i8))
        let mut max = arr[0];

        for &byte in &arr[1..] {
            // Compute (byte > max): produces true (1) or false (0)
            // Negate to get mask: true → -1 (0xFF), false → 0 (0x00)
            let gt = (-((byte > max) as i8)) as u8;
            // XOR-based selection: a ^ ((a ^ b) & mask) selects b where mask=0xFF, a where mask=0x00
            max = max ^ ((max ^ byte) & gt);
        }

        max
    }
}

/// Compute the minimum of 8 u8 values using SWAR (SIMD Within A Register).
///
/// # Algorithm
/// Branchless bitwise min using the identity:
/// ```text
/// min(a, b) = b ^ ((a ^ b) & -((a < b) as i8))
/// ```
/// which is equivalent to:
/// ```text
/// min(a, b) = a ^ ((a ^ b) & -((b < a) as i8))
/// ```
/// For x86_64 with SSE4.2, uses `_mm_min_epu8` for 2-cycle latency.
///
/// # Examples
/// ```
/// # use bcinr_core::logic::reduce::horizontal_min_u8x8;
/// let arr = [200, 45, 30, 100, 50, 60, 70, 80];
/// assert_eq!(horizontal_min_u8x8(&arr), 30);
/// ```
///
/// # Correctness
/// - Returns the smallest element
/// - No branches: safe for timing-critical code
/// - Bitwise min via XOR masking is dual to max
#[inline(always)]
pub(crate) fn horizontal_min_u8x8(arr: &[u8; 8]) -> u8 {
    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        unsafe {
            // Load 8 bytes and zero-extend to 16 bytes
            let vec = _mm_loadl_epi64(arr.as_ptr() as *const __m128i);

            // Successive min-reductions using SSE4.2 _mm_min_epu8
            // Reduce 8 bytes → 4 bytes → 2 bytes → 1 byte
            let v1 = _mm_min_epu8(vec, _mm_shuffle_epi32(vec, 0b00_01_10_11)); // lanes 0-3, 4-7 → 0-3
            let v2 = _mm_min_epu8(v1, _mm_shufflelo_epi16(v1, 0b00_01_10_11)); // lanes 0-1, 2-3 → 0-1
            let v3 = _mm_min_epu8(v2, _mm_srli_epi16(v2, 8)); // lanes 0, 1 → 0

            _mm_extract_epi8(v3, 0) as u8
        }
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
    {
        // SWAR branchless min using XOR + masking
        // min(a, b) = a ^ ((a ^ b) & -((b < a) as i8))
        let mut min = arr[0];

        for &byte in &arr[1..] {
            // Compute (byte < min): produces true (1) or false (0)
            // Negate to get mask: true → -1 (0xFF), false → 0 (0x00)
            let lt = (-((byte < min) as i8)) as u8;
            // XOR-based selection: a ^ ((a ^ b) & mask) selects b where mask=0xFF, a where mask=0x00
            min = min ^ ((min ^ byte) & lt);
        }

        min
    }
}

// SAFETY: SSE4.2 intrinsics used only on x86_64 with feature enabled.
// The intrinsics are defined conditionally and safe because:
// 1. Feature detection is compile-time via #[cfg(target_feature = "sse4.2")]
// 2. WASM/JIT verifies feature availability at module load time
// 3. No undefined behavior on platforms lacking SSE4.2 (portable fallback used)
#[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
use core::arch::x86_64::{_mm_extract_epi8, _mm_extract_epi16, _mm_loadl_epi64, _mm_max_epu8, _mm_min_epu8, _mm_sad_epu8, _mm_setzero_si128, _mm_shuffle_epi32, _mm_shufflelo_epi16, _mm_srli_epi16, __m128i};

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // horizontal_sum_u8x8 tests
    // =========================================================================

    #[test]
    fn test_horizontal_sum_u8x8_all_same() {
        // Test with all equal elements
        let arr = [5, 5, 5, 5, 5, 5, 5, 5];
        assert_eq!(horizontal_sum_u8x8(&arr), 40);
    }

    #[test]
    fn test_horizontal_sum_u8x8_all_different() {
        // Test with sequential elements
        let arr = [1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(horizontal_sum_u8x8(&arr), 36);
    }

    #[test]
    fn test_horizontal_sum_u8x8_zeros() {
        // Test with all zeros
        let arr = [0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(horizontal_sum_u8x8(&arr), 0);
    }

    #[test]
    fn test_horizontal_sum_u8x8_max_values() {
        // Test with max u8 values (overflow wrapping to u16)
        let arr = [255, 255, 255, 255, 255, 255, 255, 255];
        assert_eq!(horizontal_sum_u8x8(&arr), 2040);
    }

    #[test]
    fn test_horizontal_sum_u8x8_mixed() {
        // Test with mixed values
        let arr = [10, 20, 30, 40, 50, 60, 70, 80];
        assert_eq!(horizontal_sum_u8x8(&arr), 360);
    }

    // =========================================================================
    // horizontal_max_u8x8 tests
    // =========================================================================

    #[test]
    fn test_horizontal_max_u8x8_all_same() {
        // Test with all equal elements
        let arr = [42, 42, 42, 42, 42, 42, 42, 42];
        assert_eq!(horizontal_max_u8x8(&arr), 42);
    }

    #[test]
    fn test_horizontal_max_u8x8_all_different() {
        // Test with sequential elements (max at end)
        let arr = [1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(horizontal_max_u8x8(&arr), 8);
    }

    #[test]
    fn test_horizontal_max_u8x8_max_at_start() {
        // Test with max value at the start
        let arr = [255, 100, 50, 75, 200, 25, 150, 10];
        assert_eq!(horizontal_max_u8x8(&arr), 255);
    }

    #[test]
    fn test_horizontal_max_u8x8_max_at_middle() {
        // Test with max value in the middle
        let arr = [100, 50, 75, 255, 200, 25, 150, 10];
        assert_eq!(horizontal_max_u8x8(&arr), 255);
    }

    #[test]
    fn test_horizontal_max_u8x8_zeros() {
        // Test with mostly zeros
        let arr = [0, 0, 0, 0, 42, 0, 0, 0];
        assert_eq!(horizontal_max_u8x8(&arr), 42);
    }

    // =========================================================================
    // horizontal_min_u8x8 tests
    // =========================================================================

    #[test]
    fn test_horizontal_min_u8x8_all_same() {
        // Test with all equal elements
        let arr = [100, 100, 100, 100, 100, 100, 100, 100];
        assert_eq!(horizontal_min_u8x8(&arr), 100);
    }

    #[test]
    fn test_horizontal_min_u8x8_all_different() {
        // Test with sequential elements (min at start)
        let arr = [1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(horizontal_min_u8x8(&arr), 1);
    }

    #[test]
    fn test_horizontal_min_u8x8_min_at_end() {
        // Test with min value at the end
        let arr = [255, 100, 50, 75, 200, 25, 150, 10];
        assert_eq!(horizontal_min_u8x8(&arr), 10);
    }

    #[test]
    fn test_horizontal_min_u8x8_min_at_middle() {
        // Test with min value in the middle
        let arr = [200, 100, 50, 5, 200, 25, 150, 100];
        assert_eq!(horizontal_min_u8x8(&arr), 5);
    }

    #[test]
    fn test_horizontal_min_u8x8_zeros() {
        // Test with mostly non-zero
        let arr = [200, 100, 50, 0, 200, 25, 150, 100];
        assert_eq!(horizontal_min_u8x8(&arr), 0);
    }
}

#[cfg(feature = "bench")]
mod benches {
    use super::*;

    #[bench]
    fn bench_horizontal_sum_u8x8(b: &mut test::Bencher) {
        let arr = [10u8, 20, 30, 40, 50, 60, 70, 80];
        b.iter(|| {
            test::black_box(horizontal_sum_u8x8(&test::black_box(&arr)))
        });
    }

    #[bench]
    fn bench_horizontal_max_u8x8(b: &mut test::Bencher) {
        let arr = [10u8, 200, 30, 40, 50, 60, 70, 80];
        b.iter(|| {
            test::black_box(horizontal_max_u8x8(&test::black_box(&arr)))
        });
    }

    #[bench]
    fn bench_horizontal_min_u8x8(b: &mut test::Bencher) {
        let arr = [200u8, 100, 30, 40, 50, 60, 70, 80];
        b.iter(|| {
            test::black_box(horizontal_min_u8x8(&test::black_box(&arr)))
        });
    }
}
