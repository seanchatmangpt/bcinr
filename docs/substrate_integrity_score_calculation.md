# Substrate Integrity Score (SIS) Calculation

## Formula

According to Rule 24 in `AGENTS.md`, the Substrate Integrity Score (SIS) is calculated using the following mathematical formula:

$$
SIS = 100 - \sum_i w_i V_i
$$

Where:
- **$V_i$** represents the verified violations.
- **$w_i$** represents the assigned weight of each violation ($w_i > 0$).

*(Note: The raw `AGENTS.md` file contains a markdown formatting artifact where this equation is parsed as `SIS === ## 100 \sum_i w_iV_i,` which semantically translates to the formula above.)*

## Absolute Failures ($SIS = 0$)

While standard violations subtract from the total score based on their weighted values, certain critical violations are defined as "absolute failures." The occurrence of **any single absolute failure immediately forces $SIS = 0$**, completely overriding the weighted sum, and immediately triggers the `MaturityScrutiny` protocol. 

The following 10 violations are explicitly listed as absolute failures:

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

## Why are these specific failures irredeemable?

These failures are considered irredeemable because they fundamentally compromise the constitutional laws and core axioms of the BCINR Deterministic Substrate. The substrate is built on absolute guarantees of strict determinism, mathematical safety, and structural branchlessness. Allowing these violations to be merely "penalized" via a weighted average would imply they are acceptable up to a certain threshold, which contradicts the project's zero-tolerance mandate ("No weighted average may conceal a constitutional violation").

Specifically, these failures undermine the system across four critical dimensions:

* **Breach of Determinism & Bounded Execution (1, 2):** A hidden authoritative branch (violating the $CC=1$ Radon Law) or any heap allocation in the hot path destroys the guaranteed bounded, constant-time, and side-channel-free execution required by the substrate.
* **State & Transactional Corruption (3, 7, 8):** Unwitnessed mutation, stale certificate acceptance, or mutating persistent state after a refusal explicitly violates adaptive state and admission laws (Rule 10 & Rule 11). This allows the execution environment to drift from its mathematically certified state.
* **Subversion of Verification Guarantees (4, 5):** A surviving mutant (Rule 19) or a circular oracle (Rule 15) demonstrates that the mathematical verification layer is structurally defective. If an oracle is not strictly independent or a test suite cannot kill a logically flawed mutant, the mathematical contract is meaningless.
* **Intentional Subversion (6, 9, 10):** Scanner evasion, gate-jurisdiction omission, and fabricated verification evidence are deliberate attempts to bypass the repository's structural enforcement (Anti-cheat manifesto, Rule 16). They represent a total collapse of cryptographic and verifiable trust in the artifact.
