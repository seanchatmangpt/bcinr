# CMCA-RDF Stability Theorem Proof

This document provides the formal derivation of the 10-step CMCA-RDF Stability Theorem package, bounding the closed-loop stability of the adaptive system.

## 1. Hybrid State and Admitted Modes

The adaptive state $x_t \in \mathcal{X}$ is a hybrid state consisting of continuous learning variables and discrete configuration modes. We define the state vector as:

$$ x_t = \begin{bmatrix} m_t \\ \lambda_t \\ \rho_t \\ \mu_t \\ \sigma_t \end{bmatrix} $$

Where:
* $m_t \in [m_{\min}, m_{\max}]^N$: measured operational masses (resource traces).
* $\lambda_t \in \Delta^{|Q|-1}$: q-lens or local-expert portfolio weights, operating on the probability simplex.
* $\rho_t \in \mathcal{R}$: resolution or flatten/descend continuous state.
* $\mu_t \in [0, \mu_{\max}]^K$: vector-resource prices (dual variables for capacity constraints).
* $\sigma_t \in \Sigma$: standing state influencing valuation, mapped to a continuous allocation factor $s_t$.

An **admitted mode** $a \in \mathcal{A}$ defines the current valid subset of states, allowable actions, and discrete typings (like the typed standing mode). The closed-loop dynamics within a fixed mode $a$ are governed by:
$$x_{t+1} = F_a(x_t, \xi_t)$$
where $\xi_t$ represents the receipted environmental observations and performance metrics.

## 2. Block Update Maps

For a given admitted mode $a$, we isolate the five unperturbed feedback loops $x_{t+1} = F_a(x_t, 0)$. Let $\pi_t = \pi(m_t, \lambda_t, \rho_t, \sigma_t)$ be the unified allocation policy.

1. **Mass loop:** Exponential moving average of target operational masses $\hat{y}_t$.
   $$ m_{t+1} = (1-\beta)m_t + \beta \hat{y}(x_t) $$
2. **Portfolio loop:** Exponentiated gradient ascent on receipted rewards $\hat{R}$.
   $$ \lambda_{q,t+1} = \frac{\lambda_{q,t} \exp(\zeta \hat{R}_{q}(x_t))}{\sum_{q'} \lambda_{q',t} \exp(\zeta \hat{R}_{q'})} $$
3. **Resolution loop:** Gradient descent mapping for flatten/descend operations, gated by scale.
   $$ \rho_{t+1} = \rho_t + \alpha_\rho \nabla_\rho \hat{Y}_{\mathrm{flat/desc}}(x_t) \cdot \mathbf{1}_{\{\kappa_q(v) > \epsilon_\kappa\}} $$
4. **Price loop:** Dual projected subgradient descent on constraint violations $C\pi_t - b$.
   $$ \mu_{t+1} = \left[ \mu_t + \gamma (C \pi(x_t) - b) \right]_+ $$
5. **Standing loop:** Dynamics of the continuous standing-derived allocation factor $s_t$, assumed to follow a smooth integration of proof effort.
   $$ \sigma_{t+1} = \sigma_t + \alpha_\sigma (\text{effort}(x_t) - \sigma_t) $$

## 3. Block Gain Bounds

Let $x_a^*$ be a fixed point of $F_a(\cdot, 0)$. We compute the Jacobian block matrix $J_a = D_x F_a(x_a^*, 0)$. Let $J_{ij} = \frac{\partial (x_{t+1})_i}{\partial (x_t)_j}$. We bound the block norms $G_{ij} = \| W_i J_{ij} W_j^{-1} \|$.

Assuming Lipschitz continuity of the underlying maps ($\hat{y}, \hat{R}, \pi, \dots$) with constants $L_{i,j}$:

* **Mass Block:** 
  $J_{mm} = (1-\beta)I + \beta \frac{\partial\hat{y}}{\partial m} \implies G_{mm} \le 1 - \beta + \beta L_{y,m}$
  $G_{mj} \le \beta L_{y,j}$ for $j \in \{\lambda, \rho, \mu, \sigma\}$
* **Portfolio Block:** 
  $J_{\lambda\lambda} \approx I + \zeta \text{diag}(\lambda) \nabla^2 \hat{R} \implies G_{\lambda\lambda} \le 1 + \zeta L_{R,\lambda}$
  $G_{\lambda j} \le \zeta L_{R,j}$ for $j \neq \lambda$
* **Resolution Block:** 
  $G_{\rho\rho} \le 1 - \alpha_\rho c_\rho$ (assuming strong convexity/contraction locally)
  $G_{\rho j} \le \alpha_\rho L_{\rho, j}$
* **Price Block:** 
  $G_{\mu\mu} \le 1$ (projection operator is non-expansive)
  $G_{\mu j} \le \gamma L_{C,j}$ (where $L_{C,j} = \|C\| L_{\pi,j}$)
* **Standing Block:**
  $G_{\sigma\sigma} \le 1 - \alpha_\sigma$
  $G_{\sigma j} \le \alpha_\sigma L_{\sigma, j}$

## 4. Construction of the Nonnegative Gain Matrix $G_a$

The bounds yield a $5 \times 5$ non-negative matrix $G_a \in \mathbb{R}_{\ge 0}^{5 \times 5}$:

$$ G_a = \begin{bmatrix}
1 - \beta(1 - L_{y,m}) & \beta L_{y,\lambda} & \beta L_{y,\rho} & \beta L_{y,\mu} & \beta L_{y,\sigma} \\
\zeta L_{R,m} & 1 - \zeta(1 - L_{R,\lambda}) & \zeta L_{R,\rho} & \zeta L_{R,\mu} & \zeta L_{R,\sigma} \\
\alpha_\rho L_{\rho,m} & \alpha_\rho L_{\rho,\lambda} & 1 - \alpha_\rho c_\rho & \alpha_\rho L_{\rho,\mu} & \alpha_\rho L_{\rho,\sigma} \\
\gamma L_{C,m} & \gamma L_{C,\lambda} & \gamma L_{C,\rho} & 1 & \gamma L_{C,\sigma} \\
\alpha_\sigma L_{\sigma,m} & \alpha_\sigma L_{\sigma,\lambda} & \alpha_\sigma L_{\sigma,\rho} & \alpha_\sigma L_{\sigma,\mu} & 1 - \alpha_\sigma
\end{bmatrix} $$

## 5. Solving for Weighted Norms Minimizing $|G_a|$

By the Perron-Frobenius Theorem for non-negative matrices, if the spectral radius $\rho(G_a) < 1$, there exists a strictly positive right eigenvector $w = [w_m, w_\lambda, w_\rho, w_\mu, w_\sigma]^T \in \mathbb{R}_{>0}^5$ such that $G_a w < w$.

We define the block-scaling matrices $W_i = w_i^{-1} I$. The weighted block norm is defined as:
$$ \| x \|_W = \sum_{i \in \{m,\lambda,\rho,\mu,\sigma\}} w_i^{-1} \| x_i \| $$
Under this specific weighted norm, the induced matrix norm of the Jacobian is bounded by the spectral radius:
$$ \| J_a \|_W \le \| G_a \|_{\infty, W} < 1 $$

## 6. Sufficient Learning-Rate Inequalities

To ensure $\rho(G_a) < 1$ and $G_a w < w$, the learning rates must be sufficiently small to restrict the off-diagonal coupling (cross-loop interference). The inequalities derived from $G_a w < w$ are:

1. $\beta < \frac{w_m - \sum_{j \neq m} G_{mj} w_j}{w_m(1 - L_{y,m})}$
2. $\zeta < \frac{w_\lambda - \sum_{j \neq \lambda} G_{\lambda j} w_j}{w_\lambda(L_{R,\lambda} - 1)}$ (assuming $L_{R,\lambda} > 1$)
3. $\alpha_\rho < \frac{w_\rho - \sum_{j \neq \rho} G_{\rho j} w_j}{c_\rho w_\rho}$
4. For prices, since $G_{\mu\mu}=1$, we require strictly contractive decay fed from other blocks, imposing a strict upper bound on $\gamma \sum_{j \neq \mu} L_{C,j} w_j$.

## 7. Within-Mode Contraction

**Theorem (Local Contraction):**
If the learning-rate inequalities hold, $\rho(G_a) \le 1 - \delta$ for some $\delta > 0$. 
By the definition of the weighted block norm, the Jacobian satisfies $\|D_x F_a(x_a^*, 0)\|_W \le 1 - \delta$. 
In a neighborhood $U_a$ around $x_a^*$, $F_a$ acts as a strict contraction mapping:
$$ \| F_a(x) - F_a(y) \|_W \le (1-\delta) \| x - y \|_W $$
Consequently, the admitted state $x_t$ converges locally and exponentially to $x_a^*$.

## 8. Switching-State Conditions

When the system switches between admitted modes $a \to a'$, the fixed points and Jacobians change. 
To guarantee global stability across mode transitions, we impose an **average dwell-time condition**.

Let $N_{\mathrm{switch}}(0, T)$ be the number of mode switches in the interval $[0, T]$. If the norms $W_a$ are mode-dependent, there exists a uniform bound $\mu \ge \sup_{a, a', x} \frac{\|x\|_{W_a}}{\|x\|_{W_{a'}}}$. 
Global stability is preserved if:
$$ N_{\mathrm{switch}}(0, T) \le N_0 + \frac{T}{\tau_D} $$
where the dwell time $\tau_D$ satisfies $\tau_D > \frac{\ln \mu}{\ln(1/(1-\delta))}$. This ensures that the continuous contraction phase strictly outpaces the potential expansion caused by discrete norm-switching.

## 9. Stable-Flywheel and Anti-Collapse Corollaries

**Corollary 1 (Anti-Collapse Homeostasis):**
The system state $x_t$ is strictly prevented from boundary collapse if the homeostatic clamps are active:
* **Global leaf floor:** $\pi_t(x) \ge \frac{\eta_g}{|X|}$ strictly bounds mass away from $0$.
* **Clamps:** $m_i \in [m_{\min}, m_{\max}]$ and $\mu_j \le \mu_{\max,j}$.
* **Negative-q wing:** Counteracts arbitrary mass decay.
Thus, $x_t$ remains bounded within a safe set $\mathcal{X}_{\mathrm{safe}} \subset \text{int}(\mathcal{X})$.

**Corollary 2 (Stable-Flywheel):**
Given (1) Anti-collapse (bounded adaptation), (2) Within-mode contraction (stable feedback), and (3) a positive verified-value gradient (the $\lambda$ update strictly ascends expected objective $\hat{R}$), the closed-loop system must monotonically accumulate expected value, leading to a mathematically guaranteed self-improving flywheel.

## 10. Runtime Configuration Gates

We translate the theoretical bounds into runtime constraints for the CMCA-RDF engine:

1. **Floor Assertion Gate:** At every allocation step, assert `min(pi) >= eta_g / |X|`.
2. **Dynamic Rate Limiter:** Clip learning rates dynamically. `beta = min(beta_config, BETA_MAX(L_y))` based on runtime gradient estimates.
3. **Resolution Suppression Gate:** If the spatial scale `kappa_q(v) <= epsilon_kappa`, set `alpha_rho = 0` (disable the local resolution learner).
4. **Dwell Time Lock:** Enforce a minimum timestamp delta between mode switches: `if (t - last_switch_t < tau_D) reject_mode_switch()`.
5. **Clamp Enforcer:** Run post-update projections `m = clip(m, m_min, m_max)` and `mu = clip(mu, 0, mu_max)`.
