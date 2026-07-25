# Fixed-Point Scaling Factor Rule in `bcinr` (Q16.16)

## Why Q16.16 is Mandated

The `bcinr` (BranchlessCInRust) project enforces strict constitutional laws to guarantee deterministic, bounded, branchless, and allocation-free execution. According to `AGENTS.md`, all floating-point operations (`f32`/`f64`) are explicitly prohibited. Floating-point math introduces architecture-dependent rounding, non-deterministic `NaN` or `Infinity` states, and often relies on hardware or library implementations that contain hidden control flow, violating the absolute $CC=1$ (Cyclomatic Complexity = 1) rule.

To support real-number calculations without floats, `bcinr` mandates the **Q16.16 fixed-point format**, wrapping 32-bit integers (`u32` and `i32`) into types like `NonNegativeFixed` and `SignedFixed`. 

The Q16.16 format partitions the 32 bits evenly:
- **16 bits for the integer part:** Provides a safe dynamic range (`[-32768, 32767]` for signed).
- **16 bits for the fractional part:** Provides a resolution of $1/65536 \approx 0.0000153$.

This structure allows complex real-number approximations (like square roots, logarithms, and trigonometry) to be implemented via pure-integer SWAR (SIMD Within A Register) operations and manual Newton-Raphson unrolling, fully retaining branchless execution.

## Preserving Fractional Scale Bounds with `wrapping_mul` and `>> 16`

Mathematical operations on fixed-point numbers inherently change the scale factor. In Q16.16, a number $v$ is represented in integer form as $v \times 2^{16}$.

If you multiply two Q16.16 numbers $A$ and $B$, their raw integer representations multiply as well:
$$ (A \times 2^{16}) \times (B \times 2^{16}) = (A \times B) \times 2^{32} $$

This results in a Q32.32 scaled number, which is out of bounds for the Q16.16 format.

To restore the bounds safely and branchlessly, `bcinr` leverages a combination of 64-bit intermediate casting, `wrapping_mul`, and a `>> 16` bitshift:

1. **64-bit Intermediate Casting**: Operands are first cast to `u64` or `i64`. This prevents data loss during the intermediate step, as the Q32.32 product exceeds 32 bits.
2. **`wrapping_mul`**: The standard `*` operator in Rust can panic in debug mode on overflow, and `checked_mul` returns an `Option` that would force an unlawful `match` or `if` branch. `wrapping_mul` provides safe, deterministic hardware-level wrapping in constant time.
3. **The Downshift (`>> 16`)**: The 64-bit product is shifted right by 16 bits. This performs a fast, branchless integer division by $2^{16}$, mathematically restoring the target Q16.16 scale bounds:
   $$ \frac{(A \times B) \times 2^{32}}{2^{16}} = (A \times B) \times 2^{16} $$
4. **Branchless Saturation & Fault Accumulation**: The shifted 64-bit result is checked for bounds violations (whether it exceeds `i32::MAX`/`i32::MIN` or `u32::MAX`). Instead of using `if`, bitwise logic constructs a `CanonicalMask`. If an overflow occurs, this mask selectively clamps the output to `MAX` or `MIN` and unions an `OVERFLOW` and `SATURATION` flag into the `NumericFaultSet`—all executing in exactly the same number of clock cycles regardless of whether a fault occurred.

**Example from `SignedFixed::saturating_mul`**:
```rust
pub const fn saturating_mul(self, other: Self) -> Self {
    // 1 & 2. 64-bit intermediate and wrapping_mul
    let prod = (self.val as i64).wrapping_mul(other.val as i64);
    
    // 3. Downshift by 16 to restore Q16.16 scale
    let res_i64 = prod >> 16;

    // 4. Branchless overflow checks and mask-based saturation
    let overflow_max = CanonicalMask(0u32.wrapping_sub((res_i64 > i32::MAX as i64) as u32));
    let overflow_min = CanonicalMask(0u32.wrapping_sub((res_i64 < i32::MIN as i64) as u32));
    
    let mut res = overflow_min.select_i32(i32::MIN, res_i64 as i32);
    res = overflow_max.select_i32(i32::MAX, res);
    
    // ... accumulates NumericFaultSet bitwise
}
```
