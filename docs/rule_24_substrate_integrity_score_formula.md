# Rule 24: Substrate Integrity Score (SIS)

Rule 24 in `AGENTS.md` establishes the **Substrate Integrity Score (SIS)** as the constitutional metric for evaluating codebase adherence to the BCINR deterministic substrate laws.

## The Mathematical Formula

The base Substrate Integrity Score is defined by the following mathematical formula:

$$SIS = 100 - \sum_i w_i V_i$$

Where:
* **$100$**: The baseline score representing a perfect, violation-free state.
* **$V_i$**: Represents individual verified violations within the code.
* **$w_i$**: Represents the weight or penalty assigned to a specific violation (where $w_i > 0$).

The score is calculated by subtracting the weighted sum of all verified violations from the baseline score of 100.

## Absolute Failures

While the formula allows for weighted deductions, Rule 24 strictly mandates that **no weighted average may conceal a constitutional violation**. 

The presence of any of the following **absolute failures** bypasses the formula entirely, forcing an immediate **$SIS = 0$** and triggering the `MaturityScrutiny` protocol (Rule 25):

1. **Hidden authoritative branch**
2. **Allocation in the hot path**
3. **Unwitnessed mutation**
4. **Surviving mutant**
5. **Circular oracle**
6. **Scanner evasion**
7. **Stale certificate acceptance**
8. **State mutation after refusal**
9. **Gate-jurisdiction omission**
10. **Fabricated verification evidence**
