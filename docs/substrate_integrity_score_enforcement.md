# Rule 24: Substrate Integrity Score (SIS) Analysis

Based on the exploration of `AGENTS.md`, `maturity_auditor.py`, and `tools/bcinr-cheat-scanner/src/main.rs`, here is how the Substrate Integrity Score (SIS) is calculated and enforced across the `bcinr` codebase.

## 1. How SIS is Calculated
According to `AGENTS.md`, the theoretical formula for SIS is defined subtractively:
`SIS = 100 - Σ (w_i * V_i)`
where `V_i` represents verified violations and `w_i` represents their assigned weights.

In practice within the test harness, the calculation is evaluated additively out of 100 via the `maturity_auditor.py` script, which grades algorithms on four 25-point pillars:
1. **Determinism (25 pts)**: Ensures no jump control flow (JCC) like `if`, `match`, `while`, or `loop` exists in public functions (excluding constructors).
2. **Behavioral Oracle (25 pts)**: Requires the explicit presence of a reference model (`_reference`, `oracle`) and boundary tests (`boundaries`).
3. **Mutation Hostility (25 pts)**: Requires the file to possess at least 3 counterfactual mutants (`fn mutant_`) and 3 robust assertions of refusal/rejection (`rejects_mutant` or `counterfactual_mutant`).
4. **Axiomatic Proofs (25 pts)**: Requires the file to contain axiomatic or Hoare-logic proof documentation (and enforces a length requirement of >= 100 lines).

A file only earns "PhD-Verified" status if it hits a perfect `100/100` threshold.

## 2. Absolute Failures & `SIS = 0`
While the normal score is weighted, `AGENTS.md` explicitly defines a list of **absolute failures** that override any partial scores, immediately forcing `SIS = 0` regardless of the weighted average. 

These absolute failures include:
- Hidden authoritative branches
- Allocation in the hot path
- Unwitnessed mutation or state mutation after refusal
- Surviving mutants
- Circular oracles
- Scanner evasion
- Stale certificate acceptance
- Gate-jurisdiction omission
- Fabricated verification evidence

### The Cheat Scanner Enforcement
The `tools/bcinr-cheat-scanner/src/main.rs` tool acts as the enforcer of these absolute failures. It performs deep AST-level and text-based inspections. For example:
- **CHEAT-002 (Circular Oracle)**: It detects if an oracle's implementation body exactly mirrors the production logic.
- **CHEAT-005 (Boilerplate Verification)**: It flags mock "Hoare-logic" comments that try to fabricate proof presence.
- **CHEAT-006 (Scanner Evasion)**: It catches hidden branches inside macro definitions to prevent bypassing the `CC=1` check.
- **CHEAT-020 (Mutation Before Admission)**: It validates the exact ordering of operations to prevent state mutation prior to full input validation.

If the scanner surfaces *any* of these findings, it immediately aborts the pipeline, effectively enacting the `SIS = 0` rule.

## 3. The MaturityScrutiny Protocol
When `SIS < 100` or an absolute failure sets `SIS = 0`, the system demands a **MaturityScrutiny** protocol which requires agents to:
1. Freeze feature development and quarantine the affected code.
2. Produce a root-cause report and resolve the structural defect.
3. Rerun proofs, scanners, disassembly analysis, and regenerate dependent artifacts.
4. Pass the complete gate matrix again before issuing a new standing receipt.
