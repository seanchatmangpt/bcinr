# Branchless Absolute Value Mechanics in `bcinr`

In accordance with the **Radon Law ($CC=1$)** and the deterministic mandate of `bcinr`, computing the absolute value of a signed integer cannot rely on conditional control flow (e.g., `if val < 0 { -val }`). Instead, it must be formulated as a bitwise polynomial operating in constant time. 

This document outlines the authoritative implementation mechanics for a branchless absolute value, specifically utilizing sign-extension masks, XOR, and two's complement arithmetic.

## The Mathematical Mechanism

To conditionally negate a value without branching, we manipulate the binary representation directly using the sign bit. For a 64-bit signed integer (`i64`), the procedure relies on three steps: Masking, Bitwise Inversion, and Two's Complement completion.

### 1. The Sign Extension Mask
In Rust, the arithmetic right shift operator (`>>`) on a signed integer replicates the sign bit (the most significant bit). 

```rust
let mask = val >> 63;
```
* If `val >= 0`: The sign bit is `0`. The arithmetic shift fills the word with `0`s, yielding `0x0000000000000000` (which is `0` in decimal).
* If `val < 0`: The sign bit is `1`. The shift fills the word with `1`s, yielding `0xFFFFFFFFFFFFFFFF` (which is `-1` in two's complement).

This cleanly satisfies the **Mask-Based Execution Law** where masks must be strictly `0` or `2^w - 1`.

### 2. XOR (Conditional Inversion)
The XOR operator (`^`) interacts with our mask to conditionally invert the bits:
* `val ^ 0 = val`
* `val ^ -1 = ~val` (bitwise NOT of `val`)

### 3. The Two's Complement Negation
In two's complement arithmetic, the negative of a number is its bitwise NOT plus one (`-val = ~val + 1`). 
Since subtracting `-1` is mathematically equivalent to adding `1`, we can use `wrapping_sub(mask)` to finalize the negation:

```rust
(val ^ mask).wrapping_sub(mask)
```
*(Alternatively, `(val.wrapping_add(mask)) ^ mask` is identically valid and mathematically equivalent.)*

## The Authoritative Implementation

Complying with `bcinr`'s numeric law requirements—which mandate an explicit wrapping or saturating contract for edge cases—the implementation must explicitly declare its arithmetic boundaries.

```rust
/// Computes the absolute value of an i64 branchlessly.
/// 
/// # Contract
/// - **CC=1**: Strictly branchless and data-independent.
/// - **Wrapping Law**: If `val == i64::MIN`, the true mathematical absolute value 
///   cannot be represented in a signed 64-bit integer. Following the wrapping 
///   contract, it will evaluate to `i64::MIN`.
#[inline(always)]
pub const fn abs_branchless_i64(val: i64) -> i64 {
    let mask = val >> 63;
    (val ^ mask).wrapping_sub(mask)
}
```

## Execution Traces

### Case A: Positive Value (e.g., `val = 5`)
1. `mask = 5 >> 63 = 0`
2. `val ^ mask = 5 ^ 0 = 5`
3. `5.wrapping_sub(0) = 5`
* **Result**: `5`

### Case B: Negative Value (e.g., `val = -5`)
1. `mask = -5 >> 63 = -1`
2. `val ^ mask = -5 ^ -1 = ~(-5)`
3. `~(-5).wrapping_sub(-1) = ~(-5) + 1 = 5`
* **Result**: `5`

### Case C: The Boundary Condition (`val = i64::MIN`)
1. `mask = i64::MIN >> 63 = -1`
2. `val ^ mask = i64::MIN ^ -1 = i64::MAX`
3. `i64::MAX.wrapping_sub(-1) = i64::MAX + 1 = i64::MIN`
* **Result**: `i64::MIN` (Adheres to the exact explicit wrapping contract).

## Conclusion
This formulation successfully reduces a semantic conditional (`if`) into a fixed-width, zero-allocation, bounded execution unit, strictly adhering to the `$CC=1$` mandate required for authoritative inclusion in the `bcinr` runtime.
