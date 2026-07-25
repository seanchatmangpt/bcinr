# Research Results: AcceptedEnvelopeReceipt & AcceptedOutcomeReceipt

Under **Rule 11 (The ReceiptSound Law)** of the BCINR Deterministic Substrate Constitution, adaptive mutation of persistent state is strictly gated by a conjunctive verification gate. The runtime is structurally prohibited from mutating persistent state unless it holds an irrefutable combination of five simultaneous cryptographic proofs, with no alternate constructors or APIs permitted to bypass this requirement. 

Among these required proofs are the `AcceptedEnvelopeReceipt` and `AcceptedOutcomeReceipt`. If either is missing or invalid, the derived admission mask mathematically evaluates to `0`. Under the strict branchless constraints (Radon Law $CC=1$), this zeroed mask enforces a fallback behavior (frozen learning) via constant-time bit-level selection ($x_{t+1} = \operatorname{select}(0, x_{\mathrm{candidate}}, x_t) = x_t$), leaving the state bit-for-bit unchanged.

Here is a detailed breakdown of why these two specific receipts are mandatory for adaptive mutation:

## 1. AcceptedEnvelopeReceipt (The Stability Proof)
The `AcceptedEnvelopeReceipt` acts as the structural guarantee that the current execution is safe and bounded. 

* **Enforces Mathematical Boundaries:** It proves that the current adaptive state and proposed execution remain strictly inside the mathematical stability envelope established by the `AcceptedCertificate`. 
* **Binds Oracle Contracts to Runtime:** It is the deterministic mechanism that binds the strict mathematical limits (numeric error bounds, admissible domains, conservation laws) established by the `@hoare_oracle` to the dynamic hot-path execution.
* **Deterministic Refusal:** If an execution threatens to exceed these certified boundaries, the runtime cannot produce a valid receipt. Instead of using branching control flow (like `if/else`) or traditional errors, this failure triggers a bounded typed refusal (e.g., `StabilityRefusal::EnvelopeViolated`), zeros out the selection mask, and prevents the mutation while recording the fault.

## 2. AcceptedOutcomeReceipt (The Yield Proof)
The `AcceptedOutcomeReceipt` provides the empirical evidence required to justify an adaptive update within the autonomic (MAPE-K) loop.

* **Prevents Unwitnessed Mutation:** It proves that any proposed mutation to the adaptive state is backed by verified telemetry or a strictly measured outcome from a previous resource allocation yield. It guarantees the runtime never mutates based on speculative, untracked, or "unwitnessed" operations.
* **Certified Feedback Loop:** While the EnvelopeReceipt proves the system is playing by the rules, the OutcomeReceipt provides the certified record of the *actual computational result* and standing. This certified outcome is what validates and triggers the update of adaptive state weights or policies in the Observe/Infer phases.

By requiring both receipts simultaneously, the deterministic substrate ensures that adaptive mutation only occurs when the execution is mathematically safe (EnvelopeReceipt) and backed by verified historical yields (OutcomeReceipt).
