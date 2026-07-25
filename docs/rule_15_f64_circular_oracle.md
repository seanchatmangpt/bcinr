# Analysis of Rule 15 and CHEAT-002

Based on the `AGENTS.md` repository constitution, here is an analysis of Rule 15 (Independent oracle law), CHEAT-002 (Circular oracle), and why an "identical control structure with f64" is explicitly prohibited.

## Rule 15: Independent Oracle Law
This rule mandates that every authoritative implementation must be verified by a mathematically independent oracle. Simply placing code in `tests/reference.rs` does not qualify it as an independent oracle. To be valid, the oracle must be **structurally and logically distinct** from the production code. Permitted independent forms include direct mathematical formulas, symbolic proofs, SMT bit-vector models, or abstract state machines. 

## CHEAT-002: Circular Oracle
A circular oracle is defined as "A reference implementation copied from the production implementation." It is classified as an anti-cheat violation because it defeats the entire purpose of having an adversarial, independent verification step.

## Why an "identical control structure with f64" is banned
An oracle that merely uses the exact same control flow as the production code but swaps out the underlying arithmetic types (e.g., replacing fixed-point integers with `f64` floating-point numbers) is explicitly banned for the following reasons:

1. **Failure of Independence:** It violates the core mandate of Rule 15 that the oracle must be "structurally and logically distinct." It is essentially just a typed clone of the production code rather than an independent mathematical derivation.
2. **Inherited Flaws (Circular Logic):** If the production implementation contains a structural defect, algorithmic error, or logical flaw in its control flow, the `f64` oracle will blindly inherit that exact same flaw. The test suite will trivially pass because both algorithms make the identical mistake, rendering the verification entirely useless.
3. **Self-Certification:** The constitution strictly prohibits an implementation owner from self-certifying equivalence (Rules 5 and 27). Creating an oracle by just copying the implementation's logic and changing the numeric types is a form of self-certification, disguised as an independent test. 

By explicitly banning this pattern as a circular oracle, `bcinr` ensures that the verification process relies on a separate derivation of mathematical truth, preventing false confidence born from testing a codebase against a mirror of itself.
