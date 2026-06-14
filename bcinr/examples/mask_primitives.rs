//! # Mask Primitives Example
//!
//! Demonstrates the core branchless mask API from `bcinr_logic::mask`:
//! `select_u32`, `min_u32`, `max_u32`, `abs_i32`, `eq_mask_u32`, `lt_mask_u32`.
//!
//! **Doc reference:** `crates/bcinr-logic/src/mask.rs`
//!
//! A mask in this API is either `0x0000_0000` (false) or `0xFFFF_FFFF` (true).
//! `select_u32(mask, a, b)` returns `a` when mask is all-ones, `b` when all-zeros —
//! with zero conditional branches. The contract: a broken `select_u32` (e.g. one
//! that ignores the mask and always returns `a`) would fail the assertions below.

use bcinr::mask::{abs_i32, eq_mask_u32, lt_mask_u32, max_u32, min_u32, select_u32};

fn main() {
    // --- select_u32: mask 0xFFFF_FFFF picks first arg ---
    let chose_a = select_u32(0xFFFF_FFFF, 10, 20);
    assert_eq!(chose_a, 10, "all-ones mask must select first arg");
    let chose_b = select_u32(0x0000_0000, 10, 20);
    assert_eq!(chose_b, 20, "all-zeros mask must select second arg");
    println!("select_u32: mask=0xFFFFFFFF → {chose_a}, mask=0x0 → {chose_b}");

    // --- eq_mask_u32: produces a full mask, not just 0/1 ---
    let eq = eq_mask_u32(42, 42);
    assert_eq!(eq, 0xFFFF_FFFF, "equal inputs must yield all-ones mask");
    let ne = eq_mask_u32(42, 43);
    assert_eq!(ne, 0, "unequal inputs must yield all-zeros mask");
    println!("eq_mask_u32: 42==42 → {eq:#010x}, 42==43 → {ne:#010x}");

    // --- lt_mask_u32 ---
    let lt = lt_mask_u32(3, 7);
    assert_eq!(lt, 0xFFFF_FFFF, "3 < 7 must yield all-ones mask");
    let not_lt = lt_mask_u32(7, 3);
    assert_eq!(not_lt, 0, "7 < 3 must yield all-zeros mask");
    println!("lt_mask_u32: 3<7 → {lt:#010x}, 7<3 → {not_lt:#010x}");

    // --- min_u32 / max_u32 ---
    assert_eq!(min_u32(5, 8), 5);
    assert_eq!(min_u32(8, 5), 5);
    assert_eq!(max_u32(5, 8), 8);
    assert_eq!(max_u32(8, 5), 8);
    assert_eq!(min_u32(u32::MAX, u32::MAX), u32::MAX, "identity at MAX");
    println!("min_u32(5,8)={}, max_u32(5,8)={}", min_u32(5, 8), max_u32(5, 8));

    // --- abs_i32: edge cases (note: i32::MIN has no positive representation) ---
    assert_eq!(abs_i32(0), 0);
    assert_eq!(abs_i32(7), 7);
    assert_eq!(abs_i32(-7), 7);
    assert_eq!(abs_i32(i32::MAX), i32::MAX);
    println!("abs_i32(-7)={}", abs_i32(-7));

    // --- composition: branchless clamp-via-mask ---
    // min(max(val, lo), hi) without any conditional jump
    let val: u32 = 150;
    let lo: u32 = 0;
    let hi: u32 = 100;
    let clamped = min_u32(max_u32(val, lo), hi);
    assert_eq!(clamped, 100, "150 clamped to [0,100] must be 100");
    println!("branchless clamp({val}, {lo}, {hi}) = {clamped}");

    println!("\nAll mask primitive assertions passed.");
}
