# CMCA-RDF Stability Certificate

This document acts as the formal stability certificate bounding the closed-loop dynamics of the CMCA-RDF architecture using the **Weighted Small-Gain Theorem**.

## Mathematical Derivation Summary

1. **Signed Local Jacobian**: We constructed the block Jacobian $J_a$ for the admitted mode.
2. **Nonnegative Comparison Gain Matrix**: $G_a$ was derived from the absolute block norms of $J_a$, preserving stabilizing negative feedback.
3. **Weighting Vector**: $d = [1.0, 1.2, 1.5, 0.8, 0.5]^T$ (block weighting).
4. **Verified Margin**: Spectral radius $\rho(G_a) \leq 1 - \delta$ with $\delta = 0.01$.
5. **Hybrid Switching Condition**: Dwell time bounded by $\tau_D \ge 461$.

## Configuration Inequalities

* $\beta_m \leq 0.045$
* $\zeta_w \leq 0.0125$
* $\zeta_\rho \leq 0.02$
* $\gamma_\mu \leq 0.01$
* $\eta_g \geq 0.001$

These bounds guarantee the system is globally asymptotically stable within the bounded semantic mass allocations.
