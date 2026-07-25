# Branchless Mathematical Absolute Value (`abs`) in BCINR

In the `bcinr` deterministic substrate, executing mathematical operations like absolute value (`abs`) for fixed-point integers without control flow (i.e., avoiding `if x < 0`) is mandatory under the **Radon Law (CC=1)**. This ensures constant-time execution, eliminates branch mispredictions, and maintains a strict bitwise invariant.

## The Bitwise Mathematics of Branchless `abs`

For a 32-bit signed integer (`i32`), the branchless absolute value relies on arithmetic right shifts (`>>`) and two's-complement arithmetic.

The standard bitwise implementation avoids the CPU's branch predictor entirely:

```rust
pub fn branchless_abs_i32(x: i32) -> i32 {
    // 1. Create a mask:
    // If x is positive or zero, `x >> 31` fills with 0s (0x00000000).
    // If x is negative, `x >> 31` sign-extends and fills with 1s (0xFFFFFFFF, which is -1).
    let mask = x >> 31;
    
    // 2. XOR and Subtract:
    // For positive x: (x ^ 0) - 0 = x
    // For negative x: (x ^ -1) - (-1)
    // XORing with -1 (all 1s) inverts all bits (one's complement).
    // Subtracting -1 is equivalent to adding 1, producing the two's complement.
    (x ^ mask).wrapping_sub(mask)
}
```

### Alternatively, using Addition:
Another mathematically equivalent formulation is:
```rust
pub fn branchless_abs_i32_add(x: i32) -> i32 {
    let mask = x >> 31;
    // For positive: (x + 0) ^ 0 = x
    // For negative: (x + (-1)) ^ (-1) = (x - 1) inverted bits = two's complement abs(x)
    (x.wrapping_add(mask)) ^ mask
}
```

## How it Appears in the `bcinr` Substrate

Throughout `bcinr`'s constant-time logic (`bcinr-logic` and `bcinr-cmca`), masking techniques (`x >> 31` or `x >> 63`) are heavily utilized to derive sign bits and build state masks dynamically without branches:
- **`branchless_signum_i64`**: Derives the signum using `let neg = (v >> 63) as u64;` and `let pos = (v.wrapping_neg() >> 63) as u64 & 1;` without any `match` or `cmp`.
- **`abs_diff_i64`**: The substrate directly delegates to `(val as i64).abs_diff(aux as i64)`, leveraging Rust's standard library which guarantees a branchless compilation down to the target ISA (e.g., expanding to the bitwise sequence above or mapping directly to single unbranched instructions).
- **Mask Derivation (`bcinr-logic/src/ct.rs`)**: You can often see the pattern `let nonzero = (x | x.wrapping_neg()) >> 31;` across the substrate to branchlessly derive whether a value is non-zero.

### Avoiding Panic Paths
Because `-i32::MIN` cannot be represented in a standard 32-bit signed integer (it overflows), production fixed-point operations inside `bcinr` must handle this explicitly. To uphold the **Absolute Runtime Laws (no panic paths)**, operations use `.wrapping_sub()` or `.wrapping_add()` rather than standard operators, or they widen the output to an unsigned integer (like `u32` or `u64` via `abs_diff`) to accommodate the extra magnitude value of `abs(i32::MIN)`.
