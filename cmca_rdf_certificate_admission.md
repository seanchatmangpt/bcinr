# CMCA-RDF Certificate Admission and Envelope Monitoring

CMCA is becoming a **certificate-carrying adaptive control component**. A deployable configuration must contain an admitted mode $a$, declared operating envelope $\mathcal{E}_a$, signed Jacobian $J_a$, nonnegative comparison matrix $G_a$, weighting vector $d_a > 0$, contraction margin $\delta_a > 0$, reset bounds $\mathcal{R}_a$, dwell time $\tau_{D,a}$, and a digest binding the certificate to the generated kernel $H_a$.

## 1. Selection vs. Learning Permissions
The kernel requires two separate permissions:
- **Selection Permission**: Deterministically calculates $\pi_t = \mu_{\mathrm{CMCA}}(O_t^*)$ using admitted state, verified digests, and numeric inputs.
- **Learning Permission**: The system may update continuous adaptive states ($m, w, \rho, \mu, s$) ONLY when:
  $\operatorname{CertificateCurrent} \land \operatorname{EnvelopeValid} \land \operatorname{ModeDwellValid}$.

**Crucial Fallback**: Loss of the stability certificate does not crash the system. It transitions to `CertifiedSelectionOnly`. The global fairness floor remains active. Receipts accumulate for analysis, but learning (adaptive state mutation) freezes until the slow rail recertifies.

## 2. Runtime State Machine
Control state transitions:
```
CertifiedLearning
    ├─ envelope violation ─→ CertifiedSelectionOnly
    ├─ control-state change ─→ ModeTransitionHold
    ├─ certificate mismatch ─→ CertificateStale
    └─ valid operation ─→ CertifiedLearning
```

## 3. New Typed Refusals for Runtime Invalidation
```text
CMCA_RUNTIME_ENVELOPE_VIOLATED
CMCA_CERTIFICATE_DIGEST_MISMATCH
CMCA_CONTROL_MODE_UNCERTIFIED
CMCA_CONTROL_MODE_SWITCH_TOO_FAST
CMCA_YIELD_GAIN_BOUND_VIOLATED
CMCA_REWARD_BOUND_VIOLATED
CMCA_RESOURCE_RESPONSE_BOUND_VIOLATED
CMCA_STANDING_RESET_BOUND_VIOLATED
CMCA_LEARNING_FROZEN
```
`CMCA_LEARNING_FROZEN` is an outcome indicating selection continues under the last certified state, but mutation is disabled.

## 4. Total Digest Binding
The digest $H_a$ must cover $G_{\mathrm{RDF}}$, generated tables, kernel implementations, bounds, matrices, margin, and switching laws. Any upstream change invalidates the certificate.

## 5. Branchless Kernel Verification Math
The slow rail constructs the certificate. The branchless fast-rail kernel merely checks it.
The generated rust profile `generated/stability_profile.rs` must now expose:
```rust
pub const GAIN_MATRIX: [[Fixed; 5]; 5] = /* derived */;
pub const WEIGHT_VECTOR: [Fixed; 5] = /* derived */;
pub const CONTRACTION_MARGIN: Fixed = /* derived */;
pub const ENVELOPE: StabilityEnvelope = /* derived */;
pub const CERTIFICATE_DIGEST: Digest = /* derived */;
```
The kernel verifies $Gd \le (1-\delta)d$ using fixed multiply-accumulate operations.

## 6. Product Standing Hierarchy
- `CMCA_SELECT_ALIVE`: Deterministic allocator executes correctly.
- `CMCA_LEARNING_CERTIFIED_LOCAL`: Adaptive update is contractive inside fixed mode.
- `CMCA_LEARNING_CERTIFIED_SWITCHED`: Hybrid system satisfies reset and dwell-time conditions.
- `CMCA_LEARNING_FROZEN`: Selection available, adaptive mutation disabled due to envelope breach.
- `CMCA_HOMEOSTASIS_UNKNOWN`: Executes without an accepted stability certificate (UNSAFE).
