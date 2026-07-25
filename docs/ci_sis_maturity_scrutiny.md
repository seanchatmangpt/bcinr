Here is the documentation regarding the Substrate Integrity Score (SIS) rules, absolute failures, and the `MaturityScrutiny` protocol based on `AGENTS.md` and `docs/ci_maturity_scrutiny.md`:

# Substrate Integrity Score (SIS)

The Substrate Integrity Score (SIS) mathematically reflects the completeness of axiomatic proofs, behavioral oracles, mutation hostility, and determinism. A feature only achieves "PhD-Verified" standing when it achieves an SIS of 100/100. 

## Computing SIS
The SIS is formulated as a weighted calculation of violations:

$$
\text{SIS} = 100 - \sum_i w_i V_i
$$

Where:
- $V_i$ represents verified violations.
- $w_i > 0$ represents the weighting of those violations.

However, no weighted average may conceal a constitutional violation. The occurrence of any **Absolute Failure** immediately forces **`SIS = 0`**, overriding the formula and blocking any merge. 

## Absolute Failures
An absolute failure is triggered regardless of the current score if any of the following occur:
- Hidden authoritative branch (JCC, conditional jumps, loop backedges, panics)
- Allocation in the hot path
- Unwitnessed state mutation
- Surviving adversarial mutant
- Circular oracle
- Scanner evasion
- Stale certificate acceptance
- State mutation after refusal
- Gate-jurisdiction omission
- Fabricated verification evidence (e.g., claiming verification without mechanical proof)

## MaturityScrutiny Protocol
When `SIS < 100` (or when an absolute failure forces `SIS = 0`), the repository enters a strict lockdown known as the `MaturityScrutiny` protocol. Agents are strictly forbidden from working around the failure by moving the feature elsewhere. The following 9-step remediation process must be followed:

1. **Freeze feature development:** All feature work is halted across the tree.
2. **Quarantine affected code:** Unverified or structurally non-compliant logic must be physically isolated.
3. **Identify all reachable authoritative symbols:** Trace the transitive call graph for the scope of the violation.
4. **Rerun proofs, scans, mutants, and disassembly:** Establish baseline reproducible evidence.
5. **Produce a root-cause report:** Document the breakdown in the axioms.
6. **Repair the structural defect:** Remediate the breach via branchless fixed-point mathematics and full-width bit masks.
7. **Regenerate all dependent artifacts:** Rebuild dependencies, manifests, and signatures.
8. **Rerun the complete gate matrix:** Execute `cargo make ci` from scratch to verify 0 remaining failures.
9. **Issue a new standing receipt:** Publish a fresh, signed receipt proving compliance.
