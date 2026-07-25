# Substrate Integrity Score (SIS) and "PhD-Verified" Status

Based on `GEMINI.md` and `AGENTS.md`, here is how the Substrate Integrity Score (SIS) ties into the "PhD-Verified" status of a file, and why a flawless 100/100 is strictly required.

## The Substrate Integrity Score (SIS)
The SIS is defined mathematically in `AGENTS.md` (Section 24) as a baseline of 100 minus a weighted sum of verified violations:
`SIS = 100 - Σ (w_i * V_i)`

However, any "absolute failure" instantly forces the score to `SIS = 0`. Absolute failures include:
- Hidden authoritative branches
- Allocation in the hot path
- Unwitnessed mutation
- Surviving mutants
- Circular oracles
- Scanner evasion
- Stale certificate acceptance
- State mutation after refusal
- Gate-jurisdiction omission
- Fabricated verification evidence

## "PhD-Verified" Status and the Maturity Matrix
According to `GEMINI.md`, a file is only granted "PhD-Verified" status if it scores a perfect **100/100** on the maturity matrix. 

This matrix is built upon three mandatory pillars of verification (enforced by the roles defined in `AGENTS.md`):
1. **Proof (`@hoare_oracle`)**: The file must have formal mathematical contracts, invariants, and typed proof obligations.
2. **Oracle (`@hoare_oracle`)**: An independent, structurally distinct reference implementation, abstract state machine, or symbolic proof must exist to verify the production code.
3. **Hostile Tests (`@armstrong_fault`)**: The implementation must survive adversarial testing, specifically by killing at least three syntactically plausible mutants and returning bounded typed refusals (rather than just failing standard assertions).

## Why 100/100 is Required
A perfect score is mandatory for PhD verification because `bcinr` is engineered to be a deterministic, civilizational-scale "hard substrate" for AGI. The system requires fixed instruction shapes and zero timing side-channels. 

Any score below 100 indicates a constitutional violation or compromised deterministic guarantees. As `AGENTS.md` mandates:
- **"No weighted average may conceal a constitutional violation."**
- **The `MaturityScrutiny` Protocol (Section 25)**: If `SIS < 100`, feature development is immediately frozen, the code is quarantined, and all proofs, scans, mutants, and object-code audits must be rerun from a clean state until the defect is repaired and the score is restored to 100.
- Implementations that merely "appear correct in tests" are strictly refused. Every primitive must be an executable specification where a deviation of even 1 bit causes the verification matrix to fail.

In short, 100/100 is required because the substrate's authoritative runtime does not tolerate partial correctness, implicit branches, or unverified claims.
