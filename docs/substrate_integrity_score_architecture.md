# Substrate Integrity Score (SIS) Architecture

The **Substrate Integrity Score (SIS)** is the deterministic computational substrate's primary governance metric. It mathematically reflects the completeness of axiomatic proofs, behavioral oracles, mutation hostility, and determinism. A feature only achieves "PhD-Verified" standing when it achieves an SIS of 100/100.

## 1. SIS Calculation Formula

The SIS is formulated as a weighted deduction of verified violations from a perfect score:

$$
\text{SIS} = 100 - \sum_i w_i V_i
$$

Where:
- $V_i$ represents verified violations.
- $w_i > 0$ represents the weighting/severity of those violations.

No weighted average may conceal a constitutional violation. If an absolute failure occurs, the formula is overridden entirely.

## 2. Absolute Failures (SIS = 0)

The following violations are considered constitutional breaches. The occurrence of any of these immediately forces **`SIS = 0`**, regardless of the weighted score, and blocks merge capabilities:

1. **Hidden authoritative branch:** Any conditional jumps, JCC, loop backedges, panics, or non-deterministic paths.
2. **Allocation in the hot path:** Any heap allocation bypassing the `no_std` zero-allocation boundaries.
3. **Unwitnessed state mutation:** Any speculative or partial state mutation prior to complete admission and masking.
4. **Surviving adversarial mutant:** Any mutant introduced by `@armstrong_fault` that is not detected by a typed refusal or independent oracle.
5. **Circular oracle:** Providing a reference implementation/oracle that simply copies or relies on the production logic instead of being independent.
6. **Scanner evasion:** Altering formatting or using macro obfuscation specifically to bypass the cheat scanner.
7. **Stale certificate acceptance:** Missing validation checks on deterministic certificates.
8. **State mutation after refusal:** A rejected operation leaving persistent state modified instead of bit-for-bit unchanged.
9. **Gate-jurisdiction omission:** Reporting passing CI/scanner metrics that did not actually run across the relevant target, feature set, or generated output.
10. **Fabricated verification evidence:** Repeated boilerplate comments asserting verification without a linked proof or receipt artifact.

## 3. The MaturityScrutiny Protocol (Rule 25)

Whenever `SIS < 100` (due to either accumulated violations or an absolute failure forcing 0), the repository triggers the **`MaturityScrutiny` protocol**.

Agents may not work around a failed gate by moving the feature elsewhere. Instead, they must execute the following 9-step remediation process:

1. **Freeze feature development:** All unrelated feature work halts.
2. **Quarantine affected code:** Unverified or structurally non-compliant code is physically isolated.
3. **Identify all reachable authoritative symbols:** The transitive call graph must be traced to find the full scope of the violation.
4. **Rerun proofs, scans, mutants, and disassembly:** Establish baseline reproducible evidence of the breakdown.
5. **Produce a root-cause report:** Document precisely how the axioms were breached.
6. **Repair the structural defect:** Remediate via branchless fixed-point mathematics, SWAR construction, or full-width bit masks.
7. **Regenerate all dependent artifacts:** Rebuild dependencies, manifests, and signatures.
8. **Rerun the complete gate matrix:** Execute `cargo make ci` (and all other constitutional gates) from scratch to verify 0 remaining failures.
9. **Issue a new standing receipt:** Publish a fresh, signed receipt proving compliance.
