# CertifiedLearningMode and Theorem Discovery

In the BCINR deterministic substrate, adaptive state mutation is strictly governed by the **ReceiptSound law** (Rule 11). At the core of this mechanism is the `CertifiedLearningMode`, which acts as the explicit mathematical authorization to mutate the persistent state. This document explores what `CertifiedLearningMode` entails, how it strictly adheres to the **No Runtime Theorem Discovery** rule (Rule 12), and how the mathematical state of "learning" is isolated and enforced branchlessly.

## CertifiedLearningMode under the ReceiptSound Law (Rule 11)

Under the ReceiptSound law, adaptive mutation requires the absolute logical conjunction ($\land$) of five mandatory proofs:

1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. **`CertifiedLearningMode`**

No alternate constructors or APIs are permitted to exist. Within the MAPE-K autonomic loop, **selection and learning are strictly separated authorities**. 

The `CertifiedLearningMode` receipt represents the system's authorization to transition from utilizing existing policies (inference) to adapting its state weights or policies. Without this receipt evaluating to true, learning is mathematically "frozen." When learning is frozen:
* Deterministic selection continues utilizing existing parameters.
* All adaptive state fields remain unchanged (bit-for-bit immutable).
* Telemetry and receipts continue to accumulate.
* No automatic recertification occurs in the hot path.

## Respecting the Ban on Runtime Theorem Discovery (Rule 12)

Rule 12 mandates that the authoritative runtime may **verify** a supplied witness, but it may **never discover** one. Dynamic algorithm searches, spectral-radius estimation, Lyapunov search, or optimization over weighting vectors are strictly prohibited, as they violate the bounded, branchless execution model.

`CertifiedLearningMode` respects this rule by physically separating the discovery of stability from its verification:

1. **The Slow Rail (Discovery):** The derivation of the learning mode certificate, along with stability parameters ($G, d, \delta, R_{\mathrm{noise}}, R_{\mathrm{switch}}$), happens entirely out-of-band on the non-authoritative slow rail. This rail is permitted to perform unbounded theorem discovery and iterative searches.
2. **The Hot Path (Verification):** The authoritative runtime hot path never attempts to dynamically derive or recertify the `CertifiedLearningMode`. It merely verifies the supplied fixed witness. To guarantee stability, it uses constant-time packed value comparisons to ensure that the observed state is statically dominated by the certified graph ($\widehat{G} \leq G_{\mathrm{certified}}$) and that the certified graph meets the contraction requirement ($G_{\mathrm{certified}}d \leq (1-\delta)d$). 

If the stability bounds are violated, the system does not attempt automatic recertification. It simply drops the `CertifiedLearningMode` and gracefully degrades into the frozen fallback state.

## Branchless Isolation and Verification of "Learning"

In accordance with the **Radon Law ($CC=1$)**, the runtime must contain zero control-flow branches (`if`, `match`). Therefore, the state of "learning" cannot be isolated using conditional logic like `if certified_learning_mode { mutate() }`.

Instead, the mathematical state of learning is isolated and enforced through **constant-time masked state selection**:

1. **Mask Derivation:** During the **Accept** phase (PolicyGuard), the runtime evaluates the logical conjunction of all five required proofs. If `CertifiedLearningMode` is valid and present, the admission mask ($m_{\mathrm{admitted}}$) evaluates to a full-width true mask ($2^w - 1$). If it is missing or stability bounds are breached, the mask evaluates to `0`.
2. **Deterministic State Commit:** The actual mutation of the adaptive state is performed via a fieldwise branchless equation:
   
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$

When the system is in a frozen learning state ($m_{\mathrm{admitted}} = 0$), the equation naturally resolves to:

$$x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t$$

By evaluating learning purely as a bitwise mask applied to state transitions, `bcinr` structurally guarantees bit-for-bit immutability and isolates the learning process without introducing speculative mutations or timing side channels.
