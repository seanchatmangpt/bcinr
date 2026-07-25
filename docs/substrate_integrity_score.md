# Substrate Integrity Score (SIS)

According to Rule 24 of the `bcinr` constitution, the Substrate Integrity Score is defined mathematically as:

$$ SIS = 100 - \sum_i w_i V_i $$

where $V_i$ are verified violations and $w_i > 0$.

## Immediate `SIS = 0` Violations

Regardless of the weighted average, the following constitutional violations are considered absolute failures and force an immediate `SIS = 0` (which subsequently triggers the `MaturityScrutiny` protocol):

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

No weighted average may conceal a constitutional violation.
