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
- **`crates/bcinr-logic/`**: The core algorithmic calculus (Mask, Int, Fix, Network, Bitset, Scan, UTF-8, Parse, DFA, Reduce, Sketch, Abstractions, Algorithms, Autonomic, Patterns).
- **`crates/bcinr-api/`**: Public facade and ergonomic wrappers for the logic layer.
- **`bcinr/`**: Main crate re-exporting public API.
- **`bcinr-core/`**: Core API facade.
- **`bcinr-bench/`**: Criterion benchmarks.
- **`tools/`**: Analysis and verification tools.

## Quality Gates & Tooling
The project includes automated gates to detect and prevent systematic code-quality issues:

- **`tools/bcinr-cheat-scanner/`**: Detects 5 systematic anti-patterns (self-canceling XOR, circular test references, magic constants, artificial file-length inflation, fake Hoare-logic comments). Blocks commits in CI.
- **`tools/bcinr-contract-gate/`**: Validates branchless contract compliance (cyclomatic complexity = 1, Branchless Contract doc comments).
- **`tools/bcinr-bench-auditor/`**: Ensures all public functions have corresponding benchmarks.
- **`tools/bcinr-reporter/`**: Generates audit reports and module integrity status.

Run `cargo make scan-cheats` and `cargo make contract-gate` to validate code before merge.

## Getting Started
To evaluate or integrate `bcinr`, run the workspace-wide benchmarks:

```bash
cargo bench
```

For domain-specific kernels, refer to the individual crates in `crates/`.
