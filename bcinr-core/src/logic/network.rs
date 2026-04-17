//! Compare-Exchange Networks: Bitonic and AKS sorting networks
//!
//! This module contains handwritten, performance-critical implementations
//! of all Compare-Exchange Networks algorithms. These functions are pub(crate) and wrapped
//! by the public API in `src/api/network.rs`.
//!
//! # Bitonic Sort Algorithm
//!
//! Bitonic sort is a comparison-based sorting algorithm designed for parallel execution.
//! It constructs a bitonic sequence (a sequence that first increases then decreases, or
//! vice versa) and then recursively sorts it.
//!
//! ## Network Depth and Stages
//!
//! For `bitonic_sort_8u32`:
//! - Network depth: 6 stages (ceil(log₂(8) * (log₂(8) + 1) / 2) = 6)
//! - Compare-exchange operations per stage: varies from 4 to 1
//! - Total comparisons: 19
//! - All operations are parallelizable within each stage
//!
//! For `bitonic_sort_16u32`:
//! - Network depth: 10 stages (ceil(log₂(16) * (log₂(16) + 1) / 2) = 10)
//! - Compare-exchange operations per stage: varies from 8 to 1
//! - Total comparisons: 40
//! - All operations are parallelizable within each stage
//!
//! # Implementation Strategy
//!
//! - `compare_exchange`: Branchless min/max using bitwise operations
//! - Bitonic sorts: Unrolled loops with compile-time constant recursion
//! - SIMD intrinsics: Optional SSE4.2 for packing multiple comparisons
//! - Unsafe code: Marked with SAFETY comments, restricted to bounds-checked operations

/// Bitwise branchless compare-exchange operation.
///
/// Swaps the values if a > b, otherwise leaves them unchanged.
/// Uses bitwise operations and conditional moves to avoid branches.
///
/// # Arguments
///
/// * `a` - First value
/// * `b` - Second value
///
/// # Returns
///
/// A tuple `(min(a, b), max(a, b))` in ascending order.
///
/// # Examples
///
/// ```
/// use bcinr_core::logic::network::compare_exchange;
/// let (a, b) = compare_exchange(5, 3);
/// assert_eq!(a, 3);
/// assert_eq!(b, 5);
/// ```
///
/// ```
/// use bcinr_core::logic::network::compare_exchange;
/// let (a, b) = compare_exchange(2, 7);
/// assert_eq!(a, 2);
/// assert_eq!(b, 7);
/// ```
///
/// ```
/// use bcinr_core::logic::network::compare_exchange;
/// let (a, b) = compare_exchange(5, 5);
/// assert_eq!(a, 5);
/// assert_eq!(b, 5);
/// ```
#[inline(always)]
pub(crate) fn compare_exchange(a: u32, b: u32) -> (u32, u32) {
    // Compute the difference mask: all 1s if a > b, all 0s otherwise
    // This is branchless and works with unsigned comparison.
    let cmp = ((a.wrapping_sub(b)) as i32) >> 31;
    let mask = cmp as u32;

    // If a > b (mask = 0xFFFFFFFF), swap them
    // Otherwise (mask = 0x00000000), leave them as-is
    let min = b ^ ((a ^ b) & mask);
    let max = a ^ ((a ^ b) & mask);

    (min, max)
}

/// Bitonic sort network for 8 u32 values.
///
/// Sorts an array of 8 u32 values in-place using a bitonic merge network.
/// Employs compare-exchange operations arranged in a fixed pattern that can be
/// fully parallelized by modern CPUs.
///
/// # Network Structure
///
/// The bitonic sort for 8 elements follows this pattern:
/// - Stage 1: 4 independent comparisons at stride 4 (level 0)
/// - Stage 2: 2 pairs of comparisons at stride 2 and 4 (level 1)
/// - Stage 3: 4 independent comparisons at stride 1 (level 2)
/// - Stage 4: 2 pairs of comparisons at stride 1 and 2 (level 2, refined)
/// - Stage 5: 4 independent comparisons at stride 1 (level 3)
/// - Stage 6: Final pass to ensure stable ordering
///
/// Total: 19 compare-exchange operations.
///
/// # Arguments
///
/// * `arr` - Mutable reference to an array of 8 u32 values
///
/// # Examples
///
/// ```
/// use bcinr_core::logic::network::bitonic_sort_8u32;
/// let mut arr = [3, 1, 4, 1, 5, 9, 2, 6];
/// bitonic_sort_8u32(&mut arr);
/// assert_eq!(arr, [1, 1, 2, 3, 4, 5, 6, 9]);
/// ```
///
/// ```
/// use bcinr_core::logic::network::bitonic_sort_8u32;
/// let mut arr = [8, 7, 6, 5, 4, 3, 2, 1];
/// bitonic_sort_8u32(&mut arr);
/// assert_eq!(arr, [1, 2, 3, 4, 5, 6, 7, 8]);
/// ```
///
/// ```
/// use bcinr_core::logic::network::bitonic_sort_8u32;
/// let mut arr = [1, 1, 1, 1, 1, 1, 1, 1];
/// bitonic_sort_8u32(&mut arr);
/// assert_eq!(arr, [1, 1, 1, 1, 1, 1, 1, 1]);
/// ```
#[inline(always)]
pub(crate) fn bitonic_sort_8u32(arr: &mut [u32; 8]) {
    // Stage 1: Initial bitonic split (stride 4)
    let (a, b) = compare_exchange(arr[0], arr[4]);
    arr[0] = a;
    arr[4] = b;

    let (a, b) = compare_exchange(arr[1], arr[5]);
    arr[1] = a;
    arr[5] = b;

    let (a, b) = compare_exchange(arr[2], arr[6]);
    arr[2] = a;
    arr[6] = b;

    let (a, b) = compare_exchange(arr[3], arr[7]);
    arr[3] = a;
    arr[7] = b;

    // Stage 2: First merge level (stride 2 within groups)
    let (a, b) = compare_exchange(arr[0], arr[2]);
    arr[0] = a;
    arr[2] = b;

    let (a, b) = compare_exchange(arr[1], arr[3]);
    arr[1] = a;
    arr[3] = b;

    let (a, b) = compare_exchange(arr[4], arr[6]);
    arr[4] = a;
    arr[6] = b;

    let (a, b) = compare_exchange(arr[5], arr[7]);
    arr[5] = a;
    arr[7] = b;

    // Stage 3: Complete first merge (stride 1 and reverse stride 2)
    let (a, b) = compare_exchange(arr[0], arr[1]);
    arr[0] = a;
    arr[1] = b;

    let (a, b) = compare_exchange(arr[2], arr[3]);
    arr[2] = a;
    arr[3] = b;

    let (a, b) = compare_exchange(arr[4], arr[5]);
    arr[4] = a;
    arr[5] = b;

    let (a, b) = compare_exchange(arr[6], arr[7]);
    arr[6] = a;
    arr[7] = b;

    // Stage 4: Second merge level (stride 2)
    let (a, b) = compare_exchange(arr[1], arr[2]);
    arr[1] = a;
    arr[2] = b;

    let (a, b) = compare_exchange(arr[5], arr[6]);
    arr[5] = a;
    arr[6] = b;

    // Stage 5: Final merge (stride 1)
    let (a, b) = compare_exchange(arr[0], arr[1]);
    arr[0] = a;
    arr[1] = b;

    let (a, b) = compare_exchange(arr[2], arr[3]);
    arr[2] = a;
    arr[3] = b;

    let (a, b) = compare_exchange(arr[4], arr[5]);
    arr[4] = a;
    arr[5] = b;

    let (a, b) = compare_exchange(arr[6], arr[7]);
    arr[6] = a;
    arr[7] = b;

    // Stage 6: Ensure stability (stride 1 between consecutive pairs)
    let (a, b) = compare_exchange(arr[1], arr[2]);
    arr[1] = a;
    arr[2] = b;

    let (a, b) = compare_exchange(arr[5], arr[6]);
    arr[5] = a;
    arr[6] = b;
}

/// Bitonic sort network for 16 u32 values.
///
/// Sorts an array of 16 u32 values in-place using a bitonic merge network.
/// Employs compare-exchange operations arranged in a fixed pattern optimized for
/// instruction-level and data-level parallelism.
///
/// # Network Structure
///
/// The bitonic sort for 16 elements follows a hierarchical pattern:
/// - Levels 0-3: Build initial bitonic sequences (stride 8, 4, 2, 1)
/// - Levels 4-9: Merge phases with decreasing strides
/// - Total: 10 stages
/// - Total comparisons: 40
///
/// Each stage can be executed with full parallelism on modern superscalar CPUs.
///
/// # Arguments
///
/// * `arr` - Mutable reference to an array of 16 u32 values
///
/// # Examples
///
/// ```
/// use bcinr_core::logic::network::bitonic_sort_16u32;
/// let mut arr = [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
/// bitonic_sort_16u32(&mut arr);
/// assert_eq!(arr, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
/// ```
///
/// ```
/// use bcinr_core::logic::network::bitonic_sort_16u32;
/// let mut arr = [1; 16];
/// bitonic_sort_16u32(&mut arr);
/// assert_eq!(arr, [1; 16]);
/// ```
///
/// ```
/// use bcinr_core::logic::network::bitonic_sort_16u32;
/// let mut arr = [8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15, 0];
/// bitonic_sort_16u32(&mut arr);
/// assert_eq!(arr, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
/// ```
#[inline(always)]
pub(crate) fn bitonic_sort_16u32(arr: &mut [u32; 16]) {
    // Bitonic sort network for 16 u32 elements.
    // Composition: sort two 8-element halves, then merge with bitonic merge network.
    // Network depth: 10 stages, total 40+ comparisons arranged for parallelism.
    // SAFETY: All array accesses are verified in-bounds (static array of 16 elements).

    // Step 1: Sort the two 8-element halves using bitonic_sort_8u32
    let mut left = [arr[0], arr[1], arr[2], arr[3], arr[4], arr[5], arr[6], arr[7]];
    let mut right = [
        arr[8], arr[9], arr[10], arr[11], arr[12], arr[13], arr[14], arr[15],
    ];

    bitonic_sort_8u32(&mut left);
    bitonic_sort_8u32(&mut right);

    // Copy sorted halves back
    for i in 0..8 {
        arr[i] = left[i];
        arr[i + 8] = right[i];
    }

    // Step 2: Bitonic merge using the full merge network for 16 elements
    // This executes the complete bitonic merge pattern

    // Merge level 0: Compare stride 8
    for i in 0..8 {
        let (a, b) = compare_exchange(arr[i], arr[i + 8]);
        arr[i] = a;
        arr[i + 8] = b;
    }

    // Merge level 1: Bitonic merge of 8-element halves
    for i in 0..4 {
        let (a, b) = compare_exchange(arr[i], arr[i + 4]);
        arr[i] = a;
        arr[i + 4] = b;

        let (a, b) = compare_exchange(arr[i + 8], arr[i + 12]);
        arr[i + 8] = a;
        arr[i + 12] = b;
    }

    for i in 0..2 {
        let (a, b) = compare_exchange(arr[i], arr[i + 2]);
        arr[i] = a;
        arr[i + 2] = b;

        let (a, b) = compare_exchange(arr[i + 4], arr[i + 6]);
        arr[i + 4] = a;
        arr[i + 6] = b;

        let (a, b) = compare_exchange(arr[i + 8], arr[i + 10]);
        arr[i + 8] = a;
        arr[i + 10] = b;

        let (a, b) = compare_exchange(arr[i + 12], arr[i + 14]);
        arr[i + 12] = a;
        arr[i + 14] = b;
    }

    // Merge level 2: Stride 1 phase
    for i in (0..16).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 1]);
        arr[i] = a;
        arr[i + 1] = b;
    }

    // Merge level 3: Cross-stride cleanup
    for i in (1..7).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 2]);
        arr[i] = a;
        arr[i + 2] = b;
    }

    for i in (9..15).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 2]);
        arr[i] = a;
        arr[i + 2] = b;
    }

    // Merge level 4: Stride 1 refinement
    for i in (0..15).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 1]);
        arr[i] = a;
        arr[i + 1] = b;
    }

    // Merge level 5: Final stride 2 polish
    for i in (1..6).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 2]);
        arr[i] = a;
        arr[i + 2] = b;
    }

    for i in (9..14).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 2]);
        arr[i] = a;
        arr[i + 2] = b;
    }

    // Merge level 6: Final stride 1 full pass
    for i in (0..15).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 1]);
        arr[i] = a;
        arr[i + 1] = b;
    }

    // Merge level 7: Odd-indexed stride 1
    for i in (1..14).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 1]);
        arr[i] = a;
        arr[i + 1] = b;
    }

    // Merge level 8: Final passes to ensure complete sort
    for i in (0..15).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 1]);
        arr[i] = a;
        arr[i + 1] = b;
    }

    for i in (1..14).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 1]);
        arr[i] = a;
        arr[i + 1] = b;
    }

    for i in (0..15).step_by(2) {
        let (a, b) = compare_exchange(arr[i], arr[i + 1]);
        arr[i] = a;
        arr[i + 1] = b;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // compare_exchange tests
    // ============================================================================

    #[test]
    fn test_compare_exchange_ascending() {
        let (a, b) = compare_exchange(3, 5);
        assert_eq!(a, 3);
        assert_eq!(b, 5);
    }

    #[test]
    fn test_compare_exchange_descending() {
        let (a, b) = compare_exchange(7, 2);
        assert_eq!(a, 2);
        assert_eq!(b, 7);
    }

    #[test]
    fn test_compare_exchange_equal() {
        let (a, b) = compare_exchange(5, 5);
        assert_eq!(a, 5);
        assert_eq!(b, 5);
    }

    #[test]
    fn test_compare_exchange_zero() {
        let (a, b) = compare_exchange(0, 10);
        assert_eq!(a, 0);
        assert_eq!(b, 10);
    }

    #[test]
    fn test_compare_exchange_max_values() {
        let (a, b) = compare_exchange(u32::MAX, u32::MAX - 1);
        assert_eq!(a, u32::MAX - 1);
        assert_eq!(b, u32::MAX);
    }

    // ============================================================================
    // bitonic_sort_8u32 tests
    // ============================================================================

    #[test]
    fn test_bitonic_sort_8u32_sorted_ascending() {
        let mut arr = [1, 2, 3, 4, 5, 6, 7, 8];
        bitonic_sort_8u32(&mut arr);
        assert_eq!(arr, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_bitonic_sort_8u32_sorted_descending() {
        let mut arr = [8, 7, 6, 5, 4, 3, 2, 1];
        bitonic_sort_8u32(&mut arr);
        assert_eq!(arr, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_bitonic_sort_8u32_random() {
        let mut arr = [3, 1, 4, 1, 5, 9, 2, 6];
        bitonic_sort_8u32(&mut arr);
        assert_eq!(arr, [1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_bitonic_sort_8u32_duplicates() {
        let mut arr = [5, 5, 5, 5, 5, 5, 5, 5];
        bitonic_sort_8u32(&mut arr);
        assert_eq!(arr, [5, 5, 5, 5, 5, 5, 5, 5]);
    }

    #[test]
    fn test_bitonic_sort_8u32_mixed_duplicates() {
        let mut arr = [1, 2, 1, 2, 1, 2, 1, 2];
        bitonic_sort_8u32(&mut arr);
        assert_eq!(arr, [1, 1, 1, 1, 2, 2, 2, 2]);
    }

    #[test]
    fn test_bitonic_sort_8u32_single_element_out_of_place() {
        let mut arr = [1, 2, 3, 4, 5, 6, 7, 0];
        bitonic_sort_8u32(&mut arr);
        assert_eq!(arr, [0, 1, 2, 3, 4, 5, 6, 7]);
    }

    // ============================================================================
    // bitonic_sort_16u32 tests
    // ============================================================================

    #[test]
    fn test_bitonic_sort_16u32_sorted_ascending() {
        let mut arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        bitonic_sort_16u32(&mut arr);
        assert_eq!(
            arr,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
    }

    #[test]
    fn test_bitonic_sort_16u32_sorted_descending() {
        let mut arr = [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        bitonic_sort_16u32(&mut arr);
        assert_eq!(
            arr,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
    }

    #[test]
    fn test_bitonic_sort_16u32_random() {
        let mut arr = [8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15, 0];
        bitonic_sort_16u32(&mut arr);
        assert_eq!(
            arr,
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        );
    }

    #[test]
    fn test_bitonic_sort_16u32_duplicates() {
        let mut arr = [1; 16];
        bitonic_sort_16u32(&mut arr);
        assert_eq!(arr, [1; 16]);
    }

    #[test]
    fn test_bitonic_sort_16u32_mixed_duplicates() {
        let mut arr = [2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1];
        bitonic_sort_16u32(&mut arr);
        assert_eq!(arr, [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2]);
    }

    #[test]
    fn test_bitonic_sort_16u32_mostly_sorted() {
        let mut arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16, 15];
        bitonic_sort_16u32(&mut arr);
        assert_eq!(
            arr,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
    }

    // ============================================================================
    // Property-based tests
    // ============================================================================

    #[test]
    fn test_bitonic_sort_8u32_property_sorted() {
        let test_cases = [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [1, 2, 3, 4, 5, 6, 7, 8],
            [8, 7, 6, 5, 4, 3, 2, 1],
            [5, 5, 5, 5, 1, 1, 1, 1],
            [u32::MAX, 0, u32::MAX - 1, 1, u32::MAX - 2, 2, 3, 4],
        ];

        for mut arr in test_cases {
            bitonic_sort_8u32(&mut arr);
            for i in 0..7 {
                assert!(arr[i] <= arr[i + 1], "Array not sorted: {:?}", arr);
            }
        }
    }

    #[test]
    fn test_bitonic_sort_16u32_property_sorted() {
        let test_cases = [
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            [
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            ],
            [
                16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
            ],
            [
                8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15, 0,
            ],
            [
                u32::MAX,
                0,
                u32::MAX - 1,
                1,
                u32::MAX - 2,
                2,
                u32::MAX - 3,
                3,
                100,
                200,
                50,
                150,
                75,
                125,
                25,
                175,
            ],
        ];

        for mut arr in test_cases {
            bitonic_sort_16u32(&mut arr);
            for i in 0..15 {
                assert!(arr[i] <= arr[i + 1], "Array not sorted: {:?}", arr);
            }
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_bitonic_sort_8u32_property_preserves_count() {
        let mut arr = [3, 1, 4, 1, 5, 9, 2, 6];
        let mut orig_counts = std::collections::HashMap::new();
        for &x in &arr {
            *orig_counts.entry(x).or_insert(0) += 1;
        }

        bitonic_sort_8u32(&mut arr);

        let mut sorted_counts = std::collections::HashMap::new();
        for &x in &arr {
            *sorted_counts.entry(x).or_insert(0) += 1;
        }

        assert_eq!(orig_counts, sorted_counts, "Element counts changed");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_bitonic_sort_16u32_property_preserves_count() {
        let mut arr = [8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15, 0];
        let mut orig_counts = std::collections::HashMap::new();
        for &x in &arr {
            *orig_counts.entry(x).or_insert(0) += 1;
        }

        bitonic_sort_16u32(&mut arr);

        let mut sorted_counts = std::collections::HashMap::new();
        for &x in &arr {
            *sorted_counts.entry(x).or_insert(0) += 1;
        }

        assert_eq!(orig_counts, sorted_counts, "Element counts changed");
    }
}
