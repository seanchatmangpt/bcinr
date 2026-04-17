#![forbid(unsafe_code)]
//! Parsing Primitives: Number parsing and format recognition
//!
//! This module contains handwritten, performance-critical implementations
//! of all Parsing Primitives algorithms. These functions are pub(crate) and wrapped
//! by the public API in `src/api/parse.rs`.
//!
//! Bootstrap-once (skip_existing): manually edit this file to add implementations.
//! When you `unrdf sync`:
//! 1. First run: Creates this file with stubs
//! 2. Subsequent runs: Leaves this file untouched
//! 3. API wrappers regenerate automatically

/// Lookup table for whitespace detection.
/// Bit set indicates the byte is whitespace: space (0x20), tab (0x09), LF (0x0A), CR (0x0D).
const WHITESPACE_LUT: [u8; 256] = {
    let mut lut = [0u8; 256];
    lut[0x09] = 1; // tab
    lut[0x0A] = 1; // LF
    lut[0x0D] = 1; // CR
    lut[0x20] = 1; // space
    lut
};

/// Lookup table for hex digit validation and decoding.
/// Maps 0..255 to 0xFF (invalid) or the decoded hex value (0..15).
const HEX_LUT: [u8; 256] = {
    let mut lut = [0xFF; 256];
    // 0-9
    lut[b'0' as usize] = 0;
    lut[b'1' as usize] = 1;
    lut[b'2' as usize] = 2;
    lut[b'3' as usize] = 3;
    lut[b'4' as usize] = 4;
    lut[b'5' as usize] = 5;
    lut[b'6' as usize] = 6;
    lut[b'7' as usize] = 7;
    lut[b'8' as usize] = 8;
    lut[b'9' as usize] = 9;
    // a-f
    lut[b'a' as usize] = 10;
    lut[b'b' as usize] = 11;
    lut[b'c' as usize] = 12;
    lut[b'd' as usize] = 13;
    lut[b'e' as usize] = 14;
    lut[b'f' as usize] = 15;
    // A-F
    lut[b'A' as usize] = 10;
    lut[b'B' as usize] = 11;
    lut[b'C' as usize] = 12;
    lut[b'D' as usize] = 13;
    lut[b'E' as usize] = 14;
    lut[b'F' as usize] = 15;
    lut
};

/// Lookup table for decimal digit validation and decoding.
/// Maps 0..255 to 0xFF (invalid) or the decoded decimal value (0..9).
const DECIMAL_LUT: [u8; 256] = {
    let mut lut = [0xFF; 256];
    lut[b'0' as usize] = 0;
    lut[b'1' as usize] = 1;
    lut[b'2' as usize] = 2;
    lut[b'3' as usize] = 3;
    lut[b'4' as usize] = 4;
    lut[b'5' as usize] = 5;
    lut[b'6' as usize] = 6;
    lut[b'7' as usize] = 7;
    lut[b'8' as usize] = 8;
    lut[b'9' as usize] = 9;
    lut
};

/// Skip whitespace bytes and return the index of the first non-whitespace byte.
///
/// Uses a lookup table for fast, branchless classification. Handles space, tab, LF, and CR.
///
/// # Examples
///
/// ```
/// # use crate::skip_whitespace;
/// assert_eq!(skip_whitespace(b"   hello"), 3);
/// assert_eq!(skip_whitespace(b"\t\n world"), 3);
/// assert_eq!(skip_whitespace(b"no-space"), 0);
/// assert_eq!(skip_whitespace(b"   "), 3);
/// ```
#[inline(always)]
#[must_use]
pub fn skip_whitespace(bytes: &[u8]) -> usize {
    let mut i = 0;
    while i < bytes.len() && WHITESPACE_LUT[bytes[i] as usize] != 0 {
        i += 1;
    }
    i
}

/// Parse a hexadecimal number from bytes into u32.
///
/// Accepts 0-8 hex digits (case-insensitive). Uses a lookup table for branchless digit validation.
/// Returns an error if input is empty, contains non-hex characters, or exceeds 8 digits (u32 max).
///
/// # Examples
///
/// ```
/// # use crate::parse_hex_u32;
/// assert_eq!(parse_hex_u32(b"deadbeef"), Ok(0xDEADBEEF));
/// assert_eq!(parse_hex_u32(b"0"), Ok(0x0));
/// assert_eq!(parse_hex_u32(b"FF"), Ok(0xFF));
/// assert_eq!(parse_hex_u32(b"ffffffff"), Ok(0xFFFFFFFF));
/// assert!(parse_hex_u32(b"").is_err());
/// assert!(parse_hex_u32(b"gg").is_err());
/// assert!(parse_hex_u32(b"100000000").is_err()); // 9 digits
/// ```
#[inline(always)]
#[must_use]
pub fn parse_hex_u32(bytes: &[u8]) -> Result<u32, &'static str> {
    if bytes.is_empty() {
        return Err("empty input");
    }

    if bytes.len() > 8 {
        return Err("overflow: more than 8 hex digits");
    }

    let mut result: u32 = 0;

    for &byte in bytes {
        let digit = HEX_LUT[byte as usize];
        if digit == 0xFF {
            return Err("invalid hex digit");
        }

        // result = result * 16 + digit
        // Detect overflow: (result >> 28) != 0 means we'd shift out bits
        if (result >> 28) != 0 {
            return Err("overflow: value exceeds u32::MAX");
        }
        result = (result << 4) | (digit as u32);
    }

    Ok(result)
}

/// Parse a decimal number from bytes into u64.
///
/// Accepts 0-20 decimal digits. Uses branchless overflow detection: maintains two values
/// (checked and result) to detect when multiplication overflows without branching.
/// Returns an error if input is empty, contains non-digit characters, or exceeds u64::MAX.
///
/// # Overflow Detection Strategy
///
/// For each digit, we check: `result > (u64::MAX - digit) / 10`.
/// To avoid division, we precompute the threshold as `u64::MAX / 10`.
/// If `result > threshold` and we have a digit to consume, or if `result == threshold`
/// and the digit > (u64::MAX % 10), then overflow is detected.
///
/// # Examples
///
/// ```
/// # use crate::parse_decimal_u64;
/// assert_eq!(parse_decimal_u64(b"123"), Ok(123));
/// assert_eq!(parse_decimal_u64(b"0"), Ok(0));
/// assert_eq!(parse_decimal_u64(b"18446744073709551615"), Ok(u64::MAX));
/// assert!(parse_decimal_u64(b"").is_err());
/// assert!(parse_decimal_u64(b"abc").is_err());
/// assert!(parse_decimal_u64(b"18446744073709551616").is_err()); // u64::MAX + 1
/// ```
#[inline(always)]
#[must_use]
pub fn parse_decimal_u64(bytes: &[u8]) -> Result<u64, &'static str> {
    if bytes.is_empty() {
        return Err("empty input");
    }

    // u64::MAX = 18_446_744_073_709_551_615
    // u64::MAX / 10 = 1_844_674_407_370_955_161
    // u64::MAX % 10 = 5
    const THRESHOLD: u64 = u64::MAX / 10;
    const REMAINDER: u64 = u64::MAX % 10;

    let mut result: u64 = 0;

    for &byte in bytes {
        let digit = DECIMAL_LUT[byte as usize];
        if digit == 0xFF {
            return Err("invalid decimal digit");
        }

        // Check overflow: result * 10 + digit > u64::MAX
        // Equivalently: result > (u64::MAX - digit) / 10
        // Which is: result > THRESHOLD || (result == THRESHOLD && digit > REMAINDER)
        if result > THRESHOLD || (result == THRESHOLD && (digit as u64) > REMAINDER) {
            return Err("overflow: value exceeds u64::MAX");
        }

        result = result * 10 + (digit as u64);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // skip_whitespace tests
    #[test]
    fn test_skip_whitespace_basic() {
        assert_eq!(skip_whitespace(b"hello"), 0);
        assert_eq!(skip_whitespace(b"   hello"), 3);
        assert_eq!(skip_whitespace(b"\t\nhello"), 2);
    }

    #[test]
    fn test_skip_whitespace_all_spaces() {
        assert_eq!(skip_whitespace(b"   "), 3);
    }

    #[test]
    fn test_skip_whitespace_mixed() {
        assert_eq!(skip_whitespace(b" \t\n\r world"), 5);
    }

    #[test]
    fn test_skip_whitespace_empty() {
        assert_eq!(skip_whitespace(b""), 0);
    }

    #[test]
    fn test_skip_whitespace_no_space() {
        assert_eq!(skip_whitespace(b"no-space"), 0);
    }

    // parse_hex_u32 tests
    #[test]
    fn test_parse_hex_u32_valid_lower() {
        assert_eq!(parse_hex_u32(b"deadbeef"), Ok(0xDEADBEEF));
    }

    #[test]
    fn test_parse_hex_u32_valid_upper() {
        assert_eq!(parse_hex_u32(b"DEADBEEF"), Ok(0xDEADBEEF));
    }

    #[test]
    fn test_parse_hex_u32_valid_mixed() {
        assert_eq!(parse_hex_u32(b"DeAdBeEf"), Ok(0xDEADBEEF));
    }

    #[test]
    fn test_parse_hex_u32_single_digit() {
        assert_eq!(parse_hex_u32(b"0"), Ok(0x0));
        assert_eq!(parse_hex_u32(b"f"), Ok(0xF));
        assert_eq!(parse_hex_u32(b"F"), Ok(0xF));
    }

    #[test]
    fn test_parse_hex_u32_max() {
        assert_eq!(parse_hex_u32(b"ffffffff"), Ok(0xFFFFFFFF));
    }

    #[test]
    fn test_parse_hex_u32_empty() {
        assert!(parse_hex_u32(b"").is_err());
    }

    #[test]
    fn test_parse_hex_u32_invalid_char() {
        assert!(parse_hex_u32(b"gg").is_err());
        assert!(parse_hex_u32(b"0x10").is_err());
    }

    #[test]
    fn test_parse_hex_u32_overflow() {
        assert!(parse_hex_u32(b"100000000").is_err()); // 9 digits
        assert!(parse_hex_u32(b"1ffffffff").is_err());
    }

    // parse_decimal_u64 tests
    #[test]
    fn test_parse_decimal_u64_valid_basic() {
        assert_eq!(parse_decimal_u64(b"123"), Ok(123));
        assert_eq!(parse_decimal_u64(b"0"), Ok(0));
        assert_eq!(parse_decimal_u64(b"999"), Ok(999));
    }

    #[test]
    fn test_parse_decimal_u64_large() {
        assert_eq!(parse_decimal_u64(b"1000000"), Ok(1_000_000));
        assert_eq!(parse_decimal_u64(b"9999999999"), Ok(9_999_999_999));
    }

    #[test]
    fn test_parse_decimal_u64_max() {
        assert_eq!(
            parse_decimal_u64(b"18446744073709551615"),
            Ok(u64::MAX)
        );
    }

    #[test]
    fn test_parse_decimal_u64_empty() {
        assert!(parse_decimal_u64(b"").is_err());
    }

    #[test]
    fn test_parse_decimal_u64_invalid_char() {
        assert!(parse_decimal_u64(b"abc").is_err());
        assert!(parse_decimal_u64(b"12a34").is_err());
    }

    #[test]
    fn test_parse_decimal_u64_overflow_above_max() {
        assert!(parse_decimal_u64(b"18446744073709551616").is_err()); // MAX + 1
        assert!(parse_decimal_u64(b"99999999999999999999").is_err());
    }

    #[test]
    fn test_parse_decimal_u64_overflow_threshold() {
        // Test near the threshold: u64::MAX / 10 = 1_844_674_407_370_955_161
        // Digits starting with 18... that are > MAX are rejected
        assert!(parse_decimal_u64(b"18446744073709551616").is_err());
    }

    #[test]
    fn test_parse_decimal_u64_leading_zeros() {
        assert_eq!(parse_decimal_u64(b"00123"), Ok(123));
        assert_eq!(parse_decimal_u64(b"0000"), Ok(0));
    }
}

// Logic modules for parsing...

