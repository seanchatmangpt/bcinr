# CMCA-121: dwell-time hysteresis test proves single-switch timing, not the spec's repeated-switching property; "fixed" tree still has a residual kappa=0 node

**Type:** Test Gap / Documentation
**Priority:** Low-Medium

## Summary

Two findings about `tests/dwell_time_hysteresis.rs`, both benign (no false
assertion, no incorrect behavior) but worth tracking:

1. The spec this test cites (`stability_proof_draft.md:101-106`,
   `ORIGINAL_REQUEST.md:1355`) describes a dwell-time bound over a
   *sequence* of switches (`N_switch(0,T) <= N0 + T/tau_D`). The test only
   ever drives one switch and never re-arms to check a *second* switch is
   also correctly held for a full `tau_d` measured from the new
   `last_switch_t` — a bug that only manifests on switch #2 (e.g. a stale
   baseline reused across switches) would not be caught.
2. The test's own doc comment frames its two-level tree
   (`parent = [-1, 0, 0, 1, 1, 1, 1, 1]`) as resolving CMCA-107's kappa=0
   degeneracy that forced the tree-shape change. It resolves it *at the
   root* (the node the test actually asserts on), but node 1 — also
   internal, with all-leaf children — has the identical kappa=0 degeneracy.
   This doesn't affect the test's correctness (the assertions only check
   root-level state), but the doc comment's framing ("resolves the issue")
   overclaims relative to what's actually true one level down.

## Context

Found by adversarial review of the CMCA-101 dwell-time hysteresis test.
Manually traced the `t=0` initial-state assumptions and confirmed them
correct (no defect there); confirmed no tautological assertions exist in
either test in the file.

- Single-switch-only coverage:
  `crates/bcinr-cmca/tests/dwell_time_hysteresis.rs:60-152`
  (`dwell_time_lock_holds_switch_until_tau_d_then_switches`) drives exactly
  one mode-0-to-mode-1 transition, asserted at `t=tau_d`, and stops.
- Residual kappa=0 node: per `compute_kappa`'s formula
  (`allocator/mod.rs:1387-1424`), node 1's direct children `{3,4,5,6,7}` are
  all leaves, so `is_subtree_leaf[c] == {c}` for each, making
  `s_meas(c) == s_leaf(c)` identically — `kappa(1)` is zero for the entire
  run, meaning node 1's own MWU weight updates are dead code in this test.

## Acceptance Criteria

- [ ] Add a second phase to (or a new test alongside)
      `dwell_time_lock_holds_switch_until_tau_d_then_switches` that, after
      the first switch fires at `t=tau_d`, continues driving `t` with a
      payoff bias favoring a switch back, and asserts the second switch is
      likewise held until a full `tau_d` has elapsed from the *new*
      `last_switch_t` — proving the repeated-switching property the spec
      actually states, not just a single instance.
- [ ] Either extend the test tree to a three-level shape where node 1 also
      has genuinely nonzero kappa (so its own weight update path gets
      exercised), or correct the doc comment to state precisely what's
      resolved (root-level kappa=0) versus what residual degeneracy remains
      (node 1) — don't leave the "resolves the issue" framing as broader
      than what's true.

## Files likely touched

- `crates/bcinr-cmca/tests/dwell_time_hysteresis.rs`

## Related

- CMCA-101, CMCA-107, CMCA-112
