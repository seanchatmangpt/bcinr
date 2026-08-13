# CMCA-104: DIFFERENTIAL_TOLERANCE placeholder (0.22) lets the fixed-vs-f64 differential test pass regardless of magnitude

**Type:** Test Rigor / Correctness
**Priority:** High for the "production ready" claim specifically; Medium otherwise — a differential test that structurally cannot fail on magnitude provides false confidence for that claim without being a blocking correctness bug on its own.

## Summary

crates/bcinr-cmca/src/generated_profile.rs:20-21 documents DIFFERENTIAL_TOLERANCE: f64 = 0.22 as its own doc comment says: "not a tolerance so much as an absence of one... chosen to make the fixed-vs-f64 comparison pass, not derived from the numeric profile... a placeholder." Consistent with that, tests/differential.rs:266-286 already does not assert on the actual fixed-vs-f64 magnitude diff at all — it downgrades that check to informational-only handling (the only hard asserts left are finiteness/non-negativity of each side), so the differential test cannot currently fail on a real numeric-agreement regression between the fixed-point and f64 escort computations.

## Context

NOTE — check before starting work: a parallel hardening pass in this same session was tasked with implementing exactly this fix in crates/bcinr-cmca/tests/differential.rs. This ticket may already be IN PROGRESS or DONE. Verify current state of tests/differential.rs and generated_profile.rs (git log / git blame on both, and re-run the test) before starting new work on this.

Background: generated_profile.rs is a `ggen sync`-generated file whose own doc convention requires every constant to carry either a real derivation or an explicit UNDERIVED/POLICY marker with an owner. DIFFERENTIAL_TOLERANCE is documented as UNDERIVED-and-placeholder, and the doc comment itself proposes the fix: classify each generated differential case as inside or outside the executable envelope (using ESCORT_DYNAMIC_RANGE_LIMIT, defined at generated_profile.rs:24-25 as the max-shift-stabilized escort kernel's representable spread — max_j(q*log2(m_j)) - min_j(q*log2(m_j)) < 16), then:
- inside the envelope: assert fixed-vs-f64 agreement to an error bound *derived* from ESCORT_DYNAMIC_RANGE_LIMIT (not the flat 0.22 placeholder)
- outside the envelope: assert that the computation returns NumericRangeExceeded instead of silently comparing two numbers that were never expected to agree

Until this lands, the differential suite exercises finiteness/non-negativity only and cannot catch a real regression in fixed-point vs f64 agreement.

## Acceptance Criteria

- [ ] Confirm current state first: check git log/blame on crates/bcinr-cmca/tests/differential.rs and src/generated_profile.rs, and re-run `cargo test -p bcinr-cmca differential` to see whether this fix is already in progress or complete before starting new work.
- [ ] Each generated differential case in tests/differential.rs is classified as inside or outside the executable envelope, using ESCORT_DYNAMIC_RANGE_LIMIT (src/generated_profile.rs) as the classification bound.
- [ ] For cases inside the envelope: the println!/diagnostic-only handling is replaced with a hard assert!/assert_eq!-style check comparing fixed-vs-f64 magnitude against an error bound that is derived from ESCORT_DYNAMIC_RANGE_LIMIT (not the flat DIFFERENTIAL_TOLERANCE placeholder).
- [ ] For cases outside the envelope: the test asserts that the computation returns NumericRangeExceeded rather than comparing fixed-point and f64 values.
- [ ] DIFFERENTIAL_TOLERANCE's doc comment and/or the constant itself is updated or removed once no longer used as the placeholder driving this comparison, so generated_profile.rs no longer documents a still-live 'this is a placeholder' constant that has in fact been superseded.
- [ ] A deliberately-broken escort computation (e.g. a seeded/injected fixed-point or f64 miscalculation within the envelope) is verified to make the updated test fail, proving the differential test now actually gates on numeric agreement rather than only on finiteness/non-negativity.
- [ ] cargo test -p bcinr-cmca passes with the new hard assertions in place (on the non-broken code path).

## Files likely touched

- `crates/bcinr-cmca/tests/differential.rs`
- `crates/bcinr-cmca/src/generated_profile.rs`
