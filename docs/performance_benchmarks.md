# Performance Benchmarking Strategy

## Overview
The `bcinr` project employs a rigorous, performance-first benchmarking strategy centered around Criterion (v0.5). It emphasizes deterministic, branchless execution with precise measurements down to picosecond latencies. The suite enforces zero-allocation hot paths, cache-conscious sequential access, and ensures that fundamental operations run in $O(1)$ constant time or scale predictably.

## Core Benchmarking Requirements
- **Metrics Tracked:** Latency (picoseconds/nanoseconds), CPU Cycles per operation, and Throughput (B/s or M/s).
- **Branch Misprediction Gate:** Hot paths must incur < 0.1% branch misses of total instructions. This is verified via hardware performance counters (e.g., using `cargo make perf-branch-gate` on Linux or Instruments on macOS).
- **Zero-Allocation:** The steady state execution must perform zero heap allocations, ensuring predictable $O(1)$ latency.
- **Hardware Profile:** Baseline benchmarks are targeted and validated against Apple M-series ARM64 (Firestorm cores), while ensuring SIMD fallback verification for portability.

## The `bcinr-bench-auditor`
To enforce strict quality standards, the repository utilizes a custom static analysis tool called `bcinr-bench-auditor`. It runs as a dynamic verification gate (Feature F4 in the test infrastructure) to ensure comprehensive benchmark coverage.

### How it Works:
1. **AST Parsing:** It uses the `syn` crate to parse the Abstract Syntax Tree (AST) of all source files in the `crates/bcinr-logic/src/algorithms` directory.
2. **Symbol Extraction:** It extracts the names of all `pub fn` declarations. It automatically filters out ignored boilerplate (like `new`, `len`, `is_empty`, `check_integrity`) and functions ending with `_gate` or starting with `bench_`.
3. **Benchmark Scanning:** It then parses all files in the `bcinr-bench/benches` directory, extracting all identifiers and string literals.
4. **Validation:** It verifies that every extracted public function from the logic algorithms is explicitly referenced (either as an identifier or literal) in the benchmark suite.

### Requirements for O(1) Coverage
The project enforces an **O(1) Coverage** (1-to-1 or 100% mapping) requirement for its public capabilities. 
- Every public algorithmic primitive *must* have an exact matching identifier in the benchmark suite.
- If any function is missing a corresponding benchmark, `bcinr-bench-auditor` flags the omission and forcefully fails the build with a non-zero exit code.
- This creates an automated, constant-time mechanism to guarantee that no hot-path logic or new primitives can be merged into the codebase without accompanying benchmark metrics. It ensures that the strict $O(1)$ branchless performance claims are always substantiated by runtime measurements.
