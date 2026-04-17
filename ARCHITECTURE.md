# Architecture: bcinr

`bcinr` (BranchlessCInRust) is a performance-first, research-grade systems library providing a principled calculus for branchless algorithmics. It is designed for high-performance, deterministic autonomic systems (like PICTL) where predictable latency and memory footprint are non-negotiable.

## Design Philosophy
`bcinr` is built on the **Branchless Calculus** discipline: transforming conditional control flow into arithmetic, bitwise, or table-driven dependencies.

## Key Principles
1.  **Stable Hot Loops:** Branchless primitives minimize branch mispredictions; throughput kernels saturate cache bandwidth.
2.  **Machine-Conscious APIs:** APIs reflect machine realities (SoA layouts, register-bound operations, and fixed-capacity buffers).
3.  **Deterministic Integrity:** All kernels are reproducible across architectures, ensuring stable behavior for receipt-based telemetry.
4.  **Zero-Allocation Substrate:** The `Mem` module provides arenas, rings, and epochs to eliminate heap traffic in steady-state execution.

## Documentation Reference
Deep-dive technical details and performance charters are maintained in the `./docs/` directory:
- [Benchmark Charter](./docs/BENCHMARKS.md): Performance, memory, and complexity targets for algorithm families.

## Domain Taxonomy
The library is organized into specialized crate boundaries:
- **`crates/bcinr-logic/`**: The core algorithmic calculus (Mask, Int, Fix, Network, Bitset, Scan, UTF-8, Parse, DFA, Reduce, Sketch).
- **`crates/bcinr-api/`**: Public facade and ergonomic wrappers for the logic layer.
- **`crates/bcinr-mem/`**: Systems substrate (Arena, Ring, Slab, Epoch).
- **`crates/bcinr-exec/`**: Staged execution (Plans, Cells, Resumable Pipelines).

## Getting Started
To evaluate or integrate `bcinr`, run the workspace-wide benchmarks:

```bash
cargo bench
```

For domain-specific kernels, refer to the individual crates in `crates/`.
