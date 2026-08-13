# CMCA-122: eta_err reports the wrong refusal reason when it co-occurs with price_err, and mutant_5's clip() is now provably dead code on Ok paths

**Type:** Bug / Test Gap
**Priority:** Medium

## Summary

Two small, related findings from the CMCA-103 review, both about the
`err_val` priority chain in `allocate_in`:

1. `eta_err` is bundled into the same reported-reason bucket as
   `lr_err`/`beta_err`, surfacing as `StabilityRefusal::LearningRateOutsideEnvelope`
   — semantically wrong, since an explore-floor `eta` violation has nothing
   to do with the learning-rate parameters `zeta`/`beta`. `price_err` has no
   dedicated branch in the priority chain at all; it only surfaces via the
   final fallback (`PriceGainUnsafe`). If both `eta_err` and `price_err` are
   true simultaneously, the reported reason is `LearningRateOutsideEnvelope`,
   masking the real `PriceGainUnsafe` condition — a caller debugging off the
   returned `StabilityRefusal` gets sent to the wrong subsystem.
2. Since CMCA-103 made `price_err` refuse unconditionally, and `mu` is
   `NonNegativeFixed` (cannot be negative), `clip(mu[x], ZERO, mu_max)`
   (mod.rs, in the pricing computation) can only ever be non-identity when
   `mu[x] > mu_max` — which is now *always* refused before that computation's
   result can be observed on any `Ok` path. The `clip()` call is now
   effectively dead code for distinguishing "clamped" from "not clamped" in
   any successful return, which is also why `hostile_mutants.rs`'s
   `kill_mutant_5_consequence_truncation` can no longer distinguish that
   mutant through the public API (already flagged, unresolved, in
   CMCA-103.md).

## Context

Found by adversarial review of the CMCA-103 fix, specifically the `err_val`
selection chain in `crates/bcinr-cmca/src/allocator/mod.rs` (search for
`err_val` near the end of `allocate_in`).

## Acceptance Criteria

- [ ] Give `eta_err` its own dedicated `StabilityRefusal` variant (or at
      minimum its own priority-chain branch reporting something other than
      `LearningRateOutsideEnvelope`), and add a dedicated branch for
      `price_err` instead of relying on the fallback.
- [ ] Add a regression test: construct a case where both `eta_err` and
      `price_err` are true simultaneously, and assert the returned
      `StabilityRefusal` correctly identifies (or at minimum doesn't
      misidentify) the actual problem.
- [ ] Resolve the `clip()`-on-`mu_actual` dead-code implication: either
      remove the now-redundant `clip()` call (the admission gate already
      enforces the invariant it existed to protect) or replace it with a
      `debug_assert!` making the defense-in-depth explicit, and separately
      design a real fix for `kill_mutant_5_consequence_truncation`'s
      now-broken coverage (a `#[cfg(test)]`-only or unit-level hook that can
      still observe `clip()`'s behavior directly, bypassing the admission
      gate) rather than leaving it as an unresolved comment.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs`
- `crates/bcinr-cmca/tests/hostile_mutants.rs`

## Related

- CMCA-103, CMCA-110 (all findings from the same review of the `has_refusal`/`err_val` machinery)
