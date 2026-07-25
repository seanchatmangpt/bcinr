# AcceptedEnvelopeReceipt and Input Boundaries

## Overview
Under Rule 11 (The ReceiptSound Law) of the BCINR Deterministic Substrate Constitution, adaptive mutation represents a critical boundary that requires absolute structural verification. No adaptive state transition can occur without a strict conjunctive gate of five required cryptographic proofs, one of which is the `AcceptedEnvelopeReceipt`.

## Role of the Envelope Before the Deterministic Hot Path
The "envelope" defines the strict bounding constraints and acceptable operational limits for execution. Because the deterministic hot path is strictly governed by the **Radon Law ($CC=1$)**—meaning no conditional branching (like `if`, `match`, or early returns) is permitted—and relies purely on fixed-point arithmetic (e.g., Q16.16) and zero allocations, traditional bounds-checking and runtime exception handling are structurally impossible.

Before data is processed in the hot path, the mathematical envelope acts as an essential safeguard. It formally declares:
- The admissible domain and codomain.
- The maximum absolute and relative error bounds for approximations.
- Overflow, underflow, and saturation boundaries.

Verifying this envelope ensures that the incoming parameters are mathematically safe to execute. It prevents the system from encountering undefined behavior, relying on silent epsilons, or triggering uncertified fallbacks that would break the deterministic and constant-time execution guarantees of the substrate.

## Cryptographically Verifiable Guarantees
The `AcceptedEnvelopeReceipt` acts as a proof token (e.g., `pub struct EnvelopeReceipt { pub(crate) digest: u64 }`) that guarantees the state and its parameters operate strictly within these declared limits. It provides the following cryptographically verifiable guarantees:

1. **Mathematical Bound Validation**: It proves that the execution resides perfectly within the numerical envelope established by the accepted stability certificate (`insideEnvelope : inEnvelope state cert.envelope`).
2. **Cryptographic Continuity**: The digest inside the envelope receipt must structurally align with the `AdmittedControlState` and the `AcceptedCertificate`. The hot path validates this cryptographic alignment strictly through branchless arithmetic (e.g., `state.digest ^ env.digest`), safely isolating any mismatching bits without conditional logic.
3. **Execution Safety**: It acts as empirical evidence that processing the inputs will not result in numeric anomalies, out-of-bounds precision loss, or architectural timing side-channels.
4. **Authoritative Masking Gate**: If an input exceeds the boundaries (meaning an `AcceptedEnvelopeReceipt` is invalid or cannot be produced), it triggers a bounded typed refusal (e.g., `StabilityRefusal::EnvelopeViolated`). Instead of throwing an error or using an `if / else` statement, this evaluation mathematically produces a `0` mask. This zeroed selection mask prevents any persistent state mutation (`select(0, candidate, current) = current`), leaving the adaptive state bit-for-bit unchanged.
