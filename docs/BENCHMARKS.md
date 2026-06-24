# bcinr Benchmark Charter & Performance Baselines

Version: **v26.6.24** | Hardware: Apple M-series ARM64 (Firestorm, 3.2 GHz) | Harness: Criterion 0.5 (100 samples, 3 s warmup)

---

## POWL Workflow Runtime (bcinr-powl) — Measured v26.6.24

### Scheduler Comparison (N=4 linear chain, full run to done)

| Scheduler | Latency | Cycles | Notes |
|---|---|---|---|
| `const_tick` (Lever 4) | **436 ps** (measured) | 1.4 | Compile-time topology; N=4,8,16 identical |
| `legacy` SWAR | 16.7 ns (measured) | 53 | Branchless via kind_mask; +29% vs prior branchy impl on benign linear chain (no misprediction to avoid) |
| `wired_petri` | 524 ns (measured) | 1,675 | Full Petri reconstruction each tick |

### Hot-Path Primitives

| Operation | Latency | Cycles | Rate | Path |
|---|---|---|---|---|
| Conformance gate (Q16.16) | 395 ps | 1.3 | 2.53 B/s | Hot (per receipt) |
| `const_tick` N=4 chain | 436 ps (measured) | 1.4 | 2.29 B/s | Hot (compile-time) |
| Petri step (8 transitions) | 581 ps | 1.9 | 1.72 B/s | Warm |
| TimeWheel tick | 922 ps | 2.9 | 1.08 B/s | Hot (per tick) |
| Scheduler tick (1 op) | 2.07 ns | 6.6 | 483 M/s | Hot |
| `prefix_xor_u64x8` | 2.31 ns | 7.4 | 433 M/s | Warm |
| Fiber lifecycle | 2.64 ns | 8.4 | 379 M/s | Cold |
| OCEL emit | 3.23 ns | 10.3 | 310 M/s | Warm (per fire) |
| `union_u64_slices` | 4.97 ns | 15.9 | 201 M/s | Cold |
| MPMC push/pop | 35.4 ns | 113 | 28 M/s | Cold |
| Workflow (10 ops, E2E) | 69.6 ns | 223 | **14 M/s** | E2E |
| BLAKE3 receipt link | 336 ns | 1,075 | 2.98 M/s | Warm (per fire) |

### Wide Tape Scaling (Lever 1 — `wide_tick`, linear chain, total ticks to done)

| Ops (N) | Total time | ns/op |
|---|---|---|
| 4 | 277 ns | 69 ns |
| 16 | 1.42 µs | 89 ns |
| 64 | 8.1 µs | 127 ns |
| 256 | 139 µs | 542 ns |
| 512 | 619 µs | 1,209 ns |

Note: `wide_tick` uses a scalar eligible-set scan. The value of the wide tape is
512-op capacity (removing the 64-op limit), not raw throughput. For throughput on
small tapes use `const_tick`.

### const_tick Scaling (Lever 4 — compile-time topology, linear chain)

| N | Time (full run) | Time/op |
|---|---|---|
| 4 | 435 ps (measured) | 109 ps/op |
| 8 | 436 ps (measured) | 54 ps/op |
| 16 | 432 ps (measured) | 27 ps/op |

All three are within measurement noise — confirming full loop unrolling.

---

## Algorithm Family Targets (bcinr-logic)

| Algorithm Family | Primary Metric | Memory Profile | Complexity | Key Primitive |
|---|---|---|---|---|
| **Mask** | Cycles / Op | O(1) | O(1) | `select_u32` |
| **Int** | Throughput (GB/s) | O(1) | O(1) | `popcount_u64` |
| **Fix** | Cycles / Op | O(1) | O(1) | `add_sat_u8` |
| **Network** | Latency (ns) | O(N) | O(N log N) | `bitonic_sort_32` |
| **Bitset** | Throughput (GB/s) | O(N) | O(N) | `jaccard_u64` |
| **Scan** | Throughput (GB/s) | O(1) streaming | O(N) | `find_byte_mask` |
| **UTF-8** | Throughput (GB/s) | O(1) streaming | O(N) | `validate_utf8` |
| **Parse** | Cycles / Byte | O(1) | O(N) | `parse_decimal_u64` |
| **DFA** | Latency (ns) | O(1) | O(N) | `dfa_advance` |
| **Reduce** | Throughput (GB/s) | O(1) | O(N) | `horizontal_xor` |
| **Sketch** | Cycles / Byte | O(1) | O(N) | `murmur3_32` |
| **SIMD** | Throughput (GB/s) | O(1) | O(N/lane) | `splat_u8x16` |

## Performance Discipline

- **Branchless Mandate:** All kernels must be branchless in the hot path. Misprediction counts < 0.1% of total instructions.
- **Cache-Conscious:** Bulk operations (Scan, Bitset, Reduce) maintain sequential access patterns to saturate L1 bandwidth.
- **Determinism:** Identical output across architectures — required for receipt-based proof systems.
- **Zero-Allocation Steady State:** No heap allocation in scheduler hot path.

## Branch Misprediction Gate

Threshold: < 0.1% (10 basis points) branch misses of total instructions.
Measurement: `cargo make perf-branch-gate` (Linux only).
macOS alternative: Instruments > CPU Counters > Branch Mispredictions.
Benchmark: branchless_gate/linear_chain_32_tick_10k (10,000 scheduler_tick iterations).

## Benchmark Commands

```bash
# Full suite
make bench

# New 1000x levers only
cargo bench -p bcinr-powl -- "lever_comparison|lever4|lever1"

# Single scheduler comparison (N=4)
cargo bench -p bcinr-powl -- "lever_comparison"
```
