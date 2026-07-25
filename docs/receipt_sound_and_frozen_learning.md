# ReceiptSound and Frozen Learning Mechanics

Based on Rule 11 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the `ReceiptSound` law enforces strict boundaries for adaptive mutation. In the autonomic substrate, state updates cannot happen speculatively; they require complete cryptographic and structural proof.

## The Roles of Receipts in the Autonomic Loop

Within the MAPE-K (Observe, Infer, Propose, Accept, Execute) autonomic loop, adaptive mutation is governed by a strict conjunctive gate. The state update mechanism (formalized as a proof-carrying type in `ReceiptSound.lean`) requires five simultaneous proofs, notably including two critical receipts:

1. **`AcceptedEnvelopeReceipt` (The Stability Proof)**
   - **Role:** During the **Accept** phase (PolicyGuard), this receipt verifies that the system's parameters and proposed state remain strictly within the declared mathematical and numeric stability envelope.
   - **Verification:** It proves that the current adaptive state digest matches the valid mathematical certificate and mathematically ensures that the execution resides perfectly within the operational boundary (e.g., `insideEnvelope : inEnvelope state cert.envelope`).

2. **`AcceptedOutcomeReceipt` (The Yield Proof)**
   - **Role:** During the **Observe / Infer** phases, this receipt provides a certified record of the computational outcome, observed yield, and standing.
   - **Verification:** It prevents unwitnessed or speculative state changes by guaranteeing that any proposed mutation is backed by a verified telemetry event or resource allocation result.

Without both of these receipts (along with an `AdmittedControlState`, `AcceptedCertificate`, and `CertifiedLearningMode`), the system is mathematically blocked from mutating the state.

## Mechanics of "Frozen Learning"

The `ReceiptSound` law explicitly mandates that **selection and learning are separate authorities**. If the system is not in a certified learning mode (e.g., `LearningMode.learningFrozen`), or if any of the required receipts are missing, "learning is frozen." 

Because the substrate strictly adheres to the Radon Law ($CC=1$) and forbids control-flow branching, this frozen fallback is **not** implemented with an `if` statement. Instead, it is mechanically enforced via **constant-time, bit-level masked state selection**:

1. **Mask Derivation:** The runtime evaluates the conjunction of all five required proofs. Since learning is frozen (or a receipt is invalid), the resulting admission mask ($m_{\mathrm{admitted}}$) mathematically evaluates to `0`.
2. **Deterministic State Commit:** The state transition function executes the fieldwise branchless equation:
   $$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$
   With $m_{\mathrm{admitted}} = 0$, this becomes:
   $$ x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t $$

As a direct result of this mechanical implementation, the following behaviors are guaranteed when learning is frozen:

* **All Adaptive State Fields Remain Unchanged:** The state is structurally immutable; the bit-for-bit representation of the adaptive weights and policies is preserved without speculative mutation or silent clamping.
* **Deterministic Selection Continues:** The autonomic loop's **Execute** phase continues to run normally and make resource selections, but strictly utilizes the existing, frozen weight parameters and previously certified state.
* **Receipts Continue to Accumulate:** The system continues to observe telemetry. New observation data and outcome receipts are recorded into the background, but they evaluate against a `0` mask and do not trigger adaptive mutations in the hot path.
* **No Automatic Recertification in the Hot Path:** The hot-path runtime will not attempt to dynamically derive a new certificate or stability envelope. Recertification is strictly relegated to the out-of-band "slow rail" (e.g., through symbolic mathematics or eigenvalue search) and must be explicitly re-injected.
