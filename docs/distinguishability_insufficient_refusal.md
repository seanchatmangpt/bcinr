# `DistinguishabilityInsufficient` Typed Refusal in BCINR

Based on the `bcinr` codebase, here is a detailed breakdown of how the hot path branchlessly calculates model distinguishability and how the refusal is propagated.

## 1. The Branchless Calculation

The mathematical check for whether a candidate model is distinct enough to warrant an adaptive update is performed in `bcinr-cmca/src/allocator.rs` by the `AdaptiveUpdate::admit_adaptive_update` function. 

This function receives a `distinguishability: NonNegativeFixed` parameter (which represents the numerical distinguishability between states, likely derived from the Gram matrix bounds stored in `StabilityCandidate`) and validates it against the `distinguishability_floor` defined in the system's stability profile.

In accordance with the substrate's branchless requirements (the Radon Law, $CC=1$), the calculation strictly avoids `if` statements. Instead, it utilizes `const_lt_u32` and bitwise arithmetic to formulate a boolean mask (`dist_ok`):

```rust
// Retrieve and scale the distinguishability floor from the generated stability profile
let dist_floor = ((crate::generated::stability_profile::PROFILE
    .distinguishability_floor
    .raw
    * 65536)
    / 1_000_000_000) as u32;

// Branchlessly compare the input distinguishability against the floor.
// const_lt_u32 returns 1 if lhs < rhs. Comparing the result to 0 flips the logic, 
// yielding 1 if distinguishability >= dist_floor, and 0 otherwise.
let dist_ok = (const_lt_u32(distinguishability.value_bits(), dist_floor) == 0) as u32;

// ... [temp_ok and digests_ok are calculated similarly]

// Combine masks and index into an outcome array
let ok = temp_ok & dist_ok & digests_ok;
let outcomes = [
    None,
    Some(Self {
        _mode: core::marker::PhantomData,
    }),
];
outcomes[(ok as usize) & 1]
```

If the distinguishability falls short, `ok` evaluates to `0`, and the array returns `None` rather than yielding a sealed `AdaptiveUpdate` proof.

## 2. Refusal Propagation

While `AGENTS.md` explicitly mandates `DistinguishabilityInsufficient` as a required typed refusal category, the current implementation in the `bcinr-cmca` crate propagates this refusal structurally rather than as an isolated error enum variant (e.g., it is absent from `StabilityRefusal`).

Here is how the refusal makes its way through the hot path:

1. **Proof Denial**: Because `admit_adaptive_update` returns `None` when distinguishability is insufficient, the system is denied the cryptographic proof (`AdaptiveUpdate<CertifiedLearning>`) required to authorize an adaptive state mutation.
2. **Allocator Degradation**: When the `allocate` function is called, it receives this proof as `Option<&AdaptiveUpdate<Mode>>`. Inside `allocate`, the absence of the proof is branchlessly evaluated into a mask:
   ```rust
   let degrade_to_certified_selection = proof.is_none();
   ```
3. **Refusal Masking**: The allocator uses this boolean to enforce the state boundary. It folds the failure into the `AUTHORITY_MISSING` refusal bit via the `RefusalSet`:
   ```rust
   .union(RefusalSet::AUTHORITY_MISSING.masked(degrade_to_certified_selection as u32))
   ```
4. **State Invariance**: Because `degrade_to_certified_selection` evaluates to `true`, the state commit gate prevents any write-back of mutated weights or mode changes. The adaptive mutation is fully rejected, leaving the `weights`, `last_switch_t`, and `prev_mode` perfectly invariant.

*(Note: As observed in `tests/case_studies.rs` and `REFUSAL_REALIZATION_REPORT.md`, several specific numeric refusals like `LearningRateOutsideEnvelope` have been folded into broader bits like `PROPOSAL_REJECTED` or `AUTHORITY_MISSING` in the sealed API. The `DistinguishabilityInsufficient` refusal operates similarly by structurally failing the `AdaptiveUpdate` construction, which subsequently yields an authority/proof refusal at the allocation boundary.)*
