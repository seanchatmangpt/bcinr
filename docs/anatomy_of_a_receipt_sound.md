# Anatomy of a ReceiptSound

According to Rule 11 (the `ReceiptSound` law) in the BCINR Deterministic Substrate Constitution (`AGENTS.md`), adaptive mutation represents a critical boundary that requires absolute structural verification.

## Required Components for Adaptive Mutation

To perform an adaptive mutation, the operation must provide the logical conjunction of five absolute proofs. No alternate constructor or API may exist to bypass this requirement:

1. **`AdmittedControlState`**: Proof that the control state has been fully verified, structurally admitted, and conforms to the strict policy bounds.
2. **`AcceptedCertificate`**: A valid, unstale certificate establishing the mathematical constraints or standing of the transition.
3. **`AcceptedEnvelopeReceipt`**: Verification that the parameters operate strictly within their declared mathematical and numeric error envelopes.
4. **`AcceptedOutcomeReceipt`**: A certified record of the computational outcome or feedback to be integrated.
5. **`CertifiedLearningMode`**: Confirmation that the system is currently authorized and certified to undergo learning/adaptive state mutations.

## Why Must All Be Present?

The substrate enforces the law that **selection and learning are separate authorities**. Adaptive mutation (learning) fundamentally modifies the persistent, adaptive state of the system. 

Under the `ReceiptSound` law, these five components must all evaluate to true simultaneously. By requiring every component without exception, the architecture ensures that no speculative, incomplete, or unlawful mutation can ever occur. If any piece is missing or unverified, the mutation is structurally blocked, preserving the deterministic and allocation-free guarantees of the hot path.

## What Happens When Learning is Frozen?

If the system is not in a `CertifiedLearningMode` (i.e., learning is frozen), the runtime strictly enforces the following fallback behaviors:

* **Deterministic Selection Continues**: The system can still make deterministic choices using its existing state.
* **State Preservation**: All adaptive state fields remain completely unchanged.
* **Receipt Accumulation**: Receipts may continue to accumulate in the background for future application.
* **No Automatic Recertification**: The system will not attempt to automatically recertify itself while executing in the hot path.
* **Masked Fallback Execution**: The fallback to the frozen state must be implemented entirely by branchless, masked state selection (e.g., bitwise `select`), and *never* by conditional branching (no `if` or `match` statements).
