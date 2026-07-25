# Rule 24: Substrate Integrity Score (SIS) and Rule 25: MaturityScrutiny

Based on `AGENTS.md` in `bcinr`, here is an explanation of the Substrate Integrity Score (Rule 24) and the MaturityScrutiny protocol (Rule 25).

## The Substrate Integrity Score (SIS) Equation

The Substrate Integrity Score (SIS) is defined mathematically as:

$$
SIS = 100 - \sum_i w_i V_i
$$

Where:
- $V_i$ represents verified violations.
- $w_i > 0$ represents the weight assigned to each violation.

## Absolute Failures ($SIS = 0$)

A weighted average cannot conceal constitutional violations. The following 10 conditions are considered **absolute failures** and instantly force the $SIS = 0$, regardless of the score calculation:

1. Hidden authoritative branch
2. Allocation in the hot path
3. Unwitnessed mutation
4. Surviving mutant
5. Circular oracle
6. Scanner evasion
7. Stale certificate acceptance
8. State mutation after refusal
9. Gate-jurisdiction omission
10. Fabricated verification evidence

Any of these absolute failures immediately triggers the `MaturityScrutiny` protocol.

## MaturityScrutiny Protocol (9-Step Process)

When the $SIS < 100$, the following strict 9-step quarantine and repair process is enforced. Agents may not work around a failed gate by moving the feature elsewhere.

1. **Freeze feature development**
2. **Quarantine affected code**
3. **Identify all reachable authoritative symbols**
4. **Rerun proofs, scans, mutants, and disassembly**
5. **Produce a root-cause report**
6. **Repair the structural defect**
7. **Regenerate all dependent artifacts**
8. **Rerun the complete gate matrix**
9. **Issue a new standing receipt**
