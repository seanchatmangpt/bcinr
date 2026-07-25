# No Speculative Mutation (Rule 10)

In the `bcinr` deterministic substrate, persistent state must never be mutated speculatively. This ensures that any rejected operation leaves the system bit-for-bit unchanged without relying on panic unwinding, early returns, or heap-backed rollbacks.

## The Prohibited Pattern: Speculative Mutation

A common but strictly prohibited pattern is mutating state *before* validating the operation, using branching for error handling.

```rust
// PROHIBITED: Speculative mutation followed by an early return on failure.
state.mass[i] = candidate;
state.weight[i] = next_weight;

if invalid {
    return Err(StabilityRefusal::ContractionMarginInsufficient);
}
```

This pattern violates the deterministic substrate's rules because it relies on control flow (branching) to handle errors, and leaves the persistent state corrupted if the operation is invalid.

## The Required Transaction Shape

To guarantee safe and deterministic state transitions, `bcinr` mandates a strict, branchless transaction pipeline:

1. **Current immutable state**: Read the current state without modifying it.
2. **Fixed-size candidate state**: Compute the proposed changes.
3. **Verify all predicates**: Evaluate all validity conditions (without branching).
4. **Derive admission mask**: Convert the combined validity conditions into a full-width bitmask.
5. **Fieldwise masked commit**: Apply the candidate state to the persistent state using the admission mask.

## Allocation-Free Candidate Generation

Because `bcinr` is `#![no_std]` and strictly zero heap-allocation (The Zero-Allocation Boundary), creating a "candidate state" cannot rely on heap cloning (`clone()` backed by an allocator). 

Instead, "cloning the state" means:
* Copying data into a fixed-size stack value.
* Using a pre-allocated fixed-size scratch structure.
* Computing the candidate structurally on the fly.

## Full-Width Masked Commits

The final commit step must avoid all branching (no `if valid { ... } else { ... }`). Instead, the operation must use a full-width mask ($m \in \{0, 2^w-1\}$) to conditionally select the new state. The selection must take a form equivalent to:

$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

If the operation is valid, $m_{\mathrm{admitted}}$ is all `1`s (e.g. `0xFFFFFFFF`), and the candidate state is selected. If the operation is rejected, $m_{\mathrm{admitted}}$ is all `0`s, and the original state ($x_t$) is retained.

Because this selection is fieldwise and fixed-width, a rejected operation leaves the persistent state bit-for-bit unchanged, purely via bitwise arithmetic logic.
