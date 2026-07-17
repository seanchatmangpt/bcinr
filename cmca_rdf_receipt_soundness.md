# CMCA-RDF ReceiptSound Formalization

The system is only fully sealed when these claims survive mechanical verification:
[
\boxed{
\text{No adaptive state transition can be constructed without}
}
]
[
\boxed{
\text{an admitted certificate, an admitted control state, a valid envelope receipt, and an outcome receipt.}
}
]

## 1. Core Types

```lean
inductive LearningMode
  | certifiedLearning
  | certifiedSelectionOnly
  | modeTransitionHold
  | certificateStale
  | learningFrozen
  | refused

structure ControlState where
  graphDigest       : Digest
  generatedDigest   : Digest
  kernelDigest      : Digest
  numericDigest     : Digest
  measureRegistry   : MeasureRegistry
  lensRegistry      : LensRegistry
  standingProjection : StandingProjection

structure StabilityCertificate where
  modeId            : ModeId
  controlDigest     : Digest
  gainMatrix        : Matrix (Fin 5) (Fin 5) Fixed
  weightVector      : Fin 5 → Fixed
  contractionMargin : Fixed
  envelope          : StabilityEnvelope
  dwellLaw          : DwellLaw

structure CertificateReceipt where
  certificate       : StabilityCertificate
  verifier          : VerifierId
  accepted          : CertificateAccepted certificate

structure EnvelopeReceipt
    (cert : StabilityCertificate)
    (state : AdaptiveState) where
  insideEnvelope : state ∈ cert.envelope
  modeValid      : state.modeId = cert.modeId
  digestValid    : state.controlDigest = cert.controlDigest

structure OutcomeReceipt where
  allocationId   : AllocationId
  observedYield  : Yield
  standing       : OutcomeStanding
  admitted       : OutcomeAccepted observedYield standing
```

## 2. The Sole Adaptive-Update Constructor

```lean
inductive AdaptiveUpdate :
    AdaptiveState → AdaptiveState → Type
  | certified
      (controlAdmitted : AdmittedControlState control)
      (certificateReceipt : CertificateReceipt)
      (envelopeReceipt :
        EnvelopeReceipt certificateReceipt.certificate before)
      (outcomeReceipt : OutcomeReceipt)
      (mode :
        before.mode = LearningMode.certifiedLearning)
      (transition :
        after =
          applyCertifiedUpdate
            control
            certificateReceipt.certificate
            outcomeReceipt
            before) :
      AdaptiveUpdate before after
```

## 3. The 5 Required Theorems

1. **No unwitnessed mutation**: Prove `adaptive_update_requires_outcome_receipt` (constructor inversion).
2. **Frozen means immutable**: Prove `learning_frozen_precludes_update`.
3. **Certificate digest binding**: Prove `changed_control_invalidates_certificate`.
4. **Selection and learning are distinct**: Prove `frozen_selection_does_not_mutate`.
5. **RF/AI reduction with admitted control state**: Decompose `AllocationIntegrityWin` into receipt forgery, admission failure, or digest collision.

## 4. Work Sequence

1. **ReceiptSound Lean skeleton**: Prove the 5 theorems and the negative uninhabited fixtures.
2. **Rust constructor and typestate mirror**: Build the exact isomorphic boundaries in `crates/bcinr-cmca`.
3. **$\kappa_q$ Observatory**: Build the spatial visualizer.
4. **$\kappa_q$ admission-to-recertification workflow**: Establish how the measurement artifact becomes a new certified learning mode.
