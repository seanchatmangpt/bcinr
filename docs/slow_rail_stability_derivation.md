Here is the documentation on how the slow rail derives the stability parameters mathematically. 

*(Note: I searched extensively through `tools/`, `scripts/`, and all Python files in the workspace. There is no `cmca-generator` or any Python script currently implementing the power iteration or matrix math for $G, d, \delta$. It appears these values are currently mocked or generated out-of-band by the "Mathematical Architect" as static values in `generated/stability_profile.rs`. The documentation below explains the derivation process as defined in the architectural Markdown files like `cmca_rdf_stability_theorem.md`.)*

# Slow Rail Stability Derivation ($G$, $d$, $\delta$)

Based on the architectural laws and the **CMCA-RDF Stability Theorem**, the system relies on a non-authoritative "slow rail" (an ahead-of-time generator) to compute stability boundaries. The authoritative hot path cannot perform iterative searches (Rule 12), so the slow rail must derive the stability witness—consisting of the block gain matrix ($G$), the witness vector ($d$), and the contraction margin ($\delta$)—before generating the `stability_profile.rs` Rust code.

## Mathematical Derivation (Slow Rail)

The slow rail evaluates the closed-loop dynamics to prove that the system is locally contractive in a weighted block norm.

### 1. Hybrid State and Jacobian Block Gains
The system's adaptive state $x_t$ consists of 5 components: measured masses ($m$), portfolio weights ($\lambda$), resolution state ($\rho$), prices ($\mu$), and standing state ($\sigma$). 
The slow rail models the closed-loop update $F_a(x_t, \xi_t)$ for an admitted mode $a$ and computes the Jacobian matrix at the fixed point: $J_a = D_x F_a(x_a^*, 0)$.

### 2. Constructing the Nonnegative Gain Matrix ($G$)
The slow rail derives the block gains for the five feedback loops:
*   **Mass loop:** $J_{mm} = (1-\beta)I + \beta \frac{\partial\widehat y}{\partial\pi} \frac{\partial\pi}{\partial m}$
*   **Portfolio loop:** Involves the learning rate $\zeta$
*   **Price loop:** Involves the adaptation rate $\gamma$

It constructs the non-negative block gain matrix $G_a = [G_{ij}] \in \mathbb{R}_{\geq 0}^{5\times 5}$, where each $G_{ij}$ bounds the influence of state block $j$ on state block $i$.

### 3. Spectral Radius and Power Iteration
To prove that the system is stable (contractive), the slow rail must verify that the spectral radius of the gain matrix is less than one:
$$ \rho(G_a) < 1 $$

Since the slow rail is not bound by the Radon Law ($CC=1$) and zero-allocation constraints, it is permitted to use unbounded, iterative algorithms like **power iteration** or **eigenvalue decomposition** to compute the spectral radius of $G_a$.

### 4. Deriving the Witness Vector ($d$) and Contraction Margin ($\delta$)
Using the spectral radius, the slow rail searches for a strictly positive scaling vector (eigenvector or weighted norm vector) $d$ such that:
$$ G_a d \leq (1 - \delta) d $$
where $\delta > 0$ is the contraction margin. 

### 5. Rust Code Generation
Once $G_a$, $d$, and $\delta$ are computed and the inequalities are satisfied, the slow rail extracts the maximum permitted learning and noise rates (e.g., maximum noise radius $\beta_m$ and maximum learning rate $\zeta_w$). These bounds are serialized into fixed-point constants like `BETA_M_MAX` and `ZETA_W_MAX` and written to `crates/bcinr-cmca/src/generated/stability_profile.rs`.

The hot path then blindly imports these constants as `NonNegativeFixed` (e.g., `PROFILE.certified_noise_radius`) and performs branchless $O(1)$ assertions (using `const_lt_u32` masks) to enforce the mathematical bounds, returning a `StabilityRefusal` if breached.
