# `bcinr-logic/src/ct.rs` Architecture & Constant-Time Primitives

The `ct.rs` module provides a comprehensive suite of constant-time arithmetic and logic primitives designed for side-channel resistance. It adheres strictly to the project's **Radon Law ($CC=1$)**, ensuring that logic is expressed purely as branchless bitwise polynomials without any `if`, `match`, or data-dependent `loop` constructs.

## Formal Invariant
For all valid inputs, the execution time is strictly $Θ(1)$ and is completely independent of the provided input values.

## Core Architectural Mechanisms

### 1. Mask Generation via `wrapping_sub`
The foundation of the branchless logic lies in transforming a binary condition (`0` or `1`) into a full-width bit mask. 
* **Mechanism**: `0_u32.wrapping_sub(condition & 1)`
* **Result**: If `condition == 1`, this underflows to all-ones (e.g., `0xFFFFFFFF`). If `condition == 0`, it remains all-zeros.

### 2. Conditional Selection (`ct_select_*`)
Instead of branching, selection operations (`ct_select_u8`, `ct_select_u32`, `ct_select_u64`, `ct_select_i64`) rely on the mask technique:
```rust
let mask = 0u64.wrapping_sub(condition & 1);
(a & mask) | (b & !mask)
```

### 3. Conditional Swapping (`ct_conditional_swap_u64`)
A branchless XOR-swap is used. A mask is generated based on the condition, which zeroes out the swap difference if the condition is `0`.
```rust
let mask = 0u64.wrapping_sub(condition & 1);
let diff = (*a ^ *b) & mask;
*a ^= diff;
*b ^= diff;
```

### 4. Equality Checks (`ct_eq_*`)
Equality relies on checking if `a ^ b == 0`. It checks if any bit is set by OR-ing the result with its wrapping negation.
```rust
let x = a ^ b;
// If x != 0, (x | -x) has its MSB set
let nonzero = (x | x.wrapping_neg()) >> 31;
1u32.wrapping_sub(nonzero)
```

### 5. Magnitude Comparisons (`ct_lt_*`)
* **Unsigned (`ct_lt_u32`)**: Uses the Hacker's Delight trick to isolate the borrow bit of `a - b` without a comparison opcode:
  `((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1`
* **Signed (`ct_lt_i64`)**: Extracts the sign bit of `a`, `b`, and `a.wrapping_sub(b)`. It does a constant-time selection between the signs based on whether the signs of `a` and `b` match or differ.

### 6. Math Functions (`ct_min`, `ct_max`, `ct_clamp`, `ct_abs`)
* **Min/Max**: Combines `ct_lt_u32` with masking to mathematically select the bounds: `b.wrapping_add(a.wrapping_sub(b) & mask)`.
* **Absolute Value (`ct_abs_i64`)**: Generates a mask from the sign bit (`x >> 63`). Performs two's complement negation conditionally: `(x ^ mask).wrapping_sub(mask)`.

### 7. Byte Slice Equality (`ct_byte_slice_eq`)
While it branches on length (as length is not considered a secret in this context), it processes *all* bytes sequentially using an iterator, accumulating any differences using bitwise OR (`diff |= a[i] ^ b[i]`). The final accumulated difference is checked using `ct_eq_u8`.

## Verification & Testing
The module embraces the **"Contract with Teeth"** philosophy:
* **Proptest Suite**: Exhaustive property-based testing ensures `ct_*` functions identically match Rust's native implementations (e.g., `<` or `wrapping_abs`).
* **Hostile Mutants**: The `tests_phd_ct` module defines counterfactual mutants to enforce that logic failures trigger immediate test suite assertions, ensuring 100/100 Substrate Integrity Score logic coverage.
