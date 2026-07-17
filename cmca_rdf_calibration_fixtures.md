# CMCA-RDF \kappa_q Observatory Synthetic Calibration and Hostile Fixture Specification

The synthetic suite functions as a **sensor calibration laboratory**. It verifies that the Observatory accurately estimates population values, conservatively reports uncertainty, refuses unsupported confidence, and detects scale collapse, policy degeneracy, and telemetry dependence.

## 1. Core Calibration Fixtures

Implement these fixtures immediately to test numeric blindness, policy indistinguishability, and temporal invalidity:
* `F02_NUMERICALLY_UNCERTAIN`: True $\kappa$ lies near the activation threshold. The point estimate $\widehat\kappa > \epsilon_{\mathrm{on}}$, but the conservative bound $\underline\kappa < \epsilon_{\mathrm{on}}$. The sensor MUST refuse activation and flag `NUMERICALLY_UNCERTAIN`.
* `F03_GRAM_DEGENERATE`: Construct two identical expert policies. $\kappa$ is material, but the Gram matrix is rank-deficient ($\lambda_{\min}^{+}(\Gamma) \approx 0$). MUST flag `GRAM_DEGENERATE` and refuse activation.
* `F09_NONSTATIONARY_WINDOW`: Generate two consecutive windows where $D_{\mathrm{JS}}(s_{\mathrm{old}}, s_{\mathrm{new}}) > \epsilon_{\mathrm{drift}}$. MUST flag `DRIFTING` and refuse recertification proposals.

Followed by the positive and negative calibration anchors:
* `F00_EXACT_SCALE_COLLAPSE`: $s^{\mathrm{meas}} = s^{\mathrm{leaf}}$. $\kappa_q = 0$. MUST flag `SCALE_INERT`.
* `F01_MATERIAL_SCALE_INFORMATION`: $\underline\kappa \ge \epsilon_{\mathrm{on}}$ and $\underline\gamma_{\min}^{+} \ge \epsilon_{\mathrm{gram}}$. MUST flag `RECERTIFICATION_CANDIDATE`.

## 2. Hostile Mutants to Inject and Kill
* `M01`: Ignore numeric error in $\underline\kappa$. (Killed by `F02`).
* `M03`: Use point-estimate Gram gate ($\widehat\gamma_{\min}^{+}$) without subtracting $\varepsilon_\Gamma$. (Killed by `F04`).
* `M05`: Ignore drift (permit proposal while $D_t > \epsilon_{\mathrm{drift}}$). (Killed by `F09`).
* `M07`: Activate learner based on $\kappa$ only, ignoring Gram distinguishability. (Killed by `F03`).

## 3. Required Output Reports
The test suite must generate the following reports into `docs/cmca-rdf/observatory/`:
- `CALIBRATION_REPORT.md`
- `MUTANT_KILL_MATRIX.md`
- `NUMERIC_ERROR_REPORT.md`
- `DEPENDENCE_REPORT.md`
- `Q_PLANE_REPORT.md`
