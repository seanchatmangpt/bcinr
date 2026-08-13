# CMCA-110: eta_err has no upper bound and numeric_has_err is proof-gated — CMCA-103's bug class left half-fixed

**Type:** Bug
**Priority:** Critical (silent output corruption, same class CMCA-103 exists to close, confirmed reachable on the common `proof=None` path)

## Summary

CMCA-103 fixed `q_err`/`price_err`/`eta_err` to refuse unconditionally
(regardless of `proof`), reasoning that anything feeding the selection
computation unconditionally must refuse unconditionally too. That reasoning
is correct but was applied incompletely:

1. **`eta_err` only checks a lower bound.** There is no `ETA_G_MAX` anywhere
   in the crate. `eta > 1.0` passes `eta_err`, is caught by no other check,
   and silently corrupts the explore-floor blend via `saturating_sub`
   underflow — discarding the priced allocation entirely in favor of pure
   uniform explore, with **no refusal returned**.
2. **`numeric_has_err`** (errors accumulated from `pi_res[x].err` across all
   8 nodes — i.e. errors from the very selection computation CMCA-103's own
   fix targets) is still folded into the **proof-gated** `has_error` bucket,
   not the unconditional `selection_critical_error` bucket. So a numeric
   fault produced by the unconditional selection code (including the
   underflow from finding 1) is silently swallowed on the `proof=None` path.

## Context

Found by adversarial review of the CMCA-103 fix, specifically by tracing every
read site of every flag folded into `has_error` versus `selection_critical_error`
in `crates/bcinr-cmca/src/allocator/mod.rs`'s `allocate_in`.

- `eta_err` (mod.rs:1515): `const_lt_u32(eta.val, eta_g_min_q16) != 0` — lower
  bound only. `crates/bcinr-cmca/src/generated/stability_profile.rs` has
  `ETA_G_MIN` and no `ETA_G_MAX` (confirmed by grep).
- Explore-floor blend (mod.rs:1812):
  `let val = (eta_actual * nl_recip) + ((NonNegativeFixed::ONE - eta_actual) * p_mu);`
  — `NonNegativeFixed::sub` is `saturating_sub` (fixed.rs:126-133); when
  `eta_actual > ONE`, `(ONE - eta_actual)` underflows, clamps to 0, and sets
  `err = NumericRangeExceeded` on that intermediate value.
- `numeric_err`/`numeric_has_err` (mod.rs:1825-1831) accumulates exactly that
  kind of per-node `.err`, but is folded into `has_error` (proof-gated), not
  `selection_critical_error` (mod.rs:1864, unconditional) — so this exact
  underflow's fault flag never reaches `has_refusal` when `proof=None`.
- Confirmed no existing test exercises `eta > 1.0` (grepped `case_studies.rs`,
  `runtime_semantic_classification.rs`, `hostile_mutants.rs`,
  `jtbd_certified_actuation_chicago.rs`).
- The negative-case (graceful-degrade) contract was independently verified
  intact:
  `jtbd_drift_refusal_routes_to_selection_only_without_state_drift`
  (`tests/jtbd_certified_actuation_chicago.rs:179-198`) still correctly
  passes for `digest_err=true, proof=None` — this ticket does not ask to
  change that contract, only to close the two gaps above.

## Acceptance Criteria

- [ ] Add an `ETA_G_MAX` bound (or equivalent upper-bound check) to
      `eta_err`'s construction, so `eta > 1.0` (or whatever the correct
      ceiling is — derive it from the pricing/explore-floor math, don't
      guess) is refused.
- [ ] Fold `numeric_has_err` into the unconditional `selection_critical_error`
      bucket (or a correctly-scoped unconditional check), not the
      proof-gated `has_error` bucket — since it originates in code that
      already executes unconditionally per CMCA-103's own stated rule.
- [ ] Add a regression test: `eta > 1.0` with `proof=None` must return
      `Err(...)`, not `Ok(...)` with a silently-corrupted uniform-explore
      result.
- [ ] Add a regression test proving `numeric_has_err` now surfaces as a
      refusal on `proof=None` for a case that triggers it (e.g. the
      eta-underflow case above, checked before and after the `eta_err` fix
      to confirm both layers are real, independent gates — not one fix
      accidentally making the other untestable).
- [ ] Re-verify `jtbd_drift_refusal_routes_to_selection_only_without_state_drift`
      and `case_studies.rs`'s equivalent case still pass unchanged (the
      degrade-to-certified-selection contract for the genuinely proof-gated
      flags must not regress).
- [ ] `cargo test -p bcinr-cmca --features std` full suite green.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs`
- `crates/bcinr-cmca/src/generated/stability_profile.rs` (if a new
  `ETA_G_MAX` constant is added — check whether this file is truly
  hand-written or has a real generation path, per CMCA-105)
- `crates/bcinr-cmca/tests/case_studies.rs` or a new focused test file

## Related

- CMCA-103 (the fix this bug directly extends/completes)
- CMCA-122 (misleading refusal-reason bucketing for `eta_err`, a related but
  separate defect found in the same review pass)
