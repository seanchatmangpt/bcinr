# Refusal Enforcement Report: CertificateStale & EnvelopeViolated

In the `bcinr-cmca` crate, the enforcement of operational bounds operates strictly under the deterministic, branchless rules mandated by the substrate constitution. The refusals for `CertificateStale` and `EnvelopeViolated` are handled distinctly via a layered ownership model and bitwise algebra.

## 1. `CertificateStale` Enforcement

The condition where a previously valid certificate is no longer current is represented as `RefusalSet::CERTIFICATE_STALE` in `src/allocator.rs`. However, its actual enforcement is deliberately pushed to a different component to satisfy the "four-separate-authorities" structure (`OWNED_BY_DIFFERENT_COMPONENT`).

### Trigger & Realization
- It is never constructed within the hot path of `allocator::allocate()`.
- Instead, it is realized during mode transitions in `src/mode_switch.rs` (`apply_mode_switch`) via `ModeSwitchRefusal::CertificateDigestMismatch`. 
- The trigger happens when comparing the presented certificate against a freshly re-derived expectation: `certificate != expected_certificate`.
- A sub-case (sealed against a superseded round) is similarly refused upstream by `certification::CertificationRefusal::RoundIdentityMismatch`.

### Branchless Enforcement
In `apply_mode_switch`, the candidate state is calculated structurally and unconditionally. The state commit relies on boolean evaluation:
```rust
let cert_ok = certificate == expected_certificate;
let dwell_ok = dwell.round_identity() == round_identity && dwell.transition_identity() == transition_identity;
let state_ok = switch.admitted_state_digest == persistent.mode_digest;

let admitted = cert_ok && dwell_ok && state_ok;

// Masked commit law: compute candidate structurally, then select.
let candidate = ModeState {
    mode_digest: switch.target_mode_digest,
    generation: persistent.generation.wrapping_add(1),
};
let next = if admitted { candidate } else { *persistent };
*persistent = next;
```

## 2. `EnvelopeViolated` (Runtime & Learning Envelopes)

Historically, specific envelope violations were mapped to single-enum variants like `StabilityRefusal::RuntimeEnvelopeViolated` and `StabilityRefusal::LearningRateOutsideEnvelope`. In the current architecture, these have been folded into `RefusalSet` flags to guarantee a branchless `$O(1)$` accumulation.

### Trigger & Realization
In `src/allocator.rs`, the envelope bounds for parameters like `zeta` (learning rate), `eta`, `beta`, `q`, and `mu` (price) are evaluated unconditionally on every allocation attempt.

Instead of branching on out-of-envelope values, they are mapped to bitwise error flags:
```rust
let lr_err = const_lt_u32(zeta_w_max_q16, zeta.value_bits()) != 0;
let eta_err = const_lt_u32(eta.value_bits(), eta_g_min_q16) != 0;
let beta_err = const_lt_u32(beta_m_max_q16, beta.value_bits()) != 0;
let price_err = const_lt_u32(mu_max.value_bits(), mu[i & 7].value_bits()) != 0;
```

These errors are then collapsed into a single `PROPOSAL_REJECTED` refusal bit. As noted in `tests/case_studies.rs`, the legacy distinct `LearningRateOutsideEnvelope` variant is no longer directly reachable; it is merged into the collective bitwise envelope fault.

### Branchless Enforcement
The accumulated faults determine the `has_error` and `has_refusal` masks. The state commit is completely branchless and uses deterministic masked selection to either accept the updated values or silently preserve the old ones:

```rust
let has_error = !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;

// Gating the state write-back using `select_nnf`
unroll_8_static!(v, {
    unroll_8_static!(e, {
        weights[v & 7][e & 7] = select_nnf(
            has_refusal as u32,
            weights[v & 7][e & 7],
            local_weights[v & 7][e & 7],
        );
    });
});
```

Refusals are accumulated using a bitwise `union` layered with masks (`RefusalSet::PROPOSAL_REJECTED.masked((...) as u32)`), guaranteeing zero instruction-pointer variation regardless of how many envelope bounds are violated simultaneously.
