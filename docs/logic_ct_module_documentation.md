Here is the documentation for the `ct.rs` (Constant Time) module within the `bcinr-logic` crate.

# `bcinr-logic::ct` Module Documentation

## Core Purpose
The `ct.rs` module provides fundamental constant-time mathematical and logical operations. Its primary goal is to provide building blocks for side-channel resistant code, ensuring that the execution time of comparisons, selections, and math is strictly $\Theta(1)$ and entirely independent of the values being processed. 

This directly upholds the project's **Radon Law ($CC=1$)**, substituting all data-dependent branches (`if`, `match`, etc.) with equivalent arithmetic and bitwise polynomials. 

## Implementation Details & Primitives

The module employs several low-level branchless techniques to convert Boolean state into full-width bit masks.

### 1. Constant-Time Selection (`ct_select_u8`, `ct_select_u32`, `ct_select_u64`, `ct_select_i64`)
Selects between value `a` and value `b` based on a 0 or 1 condition.
* **Mechanism:** Converts the condition (0 or 1) into a mask of all 0s or all 1s via unsigned wrapping subtraction: `0u*.wrapping_sub(condition & 1)`. 
* The result is derived safely using bitwise logic: `(a & mask) | (b & !mask)`.

### 2. Equality Checks (`ct_eq_u8`, `ct_eq_u32`, `ct_eq_u64`)
Determines if two integers are equal without branching.
* **Mechanism:** Computes the bitwise difference `x = a ^ b`. Using two's complement math `x | x.wrapping_neg()`, it guarantees that the Most Significant Bit (MSB) is 1 if *any* bit differs. The result is right-shifted by the integer width minus 1, isolating a 1 if different, which is then subtracted from 1.

### 3. Less-Than Comparison (`ct_lt_u32`, `ct_lt_i64`)
Evaluates `a < b` returning 1 if true, 0 if false.
* **Mechanism:** For unsigned integers, it utilizes Hacker's Delight borrow trick: `((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1` which flawlessly isolates the borrow propagation without opcodes that leak timing data. For signed variables, it accounts for sign bits before selecting the correct comparison path based on whether the signs match or differ.

### 4. Slice Comparison (`ct_byte_slice_eq`)
Checks byte slices for equality in constant time (assuming the lengths are identical, as length is not treated as a secret).
* **Mechanism:** It loops sequentially without any short-circuiting (`return` or `break` on mismatch). Differences are accumulated into a `diff` accumulator using `diff |= a[i] ^ b[i]`. The final outcome is computed via `ct_eq_u8(diff, 0)`.

### 5. Utility Math (`ct_abs_i64`, `ct_min_u32`, `ct_max_u32`, `ct_clamp_u32`)
Executes typical standard library math methods branchlessly.
* **Absolute Value:** Arithmetic right shift (`x >> 63`) propagates the sign bit. The value is XORed with this mask and the mask is subtracted, forming a valid two's complement negation without branches.
* **Min/Max/Clamp:** Calculates the delta and conditionally applies it using a bitmask constructed via `ct_lt_u32`. 

## Verification & Testing
Adhering to the BCINR repository constitution, `ct.rs` uses axiomatic proof structuring:
* **Hoare-logic comments** tracking valid state boundaries.
* Extensive **proptest suite** validating behavior equivalence against standard `std` primitives (e.g., `x.wrapping_abs()`).
* **Hostile Mutation Tests** (`tests_phd_ct`) containing counterfactual mutants to enforce the rigor of the internal verifications.
