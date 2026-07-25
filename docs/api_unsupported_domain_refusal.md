Here is the documentation on how the `UnsupportedDomain` refusal is handled branchlessly in the CMCA crate.

### `UnsupportedDomain` Branchless Refusal Mechanism

In `crates/bcinr-cmca/src/allocator.rs` and `crates/bcinr-cmca/src/fixed.rs`, the condition surfaces as `StabilityRefusal::UnsupportedDomain`, which is represented internally by the `NumericFaultSet::INVALID_DOMAIN` bitmask (`1 << 3`).

#### What Structural Condition It Represents
The `UnsupportedDomain` refusal is strictly emitted when an authoritative operation receives an input that falls outside its rigorously defined and mathematically proven domain. Common triggers in the substrate include:
- Attempting to take the logarithm of zero (`is_zero` in `log2()`).
- Attempting to divide by zero (`den_is_zero` in `fixed_div()`).

Because BCINR enforces the **Radon Law ($CC=1$)**, the runtime is legally forbidden from handling this mathematically inadmissible condition with a traditional control flow jump (e.g. `if input == 0 { return Err(UnsupportedDomain); }`).

#### How the Bitmask is Set Branchlessly
The `INVALID_DOMAIN` bitmask is set and propagated through a strict 5-step SWAR (SIMD Within A Register) pipeline:

1. **Mask Generation:** Boundary conditions are evaluated into a branchless `CanonicalMask` (a strict `0xFFFFFFFF` for true or `0x00000000` for false), e.g., `let is_zero = const_eq_u32(self.val, 0);`
2. **Safe Fallback Execution:** To avoid architecture-dependent hardware panics while preserving constant-time execution, a safe fallback value (e.g. a clamped value like `-1048576` for `log2(0)`) is branchlessly selected to feed the pipeline using `is_zero.select_i32(-1048576, computed)`.
3. **Sticky Fault Selection:** The `NumericFaultSet::INVALID_DOMAIN` (and corresponding bits like `DIVIDE_BY_ZERO`) is selected branchlessly using the condition mask:
   ```rust
   let e = CanonicalMask::select_faults(
       is_zero,
       NumericFaultSet::DIVIDE_BY_ZERO.union(NumericFaultSet::INVALID_DOMAIN),
       NumericFaultSet::EMPTY,
   );
   ```
4. **Fault Accumulation:** The bitmask is accumulated with any upstream faults via bitwise union (`self.faults.union(e)`) and permanently paired with the safe value within a sealed `SignedFixed` or `NonNegativeFixed` struct.
5. **Boundary Unpacking:** The faults traverse the entire call graph until the absolute boundary of the hot path in `allocator.rs` (`AllocationOutcome`). If the aggregated fault set contains the invalid domain bit, mapping functions (like `wrap_result`) translate it safely into the explicit `StabilityRefusal::UnsupportedDomain` typed refusal. The control plane then drops the transaction, leaving persistent state bit-for-bit unchanged via masked commits.
