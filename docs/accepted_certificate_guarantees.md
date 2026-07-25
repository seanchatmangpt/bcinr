# AcceptedCertificate Guarantees and Verification

Based on Rule 11 (ReceiptSound Law) and Rule 12 (No Runtime Theorem Discovery) from the BCINR `AGENTS.md` constitution, the `AcceptedCertificate` serves as a foundational cryptographic receipt for safe adaptive mutation.

## Cryptographically Verifiable Guarantees

The `AcceptedCertificate` provides mathematical and cryptographic proofs that bind the system's dynamic resource routing configurations to strict stability envelopes without violating the deterministic, branchless constraints of the Hot Path. 

Regarding the **control state** and **learning mode**, it provides the following guarantees:

1. **State-Certificate Cohesion (Anti-Stale Protection):** The certificate contains a cryptographic digest that must perfectly match the digest of the current `AdmittedControlState`. This guarantees that the pre-calculated mathematical bounds (the witness vector $d$, the comparison matrix $G_{\mathrm{certified}}$, and the contraction margin $\delta$) correspond exactly to the current system configuration.
2. **Conjunctive Prerequisite for Adaptive Mutation:** It mathematically acts as a proof-carrying type. The adaptive state cannot mutate unless the `AcceptedCertificate` is verified simultaneously alongside an `AdmittedControlState`, an `AcceptedEnvelopeReceipt`, an `AcceptedOutcomeReceipt`, and an explicitly `CertifiedLearningMode`. 
3. **Bounded Stability Envelopes:** By validating the certificate, it guarantees the system statically adheres to the mathematical proofs of **Static Domination** ($\widehat G \leq G_{\mathrm{certified}}$) and **Contractive Stability** ($G_{\mathrm{certified}} d \leq (1-\delta)d$).

## Integration into the Conjunctive Verification Gate

Rule 12 explicitly forbids unbounded iterative algorithms (like spectral radius estimation or Lyapunov search) from running on the strictly branchless ($CC=1$) and allocation-free Authoritative Hot Path. To adhere to this without sacrificing security, the system splits theorem *discovery* from theorem *verification*:

1. **Slow Rail Theorem Discovery:** An asynchronous, non-authoritative Slow Rail—which is permitted to allocate memory, branch, and use floating-point arithmetic—calculates the Jacobian matrix and performs the spectral eigenvalue search to prove local contractivity ($\rho < 1$). It discovers the witness eigenvector ($d$) and margin ($\delta$) and packages them into the static, cryptographically digested `AcceptedCertificate`.
2. **Branchless $O(1)$ Hot Path Verification:** The authoritative runtime receives the certificate and strictly evaluates it in constant time without a single `if`, `match`, or early return:
   * **Digest Matching:** The Hot Path validates the cryptographic digest via a parallel bitwise XOR cascade (e.g., `(((state.digest ^ cert.digest) | ...) == 0) as u32`). Any bitwise difference generates a non-zero value that is cast directly into a boolean mask.
   * **Packed-Value Inequality Checks:** The static domination bounds and contraction margins are verified using simple fixed-point matrix-vector multiplication and packed-value comparisons, which are fully statically unrolled.
3. **Masked State Commit:** All branchless validations are aggregated into a master admission mask ($m_{\mathrm{admitted}}$). The conjunctive gate commits the state via deterministic fieldwise selection: 
   $$x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$$
   * If all proofs hold, the mask evaluates to full-width ones (`0xFFFFFFFF`), and the new adaptive state is accepted.
   * If a digest mismatch occurs or inequalities fail, the mask evaluates to `0`. A bounded typed refusal (e.g., `DIGEST_MISMATCH` or `ContractionMarginInsufficient`) is branchlessly formed, the learning mode is frozen, and the pre-existing state is rewritten into memory bit-for-bit unchanged.
