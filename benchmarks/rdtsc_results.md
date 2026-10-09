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
| `emit_no_objects` | 10.421875 | 10.421875 | 90.5 | 10.42 | 90.50 |
| `emit_8_objects` | 9.109375 | 9.109375 | 85.578125 | 9.11 | 85.58 |
| `emit_sla_breach` | 10.421875 | 10.421875 | 89.1875 | 10.42 | 89.19 |
| `chain_1_frame_blake3` | 416.28125 | 416.28125 | 543.234375 | 416.28 | 543.23 |
| `chain_100_frames_rolling` | 28310 | 28310 | 31684 | 28310.01 | 31684.01 |
| `conformance_check_pass` | 0.457763671875 | 0.457763671875 | 0.47216796875 | 0.46 | 0.47 |
| `conformance_check_fail` | 0.457763671875 | 0.457763671875 | 0.47216796875 | 0.46 | 0.47 |
| `replay_10_frames` | 25.390625 | 25.390625 | 26.3125 | 25.39 | 26.31 |
| `replay_64_frames_max` | 70.3125 | 70.3125 | 119.8125 | 70.31 | 119.81 |
| `denial_to_fired_mask` | 0.468017578125 | 0.468017578125 | 0.472412109375 | 0.47 | 0.47 |

Stated targets (from `receipt_bench.rs`): emit < 10 ns; BLAKE3 chain < 15 ns/frame; conformance check < 2 ns; replay < 20 ns/frame; denial mask ~ 1 ns.
