//! # Horizontal Reductions Example
//!
//! Demonstrates `bcinr_logic::reduce`: CC=1 horizontal OR/AND/XOR over u32 slices,
//! and SWAR byte-parallel sum/max/min over a packed u64 word.
//!
//! **Doc reference:** `crates/bcinr-logic/src/reduce.rs`
//! **Also see:** `examples/branchless_pipeline.rs` — uses bitset reductions in a pipeline.
//!
//! `horizontal_and_u32` on an empty slice returns 0 (not u32::MAX) — this is the
//! documented behavior verified below. An implementation that returned u32::MAX on
//! empty input would fail the empty-slice assertion.

// NOTE: horizontal_max_u8x8 and horizontal_min_u8x8 are excluded from this
// example — they panic with integer overflow in debug builds due to a SWAR
// comparison that uses plain `+` instead of `wrapping_add`, causing carry
// propagation across byte lanes. Filed as defect in DOC_COVERAGE_LOG.md.
use bcinr::reduce::{horizontal_and_u32, horizontal_or_u32, horizontal_sum_u8x8, horizontal_xor_u32};

fn main() {
    // --- horizontal_or_u32: any bit set? ---
    assert_eq!(horizontal_or_u32(&[]), 0, "empty → 0");
    assert_eq!(horizontal_or_u32(&[0, 0, 0]), 0);
    assert_eq!(horizontal_or_u32(&[0b0001, 0b0010, 0b0100]), 0b0111, "OR folds all set bits");
    assert_eq!(horizontal_or_u32(&[0xFFFF_0000, 0x0000_FFFF]), 0xFFFF_FFFF, "no overlap, full coverage");
    println!("horizontal_or_u32([1,2,4])={:#05b}", horizontal_or_u32(&[0b0001, 0b0010, 0b0100]));

    // --- horizontal_and_u32: all bits set? ---
    assert_eq!(horizontal_and_u32(&[]), 0, "empty → 0 (not u32::MAX)");
    assert_eq!(horizontal_and_u32(&[u32::MAX, u32::MAX]), u32::MAX, "all-ones AND all-ones = all-ones");
    assert_eq!(horizontal_and_u32(&[0b1111, 0b0110]), 0b0110, "AND keeps only common bits");
    assert_eq!(horizontal_and_u32(&[0xFF, 0x0F]), 0x0F);
    println!("horizontal_and_u32([0b1111,0b0110])={:#06b}", horizontal_and_u32(&[0b1111, 0b0110]));

    // --- horizontal_xor_u32: parity / toggle ---
    assert_eq!(horizontal_xor_u32(&[]), 0);
    assert_eq!(horizontal_xor_u32(&[0xFF]), 0xFF, "single element");
    assert_eq!(horizontal_xor_u32(&[0xFF, 0xFF]), 0, "same value XOR'd = 0");
    assert_eq!(horizontal_xor_u32(&[0b0001, 0b0011, 0b0101]), 0b0111);
    println!("horizontal_xor_u32([1,3,5])={:#05b}", horizontal_xor_u32(&[0b0001, 0b0011, 0b0101]));

    // --- horizontal_sum_u8x8: sum 8 bytes packed in a u64 ---
    // Pack bytes [1,2,3,4,5,6,7,8] as little-endian u64
    let packed: u64 = 0x0807_0605_0403_0201;
    let sum = horizontal_sum_u8x8(packed);
    assert_eq!(sum, 36, "1+2+3+4+5+6+7+8=36");
    let zeros: u64 = 0;
    assert_eq!(horizontal_sum_u8x8(zeros), 0);
    let all_one: u64 = 0x0101_0101_0101_0101; // eight bytes of value 1
    assert_eq!(horizontal_sum_u8x8(all_one), 8, "eight 1-bytes sum to 8");
    println!("horizontal_sum_u8x8([1..8])={sum}");

    // --- cross-product: use horizontal_or to build a presence bitmap ---
    // from a set of flags, horizontal_or gives the union; horizontal_and gives the intersection
    let flags: [u32; 4] = [0b1100, 0b1010, 0b1001, 0b1111];
    let union = horizontal_or_u32(&flags);
    let intersection = horizontal_and_u32(&flags);
    assert_eq!(union, 0b1111, "union of all flags covers all bits");
    assert_eq!(intersection, 0b1000, "only bit 3 is in all four flags");
    println!("flags union={union:#06b}, intersection={intersection:#06b}");

    println!("\nAll horizontal reduction assertions passed.");
}
