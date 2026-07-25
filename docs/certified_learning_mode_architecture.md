# Certified Learning Mode Architecture & ReceiptSound Law (Rule 11)

Under **Rule 11 (ReceiptSound law)** of the BCINR constitution, adaptive mutation of the deterministic substrate is rigorously gated. It requires the logical conjunction of five mandatory cryptographic proofs (receipts):
1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. `CertifiedLearningMode`

No alternate constructors or APIs are permitted. 

## Certified Learning Mode Architecture

In the autonomic substrate, **selection and learning are strictly separated authorities**. `CertifiedLearningMode` provides the explicit authorization required for the system to adapt its internal state, policies, or weights. Without this receipt, learning is not authorized, and the system reverts to a "frozen learning" fallback state.

### The C3 Chain and Proof-Carrying Types
The necessary receipts are generated sequentially by the **C3 (Control, Certification, Commit) Chain** through a linear sequence of branchless validations. Each hop acts as an opaque typestate token, yielding specific sealed receipts. Because adaptive mutation (`AdaptiveUpdate`) is defined as an inductive type with exactly one constructor (`certified`), it is mathematically impossible to instantiate an update without supplying all required proofs.

## How Missing Receipts Freeze Learning (The Fallback Mechanism)

If a required receipt (like `CertifiedLearningMode` or any others) is missing, the system triggers the `ReceiptMissing` typed refusal and enters a **frozen learning** state. The mechanism for this is entirely branchless, adhering strictly to the project's **Radon Law ($CC=1$)**:

1. **Mask Generation:** The C3 Chain evaluates the presence and validity of all required opaque tokens. It derives a bitwise **admission mask** ($m_{\mathrm{admitted}}$). If any required token is missing or if internal digest bindings mismatch, this mask completely collapses to `0`.
2. **Deterministic Commit (Masked State Selection):** The hot path applies the mutation via a fieldwise branchless selection equation, expressly forbidding control flow like `if frozen { ... }`:
   
   $$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

   *(Where $x_t$ is the current state and $x_{\mathrm{candidate}}$ is the proposed mutation).*
3. **Structural Immutability:** When a receipt is missing ($m_{\mathrm{admitted}} = 0$), the bitwise selection structurally defaults to the current state ($x_t$). This ensures the persistent state remains perfectly immutable bit-for-bit without introducing timing side-channels or conditional branches in the hot path.

### Guarantees During Frozen Learning

While learning is frozen, the system gracefully degrades and enforces the following fallback behaviors:
* **Deterministic selection continues:** The system still makes selections and decisions using its current, frozen parameters.
* **Adaptive state remains unchanged:** All adaptive state fields remain completely unchanged.
* **Receipts continue to accumulate:** Telemetry and outcome receipts are still collected.
* **No automatic recertification:** The runtime will not attempt to dynamically discover new theorems or derive new certificates to unfreeze itself in the hot path.
