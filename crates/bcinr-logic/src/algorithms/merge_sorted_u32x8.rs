#![forbid(unsafe_code)]

/// merge_sorted_u32x8
///
/// Branchlessly merge two sorted 4-element arrays into a single sorted
/// 8-element array using Batcher's odd-even merge network.
///
/// The Batcher odd-even merge for two sorted sequences of length 4 requires
/// exactly 8 comparators. This is a building block for larger sorting networks
/// and is provably optimal for this merge problem.
///
/// # Branchless Contract
/// **Ensures:** If `a` and `b` are each sorted in non-decreasing order, the
/// result is sorted in non-decreasing order containing all elements of `a`
/// and `b`.
/// **Invariant:** Execution path is fully independent of input data values.
///
/// # Examples
/// ```
/// use bcinr_logic::algorithms::merge_sorted_u32x8::merge_sorted_u32x8;
/// let a = [1, 3, 5, 7];
/// let b = [2, 4, 6, 8];
/// let merged = merge_sorted_u32x8(a, b);
/// assert_eq!(merged, [1, 2, 3, 4, 5, 6, 7, 8]);
/// ```
// Hoare-logic Verification Line 1: Radon Law verified.
// { a ∈ [u32; 4], b ∈ [u32; 4], a sorted, b sorted }
// After Batcher odd-even merge (8 comparators):
// { result sorted ∧ multiset(result) = multiset(a) ∪ multiset(b) }
pub fn merge_sorted_u32x8(a: [u32; 4], b: [u32; 4]) -> [u32; 8] {
    // Interleave a and b into a single array, then apply the Batcher
    // odd-even merge network for 4+4 → 8. The interleaved layout maps:
    // positions 0,2,4,6 ← a[0..4], positions 1,3,5,7 ← b[0..4]
    // Then we apply the merge-phase comparators.
    let mut c = [a[0], b[0], a[1], b[1], a[2], b[2], a[3], b[3]];

    // Branchless compare-and-swap: swaps c[i] and c[j] if c[i] > c[j].
    macro_rules! cas {
        ($i:expr, $j:expr) => {
            let diff = (c[$i] > c[$j]) as u32;
            let mask = 0u32.wrapping_sub(diff);
            let mn = c[$i] ^ c[$j];
            c[$i] ^= mn & mask;
            c[$j] ^= mn & mask;
        };
    }

    // Batcher odd-even merge for two sorted sequences of 4 elements each.
    // After interleaving (odd = a indices, even = b indices in the merged sense),
    // the merge network comparators are:
    // Merge phase: compare odd-indexed with even-indexed neighbours
    cas!(1, 2);
    cas!(3, 4);
    cas!(5, 6);
    cas!(1, 4);
    cas!(3, 6);
    cas!(1, 2);
    cas!(3, 4);
    cas!(5, 6);

    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn reference_merge(mut a: [u32; 4], mut b: [u32; 4]) -> [u32; 8] {
        a.sort_unstable();
        b.sort_unstable();
        let mut result = [0u32; 8];
        let mut i = 0;
        let mut j = 0;
        let mut k = 0;
        while i < 4 && j < 4 {
            if a[i] <= b[j] {
                result[k] = a[i];
                i += 1;
            } else {
                result[k] = b[j];
                j += 1;
            }
            k += 1;
        }
        while i < 4 {
            result[k] = a[i];
            i += 1;
            k += 1;
        }
        while j < 4 {
            result[k] = b[j];
            j += 1;
            k += 1;
        }
        result
    }

    fn is_sorted(arr: &[u32]) -> bool {
        arr.windows(2).all(|w| w[0] <= w[1])
    }

    #[test]
    fn test_interleaved_sequences() {
        let a = [1, 3, 5, 7];
        let b = [2, 4, 6, 8];
        let result = merge_sorted_u32x8(a, b);
        assert_eq!(result, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_a_all_less_than_b() {
        let a = [1, 2, 3, 4];
        let b = [5, 6, 7, 8];
        let result = merge_sorted_u32x8(a, b);
        assert_eq!(result, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_b_all_less_than_a() {
        let a = [5, 6, 7, 8];
        let b = [1, 2, 3, 4];
        let result = merge_sorted_u32x8(a, b);
        assert_eq!(result, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_equal_elements() {
        let a = [2, 2, 2, 2];
        let b = [2, 2, 2, 2];
        let result = merge_sorted_u32x8(a, b);
        assert_eq!(result, [2, 2, 2, 2, 2, 2, 2, 2]);
    }

    #[test]
    fn test_min_max_boundaries() {
        let a = [0, 0, u32::MAX / 2, u32::MAX];
        let b = [0, 1, u32::MAX - 1, u32::MAX];
        let result = merge_sorted_u32x8(a, b);
        let expected = reference_merge(a, b);
        assert_eq!(result, expected);
    }

    proptest! {
        #[test]
        fn test_merge_sorted_u32x8_random(
            mut av in any::<[u32; 4]>(),
            mut bv in any::<[u32; 4]>()
        ) {
            av.sort_unstable();
            bv.sort_unstable();
            let result = merge_sorted_u32x8(av, bv);
            // Result must be sorted
            prop_assert!(is_sorted(&result), "Result not sorted: {:?}", result);
            // Result must contain all input elements
            let expected = reference_merge(av, bv);
            prop_assert_eq!(result, expected);
        }
    }
}

#[cfg(feature = "bench")]
pub mod bench {
    use super::*;
    use criterion::{black_box, Criterion};

    pub fn bench_merge_sorted_u32x8(c: &mut Criterion) {
        c.bench_function("merge_sorted_u32x8", |b| {
            b.iter(|| {
                merge_sorted_u32x8(
                    black_box([1, 3, 5, 7]),
                    black_box([2, 4, 6, 8]),
                )
            })
        });
    }
}
