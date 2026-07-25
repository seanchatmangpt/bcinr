# Architectural Analysis of `bcinr-logic/src/bitset.rs`

The `bitset` module within `crates/bcinr-logic` provides a collection of performance-critical, branchless implementations of bitset algebra algorithms (rank, select, set, clear, parity, etc.). Adhering strictly to the project's **Radon Law ($CC=1$)**, it substitutes sequential control flow with bitwise polynomials and masked state selection.

## 1. Fixed-Size Bitsets Representation
Bitsets are implemented either as singular fixed-size `u64` words or dynamically as `&[u64]` slices. Memory allocations and bounded iterations are strictly avoided in favor of deterministic loop limits and branchless reductions.

When performing binary operations on slices of different sizes (e.g., in `intersect_u64_slices` or `hamming_u64_slices`), the module eschews standard boundary checks. Instead, it computes the loop upper bound using a branchless minimum calculation:
```rust
let min_len = (len_a & (0usize.wrapping_sub((len_a < len_b) as usize)))
    | (len_b & (0usize.wrapping_sub((len_a >= len_b) as usize)));
```

## 2. Masked Selection and Branchless Techniques
The module systematically replaces conditional logic (`if`/`else`) with bitwise masks to guarantee absolute determinism and constant-time execution paths.

### Boolean-to-Mask Conversion
Predicates are evaluated and cast to integers, then transformed into full-width masks (all 0s or all 1s) using `wrapping_sub`.
```rust
// Examples from the codebase:
let go_high_mask = 0usize.wrapping_sub((low_count < count) as usize);
```

### Branchless Binary Search (`select_bit_u64`)
The most complex primitive is finding the index of the n-th set bit. Rather than iterating, it uses a bounded, unrolled loop `(0..6).rev().for_each(...)` executing exactly 6 steps for a 64-bit word. At each step, it calculates a mask (`go_high_mask`) to dictate whether the search space shifts up or stays low:
```rust
let go_high_mask = 0usize.wrapping_sub((low_count < count) as usize);
res += step & go_high_mask;
x_copy >>= step & go_high_mask;
count -= low_count & go_high_mask;
```
This selectively updates the state based purely on arithmetic without branching.

### Branchless Division by Zero Prevention
In `jaccard_u64_slices`, mathematical safety is maintained branchlessly when calculating `intersection / union`:
```rust
(intersection as f32) / (union as f32 + (union == 0) as u32 as f32)
```
If `union` is 0, this adds 1 to the denominator to prevent division by zero, resulting in `0.0 / 1.0 = 0.0`.

### Data-Driven Returning
Where an `Option` needs to be returned depending on a condition, the condition is evaluated as `0` or `1`, which serves as an index into a fixed size array containing the two possible outcomes:
```rust
let exists = (res < 64 && count == 1 && ((x_copy & 1) != 0)) as usize;
[None, Some(res)][exists]
```
