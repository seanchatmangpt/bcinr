# BCINR Test Infrastructure Specification

This document defines the opaque-box end-to-end (E2E) testing infrastructure for the `bcinr` project. It establishes the test philosophy, inventories all key features, describes the test runner architecture, outlines real-world application scenarios, and sets coverage thresholds.

## 1. Test Philosophy

The `bcinr` test suite operates on three core principles:
- **Opaque-Box Testing**: The runner treats the verification gates and compilation tools as black boxes, validating their end-to-end behavior, exit codes, and output patterns rather than asserting on internal implementation details.
- **Requirement-Driven**: Tests are designed directly from the project laws: branchless correctness (Cyclomatic Complexity = 1), zero-allocation hot paths, benchmark coverage, and LSP canary compliance.
- **Interface Compatibility**: Every test verification guarantees that the codebase remains fully compatible with its interface contracts.

## 2. Feature Inventory

The test suite validates five primary features:

* **F1: Workspace Health**
  - Verification of overall codebase compilation (`cargo check`), unit tests, doc tests, and boundary checks.
* **F2: Contract Gate**
  - Enforcement of branchless code structures (JCC checks, CC=1), elimination of forbidden arithmetic operators in select bitwise primitives, and the presence of `"Branchless Contract"` doc comments.
* **F3: Rust Lint & Formatting Compliance**
  - Verification of standard style formatting using `cargo fmt` and static code analysis/quality checks using `cargo clippy`.
* **F4: Bench Auditor**
  - Dynamic verification of benchmark coverage for all public symbols using `bcinr-bench-auditor`.
* **F5: LSP Canary Compliance**
  - Static admissibility scanning via the `anti-llm-cheat-lsp` tool to ensure compliance with LSP anti-cheat requirements.

## 3. Test Architecture

The E2E test suite is implemented as a Python test runner (`tests/e2e_test_runner.py`) using the standard library `unittest` framework. 

### Runner Flow
1. **Setup**: Prepares clean temporary directories and files inside `crates/bcinr-logic/src/algorithms/` or in system temporary folders.
2. **Execution**: Spawns process calls to Cargo commands, Rust binaries (`bcinr-contract-gate`, `bcinr-bench-auditor`), and the `anti-llm-cheat-lsp` tool.
3. **Assertion**: Validates exit codes and inspects stdout/stderr for expected diagnostic patterns (e.g. `MISSING_U64_CONTRACT`, `FAIL: ... has Cyclomatic Complexity`, or `ANTI-LLM-*`).
4. **Tear-Down**: Cleans up all temporary files and directories to ensure zero side effects on the git workspace.

## 4. Real-World Application Scenarios (Tier 4)

Tier 4 consists of E2E verification of full-repository health checks under real-world releases:
- **Scenario 1 (Full Workspace Cargo Verification)**: Asserts compilation status of the whole workspace, verifying the build and unit tests pass.
- **Scenario 2 (Contract Gate Workspace Verification)**: Asserts the execution of `bcinr-contract-gate` on the entire repo to check for CC=1 and missing contract comments.
- **Scenario 3 (Bench Auditor Coverage Verification)**: Asserts that all public API functions are audited for benchmark coverage via `bcinr-bench-auditor`.
- **Scenario 4 (Canary LSP Scan Verification)**: Asserts that the `anti-llm-cheat-lsp` scanner correctly analyzes the repository, identifying existing non-compliance issues dynamically.
- **Scenario 5 (Codebase Format & Style Verification)**: Asserts formatting compliance on the active codebase.

## 5. Coverage Thresholds

The suite defines 60 test cases divided into 4 tiers of verification:

| Verification Tier | Focus Area | Minimum Count |
| --- | --- | --- |
| **Tier 1** | Feature Coverage (F1-F5) | 25 cases (5 per feature) |
| **Tier 2** | Boundary & Corner Cases | 25 cases (5 per feature) |
| **Tier 3** | Cross-Feature Combinations | 5 cases |
| **Tier 4** | Real-World Application Scenarios | 5 cases |
| **Total** | | **60 cases** |
