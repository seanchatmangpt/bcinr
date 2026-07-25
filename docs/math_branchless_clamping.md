# Branchless Numeric Clamping in BCINR

In accordance with Rule 14 (Numeric-law requirements) and the Radon Law ($CC=1$) from the BCINR constitutional manifesto, numeric clamping operations (`clip`, `min`, `max`) are executed entirely through branchless bitwise polynomials. This eliminates all pipeline-stalling conditional branches (`if`/`match`).

### 1. The Core Mask Calculus (`crates/bcinr-logic/src/mask.rs`)

The foundation of branchless execution is "Mask Calculus", which replaces booleans with a full-width all-ones/all-zeros mask convention (`0xFFFFFFFF` for true, `0x00000000` for false):

*   **Branchless Mask Generation:** 
    Conditions are evaluated directly into an arithmetic mask without jumps. For example, the less-than mask leverages a wrapping subtraction:
    ```rust
    pub const fn lt_mask_u32(a: u32, b: u32) -> u32 {
        // (a < b) produces 0 or 1. wrapping_sub yields 0x00000000 or 0xFFFFFFFF.
        // On x86-64, this compiles to a branchless SETB + NEG instruction pair.
        0u32.wrapping_sub((a < b) as u32)
    }
    ```

*   **Bitwise Polynomial Selection:** 
    Instead of conditionally branching, the values are selected using bitwise operations: `(mask & a) | (!mask & b)`.
    ```rust
    pub const fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
        (mask & a) | (!mask & b)
    }
    ```

### 2. `min` and `max` Implementation

Basic primitives like `min` and `max` unconditionally evaluate both inputs. They generate the comparison mask and pass it directly to the selection polynomial:

```rust
pub const fn min_u32(a: u32, b: u32) -> u32 {
    let mask = lt_mask_u32(a, b);
    select_u32(mask, a, b)
}

pub const fn max_u32(a: u32, b: u32) -> u32 {
    let mask = lt_mask_u32(a, b);
    select_u32(mask, b, a) // Inverts the selection order for max
}
```

### 3. Saturation and Clipping (`crates/bcinr-cmca/src/fixed.rs`)

In fixed-point arithmetic (`bcinr-cmca/src/fixed.rs`), clipping values to safe ranges (e.g., `saturating_mul` or `saturating_add`) utilizes `CanonicalMask`, which wraps the same bitwise selection principles:

```rust
pub const fn saturating_mul(self, other: Self) -> Self {
    let prod = (self.val as i64).wrapping_mul(other.val as i64);
    let res_i64 = prod >> 16;

    // 1. Generate full-width masks for overflow/underflow unconditionally
    let overflow_max = CanonicalMask(0u32.wrapping_sub((res_i64 > i32::MAX as i64) as u32));
    let overflow_min = CanonicalMask(0u32.wrapping_sub((res_i64 < i32::MIN as i64) as u32));

    // 2. Clamp minimum via masked select
    let mut res = overflow_min.select_i32(i32::MIN, res_i64 as i32);
    // 3. Clamp maximum via masked select
    res = overflow_max.select_i32(i32::MAX, res);

    // ... Unconditional fault aggregation (e.g., setting overflow flags bitwise) ...
}
```

### Rule 14 Compliance Verification:
*   **Fixed Bounded Execution:** All paths, whether in-bounds or requiring clamping, execute the exact same number of instructions.
*   **Constant-time Selection:** Limits like `i32::MIN` and `i32::MAX` are assigned directly via exact masks, rather than speculative or conditional assignment.
*   **Fault Preservation:** Even bounds-checking side-effects (`is_overflow`) are aggregated bitwise without short-circuiting logic (`overflow_max.raw() | overflow_min.raw()`), ensuring complete data-independence as per the project mandate.
