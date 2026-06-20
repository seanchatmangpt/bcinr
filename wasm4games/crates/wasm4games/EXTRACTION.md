# Extracting wasm4games into its own repository

> A concrete guide to lifting the `wasm4games` foundry out of the `bcinr` monorepo into a
> standalone repository. It states exactly which crates move, the **one** external
> dependency (`bcinr-logic`) and the precise module subset it consumes, the options for
> carrying that dependency, the CI commands, and the licensing — and it is honest about
> what is *not* done yet (vendoring is a follow-up).

## Goal

Produce a self-contained `wasm4games` repository that builds, tests, and lints offline
exactly as it does inside `bcinr` today, with no dependency back on the rest of the `bcinr`
monorepo except the published/vendored `bcinr-logic` primitives.

## What moves

These are the units to lift. The first three are workspace members today; the bridge is
already workspace-*excluded*.

| Unit | Path in `bcinr` today | Role | Workspace status today |
|---|---|---|---|
| `wasm4games` | `crates/wasm4games/` | the foundry crate (kernels, IR, evidence, verify, compat, corpus) | member |
| `wasm4games-capi` | `crates/wasm4games-capi/` | C-ABI `staticlib` portability spine (`w4g_kernel`, `w4g_corpus_digest`, …) | member |
| ggen surface | `crates/wasm4games/ggen/` | the ontology + SPARQL + Tera input that regenerates the kernels | sub-dir of the crate |
| `wasm4games-wasm4pm` | `crates/wasm4games-wasm4pm/` | online-only admission bridge to the external wasm4pm repos | **excluded** |

Notes:

- The **ggen surface travels with the crate** — it already lives under
  `crates/wasm4games/ggen/` (`ggen.toml`, `schema/patterns.ttl`, `queries/*.rq`,
  `templates/*.tera`). Keep it co-located so `ggen sync` continues to write the committed
  `src/patterns/*.rs`. (The `[generation] base_dir = ".."` in `ggen.toml` is relative to the
  crate root and needs no change.)
- The **`wasm4games-wasm4pm` bridge is already excluded** from the workspace and depends on
  the external wasm4pm / wasm4pm-compat git repos. Move its directory too, but keep it
  *excluded* (or in a sibling, online-only workspace) so the new repo's offline CI never
  resolves git dependencies. It is *referenced, not vendored* — that property must survive
  the move.
- Do **not** move any other `bcinr` crate. `wasm4games` does not depend on `bcinr`,
  `bcinr-core`, `crates/bcinr-api`, the tools, or the benches.

## The one external dependency: `bcinr-logic`

`wasm4games` (and therefore `wasm4games-capi`, which only re-exports it) has exactly **one**
non-dev dependency: `bcinr-logic`. Everything else is testing-only (`proptest`) or
optional (`criterion`, behind `bench`).

It consumes a **small, fixed subset** of `bcinr-logic` — verified by grepping `bcinr_logic::`
across `crates/wasm4games/src`:

| `bcinr-logic` module | What wasm4games uses it for (representative items) |
|---|---|
| `mask` | branchless select / comparisons: `select_u*`, `lt_mask_u*`, `eq_mask_u*`, `min_u*`, `max_u*` |
| `int` | saturating arithmetic + popcount: `saturating_add_i*`, `saturating_sub_i*`, `popcount_u*` |
| `fix` | clamp / bucketize fixed-point: `clamp_u*`, `bucketize_u*` |
| `bitset` | bit ops + rank: `set_bit_u*`, `clear_bit_u*`, `select_bit_u*`, `rank_u*` |
| `dfa` | deterministic transitions: `dfa_advance` |
| `network` | sorting networks: `bitonic_sort_*` |
| `patterns::integrity_receipt` | the FNV-1a rolling receipt (`DeterministicSubstrateReceipt`) under the evidence/corpus layer |

That is the entire surface. `bcinr-logic` also re-exports the `Branchless` marker trait,
which `wasm4games` re-exports in turn (`pub use bcinr_logic::Branchless;`).

## Option A (recommended first cut): keep `bcinr-logic` as a dependency

The lowest-risk extraction keeps `bcinr-logic` as an external crate and just repoints the
dependency. Two sub-options:

- **Published dep.** If `bcinr-logic` is (or will be) published to crates.io, depend on it
  by version:

  ```toml
  # wasm4games/Cargo.toml
  [dependencies]
  bcinr-logic = "26.6.13"   # or the version you publish
  ```

  This is the cleanest long-term shape: the new repo has zero path dependencies and builds
  from the registry. Requires `bcinr-logic` to be release-ready (its own
  MIT-OR-Apache-2.0, semver, docs).

- **Path / git dep (interim).** Until `bcinr-logic` is published, point at a local checkout
  or a pinned git rev:

  ```toml
  bcinr-logic = { path = "../bcinr-logic" }                       # sibling checkout
  # or
  bcinr-logic = { git = "https://github.com/<org>/bcinr", rev = "<PINNED_REV>" }
  ```

  A pinned git rev keeps the build reproducible; a moving branch does not. Prefer a path dep
  only for local development.

Today's manifests already use `bcinr-logic = { path = "../bcinr-logic", version = "26.6.13" }`,
so Option A is a one-line edit per manifest.

## Option B (follow-up): vendor the `bcinr-logic` subset

To make the repo fully self-contained with no external `bcinr-logic` at all, vendor only the
seven modules above into the new repo (e.g. a `crates/bcinr-logic-min/` or an in-tree
`vendor/bcinr_logic/`), carrying their `#![no_std]` / `#![forbid(unsafe_code)]` posture and
their MIT-OR-Apache-2.0 license headers, then depend on the vendored crate by path.

**This is a follow-up, not done.** It is listed here so the extraction plan is complete, but
it has real cost and should not block the first cut:

- the subset must be carved with its transitive internal deps inside `bcinr-logic` (the
  seven public modules may pull on shared internal helpers);
- vendored code must keep both license files and attribution;
- a drift policy is needed (how vendored copies are refreshed from upstream `bcinr-logic`);
- the corpus digest must be re-verified to be byte-identical after vendoring (it folds
  kernel outputs that flow through these primitives).

Recommendation: ship Option A first (path/published dep), prove the repo green, then do
Option B as a deliberate, separately-reviewed change.

## New workspace layout (sketch)

```text
wasm4games/                     # new repo root
├── Cargo.toml                  # [workspace] members = ["wasm4games", "wasm4games-capi"]
│                               #             exclude  = ["wasm4games-wasm4pm"]
├── LICENSE-MIT
├── LICENSE-APACHE
├── wasm4games/                 # the foundry crate (was crates/wasm4games/)
│   ├── Cargo.toml              # dep: bcinr-logic (Option A) — see above
│   ├── src/…                   # class, ir, evidence, verify, compat, corpus, patterns/*
│   └── ggen/…                  # ontology + SPARQL + Tera (travels with the crate)
├── wasm4games-capi/            # C-ABI staticlib (was crates/wasm4games-capi/)
└── wasm4games-wasm4pm/         # EXCLUDED, online-only bridge (was crates/wasm4games-wasm4pm/)
```

The excluded bridge can also live in a sibling, online-only workspace if you prefer to keep
the offline root pristine; the only hard rule is that the offline build must never see it.

## CI commands

The new repo's offline CI mirrors today's checks (these are the commands that pass in
`bcinr` now):

```bash
# Build the offline-pure crates (default = no_std, zero-alloc)
cargo build -p wasm4games
cargo build -p wasm4games-capi

# Test with the full feature set (std pulls in alloc + the proptest/breed battery)
cargo test -p wasm4games --features std

# Lint — warnings are errors (include the bench feature so the dormant bench compiles)
cargo clippy -p wasm4games --all-targets --features std,bench -- -D warnings
cargo fmt --check

# C-ABI portability receipt: the C harness must reproduce the golden digest
./wasm4games-capi/portability_proof.sh
```

Two checks worth promoting to gates in the new repo:

- **`GATE-W4G-001` — ggen byte-stable regeneration.** Run `ggen sync` then
  `git diff --exit-code` over `src/patterns/` to *receipt* that the committed kernels
  regenerate byte-for-byte. **Status: UNVERIFIED** — `ggen` is not installed in the current
  environment, so this is asserted, not yet receipted. Wiring it is part of the extraction
  follow-up.
- **Corpus digest pin.** `cargo test -p wasm4games` already asserts
  `corpus_digest() == GOLDEN_CORPUS_DIGEST` (pinned in `src/corpus.rs`); keep that test, and
  keep the C-ABI side (`w4g_corpus_digest()` vs `w4g_golden_corpus_digest()`) in CI as the
  cross-language receipt.

Do **not** wire ggen into the build itself: kernels stay committed precisely so the offline
build needs nothing but `rustc`. ggen remains a developer-time tool (the GGEN-Only covenant).

## What the new repo must preserve (don't regress the fences)

- **Offline purity.** No git/network dependency in the offline workspace. The only runtime
  dep is `bcinr-logic`; `proptest`/`criterion` stay dev/optional.
- **The GGEN-Only covenant.** `src/patterns/*` is generated output; the ggen surface is the
  hand-authored law. Carry both, and keep the generated banner.
- **Referenced, not vendored (wasm4pm).** Real admission stays external in the excluded
  bridge; the in-crate `compat` mirror + `verify` proxy stay dependency-free.
- **Law-state.** `wasm4games = VERIFIED_UNDER_SCOPE`; C-ABI portability is VERIFIED, wasm32
  is UNVERIFIED, engines/Lua are NOT_STARTED. These scopes do not change by moving the code.

## Licensing

`wasm4games` and `wasm4games-capi` are already **MIT OR Apache-2.0**, and so is
`bcinr-logic`. The new repo should:

- add top-level `LICENSE-MIT` and `LICENSE-APACHE` files;
- keep `license = "MIT OR Apache-2.0"` in every crate manifest;
- if Option B (vendoring) is taken, carry `bcinr-logic`'s license files and attribution
  alongside the vendored subset.

No license change is required by the extraction; the dual license is uniform across all
units that move.

## Summary

Lift `wasm4games`, `wasm4games-capi`, and the co-located `ggen/` surface into a new
two-member workspace (plus the still-excluded `wasm4games-wasm4pm` bridge). The sole
external dependency is `bcinr-logic`, consumed only through `mask`, `int`, `fix`, `bitset`,
`dfa`, `network`, and `patterns::integrity_receipt`. Ship it first with `bcinr-logic` as a
published/path dependency (Option A); vendoring that subset (Option B) is a deliberate
follow-up, not part of the first cut. Keep the offline CI, the GGEN-Only covenant, the
referenced-not-vendored wasm4pm boundary, and the dual MIT/Apache-2.0 license intact.
