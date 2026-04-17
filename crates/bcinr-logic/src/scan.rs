//! Byte/Word Scanning: Scanning and searching within byte/word sequences
//!
//! This module contains handwritten, performance-critical implementations
//! of all Byte/Word Scanning algorithms.

/// Create a 64-bit mask where each bit represents if the corresponding byte matches the target.
#[inline(always)]
#[must_use]
pub fn find_byte_mask(bytes: &[u8], target: u8) -> u64 {
    let mut mask = 0u64;
    for i in 0..bytes.len().min(64) {
        if bytes[i] == target {
            mask |= 1u64 << i;
        }
    }
    mask
}

/// Count the number of non-zero bytes in the slice.
#[inline(always)]
#[must_use]
pub fn count_nonzero_bytes(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b != 0).count()
}

/// Skip leading space characters (ASCII 32) and return the offset.
#[inline(always)]
#[must_use]
pub fn skip_spaces(bytes: &[u8]) -> usize {
    let mut offset = 0;
    while offset < bytes.len() && bytes[offset] == b' ' {
        offset += 1;
    }
    offset
}

/// Check if the byte slice is ASCII using 64-bit SWAR.
#[inline(always)]
#[must_use]
pub fn is_ascii_u64_slice(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i + 8 <= bytes.len() {
        // SAFETY: We checked the bounds.
        let val = unsafe { (bytes.as_ptr().add(i) as *const u64).read_unaligned() };
        if (val & 0x8080_8080_8080_8080) != 0 {
            return false;
        }
        i += 8;
    }
    while i < bytes.len() {
        if bytes[i] >= 128 {
            return false;
        }
        i += 1;
    }
    true
}

/// Find the index of the first zero byte in the slice.
#[inline]
#[must_use]
pub fn first_zero_byte(bytes: &[u8]) -> Option<usize> {
    bytes.iter().position(|&b| b == 0)
}

/// Count the number of zero bytes in the slice.
#[inline]
#[must_use]
pub fn count_zero_bytes(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == 0).count()
}

/// Find the index of the first occurrence of the target byte.
#[inline]
#[must_use]
pub fn find_byte(bytes: &[u8], target: u8) -> Option<usize> {
    bytes.iter().position(|&b| b == target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_byte_mask() {
        let b = b"a b c ";
        assert_eq!(find_byte_mask(b, b' '), 0b101010); // spaces at 1, 3, 5
    }

    #[test]
    fn test_count_nonzero_bytes() {
        assert_eq!(count_nonzero_bytes(&[0, 1, 0, 2]), 2);
    }
}
