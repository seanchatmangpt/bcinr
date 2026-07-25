# Substrate Integrity Score (SIS) Weighting System

Based on Rule 24 in `AGENTS.md` and its concrete enforcement in the test harnesses (specifically `maturity_auditor.py` and `bcinr-cheat-scanner`), here is how the Substrate Integrity Score is mathematically evaluated:

## 1. Theoretical Mathematical Definition
According to `AGENTS.md` (Rule 24), the Substrate Integrity Score is formally defined using a subtractive penalty model:
$$ SIS = 100 - \sum_i w_i V_i $$
Where:
- **$V_i$** represents verified constitutional violations in the codebase.
- **$w_i > 0$** represents the weights assigned to specific violations.

## 2. CI/Test Harness Evaluation (The Weighting System)
In practice, the test harness (`maturity_auditor.py`) calculates the SIS additively out of 100 points. The system assigns a weight of 25 points to four critical pillars of verification. A file must score a perfect **100/100** to earn `"PhD-Verified"` standing. 

The 25-point weights are evaluated as follows:

* **Determinism / Branchlessness (25 pts)**: Validates that there are absolutely zero jump control flow (JCC) operations—such as `if`, `match`, `while`, or `loop`—inside public functions (with exceptions for constructors like `new`).
* **Behavioral Oracle (25 pts)**: Ensures the file explicitly contains an independent reference model (`_reference` or `oracle`) and handles edge cases (`boundaries`).
* **Mutation Hostility (25 pts)**: Confirms the presence of a rigorous mutation matrix, requiring at least 3 distinct counterfactual mutants (`fn mutant_`) and 3 corresponding rejection assertions (`rejects_mutant` or `counterfactual_mutant`).
* **Axiomatic Proofs (25 pts)**: Verifies that formal Hoare-logic proofs (`Hoare`, `Axiomatic`, or `AXIOMATIC`) are documented within the file, while also enforcing a minimum artifact length of ≥ 100 lines.

## 3. Absolute Failures ($SIS = 0$)
`AGENTS.md` strictly dictates: *"No weighted average may conceal a constitutional violation."*

There is a severe class of violations that act as absolute failures. If any of these occur, they bypass the standard weighted calculation and immediately force **$SIS = 0$**. These rules are strictly enforced by the `bcinr-cheat-scanner` CI tool and include:
- Hidden authoritative branches or allocations in the hot path.
- Unwitnessed state mutation or mutating state after a typed refusal.
- Surviving hostile mutants.
- Circular oracles (copying production logic to a test reference) or fabricated verification evidence.
- Scanner evasion (e.g., hiding logic inside macros).

Any score of `SIS < 100` triggers the **MaturityScrutiny protocol**, which freezes development, quarantines the affected component, and mandates a full root-cause repair and artifact regeneration.
