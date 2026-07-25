# Rule 12: Theorem Constraints and Stability Parameters in BCINR

In accordance with **Rule 12 (No Runtime Theorem Discovery)** of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the authoritative runtime (the "hot path") is strictly prohibited from running bounded/iterative algorithms (e.g., spectral-radius estimation, power iteration, Lyapunov search). These algorithms violate the Radon Law ($CC=1$) and bounded execution mandates. 

Instead, BCINR forces all mathematical "discovery" of stability properties onto the **Slow Rail**, which derives a mathematical witness. The hot path is then restricted solely to $O(1)$, branchless *verification* of this witness via packed values.

Here is the exact mathematical structure of how $G$, $d$, $\delta$, $R_{\mathrm{noise}}$, and $R_{\mathrm{switch}}$ are derived and verified.

## 1. Slow Rail Derivation

The **Slow Rail** is an asynchronous, non-authoritative execution environment permitted to use unbounded loops, memory allocation, and floating-point math to deduce the closed-loop system dynamics.

### $G$ (Block Gain Matrix / Comparison Matrix Bound)
The slow rail models the closed-loop update function $F_a(x_t, \xi_t)$ for an admitted mode $a$ over the adaptive state vectors (masses $m$, portfolio weights $\lambda$, resolution state $\rho$, prices $\mu$, and standing state $\sigma$). 
- It computes the Jacobian matrix at the fixed point: $J_a = D_x F_a(x_a^*, 0)$.
- From this, it constructs $G_a = [G_{ij}] \in \mathbb{R}_{\geq 0}^{5\times 5}$, a non-negative block gain matrix where each $G_{ij}$ conservatively bounds the influence of state block $j$ on state block $i$.

### $d$ (Witness Vector / Eigenvector)
Using unbounded eigenvalue search algorithms (like power iteration or spectral decomposition), the slow rail first calculates the spectral radius of the gain matrix, proving the system is locally contractive ($\rho(G) < 1$). 
- Once contraction is proven, the slow rail derives a strictly positive right eigenvector or scaling vector $d$. 
- This vector acts as a weighted block norm across the five state dimensions.

### $\delta$ (Contraction Margin)
Paired with $d$, the slow rail derives the contraction margin $\delta > 0$. The slow rail ensures that these parameters strictly satisfy the contraction mapping proof:
$$ G d \leq (1-\delta)d $$
This proves mathematically that multiplying the system state by the gain matrix shrinks the state by a minimum guaranteed margin of $\delta$.

### $R_{\mathrm{noise}}$ & $R_{\mathrm{switch}}$ (Drift and Threshold Bounds)
The slow rail also discovers threshold boundaries for environmental interaction:
- **$R_{\mathrm{noise}}$ (Stationary Drift Bound)**: Discovers the maximum permitted noise or variance parameters (e.g., maximum noise radius $\beta_m$) the system can tolerate while remaining stable.
- **$R_{\mathrm{switch}}$ (Switching Drift Bound)**: Discovers hybrid-system reset bounds, mode transition limits, and dwell-time constraints ($\tau_{D,a}$) to preserve stability when control modes are changed.

---

## 2. Packaging into the `AcceptedCertificate`

The Slow Rail does not pass executable logic, raw formulas, or floating-point instructions back to the runtime. Instead, it serializes its findings into a mathematically rigid **Witness**—the `AcceptedCertificate`.

1. **Fixed-Point Conversion:** The matrices and bounds are mapped into fixed-width values (e.g., Q16.16 arithmetic, `NonNegativeFixed`) and statically compiled into `generated/stability_profile.rs`. 
2. **Total Digest Binding:** The derived witness is bound by a cryptographic digest ($H_a$) covering 11 domain-specific identities (such as the admitted graph, the numeric profile, pricing laws, and the generated kernel implementation itself).

The compiled representation typically looks like this:
```rust
pub const GAIN_MATRIX: [[Fixed; 5]; 5] = /* derived G */;
pub const WEIGHT_VECTOR: [Fixed; 5] = /* derived d */;
pub const CONTRACTION_MARGIN: Fixed = /* derived \delta */;
pub const ENVELOPE: StabilityEnvelope = /* contains R_noise, R_switch */;
pub const CERTIFICATE_DIGEST: Digest = /* derived hash */;
```

---

## 3. Hot Path Branchless Verification

When the hot path executes, it adheres strictly to the **ReceiptSound Law** (`AGENTS.md` §11). To authorize a state mutation, it must have a valid `AcceptedCertificate`. 

Instead of searching for eigenvalues, the runtime verifies the static domination mathematically using $O(1)$, branchless element-wise fixed-point arithmetic:
1. $$ \widehat G \leq G_{\mathrm{certified}} $$
2. $$ G_{\mathrm{certified}} d \leq (1-\delta)d $$

**Determinism via Masks:**
The runtime uses loop-unrolled SIMD-like multiplication and bitwise comparisons. If the state matrix fails the check, or if a structural digest binding mismatches, the boolean failures are branchlessly converted into two's-complement bitmasks. 

If validation fails, the admission mask evaluates to $0$. The fallback logic freezes learning (`CMCA_LEARNING_FROZEN`) and executes a masked state selection, leaving the adaptive continuous state bit-for-bit unchanged without executing a single `if` statement.
