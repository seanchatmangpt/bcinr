# CertifiedLearningMode Receipt

In the BCINR deterministic substrate, the **ReceiptSound** law (Rule 11 of the `AGENTS.md` constitution) establishes the strict mathematical preconditions required for adaptive state mutation. At the core of this authorization is the `CertifiedLearningMode` receipt. 

Adaptive mutation requires the absolute logical conjunction ($\land$) of five mandatory proofs:
1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. **`CertifiedLearningMode`**

No alternate constructors or APIs are permitted to exist or bypass these conjuncts.

## Role in Authorizing Adaptive Mutation within the MAPE-K Loop

Within the MAPE-K (Observe, Infer, Propose, Accept, Execute) autonomic loop, adaptive mutation is strictly gated. The `CertifiedLearningMode` acts as the explicit authorization that allows the system to transition from purely utilizing existing policies (inference) to adapting its state weights or policies based on accumulated telemetry and rewards. 

During the **Accept** phase (PolicyGuard), the runtime evaluates the conjunction of all required proofs. If the system possesses a valid `CertifiedLearningMode` receipt (alongside the other four), the resulting admission mask evaluates to true ($2^w - 1$), mathematically allowing the **Execute** phase to commit the proposed structural mutations to persistent adaptive state.

## Transition to "Frozen Learning" (Inference Only)

The `ReceiptSound` law explicitly mandates that **selection and learning are separate authorities**. If the system lacks the `CertifiedLearningMode` receipt, or if it is suspended due to stability bounds being violated, the system transitions into "frozen learning."

Under "frozen learning", the system's behavior is strictly defined:

* **Deterministic selection continues:** The autonomic loop's **Execute** phase continues to run normally and make selections, utilizing the existing, frozen weight parameters and previously certified state.
* **All adaptive state fields remain unchanged:** Bit-for-bit immutability is enforced. Persistent state cannot be mutated speculatively.
* **Receipts may continue to accumulate:** The system continues to gather telemetry and outcome receipts for the MAPE-K loop, even though they cannot trigger mutations.
* **No automatic recertification occurs in the hot path:** The runtime will not attempt to dynamically derive a new certificate. Recertification is strictly relegated to the slow-rail process (e.g., through symbolic mathematics or eigenvalue search) and must be explicitly re-injected.

## Branchless Enforcement via Masked Fallback

In strict adherence to the project's branchless mandate (the Radon Law, $CC=1$), the frozen fallback must **not** be implemented using control flow branches (`if learning_frozen { ... }`). 

Instead, the fallback is implemented mechanically via **constant-time, bit-level masked state selection**:

1. **Mask Derivation:** The runtime evaluates the conjunction of all five required proofs. If `CertifiedLearningMode` is missing or evaluates to false, the admission mask ($m_{\mathrm{admitted}}$) evaluates to `0`.
2. **Deterministic State Commit:** The state transition function executes the fieldwise branchless equation:
   $$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$
   With $m_{\mathrm{admitted}} = 0$, this becomes:
   $$ x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t $$

This ensures that the unmutated current state ($x_t$) is deterministically selected when learning is frozen, preserving execution timing and removing logic branches from the hot path.
