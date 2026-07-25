# Slow Rail Symbolic Mathematics in BCINR

In the `bcinr` deterministic substrate, continuous adaptation and stability analysis present a fundamental architectural conflict. Mathematical stability requires discovering eigenvalues and tracking spectral radius, but iterative mathematical algorithms strictly violate the authoritative runtime laws. 

BCINR resolves this by partitioning the architecture into two strictly isolated domains governed by Rules 6 and 12: the **Slow Rail** (which discovers theorems using symbolic/unbounded math) and the **Hot Path** (which verifies theorems using $O(1)$ branchless execution).

## 1. The Constitutional Division (Rules 6 & 12)

### Rule 6: Authoritative versus Non-Authoritative Code
According to Rule 6, the system is divided into:
- **The Authoritative Runtime (Hot Path)**: Inherits absolute runtime laws. It is zero-allocation, fixed-width, and entirely branchless ($CC=1$).
- **The Slow Rail**: Responsible for symbolic mathematics, eigenvalue search, code generation, and certificate derivation. The Slow Rail is permitted to branch, allocate, and use floating-point mathematics, provided it is **never linked into or invoked from the authoritative hot path**.

### Rule 12: No Runtime Theorem Discovery
Rule 12 expressly forbids the Authoritative Runtime from discovering mathematical witnesses. Prohibited runtime operations include:
- Spectral-radius estimation
- Power iteration
- Jacobian derivation
- Optimization over weighting vectors

Instead, the Hot Path is restricted to *verifying* a supplied static witness using fixed-width operations. 

## 2. Slow Rail Eigenvalue Search and Derivation

Because the Hot Path cannot execute data-dependent loops, the Slow Rail takes on the mathematical heavy lifting asynchronously:

1. **Jacobian and Block Gain Construction**: The Slow Rail analyzes the current resource routing configurations (the $m, \lambda, \rho, \mu, \sigma$ state vectors) to algebraically derive the system's Jacobian ($J_a$) and construct the non-negative block gain matrix ($G_a$).
2. **Eigenvalue Search (Theorem Discovery)**: Using unbounded, iterative algorithms like power iteration or spectral decompositions, the Slow Rail calculates the spectral radius ($\rho(G_a)$) to prove local contractivity ($\rho(G_a) < 1$). It also calculates bounds like the Gram Distinguishability Lower Bound ($\underline\gamma_{\min}^{+}$).
3. **Witness Construction ($G, d, \delta$)**: The Slow Rail searches for and derives the static domination witnesses required by the Hot Path:
   - **$G$ (Certified Gain Matrix)**: The bounded comparison matrix ($G_{\mathrm{certified}}$).
   - **$d$ (Scaling Vector)**: A strictly positive right eigenvector or scaling vector acting as a weighted block norm.
   - **$\delta$ (Contraction Margin)**: The required mathematical margin proving strictly contractive stability.

## 3. Packaging the `AcceptedCertificate`

The Slow Rail does not send raw instructions or floating-point states back to the runtime. It packages the pre-computed static bounds ($G_{\mathrm{certified}}$, $d$, $\delta$, and drift bounds $R_{\mathrm{noise}}$, $R_{\mathrm{switch}}$) into a cryptographic witness known as the `AcceptedCertificate`.

## 4. Hot Path $O(1)$ Packed-Value Verification

Once the `AcceptedCertificate` is submitted, the Hot Path applies the **ReceiptSound Law**. It uses strict $O(1)$ fixed-point (e.g. Q16.16) arithmetic to verify the static domination law without a single branch.

The runtime verifies the static domination mathematically:
$$ \widehat G \leq G_{\mathrm{certified}} $$
$$ G_{\mathrm{certified}} d \leq (1-\delta)d $$

Through elementwise masked operations and packed-value comparisons, the Hot Path confirms the witness. 
- If the arithmetic holds and the certificate envelope is valid, the admission mask ($m_{\mathrm{admitted}}$) evaluates to full-width ($1$), authorizing the state commit. 
- If the inequality fails (e.g., the contraction margin is insufficient), the runtime yields a branchless typed refusal (e.g., `ContractionMarginInsufficient`), forces the mask to $0$, and safely leaves the system state bit-for-bit unchanged. 

By offloading the theorem *discovery* to the Slow Rail while forcing the Hot Path to exclusively perform branchless *verification*, BCINR maintains its deterministic, zero-branch integrity while continuously adapting.
