# BCINR Mathematical Law & Oracle Research

Based on the `AGENTS.md` Deterministic Substrate Constitution and the `tests/reference.rs` implementation, here is how BCINR enforces mathematical rigor, specification, and testing.

## 1. The Hoare Oracle (Rule 4)
The `@hoare_oracle` role is the axiomatic proof lead and specification owner. Every primitive in the authoritative runtime requires a strict mathematical contract to be established before any implementation is considered valid.

### Hoare Contracts
Every primitive must have a Hoare contract of the form `{P(x)} f(x) {Q(x,f(x))}`. This contract MUST explicitly define:
- Valid input domain and output range
- Conservation laws and monotonicity laws (where applicable)
- Overflow behavior and numeric error envelopes
- Invalid-input refusal mechanisms
- Determinism and state-mutation boundaries

### Full-Domain Requirement
Random testing alone is explicitly prohibited for establishing universal standing. Full-domain standing must be proven via:
1. A formal proof.
2. Exhaustive proof over a finite partition covering the domain.
3. A bit-vector solver certificate.
4. An equivalent bounded theorem artifact.
> *Standard:* "If a property cannot be stated precisely, it is not yet law."

## 2. Independent Reference Semantics (Rule 15)
Rule 15 outlines the **Independent Oracle Law**. For a reference oracle to be valid, it must be logically and structurally distinct from the production implementation.

### Distinctness & Anti-Cheating
- Oracles cannot simply be line-by-line translations of production code, nor can they reuse production lookup tables, fixed-point helpers, or normalization routines.
- They must take independent forms such as a direct mathematical formula, abstract state machine, symbolic proof, arbitrary-precision implementation, or exhaustive reduced-domain enumerator.
- **Example in `tests/reference.rs`**: The reference file uses floating-point (`f64`) operations and standard branching structures (e.g., explicit indexing, standard `if`-based clamps, standard loops). It explicitly suppresses Rust production lints (like `clippy::needless_range_loop` and `clippy::manual_clamp`) so it can retain its distinct mathematical control-flow shape, fully isolated from the branchless SWAR fixed-point constraints of the authoritative runtime.

## 3. How Mathematical Laws are Specified and Tested
Nontrivial implementations must follow a mandatory decomposition protocol requiring extreme segregation of duties:

1. **Specification (`@hoare_oracle`)**: Creates the contracts and proof obligations.
2. **Implementation (`@von_neumann_bypass`)**: Writes the branchless bounded code that strictly satisfies the contracts. (The implementer is strictly forbidden from authoring their own final oracle or self-certifying equivalence.)
3. **Hostile Verification (`@armstrong_fault`)**: Creates counterfactual mutants. Tests must verify that a mutant violates a specific contract and triggers a **typed refusal** (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`). Relying on simple `assert_ne!` is deemed a cheat ("mutant theater").
4. **Structural Enforcement (`@turing_machine`)**: Audits source and object code to ensure strict $CC=1$ cyclomatic complexity, zero allocation, and zero hidden branches.

Mathematical laws are therefore tested both by guaranteeing structural independence via an unlinked oracle (e.g., `tests/reference.rs`), and verifying exact equivalence under bounded simulation and against adversarial typed-refusal mutations.
