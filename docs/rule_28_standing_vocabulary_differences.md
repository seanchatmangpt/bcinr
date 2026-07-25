Based on the `AGENTS.md` Constitution, here is the explanation of the differences in standing between `ALIVE`, `MUTATION_GATE_FAILED`, and the state of `MaturityScrutiny`:

### `ALIVE` (Rule 28: Standing vocabulary)
This standing indicates that the implementation successfully executes and passes all declared gates in the pinned environment. It means the code is currently passing its routine structural and operational checks.

### `MUTATION_GATE_FAILED` (Rule 19: Hostile mutation protocol)
This standing occurs when a hostile mutant (a deliberate syntactic or logical flaw injected into the code to test the robustness of the contract enforcement) survives the test suite instead of triggering a typed refusal or oracle mismatch. Reaching this standing means the test suite is fundamentally defective and immediately blocks all feature work. 

### `MaturityScrutiny` (Rule 24 & 25)
This is an emergency protocol state triggered when the Substrate Integrity Score (SIS) drops below 100, or when a constitutional "absolute failure" occurs (such as a surviving mutant, hot path allocation, or hidden branches), forcing the SIS to 0. 
In the `MaturityScrutiny` state:
- Feature development is completely frozen.
- Affected code is quarantined.
- A rigorous, multi-step recovery process begins, requiring developers to identify the root cause, repair the structural defect, regenerate all artifacts, and rerun the complete gate matrix before a new standing receipt can be issued.

**Summary**: 
`ALIVE` represents normal passing health, `MUTATION_GATE_FAILED` represents a critical failure in test rigor, and `MaturityScrutiny` is the lockdown remediation protocol invoked to recover from severe substrate failures.
