# Slow Rail's Symbolic Eigenvalue Search

In the `bcinr` deterministic substrate, closed-loop adaptive stability requires continuous monitoring of the system's spectral radius and matrix eigenvalues. However, calculating eigenvalues is an iterative, branching process that fundamentally violates the substrate's constitutional laws. 

Here is how BCINR securely bridges this gap using the **Slow Rail** and statically verifiable **AcceptedCertificates**.

## 1. The Constitutional Prohibition (Hot Path)
According to the BCINR Constitution (`AGENTS.md`, Section 12: *No runtime theorem discovery*), the deterministic **Hot Path** is expressly forbidden from executing:
- Spectral-radius estimation
- Power iteration
- Jacobian derivation
- Eigenvalue search

Because the Authoritative Runtime operates under the **Radon Law ($CC=1$)**, it cannot execute data-dependent loops or iterative convergence algorithms. The Hot Path is strictly restricted to $O(1)$ fixed-width masked arithmetic.

## 2. Structural/Symbolic Calculation on the Slow Rail
To solve this without violating Hot Path constraints, the iterative mathematical heavy lifting is relegated to the **Slow Rail**—an asynchronous, branching-allowed environment that utilizes floating-point or arbitrary-precision math.

### A. Constructing the Block Gain Matrix
The Slow Rail analyzes the current resource routing configurations (the $m, \lambda, \rho, \mu, \sigma$ state vectors) for a given admitted mode and algebraically derives the Jacobian matrix $J_a$. From this, it constructs the non-negative block gain matrix $G_a$.

### B. Eigenvalue Search and Spectral Radius
Using unrestricted algorithms (like power iteration or spectral decompositions), the Slow Rail computes:
- **The Spectral Radius ($\rho(G_a)$):** It ensures that $\rho(G_a) < 1$, mathematically proving the system is locally contractive within the mode.
- **The Gram Distinguishability Lower Bound ($\underline\gamma_{\min}^{+}$):** As defined in the `CMCA-RDF \kappa_q Estimator`, the Slow Rail calculates the conservative lower bound on the smallest positive eigenvalue of the Gram matrix. Learner activation strictly requires $\underline\gamma_{\min}^{+} \geq \epsilon_{\mathrm{gram}}$.
- **Weighted Norms:** It searches for and constructs a strictly positive right eigenvector $d$ (or scaling vector $w$) that acts as a weighted block norm, satisfying $G_a d < d$.

## 3. The `AcceptedCertificate` (The Witness)
The Slow Rail does not pass executable code or raw floating-point instructions back to the Hot Path. Instead, it packages its structural findings into a static, cryptographically digested **Witness**—the `AcceptedCertificate`.

This certificate contains only the pre-computed, fixed-point bounds:
- $G_{\mathrm{certified}}$ (The certified gain matrix bound)
- $d$ (The scaling vector / eigenvector)
- $\delta$ (The contraction margin)
- $R_{\mathrm{noise}}$ and $R_{\mathrm{switch}}$ (Stationary and switching drift bounds)

## 4. Secure Hot Path Verification (The ReceiptSound Law)
When the Slow Rail submits this `AcceptedCertificate` to the Authoritative Runtime, the Hot Path engages the **ReceiptSound Law** (`AGENTS.md`, Section 11 and `receipt_sound_property.md`).

The Hot Path does **not** search for eigenvalues. Instead, it branchlessly *verifies* the supplied witness using simple fixed-point matrix-vector multiplication and packed value comparisons:

$$ \widehat G \leq G_{\mathrm{certified}} $$
$$ G_{\mathrm{certified}} d \leq (1-\delta)d $$

If the Hot Path's $O(1)$ arithmetic confirms these inequalities, and the state digest matches the certificate's envelope, the admission mask ($m_{\mathrm{admitted}}$) evaluates to full width ($1$). If the inequalities fail, or if the certificate is missing, the mask evaluates to $0$. This mathematically freezes learning (`LearningFrozen`) and leaves the state bit-for-bit unchanged, enforcing safe homeostasis without a single `if` statement.
