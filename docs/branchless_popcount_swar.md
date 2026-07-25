# Research Report: Branchless SWAR Popcount in `bcinr`

I have explored the `bcinr` codebase to trace how Population Count / Hamming Weight primitives are currently implemented, and how a purely mathematical, hardware-agnostic SWAR (SIMD Within A Register) version fits within the substrate's laws.

## Current State in `bcinr`

Presently, `bcinr` primitives explicitly delegate to the standard library's `count_ones()` compiler intrinsic rather than manually unrolling a SWAR tree reduction. The following files act as wrappers around this intrinsic:
- `popcount_u64` (in `crates/bcinr-logic/src/int.rs`) computes `x.count_ones() as u64`
- `weight_u64` (in `crates/bcinr-logic/src/algorithms/weight_u64.rs`) computes `(val & aux).count_ones() as u64`
- `hamming_dist_simd` (in `crates/bcinr-logic/src/algorithms/hamming_dist_simd.rs`) computes `(val ^ aux).count_ones() as u64`

While this efficiently resolves to a hardware `POPCNT` instruction on modern architectures, it breaks the mathematical "pure software" constraint if you specifically want to avoid architecture-specific features and intrinsic opcodes entirely.

## The Pure SWAR Implementation ($CC=1$)

To implement this natively in `bcinr` using fixed-width polynomials (as mandated by `@von_neumann_bypass`), you must use the SWAR tree reduction method. This approach guarantees an identical instruction shape and constant bounded execution work across all hardware unconditionally, without relying on variable-latency loops.

Here is the exact mathematical SWAR specification formulated for the `bcinr` substrate:

```rust
/// Pure SWAR Population Count
///
/// Branchless Contract: Computes Hamming weight mathematically using SWAR tree 
/// reduction without relying on `POPCNT` hardware intrinsics.
///
/// Ensures: CC=1, 0-allocation, and zero data-dependent branches.
#[inline(always)]
#[must_use = "branchless popcount — ignoring discards computation"]
pub const fn popcount_swar_u64(mut x: u64) -> u64 {
    // Step 1: Count bits in 2-bit fields
    // Mask 0x5555555555555555 = 0b01010101...
    // Subtracting the high bit avoids a shift-and-mask-and-add step.
    x = x.wrapping_sub((x >> 1) & 0x5555555555555555);

    // Step 2: Sum adjacent 2-bit fields into 4-bit fields
    // Mask 0x3333333333333333 = 0b00110011...
    x = (x & 0x3333333333333333).wrapping_add((x >> 2) & 0x3333333333333333);

    // Step 3: Sum adjacent 4-bit fields into 8-bit fields
    // Mask 0x0F0F0F0F0F0F0F0F = 0b00001111...
    // We add first and mask after, since the maximum sum is 8 and won't overflow the nibble.
    x = (x.wrapping_add(x >> 4)) & 0x0F0F0F0F0F0F0F0F;

    // Step 4: Accumulate all byte fields via wrapping multiplication
    // Multiplying by 0x0101010101010101 sums all bytes into the highest byte.
    // A shift right by 56 isolates the final sum.
    (x.wrapping_mul(0x0101010101010101)) >> 56
}
```

## `bcinr` Constitutional Compliance

If this SWAR technique is swapped in for the current intrinsic-based implementations, it strictly honors the `bcinr` `AGENTS.md` mandate:
1. **The Radon Law ($CC=1$)**: Absolutely zero hidden `if`, `match`, or runtime loops.
2. **Object-Code Law**: Explicit use of `wrapping_sub`, `wrapping_add`, and `wrapping_mul` prevents Rust's debug assertions from injecting panic-paths (`panic_bounds_check`), keeping the object-code pure and fully unrolled.
3. **Von Neumann Protocol**: Converts control-flow sequential operations entirely into bit-parallel SWAR arithmetic, adhering perfectly to the rule of "Bit-parallel mechanics over byte-sequential control flow."
