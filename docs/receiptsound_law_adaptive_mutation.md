# Rule 11 (ReceiptSound Law) and Adaptive Mutation

## 1. Required Components for Adaptive Mutation
Rule 11 (the `ReceiptSound` law) strictly governs how the adaptive state can be mutated. Adaptive mutation requires a strict conjunctive gate. No transition can happen without **all** of the following cryptographic receipts and proofs being simultaneously valid and present:
- `AdmittedControlState`: Proof that the persistent control state (digests, registries, projections) has been properly admitted and verified.
- `AcceptedCertificate`: Proof that a mathematical stability certificate has been validated and accepted by an independent verifier.
- `AcceptedEnvelopeReceipt`: Proof that the current adaptive state remains safely inside the stability envelope established by the certificate.
- `AcceptedOutcomeReceipt`: Proof of a valid outcome measurement and standing result from a previous resource allocation yield.
- `CertifiedLearningMode`: The system's learning mode must explicitly evaluate to `certifiedLearning`.

## 2. Structural Enforcement in the Codebase

### Proof-Carrying Types (Lean Specification)
In `lean/ReceiptSound.lean`, this rule is enforced by embedding these exact requirements directly into the sole state update constructor: the `AdaptiveUpdate.certified` inductive type. Because there is no alternate constructor or API, the state mathematically cannot mutate unless all these receipts and proofs are supplied simultaneously.

```lean
inductive AdaptiveUpdate : AdaptiveState → AdaptiveState → Type
  | certified
      {control : ControlState}
      (controlAdmitted : AdmittedControlState control)
      (certificateReceipt : CertificateReceipt)
      (envelopeReceipt : EnvelopeReceipt certificateReceipt.certificate before)
      (outcomeReceipt : OutcomeReceipt)
      (mode : before.mode = LearningMode.certifiedLearning)
      (transition : after = applyCertifiedUpdate control certificateReceipt.certificate outcomeReceipt before) :
      AdaptiveUpdate before after
```
The specification also contains formal mathematical proofs, such as `learning_frozen_precludes_update` and `frozen_selection_does_not_mutate`, verifying that state transitions absolutely cannot occur under invalid modes.

### Bit-Level Masked State Selection (Rust Runtime)
To adhere to the project's strict deterministic constraints (Radon Law $CC=1$, zero branches), the fallback behavior is structurally enforced via constant-time, bit-level masked state selection rather than standard control-flow branching.

According to `docs/receiptsound_law_adaptive_mutation.md`, an admission mask $m_{\mathrm{admitted}}$ is derived from the conjunctive evaluation of the required receipts and the current learning mode. The deterministic state commit evaluates as:
$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

If any required cryptographic receipt is missing, invalid, or the system is in a frozen learning mode, the proofs cannot be met. The $m_{\mathrm{admitted}}$ mask evaluates to 0, leaving the state bit-for-bit unchanged without branching:
$$ x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t $$

When learning is frozen, the system enforces the following:
* **Deterministic selection may continue**: Allocations and selections continue to run normally, but only using the existing frozen weight parameters.
* **All adaptive state fields remain unchanged**: The state becomes structurally immutable.
* **Receipts may continue to accumulate**: New observation telemetry and outcome receipts are recorded, but they do not mutate the weights.
* **No automatic recertification occurs in the hot path**: The runtime will not attempt to derive a new certificate dynamically; recertification is relegated to the out-of-band "slow rail".

These structures strictly isolate the selection and learning authorities as mandated by Section 11 of the BCINR Deterministic Substrate Constitution.
