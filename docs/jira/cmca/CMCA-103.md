# CMCA-103: allocate_in silently accepts out-of-range q when proof=None (domain admission gate not enforced on common path)

**Type:** Bug
**Priority:** High
**Status:** **Done** — fixed with a narrower rule than this ticket's own default
framing proposed. Investigation found `digest_err`/`gd_ok`/`lr_err`/`beta_err`/
`dwell_err` are legitimately proof-gated (they're consumed only by the
weight-update/mode-switch code, already independently gated by `update_allowed`,
and a Chicago-TDD test — `jtbd_drift_refusal_routes_to_selection_only_without_state_drift`
— documents graceful degrade as intended for those). `q_err`, `price_err`, and
`eta_err` feed unconditionally into the selection computation itself and are now
refused unconditionally, regardless of `proof`. Three test files updated
(`case_studies.rs`, `hostile_mutants.rs`, `runtime_semantic_classification.rs`).
One follow-up flagged, not resolved: `hostile_mutants.rs::kill_mutant_5_consequence_truncation`
can no longer distinguish mutant_5 through the public API since its
out-of-domain `mu` input is now refused before mutant_5's own code path
matters — left with a comment, needs a redesigned mutation-test hook.

## Summary

allocator::allocate_in computes q_err (the ontology-declared q in [-2,2] admission check, crates/bcinr-cmca/src/allocator/mod.rs:1382-1386) unconditionally, but only turns it into an actual refusal when proof.is_some(). On the proof=None path -- the path exercised by nearly every existing test in the crate -- an out-of-range q is silently accepted and used in the computation instead of being refused.

## Context

Found while fixing an unrelated domain-inconsistency doc bug this session (MAX_LENS_MAGNITUDE=16 vs [-2,2] naming confusion in generated_profile.rs, already fixed as a docs-only change). This is a distinct, live enforcement gap, not the same issue.

Mechanism (crates/bcinr-cmca/src/allocator/mod.rs):
- Line 1344-1345: `let proof_some = proof.is_some(); let degrade_to_certified_selection = proof.is_none();`
- Line 1382-1386: `q_err` is computed unconditionally by scanning all lenses against `(-131072..=131072)` (Q16.16 fixed-point for the declared [-2,2] domain).
- Line 1401-1402: `has_error` folds `q_err` in along with the other checks (gd_ok, digest_err, lr_err, beta_err, eta_err, dwell_err, price_err) -- so far so good, has_error is correctly true.
- Line 1702-1703: `let has_error = has_error | has_cycle | numeric_has_err; let has_refusal = has_error & !degrade_to_certified_selection;` -- this is the actual gate. When proof is None, degrade_to_certified_selection is true, so has_refusal is forced false regardless of has_error/q_err, and the function proceeds to accept the state update (weights, last_switch_t, prev_mode all get set to the new values via const_select_u32(has_refusal, ...)) instead of refusing.
- Confirmed via crates/bcinr-cmca/tests/runtime_semantic_classification.rs:355 (`allocate_in_lens_domain_is_declared_but_not_enforced_without_proof`), which documents this exact gap as EXPERIMENTAL/known-not-enforced but is not itself a regression-blocking assertion of correct behavior -- it currently asserts the buggy (accepting) behavior, not a refusal.

Open question requiring investigation before the fix lands: is there a legitimate reason the proof=None / degrade-to-certified-selection path is meant to skip domain admission and only freeze the *learning rate* update (line 1403: `freeze_learning = has_error & degrade_to_certified_selection`) while still allowing state selection to proceed on invalid q? If the design intent is "no proof -> best-effort degrade, freeze learning, but still select using whatever config was validated at a different layer," that needs to be stated explicitly and the q admission check needs to happen upstream instead. Absent such a documented reason, the crate's typed-refusal discipline (StabilityRefusal::QRangeDestabilizing exists specifically for this) implies q_err should refuse unconditionally.

## Acceptance Criteria

- [ ] Investigate and document (in the fix's commit message and/or a code comment at line 1703) whether the proof=None / degrade_to_certified_selection path is intentionally meant to skip q-domain admission; if no legitimate reason is found, treat this as confirmed defect and proceed to fix.
- [ ] Fix has_refusal (or introduce a separate, unconditional gate) so that q_err (and ideally all StabilityRefusal-worthy conditions, not just the learning-rate freeze) causes allocate_in to return Err(StabilityRefusal::QRangeDestabilizing) regardless of proof.is_some()/is_none(), unless the investigation above finds q admission is deliberately delegated to an upstream caller -- in which case document that contract explicitly and add an assertion/debug_assert at the call boundary instead.
- [ ] Add a regression test (e.g. in crates/bcinr-cmca/tests/runtime_semantic_classification.rs or allocator/mod.rs unit tests) that calls allocate_in with proof=None and a lens q outside [-2,2] (matching the existing allocate_in_lens_domain_is_declared_but_not_enforced_without_proof setup) and asserts the call returns Err(StabilityRefusal::QRangeDestabilizing), replacing/updating the existing test that currently documents the gap as expected-not-enforced.
- [ ] Existing test suite (cargo test -p bcinr-cmca) passes after the fix, with no other test relying on the buggy silent-acceptance behavior on proof=None (grep for other proof: None call sites with out-of-range q in the test suite and update as needed).
- [ ] make check / make clippy / make fmt pass on the touched files per repo workflow in CLAUDE.md.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs`
- `crates/bcinr-cmca/tests/runtime_semantic_classification.rs`
