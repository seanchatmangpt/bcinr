# CMCA Cascade Allocator Mathematical Formulas

The Chatman Multifractal Cascade Allocation (CMCA) framework distributes semantic resources down a hierarchical forest of nodes using a multiplicative weights mechanism designed to adapt to environmental noise while maintaining safe exploration bounds. All calculations are structurally executed via zero-allocation branchless logic using Q16.16 fixed-point arithmetic.

## 1. Flow Propagation

For each node $v$, the incoming flow is split into a direct leaf allocation part ($F_v$) and a child propagation part ($D_v$):
$$ F_v = (1 - \rho_v) \cdot \text{alloc\_flow}[v] $$
$$ D_v = \rho_v \cdot \text{alloc\_flow}[v] $$
where $\rho_v \in [0, 1]$ is the local routing parameter.

The direct part $F_v$ is distributed to all descendant leaves:
$$ \text{flat\_alloc}[x] \leftarrow \text{flat\_alloc}[x] + F_v \cdot \frac{W_{\text{leaf}}(v, x)}{\sum_{y \in \text{leaves}(v)} W_{\text{leaf}}(v, y)} $$

The descendant part $D_v$ is distributed to direct children:
$$ \text{alloc\_flow}[c] \leftarrow \text{alloc\_flow}[c] + D_v \cdot \frac{W_{\text{child}}(v, c)}{\sum_{d \in \text{children}(v)} W_{\text{child}}(v, d)} $$

## 2. Multiplicative Weights Update (MWU) and Entropy

For each internal node $v$, routing weights between direct leaf allocation and child propagation are adjusted dynamically.

### Local Divergence Metric (Relative Entropy)
The updates are controlled by a local divergence metric $\kappa_v$ representing relative entropy:
$$ \kappa_v = \sum_{c \in \text{children}(v)} s_{\text{leaf}}(c) \cdot \log_2\left( \frac{s_{\text{leaf}}(c)}{s_{\text{meas}}(c)} \right) $$
*(Alternatively structured in continuous formulations as $\kappa_v = \operatorname{clip}\left( \frac{F_v \cdot S_v}{E_v}, -M_{\text{max}}, M_{\text{max}} \right)$)*

### Weight Updates
If $\kappa_v > \epsilon_{\kappa}$, the weights are multiplicatively updated using a learning rate ($\beta$ or $\zeta$) and the local payoff feedback:
$$ w_{t+1}(v, d) = w_t(v, d) \cdot \exp\left( \beta \cdot \text{payoff}(v, d) \right) $$
or expressed using binary exponential scaling:
$$ w_v^{(t+1)} = w_v^{(t)} \cdot 2^{\zeta \cdot \kappa_v} $$

Because CMCA strictly forbids floating-point arithmetic, the exponentials and logarithms are evaluated using branchless Q16.16 minimax approximation polynomials.

## 3. Stable Projections

Leaf allocations are projected and scaled by resource prices $\mu_x$ and operational costs $c_x$ using saturating reciprocals via Newton-Raphson refinement instead of conditional division:
$$ P_{\mu}(x) = \frac{\pi_{\text{combined}}(x) \cdot \exp(-\mu_x \cdot c_x)}{\sum_{y \in \text{leaves}} \pi_{\text{combined}}(y) \cdot \exp(-\mu_y \cdot c_y)} $$

## 4. Exploration Floors

To guarantee no branch of the cascade is fully starved, an exploration floor $\eta$ is unconditionally mixed into the probability distribution. This limits starvation and guarantees numerical stability without branching:
$$ \pi_{\text{res}}(x) = \eta \cdot \frac{1}{n_L} + (1 - \eta) \cdot P_{\mu}(x) $$
where $n_L$ is the number of leaf nodes (or active routes $K$).
