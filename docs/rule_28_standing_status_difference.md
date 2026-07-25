Based on the `AGENTS.md` constitution (specifically Rules 19, 24, 25, and 28), here is the explicit difference between `MUTATION_GATE_FAILED` and `MaturityScrutiny`:

### 1. Categorical Difference (State vs. Protocol)
* **`MUTATION_GATE_FAILED`** is a **project standing state**. It is a specific failure label assigned to the project when a hostile mutant survives the verification matrix (Rule 19).
* **`MaturityScrutiny`** is a **remediation protocol**. It is a mandatory 9-step operational procedure executed to recover the repository when the system's integrity is compromised (Rule 25).

### 2. Triggers
* **`MUTATION_GATE_FAILED`** is triggered *exclusively* by a surviving mutant in the hostile verification workstream.
* **`MaturityScrutiny`** is triggered *broadly* whenever the Substrate Integrity Score (SIS) falls below 100. This includes surviving mutants (which are an "absolute failure" forcing SIS = 0), but also encompasses many other violations like hidden branches, hot-path allocations, or scanner evasion (Rule 24).

### 3. Immediate Consequences vs. Recovery Steps
* **`MUTATION_GATE_FAILED`** acts as an emergency brake. Its explicit directive is that it immediately "blocks all feature work."
* **`MaturityScrutiny`** dictates *how* to handle the broken state. It mandates a rigid recovery process including: freezing development, quarantining code, running a root-cause analysis, repairing structural defects, regenerating artifacts, and finally issuing a new standing receipt.

### 4. Rule 28 Vocabulary Distinction
Rule 28 establishes a strict "Standing vocabulary" of bounded labels (e.g., `PROVEN`, `ALIVE`, `REFUSED`, `UNKNOWN`, `BUILD_BROKEN`). Notably, `MUTATION_GATE_FAILED` is absent from this allowed 10-label roster. This implies that while the labels in Rule 28 describe the maturity and verification level of valid, tracked code, `MUTATION_GATE_FAILED` serves as an overarching, system-halting exception state rather than a standard component lifecycle label.
