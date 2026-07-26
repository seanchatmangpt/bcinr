# bcinr — BranchlessCInRust (v26.7.25)

`bcinr` is a performance-first, research-grade systems library providing a principled calculus for branchless algorithmics. It is designed for high-performance, deterministic systems where predictable latency, memory-safety, and side-channel resilience are critical requirements.

## Key Features

- **Deterministic Latency:** All primitives are branchless ($O(1)$ constant time), eliminating pipeline stalls and side-channel timing risks.
- **$\mathcal{B}$-Calculus Formalism:** Each primitive maps to a formal framework ensuring invariant-preserving state transitions.
- **Hardware-Agnostic SIMD:** SSE4.2 with verified portable fallbacks for ARM Neon and WebAssembly.
- **Zero-Dependency Core:** The logic layer is strictly `no_std` with zero external dependencies.
- **POWL Runtime:** Partially Ordered Workflow Language scheduler with cryptographic proof of execution (14 M instances/sec, single core).

## Installation

```toml
[dependencies]
bcinr-logic = "26.7.25"   # core algorithms, no_std, zero deps
bcinr-powl  = "26.7.25"   # workflow scheduler + conformance gate
```

## Quick Start

```rust
use bcinr_logic::mask::{select_u32, min_u32, max_u32};
use bcinr_logic::fix::add_sat;

// Branchless selection: mask 0xFFFFFFFF → first arg, 0x0 → second
let val = select_u32(0xFFFFFFFF, 10, 20);
assert_eq!(val, 10);

// Saturating arithmetic — never wraps past MAX
let sum = add_sat(u32::MAX, 1u32);
assert_eq!(sum, u32::MAX);
```

See `examples/` for runnable witnesses (`cargo run --example <name>`).

## POWL Workflow Runtime

`bcinr-powl` implements a formally verified, branchless scheduler for
Partially Ordered Workflow Language (POWL) tapes with cryptographic proof
of execution via a rolling BLAKE3 receipt chain.

```
Conformance gate   395 ps   (Q16.16 branchless predicate)
Scheduler tick     2.07 ns  (single op, SWAR bit-scan)
Full workflow      69.6 ns  (10-op chain, E2E with receipt)
Throughput         14 M instances/sec, single core
const_tick (N=4)  535 ps   (compile-time topology, Lever 4)
```

### v26.6.24 — 1000× Engineering Roadmap

Three compounding performance levers, all implemented and benchmarked:

| Lever | Module | Speedup | What |
|---|---|---|---|
| **Lever 1** | `scheduler_wide` | 8–16× capacity | 512-op `KBitSet<8>` wide tape |
| **Lever 3** | `hierarchical_time_wheel` | 100× deadline density | 3-level O(1) amortized cascade |
| **Lever 4** | `const_scheduler` | 1240× vs petri, 24× vs legacy | Compile-time topology via `const fn` Kahn + `generic_const_exprs` |

**Benchmark delta (N=4 linear chain, Apple M-series):**

| Scheduler | Latency | Ratio |
|---|---|---|
| `const_tick` | **535 ps** | 1× |
| `legacy` SWAR | 12.9 ns | 24× slower |
| `wired_petri` | 664 ns | 1,240× slower |

## Documentation (Diátaxis)

- **[Tutorials](docs/diataxis/tutorials/)** — implementing kernels, SIMD vectorization
- **[How-To Guides](docs/diataxis/how-to/)** — side-channel hardening, WCET bounding
- **[Explanations](docs/diataxis/explanation/)** — Branchless Calculus, architectural design
- **[References](docs/diataxis/reference/)** — full API catalog and specifications
- **[Anti-Patterns](docs/diataxis/explanation/anti-patterns.md)** — structural hazards to avoid

[Full Documentation Index](docs/diataxis/INDEX.md)

## Formal Basis

The dual-audience arXiv/HBR paper (26 pages) covers formal proofs, Chicago TDD
conformance theory, and Fortune 5 strategic framing:
[**thesis.pdf**](./thesis.pdf) — *Agency at the Speed of Silicon*

## Performance

- **[Benchmark Charter](docs/BENCHMARKS.md)** — targets and measured baselines
- **[Architecture Overview](ARCHITECTURE.md)** — domain taxonomy and design philosophy

## Development

```bash
make check    # compile all targets
make test     # full test suite (928 tests)
make clippy   # zero-warning lint
make bench    # Criterion benchmarks
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
