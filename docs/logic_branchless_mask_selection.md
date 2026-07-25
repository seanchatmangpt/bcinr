# Branchless Mask Selection in `bcinr`

Based on the `mask.rs` file located at `/Users/sac/bcinr/crates/bcinr-logic/src/mask.rs`, branchless selection is implemented via a rigorous "Mask Calculus" designed to adhere strictly to the project's constitutional Radon Law ($CC=1$). This ensures zero conditional branches (`if`/`else`/`match`) in the hot path, entirely avoiding CPU pipeline stalls from mispredicted branches. 

## 1. The Mask Convention
All branchless conditionals rely on generating and using bitmasks with an all-ones/all-zeros convention:
- **`0xFFFFFFFF` (all ones)** represents **true** (select `a`).
- **`0x00000000` (all zeros)** represents **false** (select `b`).

## 2. The Core `select` Primitives
The base primitive for branchless selection is `select_u32` (and its 64-bit counterpart `select_u64`). It computes the conditional using pure bitwise arithmetic without any CPU branches:

```rust
pub const fn select_u32(mask: u32, a: u32, b: u32) -> u32 {
    (mask & a) | (!mask & b)
}
```
- When `mask == 0xFFFFFFFF` (true): `(0xFFFFFFFF & a) | (0x00000000 & b) == a`
- When `mask == 0x00000000` (false): `(0x00000000 & a) | (0xFFFFFFFF & b) == b`

## 3. Branchless Mask Generation
To feed `select_u32`, masks must be generated mathematically rather than through control flow logic. The library provides several zero-branch primitives for conditions:

### Less-Than Comparison (`lt_mask_u32`)
```rust
pub const fn lt_mask_u32(a: u32, b: u32) -> u32 {
    0u32.wrapping_sub((a < b) as u32)
}
```
*How it works:* The comparison `(a < b)` yields `0` or `1`. Subtracting this from `0` creates `0x00000000` (if `0`) or wraps around to `0xFFFFFFFF` (if `1`). On x86-64, this compiles to a branchless `SETB` + `NEG` sequence.

### Equality (`eq_mask_u32`)
```rust
pub const fn eq_mask_u32(a: u32, b: u32) -> u32 {
    let x = a ^ b;
    // x | -x sets the sign bit if x is non-zero
    let non_zero_msb = (x | x.wrapping_neg()) >> 31;
    non_zero_msb.wrapping_sub(1)
}
```
*How it works:* 
1. `x = a ^ b` gives `0` only if `a == b`.
2. `x | -x` sets the most significant bit (MSB) for all non-zero values.
3. Shifting down by 31 isolates the MSB (`1` if `a != b`, `0` if `a == b`).
4. Subtracting 1 gives `0xFFFFFFFF` if `a == b`, and `0x00000000` otherwise.

### Zero / Non-zero Check
Similar bitwise properties of two's complement logic are used to generate branchless zero masks (`is_zero_mask_u32` and `nonzero_mask_u32`).

## 4. Higher-Level Implementations (e.g., `min` / `max` / `abs`)
By composing mask generation and selection, operations that typically branch can be written branchlessly:

**Min/Max:**
```rust
pub const fn min_u32(a: u32, b: u32) -> u32 {
    let mask = lt_mask_u32(a, b);
    select_u32(mask, a, b)
}
```

**Absolute Value:**
```rust
pub const fn abs_i32(x: i32) -> i32 {
    let mask = x >> 31;
    (x ^ mask).wrapping_sub(mask)
}
```
*How it works:* Arithmetic right shift by 31 replicates the sign bit across all 32 bits (creating `0xFFFFFFFF` for negatives and `0` for positives). XORing with the mask and subtracting it achieves branchless two's complement negation.
