# Scope: bcinr-logic correctness and compliance

## Architecture
The system consists of the following components:
- `crates/bcinr-logic/src/algorithms/`: 307 public branchless algorithms.
- `tools/u64_audit.py`: Python audit script updating references, comments, proofs, and padding.
- `tools/bcinr-contract-gate`: AST complexity, bluff, and contract comment verification.
- `tools/bcinr-bench-auditor`: Checks for missing Criterion benchmarks.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Rust Audit Tool & Algo Correctness | Write/compile/run a Rust binary/tool to replace `u64_audit.py`'s functionality, execute updates to references/comments/proofs/padding, and refactor implementation bodies to match oracles branchlessly. | none | IN_PROGRESS |
| 2 | Codebase Warnings & Doctest Fixes | Resolve 22 compiler/lint warnings in `bcinr-logic` and fix workspace doctest failures due to dependency conflicts. | M1 | PLANNED |
| 3 | Tool Admissibility & AST Migration | Migrate substring checks in `bcinr-contract-gate` and `bcinr-bench-auditor` to AST parsing (`syn`); fix Cargo.lock version/references diagnostics (`ANTI-LLM-VERSION-001`, `ANTI-LLM-SURFACE-001`). | M1 | PLANNED |
| 4 | Benchmark Coverage | Add Criterion benchmarks for the 59 helper functions or refine `bcinr-bench-auditor` filters. | M2, M3 | PLANNED |
| 5 | Release Verification & Audit | Verify against E2E tests and run the Forensic Auditor (`teamwork_preview_auditor`). | M4 | PLANNED |

## Interface Contracts
- Each algorithm file in `crates/bcinr-logic/src/algorithms/` must match its reference function exactly (100% equivalence passing under proptest).
- All public functions in these algorithm files must be completely branchless (CC = 1), perform zero allocations, and contain the `"Branchless Contract"` comment in their doc comments.
- Verification gates (contract-gate, bench-auditor, anti-llm-cheat-lsp) must compile and exit cleanly (0 errors, 0 diagnostics).
