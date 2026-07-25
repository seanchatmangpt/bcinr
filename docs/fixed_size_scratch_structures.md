# Fixed-Size Scratch Structures and Branchless Commits

Under **Rule 10 (No mutation before complete admission)** of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), persistent state must never be speculatively mutated. The substrate enforces a strict transactional boundary: candidate mutations must be staged, validated, and finally applied (or discarded) as a single branchless operation. 

Crucially, because the authoritative runtime operates under a **Zero-Allocation Boundary** (`#![no_std]`, no heap allocation), the traditional idiom of "cloning the state" to stage a mutation is strictly forbidden if it implies heap allocation. Instead, the substrate mandates the use of **fixed-size scratch structures**.

## Design of the Fixed-Size Scratch Structure

The "scratch structure" is not a dynamic buffer or a heap-allocated tree; it is a **stack-allocated, fixed-size value**. This design is tightly coupled with the Radon Law ($CC=1$, branchless execution) and ensures predictable $O(1)$ space and time complexity.

The scratch structure takes several forms across the codebase depending on the tier of execution:
1. **Plain Rust Structs on the Stack**: For domain-specific state transitions (e.g., `ModeState` in `mode_switch.rs`), the scratch structure is just a local variable instantiated with the next state's properties.
2. **Fixed-Size Arrays**: For byte-level text/encoding microkernels (e.g., SIMD sorts or hashes in `bcinr-logic`), the scratch structure is often a fixed-width byte array (like `[u8; 8]`) or a packed `u64` register used to accumulate ranks and diffs.
3. **Pure Arithmetic Accumulators**: In the numeric hot path (e.g., `allocator.rs`), the scratch space might just be a set of `NonNegativeFixed` registers tracking candidate distributions before the final commit.

## Utilization in the Hot Path: Staging and the Masked Commit

The lifecycle of a mutation in the hot path follows a strict 4-step sequence:

### 1. Compute the Candidate Structurally (Staging)
The candidate state is computed **unconditionally**. Regardless of whether the transition is valid, the calculation occurs and the result is staged in the fixed-size scratch structure. This avoids control-flow branches that would violate $CC=1$.

```rust
// From `bcinr-cmca/src/mode_switch.rs`
// Candidate is computed unconditionally (no branch gates its computation), 
// per the masked-commit law: "compute the candidate structurally," then select.
let candidate = ModeState {
    mode_digest: switch.target_mode_digest,
    generation: persistent.generation.wrapping_add(1),
};
```

### 2. Verify Predicates and Derive Admission Mask
All validity checks (e.g., certificate matching, dwell token satisfaction) are evaluated purely mathematically to produce a single unified boolean or a `CanonicalMask`. No early returns or `if invalid { return Err(...) }` are allowed before the commit phase.

```rust
let cert_ok = certificate == expected_certificate;
let dwell_ok = dwell.round_identity() == round_identity;
let state_ok = switch.admitted_state_digest == persistent.mode_digest;
let admitted = cert_ok && dwell_ok && state_ok;
```

### 3. Fieldwise Masked Commit (`select`)
The final application of the candidate state back into the persistent state uses a branchless **select** operation. If `admitted` is true, the candidate from the scratch structure overwrites the persistent state. If false, the persistent state replaces itself, leaving it bit-for-bit unchanged. 

For smaller arithmetic values, this is done via bitwise masks. The `CanonicalMask` type in `fixed.rs` enforces this:
```rust
// From `bcinr-cmca/src/fixed.rs`
// {P: self in {TRUE, FALSE}} select_u32(a, b) {Q: result == a if self == TRUE else b}
#[inline(always)]
pub const fn select_u32(self, a: u32, b: u32) -> u32 {
    (a & self.0) | (b & !self.0)
}
```

For domain structures (where Rust compilers reliably emit `CMOV` or equivalent branchless instructions for fixed-size stack values), it looks like this:
```rust
let old_control_mode_digest = persistent.mode_digest;
// Masked commit: branchless select between scratch candidate and current persistent state
let next = if admitted { candidate } else { *persistent };
*persistent = next;
```

### 4. Typed Refusal Return
Only *after* the persistent state is safely settled via the masked commit does the function package any refusal reasons into a typed, bounded structure (e.g., `ActuationEvidence` or `RefusalSet`) to return to the caller. 

## Summary
By forcing candidates to be staged in **fixed-size scratch structures**, the architecture ensures:
- Zero dynamic memory allocation (0 heap).
- Execution time is strictly $O(1)$.
- Rejected operations are guaranteed to leave the state bit-for-bit unchanged through mathematical identity (`current & TRUE = current`), eliminating entirely the class of bugs where partial mutations leak on error paths.
