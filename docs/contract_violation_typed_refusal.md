# ContractViolation Typed Refusal

In the BCINR deterministic substrate, all authoritative operations must abide by **Rule 18: Typed Refusals**. String-based, human-readable error messages are strictly banned from the hot path to avoid hidden allocations, variable execution times, and branching formats. Instead, failures must be mapped to bounded, fixed-width enumerations. 

Among these, the `ContractViolation` refusal holds a central mathematical role.

## Mapping to `@hoare_oracle` Mathematical Contracts

The **Oracle of Invariants** (`@hoare_oracle`) establishes rigorous mathematical proof obligations for every primitive in the form of a Hoare logic contract: 

$$ \{P(x)\} \quad f(x) \quad \{Q(x,f(x))\} $$

These contracts define exact requirements, including:
* **Valid Input Domain**: The precise bounds of admissible inputs.
* **Conservation Laws**: Invariants that must be preserved across state transitions (e.g., conservation of semantic mass).
* **Monotonicity and Overflow**: Strict definitions of directionality and numeric boundaries.

A `ContractViolation` is emitted when one of these axiomatic constraints is breached. It signifies that the input is mathematically inadmissible for the requested operation, meaning the function cannot safely satisfy its $Q(x,f(x))$ postcondition without compromising the system's structural integrity.

## Structural Evaluation and the Branchless Mandate

According to the Radon Law ($CC=1$) and **Rule 8 (Absolute CC=1 Law)**, any data-dependent branch (`if`, `match`, early returns, or the `?` operator) is prohibited in the authoritative call graph. Therefore, a precondition or conservation law failure cannot be evaluated via a traditional `if (!valid) return Err(...)`.

Instead, the system relies on bit-parallel mask derivation and fixed-width arithmetic:

1. **Mask Computation**: Preconditions and boundary limits are evaluated using branchless numeric primitives (e.g., `ge_mask(x, y)` which uses signed overflow shifts to yield `!0` for valid and `0` for invalid).
2. **Constraint Intersection**: Multiple contract constraints are aggregated using bitwise operations (`&`, `|`) into a single full-width `admission_mask`.

## Bubbling Up Without Branches

To respect **Rule 10 (No mutation before complete admission)** and avoid Result-based control flow branches, the refusal is bubbled up structurally:

* **State Preservation**: The derived `admission_mask` determines state transitions via branchless multiplexers:
  `let next_state = select(admission_mask, candidate_state, current_state);`
  If a `ContractViolation` occurs (mask is 0), the candidate state is discarded, and the persistent state remains bit-for-bit unchanged.
* **Refusal Propagation**: The exact error variant (e.g., the enum discriminant for `ContractViolation`) is returned via mathematical selection. The function executes completely, generating both a candidate output and a typed refusal discriminant. The discriminant is mapped branchlessly (e.g., `select(admission_mask, OK_CODE, CONTRACT_VIOLATION_CODE)`).
* **Call Graph Propagation**: Callers receive a packed, fixed-size structure (e.g., result and discriminant) and propagate the error state further up the stack using continued bitwise intersections. This guarantees the refusal reaches the top boundary of the authoritative runtime without a single conditional instruction being executed.
