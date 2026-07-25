# Hoare Oracle Contracts (`@hoare_oracle`)

## Overview

In the `bcinr` (BranchlessCInRust) codebase, the **Oracle of Invariants** (`@hoare_oracle`) is the axiomatic proof lead and specification owner. It establishes the rigorous mathematical constraints required for a deterministic computational substrate. Every primitive must operate with absolute branchless determinism, allocation-free execution, and bounded performance.

`@hoare_oracle` holds **exclusive authority** over:
* Preconditions and postconditions
* Invariants and algebraic laws
* Admissible domains and refusal conditions
* Proof obligations
* Independent reference semantics

## Mathematical Proof Obligations & Required Outputs

Every primitive in the codebase must have a strictly defined Hoare contract:

$$ \{P(x)\} \quad f(x) \quad \{Q(x,f(x))\} $$

The contract is not merely documentation but an executable specification. Every contract **must** include:

1. **Valid Input Domain**: A mathematically precise boundary of admitted inputs.
2. **Output Range**: The exact bounded codomain of the operation.
3. **Conservation Law**: The invariant properties preserved across the transformation (e.g., semantic mass).
4. **Monotonicity Law**: (Where applicable) strictly defining directionality of state evolution.
5. **Overflow Behavior**: Precise specification of saturation or wrapping semantics.
6. **Invalid-Input Refusal**: Bounded, typed refusal codes (e.g., `ContractViolation`, `NumericRangeExceeded`) without panics or branching.
7. **Determinism**: The absolute guarantee that specific admitted inputs produce a fixed instruction shape and deterministic outputs.
8. **State-Mutation Boundary**: Defines exactly when and how the bitwise transition occurs (e.g., using `select` over full-width masks). No mutation is permitted before complete admission.
9. **Numeric Error Envelope**: The maximum absolute and relative error bounds for approximations, with independent references.

### Full-Domain Standing

A primitive must explicitly satisfy its obligations across its entire domain. Merely stating "covers the entire $2^{64}$ domain" via random testing is insufficient. Universal standing must be established through:
* A formal mathematical proof.
* An exhaustive proof over a finite partition that covers the domain.
* A bit-vector solver (SAT/SMT) certificate.
* An equivalent bounded theorem artifact.

*Standard*: If a property cannot be stated precisely, it is not yet law.

## Independent Reference Semantics (Oracle Law)

The implementation must be validated against an independent oracle. It is formally prohibited for the oracle to merely mirror the implementation.

**Prohibited Oracle Patterns (Circular Oracles):**
* Line-by-line translation of production code.
* Reuse of production lookup tables, fixed-point helpers, or normalization routines.
* Identical control structure substituted with `f64`.
* Importing the authoritative function and simply wrapping it.

**Permitted Independent Forms:**
* Direct mathematical formulas.
* Formal Hoare specifications.
* Abstract state machines.
* Symbolic proofs.
* Arbitrary-precision implementations.
* SAT/SMT bit-vector models.
* Exhaustive reduced-domain enumerators.

The oracle must be structurally and logically distinct from the production implementation, independently reviewed by `@hoare_oracle`, and capable of killing hostile mutants.

## Typed Refusals

Rejected authoritative operations must produce bounded, typed refusal codes. Human-readable text and panic paths are prohibited in the hot path. Instead, mathematical contracts define explicit rejection behaviors which are evaluated using bitwise operations rather than conditional control flow (`if`/`match`). 

## Validation and Separation of Concerns

`@hoare_oracle` establishes the mathematical contract but cannot self-certify its implementation. Verification is distributed across specific agents:
* **`@turing_machine`** verifies the structural compliance (Cyclomatic Complexity $CC=1$, branchless object code).
* **`@armstrong_fault`** validates that the contract stands against adversarial mutants.
* **`@von_neumann_bypass`** builds the bit-parallel, branchless implementation.

The implementation is only admitted when all mathematical proof obligations (pre/post-conditions), structural rules, and adversarial tests pass without exception, yielding a Substrate Integrity Score (SIS) of 100/100.
