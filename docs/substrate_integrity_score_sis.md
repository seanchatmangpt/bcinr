# Substrate Integrity Score (SIS) and MaturityScrutiny

## 1. How SIS is Calculated
The Substrate Integrity Score (SIS) is calculated using the following formula:
$$ SIS = 100 - \sum_i (w_i \times V_i) $$
Where:
- $V_i$ represents verified violations.
- $w_i > 0$ represents the weights assigned to those violations.

The score starts at a perfect 100 and is reduced by the weighted sum of any verified violations.

## 2. Why Absolute Failures Force `SIS = 0`
While typical verified violations simply reduce the SIS via a weighted average, certain defects represent fundamental breakdowns in the core architectural laws (the deterministic, branchless, allocation-free, and mathematically bounded nature of `bcinr`). 

The following defects are considered absolute failures:
- hidden authoritative branch
- allocation in the hot path
- unwitnessed mutation
- surviving mutant
- circular oracle
- scanner evasion
- stale certificate acceptance
- state mutation after refusal
- gate-jurisdiction omission
- fabricated verification evidence

Any of these failures instantly forces **`SIS = 0`**, regardless of the current score. No weighted average may conceal a constitutional violation because these are catastrophic failures of the core runtime laws. A single breach of these fundamental invariants compromises the mathematical correctness and integrity of the deterministic computational substrate.

## 3. The MaturityScrutiny Quarantine Process
When the Substrate Integrity Score falls below 100 (`SIS < 100`), the **MaturityScrutiny** protocol is immediately triggered. This quarantine process entails the following required steps:

1. **Freeze feature development:** Stop all ongoing feature work.
2. **Quarantine affected code:** Isolate the failing implementation.
3. **Identify all reachable authoritative symbols:** Map out the exact affected paths.
4. **Rerun proofs, scans, mutants, and disassembly:** Re-verify the substrate logic.
5. **Produce a root-cause report:** Document the exact failure reason.
6. **Repair the structural defect:** Implement the branchless/bounded fix.
7. **Regenerate all dependent artifacts:** Rebuild dependencies and proofs.
8. **Rerun the complete gate matrix:** Execute all audits against all targets and features.
9. **Issue a new standing receipt:** Restore standing once repaired.

*Important Note:* Agents may not attempt to work around a failed gate by moving the feature elsewhere. The structural defect must be repaired in place.
