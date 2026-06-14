# Project: BCINR release v26.6.12

## Architecture
- `crates/bcinr-logic/`: The core logic crate containing 307 branchless algorithm files under `src/algorithms/`.
- `tools/bcinr-contract-gate/`: Static analysis tool parsing rust code to verify Cyclomatic Complexity = 1 and compliance.
- `tools/bcinr-bench-auditor/`: Tool comparing public symbols against criterion benchmarks.
- `tools/u64_audit.py`: Python audit script updating doc clauses, references, proof blocks, and padding in algorithm files.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Test Infra Setup | Design and build the E2E test suite and runner | none | DONE (Conv: 403cae79-f741-45a4-b67d-1113397a0ae2) |
| 2 | Rust-based Audit & Algo Correctness | Write a Rust binary in the workspace to update references, doc clauses, proof blocks, and padding in algorithm files, and refactor implementation bodies to match references branchlessly and satisfy contract gate | M1 | DONE (Conv: 8240f309-2f4c-4f19-bddb-0cc5eaf65784) |
| 3 | Tool Admissibility & LSP Fixes | Migrate substring checks in contract-gate/bench-auditor to AST checks; fix Cargo.lock/ORIGINAL_REQUEST LSP warnings | M2 | DONE (Conv: 8240f309-2f4c-4f19-bddb-0cc5eaf65784) |
| 4 | Warn & Link Fixes | Fix 22 compiler/lint warnings and solve workspace doctest linkage conflicts | M2 | DONE (Conv: 8240f309-2f4c-4f19-bddb-0cc5eaf65784) |
| 5 | Benchmark Coverage | Add Criterion benchmarks for the 59 helper functions or refine bench-auditor filters | M3, M4 | DONE (Conv: 8240f309-2f4c-4f19-bddb-0cc5eaf65784) |
| 6 | Release & Victory Verification | Verify all E2E tests, scan diagnostics, contract gates, and run Forensic Auditor | M5 | DONE (Conv: 8240f309-2f4c-4f19-bddb-0cc5eaf65784) |

## Interface Contracts
- `crates/bcinr-logic/src/algorithms/`: Each file must export `pub fn <name>(val: u64, aux: u64) -> u64` with CC=1 and zero heap allocations.
- Each algorithm file must contain doc comments with the literal phrase `"Branchless Contract"`.
- `tools/u64_audit.py`: Python script to format and update reference code in all 307 files.
- `anti-llm-cheat-lsp`: LSP scan command must exit with 0 diagnostics.

## Code Layout
- `crates/bcinr-logic/src/algorithms/`: The branchless primitive implementations.
- `tools/`: Scanners, auditors, and formatting scripts.
- `bcinr-bench/benches/`: Criterion benchmarks.
