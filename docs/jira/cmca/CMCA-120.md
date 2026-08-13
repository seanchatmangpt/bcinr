# CMCA-120: CMCA-107's regression test only covers the negative (no-update) path, and compute_kappa redundantly recomputes 8x per call

**Type:** Test Gap / Performance
**Priority:** Medium

## Summary

Two findings from the same review pass, both about `compute_kappa`'s
integration into `allocate_in`:

1. The CMCA-107 regression test
   (`cmca_107_single_child_node_kappa_is_always_zero_gates_weight_update`)
   only exercises the *negative* path (kappa=0, update correctly withheld).
   It never asserts that a multi-child node in the same fixture *does* have
   `kappa > epsilon_kappa` for some lens and that its weights *do* update and
   match the f64 oracle within tolerance — the guard's "admit" branch is
   unverified by any deterministic test.
2. `compute_kappa` is invoked once per `(v, q_idx)` pair — 32 times per
   `allocate_in` call — but its `mass_pow` array only actually depends on
   `q_idx` (4 distinct values), not `v`. This is an 8x redundant
   recomputation of an O(N) `fixed_pow` array, contradicting this crate's
   own stated CC=1/branchless/hot-path-performance design philosophy.

## Context

Found by adversarial review of `compute_kappa`/`fixed_pow`
(`crates/bcinr-cmca/src/allocator/mod.rs:1348-1424`) and its call site
(mod.rs:1633-1659, inside `unroll_8_static!(v, {... unroll_4_static!(q_idx,
{...})})`).

- Test gap: `crates/bcinr-cmca/tests/differential.rs`'s
  `cmca_107_single_child_node_kappa_is_always_zero_gates_weight_update`
  (search for that name) asserts only node 4 (forced kappa=0 by
  construction) stays pinned, plus a loose downstream tolerance check on
  node 5. It never checks nodes 0/1/2 (all multi-child in the same fixture's
  `parent = [-1, -1, 0, 2, 1, 4, 1, 1]`), which should have genuinely
  positive kappa for at least some lens.
- Perf gap: `mass_pow` (mod.rs:1382-1385, inside `compute_kappa`) is a
  function purely of `node_masses[MEASURE_CACHE]` and `q_val` — neither
  varies across the `v` loop for a fixed `q_idx`. The correct amount of work
  is 4 (one per `q_idx`) × 8 (`fixed_pow` calls per array) = 32 total; the
  current code does 8 (per `v`) × 4 (per `q_idx`) × 8 = 256.

## Acceptance Criteria

- [ ] Add a regression test asserting the positive path: construct a case
      where a multi-child node's kappa genuinely exceeds `epsilon_kappa`,
      and assert its weights actually update (differ from initial) and
      match the f64 oracle within a measured tolerance.
- [ ] Hoist `mass_pow` computation out of the `v` loop in `allocate_in` (or
      restructure `compute_kappa`'s call site) so it's computed once per
      `q_idx`, not once per `(v, q_idx)` pair — an 8x reduction in
      `fixed_pow` calls with no behavior change.
- [ ] Benchmark before/after if this crate has a benchmarking harness
      (`bcinr-bench`) to confirm the expected improvement, matching this
      repo's own "optimize algorithm: profile -> identify bottleneck ->
      implement -> benchmark -> commit with % improvement" workflow from
      CLAUDE.md.
- [ ] `cargo test -p bcinr-cmca --features std` full suite green after the
      refactor (no behavior change expected, but verify).

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs`
- `crates/bcinr-cmca/tests/differential.rs`

## Related

- CMCA-107, CMCA-112 (same code, different concerns from the same review pass)
