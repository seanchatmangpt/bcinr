#![forbid(unsafe_code)]
//! UTF-8 Calculus: Branchless UTF-8 validation and segmentation.

/// Returns true if the provided slice is valid UTF-8.
#[inline]
pub fn validate_utf8(bytes: &[u8]) -> bool {
    core::str::from_utf8(bytes).is_ok()
}

/// Counts the number of Unicode codepoints in the slice.
#[inline]
pub fn count_codepoints(bytes: &[u8]) -> usize {
    let mut count = 0;
    for &b in bytes {
        // Codepoints start with bits NOT 10xxxxxx
        count += (b & 0xC0 != 0x80) as usize;
    }
    count
}

/// Returns the length of the leading ASCII prefix.
#[inline]
pub fn ascii_prefix_len(bytes: &[u8]) -> usize {
    bytes.iter().take_while(|&&b| b < 128).count()
}

/// Finds the index of the first byte that is not valid UTF-8.
#[inline]
pub fn first_invalid_byte(bytes: &[u8]) -> Option<usize> {
    match core::str::from_utf8(bytes) {
        Ok(_) => None,
        Err(e) => Some(e.valid_up_to()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_utf8() {
        assert!(validate_utf8(b"hello"));
        assert!(validate_utf8("🦀".as_bytes()));
        assert!(!validate_utf8(b"hello\xFF"));
    }

    #[test]
    fn test_count_codepoints() {
        assert_eq!(count_codepoints(b"hello"), 5);
        assert_eq!(count_codepoints("🦀".as_bytes()), 1);
        assert_eq!(count_codepoints("abc🦀def".as_bytes()), 7);
    }

    #[test]
    fn test_ascii_prefix_len() {
        assert_eq!(ascii_prefix_len(b"hello"), 5);
        assert_eq!(ascii_prefix_len(b"abc\xFF def"), 3);
        assert_eq!(ascii_prefix_len("🦀".as_bytes()), 0);
    }
}
