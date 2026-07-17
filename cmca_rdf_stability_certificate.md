# CMCA-RDF Stability Certificate Requirements

The qualitative timescale ordering ($\beta_m > \zeta_w > \gamma_\mu$) is a hypothesis, not the stability theorem. Stability depends on loop gains, not the bare ordering of learning rates.

The implementation team must receive **derived configuration inequalities**.

## 1. The Signed Local Jacobian and Nonnegative Gain Matrix

Let the continuous state be:
$x= [m, w, \rho, \mu, s]^T$
Let the allocation map be:
$\pi = \mathcal A(m,w,\rho,\mu,s).$

Define explicit Lipschitz or Jacobian bounds:
$A_m = \left|\frac{\partial\mathcal A}{\partial m}\right|, \quad A_w = \left|\frac{\partial\mathcal A}{\partial w}\right|, \quad A_\rho = \left|\frac{\partial\mathcal A}{\partial\rho}\right|, \quad A_\mu = \left|\frac{\partial\mathcal A}{\partial\mu}\right|, \quad A_s = \left|\frac{\partial\mathcal A}{\partial s}\right|.$

Bound the environment/receipt responses:
$Y_\pi = \left|\frac{\partial \widehat y}{\partial\pi}\right|, \quad R_\pi = \left|\frac{\partial \widehat R}{\partial\pi}\right|, \quad H_\pi = \left|\frac{\partial \widehat H_{\mathrm{resolution}}}{\partial\pi}\right|, \quad S_\pi = \left|\frac{\partial s^+}{\partial\pi}\right|.$

The price loop's local gain is bounded by:
$\left|\frac{\partial\mu^+}{\partial\pi}\right| \leq \gamma_\mu|C|.$

### The Gain Matrix
A defensible nonnegative comparison matrix has the form:
$G = [g_{ij}]_{5\times5}$

For example, the mass row:
$g_{mm} \leq 1-\beta_m + \beta_mY_\pi A_m + \beta_mY_m$
$g_{mw} \leq \beta_mY_\pi A_w$

The price row must preserve signs where necessary to prevent a crude norm bound from erasing stabilizing negative feedback:
$g_{\mu\mu} \leq 1+ \gamma_\mu|C|A_\mu$

## 2. The Weighted Small-Gain Theorem

Find positive weights $d = (d_m,d_w,d_\rho,d_\mu,d_s)>0$ such that:
$Gd < d.$
Equivalently:
$\max_i \frac{(Gd)_i}{d_i} < 1.$

This yields contraction in the weighted block maximum norm. The implementation artifact should contain this generated configuration certificate.

## 3. Required Configuration Artifact

The system must manufacture a bounded certificate resembling:
`cmca-stability-profile.ttl`
`cmca-stability-profile.shacl.ttl`
`generated/stability_profile.rs`
`STABILITY_CERTIFICATE.md`

The generated Rust should expose constants derived from the admitted profile (not handwritten magic constants):
```rust
pub const BETA_M_MAX: Fixed = /* derived */;
pub const ZETA_W_MAX: Fixed = /* derived */;
pub const ZETA_RHO_MAX: Fixed = /* derived */;
pub const GAMMA_MU_MAX: Fixed = /* derived */;
pub const ETA_G_MIN: Fixed = /* derived */;
pub const CONTRACTION_MARGIN_MIN: Fixed = /* derived */;
pub const MODE_DWELL_ROUNDS_MIN: u32 = /* derived */;
```

## 4. The Configuration Gate Refusal Types

The kernel must reject any configuration for which the certificate cannot establish $Gd \leq (1-\delta)d$.
The refusal should be typed:
```text
CMCA_STABILITY_CERTIFICATE_MISSING
CMCA_BLOCK_GAIN_BOUND_EXCEEDED
CMCA_CONTRACTION_MARGIN_INSUFFICIENT
CMCA_LEARNING_RATE_OUTSIDE_ENVELOPE
CMCA_MODE_DWELL_TIME_VIOLATED
CMCA_Q_RANGE_DESTABILIZING
CMCA_MASS_CLAMP_UNSAFE
CMCA_PRICE_GAIN_UNSAFE
CMCA_STANDING_PROJECTION_GAIN_UNSAFE
```
The runtime kernel merely checks or embeds the generated bounds.

## 5. Discrete Standing Reset Law

When $\sigma_t \neq \sigma_{t+1}$, the transition must define a bounded reset map:
$x^+ = \mathcal R_{\sigma\to\sigma'}(x^-).$

A sufficient average dwell-time relationship:
$\tau_D > \frac{\log\chi_{\max}}{-\log(1-\delta)}.$

## Final Directive

The mathematical derivation must contain:
1. **A signed local Jacobian**, preserving stabilizing negative feedback.
2. **A nonnegative comparison gain matrix**, used for conservative certification.
3. **A positive weighting vector** ($d$).
4. **A verified margin**: $Gd\leq(1-\delta)d$.
5. **A hybrid switching condition** for admitted RDF and standing-state changes.
