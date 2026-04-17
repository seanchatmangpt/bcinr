//! Byte/Word Scanning: Scanning and searching within byte/word sequences
//!
//! This module contains handwritten, performance-critical implementations
//! of all Byte/Word Scanning algorithms. These functions are pub(crate) and wrapped
//! by the public API in `src/api/scan.rs`.
//!
//! Bootstrap-once (skip_existing): manually edit this file to add implementations.
//! When you `unrdf sync`:
//! 1. First run: Creates this file with stubs
//! 2. Subsequent runs: Leaves this file untouched
//! 3. API wrappers regenerate automatically

// SIMD implementation selection based on target features
#[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
use core::arch::x86_64::{_mm_cmpeq_epi8, _mm_loadu_si128, _mm_set1_epi8};

// Portable fallback: word-aligned byte scanning
#[inline]
fn is_zero_word(word: usize) -> bool {
    // SAFETY: word size is always valid. This checks if any byte in the word is zero
    // using the classic branchless technique: (word - 0x01010101) & !word & 0x80808080
    let ones = word ^ 0;
    ((ones.wrapping_sub(0x01010101u64 as usize)) & !ones & 0x80808080u64 as usize) != 0
}

/// Find the index of the first zero byte in a slice.
///
/// Returns `Some(index)` if a zero byte is found, `None` otherwise.
///
/// # Algorithm
///
/// For x86_64 with SSE4.2, uses SIMD byte comparison (`_mm_cmpeq_epi8`) for 16-byte
/// chunks. For other platforms, uses word-aligned scanning with branchless zero detection.
///
/// # Examples
///
/// ```
/// # use bcinr_core::logic::scan::first_zero_byte;
/// assert_eq!(first_zero_byte(&[1, 2, 3, 0, 5]), Some(3));
/// assert_eq!(first_zero_byte(&[0]), Some(0));
/// assert_eq!(first_zero_byte(&[1, 2, 3, 4]), None);
/// assert_eq!(first_zero_byte(&[]), None);
/// ```
#[inline]
pub(crate) fn first_zero_byte(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() {
        return None;
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        return first_zero_byte_simd(bytes);
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
    {
        return first_zero_byte_portable(bytes);
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
#[inline]
fn first_zero_byte_simd(bytes: &[u8]) -> Option<usize> {
    // SAFETY: We check is_x86_feature_detected at runtime. SSE4.2 provides
    // _mm_cmpeq_epi8 for byte comparison and _mm_movemask_epi8 for extracting results.
    // The SIMD vector is 16 bytes, and we process aligned chunks safely.

    let mut offset = 0;
    let len = bytes.len();

    // Process 16-byte chunks with SIMD
    while offset + 16 <= len {
        unsafe {
            let chunk = bytes.as_ptr().add(offset) as *const core::arch::x86_64::__m128i;
            let zero = _mm_set1_epi8(0);
            let result = _mm_cmpeq_epi8(_mm_loadu_si128(chunk), zero);
            let mask = core::arch::x86_64::_mm_movemask_epi8(result) as u32;

            if mask != 0 {
                // Found a zero byte; use trailing zeros to get the index
                return Some(offset + (mask.trailing_zeros() as usize));
            }
        }
        offset += 16;
    }

    // Fallback for remaining bytes (< 16 bytes)
    first_zero_byte_portable(&bytes[offset..]).map(|idx| offset + idx)
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
#[inline]
fn first_zero_byte_portable(bytes: &[u8]) -> Option<usize> {
    let ptr = bytes.as_ptr();
    let len = bytes.len();

    // Align to word boundary (8 bytes on 64-bit)
    let word_size = core::mem::size_of::<usize>();
    let mut offset = 0;

    // Scan unaligned prefix
    let alignment = ptr as usize % word_size;
    if alignment != 0 {
        let prefix_len = word_size - alignment;
        let prefix_len = prefix_len.min(len);
        for i in 0..prefix_len {
            if bytes[i] == 0 {
                return Some(i);
            }
        }
        offset = prefix_len;
    }

    // Scan word-aligned body
    while offset + word_size <= len {
        unsafe {
            let word = (ptr.add(offset) as *const usize).read_unaligned();
            if is_zero_word(word) {
                // Found a zero byte in this word, scan it byte-by-byte
                for i in 0..word_size {
                    if bytes[offset + i] == 0 {
                        return Some(offset + i);
                    }
                }
            }
        }
        offset += word_size;
    }

    // Scan remaining bytes
    for i in offset..len {
        if bytes[i] == 0 {
            return Some(i);
        }
    }

    None
}

/// Count the number of zero bytes in a slice.
///
/// Returns the count of zero bytes found.
///
/// # Algorithm
///
/// For x86_64 with SSE4.2, uses SIMD byte comparison for 16-byte chunks.
/// For other platforms, uses word-aligned scanning.
///
/// # Examples
///
/// ```
/// # use bcinr_core::logic::scan::count_zero_bytes;
/// assert_eq!(count_zero_bytes(&[0, 0, 1, 0, 2]), 3);
/// assert_eq!(count_zero_bytes(&[1, 2, 3]), 0);
/// assert_eq!(count_zero_bytes(&[0]), 1);
/// assert_eq!(count_zero_bytes(&[]), 0);
/// ```
#[inline]
pub(crate) fn count_zero_bytes(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        return count_zero_bytes_simd(bytes);
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
    {
        return count_zero_bytes_portable(bytes);
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
#[inline]
fn count_zero_bytes_simd(bytes: &[u8]) -> usize {
    // SAFETY: SSE4.2 provides _mm_cmpeq_epi8 for byte comparison and
    // _mm_movemask_epi8 to extract results. We process aligned 16-byte chunks safely.

    let mut count = 0;
    let len = bytes.len();
    let mut offset = 0;

    // Process 16-byte chunks with SIMD
    while offset + 16 <= len {
        unsafe {
            let chunk = bytes.as_ptr().add(offset) as *const core::arch::x86_64::__m128i;
            let zero = _mm_set1_epi8(0);
            let result = _mm_cmpeq_epi8(_mm_loadu_si128(chunk), zero);
            let mask = core::arch::x86_64::_mm_movemask_epi8(result) as u32;
            count += mask.count_ones() as usize;
        }
        offset += 16;
    }

    // Count remaining bytes
    for i in offset..len {
        if bytes[i] == 0 {
            count += 1;
        }
    }

    count
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
#[inline]
fn count_zero_bytes_portable(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == 0).count()
}

/// Find the first occurrence of a target byte in a slice.
///
/// Returns `Some(index)` if the byte is found, `None` otherwise.
///
/// # Algorithm
///
/// For x86_64 with SSE4.2, uses SIMD byte comparison (`_mm_cmpeq_epi8`) with a
/// broadcast target byte for 16-byte chunks. For other platforms, uses simple iteration.
///
/// # Examples
///
/// ```
/// # use bcinr_core::logic::scan::find_byte;
/// assert_eq!(find_byte(&[1, 2, 3, 4, 5], 3), Some(2));
/// assert_eq!(find_byte(&[1, 2, 3], 1), Some(0));
/// assert_eq!(find_byte(&[1, 2, 3], 5), None);
/// assert_eq!(find_byte(&[], 1), None);
/// assert_eq!(find_byte(&[255], 255), Some(0));
/// ```
#[inline]
pub(crate) fn find_byte(bytes: &[u8], target: u8) -> Option<usize> {
    if bytes.is_empty() {
        return None;
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        return find_byte_simd(bytes, target);
    }

    #[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
    {
        return find_byte_portable(bytes, target);
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
#[inline]
fn find_byte_simd(bytes: &[u8], target: u8) -> Option<usize> {
    // SAFETY: SSE4.2 provides _mm_cmpeq_epi8 for byte comparison and
    // _mm_movemask_epi8 to extract results. We process aligned 16-byte chunks safely.

    let mut offset = 0;
    let len = bytes.len();

    // Process 16-byte chunks with SIMD
    while offset + 16 <= len {
        unsafe {
            let chunk = bytes.as_ptr().add(offset) as *const core::arch::x86_64::__m128i;
            let needle = _mm_set1_epi8(target as i8);
            let result = _mm_cmpeq_epi8(_mm_loadu_si128(chunk), needle);
            let mask = core::arch::x86_64::_mm_movemask_epi8(result) as u32;

            if mask != 0 {
                return Some(offset + (mask.trailing_zeros() as usize));
            }
        }
        offset += 16;
    }

    // Fallback for remaining bytes (< 16 bytes)
    find_byte_portable(&bytes[offset..], target).map(|idx| offset + idx)
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.2")))]
#[inline]
fn find_byte_portable(bytes: &[u8], target: u8) -> Option<usize> {
    bytes.iter().position(|&b| b == target)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =====================
    // first_zero_byte tests
    // =====================

    #[test]
    fn test_first_zero_byte_empty() {
        assert_eq!(first_zero_byte(&[]), None);
    }

    #[test]
    fn test_first_zero_byte_single_zero() {
        assert_eq!(first_zero_byte(&[0]), Some(0));
    }

    #[test]
    fn test_first_zero_byte_single_nonzero() {
        assert_eq!(first_zero_byte(&[1]), None);
    }

    #[test]
    fn test_first_zero_byte_at_start() {
        assert_eq!(first_zero_byte(&[0, 1, 2, 3]), Some(0));
    }

    #[test]
    fn test_first_zero_byte_at_end() {
        assert_eq!(first_zero_byte(&[1, 2, 3, 0]), Some(3));
    }

    #[test]
    fn test_first_zero_byte_in_middle() {
        assert_eq!(first_zero_byte(&[1, 2, 0, 3, 4]), Some(2));
    }

    #[test]
    fn test_first_zero_byte_multiple_zeros() {
        assert_eq!(first_zero_byte(&[0, 1, 0, 2, 0]), Some(0));
    }

    #[test]
    fn test_first_zero_byte_no_zero() {
        assert_eq!(first_zero_byte(&[1, 2, 3, 4, 5]), None);
    }

    #[test]
    fn test_first_zero_byte_long_buffer() {
        let mut buf = [1u8; 256];
        buf[128] = 0;
        assert_eq!(first_zero_byte(&buf), Some(128));
    }

    #[test]
    fn test_first_zero_byte_all_zeros() {
        assert_eq!(first_zero_byte(&[0, 0, 0, 0]), Some(0));
    }

    // =====================
    // count_zero_bytes tests
    // =====================

    #[test]
    fn test_count_zero_bytes_empty() {
        assert_eq!(count_zero_bytes(&[]), 0);
    }

    #[test]
    fn test_count_zero_bytes_single_zero() {
        assert_eq!(count_zero_bytes(&[0]), 1);
    }

    #[test]
    fn test_count_zero_bytes_single_nonzero() {
        assert_eq!(count_zero_bytes(&[1]), 0);
    }

    #[test]
    fn test_count_zero_bytes_no_zeros() {
        assert_eq!(count_zero_bytes(&[1, 2, 3, 4, 5]), 0);
    }

    #[test]
    fn test_count_zero_bytes_multiple_zeros() {
        assert_eq!(count_zero_bytes(&[0, 0, 1, 0, 2]), 3);
    }

    #[test]
    fn test_count_zero_bytes_all_zeros() {
        assert_eq!(count_zero_bytes(&[0, 0, 0, 0]), 4);
    }

    #[test]
    fn test_count_zero_bytes_alternating() {
        assert_eq!(count_zero_bytes(&[1, 0, 1, 0, 1, 0]), 3);
    }

    #[test]
    fn test_count_zero_bytes_long_buffer() {
        let buf = [0u8; 256];
        assert_eq!(count_zero_bytes(&buf), 256);
    }

    #[test]
    fn test_count_zero_bytes_sparse() {
        let mut buf = [1u8; 256];
        buf[32] = 0;
        buf[128] = 0;
        buf[255] = 0;
        assert_eq!(count_zero_bytes(&buf), 3);
    }

    // ==================
    // find_byte tests
    // ==================

    #[test]
    fn test_find_byte_empty() {
        assert_eq!(find_byte(&[], 1), None);
    }

    #[test]
    fn test_find_byte_single_match() {
        assert_eq!(find_byte(&[42], 42), Some(0));
    }

    #[test]
    fn test_find_byte_single_no_match() {
        assert_eq!(find_byte(&[42], 99), None);
    }

    #[test]
    fn test_find_byte_at_start() {
        assert_eq!(find_byte(&[1, 2, 3, 4], 1), Some(0));
    }

    #[test]
    fn test_find_byte_at_end() {
        assert_eq!(find_byte(&[1, 2, 3, 4], 4), Some(3));
    }

    #[test]
    fn test_find_byte_in_middle() {
        assert_eq!(find_byte(&[1, 2, 3, 4, 5], 3), Some(2));
    }

    #[test]
    fn test_find_byte_no_match() {
        assert_eq!(find_byte(&[1, 2, 3, 4], 99), None);
    }

    #[test]
    fn test_find_byte_multiple_matches() {
        assert_eq!(find_byte(&[1, 2, 1, 3, 1], 1), Some(0));
    }

    #[test]
    fn test_find_byte_zero_byte() {
        assert_eq!(find_byte(&[1, 2, 0, 3, 4], 0), Some(2));
    }

    #[test]
    fn test_find_byte_max_byte() {
        assert_eq!(find_byte(&[1, 2, 255, 3], 255), Some(2));
    }

    #[test]
    fn test_find_byte_long_buffer() {
        let mut buf = [1u8; 256];
        buf[128] = 42;
        assert_eq!(find_byte(&buf, 42), Some(128));
    }

    // =====================
    // Edge case tests
    // =====================

    #[test]
    fn test_find_byte_all_same_value() {
        assert_eq!(find_byte(&[5, 5, 5, 5], 5), Some(0));
    }

    #[test]
    fn test_first_zero_byte_very_long_buffer() {
        let mut buf = [1u8; 256];
        buf[255] = 0;
        assert_eq!(first_zero_byte(&buf), Some(255));
    }

    #[test]
    fn test_count_zero_bytes_very_long_buffer() {
        let buf = [0u8; 256];
        assert_eq!(count_zero_bytes(&buf), 256);
    }

    #[test]
    fn test_find_byte_boundary_alignment() {
        // Test near word boundaries (8, 16, 32 bytes)
        let mut buf = [1u8; 40];
        buf[8] = 42;
        assert_eq!(find_byte(&buf, 42), Some(8));

        buf[15] = 42;
        assert_eq!(find_byte(&buf, 42), Some(8)); // Finds first
    }
}
