# Benchmark Methodology — how to reproduce the baseline

Companion to `BASELINE.md` (same directory). Written by Lane 1, benchmark-baseline lane,
2026-10-01 wave. Reproduce before comparing any two numbers.

## 1. Hardware and environment (observed this session)

```
sysctl -n machdep.cpu.brand_string   ->  Apple M3 Max
sysctl -n hw.memsize                 ->  51539607552        (48 GiB)
sysctl -n hw.ncpu                    ->  16
rustc --version                      ->  rustc 1.99.0-nightly (daf2e5e18 2026-07-13)
```

Toolchain is pinned by `rust-toolchain.toml` (channel `nightly`, profile minimal,
rustfmt + clippy). `encode_unicode_patch` benches additionally require nightly
`libtest` (`#![feature(test)]`) — they do not build on stable.

## 2. Subject identity

All numbers in `BASELINE.md` were produced on:

- branch `release/26.9.15`, HEAD `ee2c907b28bdb2c6f176a74fb32c500eea4213bc`
- working tree shared with concurrent lanes (see `BASELINE.md` §12). A baseline number
  is only comparable to a rerun on the same HEAD with the same bench sources; bench
  files landed mid-session, so pin both the commit and the bench file mtimes when in doubt.

## 3. Exact commands (per suite)

Run from the workspace root (`~/bcinr`). Each command was executed with exit 0 on
2026-10-01 unless noted.

```bash
# bcinr-logic kernels (suite landed 2026-10-01; requires the bench feature)
cargo bench -p bcinr-logic --features bench

# bcinr-powl (all four divan suites)
cargo bench -p bcinr-powl

# bcinr-pddl (three divan suites, default features)
cargo bench -p bcinr-pddl
# 80/20 fixture set with the production planner wired in (adds composed/ingress/numeric rows)
cargo bench -p bcinr-pddl --features mfw-planner --bench pddl_80_20

# bcinr-cmca (allocation kernel; default features exclude the escort module)
cargo bench -p bcinr-cmca
# full suite: adds escort_* benches and cascade_bench (required-features = alloc)
cargo bench -p bcinr-cmca --features alloc
# single suite:
cargo bench -p bcinr-cmca --features std cascade_bench

# bcinr-guarded / bcinr-mfw-ir
cargo bench -p bcinr-guarded
cargo bench -p bcinr-mfw-ir

# encode_unicode_patch (vendored upstream; note the package name is `encode_unicode`)
cargo bench -p encode_unicode

# coverage auditor (must run with cwd = workspace root; relative paths inside)
cargo run -p bcinr-bench-auditor
```

Single divan suites can be selected with `--bench <name>`, e.g.
`cargo bench -p bcinr-powl --bench receipt_bench`.

## 4. Harness mechanics

- **divan** (`harness = false` targets): default `sample_count = 100` (quick_bench pins
  10), auto iteration scaling, output columns fastest / slowest / median / mean /
  samples / iters. `BASELINE.md` records **mean** as the headline and fastest where shown.
  Timer precision observed: **41 ns** — divan amortizes over up to 1.6M iterations, so
  sub-ns per-iter means are mathematically possible, but see caveat C1.
- **libtest bench** (`encode_unicode_patch`): 30/11 benches print `ns/iter (+/- error)`;
  linear-regression warmup, ~3 s per bench by default. `length` reads text fixtures from
  `crates/encode_unicode_patch/benches/texts/` (present); `multiiterators` needs the
  system dictionary `/usr/share/dict/american-english` (absent on this host — skipped,
  exit 0).

## 5. Measurement caveats

C1. **Sub-nanosecond rows are instruction-throughput figures, not latencies.** Rows with
mean < ~1 ns (logic kernels, cmca decomposition primitives, guarded classify, mfw-ir
event_set primitives) execute in at most a handful of cycles. They are valid for
regression tracking of the same row over time; they are not callable latencies and must
not be quoted as such without an object-code audit (AGENTS.md §20). If a sub-ns row
moves by >2x, suspect an optimizer change (inlining/const-fold) before celebrating.

C2. **Fixture realism bounds.** cmca `allocate_kernel` benches the fixed generated
case-studies profile (N=8, K=4, Q=4, flat parent, all-ONE weights) — the real call
contract, but one point in the input space. pddl `scaling` uses synthetic
deploy-services fixtures with linear chains. Deltas transfer to production only for
workloads of the same shape.

C3. **`bench_powl` N>64 rows measure a typed refusal** (DEF-3 in `BASELINE.md`): the
3300x drop from N=64 to N=128 is the `MAX_POWL_TAPE_STEPS` early exit, not an
optimization. Exclude those rows from scaling analysis.

C4. **Concurrent build pressure.** This baseline was taken while sibling lanes shared
`target/` (cargo's build lock serializes; one run logged
`Blocking waiting for file lock on build directory`). Per-bench divan means are
measured inside the bench process and were stable across the cmca default/alloc
rerun (~7% on `allocate_kernel` is the observed rerun spread), but suite wall times
include queueing. For publishable numbers: quiet machine, no other cargo processes,
`nice -n -5` not required but no competing load.

C5. **Bench profile is not release profile.** `[profile.bench]` overrides fat LTO
(`lto = false`, `codegen-units = 16`, opt-level 3) — deliberate, per the comment in the
root `Cargo.toml` (fat-LTO bench builds spiked `target/` to 24–30 GiB). Numbers are
therefore not directly comparable to release-profile object-code audit results.

C6. **First-call outliers.** divan `slowest` columns and the cmca/pddl first iterations
include cold-cache effects (e.g. `scheduler_tick_complex_dependencies` slowest 40.91 µs
vs mean 2.341 µs). Use mean for tracking; use slowest only for tail analysis.

C7. **Feature gates change the suite, silently by design.** cmca's escort benches and
cascade_bench require `alloc`/`std`; a default-features run is not "the full suite".
pddl's `mfw-planner` feature changes which `pddl_80_20` rows exist. Always record the
feature set next to the numbers.

C8. **History note.** `docs/BENCHMARKS.md` (root `docs/`, v26.6.24) records a criterion-era
baseline with different primitives (`const_tick` 436 ps, wide-tick scaling) that have no
current bench target; treat those rows as historical, not as this baseline.

## 6. Reproduction checklist

1. `git -C ~/bcinr rev-parse HEAD` == `ee2c907b28bdb2c6f176a74fb32c500eea4213bc` (or diff the bench sources).
2. No other cargo/rustc processes running (`pgrep -fl "cargo|rustc"`).
3. Run the per-suite commands from §3; capture stdout to files.
4. Compare mean columns against `BASELINE.md`; investigate deltas >10% (C4 spread) or any
   row that changed feature-gating.
5. Re-run the auditor: `cargo run -p bcinr-bench-auditor` — currently exit 1 with 337
   uncovered (DEF-1); a rerun that goes green means the gap closed.
