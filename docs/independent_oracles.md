# The Independence Protocol: Oracles in BCINR

In the BCINR Deterministic Substrate, the mathematical integrity of the runtime is governed by **@hoare_oracle**. A core tenet of this governance is the **Independent Oracle Law** and the **Mandatory Decomposition Protocol**, which strictly prohibit self-certification and dictate what qualifies as a valid oracle for testing and proofing.

## 1. What is an "Independent Oracle"?

An independent oracle serves as the axiomatic reference for any production implementation. To be deemed "independent," an oracle cannot merely live in `tests/reference.rs`; it must be **structurally and logically distinct** from the authoritative branchless implementation.

### Permitted Independent Forms
According to the BCINR Constitution, a valid independent oracle must take one of the following forms:
- **SAT/SMT Bit-Vector Model**: A structural proof solved mechanically.
- **Arbitrary-Precision Implementation**: Logic calculated using arbitrary precision (e.g., `BigInt` or `BigRational`) rather than fixed-width operations.
- **Abstract State Machine**: A formal state transition definition.
- **Direct Mathematical Formula**: A purely mathematical function definition.
- **Hoare Specification**: Preconditions and postconditions logically bounding the behavior.
- **Symbolic Proof**: A formal verification in a proof assistant.
- **Exhaustive Reduced-Domain Enumerator**: A brute-force correctness verifier over a scaled-down but functionally equivalent domain.

### Review and Governance
- **Ownership**: The oracle must be authored and reviewed by `@hoare_oracle`.
- **No Self-Certification**: The implementation agent (`@von_neumann_bypass`) is strictly prohibited from authoring the final oracle and self-certifying equivalence. Every approval must come from a different role and be backed by a mechanical artifact.

## 2. What is a "Circular Oracle"?

A **circular oracle** (classified as a strict violation under **CHEAT-002**) is an oracle that derives its expected results or structure directly from the production implementation, meaning it proves nothing more than "the code does what the code does."

### Prohibited Circular Patterns
An oracle is automatically rejected as circular if it involves any of the following:
- **Line-by-line translation** of production code.
- **Reuse of production logic**: Including production normalization, lookup tables, or fixed-point helpers.
- **Identical control structure**: e.g., using branching `f64` structures that exactly mirror the branchless structure logic.
- **Importing and wrapping**: Importing the authoritative function and calling it under the guise of an oracle.
- **Deriving test expectations from the implementation under attack**: (`@armstrong_fault` requirement).

## 3. Summary of the Independence Protocol

The overarching philosophy of the Independence Protocol can be summarized as:

1. **Rich semantics upstream, fixed deterministic mechanics downstream.** 
2. The agent implementing the branchless hot path is explicitly isolated from the agent defining the mathematical law.
3. If an implementation diverges from the independent oracle by even 1 bit, the verification matrix must mechanically fail. Human agreement is not evidence.
