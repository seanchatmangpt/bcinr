# Rule 15: Independent Oracle Law

In the BCINR deterministic computational substrate, **Rule 15 (Independent Oracle Law)** mandates that any test oracle or reference implementation must be an entirely independent and mathematically pure expression of the domain logic. An oracle is not considered independent simply by being housed in a separate file (e.g., `tests/reference.rs`). 

## Why Structurally and Logically Distinct Oracles are Required

The fundamental purpose of an oracle in BCINR is to serve as the undeniable mathematical truth against which the highly optimized, branchless, allocation-free production code is verified. The oracle must be structurally and logically distinct for the following reasons:

1. **Preventing Tautological Verification (CHEAT-002)**: If an oracle shares the same structure or logic as the production code, it becomes a "circular oracle." It will replicate the same assumptions, oversights, and logic flaws present in the implementation. Tests will pass, but they will only prove that the code equals itself, offering a false sense of security and corrupting the Substrate Integrity Score (SIS).
2. **Separation of Concerns**: Production code in BCINR is constrained by extreme architectural laws (e.g., whole-call-graph branchlessness, `CC=1`, bit-parallel mechanics over byte-sequential control flow). This often requires convoluted mechanisms like SWAR, bit-masking, and fixed-width state transitions. The oracle, however, must focus purely on the *mathematical contract* (Hoare specification, pre/postconditions) without concerning itself with runtime constraints.
3. **Rigorous Domain Coverage**: By utilizing permitted independent forms such as **SAT/SMT bit-vector models**, **symbolic proofs**, or **arbitrary-precision implementations**, the oracle can mathematically evaluate constraints across the entire domain. For instance, an SMT solver can mathematically prove that a complex bitwise mask operation perfectly matches a straightforward mathematical piecewise function across all $2^{64}$ possible inputs, achieving certainty that runtime tests cannot.

## Why Line-by-Line Translation and Resource Reuse is Prohibited

Rule 15 explicitly forbids line-by-line translation of production code, identical control structures (even if evaluated with `f64`), and the reuse of production tables, normalizations, or fixed-point helpers.

1. **Shared Corruptions in Constants and Tables**: If the production lookup table or fixed-point helper contains a derivation error, a typo, or an invalid clamp constant, reusing that table in the oracle guarantees that the oracle will mirror the error. Both the implementation and the test will agree on an incorrect result. The oracle must independently derive its constants and truth values.
2. **Inheritance of Logical Flaws**: A line-by-line translation—such as rewriting a production Rust function into another language or writing a branching version that mimics the exact same algorithmic steps—inherits the same blind spots. If the implementation improperly handles a negative-domain edge case or arithmetic overflow, the translated oracle will likely do the same.
3. **Role Independence Protocol**: Under the mandatory decomposition protocol (Rule 5), the oracle is owned by `@hoare_oracle` (Axiomatic proof lead), whereas the implementation is owned by `@von_neumann_bypass` (Architect of Arithmetic Logic). Reusing production assets violates this strict separation of powers. The implementation must be verified against an independently formulated truth, not a mirrored reflection of its own logic.
