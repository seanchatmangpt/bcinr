# Atomic Masked Commit Transaction Shape

This document explains the principles and mechanics behind Rule 10 ("No mutation before complete admission") from the BCINR `AGENTS.md` constitution.

## Rule 10: No Mutation Before Complete Admission

In the BCINR substrate, persistent state must never be mutated speculatively. The architecture mandates that any state transition only occurs after all validations and predicates have been fully evaluated and reduced to a single branchless admission mask.

### The Illegal Pattern: Speculative Mutation

The following pattern is strictly prohibited:

```rust
state.mass[i] = candidate;
state.weight[i] = next_weight;

if invalid {
    // Requires rollback or leaves state corrupted
    return Err(...);
}
```

This pattern is completely illegal because:
1. **Requires Rollback**: If an operation is invalid, reversing the partial mutations requires conditional logic (branches).
2. **Violates Branchlessness (CC=1)**: The `if invalid` block introduces a control-flow branch, violating the core deterministic laws of the substrate.
3. **Breaks Transactional Integrity**: A failure during the validation phase would leave the system in an inconsistent, partially mutated state.

## The Atomic Branchless Commit Transaction Shape

To ensure strict compliance with the branchless and allocation-free laws, BCINR enforces the following transaction shape:

1. **Current Immutable State**: Start with the existing state ($x_t$).
2. **Fixed-Size Candidate State**: Compute the new potential state ($x_{candidate}$) without modifying the current state.
3. **Verify All Predicates**: Evaluate all business rules and mathematical bounds branchlessly.
4. **Derive Admission Mask**: Reduce all verification results into a single boolean mask ($m_{admitted}$), where all bits are 1 if admitted, and 0 if rejected.
5. **Fieldwise Masked Commit**: Apply the final state transition.

The mathematical law governing the final commit is:

$$x_{t+1} = \operatorname{select}(m_{admitted}, x_{candidate}, x_t)$$

This is implemented using bitwise operations (or equivalent SWAR/SIMD mechanics):
$$x_{t+1} = (m_{admitted} \land x_{candidate}) \lor (\neg m_{admitted} \land x_t)$$

## Eliminating Rollbacks and Heap Allocations

This structural rule fundamentally removes the need for state rollbacks and heap-backed cloning:

* **Zero Rollbacks**: Because the current state ($x_t$) is never touched during the candidate generation and validation phases, there is absolutely nothing to undo if the operation is rejected. The `select` operation unconditionally executes; if $m_{admitted}$ is 0, it simply re-selects $x_t$, leaving the persistent state bit-for-bit unchanged.
* **Zero Heap Allocation**: Since the authoritative crate is allocation-free, "cloning" the state to create a candidate does not mean allocating on the heap. Because all authoritative memory access is bounded and fixed-width, candidates are constructed entirely by:
  - Copying into a fixed-size stack value
  - Using a fixed-size scratch structure
  - Computing the candidate structurally

By pushing all conditional logic into data (masks) and deferring all state mutation to a single, unconditional, bit-parallel selection, the authoritative runtime remains perfectly branchless, deterministic, and allocation-free.
