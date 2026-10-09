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

---

## Addendum — v26.9.15 optimization wave (same day, same machine, quiet-load)

Four parallel lanes (profiler, fixed-arithmetic, kernel, surface) under one
hard constraint: **bit-identical observable behavior**. Verdict per lane:

- **Kernel (accepted)**: hoisted weight normalization out of the 8
  `flow_step` calls (the divisions' operands are invariant across them;
  LLVM cannot CSE across the `#[inline(never)]` boundary); hoisted
  `compute_kappa`'s subtree mass sums to one fold per q; deduplicated the
  ancestor-doubling table in `allocate_single_lens`. Bit-identity: a
  20,000-input cross-build sweep (random forests, refusals, corrupt
  digests, proof Some/None, post-call state) hashes identical
  (digest `5c88039db6a307d7`); object code re-audited: 0 conditional
  branches, 0 divides, 0 backedges in all changed kernel symbols.
- **Surface (accepted)**: escort normalization now uses an exact u64 floor
  division proven equal to `saturating_div` over 44.2M adversarial pairs,
  pinned by a permanent guard test; cascade's negative-lens reciprocal
  likewise. CLI output byte-identical over the 22-case tamper matrix.
- **Fixed-arithmetic (truthful NO-CHANGE)**: the suspected i128 cost does
  not exist on arm64 (LLVM lowers inline), and the u64 reformulation is
  provably NOT bit-identical (exhaustive 2^31 NR-domain sweep: 70.8%
  bit-diffs; the signed sign-extension is load-bearing). No change.
- **Profiler**: stage-decomposition benchmarks added to this suite; also
  established that the original 118.6 µs baseline was measured under
  machine contention (the quiet-machine pre-change kernel was ~76 µs), so
  the honest delta is the kernel lane's back-to-back same-window A/B
  (119.5 → 73.7 µs) plus this table's same-instrument quiet rerun.

### After (official Divan instrument, quiet machine)

| Benchmark | Baseline | After | Δ |
|---|---:|---:|---|
| `allocate_kernel` | 118.6 µs* | **73.3 µs** | **−38% (1.62×)** |
| `allocate_single_lens_query` | 10.62 µs | **6.29 µs** | **−41% (1.69×)** |
| `escort_exact_integer_q` | 140 ns | **79 ns** | **−44%** |
| `escort_fractional_q` | 159 ns | **104 ns** | **−35%** |
| `power_fractional` | 9.2 ns | 9.2 ns | unchanged (no-change lane) |

\* contended-load measurement; see calibration note above.

Capacity: ~8,400 → **~13,600 allocation decisions/second/core**. Remaining
known headroom (named, not taken): reciprocal-sharing across the normalize
divisions requires a new fixed.rs API (cross-lane handoff, needs its own
bit-identity proof); the ~11% select-discarded learning machinery is the
branchless contract's price and is not removable without behavior change.
