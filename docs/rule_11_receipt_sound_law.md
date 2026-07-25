# Rule 11: ReceiptSound Law

The `ReceiptSound` law, defined in Section 11 of the `bcinr` constitution (`AGENTS.md`), establishes the boundaries and strict requirements for adaptive mutation.

## Required Components for Adaptive Mutation

Adaptive mutation requires a strict conjunctive gate. No adaptive state transition can be constructed without **all** of the following components being simultaneously present and valid:

1. `AdmittedControlState`
2. `AcceptedCertificate`
3. `AcceptedEnvelopeReceipt`
4. `AcceptedOutcomeReceipt`
5. `CertifiedLearningMode`

No alternate constructor or API may exist. By embedding these requirements directly into the only available constructor (formalized in Lean as `certified`), the update mechanism acts as a proof-carrying type. Unless all these receipts and proofs are supplied alongside an `AdmittedControlState` while in a `certifiedLearning` mode, the state update mathematically cannot occur.

## Fallback Behavior (Learning Frozen)

Selection and learning are separate authorities. When the system enters the `LearningFrozen` state (or if receipts are missing/invalid), adaptive mutation is blocked. 

Operationally, under the strict deterministic substrate constraints (Radon Law $CC=1$, zero branches), this fallback must be implemented via **constant-time, bit-level masked state selection**, rather than branching control flow (e.g., no `if learning_frozen { return }`).

The state transition function derives an admission mask $m_{\mathrm{admitted}}$ from the receipts and the current mode. When learning is frozen, the required proofs cannot be produced, causing $m_{\mathrm{admitted}}$ to evaluate to $0$. 

The deterministic state commit evaluates as:

$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$
$$ x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t $$

Consequently, when learning is frozen:
* **Deterministic selection may continue** based on existing frozen parameters.
* **All adaptive state fields remain bit-for-bit unchanged** (structurally immutable).
* **Receipts may continue to accumulate**, but without mutating weights.
* **No automatic recertification occurs in the hot path**.
