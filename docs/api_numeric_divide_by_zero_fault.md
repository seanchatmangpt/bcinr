# Branchless `DIVIDE_BY_ZERO` in `bcinr-cmca`

The `NumericFaultSet::DIVIDE_BY_ZERO` fault mask is defined in `crates/bcinr-cmca/src/fixed.rs` as the third bit in the opaque `NumericFaultSet` structure:
```rust
pub const DIVIDE_BY_ZERO: Self = Self(1 << 2);
```

This fault bit is strictly set without branches (no `if`, `match`, or early returns) to satisfy the BCINR Radon Law ($CC=1$) in two core mathematical operations: `saturating_div` and `log2`. 

### 1. The Mathematical Conditions

*   **In `saturating_div`**: It represents attempting to divide a fixed-point number by exactly zero (`const_eq_u32(other.val, 0)`). This is a mathematically undefined condition, and the operation falls back to returning the saturated maximum (`u32::MAX`).
*   **In `log2`**: It represents attempting to compute the base-2 logarithm of exactly zero (`const_eq_u32(self.val, 0)`). Since $\log_2(0) = -\infty$, this violates the valid domain for fixed-point numbers. The function clamps its fallback result to the signed lower bound (`-1048576`).

In both operations, the operation continues execution and safely tracks the fault within its return struct alongside the clamped fallback result.

### 2. How It Is Set Branchlessly

Rather than conditionally jumping or returning early upon detecting zero, the implementation translates predicates directly into bitwise arithmetic via a `CanonicalMask`. 

**Step A: Generating a `CanonicalMask`**
Instead of a standard `bool`, equality is checked via bit-parallel comparison using `const_eq_u32(a, b)`:
```rust
let den_is_zero = const_eq_u32(other.val, 0); // Used in saturating_div
```
This performs `a ^ b` followed by bitwise sign-shifts to collapse the difference into a `CanonicalMask`. This mask is guaranteed to evaluate to exactly `u32::MAX` (all-ones) if true, or `0` (all-zeros) if false.

**Step B: Mask-based Fault Selection**
Once the `CanonicalMask` is obtained, `CanonicalMask::select_faults` is used to multiplex the correct fault bits in place:
```rust
let e = CanonicalMask::select_faults(
    den_is_zero,
    NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
    // ... default/fallback fault alternative
);
```
Under the hood, `select_faults` resolves using constant-time bitwise logic: `(true_faults & mask) | (false_faults & !mask)`. 
* If the mask is `u32::MAX` (condition is true), the bitwise AND preserves the bits for `DIVIDE_BY_ZERO | INVALID_DOMAIN`. 
* If the mask is `0` (condition is false), it cleanly zeroes out those fault bits.

**Step C: Bitwise Fault Accumulation**
Finally, the resulting multiplexed fault `e` is united with any pre-existing faults via a bitwise OR (`union`) inside the returned fixed-point structure:
```rust
Self {
    val: saturate.select_u32(u32::MAX, q_corrected as u32),
    faults: self.faults.union(other.faults).union(e),
}
```

By computing all values and multiplexing them via exact bitwise masks, the execution flow strictly takes the exact same number of deterministic instructions regardless of whether the divisor is zero or non-zero, entirely avoiding control flow branches.
