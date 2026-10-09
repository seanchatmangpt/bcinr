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
| `emit_no_objects` | 9.765625 | 9.765625 | 79.4375 | 9.77 | 79.44 |
| `emit_8_objects` | 8.453125 | 8.453125 | 76.171875 | 8.45 | 76.17 |
| `emit_sla_breach` | 9.125 | 9.125 | 76.828125 | 9.12 | 76.83 |
| `chain_1_frame_blake3` | 373.328125 | 373.328125 | 538.671875 | 373.33 | 538.67 |
| `chain_100_frames_rolling` | 28434 | 28434 | 68160 | 28434.00 | 68160.00 |
| `conformance_check_pass` | 0.4169921875 | 0.4169921875 | 0.42724609375 | 0.42 | 0.43 |
| `conformance_check_fail` | 0.4169921875 | 0.4169921875 | 0.421630859375 | 0.42 | 0.42 |
| `replay_10_frames` | 13.03125 | 13.03125 | 26.046875 | 13.03 | 26.05 |
| `replay_64_frames_max` | 70.3125 | 70.3125 | 407.53125 | 70.31 | 407.53 |
| `denial_to_fired_mask` | 0.4169921875 | 0.4169921875 | 0.447509765625 | 0.42 | 0.45 |

Stated targets (from `receipt_bench.rs`): emit < 10 ns; BLAKE3 chain < 500 ns/frame measured envelope (amended 26.10.08 — the original < 15 ns/frame budget was physically unreachable: each frame hashes 131 bytes = 3 serial BLAKE3 block compressions, and hash compute is ~100% of per-frame cost. Falsifier: a rerun with `chain_1_frame_blake3` median < 250 ns/frame reopens the optimization lane); conformance check < 2 ns; replay < 20 ns/frame; denial mask ~ 1 ns.
