# Innovation Proposal: Branchless Learning Rate Controller (LRC) for Dynamic Step-Size Adaptation

## 1. Executive Summary

This proposal introduces the **Branchless Learning Rate Controller (LRC)**, a deterministic, constant-time feedback control system designed to dynamically adapt the Multiplicative Weights Update (MWU) learning rate $\zeta$ and the uniform co-allocation exploration rate $\eta$ in the `bcinr-cmca` resource allocator.

The primary objective is to replace static, manually tuned learning rates with an autonomic feedback controller that scales learning parameters based on real-time **tracking error variance** and **stability margin components**. To comply with the strict **BCINR Radon Law**, the LRC is designed with:
1. **Strict Radon Compliance**: A cyclomatic complexity of $CC=1$, zero heap allocations, and zero data-dependent control branches. All conditional updates are executed using bitwise masking and conditional selection.
2. **Smooth Variance Filtering**: An Exponential Moving Average (EMA) variance filter that tracks tracking error volatility without storing historical data arrays or triggering dynamic heap allocations.
3. **Stability-Preserving Backoff**: A branchless contraction margin estimator that forces learning rate decay when the system exhibits high Kullback-Leibler divergence ($\kappa_t$) or Jensen-Shannon drift ($d_{\mathrm{JS}}$).
4. **Anti-Collapse Uniform Boosting**: A mechanism that dynamically increases uniform mixing ($\eta$) under high tracking error variance, preventing premature allocation collapse.

---

## 2. Vulnerability & Limitation Analysis

### 2.1 The Rigidity of Static Learning Rates
In the current CMCA allocator ([allocator.rs](file:///Users/sac/bcinr/crates/bcinr-cmca/src/allocator.rs)), the learning rate parameters $\zeta$ (step size for routing weight updates) and $\eta$ (exploration rate for mixing uniform distributions) are passed as static inputs from the orchestrator:
```rust
pub fn allocate(
    ...
    eta: Fixed,
    ...
    zeta: Fixed,
    ...
) -> Result<[Fixed; N], StabilityRefusal>
```
Static configurations present a fundamental control trade-off:
- **High Learning Rate ($\zeta$)**: Accelerates convergence to optimal co-allocation profiles under stationary query distributions, but triggers destabilizing oscillations, limit cycles, or boundary collapse when query patterns drift or exhibit high variance.
- **Low Learning Rate ($\zeta$)**: Ensures convergence stability under noisy conditions, but makes the autonomic loop sluggish, leaving the routing configuration unable to adapt to live query demands in time.

### 2.2 Branching Penalties and Side Channels
Conventional adaptive controllers (such as PID loops or gain schedulers) rely on conditional branches (`if error > threshold { decrease_rate() }`) to clamp parameters. In a high-integrity computational substrate:
1. **Radon Violations**: Conditional statements violate the $CC=1$ rule of the BCINR substrate, blocking the compilation pipeline.
2. **Timing Side-Channels**: Data-dependent branches create execution timing differentials that leak information about resource consumption, exposing the substrate to side-channel analysis.
3. **Division Panics**: Naive dynamic scaling equations (`rate = alpha / variance`) risk divide-by-zero panics when the system reaches a steady state (variance $\to 0$).

---

## 3. Proposed Innovation: Branchless Learning Rate Controller (LRC)

We propose a stateful, branchless controller that evaluates tracking error variance and stability margin metrics to dynamically scale $\zeta$ and $\eta$ at each allocation step.

### 3.1 State Tracking and Variance Filtering
Let $e_t \ge 0$ be the scalar tracking error at round $t$ (e.g., the Euclidean distance between actual co-allocation flows and target demands). The controller tracks the mean error $\bar{e}_t$ and error variance $\sigma^2_t$ using coupled Exponential Moving Average (EMA) filters:

$$\bar{e}_t = (1 - \alpha) \bar{e}_{t-1} + \alpha e_t$$

$$\sigma^2_t = (1 - \alpha) \sigma^2_{t-1} + \alpha |e_t - \bar{e}_t|^2$$

where $\alpha \in [0, 1]$ is the fixed-point smoothing factor. The absolute difference $|e_t - \bar{e}_t|$ is computed branchlessly.

### 3.2 Stability Margin Estimator
The stability margin $\Phi_t \in [\Phi_{\min}, \Phi_{\max}]$ represents the system's proximity to the contraction boundary (where the spectral radius of the state Jacobian $\rho(J) \ge 1$). We estimate this margin branchlessly using the current Kullback-Leibler subtree divergence $\kappa_t$ and the Jensen-Shannon drift $d_{\mathrm{JS},t}$:

$$\Phi_t = \text{clip}(\Phi_{\max} - k_{\kappa} \kappa_t - k_d d_{\mathrm{JS},t}, \, \Phi_{\min}, \, \Phi_{\max})$$

where:
- $\Phi_{\max}$ is the nominal stability margin ($1.0$).
- $k_{\kappa}$ is the sensitivity coefficient to subtree divergence.
- $k_d$ is the sensitivity coefficient to distributional drift.

### 3.3 Dynamic Control Laws
1. **MWU Step-Size ($\zeta_t$)**: The step-size decays proportionally to the tracking error variance (filtering noise) and scales with the stability margin:
   $$\zeta_t = \text{clip}\left( \frac{\zeta_0 \cdot \Phi_t}{1 + \gamma \sigma^2_t}, \, \zeta_{\min}, \, \zeta_{\max} \right)$$
   where $\zeta_0$ is the baseline step size, and $\gamma$ governs the variance penalty. The denominator is strictly $\ge 1.0$, guaranteeing safety from division-by-zero.
   
2. **Uniform Exploration Rate ($\eta_t$)**: Under highly volatile conditions (large $\sigma^2_t$), the controller increases the uniform allocation floor to smooth routing profiles and prevent premature node starvation:
   $$\eta_t = \text{clip}(\eta_0 + \theta \sigma^2_t, \, \eta_{\min}, \, \eta_{\max})$$
   where $\theta$ governs the exploration scale boost.

---

## 4. Mathematical and Logical Contract

The mathematical contract for the LRC is defined as:

$$\{P(S, e, \kappa, d_{\mathrm{JS}}, \text{params})\} \quad \text{update\_learning\_rates}(S, e, \kappa, d_{\mathrm{JS}}, \text{params}) \quad \{Q(S, S', \zeta, \eta)\}$$

### 4.1 Preconditions $P$
- **State Validity**: $S$ must contain valid, non-NaN Q16.16 values: $S.\bar{e} \ge 0, S.\sigma^2 \ge 0$.
- **Error Domain**: $e.0 \in [0, 2^{31}-1]$.
- **Divergence and Drift Domains**: $\kappa.0 \ge 0$, $d_{\mathrm{JS}}.0 \ge 0$.
- **Parameter Bounds**: All configuration coefficients must satisfy:
  - $\alpha.0 \in [0, 65536]$ (representing $[0.0, 1.0]$).
  - $\Phi_{\min}.0 \le \Phi_{\max}.0 \le 65536$.
  - $\zeta_{\min}.0 \le \zeta_{\max}.0$.
  - $\eta_{\min}.0 \le \eta_{\max}.0$.

### 4.2 Postconditions $Q$
- **Output Bounds**: The adjusted learning rates must reside strictly within their safety envelopes:
  - $\zeta \in [\zeta_{\min}, \zeta_{\max}]$
  - $\eta \in [\eta_{\min}, \eta_{\max}]$
- **Monotonicity under Noise**: As variance increases for a fixed stability margin, $\zeta$ decreases monotonically and $\eta$ increases monotonically:
  $$\sigma^2_{A} > \sigma^2_{B} \land \Phi_A = \Phi_B \implies \zeta_A \le \zeta_B \land \eta_A \ge \eta_B$$
- **Monotonicity under Instability**: As the stability margin shrinks for a fixed variance, $\zeta$ decreases:
  $$\Phi_{A} < \Phi_{B} \implies \zeta_A \le \zeta_B$$
- **Radon Complexity Invariant**: The function must achieve $CC=1$ and execute in constant instruction cycles.

---

## 5. Implementation Architecture & Integration Plan

### 5.1 The Learning Rate Controller Module
We define the LRC structures and logic in a new struct. The code uses `Fixed` fixed-point arithmetic and branchless helpers:

```rust
use crate::fixed::Fixed;
use crate::allocator::{clip, const_lt_u32, const_select_u32};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct LrcState {
    pub mean_error: Fixed,
    pub var_error: Fixed,
    pub prev_zeta: Fixed,
    pub prev_eta: Fixed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LrcParams {
    pub alpha: Fixed,       // Smoothing factor for EMA (e.g., 0.125)
    pub phi_max: Fixed,     // Maximum stability margin (e.g., 1.0)
    pub phi_min: Fixed,     // Minimum stability margin (e.g., 0.1)
    pub zeta_0: Fixed,      // Baseline MWU step size (e.g., 0.005)
    pub zeta_min: Fixed,    // Minimum MWU step size (e.g., 0.0005)
    pub zeta_max: Fixed,    // Maximum MWU step size (e.g., 0.0125)
    pub eta_0: Fixed,       // Baseline exploration rate (e.g., 0.01)
    pub eta_min: Fixed,     // Minimum exploration rate (e.g., 0.005)
    pub eta_max: Fixed,     // Maximum exploration rate (e.g., 0.1)
    pub k_kappa: Fixed,     // Sensitivity to subtree divergence (e.g., 0.5)
    pub k_d: Fixed,         // Sensitivity to drift (e.g., 0.5)
    pub gamma: Fixed,       // Variance penalty scale for zeta (e.g., 1.0)
    pub theta: Fixed,       // Variance boost scale for eta (e.g., 2.0)
}

impl LrcState {
    /// Update the learning rates branchlessly under Radon Law (CC=1).
    #[inline(always)]
    pub fn update(
        &mut self,
        current_error: Fixed,
        kappa: Fixed,
        d_js: Fixed,
        params: &LrcParams,
    ) -> (Fixed, Fixed) {
        // 1. Mean Error Update: mean = (1 - alpha) * mean + alpha * error
        let alpha = params.alpha;
        let one_minus_alpha = Fixed::ONE - alpha;
        let mean_next = (one_minus_alpha * self.mean_error) + (alpha * current_error);

        // 2. Absolute Difference: diff = |current_error - mean_next|
        let is_lt = const_lt_u32(current_error.0, mean_next.0);
        let diff = Fixed(const_select_u32(
            is_lt,
            mean_next.0.wrapping_sub(current_error.0),
            current_error.0.wrapping_sub(mean_next.0),
        ));

        // 3. Variance Error Update: var = (1 - alpha) * var + alpha * diff^2
        let diff_sq = diff * diff;
        let var_next = (one_minus_alpha * self.var_error) + (alpha * diff_sq);

        // 4. Stability Margin Component: Phi = max(Phi_min, Phi_max - k_kappa * kappa - k_d * d_js)
        let penalty = (params.k_kappa * kappa) + (params.k_d * d_js);
        let phi = params.phi_max.saturating_sub(penalty);
        let phi_safe = clip(phi, params.phi_min, params.phi_max);

        // 5. MWU Step-Size: zeta = num / denom
        let num = params.zeta_0 * phi_safe;
        let denom = Fixed::ONE + (params.gamma * var_next);
        let zeta_next = num.saturating_div(denom);
        let zeta_clipped = clip(zeta_next, params.zeta_min, params.zeta_max);

        // 6. Uniform Exploration: eta = eta_0 + theta * var
        let eta_next = params.eta_0 + (params.theta * var_next);
        let eta_clipped = clip(eta_next, params.eta_min, params.eta_max);

        // Commit state
        self.mean_error = mean_next;
        self.var_error = var_next;
        self.prev_zeta = zeta_clipped;
        self.prev_eta = eta_clipped;

        (zeta_clipped, eta_clipped)
    }
}
```

### 5.2 Integration with `allocate`
The LRC struct can be integrated into the allocation pipeline inside `allocate`. In each allocation round, the controller intercepts the external `eta` and `zeta` values, dynamically updates them using the live observatory telemetry, and feeds the adapted values downstream to the MWU update and allocation combination blocks.

---

## 6. Verification Strategy

### 6.1 Double-Precision Reference Oracle
An independent, floating-point reference oracle is implemented in the test suite to verify the fixed-point numerical envelope:

```rust
pub struct OracleLrcState {
    pub mean_error: f64,
    pub var_error: f64,
}

pub fn oracle_update_learning_rates(
    state: &mut OracleLrcState,
    current_error: f64,
    kappa: f64,
    d_js: f64,
) -> (f64, f64) {
    let alpha = 0.125;
    let phi_max = 1.0;
    let phi_min = 0.1;
    let zeta_0 = 0.005;
    let zeta_min = 0.0005;
    let zeta_max = 0.0125;
    let eta_0 = 0.01;
    let eta_min = 0.005;
    let eta_max = 0.1;
    let k_kappa = 0.5;
    let k_d = 0.5;
    let gamma = 1.0;
    let theta = 2.0;

    state.mean_error = (1.0 - alpha) * state.mean_error + alpha * current_error;
    let diff = (current_error - state.mean_error).abs();
    state.var_error = (1.0 - alpha) * state.var_error + alpha * diff.powi(2);

    let penalty = k_kappa * kappa + k_d * d_js;
    let phi = (phi_max - penalty).clamp(phi_min, phi_max);

    let zeta = (zeta_0 * phi) / (1.0 + gamma * state.var_error);
    let zeta_clipped = zeta.clamp(zeta_min, zeta_max);

    let eta = eta_0 + theta * state.var_error;
    let eta_clipped = eta.clamp(eta_min, eta_max);

    (zeta_clipped, eta_clipped)
}
```

Differential property-based testing runs 1,000,000 randomized trajectories comparing the outputs of the fixed-point `LrcState::update` against `oracle_update_learning_rates`. Mismatch thresholds must not exceed an absolute envelope of $10^{-4}$ (equivalent to $\pm 6$ LSB steps in Q16.16).

### 6.2 Hostile Mutant Scenarios
Under `@armstrong_fault` rules, three independent mutants challenge the LRC verification suite:

1. **Mutant 1 (State Leak / Speculative Mutation)**:
   Modify the controller to update `self.mean_error` before executing the absolute difference step:
   ```rust
   self.mean_error = mean_next;
   // diff is calculated after mean has already been updated
   let diff = Fixed(const_select_u32(is_lt, mean_next.0 - current_error.0, ...));
   ```
   *Expectation*: This mutant computes $\text{diff} = 0$, causing the error variance estimation to decay to zero. The differential verification suite will detect the deviation in variance and learning rate updates immediately.
2. **Mutant 2 (Div-by-Zero Injection)**:
   Alter the step-size equation to omit the base value of $1.0$ in the denominator:
   ```rust
   let denom = params.gamma * var_next; // Omitted Fixed::ONE
   ```
   *Expectation*: When error variance is zero, this triggers division by zero, saturating $\zeta$ to `u32::MAX` (`Fixed::MAX`). The contract assertion for $\zeta \le \zeta_{\max}$ will fail, outputting a `StabilityRefusal::LearningRateOutsideEnvelope` refusal.
3. **Mutant 3 (Exploration Sign Inversion)**:
   Invert the variance contribution to the exploration rate:
   ```rust
   let eta_next = params.eta_0.saturating_sub(params.theta * var_next);
   ```
   *Expectation*: Highly volatile conditions will depress exploration rather than boost it, causing allocations to converge prematurely under noise and triggering a differential mismatch.

### 6.3 Disassembly Audit Plan
The release-profile assembly code is audited to verify:
1. **No Loop Backedges**: The compiler must generate straight-line assembly containing no jump-back directives.
2. **Conditional Move Usage**: The conditional check for the absolute error difference must compile to `sub`, `neg`, and `cmov` instructions rather than conditional jump instructions (`je`/`jne`).
3. **Zero Heap Allocation**: The symbol registry must contain no links to memory allocation symbols.

---

## 7. Downstream Impact

1. **Autonomic Adaptation**: The learning loop dynamically stabilizes itself during high-noise phases, eliminating manual parameter tuning.
2. **Mitigation of Divergence**: Under drifting workloads, the step-size is automatically curtailed before routing configurations diverge past safety limits.
3. **Maturity Standing**: Maintains a Substrate Integrity Score (SIS) of 100/100 by executing in strict constant-time and satisfying its mathematical contracts.
