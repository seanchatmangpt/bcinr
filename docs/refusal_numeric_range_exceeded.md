# `NumericRangeExceeded` in BCINR

In accordance with BCINR's strict deterministic and branchless mandate ($CC=1$), `NumericRangeExceeded` is a typed refusal code designed to handle arithmetic boundary violations (like overflow or underflow) strictly through bitwise logic, without relying on panic paths, unwinding, or early returns.

## Exact Definition
It is defined as a variant of the `StabilityRefusal` enum:
```rust
// In crates/bcinr-cmca/src/allocator.rs
pub enum StabilityRefusal {
    // ...
    NumericRangeExceeded,
    // ...
}
```
Internally, it acts as a bounded typed refusal code (mapped to index 18 in `REFUSALS`) that is surfaced when mathematical operations on Q16.16 fixed-point structures (`SignedFixed` / `NonNegativeFixed`) violate absolute maximum bounds.

## Branchless Mathematical Condition
The condition that triggers it avoids data-dependent control flow (no `if` or `match`) and operates via **SIMD Within A Register (SWAR)** masking and **Sticky Error Accumulation**:

1. **Canonical Masking for Bounds Evaluation:**
   When an operation occurs (e.g., an addition), the overflow boolean is computed mathematically into a `CanonicalMask` that strictly evaluates to `0xFFFFFFFF` (true) or `0x00000000` (false) via bitwise logic (e.g., `const_lt_u32` or wrapping subtraction).
   ```rust
   let (sum, overflow) = self.val.overflowing_add(other.val);
   let overflow_mask = CanonicalMask { val: 0u32.wrapping_sub(overflow as u32) };
   ```

2. **Masked Value Saturation:**
   The `overflow_mask` is then used to branchlessly bitwise-select between the valid calculation and a clamped mathematical boundary (e.g., `i32::MAX`, `i32::MIN`, or `u32::MAX`), forcing saturation without branching.
   ```rust
   let safe_val = overflow_mask.select_i32(sat_val, sum);
   ```

3. **Masked Error Selection and Sticky Accumulation:**
   The exact same `overflow_mask` maps the boundary violation to the refusal state. It selects either the `NumericRangeExceeded` fault state (or equivalent internal `NumericFaultSet`) or a clear state.
   ```rust
   let e = overflow_mask.select_u32(StabilityRefusal::NumericRangeExceeded as u32, u32::MAX);
   ```
   Finally, this error state is bitwise-unioned (`branchless_err_acc`) into the numeric structure's running fault state. This forms a mathematical join-semilattice where the earliest refusal sticks continuously through the operation chain without short-circuiting:
   ```rust
   err: branchless_err_acc(self.err, branchless_err_acc(other.err, e))
   ```

At the authoritative root boundary, this aggregated error state guarantees the deterministically accumulated `NumericRangeExceeded` refusal is surfaced.
