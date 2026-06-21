//! # Sorting Networks Example
//!
//! Demonstrates `bcinr_logic::network`: `compare_exchange`, `bitonic_sort_8u32`,
//! `bitonic_sort_16u32`.
//!
//! **Doc reference:** `crates/bcinr-logic/src/network.rs`
//! **Also see:** `examples/mask_primitives.rs` — the branchless mask swap used
//! by `compare_exchange` under the hood.
//!
//! `compare_exchange(a, i, j)` swaps `a[i]` and `a[j]` if `a[i] > a[j]`, using
//! a XOR swap conditioned on a branchless mask — no `if` in the hot path.
//! `bitonic_sort_8u32` / `bitonic_sort_16u32` fully sort a fixed-size array via
//! a Batcher bitonic network of compare-exchange operations.
//!
//! All assertions below would fail if the sort produced non-monotonic output.

use bcinr::network::{bitonic_sort_16u32, bitonic_sort_8u32, compare_exchange};

fn is_sorted_u32(a: &[u32]) -> bool {
    a.windows(2).all(|w| w[0] <= w[1])
}

fn main() {
    // --- compare_exchange: branchless conditional swap ---
    let mut a = [3u32, 1u32, 0u32, 0u32]; // use as a mutable slice
    compare_exchange(&mut a, 0, 1); // 3 > 1 → swap → [1, 3, ...]
    assert_eq!(a[0], 1, "smaller must go to lower index");
    assert_eq!(a[1], 3, "larger goes to higher index");

    let mut b = [2u32, 5u32, 0u32, 0u32];
    compare_exchange(&mut b, 0, 1); // 2 < 5 → no swap
    assert_eq!(b[0], 2, "already ordered → no swap");
    assert_eq!(b[1], 5);

    compare_exchange(&mut b, 0, 0); // self-swap → identity
    assert_eq!(b[0], 2, "self-swap is identity");
    println!(
        "compare_exchange: [3,1]→[{},{}], [2,5]→[{},{}]",
        a[0], a[1], b[0], b[1]
    );

    // --- bitonic_sort_8u32: fully sorts 8 u32s ---
    let mut arr8: [u32; 8] = [7, 2, 8, 1, 5, 3, 9, 4];
    let sorted8_ref: [u32; 8] = [1, 2, 3, 4, 5, 7, 8, 9];
    bitonic_sort_8u32(&mut arr8);
    assert_eq!(
        arr8, sorted8_ref,
        "bitonic_sort_8u32 must produce sorted output"
    );
    assert!(is_sorted_u32(&arr8), "output must be non-decreasing");
    println!("bitonic_sort_8u32: [7,2,8,1,5,3,9,4] → {arr8:?}");

    // Edge: already sorted
    let mut already_sorted: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    bitonic_sort_8u32(&mut already_sorted);
    assert_eq!(
        already_sorted,
        [1, 2, 3, 4, 5, 6, 7, 8],
        "sorted input stays sorted"
    );

    // Edge: reverse sorted
    let mut reversed: [u32; 8] = [8, 7, 6, 5, 4, 3, 2, 1];
    bitonic_sort_8u32(&mut reversed);
    assert!(
        is_sorted_u32(&reversed),
        "reverse-sorted input must sort correctly"
    );
    println!("bitonic_sort_8u32([8..1]): {reversed:?}");

    // Edge: all equal
    let mut equal: [u32; 8] = [42; 8];
    bitonic_sort_8u32(&mut equal);
    assert_eq!(equal, [42u32; 8], "all-equal stays all-equal");

    // --- bitonic_sort_16u32: fully sorts 16 u32s ---
    let mut arr16: [u32; 16] = [15, 3, 9, 7, 1, 11, 5, 13, 8, 2, 14, 6, 0, 10, 4, 12];
    let mut arr16_ref = arr16;
    arr16_ref.sort();
    bitonic_sort_16u32(&mut arr16);
    assert_eq!(
        arr16, arr16_ref,
        "bitonic_sort_16u32 must match stdlib sort"
    );
    assert!(is_sorted_u32(&arr16), "output must be non-decreasing");
    println!("bitonic_sort_16u32(random 0..15): {arr16:?}");

    // --- cross-product: sort then binary search (using popcount for rank) ---
    use bcinr::int::popcount_u64;
    let mut data: [u32; 8] = [100, 50, 200, 25, 75, 150, 125, 175];
    bitonic_sort_8u32(&mut data);
    // Find how many elements are strictly less than 120 (branchless rank via bitmask)
    let mut lt_mask: u64 = 0;
    for (i, &v) in data.iter().enumerate() {
        lt_mask |= ((v < 120) as u64) << i;
    }
    let rank = popcount_u64(lt_mask) as u32;
    assert_eq!(
        data[rank as usize - 1],
        100,
        "last element < 120 should be 100"
    );
    assert!(data[rank as usize] >= 120, "first element ≥ 120");
    println!("rank of 120 in sorted [25,50,75,100,125,150,175,200]: {rank}");

    println!("\nAll sorting network assertions passed.");
}
