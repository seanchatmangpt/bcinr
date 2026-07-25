# Rule 2: Constitutional Precedence & Typed Refusal in BCINR

In the BCINR deterministic substrate, **Rule 2 (Constitutional Precedence)** dictates a strict, non-negotiable hierarchy for resolving conflicts during development. The core governing principle is: **"Rich semantics upstream. Fixed deterministic mechanics downstream."**

## The Hierarchy of Precedence
When instructions, goals, or methodologies conflict, this strict 7-step order applies (highest to lowest priority):

1. **Mathematical safety and typed refusal**
2. **`AGENTS.md`** (The Constitution)
3. **Repository contract gates**
4. **Crate-local architecture documents**
5. **Issue or task requirements**
6. **Agent preferences**
7. **Implementation convenience**

## Elevation over Idioms and Convenience
**Mathematical safety and typed refusal (Rank 1)** structurally dominate standard Rust idioms (Rank 6) and implementation convenience (Rank 7). The project explicitly nullifies common engineering defenses:

- **No Weakening:** No agent may weaken a higher-order rule to satisfy a lower-order objective.
- **Banned Justifications:** Claims such as "faster", "simpler", "idiomatic", or "the compiler will optimize it" are constitutionally invalid if they compromise Rank 1 or Rank 2 laws.
- **Strict Enforcement:** Introducing an idiomatic `if let`, standard `unwrap`, or a bounds-check panic inherently violates the $CC=1$ (branchless) mandate. The codebase enforces this via `bcinr-cheat-scanner`, automated object-code audits (`@turing_machine`), and zero-warning merge blocking. Violations instantly drop the Substrate Integrity Score (SIS) to `0`.

## Application & Typed Refusal Overrides
Because standard error handling (e.g., `Result::Err` with `?`, `panic!`, `if`/`else`) introduces control-flow branches, BCINR overrides idiomatic Rust error handling with **Typed Refusals**. A typed refusal is a bounded domain or stability violation enforced mathematically—using strict bitwise polynomials and masked state selection—rather than through control-flow syntax.

### Examples of Constitutional Precedence in Action:

1. **The "Faster" Implementation Override**
   - *Scenario:* A developer removes a bounds-checking mask in a division algorithm to improve micro-benchmark speed (Rank 7).
   - *Application:* Rejected. Removing the mask compromises Rank 1 (Mathematical safety). Execution speed cannot override the mathematical requirement for strict boundary evaluation.

2. **The "Simpler" Idiomatic Option Override**
   - *Scenario:* Using `if let Some(x) = ...` or `match` to unwrap an `Option`, arguing it is idiomatic, readable Rust (Rank 6).
   - *Application:* Rejected. `if let` introduces a branch, violating Rank 2 ($CC=1$). Standard idiomatic control flow is completely overridden by the mandate for bitwise deterministic masks.

3. **The "Compiler Will Optimize It" Override**
   - *Scenario:* Writing `for item in variable_slice` assuming the compiler will unroll the loop in release mode.
   - *Application:* Rejected. The constitution mandates structural branchlessness. Trusting opaque compiler optimization heuristics is not accepted as a mathematical proof.

### Examples of Branchless Typed Refusals Overriding Standard Errors:
- **`StabilityRefusal::EnvelopeViolated`:** Instead of panicking or throwing an error on an out-of-bounds `BumpArena` allocation, the boundary violation mathematically derives a `0` admission mask. This deterministically enforces `select(0, candidate, current) = current`, leaving persistent state bit-for-bit unchanged without a single branch.
- **`StabilityRefusal::CertificateDigestMismatch`:** Rather than early-returning `Err` upon a certificate mismatch, the execution logically folds the fault into a global `RefusalSet` bitflag using constant-time $O(1)$ operations, securely propagating the refusal to the Autonomic Loop.
