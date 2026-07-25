# Mask Calculus Architecture in `bcinr`

Based on the research into `crates/bcinr-logic/src/mask.rs`, here is the documentation of the fundamental mask abstraction and architecture within the `bcinr` repository.

## 1. Core Idea & The Radon Law

The `mask.rs` module provides branchless conditional selection and masking primitives to form the foundation of the B-Calculus framework. It strictly adheres to the project's **Radon Law** ($CC=1$), replacing CPU-pipeline-stalling branches (`if`, `match`) with pure, data-independent bitwise arithmetic polynomials.

## 2. Mask Convention

The library relies entirely on the **all-ones/all-zeros convention**:
*   **True** mask: `0xFFFF_FFFF` (all ones)
*   **False** mask: `0x0000_0000` (all zeros)

*Note: Intermediate mask values produce implementation-defined behavior.*

## 3. The Fundamental Mask Abstraction (`select`)

In formal B-Calculus notation, a mask operation is defined as `M(c, a, b) = (c & a) | (~c & b)`. This identity is mapped directly into code as the fundamental conditional selector:

```rust
#[inline(always)]
pub const fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    (mask & a) | (!mask & b)
}

#[inline(always)]
pub const fn select_u64(mask: u64, a: u64, b: u64) -> u64 {
    (mask & a) | (!mask & b)
}
```

This ensures `O(1)` performance with a perfectly predictable instruction count.

## 4. Mask Generation Functions

Because there are no branches, boolean comparisons must themselves be converted directly into the all-ones/all-zeros masks mathematically. `mask.rs` defines several primitives for this:

*   **`eq_mask_u32(a, b)`**: Uses XOR and two's-complement properties. It evaluates `x = a ^ b` and then calculates `(x | -x)` to shift the sign bit, collapsing non-zero patterns into a distinguishable bit to yield `0xFFFF_FFFF` only if `x == 0`.
*   **`lt_mask_u32(a, b)`**: Casts the comparison boolean to `u32` (0 or 1) and leverages `0u32.wrapping_sub(...)` to emit a branchless `SETB` + `NEG` sequence on x86-64 without any pipeline jumps.
*   **`is_zero_mask_u32(x)` / `nonzero_mask_u32(x)`**: Uses `(x | -x)` properties similarly to equality checks to branchlessly detect zero/non-zero values.

## 5. Higher-Level Composition

All higher-level primitives are expressed in terms of these generated masks and `select`:

*   **`min_u32` / `max_u32`**: Evaluates `lt_mask_u32(a, b)` and unconditionally passes both `a` and `b` to `select_u32` along with the mask.
*   **`abs_i32(x)`**: Extracts the sign bit via an arithmetic shift right (`x >> 31`) to produce `0xFFFF_FFFF` or `0`, followed by XOR and subtraction (`(x ^ mask).wrapping_sub(mask)`) to complete a branchless two's complement negation if negative.

## 6. Architectural Enforcements

At the end of the file, Hoare-logic verification lines and a suite of *counterfactual mutants* exist in the test module (`mutant_mask_1`, `mutant_mask_2`, etc.). These exist to test mathematical boundaries and ensure that any deviation from the bit-exact axiomatic contract fails the matrix, in compliance with `bcinr`'s deterministic constitution.
