# The `ReceiptMissing` Refusal and Structural Evaluation in BCINR

## Structural Evaluation (Radon Law, $CC=1$)
Under the strict anti-branching constraints, the system cannot evaluate missing receipts using traditional control flow (e.g., `if missing { return Err(ReceiptMissing); }`). Instead, it relies on a branchless transaction pipeline:

1. **Receipt Accumulation & Mask Generation**: The C3 (Control, Certification, Commit) Chain evaluates the presence and validity of required opaque tokens (like `CertificateReceipt`, `EnvelopeReceipt`, `OutcomeReceipt`). It derives a bitwise **admission mask** ($m_{\mathrm{admitted}}$). If any required token is missing or if internal digest bindings mismatch, this mask completely collapses to `0`.
2. **The Deterministic Commit (`select`)**: The hot path implements the actual state mutation using a fixed-width `select` operation:
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
   If $m_{\mathrm{admitted}}$ is `0`, the operation deterministically assigns $x_t$ over $x_t$, ensuring the persistent state remains bit-for-bit unchanged.
3. **Yielding the Typed Refusal**: The deterministic function subsequently maps the collapsed mask state to yield the bounded typed refusal code `ReceiptMissing`. This satisfies the constitution's requirement for rejecting uncertified operations securely without utilizing prohibited control flow (like `?`, `match`, or early returns).

## Mathematical Guarantees: The `ReceiptSound` Law (Rule 11)
The `ReceiptSound` law guarantees that all adaptive mutations structurally restrict the type system. As formalized in the system's Lean specifications, adaptive mutation (`AdaptiveUpdate`) is defined as an inductive type with exactly **one** constructor: `certified`. 

A state update is uninhabited (mathematically impossible to construct) unless the caller simultaneously supplies:
- **`CertificateReceipt`**: Proves stability parameters are certified.
- **`EnvelopeReceipt`**: Proves the state digest matches the certificate and the current state is within the admitted envelope.
- **`OutcomeReceipt`**: Provides the specific observed yield and standing.
- **`AdmittedControlState`** and a proof that the system is in `certifiedLearning` mode.

By embedding these requirements directly into the only available constructor, an adaptive transition cannot be instantiated mathematically or in the Rust isomorphic implementation without the complete set of valid receipts.
