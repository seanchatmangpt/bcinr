# AcceptedEnvelopeReceipt and the ReceiptSound Law

Under **Rule 11 (The ReceiptSound Law)** of the BCINR Deterministic Substrate Constitution, adaptive mutation is strictly gated. The runtime cannot modify persistent state unless it holds an irrefutable combination of five structural proofs (the conjunctive verification gate):

1. `AdmittedControlState`
2. `AcceptedCertificate`
3. **`AcceptedEnvelopeReceipt`**
4. `AcceptedOutcomeReceipt`
5. `CertifiedLearningMode`

No alternate constructors or APIs are permitted to bypass this requirement. 

## Role in the Conjunctive Verification Gate

The `AcceptedEnvelopeReceipt` acts as a required cryptographic proof token (e.g., `pub struct EnvelopeReceipt { pub(crate) digest: u64 }`). Its specific role in the conjunction is to serve as the **Stability Proof**. It mathematically proves that the state and its parameters are operating strictly within their declared limits, specifically ensuring that the current adaptive state remains safely inside the stability envelope established by the certificate (`insideEnvelope : inEnvelope state cert.envelope`).

Without this receipt, the conjunctive gate fails, and any adaptive mutation is structurally blocked.

## Binding Runtime Execution to the `@hoare_oracle` Envelopes

The `@hoare_oracle` establishes strict mathematical contracts, including numeric error bounds, admissible domains, and conservation laws. The `AcceptedEnvelopeReceipt` is the mechanism that binds these static, mathematically proven boundaries to the dynamic runtime execution in a branchless, deterministic manner:

1. **Boundary Verification without Branches**: The runtime evaluates the computational boundaries (the "envelope") prior to hot-path execution. It validates the state digest and numeric parameters against the exact limits established by the `@hoare_oracle`. 
2. **Authoritative Masking (Zero-Mask Generation)**: If the runtime execution exceeds the certified boundaries—meaning a valid `AcceptedEnvelopeReceipt` cannot be produced or its digest mismatches—it does not throw a traditional error or use branching control flow (`if/else`). Instead, it triggers a bounded typed refusal (e.g., `StabilityRefusal::EnvelopeViolated`).
3. **Deterministic Refusal Execution**: This typed refusal evaluation mathematically derives a `0` admission mask. 
4. **State Preservation**: The resulting zeroed selection mask structurally prevents any persistent state mutation using branchless fixed-width state transitions: `select(0, candidate, current) = current`. This guarantees the adaptive state remains bit-for-bit unchanged if the mathematical envelope is violated.
5. **Fault Accumulation**: If the boundaries of the `AcceptedEnvelopeReceipt` are breached, the generated mask is also used to structurally select the appropriate bitwise fault code (e.g., `RANGE_VIOLATION` or `APPROX_ENVELOPE`). This fault is structurally recorded via a bitwise union (e.g., `self.faults = self.faults.union(e)`), ensuring fault tracking operates as a branchless join-semilattice without short-circuiting.

By reducing the `@hoare_oracle`'s mathematical constraints into a strictly required structural receipt and a deterministic masking operation, the `AcceptedEnvelopeReceipt` ensures that any execution outside the proven bounds is physically incapable of mutating the system's persistent state.
