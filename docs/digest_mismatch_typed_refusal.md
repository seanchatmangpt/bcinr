# Research: DigestMismatch Typed Refusal

## What is `DigestMismatch`?
According to Rule 18 ("Typed refusals") of the BCINR `AGENTS.md` constitution, all rejected authoritative operations must produce a bounded typed refusal code. `DigestMismatch` is one of these mandatory categories. It is used when an input's cryptographic or integrity digest fails to match the expected, certified value. Rule 18 strictly prohibits handling unsupported or invalid inputs by panicking, silently clamping, or falling back to a simpler algorithm. 

## Relationship to the `ReceiptSound` Law (Rule 11)
Rule 11 governs "Adaptive mutation" of persistent state. It dictates that mutations can only occur if a strict logical conjunction is satisfied:
- `AdmittedControlState`
- `AcceptedCertificate`
- `AcceptedEnvelopeReceipt`
- `AcceptedOutcomeReceipt`
- `CertifiedLearningMode`

To transition adaptive state, the runtime must process external certificates and receipts. The `DigestMismatch` refusal acts as the primary gatekeeper for these requirements. If a supplied certificate, envelope, or receipt has an invalid or corrupted digest, the runtime must reject the mutation. The `DigestMismatch` refusal enforces the "Accepted" properties of the `ReceiptSound` law, ensuring that no malicious, stale, or forged certificates can alter the system's adaptive state.

## Why Form the Refusal Branchlessly Instead of Panicking?
BCINR is a "deterministic computational substrate for bounded, branchless, allocation-free execution." The system must strictly adhere to the Radon Law (`CC=1`, Cyclomatic Complexity of 1) across the entire authoritative call graph (Rule 3 and 8).

1. **No Control-Flow Branches**: Using a `panic!`, early `return Err(...)`, or `?` operator introduces hidden data-dependent branches and unwinding paths into the compiled object code. This violates the `CC=1` requirement and could introduce execution timing side-channels, breaking the substrate's deterministic execution guarantee.
2. **Fixed Execution Work**: The runtime requires "fixed bounded execution work." Evaluating an invalid digest must take the exact same number of CPU cycles and instructions as evaluating a valid one. 
3. **Mask-Based Execution (Rules 9 & 10)**: Instead of branching upon a digest mismatch, the result of the digest comparison must be evaluated into a full-width boolean mask (e.g., all 0s or all 1s). This mask is then used to branchlessly select between the `candidate` state and the `current` state. If a `DigestMismatch` is detected, the mask forces the selection of the unmodified `current` state, and the `DigestMismatch` refusal code is selected as the output—all without altering the linear flow of CPU instructions.
