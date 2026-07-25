# Rule 19: Mutant Ledger and Hostile Mutation Protocol

According to Rule 19 (Hostile mutation protocol) in `AGENTS.md`, the mutant ledger must record kill evidence containing exactly these fields:

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

### 1. How mutants are injected through the real build path
Rule 19 defines a strict 6-step protocol for hostile mutation on every implementation file:

1. Identify at least three load-bearing laws.
2. Produce one mutant per law.
3. Inject the mutant through the real build path.
4. Run the normal test suite.
5. Verify the expected typed refusal or oracle mismatch.
6. Record the kill evidence in the ledger.

Rule 4 (`@armstrong_fault` role) further dictates that typed-refusals are required. Tests must prove that the corrupted implementation either violates a specific contract or triggers a typed refusal (e.g., `assert_eq!(result, Err(StabilityRefusal::ContractionMarginInsufficient));`). Just checking `assert_ne!(baseline, mutant)` is prohibited.

### 2. How the project/CI handles a surviving mutant (`MUTATION_GATE_FAILED` state)
A surviving mutant is treated as a critical constitutional violation in the project:
* **Rule 19:** A surviving mutant immediately changes the project standing to `MUTATION_GATE_FAILED` and blocks all feature work.
* **Rule 24 (Substrate Integrity Score):** A "surviving mutant" is explicitly listed as an absolute failure. Any absolute failure forces the Substrate Integrity Score (SIS) to `0` regardless of other scores, and triggers `MaturityScrutiny`.
* **Rule 25 (MaturityScrutiny protocol):** Triggering this requires the following steps:
   1. Freeze feature development;
   2. Quarantine affected code;
   3. Identify all reachable authoritative symbols;
   4. Rerun proofs, scans, mutants, and disassembly;
   5. Produce a root-cause report;
   6. Repair the structural defect;
   7. Regenerate all dependent artifacts;
   8. Rerun the complete gate matrix;
   9. Issue a new standing receipt.
