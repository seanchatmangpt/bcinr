# BCINR Rule 14: Numeric-law requirements & Branchless Approximation Architecture

The `bcinr` substrate strictly enforces a mathematical contract (Rule 14 and the "Radon Law" $CC=1$) where no dynamic control flow (`if`, `match`, data-dependent loops) is permitted in the hot path. All numeric operations, including fixed-point approximations and division replacements, must guarantee no NaN/infinity and strictly enforce error envelopes and fault tracking via mask-based execution.

Here is an architectural breakdown of how branchless fixed-point approximations (division, reciprocal, exponential, logarithm) are implemented across `bcinr-logic` and `bcinr-cmca`.

## 1. Division Replacements & Zero-Guarding

In branchless programming, division by zero cannot be avoided by early returns or panics. Instead, zero divisors must be gracefully replaced with a safe scalar (typically `1`) via bitwise masking, while the fault is tracked.

**`q16_div` (`bcinr-logic/src/fix.rs`)**
```rust
let safe_b = b | ((b == 0) as i32);
((a as i64 * (1 << 16)) / safe_b as i64) as i32
```
* **Mechanism**: If `b == 0`, `(b == 0) as i32` evaluates to `1`. `0 | 1` yields `1`. If `b != 0`, `(b == 0) as i32` evaluates to `0`. `b | 0` yields `b`. This replaces a zero denominator with `1` perfectly without branching, averting hardware traps.

**`NonNegativeFixed::saturating_div` (`bcinr-cmca/src/fixed.rs`)**
```rust
let den_is_zero = const_eq_u32(other.val, 0); // CanonicalMask
let d = den_is_zero.select_u32(1, other.val);
// ... Newton-Raphson approximation ...
let saturate = CanonicalMask(overflow.raw() | den_is_zero.raw());
```
* **Mechanism**: Evaluates the division using a Newton-Raphson-like polynomial sequence. If the denominator is zero, the returned output is saturated to `u32::MAX` branchlessly using `saturate.select_u32(u32::MAX, q_corrected as u32)`.

## 2. Reciprocal (`q16_recip`)

Instead of standard division, reciprocal is computed using Newton-Raphson iterations to refine a bit-shifted initial estimate.

**Implementation (`bcinr-logic/src/fix.rs`)**
```rust
let is_zero = (x == 0) as i32; // 0 or 1
let zero_mask = 0i32.wrapping_sub(is_zero); // 0 or 0xFFFFFFFF
let safe_x = (x & !zero_mask) | (1i32 & zero_mask); // replace 0 with 1
// ... 2 NR iterations ...
(r2 & !zero_mask) | (i32::MAX & zero_mask)
```
* **Mechanism**: Generates an all-ones mask `0xFFFFFFFF` if `x` is `0`. `safe_x` guarantees an un-trapped execution of the Newton-Raphson sequence. At the very end, the result is masked to return `i32::MAX` natively for a `0` input, ensuring constant-time saturation.

## 3. Logarithm

Logarithm operations rely on `leading_zeros` to extract the integer exponent and interpolate the fractional mantissa.

**`fixed_point_log2` (`bcinr-logic/src/algorithms/fixed_point_log2.rs`)**
```rust
let lz = val.leading_zeros(); // 64 when val == 0
let nz = ((val | val.wrapping_neg()) >> 63) & 1; // 1 iff val != 0
let ip = 63u64.wrapping_sub(lz as u64) & nz.wrapping_neg(); // 0 when val == 0
```
* **Mechanism**: If `val == 0`, `lz = 64`. `nz` isolates whether the value is non-zero without branches. The integer part (`ip`) naturally collapses to `0` when `val == 0` because `nz.wrapping_neg()` yields `0x00...00`.

**`NonNegativeFixed::log2` (`bcinr-cmca/src/fixed.rs`)**
* **Mechanism**: Generates an interpolation polynomial for the mantissa. If the input is zero, the result is masked entirely using `is_zero.select_i32(-1048576, computed)` (outputting a saturated negative integer equivalent to negative infinity in Q16 bounds).

## 4. Exponential (`exp2`, `exp`)

Exponential functions convert the input into an integer shift and a fractional polynomial approximation. Large exponents must saturate to maximum integer bounds without conditional checks.

**`exp2_u64_fixed` (`bcinr-logic/src/algorithms/exp2_u64_fixed.rs`)**
```rust
let saturated = (int_exp >= 48) as u64;
let sat_mask = saturated.wrapping_neg(); // all-ones if saturated, zero otherwise
let safe_exp = int_exp & 63; // Prevent undefined shift behavior
let result = 65536u64.wrapping_shl(safe_exp);
(result & !sat_mask) | sat_mask // returns u64::MAX if sat_mask is all-ones
```
* **Mechanism**: Safely clamps the shift operator using `& 63` to prevent language panics, computes the theoretical result, and overrides it entirely to `u64::MAX` if `int_exp` crossed the valid threshold.

**`SignedFixed::exp2` (`bcinr-cmca/src/fixed.rs`)**
* **Mechanism**: Determines underflow (`is_underflow`) or overflow (`is_overflow`) by evaluating the sign bit of `ip`. The result uses chained masking:
```rust
is_overflow.select_u32(u32::MAX, is_underflow.select_u32(0, val_shifted))
```
This resolves saturation bounds purely via logical AND/OR selection operations.

## 5. Architectural Safety: Fault Sets and `CanonicalMask`

The foundation of Rule 14 implementation in `bcinr` relies on the `CanonicalMask` and `NumericFaultSet` primitives.

* **`CanonicalMask`**: A struct wrapping a `u32` guaranteed to be exactly `0` or `u32::MAX`. All comparisons (e.g., `const_lt_u32`) map natively to this type. Mask selection `(a & mask) | (b & !mask)` acts as the physical replacement for `if/else`.
* **`NumericFaultSet`**: A branchless-composable bit-field capturing numeric faults (`OVERFLOW`, `UNDERFLOW`, `DIVIDE_BY_ZERO`, `INVALID_DOMAIN`, `RANGE_VIOLATION`). It operates as a join-semilattice. Faults are joined using bitwise union (`|`) alongside the actual computation without stopping execution. This prevents early returns and ensures that no fault is lost (`first-wins` or `last-wins`), completely fulfilling the deterministic fixed bounded execution work mandate.
