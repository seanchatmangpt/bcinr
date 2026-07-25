# Research Report: `CertificateMissing` and `ReceiptSound` Law

Based on the `AGENTS.md` guidelines for the BCINR Deterministic Substrate, here is the detailed breakdown of the `CertificateMissing` typed refusal and its relationship to the `ReceiptSound` law.

## `CertificateMissing` Typed Refusal (Rule 18)
In the BCINR constitution, human-readable error text is prohibited in the hot path. Instead, any rejected authoritative operation must yield a **bounded typed refusal code**. 

`CertificateMissing` is one of the strictly required refusal categories. If an authoritative operation fails because a required cryptographic, authority, or state certificate has not been supplied, the system must reject the operation by emitting `CertificateMissing`. It must do so strictly rather than panicking, silently clamping, mutating partial state, or providing a plausible default.

## Relationship to the `ReceiptSound` Law (Rule 11)
Rule 11 (the **ReceiptSound law**) governs adaptive mutations and dictates the inflexible requirements under which persistent state transitions are allowed.

The law mandates that **adaptive mutation** requires *all* of the following present concurrently:
- `AdmittedControlState`
- `AcceptedCertificate`
- `AcceptedEnvelopeReceipt`
- `AcceptedOutcomeReceipt`
- `CertifiedLearningMode`

Because the `ReceiptSound` law explicitly declares that **"no alternate constructor or API may exist"** to bypass these requirements, an `AcceptedCertificate` is a non-negotiable prerequisite for any adaptive mutation to occur. 

If a system component attempts an adaptive mutation without this required certificate, it directly violates the `ReceiptSound` law. In accordance with Rule 18, the operation must be safely and deterministically rejected by producing a `CertificateMissing` typed refusal. This structural combination ensures that the system firmly preserves its bounded, branchless, and allocation-free properties while rigorously guarding its state isolation.
