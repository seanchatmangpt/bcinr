# CMCA-109: power(0, negative q) silently returns a saturated max value tagged as "no fault"

**Type:** Bug
**Priority:** Critical (real correctness bug in shipped, currently-used production code)

## Summary

`allocator::power`'s zero-base/negative-exponent branch returns
`NonNegativeFixed::MAX` (`val = u32::MAX`, ~65535.99998) with `err = u32::MAX`
— this crate's "no fault, fully valid" sentinel — for an input that is
mathematically `0^(negative) = +infinity`, undefined/degenerate. A caller
checking `result.err != u32::MAX` (the crate's own documented convention for
"is this safe to use") sees a perfectly valid-looking large number, not a
refusal, for an input this function structurally cannot answer correctly.

## Context

Found by adversarial review of `CMCA-106`'s full-domain `power()` error sweep
(`crates/bcinr-cmca/tests/power_error_bound.rs`), which never swept `base = 0`
and therefore never surfaced this.

`crates/bcinr-cmca/src/allocator/mod.rs:1004-1031` (`power`): for
`base_is_zero` and `exp_gt_zero == 0, exp_eq_zero == 0` (i.e. exponent strictly
negative), the branchless select chain computes:

```
zero_res = const_select_u32(exp_gt_zero, 0, u32::MAX)   // = u32::MAX (saturated)
err      = const_select_u32(base_is_zero, u32::MAX, pow_val.err)  // = u32::MAX (OK)
```

Confirmed `u32::MAX` is the crate-wide "ok" sentinel for `err` at
`crates/bcinr-cmca/src/fixed.rs:71,77,81` (every "ok" constructor sets
`err: u32::MAX`). So this path returns a saturated-max value **tagged as
valid**, not refused — for a case that is `0^(negative)`, which has no finite
correct answer.

This is a real, reachable production input, not a hypothetical: this
crate's own shipped `OBJECT_REGISTRY` includes `PackedSemanticState`s with
exactly-zero factor values (confirmed in this session's own earlier review),
and `escort::escort_distribution` (the public API that calls `power` for
fractional `q`) accepts negative `q` up to `|q| <= 16` — so
`escort_distribution(masses_including_zero, negative_q)` can reach this exact
bug path today with real shipped data.

Confirmed unswept: `power_error_bound.rs`'s `base_grid()` runs `2^-6` to
`2^10`, always strictly positive (module doc explicitly states
`base == 0` is excluded), so CMCA-106's sweep never touched this input despite
it being fully in-domain for `escort_distribution`'s public contract.

## Acceptance Criteria

- [x] Decide the correct behavior for `base=0, exponent<0`: either refuse
      explicitly (set `err` to a real `StabilityRefusal`/fault code, matching
      how `cascade::escort_power` already handles the analogous
      `ZeroMassUnderNegativeLens` case for its exact-integer path — see
      `crates/bcinr-cmca/src/cascade.rs`) or document precisely why saturating
      to `MAX` and calling it "no fault" is the intended contract (if so,
      escort_distribution's callers need to know this explicitly, not
      discover it by reading `power`'s branchless internals).
      **Decided:** refuse explicitly. `power`'s `base=0, exponent<0` branch
      now sets `err = StabilityRefusal::UnsupportedDomain as u32`, matching
      the existing crate-wide convention `NonNegativeFixed::saturating_div`
      (zero denominator) and `SignedFixed::log2` (zero input) already use for
      this exact "undefined at zero" shape — see `fixed.rs`.
- [x] Fix `power` (or wrap it at the `escort_distribution` call site) to match
      the decided behavior. Fixed directly in `power`
      (`crates/bcinr-cmca/src/allocator/mod.rs`); `escort_distribution`
      needed no code change since it already checked `w.err != u32::MAX` and
      now inherits the correct refusal automatically (its stale doc comment
      describing the old saturating behavior was updated).
- [x] Add a regression test to `power_error_bound.rs` (or a new focused test)
      covering `base=0` with negative, zero, and positive exponents —
      currently entirely unswept.
      Added `power_zero_base_negative_exponent_is_refused_not_saturated_ok`
      and `power_zero_base_nonnegative_exponent_stays_ok` to
      `crates/bcinr-cmca/tests/power_error_bound.rs`.
- [x] Add a regression test exercising `escort_distribution` with a real
      zero-mass sibling under a negative `q`, asserting the crate's decided
      contract (refusal or documented saturation) rather than silently
      passing.
      Added `zero_mass_under_fractional_negative_q_is_refused` to
      `crates/bcinr-cmca/src/escort.rs`'s test module. Also updated
      `runtime_semantic_classification.rs`'s
      `escort_distribution_fractional_negative_lens_diverges_from_integer_path`
      (renamed to `..._now_names_the_zero_mass_element`), which had pinned
      the old bug's downstream symptom (collapse to a generic
      `DegenerateNormalization`) as documented, permanent behavior — it now
      asserts the fixed behavior (`EscortRefusal::NumericFault` naming the
      zero-mass element).
- [x] `cargo test -p bcinr-cmca --features std` stays green with the fix.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs` (`power`)
- `crates/bcinr-cmca/src/escort.rs` (`escort_distribution`'s handling, per its own doc comment already flagging "a zero mass under q < 0 follows power's own zero-base convention... rather than a dedicated refusal")
- `crates/bcinr-cmca/tests/power_error_bound.rs`
