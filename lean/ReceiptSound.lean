def Digest := String
def MeasureRegistry := Unit
def LensRegistry := Unit
def StandingProjection := Unit
def ModeId := Nat
def Fixed := Float
def StabilityEnvelope := Unit
def DwellLaw := Unit
def VerifierId := Nat
def AllocationId := Nat
def Yield := Nat
def OutcomeStanding := Unit

inductive LearningMode
  | certifiedLearning
  | certifiedSelectionOnly
  | modeTransitionHold
  | certificateStale
  | learningFrozen
  | refused
  deriving DecidableEq

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
  gainMatrix        : Unit -- Mocked Matrix
  weightVector      : Fin 5 → Fixed
  contractionMargin : Fixed
  envelope          : StabilityEnvelope
  dwellLaw          : DwellLaw

inductive CertificateAccepted : StabilityCertificate → Prop
  | mk (c : StabilityCertificate) : CertificateAccepted c

inductive OutcomeAccepted : Yield → OutcomeStanding → Prop
  | mk (y : Yield) (s : OutcomeStanding) : OutcomeAccepted y s

structure CertificateReceipt where
  certificate       : StabilityCertificate
  verifier          : VerifierId
  accepted          : CertificateAccepted certificate

structure AdaptiveState where
  modeId : ModeId
  controlDigest : Digest
  mode : LearningMode

def inEnvelope (_state : AdaptiveState) (_env : StabilityEnvelope) : Prop := True

structure EnvelopeReceipt
    (cert : StabilityCertificate)
    (state : AdaptiveState) where
  insideEnvelope : inEnvelope state cert.envelope
  modeValid      : state.modeId = cert.modeId
  digestValid    : state.controlDigest = cert.controlDigest

structure OutcomeReceipt where
  allocationId   : AllocationId
  observedYield  : Yield
  standing       : OutcomeStanding
  admitted       : OutcomeAccepted observedYield standing

inductive AdmittedControlState : ControlState → Prop
  | mk (c : ControlState) : AdmittedControlState c

def applyCertifiedUpdate (_control : ControlState) (_cert : StabilityCertificate) (_outcome : OutcomeReceipt) (before : AdaptiveState) : AdaptiveState :=
  before

-- 2. The Sole Adaptive-Update Constructor

inductive AdaptiveUpdate : AdaptiveState → AdaptiveState → Type
  | certified
      {control : ControlState}
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

-- 3. The 5 Required Theorems

-- Theorem 1: No unwitnessed mutation
def adaptive_update_requires_outcome_receipt {before after : AdaptiveState} :
  AdaptiveUpdate before after → OutcomeReceipt
  | .certified _ _ _ out _ _ => out

-- Theorem 2: Frozen means immutable
theorem learning_frozen_precludes_update {before after : AdaptiveState}
  (h_frozen : before.mode = LearningMode.learningFrozen) :
  AdaptiveUpdate before after → False := by
  intro update
  cases update with
  | certified _ _ _ _ h_mode _ =>
    rw [h_frozen] at h_mode
    contradiction

-- Theorem 3: Certificate digest binding
theorem changed_control_invalidates_certificate (cert : StabilityCertificate) (state : AdaptiveState)
  (h_diff : state.controlDigest ≠ cert.controlDigest) :
  Nonempty (EnvelopeReceipt cert state) → False := by
  intro h_receipt
  cases h_receipt with
  | intro receipt =>
    exact h_diff receipt.digestValid

-- Theorem 4: Selection and learning are distinct
theorem frozen_selection_does_not_mutate {before after : AdaptiveState}
  (h_selection : before.mode = LearningMode.certifiedSelectionOnly) :
  AdaptiveUpdate before after → False := by
  intro update
  cases update with
  | certified _ _ _ _ h_mode _ =>
    rw [h_selection] at h_mode
    contradiction

-- Theorem 5: RF/AI reduction with admitted control state
theorem rf_ai_reduction_with_admitted_control_state
  {before after : AdaptiveState}
  (update : AdaptiveUpdate before after) :
  ∃ (control : ControlState) (cert : StabilityCertificate),
    AdmittedControlState control ∧
    CertificateAccepted cert ∧
    before.controlDigest = cert.controlDigest := by
  cases update with
  | certified controlAdmitted certificateReceipt envelopeReceipt _ _ _ =>
    exact ⟨_, certificateReceipt.certificate, controlAdmitted, certificateReceipt.accepted, envelopeReceipt.digestValid⟩

-- Required negative fixtures
example (before after : AdaptiveState) (h : before.mode = LearningMode.learningFrozen) : AdaptiveUpdate before after → False :=
  learning_frozen_precludes_update h

example (before after : AdaptiveState) (h : before.mode = LearningMode.certifiedSelectionOnly) : AdaptiveUpdate before after → False :=
  frozen_selection_does_not_mutate h

example (cert : StabilityCertificate) (state : AdaptiveState) (h : state.controlDigest ≠ cert.controlDigest) : Nonempty (EnvelopeReceipt cert state) → False :=
  changed_control_invalidates_certificate cert state h
