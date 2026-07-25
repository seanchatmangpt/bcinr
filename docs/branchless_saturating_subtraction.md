# Branchless Saturating Subtraction in `bcinr`

## The Problem with Control Flow
In a strict $CC=1$ environment, conditional branches like `if x < y { 0 } else { x - y }` are prohibited. These constructs introduce variable execution paths, defying the constant-time mathematical requirements that prevent timing side-channels and guarantee deterministic outputs.

## Mathematical Execution
In `bcinr`, saturating subtraction for fixed-point numbers (defined in `crates/bcinr-cmca/src/fixed.rs` as `NonNegativeFixed` and `SignedFixed`) is achieved via bitwise polynomials and canonical masks.

### 1. Branchless Comparison Operators
First, a branchless comparison is performed to mathematically deduce whether a subtraction would underflow or overflow.

For unsigned (non-negative) numbers, the bitwise logic is:
```rust
#[inline(always)]
pub const fn const_lt_u32(a: u32, b: u32) -> CanonicalMask {
    let diff = ((a ^ ((a ^ b) | (a.wrapping_sub(b) ^ b))) >> 31) & 1;
    CanonicalMask(0u32.wrapping_sub(diff))
}
```
This computes the predicate $a < b$ by analyzing the sign bit and potential wrapping effects after subtraction, extracting exactly a `1` if true and `0` if false.

For signed numbers, it checks the signs of the operands:
```rust
#[inline(always)]
pub const fn const_lt_i32(a: i32, b: i32) -> CanonicalMask {
    let diff = (a as u32).wrapping_sub(b as u32);
    let a_sign = (a as u32) >> 31;
    let b_sign = (b as u32) >> 31;
    let diff_sign = diff >> 31;
    let res = (a_sign & (b_sign ^ 1)) | ((!(a_sign ^ b_sign)) & diff_sign);
    CanonicalMask(0u32.wrapping_sub(res))
}
```
This perfectly isolates the sign bits and resolves $a < b$ deterministically, addressing standard signed overflow problems without any `if` statements.

### 2. Canonical Mask Generation
The `CanonicalMask(0u32.wrapping_sub(diff))` operation takes the boolean-equivalent `0` or `1` and applies a two's-complement negation to convert it to a full-width bitmask:
- If `diff` is `0`: `0 - 0 = 0x00000000` (All zeros, `FALSE`)
- If `diff` is `1`: `0 - 1 = 0xFFFFFFFF` (All ones, `TRUE`)

### 3. Bitwise Selection (`select`)
The runtime executes the actual subtraction deterministically, ignoring potential overflow effects initially, and then selects between the saturation limit (e.g., `0`, `MIN`, `MAX`) and the computed difference using the generated canonical mask.
```rust
#[inline(always)]
pub const fn select_u32(self, a: u32, b: u32) -> u32 {
    (a & self.0) | (b & !self.0)
}
```
If the mask is `TRUE` (`0xFFFFFFFF`), `a` is fully preserved. If `FALSE` (`0x00000000`), `b` is preserved.

### 4. Application in `NonNegativeFixed`
For non-negative Q16.16 fixed-point numbers, saturating subtraction brings everything together into a constant-time equation:
```rust
#[inline(always)]
pub const fn saturating_sub(self, other: Self) -> Self {
    let underflow = const_lt_u32(self.val, other.val);
    let e = CanonicalMask::select_faults(
        underflow,
        NumericFaultSet::UNDERFLOW,
        NumericFaultSet::EMPTY,
    );
    Self {
        val: underflow.select_u32(0, self.val.wrapping_sub(other.val)),
        faults: self.faults.union(other.faults).union(e),
    }
}
```
Even the `NumericFaultSet` state relies on branchless selection and bitwise unions rather than short-circuiting accumulation.

### 5. Application in `SignedFixed`
Signed fixed-point numbers handle both underflow and overflow conditions cleanly:
```rust
#[inline(always)]
pub const fn saturating_sub(self, other: Self) -> Self {
    let (diff, overflow) = self.val.overflowing_sub(other.val);
    let is_neg = const_lt_i32(self.val, 0);
    let sat_val = is_neg.select_i32(i32::MIN, i32::MAX);
    let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow as u32));
    let e = CanonicalMask::select_faults(
        overflow_mask,
        NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
        NumericFaultSet::EMPTY,
    );
    Self {
        val: overflow_mask.select_i32(sat_val, diff),
        faults: self.faults.union(other.faults).union(e),
    }
}
```
Here, `overflowing_sub` performs the subtraction inherently safely. If an overflow is detected, `overflow_mask` becomes all-ones. A secondary branchless check (`const_lt_i32(self.val, 0)`) determines whether to saturate to `i32::MIN` or `i32::MAX`, avoiding all $CC=1$ violations.
