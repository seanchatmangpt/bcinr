# AcceptedCertificate Boundaries and Verification

In the BCINR deterministic substrate, the `AcceptedCertificate` is a foundational cryptographic receipt mandated by **Rule 11 (The ReceiptSound Law)**. It bridges the architectural division between the asynchronous Slow Rail (which is allowed to branch and use floating-point math) and the Authoritative Hot Path (which must remain strictly branchless and zero-allocation).

## 1. Mathematical Boundaries Established

The `AcceptedCertificate` establishes static mathematical boundaries proving the system's contractive stability and resource domination. It packages the following fixed-point bounds:
*   **$G_{\mathrm{certified}}$ (The Certified Gain Matrix):** A bounded comparison matrix for the system.
*   **$d$ (The Scaling Vector/Eigenvector):** A strictly positive right eigenvector that acts as a weighted block norm.
*   **$\delta$ (The Contraction Margin):** The required mathematical margin proving strictly contractive stability.
*   **$R_{\mathrm{noise}}$ and $R_{\mathrm{switch}}$:** Stationary and switching drift bounds.

By supplying this certificate, it guarantees that the current dynamic system ($\widehat G$) is bounded by the certified matrix and strictly contracts, establishing the mathematical proofs:
1.  **Static Domination:** $ \widehat G \leq G_{\mathrm{certified}} $
2.  **Contractive Stability:** $ G_{\mathrm{certified}} d \leq (1-\delta)d $

## 2. Slow Rail Certificate Derivation (Rule 12)

Because **Rule 12 (No runtime theorem discovery)** forbids the Authoritative Runtime from computing eigenvalues or using iterative algorithms, this heavy lifting is offloaded to the Slow Rail. 

The Slow Rail generates the certificate via the following process:
1.  **System Analysis:** It analyzes the current resource routing configurations ($m, \lambda, \rho, \mu, \sigma$ state vectors) and algebraically derives the Jacobian matrix ($J_a$) and non-negative block gain matrix ($G_a$).
2.  **Eigenvalue Search:** Using unbounded iterative algorithms (e.g., power iteration or spectral decompositions), it computes the spectral radius ($\rho(G_a) < 1$) and searches for the strictly positive scaling vector ($d$) to prove local contractivity.
3.  **Packaging the Witness:** It does not send raw instructions or floating-point states back. Instead, it packages these discovered witnesses ($G_{\mathrm{certified}}$, $d$, $\delta$, and drift bounds) into a static, cryptographically digested fixed-point structure—the `AcceptedCertificate`.

## 3. Secure Branchless Verification in the Hot Path (Rule 9, 10 & 18)

When the Slow Rail submits the `AcceptedCertificate`, the Authoritative Hot Path engages the **ReceiptSound Law** and strictly evaluates it in $O(1)$ constant time without a single `if`, `match`, or early return ($CC=1$).

### Branchless Digest Validation
To ensure the cryptographic digest perfectly matches the expected `AdmittedControlState`, the hot path relies on a parallel bitwise XOR cascade rather than control flow:
```rust
let digests_ok = (((state.digest ^ cert.digest)
    | (state.digest ^ env.digest)
    | (state.digest ^ outcome.digest))
    == 0) as u32;
```
If `state.digest` perfectly matches `cert.digest` (the `AcceptedCertificate` digest), the XOR yields `0`. Any bitwise difference generates a non-zero value, which is then cast into a deterministic boolean mask.

### Masked Typed Refusals
If a mismatch is detected, the runtime generates a bounded typed refusal (e.g., `DIGEST_MISMATCH` or `ContractionMarginInsufficient`) strictly using full-width computational masks:
```rust
RefusalSet::DIGEST_MISMATCH.masked(digest_err as u32)
```
This mathematically applies a bitwise AND to union the refusal into a fixed-width `RefusalSet`, avoiding string-based logs or panic paths.

### Packed-Value Inequality Verification and State Selection
The hot path confirms the static domination bounds ($ \widehat G \leq G_{\mathrm{certified}} $ and $ G_{\mathrm{certified}} d \leq (1-\delta)d $) via simple fixed-point matrix-vector multiplication and packed-value comparisons. 

All verification outcomes are aggregated into a master admission mask ($m_{\mathrm{admitted}}$). The runtime then performs a fieldwise masked commit (using operations like `select_nnf`):
*   If all checks pass, the mask evaluates to full-width ones (`0xFFFFFFFF`), committing the new state.
*   If verification fails or the digest is corrupted, the mask evaluates to `0`, the learning mode is frozen, and the pre-existing state is rewritten into memory mathematically, leaving the persistent state bit-for-bit unchanged.
