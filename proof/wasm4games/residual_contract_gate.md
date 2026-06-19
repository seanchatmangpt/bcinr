# wasm4games — Law-State Receipt

> This is a process-evidence receipt, not a marketing claim. Status is fenced.
> Do not launder pre-existing residuals into a crown pass.

```text
wasm4games            = VERIFIED_UNDER_SCOPE
bcinr workspace crown = BLOCKED_BY_PRE_EXISTING_CONTRACT_GATE_RESIDUAL
GATE-W4G-001          = UNVERIFIED (ggen not available in this environment)
```

- **Date:** 2026-06-19
- **Branch:** `claude/determined-hypatia-t8ajak`
- **Toolchain:** `nightly` (1.98.0-nightly, bc2112ed5) per `rust-toolchain.toml`
- **Feature commit:** `89b709c` — wasm4games kernels/evidence/ggen/bridge/docs
- **Repair commit (separate):** `3faa9d5` — pre-existing fmt + unused-var fixes surfaced by the nightly bump (NOT part of the feature; kept separate by intent)

## Scope boundary

`crates/wasm4games` is an offline-pure `no_std` crate built on `bcinr-logic`. The CI
tool-gates (`bcinr-cheat-scanner`, `bcinr-contract-gate`, `bcinr-bench-auditor`) only scan
`crates/bcinr-logic/src/algorithms`, so wasm4games is **exempt by construction** and is not
the cause of any gate failure below. The wasm4pm admission bridge
(`crates/wasm4games-wasm4pm`) is `[workspace] exclude`-d and references the external
`wasm4pm` / `wasm4pm-compat` repos rather than vendoring them.

## Verification battery (real exit codes)

| Check | Command | Result |
|---|---|---|
| crate build | `cargo build -p wasm4games` | `EXIT=0` ✅ |
| unit + integration tests | `cargo test -p wasm4games --features std` | **109 passed** (`EXIT=0`) ✅ |
| doc tests (std/alloc) | `cargo test -p wasm4games --features std` (doc) | **3 passed** ✅ |
| doc tests (default features) | `cargo test -p wasm4games --doc` | **2 passed** (`EXIT=0`) ✅ — the `alloc`-gated `OcelLog::to_json` doctest is inert without `alloc` |
| clippy (deny warnings) | `cargo clippy -p wasm4games --all-targets --features std,bench -- -D warnings` | `EXIT=0` ✅ |
| rustfmt | `cargo fmt -p wasm4games -- --check` | `EXIT=0` ✅ |
| workspace build | `cargo build --workspace --all-features` | `EXIT=0` ✅ |
| cheat-scanner | `cargo run -p bcinr-cheat-scanner --release -q` | `EXIT=0` — "OK: no cheat patterns detected across 308 algorithm files." ✅ |
| contract-gate | `cargo run -p bcinr-contract-gate --release -q` | `EXIT=1` — **17 pre-existing FAILs** ⛔ RESIDUAL |

`wasm4games` is therefore **VERIFIED_UNDER_SCOPE**: everything it owns is green. The
workspace crown is **BLOCKED** solely by the pre-existing contract-gate residual recorded
below.

## RESIDUAL: contract-gate failures (pre-existing, not wasm4games)

`bcinr-contract-gate` exits non-zero on **17** `bcinr-logic/algorithms` files whose kernels
have cyclomatic complexity ≥ 2 ("Branch detected!"). None were created or modified by the
wasm4games work; the three files this branch *did* touch
(`bit_permute_identity_64.rs`, `blsi_u64.rs`, `blsr_u64.rs` — test-oracle param renames only)
are **not** in this list.

```text
FAIL: merge_u32_slices_branchless            CC=3  crates/bcinr-logic/src/algorithms/merge_u32_slices_branchless.rs
FAIL: parallel_bits_deposit_u64              CC=2  crates/bcinr-logic/src/algorithms/parallel_bits_deposit_u64.rs
FAIL: norm_u32                               CC=2  crates/bcinr-logic/src/algorithms/norm_u32.rs
FAIL: fp_sqrt_u32_q16                        CC=2  crates/bcinr-logic/src/algorithms/fp_sqrt_u32_q16.rs
FAIL: lcm_u64_branchless                     CC=2  crates/bcinr-logic/src/algorithms/lcm_u64_branchless.rs
FAIL: normalize_slice_branchless             CC=2  crates/bcinr-logic/src/algorithms/normalize_slice_branchless.rs
FAIL: gcd_u64_branchless                     CC=2  crates/bcinr-logic/src/algorithms/gcd_u64_branchless.rs
FAIL: halton_sequence_u32                    CC=2  crates/bcinr-logic/src/algorithms/halton_sequence_u32.rs
FAIL: nth_element_branchless                 CC=4  crates/bcinr-logic/src/algorithms/nth_element_branchless.rs
FAIL: parallel_bits_extract_u64             CC=2  crates/bcinr-logic/src/algorithms/parallel_bits_extract_u64.rs
FAIL: scatter_bits_u64                       CC=2  crates/bcinr-logic/src/algorithms/scatter_bits_u64.rs
FAIL: locality_sensitive_hash_cosine         CC=2  crates/bcinr-logic/src/algorithms/locality_sensitive_hash_cosine.rs
FAIL: count_consecutive_set_bits_u64         CC=2  crates/bcinr-logic/src/algorithms/count_consecutive_set_bits_u64.rs
FAIL: is_permutation_branchless              CC=4  crates/bcinr-logic/src/algorithms/is_permutation_branchless.rs
FAIL: linear_search_simd_u8                  CC=2  crates/bcinr-logic/src/algorithms/linear_search_simd_u8.rs
FAIL: rank_u128                              CC=2  crates/bcinr-logic/src/algorithms/rank_u128.rs
FAIL: gather_bits_u64                        CC=2  crates/bcinr-logic/src/algorithms/gather_bits_u64.rs
```

**Disposition:** RESIDUAL, preserved — not silently erased, not laundered into the feature.
Clearing these is a separate residual-clear checkpoint (each kernel must be made genuinely
branchless or its complexity gate explicitly justified). Do **not** mix that work into the
wasm4games feature.

## GATE-W4G-001 — ggen byte-stable reproduction (UNVERIFIED)

The doctrine "generated files are reproducible ggen outputs, not hand-authored claims" is
only *real* once this gate passes. It **cannot** be verified in this environment because
`ggen` is not installed (`command -v ggen` → absent).

Verifier (run where `ggen` from <https://github.com/seanchatmangpt/ggen> is installed):

```bash
cd crates/wasm4games/ggen
ggen sync
cargo fmt -p wasm4games
git diff --exit-code            # must be empty: committed kernels == ggen output
cargo test -p wasm4games --features std
```

Until that runs green, the committed `src/patterns/*.rs` are **asserted** to match the
`ggen/` surface but **not yet receipted** as byte-stable. Status: `UNVERIFIED`.

## Honest summary

- wasm4games: every owned check is green → `VERIFIED_UNDER_SCOPE`.
- workspace crown: `BLOCKED_BY_PRE_EXISTING_CONTRACT_GATE_RESIDUAL` (17 files, listed above).
- ggen reproduction gate: `UNVERIFIED` (ggen absent here).
- No residual was laundered into a crown pass.
