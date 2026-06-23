# Original User Request

## Initial Request — 2026-06-23T04:14:27Z

Implement a branchless, deterministic "process intelligence" library suite inside the `bcinr` playground, porting and refactoring functionality from reference repositories (`wasm4pm-compat`, `wasm4pm`, `dteam`, and `unibit`) to use `bcinr`'s branchless, zero-allocation primitives.

Working directory: /Users/sac/bcinr/playground
Integrity mode: development

## Requirements

### R1. Modular Process Intelligence Core
Implement four distinct process intelligence layers in `playground/src/`:
1. **Branchless Petri Net Engine (`petri`)**: High-performance token-based replay utilizing bitwise representations (`u64` bitmasks) for places, transitions, and markings.
2. **YAWL Routing Semantics (`yawl`)**: A bYAWL engine implementing AND/XOR/OR splits and joins, Cancelling Discriminators, and Interleaved Routing using state words and mask calculus.
3. **POWL/Ontology Matrix Compiler (`powl`)**: Lower hierarchical process trees and ontologies into a flat array of 64-aligned `Powl64Op` operations with precomputed predecessor/successor completion masks.
4. **Wasm API Boundary (`wasm`)**: A `#![no_std]` WebAssembly API wrapper for the Petri Net and bYAWL engines that avoids dynamic allocations and produces execution/conformance receipts.

### R2. Zero-Allocation & Branchless Execution ($CC=1$)
All hot-path computations must perform 0 heap allocations and avoid data-dependent branching (`if`, `match`, data-dependent loops). Instead, express control flow and state updates using constant-time bitwise polynomials, SIMD blends, and `bcinr` mask selection primitives (e.g. `select_u32`).

### R3. Reference Porting
Lower and refactor logic directly from the user's local reference repositories:
- Petri Net replay from `/Users/sac/wasm4pm-compat/src/petri.rs` and `/Users/sac/dteam/src/conformance/bitmask_replay.rs`
- YAWL engine from `/Users/sac/dteam/src/b_yawl/engine.rs`
- POWL matrix compilation from `/Users/sac/unibit/crates/unibit-powl64/src/lib.rs`
- WebAssembly wrapping from `/Users/sac/wasm4pm/src/wasm.rs`

### R4. Differential & Property-Based Testing
Rigorously test the correctness of the ported logic.
- Implement differential testing comparing the new branchless modules against the branching reference logic over randomized traces and inputs.
- Implement property-based tests (using `proptest` or similar) to verify correctness, $CC=1$ compliance, and soundness laws.

## Acceptance Criteria

### Execution & Compilation
- [ ] The `playground` crate compiles successfully under `#![no_std]`.
- [ ] No heap allocations are performed on the hot execution paths of any module.

### Core Modules
- [ ] `petri`: Bitmask-based token replay executes branchlessly and correctly replays traces.
- [ ] `yawl`: Supports AND/XOR/OR splits and joins, Cancelling Discriminators, and Interleaved Routing.
- [ ] `powl`: Flattens process trees into non-recursive `Powl64Op` instructions executed via static masks.
- [ ] `wasm`: Correctly interfaces with the engines without std allocations.

### Verification
- [ ] Differential tests prove that the branchless implementation yields identical outputs to the branching references.
- [ ] Property-based tests pass cleanly under `cargo test`.
