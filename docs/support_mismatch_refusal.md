# Branchless Handling of `SupportMismatch` Typed Refusals in BCINR

In `bcinr`, the deterministic computational substrate mandates strict adherence to the **Radon Law** ($CC=1$). Hot-path execution must contain zero data-dependent branches, early returns (`?`), panics, or unbounded loops. When evaluating bounded data structures, resource constraints, or topological graphs (such as partially ordered workflow graphs), any deviation from the mathematically proven operational boundaries must trigger a `SupportMismatch` typed refusal.

According to `AGENTS.md`, `bcinr` cannot use branching logic such as `if !valid_support { return Err(SupportMismatch); }`. Instead, it resolves bounds checking and mismatches structurally using bitwise polynomials and SWAR (SIMD Within A Register) masking. 

Here is the step-by-step branchless mechanism for evaluating support bounds and topological mismatches:

## 1. Branchless Mask Derivation for Topological Support

Instead of using boolean logic for control flow, BCINR translates the bounds check into a full-width **Canonical Mask** (e.g., `0xFFFFFFFFFFFFFFFF` for true, `0x0000000000000000` for false). When verifying whether an incoming transition or allocation request aligns with the admitted topological support, the constraints are calculated in bit-parallel.

```rust
// A support mismatch occurs if any requested nodes fall outside the statically admitted topology mask.
let out_of_bounds = requested_nodes & !admitted_support_mask;
let has_mismatch = (out_of_bounds != 0) as u64;

// Transform the mismatch boolean into a full-width SWAR mask
let mismatch_mask = 0u64.wrapping_sub(has_mismatch);
```

## 2. Safe Fallback Selection for Continuous Execution

To guarantee constant-time execution without triggering architecture-dependent hardware panics or early returns, the hot path mathematically executes the full transition pipeline. If a mismatch is detected, the substrate uses a branchless multiplexer (`select(mask, active, fallback)`) to enforce a safe, fixed fallback state (such as a null op or a safely clamped configuration). This ensures the underlying processing pipeline completes unharmed regardless of input validity.

```rust
// Branchlessly route to a safe fallback state if a SupportMismatch is detected
let candidate_state = select(
    mismatch_mask, 
    safe_fallback_state, 
    computed_next_state
);
```

## 3. "Sticky" Fault Accumulation

Instead of immediately halting execution and unwinding, the `SupportMismatch` fault is accumulated into a bitwise fault accumulator (e.g., `TopologyFaultSet`). The `mismatch_mask` branchlessly selects the `SUPPORT_MISMATCH` fault bit and logically `OR`s it into the transaction's running fault state. This forms a join-semilattice under bitwise union, allowing faults to accumulate in strict constant time without any "first-error-wins" short-circuiting logic.

```rust
let e = CanonicalMask::select_faults(
    mismatch_mask,
    TopologyFaultSet::SUPPORT_MISMATCH,
    TopologyFaultSet::EMPTY,
);
```

## 4. Bounded Sealed Structs

The function returns a fixed-width candidate struct that explicitly pairs the mathematical computation (even if derived from safe fallbacks) with the accumulated fault mask.

```rust
BoundedTopologyResult {
    candidate: candidate_state,
    faults: self.faults.union(e), // Merge upstream faults with the SupportMismatch fault
}
```

## 5. Hot Path Boundary Unpacking and Masked Commit

At the outer boundary of the authoritative hot path, the overarching transaction mask is calculated. If the accumulated fault set contains the `SUPPORT_MISMATCH` bit, the transition is classified as unadmitted. The system enforces the rejection via a field-wise masked commit:

$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$

Because the transaction is rejected, the state remains bit-for-bit unchanged. Finally, the safe API boundary translates the fault bit into the `Err(StabilityRefusal::SupportMismatch)` enumeration result for the upstream caller—fulfilling the `AGENTS.md` typed refusal requirement with zero cyclomatic complexity.
