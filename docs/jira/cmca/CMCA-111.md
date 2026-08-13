# CMCA-111: allocate_single_lens's "reproduces the blend" guarantee is false whenever MWU weights have actually updated

**Type:** Bug / Documentation Overclaim
**Priority:** High

## Summary

`allocate_single_lens`'s doc comment states, unconditionally, that
`sum_{k,q} lambda[k][q] * allocate_single_lens(..., k, q, ...)` reproduces
`allocate()`'s `pi_combined`. This is only true when the MWU weight update
inside `allocate_in` is a no-op — i.e. all-zero `payoffs`, exactly the
scenario the shipped verification test uses. For any real call with non-zero
payoffs where the divergence guard admits an update (`kappa > epsilon_kappa`),
`allocate_in`'s internal `local_weights` diverge from the raw `weights` a
caller holds, and `allocate_single_lens` computed on the raw weights will not
reproduce the per-lens term `allocate_in` actually summed.

## Context

Found by adversarial review of `allocate_single_lens`
(`crates/bcinr-cmca/src/allocator/mod.rs:2060-2226`), specifically by tracing
what `allocate_in` actually does to `weights` versus what
`allocate_single_lens` assumes.

- `allocate_in` (mod.rs:1444, 1457) copies the caller's `weights` into
  `local_weights`, then **mutates** `local_weights` via a real multiplicative
  MWU update (mod.rs:1650-1679: `w_flat * exp(beta*payoff)`, gated by
  `kappa_exceeds`/`update_allowed`) *before* calling `compute_pi_kq_for_kq`
  (mod.rs:1738-1747) with the **post-update** weights.
- `allocate_single_lens` (mod.rs:2119-2226) takes the caller's `weights`
  snapshot as-is and passes it straight into `compute_pi_kq_for_kq`
  (mod.rs:2216-2225) — no MWU update is applied.
- The doc comment (mod.rs:2093-2102) states the identity holds
  unconditionally — no caveat about payoffs or weight-update state.
- Why the shipped test doesn't catch this:
  `blend_equals_the_lambda_weighted_sum_of_single_lens_results`
  (`tests/single_lens_allocation.rs:157-241`) sets `payoffs = [[ZERO; 2*Q]; N]`
  (line 165). With all-zero payoffs, `exp(beta*payoff) == exp(0) == 1`, so the
  multiplicative update is a no-op regardless of whether `is_updating` fires.
  The only remaining change to `local_weights` is a scale-invariant
  renormalization (mod.rs:1681-1696) that leaves each lens's `rho` ratio
  unchanged — so the test's own scenario is structurally incapable of
  exercising the divergence this ticket describes.

## Acceptance Criteria

- [ ] Decide the correct fix: either (a) narrow the doc comment's claim to
      state the precondition explicitly (identity holds only when `weights`
      passed to `allocate_single_lens` is the post-MWU-update snapshot from
      an `allocate`/`allocate_in` call with matching inputs — and even then,
      only if the caller reconstructs that exact post-update state, which
      nothing currently helps them do), or (b) give `allocate_single_lens`
      (or a new variant) a way to apply the same MWU update logic so the
      identity can hold for real, non-degenerate calls.
- [ ] Add a regression test with **non-zero, differentiated payoffs** (not
      the degenerate all-zero case) that actually exercises weight
      divergence between the two paths, and assert the corrected
      documented behavior (either the narrowed claim holds, or the new
      reconciliation mechanism works).
- [ ] Update the doc comment's "core architectural claim" language
      (referenced in `tests/single_lens_allocation.rs:141-143`) to match
      whatever is actually true after the fix — no unconditional identity
      claim should ship without a test that can falsify it.

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs` (`allocate_single_lens` doc comment, possibly its implementation)
- `crates/bcinr-cmca/tests/single_lens_allocation.rs`
