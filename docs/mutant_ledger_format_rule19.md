# Rule 19 (Mutant Ledger) Summary

Based on the BCINR Deterministic Substrate Constitution (`AGENTS.md`) and supplementary documentation (`docs/hostile_mutation_ledger_requirements.md`), here is the documentation on how the mutant ledger operates.

## Hostile Mutation Protocol Enforcement
According to **Rule 19. Hostile mutation protocol**, every authoritative implementation file must undergo adversarial verification by the `@armstrong_fault` (Master of Failure Law) agent. Standard software testing practices (like high line coverage) are insufficient; a cryptographic-style ledger is mandatory to provide auditable proof of adversarial adequacy.

For every implementation file, the protocol enforces these steps:
1. Identify at least three load-bearing laws.
2. Produce one syntactically plausible mutant per law.
3. Inject the mutant through the real build path.
4. Run the normal test suite.
5. Verify the expected typed refusal or oracle mismatch.
6. Record the kill evidence in the mutant ledger.

## Mutant Ledger Format
The mutant ledger must structurally log the kill evidence using the following exact fields:

```text
mutant id
source file
changed law
exact mutation
expected detection
actual detection
test name
receipt digest
standing
```

### Key Field Requirements:
- **mutant id**: A unique identifier for the specific mutant being tested.
- **changed law**: The specific mathematical or structural law being altered (e.g., sign inversion, dropped factor, bypassed refusal, incorrect mask).
- **expected detection**: The specific Typed Refusal (e.g., `Err(StabilityRefusal::ContractionMarginInsufficient)`) or exact oracle postcondition violation expected. Simple `assert_ne!` checks are strictly prohibited.
- **receipt digest**: A cryptographic hash/digest of the test execution receipt, providing undeniable proof that the test was actually run against the mutant and successfully killed it.

## Consequences of Enforcement (Surviving Mutants)
The ledger serves as the authoritative source of truth for project standing.
- The `standing` field must be `ALIVE` if the mutant is killed.
- A surviving mutant immediately changes project standing to `MUTATION_GATE_FAILED` and **blocks all feature work**.
- Furthermore, a surviving mutant is considered an "absolute failure." This forces the Substrate Integrity Score (SIS) to `0` and triggers the `MaturityScrutiny` protocol, which mandates freezing feature development, quarantining code, repairing the structural defect, and rerunning all gates before a new standing receipt is issued.
