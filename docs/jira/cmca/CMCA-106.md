# CMCA-106: allocator::power error bound unmeasured beyond |q|=16, already 36% at the admitted domain edge

**Type:** Correctness / Numerical Verification
**Priority:** High

## Summary

escort.rs's fractional-q fallback (allocator::power) has an empirically swept relative-error bound of ~7.6% for |q|&lt;=4, growing to ~36% near |q|=16 -- and cascade::MAX_LENS_MAGNITUDE (the domain escort_distribution actually admits) is exactly 16. The admitted domain's edge is already documented at 36% error, and nothing past |q|=16 has been measured at all, so the full extent of the gap between "admitted" and "measured-safe" is unknown.

## Context

NOTE FOR WHOEVER PICKS THIS UP: a parallel hardening pass in this same working session may have already extended tests/power_error_bound.rs and/or added a refusal path. Before starting new work, check `git log --oneline -- crates/bcinr-cmca/tests/power_error_bound.rs crates/bcinr-cmca/src/escort.rs crates/bcinr-cmca/src/cascade.rs` and re-read the current escort.rs doc comment (~lines 56-64) for whether the "no bound is claimed... at larger |q|" language has already been replaced. This ticket is written as a fresh report of the gap as it stood as of commit d45e9ee0; treat it as possibly already IN PROGRESS or DONE.

Source: crates/bcinr-cmca/src/escort.rs, doc comment ~lines 56-64 (module-level docs on `allocator::power`'s relationship to `escort_distribution`):

"A domain-wide characterization of `power`'s own relative error (not mixed with normalization cancellation...) now exists in `tests/power_error_bound.rs`: an empirical, swept bound of ~7.6% relative error for `|q| &lt;= 4`, with error measured to grow with `|q|` (up to ~36% near `|q| = 16`) -- so no bound is claimed, here or there, for `power`'s behavior at larger `|q|`."

`escort_distribution` refuses any `q` whose magnitude exceeds `cascade::MAX_LENS_MAGNITUDE` (crates/bcinr-cmca/src/cascade.rs:60, `pub const MAX_LENS_MAGNITUDE: u32 = 16`) -- the same constant cascade.rs itself calls "a provisional domain, not one derived from a specification" (escort.rs doc, following paragraph). So the full admitted input range for fractional q runs right up to the boundary where the measured error is already ~36%, and the sweep (tests/power_error_bound.rs, added in commit 9764d2c1, "Checkpoint F") does not appear to extend past |q|=16 at all -- meaning callers can legally pass q values at or near the domain boundary with no stated worst-case error bound, and no refusal triggers on that basis (the only refusal is the hard |q|&gt;16 domain check in cascade.rs:289, which fires on the boundary itself, not on error magnitude).

This is a real production-readiness gap, not a cosmetic one: any consumer of `escort_distribution` with fractional q near |q|=16 is getting normalized-power output whose accuracy is documented as unverified past a measured 36% relative error at the edge, with silent degradation (no distinct error/refusal signal tied to expected error magnitude) as q approaches that edge from either side.

## Acceptance Criteria

- [ ] Before starting: confirm via git log / current file contents whether this gap has already been closed by a concurrent session; if so, close this ticket with a pointer to the commit(s) instead of duplicating work.
- [ ] The empirical sweep in tests/power_error_bound.rs is extended to cover the full admitted domain for fractional q (i.e., swept at least up to |q| = MAX_LENS_MAGNITUDE = 16, not stopping short at the ~36%-near-16 sample point referenced in the doc comment), OR MAX_LENS_MAGNITUDE (or a separate fractional-q-specific bound) is narrowed to a range that is actually measured-safe, with the narrower bound enforced by a real refusal in code, not just documented.
- [ ] Either (a) a hard, numerically-stated worst-case relative-error bound for allocator::power is documented across the full admitted domain (replacing the current 'no bound is claimed... at larger |q|' language in escort.rs with real numbers), or (b) escort_distribution / allocator::power gains a refusal path that rejects (or flags) inputs where the swept/interpolated expected error would exceed a stated threshold -- not silent degradation.
- [ ] The escort.rs module doc comment (~lines 56-64 as of commit d45e9ee0) is updated to match whichever of the above was implemented, so the doc no longer disclaims a bound at the domain edge if one now exists.
- [ ] New/updated coverage is exercised by a real test run (not a memory of a prior run) whose output is captured as verification evidence before the ticket is marked done, per this project's evidence-before-assertions discipline.

## Files likely touched

- `crates/bcinr-cmca/src/escort.rs`
- `crates/bcinr-cmca/src/cascade.rs`
- `crates/bcinr-cmca/tests/power_error_bound.rs`
- `crates/bcinr-cmca/src/allocator.rs`
