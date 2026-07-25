# `LearningFrozen` in BCINR

Based on the codebase analysis, here is the exact definition of `LearningFrozen` and the branchless mathematical condition that triggers it.

## Exact Definition

`LearningFrozen` is a bounded typed refusal code defined as a variant of the `StabilityRefusal` enum in `crates/bcinr-cmca/src/allocator.rs`. 

Governed by the `ReceiptSound` law (Rule 11), it represents a system fallback state where adaptive mutation is categorically blocked. Adaptive mutation requires a strict conjunctive gate (the simultaneous presence of an `AdmittedControlState`, `AcceptedCertificate`, `AcceptedEnvelopeReceipt`, `AcceptedOutcomeReceipt`, and `CertifiedLearningMode`). If any of the cryptographic receipts are missing, invalid, or fail the mathematical bounds check, the requisite proofs are absent, and the system structurally falls back to the `LearningFrozen` refusal state.

## Branchless Mathematical Trigger Condition

Under Rule 12, the substrate forbids dynamic theorem discovery at runtime. Instead, the hot path enforces contractive stability and static domination bounds by branchlessly verifying two fixed-point, packed-value inequalities against a static witness:

1. $\widehat{G} \leq G_{\mathrm{certified}}$  
   *(The dynamic system matrix is bounded by the certified matrix)*
2. $G_{\mathrm{certified}} d \leq (1-\delta)d$  
   *(The certified system strictly contracts the bounding witness vector $d$ by the margin $\delta$)*

**The Trigger:**  
If the fixed-point arithmetic calculation yields $(G_{\mathrm{certified}} d)_i > (1 - \delta) d_i$ for any dimension $i$, the proposed dynamics fail to prove bounded convergence (emitting `ContractionMarginInsufficient`). Because the static domination check fails, the certificate is rendered invalid, and the requisite proofs for learning are absent. This triggers the `LearningFrozen` fallback.

### Branchless Enforcement (The Radon Law: $CC=1$)

To comply with the strict zero-branching mandate, the system cannot use conditional control flow (e.g., `if learning_frozen { return }`). Instead, the freeze mechanism is enforced via constant-time, bit-level masked state selection.

The state transition function derives an admission mask ($m_{\mathrm{admitted}}$). A failed validation structurally zeroes out the mask ($m_{\mathrm{admitted}} = 0$).

The deterministic commit phase executes as:
$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

When `LearningFrozen` is triggered ($m_{\mathrm{admitted}} = 0$):
$$ x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t $$

This branchless mathematical condition guarantees that when learning is frozen, the persistent adaptive state remains bit-for-bit unchanged ($x_t$), while deterministic selection securely continues using the existing bounded parameters.
