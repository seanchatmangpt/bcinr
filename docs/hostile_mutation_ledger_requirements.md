# Hostile Mutation Ledger Requirements

As mandated by Rule 19 (Hostile mutation protocol) of the BCINR Deterministic Substrate Constitution, every authoritative implementation file must be subjected to adversarial verification by `@armstrong_fault`. The evidence of this verification must be recorded in a cryptographic-style **mutant ledger**.

## The Hostile Mutation Protocol

For every implementation file, `@armstrong_fault` (Master of Failure Law) must execute the following protocol:
1. Identify at least three load-bearing laws.
2. Produce one syntactically plausible mutant per law.
3. Inject the mutant through the real build path.
4. Run the normal test suite.
5. Verify the expected typed refusal or oracle mismatch.
6. Record the kill evidence in the mutant ledger.

## Required Ledger Fields

To provide verifiable proof of boundary testing, the mutant ledger must contain the following specific fields for every injected mutant:

* **mutant id**: A unique identifier for the specific mutant being tested.
* **source file**: The exact path to the authoritative implementation file being mutated.
* **changed law**: The specific mathematical or structural law being altered (e.g., sign inversion, dropped factor, bypassed refusal, incorrect mask).
* **exact mutation**: The precise source code alteration made to create the mutant.
* **expected detection**: The specific Typed Refusal (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`) or exact oracle postcondition violation expected. (Note: simple `assert_ne!` checks are strictly prohibited).
* **actual detection**: The actual refusal or mismatch observed when the test suite is run against the mutant, which must exactly match the expected detection.
* **test name**: The specific test in the suite that successfully caught and killed the mutant.
* **receipt digest**: A cryptographic hash/digest of the test execution receipt, providing undeniable proof that the test was actually run against the mutant and successfully killed it.
* **standing**: The resulting project standing (must be `ALIVE` if killed; any surviving mutant immediately yields `MUTATION_GATE_FAILED`).

## Why Cryptographic-Style Record-Keeping is Mandatory

In the BCINR project, standard software testing practices (like high line coverage) are insufficient for a civilizational-scale deterministic substrate. This rigid, cryptographic-style ledger is mandatory for several reasons:

1. **Proof of Adversarial Adequacy:** The constitution states, *"A suite that cannot kill a plausible mutant is itself defective."* The ledger proves that `@armstrong_fault` has effectively tested the boundaries by confirming the test suite actually enforces contract boundaries and typed refusals, rather than merely passing on the "happy path."
2. **Elimination of Self-Certification:** It guarantees that `@armstrong_fault` operates independently from the implementation owner (`@von_neumann_bypass`). It provides a mechanical, auditable artifact proving that the implementation's refusal paths, error envelopes, and invariants are structurally sound.
3. **Traceability and Mathematical Reproducibility:** By requiring the `exact mutation`, specific typed refusal, and a `receipt digest`, the ledger creates an indisputable trail. Automated gates and auditors can mathematically verify that the exact mutant was injected and precisely caught by the specified test.
4. **Immediate Consequence Enforcement:** The ledger serves as the authoritative source of truth for project standing. If a mutant survives, the project standing immediately changes to `MUTATION_GATE_FAILED`, blocking all feature work. The ledger ensures this maturity scrutiny is triggered automatically without human intervention or subjective judgment.
