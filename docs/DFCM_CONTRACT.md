# DfCM Benchmark Contract

```text
The entire DfCM suite must complete in ≤ 5.000s wall-clock.
```

This proves the suite is actually respecting the 8/64 constraint, not
accidentally benchmarking an unbounded search.

## Hard rules

1. No full-workspace bench.
2. No unbounded input generation.
3. No external path deps.
4. No network.
5. No hidden exhaustive search beyond 64 ops.
6. No benchmark case may exceed 64 actions / 64 tape ops.
7. Total suite wall clock ≤ 5s.

## Suite shape

One crown bench file: `bcinr-bench/benches/dfcm_crown_bench.rs`, plus a
shared, reusable suite implementation in `bcinr-pddl::dfcm_crown` so the same
logic backs both the bench and the wall-clock gate test
(`crates/bcinr-pddl/tests/dfcm_crown_suite.rs`).

Fixed bounded matrix:

```text
ops:        8, 16, 32, 64    (worker count == durative-action ground instances)
capacity:   1, 2, 4, 8       (available-workers resource cap)
```

16 cells, each deterministic and tiny: a generated `assign-worker` capacity
domain (the same pattern used in `tests/capacity.rs`) scaled to N workers,
run once per cell through the full pipeline.

## Headline metrics (`DfcmBenchReceipt`)

```rust
pub struct DfcmBenchReceipt {
    pub wall_clock_ms: u128,
    pub topology_ns: u128,
    pub planning_ns: u128,
    pub analysis_ns: u128,
    pub admission_ns: u128,
    pub receipt_ns: u128,
    pub replay_ns: u128,
    pub max_ops: u8,
    pub max_parallelism: u8,
    pub suite_passed_5s_gate: bool,
}
```

Measurement honesty note: `admission_ns` times `execute_temporal_plan` as a
whole (admission-gate checking dominates its cost); `receipt_ns` times an
isolated `compute_plan_chain` call (the chain-hash computation in isolation,
not a true sub-component split of `execute_temporal_plan`). This is not a
perfect decomposition of one function into two — it is two real,
independently-measured operations, documented as such rather than presented
as a precise breakdown of a single call.

## What it proves

Not "this is the fastest planner." It proves:

```text
bounded topology
+ bounded scheduling
+ bounded analysis
+ bounded admission
+ bounded receipts
+ bounded replay

all compose within one fixed wall-clock envelope
```

That composition, gated at 5s wall-clock for the full 16-cell matrix, is the
CM (combinatorial-maximalist) proof: the substrate's bound is real, not
incidental.
