# Branchless Constant-Time Greater-Than (`>`) and Greater-Than-Or-Equal (`>=`) in `bcinr`

## Overview
In the `bcinr` (BranchlessCInRust) deterministic substrate, conditional control flow (`if`, `match`) is strictly forbidden by the "Radon Law" ($CC=1$). Consequently, relational operators like Greater-Than (`>`) and Greater-Than-Or-Equal (`>=`) must be evaluated as bitwise polynomials or compiler-verified mask selections. 

Depending on the security and performance context, the substrate employs two distinct mechanisms to evaluate these operators for fixed-point metrics (`Q16.16` represented as `i32` or `u32`) without control flow branches.

## 1. The Pure Algebraic Approach: Flagless Hardware Independence (`ct.rs`)
For strict, timing-attack-resistant cryptographic contexts, `bcinr` avoids even hardware condition flags (like `EFLAGS` on x86) by calculating relations using pure arithmetic and bitwise isolation (Hacker's Delight). 

Since `>` and `>=` are logical inverse/swaps of `<`, the substrate builds on a foundational less-than (`<`) primitive.

### Unsigned Evaluation (`u32`)
To evaluate `a > b`, the substrate evaluates `b < a` using the **borrow propagation trick**:
```rust
// From bcinr_logic::ct::ct_lt_u32
pub fn ct_lt_u32(a: u32, b: u32) -> u32 {
    // Isolates the borrow bit without comparison opcodes or hardware flags
    ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1
}
```
* **Greater-Than (`a > b`)**: Evaluated as `ct_lt_u32(b, a)`.
* **Greater-Than-Or-Equal (`a >= b`)**: Evaluated as `1 - ct_lt_u32(a, b)` or by bitwise flipping the extracted bit.

### Signed Fixed-Point Evaluation (`i64` / `i32`)
Fixed-point Q16.16 metrics often use signed integers. `bcinr` extracts the sign bit (`>> 63` or `>> 31`) of a wrapping subtraction to determine the relationship:
```rust
// From bcinr_logic::ct::ct_lt_i64
pub fn ct_lt_i64(a: i64, b: i64) -> u64 {
    let ua = a as u64; let ub = b as u64;
    let sign_a = ua >> 63;
    let sign_b = ub >> 63;
    let sign_diff = ua.wrapping_sub(ub) >> 63;
    
    // If signs differ: a < b iff a is negative
    // If signs match: a < b iff a - b is negative
    let signs_differ = sign_a ^ sign_b;
    ((signs_differ & sign_a) | ((!signs_differ) & sign_diff)) & 1
}
```
For fixed point logic requiring `a >= b`, the substrate simply selects `sign_diff` based on inverse rules or evaluates `1 - ct_lt_i64(a, b)`.

## 2. The Mask Calculus Approach: Compiler-Verified Intrinsic (`mask.rs` & `fix.rs`)
For high-performance algorithms outside of strict side-channel boundaries (e.g., autonomic metric bucketization and clamping), `bcinr` uses compiler-emitted hardware instructions that do not branch. 

Instead of providing an explicit `gt_mask_u32` function, `bcinr` leverages Rust's cast-to-integer behavior and two's-complement arithmetic to generate an all-ones (`0xFFFFFFFF`) or all-zeros (`0x00000000`) mask:

```rust
// Generating a Greater-Than mask (e.g. from fix.rs clamp_u32)
let gt_max = (res > max) as u32; 
let mask = 0u32.wrapping_sub(gt_max); // 0 or 0xFFFFFFFF
```

### How this respects $CC=1$ without branches:
1. `(res > max)` evaluates the comparison. 
2. `as u32` tells the compiler to map the boolean to `0` or `1`. On x86-64 architectures, the "Turing Machine" enforcer validates that this compiles directly to a `SETcc` (Set on Condition) instruction (like `SETB` or `SETG`), which uses standard hardware flags but **does not emit a jump instruction**.
3. `0u32.wrapping_sub(...)` uses a `NEG` instruction to smear the `1` across all 32 bits, creating `0xFFFFFFFF`.

### Fixed-Point Selection
Once the mask is generated, `bcinr` branchlessly applies it using the fundamental B-Calculus identity: `M(c, a, b) = (c & a) | (~c & b)`.
For example, to branchlessly cap a Q16.16 value at `max`:
```rust
// M(res > max, max, res)
res = (max & mask) | (res & !mask);
```

### Summary
To evaluate `>` and `>=` without control flow branches, `bcinr` relies on:
1. **Mask Calculus**: Exploiting `SETcc` + `NEG` intrinsic chains to create full-width bitmasks from hardware comparisons (when hardware flags are acceptable).
2. **Pure Arithmetic (CT)**: Using XOR, OR, wrapping subtraction, and sign-bit shifts (the borrow propagation trick) to evaluate inequalities mathematically without touching hardware condition flags or comparison instructions at all.
