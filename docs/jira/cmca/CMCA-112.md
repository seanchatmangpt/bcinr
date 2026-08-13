# CMCA-112: compute_kappa's fixed_pow saturates for in-domain masses, destroying the divergence guard's precision exactly where it matters

**Type:** Bug
**Priority:** High

## Summary

`fixed_pow`'s doc comment claims the max-shift-stabilization constant is
unnecessary because "kappa only ever consumes ratios of these values... a
missing max-shift constant cancels out algebraically." This is false in
Q16.16: individual `mass_pow[i]` terms saturate to `NonNegativeFixed::MAX` or
underflow to exactly `0` for mass/`q` combinations that are fully inside this
crate's own documented feasible region and proptest domain — and once two
sibling terms both saturate to the same value, the ratio the comment claims
"cancels" the problem instead **destroys the differentiation kappa depends
on**. Separately, the `0/0` case in `compute_kappa` saturates to `MAX` rather
than failing safe, diverging from the f64 reference oracle's `NaN`
fail-to-no-update semantics — in the exact dimension CMCA-107 was opened to
fix.

## Context

Found by adversarial review of CMCA-107's `compute_kappa`/`fixed_pow`
(`crates/bcinr-cmca/src/allocator/mod.rs:1348-1424`).

**Saturation destroys differentiation (not merely loses precision):**
`fixed_pow` saturates to `MAX` whenever `mass >= 2^(16/q)` (per `exp2`'s
integer-part-≥16 saturation, `fixed.rs:348-349,366`) and underflows to `0`
when `mass < 2^(-17/q)` (`fixed.rs:351-352`). `FeasibleRegion::CURRENT` clips
masses into `[6/65536, 65536000/65536] ≈ [9.2e-5, 1000]`
(`feasible_region.rs:207-208`), and `test_differential_allocator`'s proptest
drives lens exponents over `-1.99..1.99` (`tests/differential.rs:357`). At
`q=1.99`, the saturation threshold is `2^(16/1.99) ≈ 263` — well inside
`[m_min, 1000]`. Two sibling masses both `≥ 263` (e.g. 300 and 1000, true
ratio ≈1:23 in `mass^q` space) collapse to the identical saturated value in
`mass_pow[]`, making `s_meas` uniform across them regardless of their real
relative mass. Verified by hand: at `q=1.99, mass=m_min≈9.2e-5`,
`log2(m_min)≈-13.4`, exponent ≈-26.7, underflows to exactly 0. Neither the
CMCA-107 regression test (`lens_qs` max out at 1.5, factor values kept well
under the saturation threshold) nor default-configuration proptest (`PROPTEST_CASES=1`)
gives meaningful coverage of this region.

**`0/0` divergence from the f64 oracle's fail-safe:**
`s_meas`'s division (mod.rs:1415-1422, via `saturating_div`) forces
`val = u32::MAX` when the denominator is `0`, regardless of numerator
(`fixed.rs:191-201`, `den_is_zero` branch). The f64 reference oracle's
equivalent `0.0/0.0` (`tests/reference.rs:154,163`) is `NaN`, and any
`kappa > epsilon_kappa` comparison against a NaN-poisoned kappa is `false`
under IEEE-754 — the f64 oracle fails safe to "no update." If every direct
child of a node underflows to `mass_pow=0` (reachable per the saturation
finding above) while a deeper subtree leaf does not, `sum_meas_den == 0` and
`s_meas` saturates to `MAX` — an out-of-domain "probability" fed into
`log2()`, producing a finite (not nulled) kappa contribution where the f64
oracle would instead gate the update off entirely. This is exactly the class
of fixed-vs-f64 divergence CMCA-107 exists to close, left open for this
specific extreme-mass-distribution input shape.

## Acceptance Criteria

- [x] Correct `fixed_pow`'s doc comment (it currently states an incorrect
      mathematical claim) or, better, fix the underlying saturation issue —
      determine whether a max-shift-stabilized variant (matching
      `compute_pi_kq_for_kq`'s own approach, which `fixed_pow`'s comment
      explicitly contrasts itself against) is needed here too.
      Resolution: confirmed saturation/underflow is reachable inside
      `FeasibleRegion::CURRENT` at `q` near the proptest domain's `1.99`
      bound (see the added regression tests). Corrected the doc comment to
      retract the false "cancels out algebraically" claim and documented
      that a genuine precision gap remains when two siblings both saturate
      to the identical `MAX` (as opposed to the `0/0` case, which is now
      fixed below) — a max-shift-stabilized `fixed_pow` variant would be
      required to close that remaining gap and is left as follow-up scope
      (not required by this ticket's `0/0` divergence fix).
- [x] Decide and implement the correct `0/0` behavior for `s_meas`/`s_leaf` in
      `compute_kappa` — either fail closed (treat as kappa=0/no-update,
      matching the f64 oracle's NaN-implies-false semantics) rather than
      saturating to a spurious large finite value.
      Resolution: `compute_kappa` now detects `sum_meas_den == 0` (the
      genuine `0/0` case for a child's `s_meas`) and excludes that child's
      contribution from `kappa` instead of trusting `saturating_div`'s
      forced `MAX`, matching the f64 oracle's `NaN`-poisons-to-no-update
      fail-safe (`kappa > epsilon_kappa` is `false` for both `NaN` in f64
      and the excluded/zero contribution in fixed-point).
- [x] Add regression tests sweeping mass/`q` combinations near the
      saturation/underflow boundaries identified above (e.g. masses ≥ 263 at
      `q` near 2, masses near `m_min` at `q` near 2) and assert
      `compute_kappa`'s fixed-point result matches the f64 oracle within a
      measured tolerance — this is currently untested by both the CMCA-107
      regression test and default-configuration proptest.
      Resolution: added `allocator::kappa_saturation_tests` (in
      `crates/bcinr-cmca/src/allocator/mod.rs`) covering the saturation
      boundary (mass=300/1000 at q=1.99), the underflow boundary (mass near
      `m_min` at q=1.99), the `0/0` fail-safe (direct child underflows while
      a deeper subtree leaf does not), and a non-degenerate sanity case
      confirming the fail-safe doesn't zero out real divergence signals.
- [x] `cargo test -p bcinr-cmca --features std` full suite green.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs` (`fixed_pow`, `compute_kappa`)
- `crates/bcinr-cmca/tests/differential.rs` or a new focused test file

## Related

- CMCA-107 (the ticket this bug directly extends — the divergence guard this
  session added has its own precision gap)
- CMCA-120 (the CMCA-107 regression test's positive-path coverage gap and the
  8x redundant `fixed_pow` recomputation, found in the same review pass)
