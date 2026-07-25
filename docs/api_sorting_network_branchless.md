# Branchless Sorting in BCINR

The system uses branchless bitonic sorting networks to sort arrays with deterministic latency, ensuring safety from timing side-channels and strictly adhering to the architectural Radon Law ($CC=1$). 

The entire sorting process is built upon two conceptual layers:

## 1. The Constant-Time Compare-Exchange Kernel

The fundamental primitive for all sorting networks in the project is the `compare_exchange` function. Traditional sorting algorithms rely on conditional branches (e.g., `if a[i] > a[j] { swap(a, i, j) }`), which create timing variations depending on the input data permutation.

To prevent this, conditionals are replaced by the $\mathcal{B}$-Calculus framework using bitwise arithmetic:

```rust
#[inline(always)]
pub fn compare_exchange(a: &mut [u32], i: usize, j: usize) {
    let mask = (a[i] > a[j]) as u32;
    let diff = (a[i] ^ a[j]) & 0u32.wrapping_sub(mask);
    a[i] ^= diff;
    a[j] ^= diff;
}
```

### How it works:
1. **Condition Evaluation:** `(a[i] > a[j]) as u32` evaluates to `1` if the elements need to be swapped, and `0` otherwise.
2. **Mask Generation:** `0u32.wrapping_sub(mask)` translates the boolean integer to a full-width bitmask. If `mask` is `1`, it underflows to `0xFFFFFFFF` (all ones). If `mask` is `0`, it evaluates to `0x00000000` (all zeros).
3. **Difference Calculation:** `diff` computes the XOR difference between the two values, then masks it. If they shouldn't be swapped (mask is `0`), `diff` becomes `0`.
4. **Branchless Swap:** Both elements are XOR'd with `diff`. If `diff` is `0`, the values remain identical. If `diff` holds the XOR difference, the bitwise XOR effectively swaps `a[i]` and `a[j]`. 

## 2. Bitonic Sorting Networks

With a constant-time `compare_exchange`, entire arrays are sorted using static compare-exchange networks that execute the same instructions regardless of input structure. For instance, the `bitonic_sort_8u32` and `bitonic_sort_16u32` functions process 8 and 16 elements respectively.

```rust
#[inline]
pub fn bitonic_sort_8u32(a: &mut [u32; 8]) {
    (0..3).for_each(|i| {
        let step = 1 << i;
        (0..step).for_each(|j| {
            (0..8).step_by(step * 2).for_each(|k| {
                compare_exchange(a, k + j, k + step * 2 - 1 - j);
            });
        });
        (0..i).rev().for_each(|j| {
            let step_inner = 1 << j;
            (0..8).step_by(step_inner * 2).for_each(|k| {
                (0..step_inner).for_each(|l| {
                    compare_exchange(a, k + l, k + l + step_inner);
                });
            });
        });
    });
}
```

### Execution Properties:
- **O(N log² N) Static Comparisons:** An 8-element sort processes exactly 24 compare-exchange operations across $\log_2(8) = 3$ stages. 
- **Fully Unrolled Boundaries:** The iterators (`for_each`) and steps are deterministic, iterating over boundaries known purely at compile-time. They map directly into a static sequence of branchless instructions (straight-line code).
- **Absolute Determinism:** No data-dependent loops or branches exist anywhere in the call graph. Processing untrusted external data always consumes the exact same number of CPU cycles.
