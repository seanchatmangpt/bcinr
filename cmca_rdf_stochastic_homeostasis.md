# CMCA-RDF Stochastic Homeostasis Envelope

CMCA converges to a stochastic homeostatic envelope, not generally to a point.
The deployed system $x_{t+1}=F_a(x_t,\xi_t)$ involves stochastic receipt noise and importance weighting.

## 1. Stochastic Tracking Bound
The correct tracking bound is:
$$ \mathbb E |x_t-x_a^*|_W^2 \leq (1-\delta)^t |x_0-x_a^*|_W^2 + R_{\mathrm{noise}}^2 $$

The global floor bounds the variance: $\eta_g\downarrow \implies$ higher estimator variance $\implies$ larger envelope.

## 2. Learner Distinguishability and Freezing
MWU blocks require crowding curvature, a persistent winner, or removal by gating.
If distinguishability is near zero ($\lambda_{\min}^{+}(\Gamma) \approx 0$ or $\kappa_q(v) \approx 0$), the learner must be frozen.
**Gating Law:** $\operatorname{LearnerActive}(v) \iff \operatorname{Distinguishability}(v) > \epsilon_{\mathrm{dist}}.$

## 3. Dynamical Temperature Ceiling
Locally increasing returns (positive feedback) create a temperature ceiling.
**Admitted Limit:** $q_{\max}^{\mathrm{admitted}} = \min(q_c, q_{\mathrm{dyn,max}}, q_{\mathrm{numeric,max}}).$

## 4. Bounded Semantic Mode Changes
Semantic mode changes require both dwell-time and bounded fixed-point displacement.
The total homeostatic radius is:
$R_{\mathrm{homeostasis}} = R_{\mathrm{noise}} + R_{\mathrm{switch}}.$

## 5. Expanded Certificate Profile
The generated `stability_profile.rs` must include the noise and switching radius bounds.

```rust
pub struct StabilityProfile {
    pub gain_matrix: [[Fixed; 5]; 5],
    pub weight_vector: [Fixed; 5],
    pub deterministic_margin: Fixed,

    pub noise_second_moment_bounds: [Fixed; 5],
    pub certified_noise_radius: Fixed,

    pub mode_jump_bound: Fixed,
    pub minimum_dwell_rounds: u32,
    pub certified_switching_radius: Fixed,

    pub total_homeostatic_radius: Fixed,

    pub temperature_ceiling: Fixed,
    pub distinguishability_floor: Fixed,
    pub floor_minimum: Fixed,

    pub certificate_digest: Digest,
}
```

The principal claim is: "The state remains within a certified radius of the current admitted equilibrium."
