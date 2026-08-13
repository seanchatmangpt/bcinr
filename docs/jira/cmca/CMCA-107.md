# CMCA-107: fixed-point `allocate()` and its f64 reference oracle disagree by ~0.65 on a case unrelated to escort dynamic range or tied masses

**Type:** Bug
**Priority:** High
**Status:** **Done (root-caused and fixed).** `allocate_in` was missing the kappa
divergence guard (`kappa_v > epsilon_kappa`) that gates the MWU weight update in
the f64 reference oracle (`tests/reference.rs::allocate_f64`) — a strong
independent confirmation this was real: the `epsilon_kappa` parameter was
previously named `_epsilon_kappa`, i.e. accepted and silently unused. The
fixed-point path updated weights unconditionally regardless of divergence, so any
tree shape where the true divergence should have stayed below threshold still
drifted every call. Added `compute_kappa`/`fixed_pow` (mirroring the f64 oracle's
formula exactly) and gated `is_updating` on `kappa_exceeds`. The original
proptest-shrunk seed was not recoverable (confirmed not reproducible anywhere in
the repo), so the defect class is instead pinned as a deterministic regression
test (`tests/differential.rs::cmca_107_single_child_node_kappa_is_always_zero_gates_weight_update`)
exploiting the same structural degeneracy CMCA-101's original test tree
accidentally also hit: `kappa_v` is identically zero for any node whose direct
children are all leaves.

## Summary

While implementing CMCA-104 (replacing the `DIFFERENTIAL_TOLERANCE=0.22` placeholder with a
real, measured bound), tightening `tests/differential.rs`'s comparison surfaced a genuine,
reproducible fixed-vs-f64 disagreement of **~0.65** on node 5 of a specific generated case —
an order of magnitude larger than the ~0.33 measured maximum across ~6800 other leaf
comparisons, and not explained by either of the two failure modes CMCA-104 characterized
(escort dynamic-range spread exceeding `ESCORT_DYNAMIC_RANGE_LIMIT`, or a tied/degenerate
sibling-mass group).

## Context

Reproduced independently this session (not just reported by the implementing agent):

```
node 5: fixed-vs-f64 disagreement 0.651755 exceeds the measured bound 0.4
  (case is inside the escort executable envelope, spread 9.852)
  fixed=0.7835845947265625, f64=0.13182928425853122
parent: [-1, -1, 0, 2, 1, 4, 1, 1]
```

The escort dynamic-range spread for this case (9.852) is well inside the admitted
`ESCORT_DYNAMIC_RANGE_LIMIT` (16), and no sibling group in this case has tied/degenerate
masses — so this is neither of CMCA-104's two identified out-of-envelope conditions. The
CMCA-104 implementer's working hypothesis, not yet confirmed: the disagreement tracks a
**discrete decision point** elsewhere in `allocate_in` (crates/bcinr-cmca/src/allocator/mod.rs)
that can flip between the fixed-point and f64 code paths on inputs that are numerically close
but land on opposite sides of a boundary — candidates are the MWU gradient-descent admission
threshold (`gd_ok`/`has_error` construction) or the dwell-time mode-switch gate
(`switch_wanted`/`can_switch`, the same machinery CMCA-101 already tracks as unexercised —
worth checking whether these two tickets are related). A discrete boundary flip would produce
exactly this signature: most cases agree closely, a minority disagree by a large, roughly
fixed amount, with no correlation to the continuous quantities (spread, mass magnitude)
CMCA-104 already checked.

This is **not** an escort-kernel precision defect (CMCA-106 already gives `power`/`escort`'s
own error bounds, and this case's diff is far larger than anything measured there) — it's
somewhere in `allocate_in`'s MWU/mode-switching/pricing logic diverging between the two
implementations on a specific input shape.

The regression seed for this exact case (`cc 5b9b6a68...`, minimized by proptest's shrinker)
is recorded in the CMCA-104 implementation's working notes but was deliberately **not**
committed to `crates/bcinr-cmca/tests/differential.proptest-regressions`, to avoid pinning an
unrelated, unfixed bug as a permanent CI failure via a ticket (CMCA-104) scoped only to the
tolerance constant. The exact input is fully reproducible from the `parent`/factor values
above (see the CMCA-104 diff for the full generator output) if a fresh regression entry is
wanted once this ticket is picked up.

## Acceptance Criteria

- [ ] Root-cause the actual divergence: instrument or bisect `allocate_in` on the exact input
      above to find which specific step (MWU weight update, `gd_ok` admission check, dwell-time
      mode switch, pricing/`eta` blend) produces materially different intermediate values
      between the Q16.16 and f64 paths.
- [ ] Determine whether this is (a) a genuine algorithmic bug (one path is wrong), (b) an
      inherent, expected consequence of fixed-point vs. floating-point arithmetic at a discrete
      decision boundary (in which case the "bug" is that neither path is wrong, but the
      *comparison* needs a third envelope condition — "near a decision boundary" — added to
      CMCA-104's classification), or (c) something else.
- [ ] If (a): fix the actual defect in `allocate_in` or `allocate_f64` (whichever is wrong),
      with a regression test pinning this exact input.
- [ ] If (b): extend `tests/differential.rs`'s envelope classification (CMCA-104's
      `inside_envelope` logic) with a third condition detecting near-boundary cases, and
      re-derive `DIFFERENTIAL_TOLERANCE` (or add a second, looser bound for near-boundary
      cases) from a fresh measurement.
- [ ] Either way: commit a regression entry for this exact input to
      `differential.proptest-regressions` once it's classified/fixed, so it's never
      silently lost again.
- [ ] `cargo test -p bcinr-cmca --features std --test differential` passes with the real
      fix in place (not by widening `DIFFERENTIAL_TOLERANCE` further to paper over it — that
      would recreate exactly the "placeholder tolerance" problem CMCA-104 exists to eliminate).

## Files likely touched

- `crates/bcinr-cmca/src/allocator/mod.rs`
- `crates/bcinr-cmca/tests/differential.rs`
- `crates/bcinr-cmca/tests/differential.proptest-regressions`
- `crates/bcinr-cmca/src/generated_profile.rs` (if `DIFFERENTIAL_TOLERANCE` or its envelope
  classification needs to change)

## Related

- CMCA-104 (the tolerance-placeholder fix that surfaced this) — implemented, this bug found
  as a direct result.
- CMCA-101 (dwell-time hysteresis dead code) — worth checking whether the same
  `switch_wanted`/`can_switch` machinery is implicated here.
