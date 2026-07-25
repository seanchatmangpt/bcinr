# Analysis: `ControlStateUnadmitted` Typed Refusal

## 1. Search Results
A search for `ControlStateUnadmitted` within `crates/bcinr-cmca/src/` and `crates/bcinr-api/src/` yielded no results. However, cross-referencing with the project's documentation (`docs/control_state_unadmitted_refusal.md` and `docs/cmca_control_state_unadmitted.md`) reveals that the refusal is mandated by Rule 18 but is currently missing from the codebase. The implementation in `crates/bcinr-cmca/src/allocator.rs` currently returns `Option::None` instead of branchlessly emitting the `ControlStateUnadmitted` typed refusal, violating Rule 18 and Rule 8.

## 2. Structural and Mathematical Checks
The logic that is supposed to produce the `ControlStateUnadmitted` refusal branchlessly is located in `crates/bcinr-cmca/src/allocator.rs` within the `AdaptiveUpdate::admit_adaptive_update` function.

The function determines if a proposed state update is admitted by branchlessly computing a unified bitmask from three mathematical predicates:

### A. Temperature Ceiling Check (`temp_ok`)
Ensures the proposed temperature does not exceed the profile ceiling using branchless comparison:
```rust
let temp_ok = (const_lt_u32(temp_ceil, temperature.value_bits()) == 0) as u32;
```

### B. Distinguishability Floor Check (`dist_ok`)
Ensures the distinguishability meets or exceeds the profile floor:
```rust
let dist_ok = (const_lt_u32(distinguishability.value_bits(), dist_floor) == 0) as u32;
```

### C. Cryptographic Digest Match (`digests_ok`)
Verifies that the `state`, `cert`, `env`, and `outcome` digests all match identically by accumulating their differences via bitwise XOR and OR operations. A match yields `0`:
```rust
let digests_ok = (((state.digest ^ cert.digest)
    | (state.digest ^ env.digest)
    | (state.digest ^ outcome.digest))
    == 0) as u32;
```

## 3. Branchless Selection Mechanism
These individual predicates are bitwise ANDed together to form the final admission mask:
```rust
let ok = temp_ok & dist_ok & digests_ok;
```

To branchlessly select the outcome (avoiding `if` or `match`), the code uses array indexing:
```rust
let outcomes = [
    None, // <- Should emit ControlStateUnadmitted instead of None
    Some(Self {
        _mode: core::marker::PhantomData,
    }),
];
outcomes[(ok as usize) & 1]
```

When `ok == 0` (meaning at least one predicate failed), the code branchlessly yields the first array element. Under the constitutional rules of BCINR, this should return the `ControlStateUnadmitted` typed refusal, enforcing the requirement mathematically without control-flow branching (`CC=1`).
