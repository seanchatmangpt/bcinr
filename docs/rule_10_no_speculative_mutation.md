# Rule 10: No Mutation Before Complete Admission

## Overview
In the BCINR Deterministic Substrate, Rule 10 strictly prohibits the speculative mutation of persistent state. Any state changes must be fully verified and unconditionally admitted before being applied. 

## The 5-Step Required Transaction Shape
To comply with BCINR's allocation-free and branchless architecture, every state update must follow this exact 5-step sequence:

1. **Current immutable state**: Begin by treating the existing persistent state as strictly read-only.
2. **Fixed-size candidate state**: Construct the new potential state. Because the authoritative runtime strictly forbids heap allocations, "cloning" the state means copying it into a fixed-size stack value, using a fixed-size scratch structure, or computing the candidate structurally.
3. **Verify all predicates**: Mathematically evaluate all conditions, bounds, and admission requirements for the transaction.
4. **Derive admission mask**: Condense the predicate evaluations into a single full-width bitmask ($m_{\mathrm{admitted}}$).
5. **Fieldwise masked commit**: Apply the state change unconditionally using a branchless selection function:
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
   If the mask indicates rejection, $x_{t+1}$ automatically becomes $x_t$.

## Why is Speculative Mutation Prohibited?

Speculative mutation (e.g., modifying state arrays and then reverting or returning early if an error is caught) is fundamentally incompatible with the BCINR constitution for several reasons:

1. **The Radon Law (CC=1) Violation**: Rollbacks require conditional branches (e.g., `if invalid { return Err(); }`). The absolute runtime law dictates that the authoritative instruction shape must not depend on semantic input, prohibiting any `if`, `match`, or early returns.
2. **Bit-for-bit Guarantee**: The constitution dictates that a rejected operation must leave persistent state "bit-for-bit unchanged." Speculative mutation risks transient invalid states, tearing, and potential side-channel leaks if an interruption occurs between mutation and rollback.
3. **Avoidance of Rollback Allocations**: Conventional transaction rollbacks typically rely on heap-backed snapshots, undo logs, or dynamic memory handling. BCINR is an allocation-free environment with a strict zero-heap boundary.
4. **Enforcing Mathematical Law**: The overarching principle is "Rich semantics upstream. Fixed deterministic mechanics downstream." By delaying the commit until a pure, arithmetic mask is derived, the substrate ensures that state mutations mathematically map to the specified Hoare contracts before touching persistent memory.
