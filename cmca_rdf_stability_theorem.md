# CMCA-RDF Stability Theorem

This document defines the mathematical boundary for closed-loop stability in CMCA-RDF.
The product is not fully an organism until it has:

[
\boxed{
\text{immune system}
+
\text{memory}
+
\text{homeostasis}
}
]

Those correspond exactly to:
1. **Admitted control state** — what is allowed to influence allocation.
2. **Receipted allocation artifacts** — what the system decided and why.
3. **Closed-loop stability** — whether learning improves allocation or collapses it.

## 1. The correct state model

The adaptive state should be written as:

[
x_t
===

\begin{bmatrix}
m_t\
\lambda_t\
\rho_t\
\mu_t\
\sigma_t
\end{bmatrix}.
]

Where:
* $m_t$: measured operational masses;
* $\lambda_t$: q-lens or local-expert portfolio weights;
* $\rho_t$: resolution or flatten/descend state;
* $\mu_t$: vector-resource prices;
* $\sigma_t$: standing state influencing valuation.

The closed loop is:
$x_{t+1}=F_a(x_t,\xi_t),$
where $a$ is the currently admitted control-state mode, and $\xi_t$ represents receipted environmental observations.

## 2. The first theorem is a within-mode contraction theorem

Fix one admitted mode $a$. Let $x_a^*$ be a fixed point: $F_a(x_a^*,0)=x_a^*.$
Define the Jacobian: $J_a = D_xF_a(x_a^*,0).$

The correct implementation theorem should use a **block gain matrix**.

Let:
[
J_a
===
\begin{bmatrix}
J_{mm} & J_{m\lambda} & J_{m\rho} & J_{m\mu} & J_{m\sigma}\
J_{\lambda m} & J_{\lambda\lambda} & J_{\lambda\rho} & J_{\lambda\mu} & J_{\lambda\sigma}\
J_{\rho m} & J_{\rho\lambda} & J_{\rho\rho} & J_{\rho\mu} & J_{\rho\sigma}\
J_{\mu m} & J_{\mu\lambda} & J_{\mu\rho} & J_{\mu\mu} & J_{\mu\sigma}\
J_{\sigma m} & J_{\sigma\lambda} & J_{\sigma\rho} & J_{\sigma\mu} & J_{\sigma\sigma}
\end{bmatrix}.
]

Define block gains: $G_{ij} = \left| W_iJ_{ij}W_j^{-1} \right|,$ for positive block-scaling matrices $W_i$.

Then $G_a = [G_{ij}] \in\mathbb R_{\geq0}^{5\times5}.$

The practical theorem becomes:
[
\boxed{
\rho(G_a)<1 \Longrightarrow F_a \text{ is locally contractive in a weighted block norm}.
}
]

## 3. The five feedback loops

### Mass loop
$m \rightarrow \pi \rightarrow y \rightarrow m.$
The update is: $m_{t+1} = (1-\beta)m_t + \beta\widehat y_t.$
Its local derivative contains: $J_{mm} = (1-\beta)I + \beta \frac{\partial\widehat y}{\partial\pi} \frac{\partial\pi}{\partial m}.$

### Portfolio loop
$\lambda \rightarrow \pi \rightarrow \widehat R \rightarrow \lambda.$
For multiplicative weights: $\lambda_{q,t+1} \propto \lambda_{q,t} e^{\zeta\widehat R_{q,t}}.$

### Resolution loop
$\rho \rightarrow \pi \rightarrow \widehat Y_{\mathrm{flat/desc}} \rightarrow \rho.$
This loop should be active only when $\kappa_q(v)>\epsilon_\kappa.$

### Price loop
$\mu \rightarrow \pi \rightarrow C\pi-b \rightarrow \mu.$
The update is: $\mu_{t+1} = \left[ \mu_t + \gamma(C\pi_t-b) \right]_+.$

### Standing loop
$\pi \rightarrow \text{proof effort} \rightarrow \sigma \rightarrow z \rightarrow m \rightarrow \pi.$
The rigorous architecture should distinguish $\sigma=\text{typed standing mode}$ from $s=\text{continuous standing-derived allocation factor}.$

## 4. The real theorem must be hybrid
A single Jacobian theorem is insufficient because admitted RDF state and typed standing can change.
### Common-contraction strategy
Find one weighted norm such that $\sup_{a\in\mathcal A} \left| D F_a \right|_W \leq 1-\delta$.
### Dwell-time strategy
Require average dwell-time condition: $N_{\mathrm{switch}}(0,T) \leq N_0+\frac{T}{\tau_D}.$

## 5. The anti-collapse components become theorem hypotheses
The flywheel does not arise automatically. The homeostatic apparatus includes:
* **Global leaf floor**: $\pi_t(x) \geq \frac{\eta_g}{|X|}.$
* **Negative-q wing**: Counteracts mass loss ($m_i\downarrow \Longrightarrow L_q(i)\uparrow$ for $q<0$).
* **Optimism for cold cells**: Prevents starving unobserved cells.
* **Clamps**: $m_{\min}\leq m_i\leq m_{\max}$, $q_{\min}\leq q\leq q_{\max}$, $0\leq\mu_j\leq\mu_{\max,j}$.
* **Scale suppression**: If $\kappa_q(v)\leq\epsilon_\kappa$, disable local resolution learner.

## 6. CMCA Local Homeostasis Theorem
For a fixed admitted mode $a$, assume bounded parameters and positive floor.
If the resulting gain matrix obeys $\rho(G_a)<1$, then there exists a weighted block norm and neighborhood $U_a$ such that $|F_a(x)-F_a(y)|_W \leq (1-\delta) |x-y|_W$.
Consequently, the admitted CMCA state converges locally and exponentially toward its fixed point.

## 7. The flywheel corollary
[
\boxed{
\text{bounded adaptation} + \text{stable feedback} + \text{positive verified-value gradient} \Rightarrow \text{self-improvement}.
}
]

## Work Package Decision
1. define the hybrid state and admitted modes;
2. derive the five block update maps;
3. calculate or bound every block gain;
4. construct the nonnegative gain matrix (G_a);
5. solve for weighted norms minimizing (|G_a|);
6. derive sufficient learning-rate inequalities;
7. prove within-mode contraction;
8. add switching-state conditions;
9. derive the stable-flywheel and anti-collapse corollaries;
10. turn the inequalities into runtime configuration gates.
