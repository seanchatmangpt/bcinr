# ContractionMarginInsufficient & LearningFrozen Refusals

In the BCINR Deterministic Substrate, adaptive mutation and systemic stability are tightly governed by typed refusals. Under Rule 18, `ContractionMarginInsufficient` and `LearningFrozen` are critical refusal mechanisms that enforce contractive stability proofs while adhering to the strict branchless ($CC=1$) constraints of the hot path.

## Enforcing Static Domination and Contractive Stability

Rule 12 explicitly forbids dynamic theorem discovery at runtime (e.g., spectral-radius estimation or power iteration). Instead, stability is verified via a proof supplied by the `AcceptedCertificate`. The "slow rail" computes and serializes a fixed static witness consisting of a certified system matrix ($G_{\mathrm{certified}}$), a strictly positive witness vector ($d$), and a contraction margin ($\delta$).

The hot path enforces static domination bounds by verifying two fixed-point, packed-value inequalities:
1. $\widehat{G} \leq G_{\mathrm{certified}}$ (The dynamic system matrix is bounded by the certified matrix)
2. $G_{\mathrm{certified}} d \leq (1-\delta)d$ (The certified system strictly contracts the bounding vector by the margin $\delta$)

If the arithmetic calculation yields $(G_{\mathrm{certified}} d)_i > (1 - \delta) d_i$ for any dimension, the proposed dynamics fail to prove bounded convergence. The hot path immediately rejects the state transition by emitting the `ContractionMarginInsufficient` typed refusal. 

Furthermore, under the `ReceiptSound` law (Rule 11), adaptive mutation requires the simultaneous presence of an `AdmittedControlState`, `AcceptedCertificate`, envelope/outcome receipts, and `CertifiedLearningMode`. If the static domination check fails, the certificate is invalid, the requisite proofs are absent, and the system falls back to a frozen state, emitting the `LearningFrozen` refusal.

## Mathematical Impact on the Adaptive State (Branchless Execution)

Because the substrate is bound by the Radon Law (Rule 8: $CC=1$), the system cannot use conditional control flow (e.g., `if invalid { return Err(...) }`) to abort a state update. The emission of these refusals and the fallback behavior must occur via constant-time, bit-level masked state selection.

When a proposed transition violates the static domination bounds or lacks the requisite proofs for learning, the runtime derives an admission mask ($m_{\mathrm{admitted}}$). A failed validation structurally evaluates to a mask of $0$.

The deterministic commit phase executes the transition mathematically as:

$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

When `ContractionMarginInsufficient` or `LearningFrozen` are triggered ($m_{\mathrm{admitted}} = 0$):

$$ x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t $$

Mathematically, this guarantees that:
- **Zero Speculative Mutation:** The persistent adaptive state remains bit-for-bit unchanged.
- **Divergence Prevention:** Unproven dynamics cannot corrupt the admitted state.
- **Continuous Execution:** Deterministic selection continues using the existing, safely bounded state, without interrupting the flow of the application or initiating unpredictable runtime bounds.
