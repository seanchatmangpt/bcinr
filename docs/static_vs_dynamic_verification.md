# Rule 12 (No Runtime Theorem Discovery): Static vs Dynamic Verification Architecture

According to **Rule 12** of the BCINR Deterministic Substrate Constitution, the authoritative runtime (the "hot path") is strictly prohibited from discovering or deriving stability parameters. Its sole responsibility is to execute deterministic, branchless verification of a fixed witness.

The architecture strictly separates dynamic theorem exploration (which requires iterative branches and memory allocations) from static verification (which is constrained to $CC=1$ and zero heap allocations).

## 1. Dynamic Exploration (The Slow Rail)

The "slow rail" is an asynchronous, non-authoritative environment that is explicitly permitted to branch, allocate memory, and execute unbounded computations. 

It handles all active discovery algorithms prohibited on the hot path:
*   **Theorem Discovery:** Executes complex algorithms such as spectral-radius estimation, power iteration, Jacobian derivation, Lyapunov search, and optimization over weighting vectors.
*   **Eigenvalue Calculation:** Computes the spectral radius ($\rho(G_a) < 1$) and the conservative lower bound on the smallest positive eigenvalue of the Gram matrix ($\underline\gamma_{\min}^{+}$).
*   **Witness Construction:** Identifies and derives the exact static domination parameters required for stability:
    *   $G_{\mathrm{certified}}$: The verified gain / comparison matrix
    *   $d$: The strictly positive right eigenvector or scaling vector (fixed witness)
    *   $\delta$: The contraction margin
*   **Certificate Generation:** Packages these structural findings into a static, cryptographically digested **Witness** (the `AcceptedCertificate` or `MeasurementArtifact`).

## 2. Static Verification (The Hot Path / `bcinr-cmca` & `bcinr-logic`)

The hot path cannot run loop-dependent mathematical algorithms. Instead, the logic natively *verifies* the fixed witness provided by the slow rail using the constant-time mechanics and bitwise abstractions (such as those provided by `bcinr-logic`). 

### Verifying Static Domination ($G \cdot d \le (1 - \delta) \cdot d$)

Instead of computing the spectral radius of the state transition graph dynamically, the substrate evaluates the fixed witness using packed values to branchlessly enforce two static domination conditions:
$$ \widehat G \leq G_{\mathrm{certified}} $$
$$ G_{\mathrm{certified}} d \leq (1-\delta)d $$

In `crates/bcinr-cmca/src/stability.rs`, this is implemented as:
*   **Fixed-Bounded Execution:** Verification operates over a compile-time fixed matrix dimension (e.g., `DIM = 2`) to avoid variable iteration loops (Rule 13).
*   **$O(1)$ Fixed-Point Arithmetic:** Inputs are handled as `Q16.16` scaled fixed-point integers. The matrix-vector multiplication $G \cdot d$ is computed precisely by temporarily upscaling to `i128` during multiplication to avoid overflow, all without branches.
*   **Elementwise Domination Check:** The calculation $(1 - \delta) d$ is evaluated against $G \cdot d$. If the check fails, the execution explicitly issues a typed refusal (`StabilityDerivationRefusal::ContractionMarginInsufficient`).
*   **Independent Recomputation:** When minting a cryptographic `CertificateReceipt` (`crates/bcinr-cmca/src/certification.rs`), the runtime refuses to trust previous boolean assertions. It invokes a `witness_holds` check to *independently recompute* the static domination law directly from the candidate's inner fields.

### Branchless Eigenvalue Constraints

Eigenvalue lower bounds are evaluated without search or optimization iterations:
*   The minimum positive Gram eigenvalue ($\underline\gamma_{\min}^{+}$) is supplied by the slow rail via the `MeasurementArtifact` (`artifact.gram_lower_bound`), rather than discovered on the hot path.
*   The telemetry engine applies a branchless bitwise comparison (e.g., `const_lt_u32(gamma_min_plus_under, epsilon_gram)`) to check this bound against the stability limit.
*   The result is evaluated purely via bitwise `&` logic to form aggregated safety flags (like `is_gram_degenerate`), eliminating any conditional jumps.

## Summary

By stripping iterative discovery operations out of the execution layer, BCINR enforces structural constraints, contraction margins, and eigenvalue limits purely mathematically. 

This architecture guarantees that the substrate resolves complex stability theorems dynamically off-path, while executing them on-path with absolute structural determinism, branchless $CC=1$ cyclomatic complexity, and bounded $O(1)$ mechanics.
