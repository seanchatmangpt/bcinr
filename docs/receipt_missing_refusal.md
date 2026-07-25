# The `ReceiptMissing` Refusal and Structural Evaluation in BCINR

## 1. What is the `ReceiptMissing` Refusal?

In the `bcinr` deterministic substrate, the `ReceiptMissing` typed refusal is triggered when an authoritative operation attempts to perform an adaptive state mutation without providing all the cryptographically bound proofs (receipts) mandated by the mathematical contract.

## 2. The C3 Chain and Authority Hops

The necessary receipts are generated sequentially by the **C3 (Control, Certification, Commit) Chain**. This is a linear, unforgeable sequence of branchless validations that governs state admission. Each "hop" acts as an opaque typestate token, producing specific sealed receipts:
- **Hop 2 (Shadow Execution):** Yields a `ShadowExecutionReceipt`.
- **Hop 3 (Jump Analysis):** Yields a `JumpAnalysisReceipt`.
- **Hop 5 (Certification):** Mints the core `CertificateReceipt`, evaluating temporal dwell (`DwellSatisfied`) and stability witness parameters.
- **Hop 6 (Mode Switch / Commit):** The actuating step, which structurally requires the compiled cryptographic evidence of the previous hops.

There are no public constructors for these receipts—they are exclusively minted by their exact, authorized logic.

## 3. Structural Evaluation Without Branching (Radon Law, $CC=1$)

Under the strict anti-branching constraints (Radon Law, $CC=1$), the system cannot evaluate missing receipts using traditional control flow like `if missing { return Err(ReceiptMissing); }`. Instead, it relies on a branchless transaction pipeline:

1. **Receipt Accumulation & Mask Generation**: The C3 Chain evaluates the presence and validity of the required opaque tokens (e.g., `CertificateReceipt`, `EnvelopeReceipt`, `OutcomeReceipt`). It derives a bitwise **admission mask** ($m_{\mathrm{admitted}}$). If any required token is missing or if the internal digest bindings mismatch, this mask completely collapses to `0`.
2. **The Deterministic Commit (`select`)**: The hot path implements the actual state mutation using a fixed-width `select` operation:
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
   If $m_{\mathrm{admitted}}$ is `0`, the operation deterministically assigns $x_t$ over $x_t$, ensuring the persistent state remains bit-for-bit unchanged without needing an early `return` or branch.
3. **Yielding the Typed Refusal**: The deterministic function subsequently maps the collapsed mask state to yield the bounded typed refusal code `ReceiptMissing`. This satisfies the constitution's requirement for rejecting uncertified operations securely without utilizing prohibited control flow (like `?`, `match`, or early returns).

## 4. Mathematical Guarantees: The `ReceiptSound` Law (Rule 11)

The `ReceiptSound` law guarantees that all adaptive mutations structurally restrict the type system. As formalized in the system's Lean specifications, adaptive mutation (`AdaptiveUpdate`) is defined as an inductive type with exactly **one** constructor: `certified`.

Because `certified` is the *only* constructor, it acts as a proof-carrying type. A state update is uninhabited (mathematically impossible to construct) unless the caller simultaneously supplies:
- **`CertificateReceipt`**: Proves stability parameters are certified and binds to a specific `modeId` and `controlDigest`.
- **`EnvelopeReceipt`**: Proves the state digest matches the certificate and the current state is within the admitted envelope.
- **`OutcomeReceipt`**: Provides the specific observed yield and standing that trigger the mutation.
- **`AdmittedControlState`** and a proof that the system is in `certifiedLearning` mode.

By embedding these requirements directly into the only available constructor—and processing the typestates through the C3 chain into branchless bitwise polynomials—the `ReceiptSound` law guarantees inescapability. An adaptive transition cannot be instantiated in the Lean mathematical model or the Rust isomorphic implementation without the complete set of valid, cryptographically sound receipts.
