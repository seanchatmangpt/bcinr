# Branchless Overflow Handling and `NumericRangeExceeded` in BCINR

In accordance with BCINR's strict deterministic and branchless mandate ($CC=1$), the fixed-point math engine handles overflow without relying on panic paths, unwinding, or early-returns. When a Q16.16 fixed-point calculation (such as addition or division) exceeds absolute maximum bounds, the "hot path" handles it seamlessly through SIMD Within A Register (SWAR) masking and sticky error accumulation.

## 1. Overflow Evaluation and Canonical Masking

To avoid data-dependent branching like `if overflow`, the engine uses `CanonicalMask`, a construct that evaluates strictly to either `0xFFFFFFFF` (true) or `0x00000000` (false). The mask is derived mathematically from the bitwise behaviors of operations.

For example, during a saturating addition, the boolean overflow result is cast into a `CanonicalMask` using wrapping arithmetic:
```rust
let (sum, overflow) = self.val.overflowing_add(other.val);
let overflow_mask = CanonicalMask { val: 0u32.wrapping_sub(overflow as u32) };
```

## 2. Mathematical Selection and Saturation

Instead of silently wrapping or trapping, the calculation uses the `overflow_mask` to branchlessly select a saturated value (`i32::MAX` or `i32::MIN`). `CanonicalMask::select_i32` applies bitwise `AND` and `OR` operations to enforce the saturated boundaries mathematically:

```rust
let is_neg = const_lt_i32(self.val, 0);
let sat_val = is_neg.select_i32(i32::MIN, i32::MAX);

// Saturated bounded equivalent:
let safe_val = overflow_mask.select_i32(sat_val, sum);
```

## 3. Triggering the `NumericRangeExceeded` Typed Refusal

The Q16.16 structures (`SignedFixed` and `NonNegativeFixed`) couple their computational bits with an internal `err` state. The same `overflow_mask` that enforced saturation is reused to map the overflow condition to the `StabilityRefusal::NumericRangeExceeded` bounded typed refusal code:

```rust
let e = overflow_mask.select_u32(StabilityRefusal::NumericRangeExceeded as u32, u32::MAX);
```

## 4. Branchless Sticky Error Accumulation

BCINR utilizes a "Sticky Error Accumulator" (`branchless_err_acc`) to retain the earliest typed refusal triggered in an operation chain. It bitwise-unions the new error into the struct's existing error state. 

```rust
#[inline(always)]
pub const fn branchless_err_acc(e1: u32, e2: u32) -> u32 {
    let e1_is_ok = const_eq_u32(e1, u32::MAX);
    e1_is_ok.select_u32(e2, e1)
}
```

The fully branchless arithmetic structure then returns both the saturated value and the accumulated error:

```rust
Self {
    val: safe_val,
    err: branchless_err_acc(self.err, branchless_err_acc(other.err, e)),
}
```

This guarantees that the invalid-input refusal is deterministically propagated through any subsequent hot-path operations without breaking the instruction stream. Later validation components correctly handle the accumulated `NumericRangeExceeded` typed refusal.
