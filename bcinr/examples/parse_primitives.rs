//! # Parse Primitives Example
//!
//! Demonstrates `bcinr_logic::parse`: `skip_whitespace` and `parse_hex_u32`.
//!
//! **Doc reference:** `crates/bcinr-logic/src/parse.rs`
//! **Also see:** `examples/scan_primitives.rs` — `skip_spaces` (space-only) vs
//! `skip_whitespace` (all bytes ≤ 32) and `find_byte_mask`.
//!
//! `parse_hex_u32` is branchless: it decodes up to 8 hex digits from a byte slice
//! into a `u32`, returning `Err(())` on any non-hex byte or on empty/over-length
//! input. The assertions below would fail if parsing produced wrong values, failed
//! to reject bad input, or silently truncated instead of returning `Err`.

use bcinr::parse::{parse_hex_u32, skip_whitespace};

fn main() {
    // --- skip_whitespace: all bytes ≤ 32 are whitespace (space, tab, newline, ...) ---
    assert_eq!(skip_whitespace(b"   hello"), 3, "three spaces");
    assert_eq!(
        skip_whitespace(b"\t\nhello"),
        2,
        "tab + newline = 2 whitespace chars"
    );
    assert_eq!(skip_whitespace(b"hello"), 0, "no leading whitespace");
    assert_eq!(skip_whitespace(b""), 0, "empty = 0");
    assert_eq!(skip_whitespace(b"   "), 3, "all whitespace");
    // Unlike skip_spaces, byte 9 (tab) and 10 (newline) are also skipped
    assert_eq!(skip_whitespace(b"\tx"), 1, "tab is whitespace");
    println!(
        "skip_whitespace(b\"\\t\\nhello\")={}",
        skip_whitespace(b"\t\nhello")
    );

    // --- parse_hex_u32: decode hex string to u32 ---
    // Basic hex values
    assert_eq!(parse_hex_u32(b"0"), Ok(0));
    assert_eq!(parse_hex_u32(b"1"), Ok(1));
    assert_eq!(parse_hex_u32(b"a"), Ok(10), "lowercase hex");
    assert_eq!(parse_hex_u32(b"A"), Ok(10), "uppercase hex");
    assert_eq!(parse_hex_u32(b"ff"), Ok(255));
    assert_eq!(parse_hex_u32(b"FF"), Ok(255));
    assert_eq!(parse_hex_u32(b"DEADBEEF"), Ok(0xDEAD_BEEF));
    assert_eq!(parse_hex_u32(b"deadbeef"), Ok(0xDEAD_BEEF), "lowercase");
    assert_eq!(parse_hex_u32(b"00000000"), Ok(0), "8 zeros");
    assert_eq!(parse_hex_u32(b"FFFFFFFF"), Ok(u32::MAX), "max u32");
    println!(
        "parse_hex_u32(b\"DEADBEEF\")={:?}",
        parse_hex_u32(b"DEADBEEF")
    );

    // Error cases: must return Err(())
    assert_eq!(parse_hex_u32(b""), Err(()), "empty input is Err");
    assert_eq!(parse_hex_u32(b"xyz"), Err(()), "non-hex chars");
    assert_eq!(parse_hex_u32(b"GG"), Err(()), "G is not hex");
    assert_eq!(parse_hex_u32(b"123456789"), Err(()), "9 digits > 8 → Err");
    assert_eq!(parse_hex_u32(b"1G"), Err(()), "mixed valid + invalid");
    println!(
        "parse_hex_u32(b\"\")={:?}, parse_hex_u32(b\"xyz\")={:?}",
        parse_hex_u32(b""),
        parse_hex_u32(b"xyz")
    );

    // Case insensitivity
    assert_eq!(
        parse_hex_u32(b"AbCdEf"),
        parse_hex_u32(b"abcdef"),
        "case insensitive"
    );
    assert_eq!(
        parse_hex_u32(b"AbCdEf"),
        parse_hex_u32(b"ABCDEF"),
        "case insensitive"
    );

    // --- cross-product: skip_whitespace then parse_hex_u32 ---
    // Simulate parsing "  0xDEAD" (whitespace + '0x' prefix + hex)
    // First: skip whitespace
    let input = b"  CAFE";
    let start = skip_whitespace(input);
    let hex_part = &input[start..];
    let val = parse_hex_u32(hex_part);
    assert_eq!(val, Ok(0xCAFE), "parse after whitespace skip");
    println!("skip+parse(b\"  CAFE\"): start={start}, val={val:?}");

    println!("\nAll parse primitive assertions passed.");
}
