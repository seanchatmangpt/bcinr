# Architecture and Configuration of BCINR

`bcinr` (BranchlessCInRust) is a performance-first, research-grade systems library providing a principled calculus for branchless algorithmics. It serves as a deterministic computational substrate.

## Top-Level Architecture

The repository is structured as a Cargo workspace with distinct crate boundaries to enforce domain taxonomy and strict execution laws. 

### Core Laws and Principles
1. **The Radon Law (CC=1):** All authoritative public primitives must avoid data-dependent branching (`if`, `match`, or dynamic loops). Logic must use bitwise polynomials and mask selection.
2. **Zero-Allocation Substrate:** Hot-path execution is `#![no_std]` and strictly heap-allocation-free, utilizing arenas and fixed-capacity buffers (e.g., `BumpArena`, `LockFreeSlab`).
3. **Deterministic Integrity:** Kernels must be reproducible across architectures. State mutation requires full admission and complete transactions (ReceiptSound law).
4. **Substrate Integrity Score (SIS):** A verified file must achieve 100/100 across mathematical proof, structural enforcement, adversarial mutants, and branchless implementation.

### Workspace Crates Taxonomy
- **`crates/bcinr-logic/`**: The core algorithmic calculus covering fixed-point math, networks, bitsets, scanning, parsing, DFA, and abstractions.
- **`crates/bcinr-api/`**: The public facade and ergonomic wrappers for the logic layer.
- **`crates/bcinr-cmca/`**: Autonomic control and state mutation logic, featuring heavily monitored RDF artifacts and receipts.
- **`crates/bcinr-powl/`, `crates/bcinr-pddl/`, `crates/bcinr-mfw-ir/`**: Engines for POWL ontology compilation, process trees, and routing semantics.
- **`crates/bcinr-mcp/`**: An MCP (Model Context Protocol) server implementation.
- **`bcinr/`** & **`bcinr-core/`**: Top-level API and facade.
- **`playground/`**: E2E process intelligence testbed containing branchless, zero-allocation engines for Petri nets, YAWL routing, POWL compilation, and `#![no_std]` WASM APIs.
- **`tools/`**: Quality and verification gates:
  - `bcinr-cheat-scanner`: Detects semantic anti-patterns (e.g., self-canceling XOR, magic constants).
  - `bcinr-contract-gate`: Validates branchless contract compliance (CC=1).
  - `bcinr-bench-auditor`: Ensures full benchmark coverage for all public functions.

## Core Configuration Files

- **`Cargo.toml` (Root Workspace)**:
  - Configures the workspace members and default profiles.
  - Customizes `profile.release` for LTO and `profile.bench` for optimal relative-timing benchmark compiles.
  - Contains specific registry path patches to synchronize nested dependencies (e.g., `wasm4pm-compat`).

- **`Makefile.toml`**: 
  - Comprehensive `cargo-make` definition file driving the CI, compilation, and audit pipelines.
  - Encodes rigorous gates such as:
    - `scan-cheats` and `contract-gate` for static structural analysis.
    - `test-mutants` for adversarial unit testing over specific domain mutants.
    - `audit-object-code` for raw disassembly inspection.
    - `verify-generated` for zero-generation validation of committed artifact digests.

- **`.deny.toml`**: 
  - Enforces dependency graph constraints via `cargo-deny`.
  - Configured to validate license constraints (e.g., permitting MIT/Apache-2.0), forbid duplicated or banned dependencies, and ensure components derive from trusted registries.

- **`AGENTS.md` & `GEMINI.md`**: 
  - Define the authoritative architectural constitution, including the rigorous multi-agent workstream decomposition (Hoare Oracle, Turing Machine, Armstrong Fault, Von Neumann Bypass).
  - Override standard rust or optimization principles with absolute deterministic substrate rules.

- **`.mcp.json`**: 
  - Simple MCP configuration pointing to the local `bcinr-mcp` target binary.
