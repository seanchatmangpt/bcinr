# Slow Rail Certificate Derivation in BCINR

In the `bcinr` deterministic substrate, closed-loop adaptive stability requires continuous mathematical derivation of system parameters like spectral radius, eigenvalues, and contraction margins. However, deriving these parameters requires iterative algorithms (e.g., power iteration, Lyapunov search) that fundamentally violate the authoritative Hot Path's constitutional constraints (the Radon Law $CC=1$, zero allocation, bounded execution).

To bridge this gap, BCINR strictly enforces **No Runtime Theorem Discovery** (`AGENTS.md` §12). Heavy computation is delegated to the **Slow Rail**, which asynchronously derives the theorems, while the **Hot Path** is restricted to branchlessly *verifying* them.

## 1. Slow Rail Derivation (The Heavy Compute)

The Slow Rail is permitted to use unbounded algorithms, allocations, and floating-point math. When evaluating a new mode or adapting to system drift, the Slow Rail performs the symbolic and mathematical heavy lifting:

*   **Jacobian and Block Gain Construction**: Analyzes resource routing configurations (state vectors) to derive the system's Jacobian ($J_a$) and non-negative block gain matrix ($G_a$).
*   **Eigenvalue Search**: Uses power iteration or spectral decompositions to find the spectral radius ($\rho(G_a)$) and prove it is strictly less than 1 (contractive).
*   **Witness Construction**: Computes a strictly positive eigenvector or scaling vector ($d$) and a contraction margin ($\delta$) to serve as a verifiable mathematical witness.
*   **Threshold Discovery**: Discovers stationary ($R_{\mathrm{noise}}$) and switching ($R_{\mathrm{switch}}$) drift bounds.

## 2. Packaging the Witness (`AcceptedCertificate`)

The Slow Rail does not send raw floating-point instructions or executable code back to the Hot Path. Instead, it packages its verified mathematical findings into a static, cryptographically digested structure (often termed the `AcceptedCertificate`). 

This witness contains only fixed-point variables:
*   The comparison matrix bound ($G$)
*   The positive witness vector ($d$)
*   The contraction margin ($\delta$)
*   Noise/switch radii and other domain-specific bounds.

## 3. Hot Path Verification (`derive_stability_candidate`)

When the Slow Rail submits this witness to the authoritative runtime, the Hot Path engages strict $O(1)$ verification, as seen in `crates/bcinr-cmca/src/stability.rs`.

The Hot Path uses fixed-point (Q16.16) arithmetic to verify the static domination law:
$$ G d \leq (1-\delta)d $$

This check occurs **branchlessly**. The Hot Path elementwise multiplies the supplied matrix and vector. If the inequality holds, it successfully seals a `StabilityCandidate`. If the witness fails or contains non-positive values, the function yields a typed refusal (e.g., `StabilityDerivationRefusal::ContractionMarginInsufficient`) without executing a single `if` statement on the candidate data.

## 4. Sealing the `CertificateReceipt` (`seal_certificate`)

The final derivation of the cryptographic proof happens in `crates/bcinr-cmca/src/certification.rs`. 

The function `seal_certificate` acts as the final gatekeeper (Authority hop 5 of the C3 chain). To mint a `CertificateReceipt`, it must verify:
1.  **The Domination Witness**: It recomputes the $G d \leq (1-\delta)d$ check independently from the candidate's own fields to ensure the math wasn't spoofed.
2.  **Domain-Specific Bindings**: It strictly checks 11 domain-specific cryptographic identities (including admitted graph, generated payload, numeric profile, round identity, control mode).

If any single binding mismatches or the witness fails, the check returns a typed refusal. If everything passes, it hashes the bindings with the candidate digest (`seal_digest`) and calls `CertificateReceipt::admit_certificate`.

## 5. The ReceiptSound Law Integration

The resulting `CertificateReceipt` is a cryptographic, opaque proof that the stability parameters were mathematically sound and strictly validated. Under the **ReceiptSound Law** (`AGENTS.md` §11), adaptive mutation is impossible without this receipt. 

If the receipt is valid, it forms one piece of the admission mask ($m_{\mathrm{admitted}}$), which is evaluated bitwise to authorize the state commit:
$$ x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t) $$

By forcing the Slow Rail to do the iterative *discovery* and the Hot Path to do the $O(1)$ *verification*, BCINR guarantees civilizational-scale stability without compromising its zero-branch deterministic substrate.
