# BCINR Entry Points and APIs

This document outlines the main entry points, API endpoints, and CLI commands available in the `bcinr` repository. 

## 1. Public API (Rust Library)
The core algorithmic functionality is exposed via the `bcinr` root crate, which serves as a facade for `bcinr-api` and `bcinr-logic`. 
- **Modules**: Includes academic-grade branchless algorithm implementations for SIMD (`simd`), bitsets (`bitset`), fixpoint arithmetic (`fix`), masking (`mask`), SWAR (`swar`), DFA state logic (`dfa`), and UTF-8 validation (`utf8`).
- **Public Symbols Reference**: The repository maintains an exhaustive list of all public symbols in `all_public_symbols.txt` at the root. Examples include `splat_u8x16`, `select_u64`, `bitonic_sort_32u32`, and `validate_utf8`. 

## 2. Server Entry Points (CLI Executables)

### Model Context Protocol (MCP) Server
- **Path**: `crates/bcinr-mcp/src/main.rs`
- **Description**: Exposes the library's capabilities as an MCP server running over `stdio`.
- **Tools Provided** (25 total across 6 groups):
  - **PDDL**: `pddl_domain_info`, `pddl_parse_domain`, `pddl_parse_problem`, `pddl_plan`, `manufacture_world`, `pddl_admit_domain`, `pddl_temporal_plan_info`, `route_capability_plan`
  - **POWL**: `powl_compile_sequence`, `powl_compile_choice`, `powl_admit_context`, `powl_capability_check`, `powl_plan_to_tape`, `analyze_schedule64`
  - **Core bcinr**: `bcinr_library_info`, `bcinr_mask_ops`, `bcinr_powl_info`
  - **bcinr-logic**: `utf8_validate`, `bitset_operations`, `dfa_info`, `scan_patterns`, `reduce_sequence`, `simd_string_info`
  - **Receipts**: `receipt_inspect`
  - **Cross-crate**: `system_capabilities`

### PDDL LSP Server
- **Path**: `crates/bcinr-pddl-lsp/src/main.rs`
- **Description**: An LSP server implementation for PDDL built with `lsp-max`. Communicates via `stdio`.

## 3. Auditing & Verification CLI Tools
These internal CLI tools (under `tools/`) are crucial for enforcing the deterministic, branchless constitutional rules (as defined in `AGENTS.md`).

- **`bcinr-cheat-scanner`**: Scans `crates/bcinr-logic` and `crates/bcinr-cmca` for anti-cheat violations (e.g., magic constants, self-canceling ops, scanner evasion). Exits with code `1` if violations are found.
- **`bcinr-contract-gate`**: Parses ASTs to verify reachability and enforces zero-allocation and branchless requirements on authoritative roots. 
  - *Usage*: `cargo run --bin bcinr-contract-gate [optional_path]`
- **`bcinr-bench-auditor`**: Analyzes `bcinr-logic/src/algorithms` to ensure all public functions are covered by benchmarks in `bcinr-bench/benches`.
- **`bcinr-reporter`**: Generates a substrate integrity report by running `cargo test --all-features` in `bcinr-logic` and tracking module-level pass/fail statistics.
- **`ggen`**: Test generator that automatically discovers all 300+ algorithms and generates falsification tests for arithmetic and bitwise invariants.
- **`bcinr-cmca-audit-harness`**: Links the allocator into an executable, making it inspectable by object-code audit tools (like `otool-classic`).
- **`rust_audit`**: A utility script tracking algorithm migration statuses based on tracking files.

## 4. Make Tasks (`cargo-make`)
The repository delegates complex workflows via `Makefile.toml`. Use `cargo make <task>` to run them. Notable tasks include:
- **Verification Gates**: `scan-cheats`, `contract-gate`, `audit-object-code`, `test-mutants`, `perf-branch-gate`, `verify-generated`, `ci`.
- **Factory/Kaizen**: `factory-build`, `factory-verify`, `factory-bench`, `factory-kaizen`, `chess-factory-sync`.
