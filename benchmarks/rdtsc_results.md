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
| `emit_no_objects` | 9.125 | 9.125 | 78.125 | 9.12 | 78.12 |
| `emit_8_objects` | 7.8125 | 7.8125 | 75.515625 | 7.81 | 75.52 |
| `emit_sla_breach` | 8.46875 | 8.46875 | 71.609375 | 8.47 | 71.61 |
| `chain_1_frame_blake3` | 371.359375 | 371.359375 | 377.234375 | 371.36 | 377.23 |
| `chain_100_frames_rolling` | 28268 | 28268 | 28517 | 28268.00 | 28517.00 |
| `conformance_check_pass` | 0.4169921875 | 0.4169921875 | 0.42138671875 | 0.42 | 0.42 |
| `conformance_check_fail` | 0.4169921875 | 0.4169921875 | 0.42138671875 | 0.42 | 0.42 |
| `replay_10_frames` | 13.015625 | 13.015625 | 13.296875 | 13.02 | 13.30 |
| `replay_64_frames_max` | 69.03125 | 69.03125 | 70.875 | 69.03 | 70.88 |
| `denial_to_fired_mask` | 0.4169921875 | 0.4169921875 | 0.42724609375 | 0.42 | 0.43 |

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

### Follow-up: reset-free reuse via update-after-finalize (26.10.09)

Second attempt at the same lane: one persistent `Hasher` in
`OcelCausalReceipt`, per frame `update(frame_bytes)` then `finalize()` to a
snapshot, **never resetting** — supported in blake3 1.8.5 (`finalize(&self)`
is non-consuming, verified against the vendored source). Note this variant
changes the chain function: `chain_hash(t)` becomes the running
`BLAKE3(genesis || frame_0 || ... || frame_t)` instead of the per-frame fold
`BLAKE3(chain_hash(t-1) || frame_t)` (still deterministic and causal since
each frame embeds `prior_hash`). Same host/config harness.

| kernel | baseline median (same code) | variant median |
|---|---:|---:|
| `chain_1_frame_blake3` | 322.55 ns | 350.53 / 350.25 ns |
| `chain_100_frames_rolling` | 24393 ns | 31226 / 31185 ns |

No improvement: the 1-frame kernel medians sit inside the session's observed
baseline noise band (identical baseline code re-measured 322–373 ns across
runs this session), and the 100-frame kernel trended ~25–30% *worse*.
**Reverted** (<20% gate); working tree restored to the HEAD chain fold and
the committed tick table re-regenerated on the reverted code. The 26.10.08
budget amendment stands; the <250 ns/frame falsifier remains unmet. (This
subsection is hand-maintained; the harness regenerates only the table.)
