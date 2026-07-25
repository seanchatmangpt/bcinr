Here is the detailed information on the Substrate Integrity Score (SIS) as defined under Rule 24 in `AGENTS.md`:

### Substrate Integrity Score (SIS) Formula

The Substrate Integrity Score is defined mathematically as:

$$ SIS = 100 - \sum_i w_i V_i $$

where $V_i$ are verified violations and $w_i > 0$.

### Absolute Failures (SIS = 0)

The following conditions are considered absolute failures regardless of the calculated score. Any of these will immediately force **SIS = 0** and trigger `MaturityScrutiny`, as no weighted average may conceal a constitutional violation:

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
