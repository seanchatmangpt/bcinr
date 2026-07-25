I have researched the `DistinguishabilityInsufficient` typed refusal in the `bcinr` codebase. Here are the details you requested:

### What it is
`DistinguishabilityInsufficient` is a typed refusal category mandated by Rule 18 of the BCINR deterministic substrate constitution. It occurs when a candidate model or adaptive update lacks sufficient numerical distinction from the current state (often derived from Gram matrix bounds). The distinguishability value is compared against a predefined `distinguishability_floor` from the system's stability profile.

### When it is surfaced
This refusal is surfaced during the authorization of an adaptive model update, specifically within `bcinr-cmca/src/allocator.rs` (`AdaptiveUpdate::admit_adaptive_update`). 
Instead of throwing an error or branching, the system performs a branchless bitwise calculation (using `const_lt_u32`) to generate a boolean mask. If the distinguishability is lower than the floor, the mask evaluates to `0`, causing the function to return `None` rather than yielding the `AdaptiveUpdate` cryptographic proof. 
At the final allocation boundary, the absence of this proof is branchlessly converted into an `AUTHORITY_MISSING` refusal bit in the `RefusalSet`, leaving persistent states (like weights and modes) bit-for-bit invariant.

### Why it is a refusal rather than a branching failure or assertion
Under the BCINR constitution, it must be a bounded typed refusal for several critical reasons:
1. **Strict Branchlessness (Radon Law, CC=1):** Assertions, panics, `if` statements, or early returns are strictly prohibited on the hot path. A typed refusal allows the failure to be propagated purely via bitwise masking and fixed-width arithmetic.
2. **Zero-Allocation and Bounded Execution:** Panics or string-based exceptions would violate the `#![no_std]` and zero-heap allocation laws. A typed refusal guarantees constant-time bounded execution memory and execution work.
3. **Deterministic State Invariance (Rule 10):** Instead of panicking halfway or allowing partial mutations, the refusal is smoothly folded into the commit gate mask, ensuring the operation is cleanly rejected and the persistent state remains exactly unchanged.
4. **Hostile Mutants Verification (Rule 19):** Adversarial tests (`@armstrong_fault`) require predictable, verifiable typed refusals to prove that hostile mutants are caught. Testing for a specific typed refusal is mandatory, whereas testing for panics or `assert_ne!` is prohibited.
