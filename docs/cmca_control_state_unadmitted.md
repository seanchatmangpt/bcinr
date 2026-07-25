Here is the analysis regarding the `ControlStateUnadmitted` typed refusal and the branchless mathematical checks that produce it.

### Findings on `ControlStateUnadmitted`
Based on a search across the workspace, the `ControlStateUnadmitted` typed refusal is mandated by Rule 18 of the BCINR constitution (`AGENTS.md`) and documented in `docs/control_state_unadmitted_refusal.md`. 

However, it is currently missing from the actual source code in `crates/bcinr-cmca/src/` and `crates/bcinr-api/src/`. As noted in the project's own documentation (`docs/control_state_unadmitted_refusal.md`), the implementation in `crates/bcinr-cmca/src/allocator.rs` currently violates Rule 18 and Rule 8 by returning `Option::None` instead of branchlessly emitting the `ControlStateUnadmitted` typed refusal.

### The Structural and Mathematical Check
The check that is supposed to produce this refusal (but currently produces `Option::None`) is located in `crates/bcinr-cmca/src/allocator.rs` within the `AdaptiveUpdate::admit_adaptive_update` function (around line 1035).

The function branchlessly determines if a proposed state update is admitted by calculating a unified `ok` bitmask from three mathematical predicates:

1. **Temperature Ceiling Check (`temp_ok`)**
   Ensures the proposed temperature does not exceed the profile ceiling using branchless comparison:
   ```rust
   let temp_ok = (const_lt_u32(temp_ceil, temperature.value_bits()) == 0) as u32;
   ```

2. **Distinguishability Floor Check (`dist_ok`)**
   Ensures the distinguishability meets or exceeds the profile floor:
   ```rust
   let dist_ok = (const_lt_u32(distinguishability.value_bits(), dist_floor) == 0) as u32;
   ```

3. **Cryptographic Digest Match (`digests_ok`)**
   Verifies that the `state`, `cert`, `env`, and `outcome` digests all match identically by accumulating their differences via bitwise XOR (`^`) and OR (`|`). If they all match, the result is `0`:
   ```rust
   let digests_ok = (((state.digest ^ cert.digest)
       | (state.digest ^ env.digest)
       | (state.digest ^ outcome.digest))
       == 0) as u32;
   ```

**Branchless Selection**
These predicates are bitwise ANDed together to form the final admission mask:
```rust
let ok = temp_ok & dist_ok & digests_ok;
```
Finally, the code uses array indexing to branchlessly select the outcome without any `if` or `match` statements:
```rust
let outcomes = [
    None, // <- This is where ControlStateUnadmitted should be emitted
    Some(Self {
        _mode: core::marker::PhantomData,
    }),
];
outcomes[(ok as usize) & 1]
```

When `ok == 0`, the predicate pipeline fails, and the code branchlessly yields `None` (representing the condition that should yield `ControlStateUnadmitted`), fulfilling the `CC=1` (Cyclomatic Complexity of 1) requirement mandated by the constitution.
