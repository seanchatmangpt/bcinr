# CMCA-RDF \kappa_q Estimator Measurement Contract

The Observatory must formally bound its vision before proposing learning mutations. The estimator must bound statistical uncertainty, fixed-point numerical error, support boundaries, and temporal drift.

## 1. Bounded Activation Interval
The runtime observes $\widehat\kappa_{q,t}(v)$. The activation interval must explicitly separate and subtract numeric and statistical errors:
[
\underline\kappa = \max \left( 0, \widehat\kappa - \varepsilon_{\mathrm{stat}} - \varepsilon_{\mathrm{numeric}} \right)
]
[
\overline\kappa = \widehat\kappa + \varepsilon_{\mathrm{stat}} + \varepsilon_{\mathrm{numeric}}
]

## 2. Strict Conjunction for Proposals
Only this conjunction may produce `PROPOSE_MODE_DELTA`:
[
\underline\kappa \geq \epsilon_{\mathrm{on}} \land \underline\gamma_{\min}^{+} \geq \epsilon_{\mathrm{gram}} \land n_{\mathrm{eff}} \geq n_{\min} \land D_t \leq \epsilon_{\mathrm{drift}}.
]

## 3. Effective Sample Size & Drift
Receipt sequences are not IID. $n_{\mathrm{eff}}$ must be calculated using an admitted dependence model. 
Drift between windows ($D_t$) must remain below $\epsilon_{\mathrm{drift}}$. If exceeded, the state becomes `CMCA_KAPPA_WINDOW_NONSTATIONARY`.

## 4. Measurement Standing
Each cell is assigned a rigid status based on the measurement artifact:
```text
INSUFFICIENT_DATA
DEPENDENCE_UNCERTIFIED
SUPPORT_INVALID
NUMERICALLY_UNCERTAIN
DRIFTING
SCALE_INERT
SCALE_INFORMATIVE
GRAM_DEGENERATE
RECERTIFICATION_CANDIDATE
```

## 5. Gram Distinguishability Lower Bound
Learner activation requires the conservative lower bound on the smallest positive eigenvalue of the Gram matrix:
[
\underline\gamma_{\min}^{+} = \widehat\gamma_{\min}^{+} - \varepsilon_\Gamma \geq \epsilon_{\mathrm{gram}}
]

## 6. Support Policy
KL divergence requires explicit support rules. Reject if $\operatorname{supp}(s^{\mathrm{leaf}}) \nsubseteq \operatorname{supp}(s^{\mathrm{meas}})$, or use an explicitly named, admitted, digest-bound prior $\alpha$ for smoothing. Do not insert silent epsilons.

## 7. Visual Encodings
The UI must map:
- **Intensity**: $\log(1+\widehat\kappa_q(v))$
- **Opacity**: Confidence / $n_{\mathrm{eff}}$
- **Border**: Learner state
- **Hatch**: Drift or Support Failure
- **Secondary Glyph**: $\underline\gamma_{\min}^{+}$
- **Lens Plane**: Render separate maps for sparse ($q<0$), coverage ($q=0$), and concentration ($q>1$). Do not average $q$ planes.
