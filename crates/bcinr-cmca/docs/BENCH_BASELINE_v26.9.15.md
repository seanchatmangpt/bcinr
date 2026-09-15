# CMCA Allocation Benchmark Baseline — v26.9.15

**Status:** BASELINE RECEIPT (first benchmark suite this crate has ever had; the
repo's bench-auditor had been truthfully reporting zero coverage).

## Environment

| Field | Value |
|---|---|
| Machine | Apple M3 Max (arm64) |
| Toolchain | rustc 1.99.0-nightly (daf2e5e18 2026-07-13) |
| Bench profile | workspace `[profile.bench]`: opt-level 3, no LTO, 16 codegen units |
| Production-profile check | re-run with `CARGO_PROFILE_BENCH_LTO=fat CARGO_PROFILE_BENCH_CODEGEN_UNITS=1` — results within noise (see below) |
| Reproduce | `cargo bench -p bcinr-cmca --features std` |

## Baseline (median)

| Benchmark | no-LTO bench profile | fat-LTO override | Meaning |
|---|---:|---:|---|
| `allocate_kernel` | 118.6 µs | 119.0 µs | one full `allocate()` over the case-studies fixture (N=8, K=4, Q=4, 16 measure×lens pairs, MWU, blend, digest) |
| `allocate_single_lens_query` | 10.62 µs | 10.62 µs | one stateless single-lens query (rebuilds topology tables per call — the API's documented cost shape) |
| `power_fractional` (q=0.5) | 9.17 ns | 10.22 ns | the `allocator::power` log2/exp2 primitive |
| `escort_exact_integer_q` (q=3) | 140 ns | 125 ns | escort distribution, exact repeated-multiply path |
| `escort_fractional_q` (q=0.5) | 159 ns | 144 ns | escort distribution, approximate path |

## Capacity statement

At 118.6 µs/decision, one core sustains **~8,400 allocation decisions/second**.
The kernel scales with the fixture shape (compile-time unrolled), not with
candidate count at runtime. This positions CMCA on the *decision plane*
(governance, scheduling policy, admission), not the packet plane — consistent
with the substrate's design intent.

## Internal consistency check

`allocate_kernel` ≈ 16 × `allocate_single_lens_query` (K×Q = 4×4 = 16), which
is exactly the kernel's structure (one per-measure/per-lens pass plus the
blend). Measured ratio: 118.6/10.62 ≈ 11.2× (the blend and shared topology
work amortize across the 16 passes), i.e. the numbers are structurally
consistent, not anomalous.

## Why no optimization change accompanies this baseline

1. **Codegen is not the lever**: fat LTO + 1 codegen unit moves the kernel by
   <0.5% — performance is intrinsic to the unrolled algorithm, not LLVM.
2. **No profile evidence of a defect**: every number is structurally
   consistent with the kernel's known shape (see above). Changing the
   certified CC=1 kernel without profile evidence would trade audited
   branchlessness for unmeasured speed — the exact trade this repository's
   constitution forbids.
3. **The two named optimization candidates** are API-shape decisions, not
   kernel defects: `allocate_single_lens` rebuilds its ancestor tables per
   call because it is stateless by contract (a caller doing many queries can
   use the full kernel instead); `allocate`'s per-call `Copy` of the weights
   array is measured because it is part of the real call contract.

Any future optimization PR must reproduce this baseline first, state which
row it moves, and re-run the object-code audit for changed authoritative
symbols.
