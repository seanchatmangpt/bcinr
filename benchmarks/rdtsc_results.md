# RDTSC Measured Tick Tables — receipt subsystem (`bcinr-powl`)

Grounds the dissertation Ch9 latency claims with measured hardware-counter ticks. Regenerate with `cargo bench -p bcinr-powl --bench rdtsc` (harness: `crates/bcinr-powl/benches/rdtsc.rs`).

## Environment

- Counter: **CNTVCT_EL0 (aarch64 fallback surrogate; rdtsc unavailable on this host)**
- Host: Apple M3 Max, arm64 (Darwin), macos/aarch64, rustc 1.99.0-nightly (daf2e5e18 2026-07-13)
- Calibration: 1.000 ticks/ns, measured against `std::time::Instant` over a 0.25 s busy loop
- Counter-read overhead: 0 ticks (median of paired reads; subtracted from every timed sample)
- Method: 2000 timed samples per kernel, each sampling a fixed inner-iteration count K (64–4096 per kernel, 1 for the 100-frame chain; see harness) with 200 warmup; per-op ticks = (sample ticks − overhead) / K

## Tick table

| kernel | median ticks | p50 ticks | p99 ticks | median ns | p99 ns |
|---|---:|---:|---:|---:|---:|
| `emit_no_objects` | 7.8125 | 7.8125 | 67.71875 | 7.81 | 67.72 |
| `emit_8_objects` | 7.171875 | 7.171875 | 67.328125 | 7.17 | 67.33 |
| `emit_sla_breach` | 7.8125 | 7.8125 | 69.015625 | 7.81 | 69.02 |
| `chain_1_frame_blake3` | 322.546875 | 322.546875 | 369.140625 | 322.55 | 369.14 |
| `chain_100_frames_rolling` | 24434 | 24434 | 29226 | 24434.00 | 29226.00 |
| `conformance_check_pass` | 0.356201171875 | 0.356201171875 | 0.44775390625 | 0.36 | 0.45 |
| `conformance_check_fail` | 0.35595703125 | 0.35595703125 | 0.44775390625 | 0.36 | 0.45 |
| `replay_10_frames` | 11.0625 | 11.0625 | 13.671875 | 11.06 | 13.67 |
| `replay_64_frames_max` | 59.90625 | 59.90625 | 75.53125 | 59.91 | 75.53 |
| `denial_to_fired_mask` | 0.3662109375 | 0.3662109375 | 0.370361328125 | 0.37 | 0.37 |

Stated targets (from `receipt_bench.rs`): emit < 10 ns; BLAKE3 chain < 500 ns/frame measured envelope (amended 26.10.08 — the original < 15 ns/frame budget was physically unreachable: each frame hashes 131 bytes = 3 serial BLAKE3 block compressions, and hash compute is ~100% of per-frame cost. Falsifier: a rerun with `chain_1_frame_blake3` median < 250 ns/frame reopens the optimization lane); conformance check < 2 ns; replay < 20 ns/frame; denial mask ~ 1 ns.

## Negative result — incremental chain hasher (26.10.09)

Attempted single-pass `Hasher` reuse across frames (persistent
`blake3::Hasher` in `OcelCausalReceipt`, `reset()`+update+finalize per frame,
amortizing per-frame `Hasher::new()`). Same host/config harness.

| kernel | before median | after median |
|---|---:|---:|
| `chain_1_frame_blake3` | 322.55 ns | 324.88 ns |
| `chain_100_frames_rolling` | 24393 ns | 23851 ns |

Both deltas are inside run-to-run noise (session baselines ranged
322–373 ns for `chain_1_frame_blake3`). **Reverted** (<20% gate). The three
serial 64-byte BLAKE3 block compressions are the hash of the 131-byte frame
input itself and cannot be amortized; only the trivial `Hasher::new()` IV
store was saved. The 26.10.08 amendment stands; the <250 ns/frame falsifier
remains unmet. (This section is hand-maintained; `cargo bench --bench rdtsc`
regenerates only the table above.)
