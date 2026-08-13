# CMCA-113: stability_profile.rs's production-critical constants are opaque, hand-picked, and untraceable

**Type:** Design Gap / Production Readiness
**Priority:** High (directly relevant given autofde-lab production tuning is imminent)

## Summary

`crates/bcinr-cmca/src/generated/stability_profile.rs` carries constants that
are genuinely load-bearing in production allocator logic —
`CERTIFICATE_DIGEST` (a hardcoded 32-byte literal gating whether `allocate_in`
admits a call at all), `MODE_DWELL_ROUNDS_MIN` (461, gates dwell-time
refusal), `GAIN_MATRIX`/`WEIGHT_VECTOR`/`CONTRACTION_MARGIN` (feed a numeric
stability inequality) — but none of them have any ontology backing, derivation
record, or formal process. CMCA-105 already confirmed this file has zero
ontology/generator coverage (present hand-written since the crate's first
commit). This ticket names the specific, concrete risk that gap creates:
someone tunes one of these numbers for a real deployment (exactly what
autofde-lab integration will require) with no way to check the edit against
anything, and no test would catch a "reasonable-looking but wrong" change.

## Context

Found by adversarial review of CMCA-105's generator fixes, extending that
ticket's honest "no ontology backing" finding into its concrete production
consequence.

- `crates/bcinr-cmca/src/generated/stability_profile.rs:96`:
  `MODE_DWELL_ROUNDS_MIN = 461` — no comment explaining why 461, no
  derivation.
- `stability_profile.rs:104-108`: `CERTIFICATE_DIGEST`, a hardcoded 32-byte
  array literal with zero derivation shown anywhere in the repo.
- Consumed as hard gates: `crates/bcinr-cmca/src/allocator/mod.rs:1468-1501` —
  `CERTIFICATE_DIGEST` is compared byte-for-byte against a caller-supplied
  digest to authorize allocation at all (mod.rs:1469-1475);
  `GAIN_MATRIX`/`WEIGHT_VECTOR`/`CONTRACTION_MARGIN` feed a numeric stability
  inequality (mod.rs:1478-1489); `MODE_DWELL_ROUNDS_MIN` gates dwell-time
  refusal (mod.rs:1499-1502).
  `crates/bcinr-cmca/src/allocation_receipt.rs:411` also threads
  `CERTIFICATE_DIGEST` directly into a test as if it were a legitimate
  provenance hash.
- Unlike `RDF_INPUT_DIGEST`/`GENERATOR_SOURCE_DIGEST` (real `sha256` of real
  files, `generator.py:140-141`), none of these have a computed/checkable
  derivation.
- This is exactly the failure shape the `MAX_LENS_MAGNITUDE` domain-comment
  bug (fixed earlier this session, see the `feat(cmca): expose
  allocate_single_lens...` commit) demonstrated can happen when a value's
  provenance isn't traceable — a real, previously-shipped bug in this same
  crate, in this same class of "policy constant with no derivation record."

## Acceptance Criteria

- [x] For each load-bearing constant in `stability_profile.rs`
      (`CERTIFICATE_DIGEST`, `MODE_DWELL_ROUNDS_MIN`, `GAIN_MATRIX`,
      `WEIGHT_VECTOR`, `CONTRACTION_MARGIN`, and any others found on review),
      document its real provenance: was it derived from a stated formula
      (like `RDF_INPUT_DIGEST`), chosen as a policy decision by a named
      owner (matching `generated_profile.rs`'s own `POLICY (owner: ...)`
      convention), or is it genuinely arbitrary and needs to be replaced
      before production tuning begins? Done: every field on
      `StabilityProfile` now carries a doc comment stating one of `POLICY
      (owner: bcinr-cmca::allocator)` (for `gain_matrix`/`weight_vector`/
      `deterministic_margin`, which are jointly constrained by the
      contraction inequality; `minimum_dwell_rounds`; `certificate_digest`)
      or `ARBITRARY` (for `noise_second_moment_bounds`,
      `certified_noise_radius`, `mode_jump_bound`,
      `certified_switching_radius`, `total_homeostatic_radius`,
      `temperature_ceiling`, `distinguishability_floor`, `floor_minimum` —
      no formula or named-owner rationale could be found for these on
      review; honestly marked rather than assigned an invented derivation).
- [x] For policy constants: add the same `POLICY (owner: ...)` doc-comment
      convention `generated_profile.rs` already uses, so a future editor
      knows who to consult before changing a value. Done (see above); the
      misleading "GENERATED STABILITY PROFILE" file header (this file is
      hand-authored, per CMCA-105) was also corrected to stop implying
      machine provenance it does not have.
- [x] Decide whether `CERTIFICATE_DIGEST` in particular (an authorization
      gate, not just a numeric tuning knob) needs a real generation/rotation
      process before autofde-lab starts depending on it — a hardcoded,
      undocumented 32-byte "certificate" gating production allocation calls
      is a real operational risk if nobody currently knows how to correctly
      change it. Decision recorded in the doc comment on
      `certificate_digest`: today it is a same-crate round-trip
      self-consistency check, not caller-independent authorization (any
      caller can import the exact constant `allocate_in` checks against, as
      this crate's own doctest and `allocation_receipt.rs` tests already
      do) — building a real, caller-independent authorization mechanism is
      out of this ticket's scope and tracked under CMCA-114 (the sibling
      finding that the surrounding "certified" proof chain is itself
      unvalidated `Self { .. }` construction).
- [x] If any of these should be under real ontology/generator coverage,
      scope that as a follow-up (may overlap with CMCA-105's own remaining
      work). Recorded: the `ARBITRARY`-marked constants above and
      `CERTIFICATE_DIGEST`'s rotation process are the concrete follow-up
      scope; both are named explicitly in their doc comments rather than
      left implicit, so a future generator/ontology pass has a checklist.

## What was actually verified, not just documented

Documentation alone would not have addressed this ticket's real risk (FMEA:
Detection=9 — "no test... would catch a 'reasonable-looking but wrong'
edit"). `tests/stability_profile_invariants.rs` (new) adds a real regression
test: it recomputes `allocate_in`'s diagonal-dominance/contraction
inequality (`sum_j gain_matrix[i][j] * weight_vector[j] <=
weight_vector[i] * (1 - deterministic_margin)`) directly against the live
`GAIN_MATRIX`/`WEIGHT_VECTOR`/`CONTRACTION_MARGIN` constants using the same
1e9-scaled fixed-point integer arithmetic the production code uses, so an
edit that breaks the invariant now fails CI instead of shipping silently. It
also exercises `CERTIFICATE_DIGEST`'s real, current behavior (round-trip
match succeeds; a single flipped byte is refused) as a running assertion,
not just a prose claim.

## Files likely touched

- `crates/bcinr-cmca/src/generated/stability_profile.rs`
- `crates/bcinr-cmca/src/allocator/mod.rs` (consumers, for cross-reference doc comments)

## Related

- CMCA-105 (already found this file has no ontology backing; this ticket is
  the concrete production-risk follow-up)
