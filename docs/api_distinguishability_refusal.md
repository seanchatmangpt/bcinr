# `DistinguishabilityInsufficient` Typed Refusal Implementation

I have located the structural components responsible for enforcing the `DistinguishabilityInsufficient` condition branchlessly within the `bcinr-cmca` crate.

Per the `bcinr` design guidelines (Rule 18 and Rule 10), rather than producing an isolated `DistinguishabilityInsufficient` enum error variant, this refusal is manifested as a structural invariant at the allocation boundary.

## 1. Branchless Check in `AdaptiveUpdate::admit_adaptive_update`

In `crates/bcinr-cmca/src/allocator.rs`, the mathematical check validates the candidate model's `distinguishability` (derived from the Gram matrix bounds) against a system-wide `distinguishability_floor`:

```rust
// 1. Retrieve the floor from the generated profile
let dist_floor = ((crate::generated::stability_profile::PROFILE
    .distinguishability_floor
    .raw
    * 65536)
    / 1_000_000_000) as u32;

// 2. Perform the constant-time (CC=1) branchless check
// `const_lt_u32` returns 1 if lhs < rhs. Comparing to 0 means we get 1 if dist >= floor.
let dist_ok = (const_lt_u32(distinguishability.value_bits(), dist_floor) == 0) as u32;

// ...
let ok = temp_ok & dist_ok & digests_ok;

// 3. Return the structurally proven capability or `None` without branching
let outcomes = [
    None,
    Some(Self {
        _mode: core::marker::PhantomData,
    }),
];
outcomes[(ok as usize) & 1]
```

## 2. Refusal Propagation at the Allocation Boundary

If the check fails, `None` is returned, denying the cryptographic proof (`AdaptiveUpdate`). This failure cascades structurally in the main allocation path (`crates/bcinr-cmca/src/allocator.rs`):

```rust
// Convert the absence of a proof into a mask
let degrade_to_certified_selection = proof.is_none();
```

Later, during state committal, the system incorporates this boolean directly into the typed refusal set bitmask, yielding `AUTHORITY_MISSING`:

```rust
.union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as u32))
```

This ensures the persistent state (weights and learning modes) remains perfectly invariant and the typed refusal guarantees deterministic operation (0 heap allocations, `CC=1` enforcement, zero panic paths), all matching the BCINR constitutional mandate.
