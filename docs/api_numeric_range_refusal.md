# Branchless Evaluation of Bounded Domains and `NumericRangeExceeded`

Based on an analysis of the `crates/bcinr-cmca/src/allocator.rs` and `crates/bcinr-cmca/src/fixed.rs` files, here is how the BCINR deterministic substrate enforces bounded domains and propagates the `NumericRangeExceeded` refusal branchlessly ($CC=1$).

## 1. Locations of `NumericRangeExceeded`

`NumericRangeExceeded` is located exclusively within the `StabilityRefusal` enum and its mapping arrays in `crates/bcinr-cmca/src/allocator.rs`:
- **Line 404:** Definition within the `StabilityRefusal` enum.
- **Line 443:** Branchless mapping inside `StabilityRefusal::from_u32`.
- **Line 485:** Pre-computed element mapped in the `REFUSALS` constant array at index 18.

## 2. Branchless Bounds Evaluation

The evaluation of inputs against mathematical bounds strictly avoids data-dependent control flow (e.g., `if overflow`, `unwrap`, or `match`). Instead, it relies on Bitwise Polynomials and SIMD Within A Register (SWAR) techniques.

### Canonical Masks
When evaluating an operation against its bounds (e.g., an addition that might overflow), the engine computes the boundary condition into a `CanonicalMask`. This mask represents the evaluation strictly as either all ones (`0xFFFFFFFF` for true) or all zeros (`0x00000000` for false), entirely avoiding branches.

```rust
// Example from fixed.rs (saturating_add):
let sum = self.val.wrapping_add(other.val);
let overflow = const_lt_u32(sum, self.val); // Returns a CanonicalMask branchlessly
```

### Masked Selection for Bounded Saturation
Instead of returning early or trapping on a bounds violation, the operation uses the `CanonicalMask` to bitwise-select between the valid wrapped result and the clamped domain boundary (e.g., `u32::MAX` for upper bounds).

```rust
val: overflow.select_u32(u32::MAX, sum),
```
This forces the value to safely saturate without breaking straight-line execution.

## 3. Propagation to Typed Refusals

The core fixed-point structures (`NonNegativeFixed` and `SignedFixed`) tightly couple their numeric bits with a sticky error accumulator state known as a `NumericFaultSet`.

1. **Mapping the Mask to a Fault:** The same canonical mask used to saturate the computation value is reused to select the mathematical fault flag (`OVERFLOW`, `SATURATION`, etc.).
   ```rust
   let e = CanonicalMask::select_faults(
       overflow,
       NumericFaultSet::OVERFLOW.union(NumericFaultSet::SATURATION),
       NumericFaultSet::EMPTY,
   );
   ```

2. **Branchless Error Accumulation:** The new fault is bitwise-unioned into the operation's running fault state. This forms a join-semilattice where errors stick continuously without short-circuiting.
   ```rust
   faults: self.faults.union(other.faults).union(e),
   ```

3. **Surfacing at the Boundary:** The aggregated numeric faults bubble up to the authoritative root (`allocate()`), which folds all candidate paths into an `AllocationOutcome`. When adapted for external callers using the legacy `wrap_result` function or translated via error codes, any bounds violation mapped via the numeric faults (index 18) surfaces as a `StabilityRefusal::NumericRangeExceeded` typed refusal.

By strictly isolating bitwise logic and carrying fault sets dynamically within the return types, the substrate perfectly enforces its deterministic domain boundaries while completely avoiding jump instructions.
