# AdmittedControlState and Immutable Constructor Requirements in BCINR

Under **Rule 11 (ReceiptSound Law)** of the BCINR deterministic substrate constitution, the `AdmittedControlState` represents a mathematically undeniable structural proof that a proposed control state is lawful. Its integration into the system's adaptive mutation pipeline enforces strict constraints on how and when persistent state can be updated.

## Why Conjunctive Verification is Strictly Required

Adaptive mutation in BCINR is not a simple state assignment; it is a rigid, branchless mathematical equation. Rule 11 mandates that a state transition can only occur through the **conjunctive verification** of five exact proofs:

1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. `CertifiedLearningMode`

### The Mechanism of Conjunctive Verification

1. **Proof-Carrying Gate:** The `AdmittedControlState` acts as a required receipt that a speculative candidate state (calculated purely on the stack without heap allocation) has passed all mathematical policy guards. 
2. **Branchless Masked Commit:** These five components are evaluated to derive an **admission mask** ($m_{\mathrm{admitted}}$). Because BCINR forbids control flow branching ($CC=1$, Radon Law), the final commit step is purely arithmetic:
   $$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$
3. **The Fallback Guarantee:** If any of the five required proofs are missing or invalid (for instance, if the system is in `learningFrozen` mode, or an outcome receipt is rejected), the admission mask mathematically collapses to $0$. The selection function then perfectly falls back to $x_t$, leaving the persistent state bit-for-bit unchanged without the need for an `if` statement or panic path.

## Why Alternate Constructors or APIs are Mathematically Illegal

The absolute prohibition on alternate constructors or APIs is necessary to maintain the formal mathematical standing of the substrate.

1. **Proof-Carrying Types:** In the formal Lean specification for BCINR (`ReceiptSound.lean`), the state transition is defined by the inductive type `AdaptiveUpdate`. This type possesses **exactly one constructor** (`certified`). 
   ```lean
   inductive AdaptiveUpdate : AdaptiveState → AdaptiveState → Type
     | certified
         {control : ControlState}
         (controlAdmitted : AdmittedControlState control)
         (certificateReceipt : CertificateReceipt)
         (envelopeReceipt : EnvelopeReceipt certificateReceipt.certificate before)
         (outcomeReceipt : OutcomeReceipt)
         (mode : before.mode = LearningMode.certifiedLearning)
         (transition : after = applyCertifiedUpdate ...) :
         AdaptiveUpdate before after
   ```
   Because there is only one constructor, it is formally impossible to instantiate an `AdaptiveUpdate` without supplying all the required proofs simultaneously. 

2. **Closing Side-Channels and Evasion:** If an alternate API or constructor were permitted (such as a forced setter, a bypass function, or a partial update method), it would break the proof-carrying type model. It would allow speculative or unwitnessed mutation to bypass the branchless $m_{\mathrm{admitted}}$ mask derivation, violating the *no mutation before complete admission* law.
   
3. **Distinct Authorities:** Selection and learning are distinct authorities. In modes like `certifiedSelectionOnly` or `learningFrozen`, deterministic selection continues using the frozen parameters, but learning (adaptive mutation) is mathematically blocked. Alternate APIs would violate this separation of authority by creating unprotected vectors into the persistent state, risking the integrity of the autonomic control loop.

By mathematically binding the state transition directly to the presentation of the `AdmittedControlState` and its companion receipts, BCINR structurally guarantees that the runtime can never deviate from its axiomatic laws.
