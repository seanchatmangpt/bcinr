# `DistinguishabilityInsufficient`

## Definition
`DistinguishabilityInsufficient` is a typed refusal category mandated by **Rule 18** of the BCINR deterministic substrate constitution (`AGENTS.md`). It occurs when a candidate model or adaptive update lacks sufficient numerical distinction from the current state (often derived from Gram matrix bounds). 

Specifically, this refusal is triggered when the numerical distinguishability of a candidate state falls below a predefined `distinguishability_floor` established in the system's stability profile.

## Branchless Mathematical Condition
In compliance with the Radon Law ($CC=1$) and the strict ban on control-flow branches, the refusal condition is enforced without `if`, `match`, or early returns. The check occurs in `bcinr-cmca/src/allocator.rs` within `AdaptiveUpdate::admit_adaptive_update`.

The mathematical condition is evaluated using a branchless bitwise calculation to form a boolean mask:

```rust
// 1. Retrieve and scale the distinguishability floor from the stability profile
let dist_floor = ((crate::generated::stability_profile::PROFILE
    .distinguishability_floor
    .raw
    * 65536)
    / 1_000_000_000) as u32;

// 2. Branchlessly compare distinguishability against the floor
// const_lt_u32 returns 1 if lhs < rhs. Comparing the result to 0 flips the logic, 
// yielding 1 if distinguishability >= dist_floor, and 0 otherwise.
let dist_ok = (const_lt_u32(distinguishability.value_bits(), dist_floor) == 0) as u32;

// 3. Combine masks and index into an outcome array (0 -> None, 1 -> Some(Proof))
let ok = temp_ok & dist_ok & digests_ok;
let outcomes = [
    None,
    Some(Self {
        _mode: core::marker::PhantomData,
    }),
];
outcomes[(ok as usize) & 1]
```

### Refusal Propagation
If the distinguishability is less than the floor, `dist_ok` becomes `0`, forcing the index to `0` and returning `None`. At the allocation boundary, this absence of cryptographic proof branchlessly sets the `AUTHORITY_MISSING` mask:

```rust
let degrade_to_certified_selection = proof.is_none();
// ...
.union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as u32))
```
This forces the commit gate to reject the update entirely, leaving the persistent state bit-for-bit unchanged.
