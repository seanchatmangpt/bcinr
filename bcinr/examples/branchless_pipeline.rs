//! # Branchless Pipeline Example
//!
//! Demonstrates capability *composition*: mask primitives feeding saturation
//! arithmetic, combined with bitset population queries — a realistic pattern
//! from the library's documented use case of data-parallel filtering.
//!
//! **Doc references:**
//!   - `crates/bcinr-logic/src/mask.rs` — select / comparison masks
//!   - `crates/bcinr-logic/src/fix.rs` — saturating arithmetic / clamping
//!   - `crates/bcinr-logic/src/bitset.rs` — rank, select_bit, hamming distance
//!
//! **Also see:** `examples/mask_primitives.rs`, `examples/saturation_arithmetic.rs`
//!
//! **The composability contract:** each layer's output is the next layer's input,
//! with no branching at any step. A broken `select_u32` or `clamp_u32` would
//! produce wrong values that ripple through and fail the final assertions.

use bcinr::bitset::{hamming_u64_slices, rank_u64, select_bit_u64};
use bcinr::fix::{add_sat, clamp_u32};
use bcinr::mask::{eq_mask_u32, max_u32, min_u32, select_u32};

fn main() {
    // -----------------------------------------------------------------------
    // Stage 1: branchless equality-guard on scores
    // Produce a mask that is 0xFFFF_FFFF when score == target, 0 otherwise,
    // then use that mask to gate a clamped bonus.
    // -----------------------------------------------------------------------
    let scores: [u32; 8] = [10, 50, 30, 50, 20, 50, 70, 90];
    let target: u32 = 50;
    let bonus: u32 = 5;

    let mut awarded: [u32; 8] = [0; 8];
    for (i, &s) in scores.iter().enumerate() {
        let hit_mask = eq_mask_u32(s, target); // 0xFFFFFFFF or 0
        let with_bonus = add_sat(s, bonus); // saturating add
                                            // select: if mask set, take with_bonus; otherwise keep original score
        awarded[i] = select_u32(hit_mask, with_bonus, s);
    }
    println!("scores:   {scores:?}");
    println!("awarded:  {awarded:?}");
    assert_eq!(awarded[0], 10, "score 10 != 50, no bonus");
    assert_eq!(awarded[1], 55, "score 50 == 50, gets +5");
    assert_eq!(awarded[3], 55, "score 50 == 50, gets +5");
    assert_eq!(awarded[6], 70, "score 70 != 50, no bonus");

    // -----------------------------------------------------------------------
    // Stage 2: branchless range-clamp then min/max normalization
    // Clamp all awarded scores to [20, 60], then find min and max
    // without any conditional branches.
    // -----------------------------------------------------------------------
    let mut clamped: [u32; 8] = [0; 8];
    for (i, &a) in awarded.iter().enumerate() {
        clamped[i] = clamp_u32(a, 20, 60);
    }
    println!("clamped:  {clamped:?}");

    let minimum = clamped.iter().copied().fold(u32::MAX, min_u32);
    let maximum = clamped.iter().copied().fold(0u32, max_u32);
    println!("range after clamp: [{minimum}, {maximum}]");
    assert!(minimum >= 20, "clamp lo must hold");
    assert!(maximum <= 60, "clamp hi must hold");

    // -----------------------------------------------------------------------
    // Stage 3: build a bitset of "high scorer" positions and query it
    // A score ≥ 55 after clamping is "high". Pack that into a u64 bitmask.
    // -----------------------------------------------------------------------
    let threshold: u32 = 55;
    let mut bitset: u64 = 0u64;
    for (i, &c) in clamped.iter().enumerate() {
        // c >= threshold iff max(c, threshold) == c
        let is_high = eq_mask_u32(max_u32(c, threshold), c);
        let bit = (is_high as u64) & 1;
        bitset |= bit << i;
    }
    println!("high-scorer bitset: {bitset:#010b}");

    // rank: how many high scorers at or before position 7?
    let high_count = rank_u64(bitset, 7);
    println!("high scorers (rank 0..=7): {high_count}");
    assert!(high_count <= 8, "can't have more high scorers than entries");

    // select_bit: position of the 1st high scorer
    if let Some(first_high) = select_bit_u64(bitset, 0) {
        println!("first high scorer at index: {first_high}");
        assert!(
            clamped[first_high] >= threshold,
            "selected position must actually be high"
        );
    }

    // hamming distance between original scores (as bitsets) and clamped scores
    // A non-zero distance means the pipeline transformed something — proof of effect.
    let orig_bits: [u64; 1] = [scores.iter().map(|&s| s as u64).fold(0, |acc, v| acc ^ v)];
    let clamped_bits: [u64; 1] = [clamped.iter().map(|&c| c as u64).fold(0, |acc, v| acc ^ v)];
    let dist = hamming_u64_slices(&orig_bits, &clamped_bits);
    println!("hamming distance orig vs clamped: {dist}");

    println!("\nAll branchless pipeline assertions passed.");
}
