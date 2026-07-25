### Mathematical Design of Independent Mutants

According to `AGENTS.md` and the theory of hostile mutants, each authoritative implementation file must have at least three independent, syntactically plausible mutants. These mutants are designed by deliberately altering **meaningful, load-bearing mathematical or structural laws**. They are not simply arbitrary code changes or `assert_ne!`-only theater; they represent counterfactual scenarios.

Examples of mathematical design for mutants include:
- **Sign inversion:** Reversing the mathematical logic or flipping a positive operation to negative.
- **Dropped factor:** Omitting a multiplier or crucial term in a mathematical calculation.
- **Incorrect mask:** Using a bitmask that selects the wrong state or fails to isolate intended bits properly.
- **Normalization omission:** Failing to normalize a vector or value where the contract requires it.
- **Index skew:** Off-by-one errors or selecting the wrong element in a bounded lookup table.
- **Incorrect clamp:** Applying wrong boundaries or failing to saturate arithmetic correctly.
- **Bypassed refusal:** Removing early-exit or failure paths that reject invalid inputs.

### Tracking in the Mutant Ledger

The kill evidence of these mutants is systematically tracked in a mutant ledger (e.g., `MUTANT_KILL_MATRIX.md`). Rule 19 explicitly mandates that the ledger must contain the following fields:

- `mutant id` (e.g., `mutant_1`)
- `source file` (the file being mutated)
- `changed law` (description of the targeted invariant, e.g., "sign inversion on Q-value")
- `exact mutation` (details of how the code is altered)
- `expected detection` (the anticipated typed refusal or numeric fault)
- `actual detection` (the precise error caught by the oracle)
- `test name` (the dedicated oracle test)
- `receipt digest`
- `standing` (e.g., `KILLED_BY_INTENDED_ORACLE`)

### The Hostile Mutant Lifecycle (Rule 19 Protocol)

The lifecycle of a hostile mutant is governed by a strict 6-step protocol to ensure tests do not passively pass, but actively trap regressions:

1. **Identify** at least three load-bearing laws for the target implementation file.
2. **Produce** one mutant per law (a syntactically plausible, real code change).
3. **Inject** the mutant through the real build path (e.g., via a compile-time `cfg` gate or feature flag, never via a test mock).
4. **Run** the normal test suite against the corrupted implementation.
5. **Verify** the expected **typed refusal** or **independent oracle mismatch**. (Simple `assert_ne!` assertions are strictly prohibited; the suite must prove the system mathematically and semantically identified the exact violated invariant).
6. **Record** the kill evidence in the mutant ledger.

**Enforcement:** If any mutant survives (i.e., step 5 fails), the project standing immediately changes to `MUTATION_GATE_FAILED`, which forcefully blocks all feature work until the structural defect in the test suite or oracle is repaired.
