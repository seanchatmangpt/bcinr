# DOC-HDIT certify receipt — bcinr v26.10.8 (lane R10)

Status: **REFUSED:DOC_HDIT_CERTIFY** — S_coverage gate failed; no doc-hdit receipt minted; **no v26.10.8 tag minted**.
Thresholds were NOT relaxed. This is the honest certification record for the lane.

## Subject

- Repo: `/Users/sac/bcinr`, branch `bench/rdtsc-tick-tables` @ `01669d20e` (HEAD at both certify runs; the prose repairs below were uncommitted working-tree state at run time)
- Binary: `/Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/target/release/doc-hdit`
- Extractor pin: `ggen-marketplace/scripts/gen_doc_surface.py` sha256 `4c862576ab63595f9cd0417b35341af3ec1001f49450e79bf2e4c291a4a4246f` (verified with `shasum -a 256` before the runs)

## Commands + exits

```sh
# run 1 (pre-fix baseline; clean tree at 01669d20)
DOC_HDIT_BIN=<doc-hdit> rollout.sh --out /tmp/hdit/bcinr-r10  /Users/sac/bcinr    # exit 1
# run 2 (after prose grounding repairs + .doc-surface.toml; fresh extraction,
# stale /tmp/hdit/bcinr.*.inputs.json cache deleted first)
DOC_HDIT_BIN=<doc-hdit> rollout.sh --out /tmp/hdit/bcinr-r10c /Users/sac/bcinr    # exit 1
```

Both runs via `packs/rust-doc-hdit-pack/scripts/rollout.sh` (extract -> vectorize -> certify
against `courts/doc_quality.court`). A first post-fix re-run reused the HEAD-keyed inputs cache
and is discarded; run 2 is the recorded result.

## Metrics (run 2, pinned extractor)

| gate | value | threshold | verdict |
|---|---|---|---|
| S_coverage (set, gated, public surface) | 0.3480 | >= 0.90 | **REFUSED** |
| Phi_halluc | 0.0 | <= 0.001 | PASS |
| Q_density | 1.0 | >= 0.65 | PASS |
| S_coverage_raw (report-only) | 0.3488 | — | — |

## Repairs landed this lane (real grounding fixes; thresholds untouched)

1. `docs/thesis/bcinr-pddl-lsp.md` — 10 phantom `has_param` claims. The PDDL8 domain table
   backticked ten action names (`create_prd` .. `publish_release`) that exist nowhere in the
   code surface (design-target names for the unbuilt bcinr-pddl-lsp server, removed from the
   workspace per root `Cargo.toml`). The extractor mints `has_param` claims from table-cell
   code spans; the names are now unbackticked (design names, not symbol references).
2. `docs/diataxis/reference/api-catalog.md` — 2 phantom `mentions` claims. Slash-shorthand
   `saturating_add/sub/mul_i64(a, b)` and `select_u32/u64(mask, a, b)` defeated exact-match
   grounding; expanded to the real identifiers (`saturating_add(a, b)`, `saturating_sub(a, b)`,
   `saturating_mul(a, b)`, `select_u32(mask, a, b)`, `select_u64(mask, a, b)`) — all five
   verified present in `crates/bcinr-cmca/src/fixed.rs` / `crates/bcinr-logic`.
3. `.doc-surface.toml` (new) — declares
   `crates/bcinr-cmca/src/generated/consequence_mass/{generalization,case_studies}.rs` as
   `[[generated]]` (generator.py output, "DO NOT EDIT" headers) per the frozen-duckdb
   generated-surface mechanism (ggen-marketplace backlog [107]/[64]). 148 machine-generated
   items leave the public denominator (2855 -> 2658). `generated/stability_profile.rs` is
   deliberately NOT listed (hand-written per its own header, CMCA-105).

Result: all 12 phantom claims eliminated — Phi_halluc 0.0042 -> 0.0, Q_density 0.9958 -> 1.0.
S_coverage moved 0.3412 -> 0.3480 only.

## Why certification is refused and no tag minted

S_coverage 0.3480 = 925 covered / 2658 public items — **1733 public items carry no doc
claims**. This is an honest documentation gap, the same class the doc-hdit pilot itself
recorded for ferroplan (honest FAIL, "a documentation defect, not a metric artifact" —
`ggen-marketplace/docs/sjira/v26.10.8/DOC-HDIT-PILOT.md`).

Close it (post-landing work order, not a threshold change): document the uncovered public
items — concentrated in `crates/bcinr-logic` (~1026 items), `crates/bcinr-pddl` (~738),
`crates/bcinr-powl` (~447) — then re-run the pinned pipeline and re-certify.

## Test evidence

- `cargo test --workspace` (clean serial run on the same working tree): **2703 passed, 0
  failed**, cargo exit 0. A first concurrent run showed one load-induced timeout of
  `full_horizon_does_not_reach_the_whole_release` (bcinr-pddl, 209s wall under the
  simultaneously running extraction pipeline); the test PASSED in isolation (110s).
