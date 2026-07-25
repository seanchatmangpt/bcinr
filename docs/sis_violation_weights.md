# Substrate Integrity Score (SIS) Violation Weights

The Substrate Integrity Score (SIS) mathematically reflects the completeness, mathematical rigor, and strict determinism of the codebase. The repository constitution (`AGENTS.md`) defines the core formula as:

$$ \text{SIS} = 100 - \sum_i w_i V_i $$

where $V_i$ represents verified violations and $w_i > 0$ represents their respective penalty weights.

## Mathematical Application and MaturityScrutiny

The SIS begins at a baseline perfect score of 100. During the CI pipeline (e.g., `cargo make ci`), for every verified violation ($V_i$) found in the codebase, the score is reduced by its corresponding weight ($w_i$). 

According to the constitution, a component is only considered "PhD-Verified" and admissible if it maintains a perfect score of **100/100**. Therefore, mathematically, **any** non-absolute violation (since $w_i > 0$) will drop the score below 100 ($SIS < 100$). 

The moment the score falls below 100, the **MaturityScrutiny** protocol is immediately triggered. This strictly mandates a 9-step chronological quarantine and repair process (freezing feature development, tracing call graphs, producing root-cause reports, and repairing the defect) before development can resume.

*(Note: "Absolute Failures" bypass this summation entirely. Constitutional violations such as hidden branches, hot-path allocations, surviving mutants, circular oracles, and scanner evasion immediately force **$SIS = 0$**, regardless of the weighted sum).*

## Non-Absolute Violations ($V_i$)

Non-absolute violations ($V_i$) encompass the remaining structural and testing rules enforced by the `bcinr-cheat-scanner` and repository gates that do not trigger an immediate 0 score. 

While the constitution provides the algebraic framework ($w_i > 0$), the repository **does not currently tabulate explicit, distinct numerical weights ($w_i$)** for these non-absolute violations. Because the required passing score is exactly 100, any single infraction deducts from the score and functionally acts as a strict blocking gate.

Based on the Anti-Cheat Manifesto (Rule 16) and the `bcinr-cheat-scanner` implementation, non-absolute violations include:

1. **CHEAT-001 (Self-Canceling Operations):** Using operations that artificially inflate complexity without altering behavior (e.g., `A ^ A` or `A - A`).
2. **CHEAT-003 (Magic Constants):** Hardcoding unexplained configuration literals (e.g., `0xDEADBEEF`) instead of using derived or certified constants.
3. **CHEAT-004 (Artificial File Inflation):** Padding files with repetitive comments or dead code to meet arbitrary length expectations.
4. **CHEAT-005 (Boilerplate Verification Claims):** Pasting mock "Hoare-logic verification" comments without real axiomatic proof linkages.
5. **CHEAT-007 (Dead Path Compliance):** Placing compliant, branchless code in unreachable blocks (e.g., `if false { ... }`) while the active path remains unlawful.
6. **CHEAT-008 (Benchmark Theater):** Executing benchmarks without consuming outputs (e.g., missing `core::hint::black_box`), allowing the compiler to optimize the logic away.
7. **CHEAT-009 (Mutant Theater):** Writing adversarial tests with weak assertions (`assert_ne!`) instead of asserting exact, typed refusal codes (e.g., `Err(StabilityRefusal::)`).
8. **CHEAT-021 (Rejection State Drift):** Failing to include `test_rejection_invariance` checks to ensure rejected transactions leave persistent state bit-for-bit unchanged.
9. **CHEAT-031 (Black Box Claims):** Invalidly asserting in documentation that `black_box` guarantees machine-level branchlessness (which requires object-code disassembly audits to prove).
