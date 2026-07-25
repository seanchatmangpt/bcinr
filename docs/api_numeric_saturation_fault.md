I have investigated `NumericFaultSet::SATURATION` in `crates/bcinr-cmca/src/fixed.rs` and documented my findings below.

# `NumericFaultSet::SATURATION` Analysis

`NumericFaultSet::SATURATION` is a specific fault bit (`1 << 6`) defined in `NumericFaultSet` within `crates/bcinr-cmca/src/fixed.rs`.

## Mathematical Condition

The `SATURATION` fault represents that a fixed-point arithmetic operation has exceeded the maximum (or minimum) bounds of the target representable domain (`[0, u32::MAX]` for `NonNegativeFixed` or `[i32::MIN, i32::MAX]` for `SignedFixed`), causing the output to be clamped (saturated) to the bound instead of wrapping around.

It is typically combined via a bitwise union with `NumericFaultSet::OVERFLOW`. The condition applies in operations like:
- `NonNegativeFixed::saturating_add`: When the mathematical sum exceeds `u32::MAX`.
- `NonNegativeFixed::saturating_mul`: When the 64-bit product, right-shifted by 16 bits, exceeds `u32::MAX`.
- `SignedFixed::saturating_add` and `saturating_sub`: When integer overflow or underflow boundaries are crossed.

## Branchless Implementation Mechanics

In strict adherence to the project's **Radon Law (CC=1)**, `SATURATION` is set without any conditional branching (`if`, `match`, or early returns). The branchless injection of this fault involves three steps:

### 1. Deriving an Overflow Boolean
The overflow condition is computed as a 1 or 0 value purely through bitwise arithmetic. 
For example, in unsigned saturating addition, it checks if `sum < a` without branching:
```rust
let sum = self.val.wrapping_add(other.val);
let overflow = const_lt_u32(sum, self.val);
```
*(Where `const_lt_u32` computes `<` via bitwise XORs and sign-bit extraction).*

### 2. Expanding to a Canonical Mask
The boolean is cast into a `CanonicalMask`, which guarantees an all-1s (`u32::MAX`) or all-0s (`0`) value. This is typically done via bitwise negation or wrapping subtraction:
```rust
// e.g. for multiplication
let overflow = (high | high.wrapping_neg()) >> 31; 
let overflow_mask = CanonicalMask(0u32.wrapping_sub(overflow));
```

### 3. Masked Fault Selection
Finally, `CanonicalMask::select_faults` is used to assign the fault bits bitwise. 
```rust
let e = CanonicalMask::select_faults(
    overflow,
    NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
    NumericFaultSet::EMPTY, // 0
);
```
Internally, `select_faults` performs an AND/OR mask operation: `(fault_a & mask) | (fault_b & !mask)`. 

The returned fault `e` is then unioned with the existing faults using a bitwise `|`, ensuring the failure condition propagates forward purely through data-dependencies rather than control-flow jumps.
