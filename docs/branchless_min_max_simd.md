# Branchless SIMD Min/Max Operations in BCINR

In the `bcinr` deterministic substrate, executing instructions across packed vectors without hardware-specific SIMD intrinsics (such as `_mm_min_epi8`) requires strict adherence to the **Radon Law ($CC=1$)**. Data-dependent conditional jumps, such as `if a < b { a } else { b }`, are completely forbidden. 

When hardware vector instructions are not admitted—either to maintain strict `no_std` portability across architectures or to ensure a fixed deterministic hot-path—`bcinr` utilizes **SWAR (SIMD Within A Register)** and constant-time mask calculus to evaluate Minimum and Maximum operations branchlessly over packed data.

## 1. Portable 128-bit SIMD via Scalar Arrays

Instead of using non-portable target-feature gates (like `#[cfg(target_feature = "sse4.2")]`) which create multiple code paths, `bcinr` abstracts 128-bit vectors as simple scalar arrays: `[u8; 16]`. 

Operations iterate over lanes using deterministic `for_each` loops. Modern Rust compilers safely auto-vectorize these structures into actual SIMD (like `PSHUFB` or `VPCMPEQB`) without unsafe blocks, while guaranteeing fallback to safe, branchless scalar execution on unsupported targets.

## 2. Generating the "Less-Than" Mask

To find the minimum or maximum of two lanes without branching, the system first generates a bitwise condition. For packed unsigned integers, `bcinr` uses the Hacker's Delight borrow bit trick. The bit-parallel logic isolates the borrow bit during subtraction:

```rust
// Constant-time less-than for a 32-bit lane
pub fn ct_lt_u32(a: u32, b: u32) -> u32 {
    // Isolates the borrow bit for unsigned subtraction
    ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1
}
```
This safely outputs a `1` if `a < b` and `0` otherwise, purely via arithmetic operators.

## 3. Bitwise Mask Propagation

The critical step in avoiding control flow is transforming the `0` or `1` boolean condition into a **full-width selection mask**. This is achieved by utilizing two's complement wrapping subtraction:

```rust
// Converts 1 -> 0xFFFF_FFFF, and 0 -> 0x0000_0000
let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
```

This canonical mask is propagated across the width of the lane (e.g., 32 bits).

## 4. B-Calculus Multiplexing for Min and Max

With the full-width mask established, the minimum and maximum are evaluated using branchless arithmetic multiplexing—the **B-Calculus Selection Identity**: `M(mask, a, b) = (mask & a) | (!mask & b)`.

Alternatively, delta-based arithmetic selection can be used:

```rust
// Branchless ct_min
pub fn ct_min_u32(a: u32, b: u32) -> u32 {
    let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
    b.wrapping_add(a.wrapping_sub(b) & mask)
}

// Branchless ct_max
pub fn ct_max_u32(a: u32, b: u32) -> u32 {
    let mask = 0u32.wrapping_sub(ct_lt_u32(a, b));
    a.wrapping_add(b.wrapping_sub(a) & mask)
}
```
- **If `a < b`**: `mask` is all-ones (`0xFFFFFFFF`). The bitwise AND extracts the difference, resulting in $b + (a - b) = a$ for the minimum.
- **If `a >= b`**: `mask` is all-zeros (`0x00000000`). The bitwise AND wipes out the difference, resulting in $b + 0 = b$ for the minimum.

## 5. Implementation in Sorting Networks

This lane-by-lane branchless min/max logic serves as the foundation for the repository's constant-time SIMD algorithms, such as `sort_pairs_u32x4` or `odd_even_merge_sort_16u32`. When data is packed into a 64-bit integer (`u64` containing four `u16` lanes), the register is unpacked mathematically into scalar components, the branchless `min` and `max` operations run unconditionally on the paired lanes, and the result is bitshifted back into a unified `u64`. 

By strictly applying masked state selection to every component of packed vector evaluation, `bcinr` entirely eliminates CPU branch predictors from the hot path and complies mathematically with the stringent `bcinr` constitutional mandates.
