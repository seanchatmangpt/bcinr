# Core Modules and Libraries of BCINR

`bcinr` (BranchlessCInRust) is a performance-first, research-grade systems library providing a principled calculus for branchless algorithmics. The main functionality is built around deterministic computation with strictly bounded latency, zero dynamic heap allocations, and zero data-dependent branching (enforcing the Radon Law where cyclomatic complexity $CC=1$).

## Architectural Foundations
The project is built on the **Branchless Calculus** discipline, ensuring stable hot loops that eliminate branch mispredictions and side-channel timing risks. All core functionality is built `#![no_std]` and avoids the Rust `alloc` crate in execution paths. 

## Primary Crates and Modules

### 1. `bcinr-logic` (The Core Calculus)
This is the heart of the mathematical substrate. It contains the core branchless algorithms with zero dependencies:
- **`mask`**: Branchless bitwise masks and selection.
- **`int` & `fix`**: Low-level integer manipulation and strictly bounded Q16.16 fixed-point arithmetic.
- **`bitset`**: Branchless bitset logic operations.
- **`network`**: Sorting and permutation networks.
- **`scan`, `utf8`, `parse`, `dfa`, `reduce`, `sketch`**: Fast string parsing, hashing, SIMD-accelerated reductions, and DFA-based parsing.

### 2. `bcinr-api` / `bcinr-core` / `bcinr`
These crates serve as public facades. They wrap the complex `bcinr-logic` primitives into ergonomic, stable APIs for downstream consumers. 

### 3. `bcinr-powl` & `bcinr-powl-receipt`
Implements a partially ordered workflow language (POWL) scheduler.
- Features a formally verified, branchless runtime capable of executing workflow tapes with cryptographic proof of execution via a rolling BLAKE3 receipt chain.
- Utilizes layers like `const_scheduler` and `hierarchical_time_wheel` for extraordinary throughput (14M instances/sec on a single core).

### 4. `playground` (Process Intelligence Suite)
The central suite for executing process intelligence without allocations:
- **`petri`**: Bitmask-based Petri Net token replay engine.
- **`yawl`**: YAWL routing semantics engine supporting complex OR/AND/XOR splits and parallel routing.
- **`powl`**: POWL ontology matrix compiler for flat non-recursive execution.
- **`wasm`**: No_std WebAssembly C-interface boundary wrapping these engines.

### 5. `bcinr-cmca` (Covariance Monitoring and Calibration Assessment)
Implements an autonomic feedback loop based on the MAPE-K (Monitor, Analyze, Plan, Execute, Knowledge) model.
- Includes fixed-point arithmetic (`fixed`), panic-free bump allocators (`allocator`), and calibration safety threshold evaluation (`observatory`).

### 6. Tools (`tools/`)
The repository includes several custom tools that enforce the strict deterministic contracts:
- **`bcinr-cheat-scanner`**: Detects syntactic workarounds or fake implementation anti-patterns.
- **`bcinr-contract-gate`**: Validates the mandatory $CC=1$ cyclomatic complexity branchless contract.
- **`bcinr-bench-auditor`**: Enforces O(1) performance coverage requirements across primitives.
