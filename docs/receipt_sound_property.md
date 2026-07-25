# The ReceiptSound Law

The `ReceiptSound` law, defined in Section 11 of `AGENTS.md`, establishes the boundaries and strict requirements for adaptive mutation within the `bcinr` deterministic substrate.

## Unlocking Adaptive Mutation

According to the constitution, adaptive mutation requires a strict conjunctive gate. No adaptive state transition can be constructed without all of the following components being simultaneously present and valid:
1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. `CertifiedLearningMode`

Mathematically and structurally (as formalized in `ReceiptSound.lean`), these pieces combine as required proofs (arguments) to invoke the sole adaptive-update constructor. 

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

Because `certified` is the *only* constructor for `AdaptiveUpdate`, it acts as a proof-carrying type. Unless a valid `CertificateReceipt`, `EnvelopeReceipt`, and `OutcomeReceipt` are supplied alongside an `AdmittedControlState` while in `certifiedLearning` mode, the state update mathematically cannot exist.

*   **`AcceptedCertificate`** (`CertificateReceipt`) proves that the stability parameters are certified and binds to a specific `modeId` and `controlDigest`.
*   **`AcceptedEnvelopeReceipt`** (`EnvelopeReceipt`) proves that the state digest matches the certificate and the state is within the admitted stability envelope (`insideEnvelope : inEnvelope state cert.envelope`).
*   **`AcceptedOutcomeReceipt`** (`OutcomeReceipt`) provides the specific observed yield and standing that trigger the mutation, preventing unwitnessed state changes.

## Mathematical Behavior During the "LearningFrozen" State

When the system enters the `LearningFrozen` state (`LearningMode.learningFrozen`), selection and learning are treated as separate authorities. 

Mathematically, any attempt to perform an adaptive mutation when the mode is `learningFrozen` evaluates to a logical impossibility (uninhabited type). This is formalized in Lean as Theorem 2 (`learning_frozen_precludes_update`):

```lean
theorem learning_frozen_precludes_update {before after : AdaptiveState}
  (h_frozen : before.mode = LearningMode.learningFrozen) :
  AdaptiveUpdate before after → False := by
  intro update
  cases update with
  | certified _ _ _ _ h_mode _ =>
    rw [h_frozen] at h_mode
    contradiction
```

At the operational substrate level, this logical constraint is implemented via constant-time, bit-level masked selection without branches (mandated by the Radon Law $CC=1$). The state transition function derives an admission mask $m_{\mathrm{admitted}}$ from the receipts and the current mode.

When learning is frozen, the system cannot produce the required proofs, so $m_{\mathrm{admitted}}$ evaluates to $0$. The deterministic state commit evaluates as:

$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$
$$ x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t $$

Therefore:
1. **Adaptive state fields remain bit-for-bit unchanged** (structurally immutable).
2. **Deterministic selection continues** (selection evaluates based on existing frozen parameters).
3. **Receipts continue to accumulate**, but without mutating weights or triggering automatic hot-path recertification.
4. **No branching occurs**; the freeze mechanism is mathematically enforced purely through the zeroed selection mask rather than control flow like `if learning_frozen { return }`.
