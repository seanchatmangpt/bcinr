# CertifiedLearningMode and Frozen Fallback

Under Rule 11 (**ReceiptSound law**) in `AGENTS.md`, adaptive mutation of the BCINR substrate is strictly gated. The capability to mutate state is fundamentally governed by the logical conjunction ($\land$) of five mandatory proofs:

1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. **`CertifiedLearningMode`**

No alternate constructor or API is permitted to exist. 

## Role in Adaptive Mutation

Within the autonomic substrate, **selection and learning are completely separate authorities**. `CertifiedLearningMode` acts as the explicit authorization required for the system to adapt its internal state, policies, or weights. Without this receipt evaluating to true, the system is blocked from committing proposed mutations into the persistent adaptive state.

## Fallback Behavior When Learning is Frozen

If the system is not in a certified learning mode (e.g., the receipt is missing or stability bounds are violated), it enters a **frozen learning** state. In this state, the substrate gracefully degrades to utilizing only existing policies:

- **Deterministic selection may continue:** The system will still make selections and decisions using its current, frozen parameters.
- **Adaptive state fields remain unchanged:** Persistent state is preserved bit-for-bit.
- **Receipts may continue to accumulate:** Telemetry and outcome receipts are still collected.
- **No automatic recertification occurs in the hot path:** The runtime will not attempt to dynamically discover new theorems or derive new certificates to unfreeze itself.

## Branchless Implementation (Masked State Selection)

In accordance with the Radon Law ($CC=1$) and the project's strict branchlessness mandate, the frozen fallback is not implemented via traditional control flow (e.g., `if frozen { ... } else { ... }`). 

Instead, the fallback behavior is mechanically enforced through **masked state selection**:

1. All required proofs are evaluated to derive an admission mask ($m_{\mathrm{admitted}}$). If `CertifiedLearningMode` is absent, the mask evaluates to `0`.
2. The mutation is applied via a fieldwise branchless equation:
   
   $$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

   *(where $x_t$ is the current state and $x_{\mathrm{candidate}}$ is the proposed mutation)*

When learning is frozen ($m_{\mathrm{admitted}} = 0$), the bitwise selection structurally defaults to the current state ($x_t$). This ensures that the state remains perfectly immutable without introducing timing side channels or conditional branches in the hot path.
