//! # Algorithm Cross-Section Example
//!
//! Demonstrates a representative sample from `bcinr_logic::algorithms` — one from
//! each difficulty tier (1-100, 101-200, 201-300) — and shows them composing.
//! All 308 algorithm functions share the signature `fn(val: u64, aux: u64) -> u64`
//! and each carries a `/// # Branchless Contract` doc.
//!
//! **Doc references:**
//!   - `crates/bcinr-logic/src/algorithms/abs_diff_u64.rs` (tier 1-100)
//!   - `crates/bcinr-logic/src/algorithms/rotate_left_u64.rs` (tier 1-100)
//!   - `crates/bcinr-logic/src/algorithms/gcd_u64_branchless.rs` (tier 101-200)
//!   - `crates/bcinr-logic/src/algorithms/popcount_u128.rs` (tier 101-200)
//!   - `crates/bcinr-logic/src/algorithms/leb128_decode_u64.rs` (tier 201-300)
//!
//! **Also see:** `examples/branchless_pipeline.rs` — cross-module composition.
//!
//! Composition proof: the hash-and-reduce pipeline below chains 4 algorithm
//! functions together; each assertion would fail if any link in the chain produced
//! the wrong output.

use bcinr::algorithms::{
    abs_diff_u64::abs_diff_u64, gcd_u64_branchless::gcd_u64_branchless,
    leb128_decode_u64::leb128_decode_u64, popcount_u128::popcount_u128,
    rotate_left_u64::rotate_left_u64,
};

fn main() {
    // --- Tier 1-100: abs_diff_u64 — |val - aux| branchlessly ---
    assert_eq!(abs_diff_u64(10, 3), 7);
    assert_eq!(abs_diff_u64(3, 10), 7, "symmetric");
    assert_eq!(abs_diff_u64(0, 0), 0);
    assert_eq!(abs_diff_u64(u64::MAX, 0), u64::MAX);
    println!(
        "abs_diff_u64(10,3)={}, abs_diff_u64(3,10)={}",
        abs_diff_u64(10, 3),
        abs_diff_u64(3, 10)
    );

    // --- Tier 1-100: rotate_left_u64 — barrel rotate by (aux & 63) bits ---
    assert_eq!(rotate_left_u64(1, 0), 1, "rotate by 0 is identity");
    assert_eq!(rotate_left_u64(1, 1), 2, "rotate left 1");
    assert_eq!(
        rotate_left_u64(0x8000_0000_0000_0000, 1),
        1,
        "MSB wraps to LSB"
    );
    assert_eq!(
        rotate_left_u64(rotate_left_u64(0xABCD, 7), 64 - 7),
        0xABCD,
        "rotate left then right = identity"
    );
    println!(
        "rotate_left_u64(1,1)={}, rotate back={}",
        rotate_left_u64(1, 1),
        rotate_left_u64(rotate_left_u64(0xABCD, 7), 57)
    );

    // --- Tier 101-200: gcd_u64_branchless — Binary GCD ---
    assert_eq!(gcd_u64_branchless(12, 8), 4);
    assert_eq!(gcd_u64_branchless(8, 12), 4, "symmetric");
    assert_eq!(gcd_u64_branchless(0, 7), 7, "gcd(0, n) = n");
    assert_eq!(gcd_u64_branchless(7, 0), 7, "gcd(n, 0) = n");
    assert_eq!(gcd_u64_branchless(7, 1), 1, "gcd(n, 1) = 1");
    assert_eq!(gcd_u64_branchless(100, 75), 25);
    println!(
        "gcd(12,8)={}, gcd(100,75)={}",
        gcd_u64_branchless(12, 8),
        gcd_u64_branchless(100, 75)
    );

    // --- Tier 101-200: popcount_u128 — popcount of both val and aux combined ---
    // Implementation: val.count_ones() + aux.count_ones()
    assert_eq!(popcount_u128(0, 0), 0);
    assert_eq!(
        popcount_u128(0b1011, 0b0100),
        4,
        "3 bits in val + 1 bit in aux"
    );
    assert_eq!(popcount_u128(u64::MAX, 0), 64, "all 64 bits in val");
    assert_eq!(popcount_u128(u64::MAX, u64::MAX), 128, "all 128 bits total");
    println!(
        "popcount_u128(0b1011, 0b0100)={}",
        popcount_u128(0b1011, 0b0100)
    );

    // --- Tier 201-300: leb128_decode_u64 — LEB128 variable-length integer decode ---
    // Single-byte encoding: value < 128, high bit = 0 → value is the low 7 bits
    // val=0x05 → byte 5 (high bit clear, so it's a complete encoding)
    let encoded_five: u64 = 0x05;
    let decoded = leb128_decode_u64(encoded_five, 0);
    assert_eq!(decoded, 5, "LEB128 single byte 0x05 = 5");
    // Two-byte: 0x8001 → byte 0x80 (continuation, low 7 = 0) + byte 0x01 (value=1) → 128
    let encoded_128: u64 = 0x0180; // little-endian: first byte 0x80, second 0x01
    let decoded128 = leb128_decode_u64(encoded_128, 0);
    assert_eq!(decoded128, 128, "LEB128 two-byte 0x80 0x01 = 128");
    println!("leb128_decode(0x05)={decoded}, leb128_decode(0x0180)={decoded128}");

    // --- Composition: GCD-based normalization pipeline ---
    // Take two numbers, compute GCD, use it to normalize both,
    // then count the combined set bits in the normalized pair.
    let a: u64 = 48;
    let b: u64 = 36;
    let g = gcd_u64_branchless(a, b); // GCD = 12
    let na = a / g; // normalized: 4
    let nb = b / g; // normalized: 3
    let combined_bits = popcount_u128(na, nb); // popcount(4) + popcount(3) = 1 + 2 = 3
    assert_eq!(g, 12, "GCD(48,36)=12");
    assert_eq!(na, 4);
    assert_eq!(nb, 3);
    assert_eq!(combined_bits, 3, "popcount(4)+popcount(3) = 1+2 = 3");
    // The distance between the normalized values, rotated by their GCD mod 64
    let dist = abs_diff_u64(na, nb); // |4-3| = 1
    let rotated = rotate_left_u64(dist, g & 0x3F); // rotate 1 by 12 = 4096
    assert_eq!(rotated, 1u64 << 12, "1 rotated left by 12 = 4096");
    println!("pipeline: gcd={g}, normalized=({na},{nb}), combined_bits={combined_bits}, rotate={rotated}");

    println!("\nAll algorithm cross-section assertions passed.");
}
