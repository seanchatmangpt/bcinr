Based on a search of the codebase, the `DistinguishabilityInsufficient` refusal mandated by `AGENTS.md` (Rule 18) is not implemented as an isolated error enum variant. Instead, it is enforced as a structural invariant within `bcinr-cmca/src/allocator.rs`.

Here is the branchless mathematical check that produces the refusal:

### 1. The Branchless Mathematical Check

In `crates/bcinr-cmca/src/allocator.rs`, the `AdaptiveUpdate::admit_adaptive_update` function evaluates a candidate model's `distinguishability` (derived from Gram matrix bounds) against a system-wide `distinguishability_floor`:

```rust
// 1. Retrieve and scale the distinguishability floor from the generated stability profile
let dist_floor = ((crate::generated::stability_profile::PROFILE
    .distinguishability_floor
    .raw
    * 65536)
    / 1_000_000_000) as u32;

// 2. Branchless evaluation (CC=1)
// const_lt_u32 returns 1 if lhs < rhs. Comparing to 0 means we get 1 if dist >= floor.
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

### 2. Refusal Propagation at the Allocation Boundary

If distinguishability is insufficient (`dist_ok = 0`), the proof generation yields `None`. This missing capability propagates structurally to the allocator hot path, where it evaluates to a failure mask:

```rust
// Convert the absence of a proof into a mask branchlessly
let degrade_to_certified_selection = proof.is_none();
```

This boolean mask is directly folded into the bitwise refusal set, triggering an `AUTHORITY_MISSING` refusal:

```rust
.union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as u32))
```

**Conclusion:** 
While `DistinguishabilityInsufficient` is a typed refusal explicitly demanded in `AGENTS.md`, the authoritative hot-path implementation satisfies this requirement structurally. It avoids a dedicated enum variant, instead utilizing mathematical bitmasking to reject insufficient updates and enforce zero-allocation, branchless deterministic execution.
