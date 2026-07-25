# Substrate Integrity Score (SIS) Mechanics

Based on Rule 24 in the `AGENTS.md` Deterministic Substrate Constitution, the Substrate Integrity Score (SIS) measures the structural maturity and compliance of code within the BCINR project. 

## The Formula: $SIS = 100 - \sum_i w_iV_i$

The baseline score for an implementation starts at a perfect **100**. Points are deducted based on verified violations ($V_i$) weighted by their severity ($w_i$). 

- **$V_i$ (Verified Violations):** These represent specific infractions or deviations from the constitutional rules, found through structural audits, cheat scanners, or mutant testing. 
- **$w_i > 0$ (Weights):** Each violation type has a positive weight corresponding to its severity. 

The system calculates the impact of infractions as a weighted deduction, reducing the score proportionally from the ideal 100.

## Absolute Failures (The "Zero Score" Triggers)

Certain violations are considered constitutional breaches. These are completely incompatible with the deterministic, authoritative goals of the runtime. If any of these are present, the weighted sum formula is bypassed, and the SIS is forced to **0**, instantly triggering the `MaturityScrutiny` protocol. 

The absolute failures are:

1. **Hidden authoritative branch:** Any conditional control flow branching within the authoritative call graph (violating the strict $CC=1$ rule).
2. **Allocation in the hot path:** Using heap allocation in performance-critical execution, breaching the zero-allocation boundary.
3. **Unwitnessed mutation:** Modifying state without correctly verifying required predicates and applying full-width masks.
4. **Surviving mutant:** Failing to kill a hostile mutant during adversarial testing, revealing incomplete verification.
5. **Circular oracle:** Providing a reference implementation that was simply copied from the production code rather than acting as a structurally distinct mathematical specification.
6. **Scanner evasion:** Using macros, formatting, or other obfuscation to hide prohibited code from the `bcinr-cheat-scanner`.
7. **Stale certificate acceptance:** Executing an adaptive mutation or state transition using expired, uncertified, or invalid receipts.
8. **State mutation after refusal:** Failing to leave persistent state bit-for-bit unchanged when an operation is rejected.
9. **Gate-jurisdiction omission:** Excluding changed files, generated output, or specific targets from the required structural gates, thereby bypassing inspection.
10. **Fabricated verification evidence:** Falsifying reports, command outputs, verification artifacts, or manually editing generated outputs.

## The Principle: No Concealment

The fundamental rule states: **No weighted average may conceal a constitutional violation.**

The BCINR project enforces that the authoritative runtime must remain bounded, branchless, deterministic, and allocation-free. If a weighted average were allowed to absorb major infractions—for instance, an otherwise perfect implementation scoring a 90/100 despite having a hidden branch—the foundational guarantees of the hard substrate would be compromised. The mathematical contract of the system is absolute. A high overall score cannot be used to mask a fundamental breach of this architecture; therefore, absolute failures enforce strict compliance via an automatic $SIS = 0$.
