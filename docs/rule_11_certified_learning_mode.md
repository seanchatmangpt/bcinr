Here is the research on `CertifiedLearningMode` and its role in adaptive mutation based on `AGENTS.md`.

According to **Rule 11 (ReceiptSound law)** of the `AGENTS.md` constitution, `CertifiedLearningMode` is a mandatory proof component for any adaptive mutation to take place. 

### Role in Adaptive Mutation

For adaptive mutation to occur, the runtime strictly requires the logical conjunction of five proofs, with no alternate constructors or APIs permitted:
1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. **`CertifiedLearningMode`**

**Key Details on `CertifiedLearningMode`:**
* **Separation of Concerns**: The constitution dictates that "Selection and learning are separate authorities." `CertifiedLearningMode` acts as the explicit authorization that learning is active and permitted, decoupled from the selection process.
* **Frozen Learning Fallback**: When `CertifiedLearningMode` is not active (i.e., learning is "frozen"):
  * All adaptive state fields must remain unchanged.
  * Deterministic selection is allowed to continue.
  * Receipts may continue to accumulate.
  * No automatic recertification is permitted in the hot path.
* **Branchless Enforcement**: In accordance with the project's core laws (e.g., Radon Law, $CC=1$), the fallback mechanism when learning is frozen must be implemented strictly via **masked state selection**, rather than control flow branching (e.g., `if` statements).
* **Admitted Evidence**: This ties into the broader constitutional mandate (stated later in the file) that *"No adaptive mutation may occur without admitted evidence."* `CertifiedLearningMode` forms a critical part of that required evidence bundle.
