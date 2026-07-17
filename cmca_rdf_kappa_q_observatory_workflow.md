# CMCA \kappa_q Observatory Admission and Recertification Protocol

The \kappa_q Observatory is permitted to **measure and propose**. It must never activate, deactivate, or retune a learner directly.

## 1. Governing Law
The divergence scale is calculated as:
[
\kappa_q(v) = \sum_{c\in C(v)} s_q^{\mathrm{leaf}}(c\mid v) \log \frac{s_q^{\mathrm{leaf}}(c\mid v)}{s_q^{\mathrm{meas}}(c\mid v)}
]
Under `AggregatedQNorm`, $\kappa_q(v)=0$ by construction and learners remain inactive. Adaptive control requires `Measured` mass convention.

## 2. Authority Boundary
The Observatory may only `OBSERVE`, `ESTIMATE`, `VISUALIZE`, and `PROPOSE_MODE_DELTA`.
It cannot `ACTIVATE_LEARNER`, `UPDATE_WEIGHT`, or `CHANGE_CONTROL_MODE`.

The lawful chain is:
$\text{telemetry} \rightarrow \kappa_q \text{ measurement} \rightarrow \text{admitted proposal} \rightarrow \text{recertification} \rightarrow \text{new control mode}.$

## 3. Joint Distinguishability Gate
A learner may only be proposed when $\underline\kappa_q(v) \ge \epsilon_{\mathrm{on}}$ AND the Gram matrix $\gamma_{\min}^+(v) \ge \epsilon_{\mathrm{gram}}$.
$\operatorname{LearnerInformative}(v) \iff \kappa \text{ is material} \land \Gamma_v \text{ has adequate rank and separation}.$

## 4. Recertification Workflow (9 Stages)
1. **Measurement admission**: Validate artifacts, bounds, and provenance to admit the measurement.
2. **Shadow construction**: Construct the proposed mode without affecting production.
3. **Jump analysis**: Bound the deterministic jump $\Delta_\pi$ and the fixed-point jump $\Delta_* \le \Delta_{\max}$.
4. **Stability regeneration**: Regenerate $J_{a'}, G_{a'}, d_{a'}, \delta_{a'}$ and verify $G_{a'}d_{a'} \le (1-\delta_{a'})d_{a'}$.
5. **Influence digest**: Bind the new profile into $H_{a'}$.
6. **Certificate admission**: Produce the admitted certificate.
7. **Transition hold**: Enter `ModeTransitionHold` for the required dwell time.
8. **Atomic mode switch**: Switch when the transition receipt is admitted.
9. **Certified learning**: Only after switch admission does the new learner mutate.

## 5. Output-Preserving Reset Maps
- **Deactivation**: Replace adaptive local mixture with a frozen policy. Jump $\Delta\pi_v = 0$.
- **Activation**: Initialize expert weights via KL projection onto the convex hull of new experts.

## 6. Typed Refusals
```text
CMCA_KAPPA_ARTIFACT_STALE
CMCA_KAPPA_CONTROL_DIGEST_MISMATCH
CMCA_KAPPA_MASS_CONVENTION_UNDECLARED
CMCA_KAPPA_ZERO_MASS_UNADMITTED
CMCA_KAPPA_SUPPORT_MISMATCH
CMCA_KAPPA_NUMERIC_ERROR_EXCEEDED
CMCA_KAPPA_ESTIMATE_UNCERTAIN
CMCA_KAPPA_HYSTERESIS_NOT_CLEARED
CMCA_KAPPA_GRAM_RANK_INSUFFICIENT
CMCA_KAPPA_LEARNER_INDISTINGUISHABLE
CMCA_KAPPA_MODE_DELTA_UNADMITTED
CMCA_KAPPA_RESET_MAP_INVALID
CMCA_KAPPA_POLICY_JUMP_EXCEEDED
CMCA_KAPPA_FIXED_POINT_JUMP_EXCEEDED
CMCA_KAPPA_RECERTIFICATION_REQUIRED
CMCA_KAPPA_STABILITY_CERTIFICATE_FAILED
CMCA_KAPPA_MODE_DWELL_VIOLATED
CMCA_KAPPA_TRANSITION_RECEIPT_MISSING
CMCA_KAPPA_LEARNER_ACTIVATION_REFUSED
CMCA_KAPPA_LEARNER_DEACTIVATION_REFUSED
```

## 7. Visual Contract
The visual must distinguish: `MEASURED`, `PROPOSED`, `ADMITTED`, `CERTIFIED`, `ACTIVE`, `FROZEN`.
It must show $\log(1+\kappa_q)$ as intensity, uncertainty as opacity, and distinguish active (solid) vs gated (crossed) learners.
