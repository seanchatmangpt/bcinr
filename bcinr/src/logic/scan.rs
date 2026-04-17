//! Byte/Word Scanning: Scanning and searching within byte/word sequences
//!
//! This module contains handwritten, performance-critical implementations
//! of all Byte/Word Scanning algorithms.

#[inline(always)]
pub fn find_byte_mask(bytes: &[u8], target: u8) -> u64 {
    let mut mask = 0u64;
    for i in 0..bytes.len().min(64) {
        if bytes[i] == target {
            mask |= 1u64 << i;
        }
    }
    mask
}

#[inline(always)]
pub fn count_nonzero_bytes(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b != 0).count()
}

#[inline(always)]
pub fn skip_spaces(bytes: &[u8]) -> usize {
    let mut offset = 0;
    while offset < bytes.len() && bytes[offset] == b' ' {
        offset += 1;
    }
    offset
}

// ... existing first_zero_byte, count_zero_bytes, find_byte etc ...
#[inline]
pub fn first_zero_byte(bytes: &[u8]) -> Option<usize> {
    bytes.iter().position(|&b| b == 0)
}

#[inline]
pub fn count_zero_bytes(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == 0).count()
}

#[inline]
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
