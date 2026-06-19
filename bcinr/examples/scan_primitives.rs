//! # Scan Primitives Example
//!
//! Demonstrates `bcinr_logic::scan`: `find_byte_mask`, `skip_spaces`, `is_ascii_u64_slice`.
//!
//! **Doc reference:** `crates/bcinr-logic/src/scan.rs`
//! **Also see:** `examples/horizontal_reductions.rs` — reductions over byte-level data.
//!
//! `find_byte_mask` returns a u64 bitmask with bit i set iff `bytes[i] == target`.
//! `skip_spaces` returns the count of leading spaces branchlessly.
//! `is_ascii_u64_slice` checks that all bytes have their high bit clear.
//! All assertions below would fail if any function returned a wrong mask or count.

use bcinr::scan::{find_byte_mask, is_ascii_u64_slice, skip_spaces};

fn main() {
    // --- find_byte_mask: bitmask of positions matching target byte ---
    let data = b"hello world";
    let mask = find_byte_mask(data, b'l');
    // 'l' appears at positions 2, 3, 9 → bits 2, 3, 9 set
    assert_eq!(mask & (1 << 2), 1 << 2, "bit 2 must be set (first 'l')");
    assert_eq!(mask & (1 << 3), 1 << 3, "bit 3 must be set (second 'l')");
    assert_eq!(mask & (1 << 9), 1 << 9, "bit 9 must be set (third 'l')");
    assert_eq!(mask & (1 << 0), 0, "bit 0 must NOT be set ('h' ≠ 'l')");
    println!("find_byte_mask(b\"hello world\", b'l') = {mask:#013b}");

    // no match → all zeros
    let no_match = find_byte_mask(b"abc", b'z');
    assert_eq!(no_match, 0, "no 'z' in 'abc' → mask must be 0");

    // single byte
    let single = find_byte_mask(b"x", b'x');
    assert_eq!(single, 1, "single match at position 0 → bit 0 set");
    println!("find_byte_mask(b\"x\", b'x') = {single}");

    // --- skip_spaces: count of leading spaces ---
    assert_eq!(skip_spaces(b"   hello"), 3, "three leading spaces");
    assert_eq!(skip_spaces(b"hello"), 0, "no leading spaces");
    assert_eq!(skip_spaces(b""), 0, "empty → 0");
    assert_eq!(skip_spaces(b"   "), 3, "all spaces");
    assert_eq!(
        skip_spaces(b" a b"),
        1,
        "one leading space then non-space stops count"
    );
    println!("skip_spaces(b\"   hello\") = {}", skip_spaces(b"   hello"));
    println!("skip_spaces(b\"hello\") = {}", skip_spaces(b"hello"));

    // --- is_ascii_u64_slice: all bytes must have high bit clear ---
    assert!(
        is_ascii_u64_slice(b"hello world"),
        "pure ASCII must return true"
    );
    assert!(is_ascii_u64_slice(b""), "empty slice is trivially ASCII");
    assert!(
        is_ascii_u64_slice(&[0x7F, 0x00, 0x41]),
        "0x7F is ASCII (DEL)"
    );
    // High bit set → not ASCII
    assert!(
        !is_ascii_u64_slice(&[0x80]),
        "0x80 has high bit set → not ASCII"
    );
    assert!(!is_ascii_u64_slice(b"\xFF"), "0xFF → not ASCII");
    // Slice with non-ASCII embedded
    let mixed = b"abc\xC3\xA9xyz"; // 'é' in UTF-8 has bytes 0xC3, 0xA9
    assert!(!is_ascii_u64_slice(mixed), "UTF-8 multibyte → not ASCII");
    println!(
        "is_ascii_u64_slice(b\"hello world\")={}",
        is_ascii_u64_slice(b"hello world")
    );
    println!(
        "is_ascii_u64_slice(&[0x80])={}",
        is_ascii_u64_slice(&[0x80])
    );

    // --- cross-product: find then skip ---
    // In a tokenizer pipeline: skip leading spaces, then find next space to get token length
    let input = b"   token rest";
    let offset = skip_spaces(input);
    let remainder = &input[offset..];
    let space_mask = find_byte_mask(remainder, b' ');
    let token_end = space_mask.trailing_zeros() as usize;
    let token = &remainder[..token_end];
    assert_eq!(token, b"token", "extracted token must be 'token'");
    println!(
        "tokenizer: skipped {offset} spaces, token={}",
        core::str::from_utf8(token).unwrap()
    );

    println!("\nAll scan primitive assertions passed.");
}
