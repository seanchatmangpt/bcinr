Here is the detailed research on the "Test-only oracle" classification from `AGENTS.md`:

### Test-Only Oracle

Under **Rule 6 (Authoritative versus non-authoritative code)** of the `AGENTS.md` constitution, a **Test-only oracle** is defined as an independent mathematical specification that is strictly excluded from production features.

#### What it is
* **Independent Reference**: It is a structurally and logically distinct model used to verify the correctness of the authoritative runtime.
* **Permitted Forms (Rule 15)**: The oracle can take the form of a direct mathematical formula, Hoare specification, abstract state machine, symbolic proof, arbitrary-precision implementation, SAT/SMT bit-vector model, or exhaustive reduced-domain enumerator.
* **Strict Independence**: It must not simply be a translation of production code or reuse production structures (e.g., no reusing production normalization, lookup tables, or wrapping the authoritative function). 

#### How it relates to production features
* **Mandatory for Acceptance (Rule 1)**: Every authoritative primitive in production must have an independent oracle or proof; a feature is not considered complete without it.
* **Excluded from the Hot Path (Rule 6)**: While essential for verifying production code, the oracle itself is explicitly excluded from the production feature set.
* **Verification over Self-Certification (Rules 5 & 27)**: The oracle evaluates the production implementation but cannot be authored by the same agent writing the production code (`@von_neumann_bypass`). It must be owned and reviewed by the axiomatic proof lead (`@hoare_oracle`).
* **Mutant Detection (Rule 4 & 19)**: When hostile mutants (corrupted implementations) are introduced into production code, the independent oracle helps identify the exact violated postcondition to ensure the test suite properly kills the mutation.
