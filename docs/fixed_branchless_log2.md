Based on the `grep_search` and subsequent `view_file` calls in `crates/bcinr-cmca/src/fixed.rs` and `crates/bcinr-logic/src/`, I have found the branchless implementations of `log2`. 

The most mathematically advanced implementation—featuring exponent isolation, mantissa extraction, and a parabolic fractional correction polynomial—resides in `crates/bcinr-cmca/src/fixed.rs` as the `log2` method of `NonNegativeFixed`.

Here is the documentation detailing how it calculates the fixed-point logarithm branchlessly.

### 1. Isolating the Integer Exponent
To find the base-2 integer logarithm branchlessly (which corresponds to finding the most significant bit), the function uses hardware intrinsics to count leading zeros (`lz`). It also produces a deterministic nonzero mask `nz`:
```rust
let x = self.val as u64;
let lz = x.leading_zeros();
let nz = ((x | x.wrapping_neg()) >> 63) & 1; // 1 iff val != 0
let ip = 63u64.wrapping_sub(lz as u64) & nz.wrapping_neg();
```
- The `nz` bit acts as a branchless boolean (1 if `x != 0`, 0 if `x == 0`).
- The integer part `ip` computes `63 - lz`, effectively identifying `floor(log2(x))`. The `& nz.wrapping_neg()` forces the integer part to 0 without jumping if the input was 0.

### 2. Extracting the Linear Mantissa (Fractional Bits)
Next, the algorithm isolates the bits trailing the most significant bit to form the fractional base of the logarithm estimate:
```rust
let mantissa = x.wrapping_shl(lz.wrapping_add(1));
let f = (mantissa >> (64 - 16)) as u32;
```
- It shifts out all leading zeros *and* the implicit leading 1 bit by shifting left by `lz + 1`.
- It then shifts right by `64 - 16` to take exactly the top 16 bits of the mantissa, providing a `Q0.16` linear interpolation factor `f`.

### 3. Parabolic (Polynomial) Fraction Correction
A pure linear fractional mantissa provides a piecewise linear approximation of the logarithm curve, but $\log_2(1+f)$ is a curve, not a line. The algorithm applies a deterministic, branchless polynomial correction:
```rust
let diff = 65536 - f;
let correction = (f * diff) >> 16;
let corrected_frac = f + ((correction * 29013) >> 16);
```
- `diff` calculates $(1 - f)$ within `Q0.16`.
- `correction` calculates $f \times (1 - f)$, which serves as a parabolic bounding curve.
- The `corrected_frac` uses an empirically optimal, statically-admitted scaling constant (`29013`) to adjust the linear term $f$. This effectively computes $f + c \cdot f(1 - f)$ to approximate the curvature of $\log_2(1+f)$ while avoiding branching, division, or floating-point usage.

### 4. Q16.16 Domain Re-Centering
Because the input `val` is intrinsically a `Q16.16` fixed-point value, computing its logarithm means computing $\log_2(\text{val} \cdot 2^{-16})$. The mathematical law $\log_2(a \cdot b) = \log_2(a) + \log_2(b)$ applies:
```rust
let res = (ip << 16).wrapping_add(corrected_frac as u64);
let computed = (res as u32).wrapping_sub(16 << 16) as i32;
```
- The integer part `ip` and `corrected_frac` are unified.
- The term `- 16 << 16` subtracts 16 (in `Q16.16` format) to re-center the log value for the fractional scale.

### 5. Branchless Refusal & Fault Masking
In accordance with the `bcinr` architectural laws, invalid domains like `0` must not panic, branch, or dynamically dispatch. Instead, they produce a bounded typed refusal via a `CanonicalMask`:
```rust
let is_zero = const_eq_u32(self.val, 0);
let e = CanonicalMask::select_faults(
    is_zero,
    NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
    NumericFaultSet::EMPTY,
);
SignedFixed {
    val: is_zero.select_i32(-1048576, computed), // -1048576 is -16 in Q16.16
    faults: self.faults.union(e),
}
```
- `const_eq_u32` evaluates equality branchlessly into an all-ones (`u32::MAX`) or all-zeros mask.
- A zero input selects `-16` (or `-1048576` raw) as a fallback clamping behavior, whilst deterministically bitwise-ORing the `INVALID_DOMAIN` and `DIVIDE_BY_ZERO` fault flags into the computation’s persistent fault state.

*(Note: Additional, simpler log algorithms exist in `crates/bcinr-logic/src/algorithms/fixed_point_log2.rs` and `log2_u64_fixed.rs`, but they rely merely on simple piecewise linear estimations without the `f * (1 - f)` polynomial curvature adjustments found in the `NonNegativeFixed` CMCA hot path).*
