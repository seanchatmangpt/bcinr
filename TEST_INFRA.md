# BCINR Test Infrastructure Specification

This document defines the opaque-box end-to-end (E2E) and differential testing infrastructure for the `bcinr` project and its Process Intelligence extensions. It establishes the test philosophy, inventories all key features (F1–F10), describes the E2E and differential test architecture, and sets coverage thresholds.

## 1. Test Philosophy

The `bcinr` test suite operates on three core principles:
- **Opaque-Box Testing**: The runner treats verification gates, compilation tools, and reference models as black boxes. It validates their end-to-end behavior, exit codes, and output patterns rather than asserting on internal implementation details.
- **Requirement-Driven**: Tests are designed directly from the project requirements: branchless correctness (Cyclomatic Complexity = 1 / Radon Law), zero-allocation hot paths, benchmark coverage, and semantic conformance.
- **Interface Compatibility**: Every test verification guarantees that the codebase remains fully compatible with its interface contracts across all layers (Petri, YAWL, POWL, WASM).

## 2. Feature Inventory

The test suite validates ten primary features:

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
* **F6: Petri Net Replay**
  - Validation of Petri net structures, bipartite arc direction constraints, token distribution markings, and bitmask-based token replay logic. Compares the replay outcome against the reference implementation to assert conformance.
* **F7: YAWL Routing Engine**
  - Verification of the 43 Workflow Patterns branchlessly using the Binary YAWL Engine. Handles AND/OR/XOR/Complex splits and joins, multiple-instance (MI) tasks, and cancellation regions.
* **F8: POWL Compiler**
  - Ensures correct lowering of POWL v2 (kinetic/geometric dialect ASTs) to the flat, table-driven POWL64 representation without graph traversal overhead. Emits scope descriptions, opcodes, and control masks.
* **F9: POWL Executor**
  - Validates unified executor execution over the flat opcode tape, verifying watchdog drains, residence promotions/demotions, scope stack projections, and concurrent dispatcher slot routing.
* **F10: WASM API C-Interface Wrappers**
  - Verification of the C-compatible FFI entry points (`extern "C"` functions) exporting Petri, YAWL, and POWL functionalities to the WASM host boundary, ensuring no-allocation safety.

## 3. Test Architecture

The Process Intelligence E2E and differential test suite uses a hybrid verification runner architecture that tests compilation, structural invariants, and runtime execution behavior.

```
                  ┌──────────────────────────────────────────────┐
                  │              E2E Test Runner                 │
                  └──────────────┬────────────────┬──────────────┘
                                 │                │
            ┌────────────────────▼────┐      ┌────▼────────────────────┐
            │    Differential Test    │      │    E2E & Integration    │
            │    Replay Verification  │      │    Static Code Checks   │
            └──────────┬──────────────┘      └────────────┬────────────┘
                       │                                  │
      ┌────────────────┴──────────────┐         ┌─────────┴─────────┐
      │                               │         │                   │
┌─────▼───────────────┐     ┌─────────▼─────┐ ┌─▼─────────┐   ┌─────▼─────────┐
│  bcinr Codebase     │     │ Branching     │ │  Contract │   │ anti-llm-cheat│
│  Implementation     │     │ Reference     │ │  Gate     │   │ LSP Canary    │
└─────────────────────┘     └───────────────┘ └───────────┘   └───────────────┘
```

### 3.1. Differential Testing
Differential tests verify that the highly-optimized branchless execution paths produce the exact same bitwise outcomes as the clean, branching reference suite. The test runner feeds generated traces and event logs to:
1. The `bcinr` production implementations.
2. The reference models (`playground/tests/reference/*`).
Any discrepancy in marking updates, token counts, or execution events constitutes a test failure.

### 3.2. Static and Dynamic Verification Gates
- **Contract Gate**: Exercises a custom parser and cyclomatic complexity checker over the codebase, verifying branchless invariants (Radon Law) at compilation boundaries.
- **Canary Scanning**: Feeds test inputs containing known LLM cheat patterns (e.g. self-canceling XOR, circular reference oracles, magic constants) into the project's cheat scanners to verify detection limits.

## 4. Verification Tiers

The test suite defines four tiers of verification to systematically evaluate code health:

### Tier 1: Unit & Component Tests
- Verifies the isolated correctness of individual modules (Petri structures, YAWL state mask updates, POWL opcode compile steps, and FFI wrappers).
- Focuses on individual requirements and strict type assertions.

### Tier 2: Boundary & Corner Cases
- Tests extreme inputs: empty nets, maximum-bound nets (64 places/transitions), invalid YAWL multiple-instance specifications, tripped watchdog states, and unbalanced scope exit bounds.
- Asserts that all edge inputs fail gracefully with structured error codes rather than panicking.

### Tier 3: Cross-Feature Integration & Differential Tests
- Co-executes multiple features, such as compiling a complex POWL model and immediately executing it on the branchless engine while diff-testing the output event receipts against the branching reference model.
- Asserts that FFI wrappers match direct Rust invocations under multi-threaded execution.

### Tier 4: Real-World Scenarios & Mutation Testing
- Validates the entire repository against end-to-end integration workflows (e.g., executing the complete `cargo test` suite, running the contract gate checks on all active algorithms, and auditing benchmark coverage).
- Includes adversarial mutant generation to verify that tests actively catch syntactically plausible bugs.

## 5. Coverage and Quality Thresholds

To achieve "PhD-Verified" status (SIS = 100/100), the codebase must satisfy the following thresholds:

| Verification Tier | Focus Area | Target Coverage / Count |
| --- | --- | --- |
| **Tier 1** | Feature Coverage (F1-F10) | Minimum 50 distinct test cases (5 per feature) |
| **Tier 2** | Boundary & Corner Cases | Minimum 40 edge-case tests across all layers |
| **Tier 3** | Cross-Feature / Differential | 100% agreement on trace acceptance & event outputs |
| **Tier 4** | E2E & Mutation | 100% mutant detection rate (all mutant codes fail tests) |
| **Total** | **Robustness Matrix** | **100+ Verified Test Scenarios** |
