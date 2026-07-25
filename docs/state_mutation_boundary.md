# State-Mutation Boundary in `bcinr`

In the context of `@hoare_oracle` (Rule 4) and Rule 10 (No mutation before complete admission) within the BCINR `AGENTS.md` constitution, the **State-Mutation Boundary** is the rigid transactional line separating immutable candidate derivation from persistent state updates. 

The authoritative hot path **does not** pass mutable references around dynamically to gradually update fields. Doing so would violate the rule against speculative mutation (e.g., mutating a field and later returning an `Err(...)` if a subsequent predicate fails, which would leave partial or corrupted state changes behind).

Instead, the boundary is physically enforced in the Rust codebase via a strict, five-step atomic selection pattern:

## The 5-Step Transaction Shape

1. **Current Immutable State**: The hot path begins by reading the persistent state (`x_t`) entirely immutably.
2. **Fixed-Size Candidate State**: Because the authoritative crate is allocation-free (0 heap allocations), the hot path structurally computes a fixed-size `candidate` state entirely on the stack (without heap-backed cloning). 
3. **Verify All Predicates**: It evaluates stability, admission, and certification constraints as branchless bitwise polynomials.
4. **Derive Admission Mask**: It compresses the result of all verified predicates into a full-width binary mask (`m_admitted`). 
5. **Fieldwise Masked Commit**: The exact physical boundary is a branchless assignment representing `x_{t+1} = select(m_admitted, x_candidate, x_t)`. 

## Physical Implementation

Within the Rust codebase (such as in `bcinr-cmca` and `bcinr-api`), the state boundary avoids `if valid { apply } else { rollback }` and instead utilizes fixed-width masks (like `CanonicalMask` for fixed point numbers or primitive bitwise selection). 

Examples of physical primitives used at the boundary include:
* `select_nnf()` / `select_sf()`: Selects between two `NonNegativeFixed` or `SignedFixed` alternatives, natively preserving faults of the selected path while dropping faults of the unselected one.
* `const_select_u32()` / `select_u64()`: Branchless, bitwise selection operations that replace conditional branches for primitive fields.
* `State::select(mask, candidate, current)`: Aggregates fieldwise bit-selections across an entire structural state representation.

By computing the candidate unconditionally and delaying the masked commit until the very end, any rejected operation natively leaves the persistent state **bit-for-bit unchanged** with zero rollback or unwinding logic required.
