//! # Bitset Algebra Example
//!
//! Demonstrates `bcinr_logic::bitset` as a standalone cluster: `rank_u64`,
//! `select_bit_u64`, `parity_u64_slice`, `jaccard_u64_slices`, `hamming_u64_slices`,
//! `intersect_u64_slices`, `union_u64_slices`, `any_bit_set_u64_slice`.
//!
//! **Doc reference:** `crates/bcinr-logic/src/bitset.rs`
//! **Also see:** `examples/branchless_pipeline.rs` — bitset used mid-pipeline.
//!
//! `rank_u64(x, pos)`: count of set bits in x at positions 0..=pos.
//! `select_bit_u64(x, n)`: position of the n-th (0-based) set bit, or None.
//! These are the building blocks of succinct data structures. Assertions below
//! would fail if rank overcounted, or if select returned the wrong position.

use bcinr::bitset::{
    any_bit_set_u64_slice, hamming_u64_slices, intersect_u64_slices, jaccard_u64_slices,
    parity_u64_slice, rank_u64, select_bit_u64, union_u64_slices,
};

fn main() {
    // --- rank_u64: popcount of bits 0..=pos ---
    let x: u64 = 0b1011_0101; // bits set at positions 0,2,4,5,7
    assert_eq!(rank_u64(x, 0), 1, "bit 0 is set → rank=1");
    assert_eq!(rank_u64(x, 1), 1, "bit 1 not set → rank unchanged");
    assert_eq!(rank_u64(x, 2), 2, "bit 2 set → rank=2");
    assert_eq!(rank_u64(x, 7), 5, "5 bits set in positions 0..=7");
    assert_eq!(rank_u64(0, 63), 0, "no bits set → rank=0");
    assert_eq!(rank_u64(u64::MAX, 63), 64, "all bits set → rank=64");
    println!("rank_u64(0b10110101, 7)={}", rank_u64(x, 7));

    // --- select_bit_u64: position of n-th set bit (0-based n) ---
    assert_eq!(select_bit_u64(x, 0), Some(0), "0th set bit at position 0");
    assert_eq!(select_bit_u64(x, 1), Some(2), "1st set bit at position 2");
    assert_eq!(select_bit_u64(x, 2), Some(4), "2nd set bit at position 4");
    assert_eq!(select_bit_u64(x, 4), Some(7), "4th set bit at position 7");
    assert_eq!(
        select_bit_u64(x, 5),
        None,
        "only 5 bits set (0-4), index 5 → None"
    );
    assert_eq!(select_bit_u64(0, 0), None, "no bits set → None");
    println!("select_bit_u64(0b10110101, 2)={:?}", select_bit_u64(x, 2));

    // rank-select round-trip: rank(select(x, n), pos) = n + 1
    for n in 0..5 {
        let pos = select_bit_u64(x, n).unwrap();
        assert_eq!(rank_u64(x, pos), n + 1, "rank-select round-trip for n={n}");
    }
    println!("rank-select round-trip: OK for all 5 set bits");

    // --- parity_u64_slice: XOR all words then popcount parity ---
    let even_bits: &[u64] = &[0b0011, 0b1100]; // 2+2 = 4 bits total → even parity
    let odd_bits: &[u64] = &[0b0111]; // 3 bits → odd parity
    assert_eq!(
        parity_u64_slice(even_bits),
        0,
        "4 set bits → even parity = 0"
    );
    assert_eq!(parity_u64_slice(odd_bits), 1, "3 set bits → odd parity = 1");
    assert_eq!(parity_u64_slice(&[]), 0, "empty slice → 0");
    println!("parity_u64_slice([0b0111])={}", parity_u64_slice(odd_bits));

    // --- jaccard_u64_slices: |A ∩ B| / |A ∪ B| ---
    let a: &[u64] = &[0b1100]; // bits 2,3
    let b: &[u64] = &[0b1010]; // bits 1,3
                               // intersection = 0b1000 (bit 3) → 1 bit; union = 0b1110 → 3 bits; jaccard = 1/3 ≈ 0.333
    let j = jaccard_u64_slices(a, b);
    let expected = 1.0f32 / 3.0f32;
    assert!((j - expected).abs() < 1e-5, "jaccard must be ~1/3, got {j}");
    assert_eq!(
        jaccard_u64_slices(&[u64::MAX], &[u64::MAX]),
        1.0,
        "identical sets → jaccard=1.0"
    );
    assert_eq!(
        jaccard_u64_slices(&[0b1100], &[0b0011]),
        0.0,
        "disjoint sets → jaccard=0.0"
    );
    println!("jaccard([0b1100],[0b1010])={j:.4}");

    // --- hamming_u64_slices: number of differing bits ---
    let h = hamming_u64_slices(&[0b1100u64], &[0b1010u64]);
    assert_eq!(h, 2, "two bits differ: bit 1 and bit 2");
    assert_eq!(
        hamming_u64_slices(&[0u64], &[0u64]),
        0,
        "identical → distance 0"
    );
    assert_eq!(
        hamming_u64_slices(&[u64::MAX], &[0u64]),
        64,
        "all bits differ"
    );
    println!("hamming_u64_slices([0b1100],[0b1010])={h}");

    // --- intersect_u64_slices / union_u64_slices (in-place) ---
    let mut set_a = vec![0b1111u64];
    let set_b = [0b1010u64];
    intersect_u64_slices(&mut set_a, &set_b);
    assert_eq!(set_a[0], 0b1010, "intersection: 0b1111 & 0b1010 = 0b1010");

    let mut set_c = vec![0b0101u64];
    let set_d = [0b1010u64];
    union_u64_slices(&mut set_c, &set_d);
    assert_eq!(set_c[0], 0b1111, "union: 0b0101 | 0b1010 = 0b1111");
    println!("intersect 0b1111 & 0b1010 = {:#06b}", set_a[0]);
    println!("union 0b0101 | 0b1010 = {:#06b}", set_c[0]);

    // --- any_bit_set_u64_slice ---
    assert!(
        any_bit_set_u64_slice(&[0u64, 1u64]),
        "second word has a bit set"
    );
    assert!(!any_bit_set_u64_slice(&[0u64, 0u64]), "no bits set");
    assert!(!any_bit_set_u64_slice(&[]), "empty → false");
    println!(
        "any_bit_set_u64_slice([0,1])={}",
        any_bit_set_u64_slice(&[0, 1])
    );

    println!("\nAll bitset algebra assertions passed.");
}
