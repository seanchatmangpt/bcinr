# AcceptedOutcomeReceipt in the BCINR Substrate

In the context of the BCINR deterministic substrate and the MAPE-K (Observe, Infer, Propose, Accept, Execute) autonomic loop, the `AcceptedOutcomeReceipt` plays a critical role in enforcing **Rule 11 (The ReceiptSound Law)**. This law dictates that adaptive mutation requires an irrefutable combination of structural proofs, preventing speculative or unwitnessed state changes.

## What it Cryptographically Guarantees

The `AcceptedOutcomeReceipt` (often referred to as the **Yield Proof**) provides a certified, cryptographic record of the computational outcome, observed yield, and standing resulting from a previous resource allocation or deterministic transition.

Specifically, in the **Observe / Infer** phases of the MAPE-K loop, it guarantees:
1. **Verified Telemetry and Outcomes:** Any proposed mutation to the adaptive state is backed by a verifiable telemetry event or a strictly measured outcome of a prior transition. 
2. **Prevention of Unwitnessed Mutation:** It ensures the runtime cannot mutate persistent state based on speculative, untracked, or "unwitnessed" operations. The outcome of an execution must be fully captured and formally verified as a receipt before it can influence future adaptive state weights or policies.

## How it Differs from AcceptedEnvelopeReceipt and AcceptedCertificate

While all three are required simultaneously as part of the strict conjunctive gate for adaptive mutation under Rule 11 (alongside `AdmittedControlState` and `CertifiedLearningMode`), they serve distinct architectural purposes in the autonomic loop:

*   **`AcceptedCertificate` (The Foundation):** 
    *   **Purpose:** Proves that the mathematical and stability parameters governing the system have been formally certified (e.g., proving static domination or spectral-radius bounds).
    *   **Role:** It establishes the system's operational boundaries, binding to a specific `modeId` and `controlDigest`. It is typically derived out-of-band on the "slow rail" and injected into the hot path.

*   **`AcceptedEnvelopeReceipt` (The Stability Proof):**
    *   **Purpose:** Proves that the *current execution and proposed state* reside perfectly within the mathematical and numerical boundary established by the `AcceptedCertificate`.
    *   **Role:** Used during the **Accept** phase (PolicyGuard), it cryptographically guarantees that the system is not exceeding its operational envelope (e.g., `insideEnvelope : inEnvelope state cert.envelope`), protecting against numeric overflow or stability violations.

*   **`AcceptedOutcomeReceipt` (The Yield Proof):**
    *   **Purpose:** Proves the *actual outcome* or feedback of a prior deterministic transition. 
    *   **Role:** Used during the **Observe / Infer** phases, it provides empirical evidence of an execution. While the *certificate* defines the rules and the *envelope receipt* proves we are playing by them, the *outcome receipt* is the certified record of the result that triggers the actual adaptive mutation (e.g., updating weights based on verified feedback).

## Constant-Time Enforcement

In accordance with the Radon Law ($CC=1$), the absence or invalidity of an `AcceptedOutcomeReceipt` (or any other required conjunct) mathematically evaluates the admission mask ($m_{\mathrm{admitted}}$) to `0`. This mechanically blocks state mutation via constant-time, bit-level selection ($x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t$), freezing learning while allowing deterministic selection and receipt accumulation to continue seamlessly.
