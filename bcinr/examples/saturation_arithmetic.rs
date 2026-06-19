//! # Saturation Arithmetic Example
//!
//! Demonstrates `bcinr_logic::fix`: `add_sat_u8`, `sub_sat_u8`, `clamp_u32`,
//! and `bucketize_u32`.
//!
//! **Doc reference:** `crates/bcinr-logic/src/fix.rs`
//! **Also see:** `examples/mask_primitives.rs` — mask layer these saturating ops build on.
//!
//! Saturation arithmetic never overflows or underflows — values clamp at the
//! type boundary. The assertions below would break if any function overflowed
//! or returned `Result` instead of the direct saturated value.

// All saturation primitives are in bcinr_logic::fix (re-exported via bcinr::fix)
use bcinr::fix::{add_sat, bucketize_u32, clamp_u32};

fn main() {
    // --- add_sat (u32): saturates at u32::MAX, never wraps ---
    assert_eq!(add_sat(200, 100), 300, "no saturation when room exists");
    assert_eq!(add_sat(u32::MAX, 1), u32::MAX, "MAX+1 must stay at MAX");
    assert_eq!(add_sat(u32::MAX, u32::MAX), u32::MAX, "MAX+MAX saturates");
    assert_eq!(add_sat(0, 0), 0);
    println!(
        "add_sat: 200+100={}, MAX+1={}",
        add_sat(200, 100),
        add_sat(u32::MAX, 1)
    );

    // --- clamp_u32: returns u32 directly (not Result) ---
    assert_eq!(clamp_u32(150, 0, 100), 100, "150 above hi clamps to 100");
    assert_eq!(clamp_u32(50, 0, 100), 50, "50 in range passes through");
    assert_eq!(clamp_u32(0, 10, 100), 10, "0 below lo clamps to 10");
    assert_eq!(clamp_u32(0, 0, 0), 0, "identity at boundary");
    println!(
        "clamp_u32: 150∈[0,100]={}, 50∈[0,100]={}, 0∈[10,100]={}",
        clamp_u32(150, 0, 100),
        clamp_u32(50, 0, 100),
        clamp_u32(0, 10, 100)
    );

    // --- bucketize_u32: round down to nearest multiple of step ---
    assert_eq!(bucketize_u32(0, 16), 0);
    assert_eq!(bucketize_u32(15, 16), 0);
    assert_eq!(bucketize_u32(16, 16), 16);
    assert_eq!(bucketize_u32(31, 16), 16);
    assert_eq!(bucketize_u32(32, 16), 32);
    println!(
        "bucketize_u32(15,16)={}, bucketize_u32(31,16)={}",
        bucketize_u32(15, 16),
        bucketize_u32(31, 16)
    );

    println!("\nAll saturation arithmetic assertions passed.");
}
