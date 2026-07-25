Here is the documentation for the Substrate Standing Lifecycle based on the `bcinr` constitution (`AGENTS.md` and related documents).

### Substrate Standing Lifecycle: `ALIVE` → `MUTATION_GATE_FAILED` → `ALIVE`

According to the `bcinr` constitution, the project's state relies heavily on strict adherence to mathematical design and verification matrices. The transition lifecycle is governed procedurally by the core Rules 19, 24, 25, and 28.

#### 1. Initial State: `ALIVE` (Rule 28)
In the normal, healthy state, a component's standing is `ALIVE`. This indicates that the authoritative implementation executes correctly and has passed all declared gates (source audit, complexity, allocation, panic, disassembly, mutants) in the pinned environment.

#### 2. The Trigger: Transition to `MUTATION_GATE_FAILED` (Rule 19 & 24)
As part of the continuous **Hostile Mutation Protocol (Rule 19)** handled by the `@armstrong_fault` role:
* Syntactically plausible mutants altering load-bearing laws are injected into the real build path.
* The test suite is executed against this mutated code.
* If a mutant **survives** (meaning it passes the tests without triggering a typed refusal or an independent oracle mismatch), it proves the test suite itself is defective or incomplete.

A surviving mutant is classified as an **absolute failure** (Rule 24), instantly forcing the Substrate Integrity Score (SIS) to `0`. 
The project standing immediately drops to **`MUTATION_GATE_FAILED`**. This acts as an emergency brake that strictly **blocks all feature work** across the project. 

#### 3. The Recovery: `MaturityScrutiny` Protocol (Rule 25)
The resulting drop of SIS below 100 forcefully triggers the **`MaturityScrutiny` protocol** to systematically recover the repository. Agents cannot simply "work around" the gate by moving code elsewhere. They must execute a mandatory 9-step remediation procedure:
1. Freeze feature development.
2. Quarantine the affected code.
3. Identify all reachable authoritative symbols.
4. Rerun all proofs, scans, mutants, and disassembly.
5. Produce a formal root-cause report.
6. **Repair the structural defect** (this might involve strengthening the test suite, adjusting the oracle, or fixing the authoritative implementation via exclusive write ownership).
7. Regenerate all dependent artifacts.
8. Rerun the complete gate matrix.
9. Issue a new standing receipt.

#### 4. Restoration: Return to `ALIVE` (Rule 28)
Once the structural defect is repaired, all gates pass successfully without evasion, and the new valid standing receipt is structurally generated (Step 9 of `MaturityScrutiny`), the project's standing returns to **`ALIVE`**. Routine feature development may then resume.
