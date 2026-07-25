# BCINR Standing Vocabulary

## Core Standing Labels (Rule 28)
Rule 28 strictly bounds the acceptable standing vocabulary for components to these 10 labels:
- **`PROVEN`**: A specific theorem is machine-checked or exhaustively established over its declared domain.
- **`INVARIANT`**: True by construction or type exclusion.
- **`ALIVE`**: The implementation executes and passes all declared gates in the pinned environment.
- **`SOURCE_BRANCHLESS_PARTIAL`**: Source appears branchless, but complete object-code standing is not established.
- **`BRANCHLESS_ALIVE`**: The authoritative call graph passes source, complexity, allocation, panic, and disassembly audits.
- **`REPORTED_ALIVE`**: An agent reports success, but independent reproduction has not occurred.
- **`PARTIAL_ALIVE`**: Some required gates remain incomplete.
- **`UNKNOWN`**: Evidence is insufficient.
- **`REFUSED`**: The input or configuration is outside the admitted domain.
- **`BUILD_BROKEN`**: The pinned build fails.

## Critical Exception & Lockdown States
While not part of the standard component lifecycle labels, the following states govern emergency halting and absolute failures:
- **`MUTATION_GATE_FAILED`** (Rule 19): A project-wide standing state assigned when a hostile mutant survives the verification matrix (test suite). It acts as an emergency brake that immediately blocks all feature work until the test rigor is repaired.
- **`MaturityScrutiny`** (Rule 25): A strict lockdown remediation protocol/state. It is triggered when the Substrate Integrity Score (SIS) falls below 100, freezing all feature development and quarantining code until a 9-step root-cause repair and artifact regeneration is completed.

## CI Tools / Test Harness Nomenclature
The standing terminology translates into specific outputs in the CI/CD pipeline:

### `maturity_auditor.py`
The test harness evaluates the Substrate Integrity Score (SIS) across four 25-point pillars (Determinism, Behavioral Oracle, Mutation Hostility, Axiomatic Proofs) and categorizes standing as:
- **`PhD-Verified ✅`**: Assigned when a file scores a perfect 100/100.
- **`NEEDS WORK ⚠️`**: Assigned when the file's score is < 100.

### `bcinr-cheat-scanner`
This AST-parsing tool enforces the Anti-Cheat Manifesto. 
- It outputs findings using the format **`CHEAT[rule-id]`** (e.g., `CHEAT[CHEAT-006]`).
- Any finding is classified as an **Absolute Failure**, meaning it bypasses any weighted score, instantly forces `SIS = 0`, and triggers `MaturityScrutiny`.
