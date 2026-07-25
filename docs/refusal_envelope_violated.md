# The `EnvelopeViolated` Typed Refusal

In the BCINR deterministic substrate, the `EnvelopeViolated` typed refusal (implemented as `StabilityRefusal::RuntimeEnvelopeViolated` in `crates/bcinr-cmca/src/allocator.rs`) is a critical safety mechanism mandated by Rule 18 of the project's constitution (`AGENTS.md`).

## Exact Definition

`EnvelopeViolated` structurally enforces mathematical and operational boundaries defined prior to hot-path execution via the `AcceptedEnvelopeReceipt` (per Rule 11, the ReceiptSound Law). It acts as a bounded, typed failure code, strictly complying with the substrate's zero-allocation and branchless execution rules (the Radon Law, $CC=1$). 

It is triggered when an authoritative computation exceeds dynamically certified boundaries in three structural circumstances:
1. **Numeric Limits Exceeded**: Fixed-point arithmetic operations (e.g., Q16.16) overflow, underflow, or breach declared admissible domains.
2. **Approximation Error Bounds Exceeded**: Parameter evaluations surpass explicitly declared maximum absolute or relative error bounds (e.g., for polynomial approximations).
3. **Capacity Limits Violated**: Structural allocations, like bumping offsets in a `BumpArena`, exceed fixed deterministic capacities.

## Branchless Mathematical Condition

Because BCINR forbids panics, `if`/`else` control flow, or early returns, `EnvelopeViolated` boundaries are enforced via constant-time mathematical operations and boolean bit-masks:

1. **Mask Generation**: The boundary condition is evaluated mathematically to yield a bit-mask. For instance, testing bounds using constant-time comparators like `const_eq_u32` or bitwise logic. A valid operation produces a mask of all ones (`0xFFFFFFFFFFFFFFFF`), whereas a breach evaluates to all zeros (`0x0`).
2. **Fault Accumulation**: The substrate mathematically unions fault bits into a running state (e.g., `self.faults = self.faults.union(e)`), accumulating errors linearly like a join-semilattice rather than short-circuiting.
3. **Gating State Mutation**: To enforce "no mutation before complete admission", persistent state updates are gated through a branchless fixed-width selection function:
   ```rust
   next_state = select(mask, candidate_state, current_state)
   ```
   When the envelope is violated, `mask` evaluates to `0`. Consequently, `select(0, candidate, current) = current`. The rejected operation leaves the state bit-for-bit unchanged.
4. **Substrate Boundary Mapping**: The operation completes its fixed $O(1)$ cycle time opaquely. The accumulated bitwise fault mask is unwrapped at the substrate boundary (via mappers like `wrap_result` and `StabilityRefusal::from_u32`). The translation uses a fixed lookup table and branchless array indexing (`const_select_u32(in_bounds, val, 21) & 31`) to output the `StabilityRefusal::RuntimeEnvelopeViolated` enum. This cleanly signals the MAPE-K Autonomic Loop to begin recovery.
