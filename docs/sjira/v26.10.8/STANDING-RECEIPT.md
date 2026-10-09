# STANDING-RECEIPT — bcinr v26.10.8 doc-hdit certification (lane R111)

Standing: **CERTIFIED** — the doc-hdit court verdict is **ACCEPTED at this
exact subject** (`349afd7b`, tree clean). This receipt completes the
R10 → R29 → R69 lineage: R10 REFUSED (S_coverage 0.3480, 1733 undocumented
public items) → R29 doc-debt close-out (8 commits on `f3e9c23f`, S_coverage
0.9658) → R69 independent read-only re-witness (`rollout.sh --report-only`
at `b531b49c`, `/tmp/hdit/r69`: S_coverage 0.9658, Φ_halluc 0.000182,
Q_density 0.9998 — thresholds identical to R10, verdict ACCEPTED).

## Subject

- Repo: `/Users/sac/bcinr`, branch `bench/rdtsc-tick-tables` (== `main` ==
  `origin/main` @ `349afd7b82c898e5ff5aa9e65760d536960cb695`, working tree
  clean at receipt time)
- Court: `packs/rust-doc-hdit-pack/courts/doc_quality.court`
  (S_coverage ≥ 0.90, Φ_halluc ≤ 0.001, Q_density ≥ 0.65 — thresholds
  byte-identical to the R10 run, untouched)
- Standing figures are re-read from the on-disk receipts, not recalled:
  `DOC-HDIT-R29-RECEIPT.md` (certify) and the "Standing row — lane R69
  closeout" section of `DOC-HDIT-CERTIFY-RECEIPT.md` (re-witness).

## Extractor pin (3d2abae1-era, recomputed)

- Extractor: `/Users/sac/ggen-marketplace/scripts/gen_doc_surface.py`
- sha256 recomputed on-disk 2026-10-09:
  `3d2abae19dac9f529b8250a0f96a02b34dbf86348dad241fbdb003410dc35590`
  (the pin bound at ggen-marketplace `0f3d840ff`, R64 module-level
  denominator; unchanged at ggen-marketplace HEAD `e9883e47b`)
- Note: the R29 certify ran on the rotated `b88297e6…` pin; the
  `3d2abae1…` pin is the current extractor era. Re-running the pinned
  pipeline at this pin is the replay path (R29 receipt, "Falsifier /
  replay").

## Tag

- `v26.10.8` tag: **minted** at `349afd7b` (annotated tag object
  `e440420da9dc6d1c6e72a3460196c979570ff781`, coordinator-minted, pushed
  to origin; zero-citation sweep basis). Resolves the R69 standing-row
  note ("tag withheld — minting remains an explicit work order").

## Threshold-falsifier disclosure

- `cargo test --workspace` contains load-sensitive timing assertions
  (`frontier_phase_planning_is_cheap_for_every_phase`, `ms < 30_000` on
  the 6-suite phase; also `full_horizon_does_not_reach_the_whole_release`
  in the R10 run). These fail under suite load and pass in isolation,
  disclosed per campaign precedent — this is a disclosed test-gate
  limitation, not a standing qualifier on the doc-hdit court verdict.

## Supersession

- The R10 REFUSED receipt carries an R34 superseded-by note
  (`DOC-HDIT-CERTIFY-RECEIPT.md`, line 12): "superseded-by
  f51d81ac4f7e4119dff950327237effee4968f4aa9aba52c7d62c5362f441fa9
  as-of 2026-10-09 (R34); pin retained as historical subject identity —
  REFUSED receipt, no live-standing claim." That supersession is
  confirmed here: the live standing for v26.10.8 is this CERTIFIED
  receipt, not the R10 REFUSED row.

## Replay

Re-run `rollout.sh --report-only` at `349afd7b` with the pinned extractor;
if certify REFUSES (or S_coverage < 0.90 with fresh extraction), this
standing is refuted and the inventories must be regenerated per the R29
receipt recipe.
