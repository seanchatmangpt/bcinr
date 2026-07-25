# MaturityScrutiny Protocol: Substrate Integrity Score (SIS) Quarantine & Repair

In the BCINR deterministic substrate, the **Substrate Integrity Score (SIS)** is a critical metric for maintaining the absolute runtime laws and ensuring constitutional precedence. 

## The Substrate Integrity Score (SIS)

The SIS is formally defined as:

$$SIS = 100 - \sum_i w_i V_i$$

where $V_i$ represents verified violations and $w_i > 0$ represents their weights.

### Absolute Failures (Forcing SIS = 0)
No weighted average may conceal a constitutional violation. The following constitutional violations are considered **absolute failures**, meaning they bypass the standard weighted calculation and immediately force **`SIS = 0`**, regardless of score. This immediately triggers the `MaturityScrutiny` protocol:

* Hidden authoritative branch
* Allocation in the hot path
* Unwitnessed mutation
* Surviving mutant
* Circular oracle
* Scanner evasion
* Stale certificate acceptance
* State mutation after refusal
* Gate-jurisdiction omission
* Fabricated verification evidence

## MaturityScrutiny: Strict Chronological Quarantine & Repair Process

When the `SIS < 100` (which is guaranteed when an absolute failure occurs and forces it to 0), the `MaturityScrutiny` protocol dictates a strict, nine-step chronological repair process. Agents may not work around a failed gate by moving the feature elsewhere; this process must be followed sequentially before any feature work can resume.

1. **Freeze feature development:** Stop all ongoing feature work immediately.
2. **Quarantine affected code:** Isolate the codebase sections implicated in the absolute failure or violations.
3. **Identify all reachable authoritative symbols:** Trace and document all symbols in the authoritative call graph that can be reached from the quarantined code.
4. **Rerun proofs, scans, mutants, and disassembly:** Execute the complete verification suite against the quarantined and reachable symbols to fully understand the extent of the failure.
5. **Produce a root-cause report:** Formally document the exact cause of the structural defect or violation.
6. **Repair the structural defect:** Implement the fix, ensuring adherence to all absolute runtime laws (e.g., branchless, no-allocation, fixed bounded execution).
7. **Regenerate all dependent artifacts:** Re-create any generated source code, proofs, or documents dependent on the repaired code.
8. **Rerun the complete gate matrix:** Execute all required repository gates (e.g., cheat-scanner, contract-gate, mutants, object-code audit, generated verification) across all supported feature and target configurations.
9. **Issue a new standing receipt:** Once all gates pass, generate a new receipt establishing the repository's standing, officially returning the `SIS` to 100 and allowing feature development to resume.
