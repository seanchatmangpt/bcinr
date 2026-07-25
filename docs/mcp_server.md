# bcinr-mcp: Model Context Protocol Server

The `bcinr-mcp` crate implements a native Rust Model Context Protocol (MCP) server that exposes the entire bcinr ecosystem as a Unified Execution Platform. It realizes the **BRCE** (Branchless, Receipt-producing, Cryptographic, Execution) model by providing zero-trust admission, deterministic branchless execution paths, and cryptographic BLAKE3 receipts.

## System Architecture

The MCP server coordinates a complete candidate-future execution loop:

1. **PDDL8 Planning**: Validate domains/problems and find temporal plans.
2. **POWL Compilation**: Convert plans into POWL (linear or branching) execution tapes.
3. **Prolog8 Admission**: O(1) branchless capability and execution context verification.
4. **Execution & Receipts**: Atomic execution emitting BLAKE3 receipts integrated with OCEL (Object-Centric Event Logs).

## Available MCP Tools (23 Tools)

The server exposes 23 tools divided into 6 functional groups:

### 1. PDDL Planning (7 tools)
- `pddl_domain_info` — Parse & describe PDDL 3.1 domain
- `pddl_parse_domain` — Validate domain, return JSON metadata
- `pddl_parse_problem` — Validate problem, return JSON metadata
- `pddl_plan` — BFS planner (STRIPS → temporal plans)
- `pddl_admit_domain` — Prolog8 R ⊢ A admission gate
- `pddl_temporal_plan_info` — Extract temporal constraints & metrics
- `manufacture_world` — **Atomic operation**: admit → plan → execute → BLAKE3 receipt

### 2. POWL Workflow Orchestration (5 tools)
- `powl_compile_sequence` — Compile labels → POWL Sequence tape (linear)
- `powl_compile_choice` — Compile labels → POWL XorChoice tape (branching)
- `powl_admit_context` — O(1) execution context LUT dispatch (Priority/Standard/Background/Quarantine)
- `powl_capability_check` — Branchless O(1) permission bitset verification
- `powl_plan_to_tape` — Bridge PDDL plans → POWL op specs

### 3. Core bcinr Library (3 tools)
- `bcinr_library_info` — Library overview (crates, modules, capabilities)
- `bcinr_mask_ops` — Branchless bitset algebra (and/or/xor/andn/nand/nor/popcount/lz/tz)
- `bcinr_powl_info` — POWL runtime description (phase lattice, topologies, ops)

### 4. Branchless Algorithms (6 tools)
- `utf8_validate` — Branchless UTF-8 byte sequence validation
- `bitset_operations` — O(1) bitset operations (popcount/leading_zeros/trailing_zeros/msb/lsb)
- `dfa_info` — DFA & automata capabilities
- `scan_patterns` — Branchless pattern scanning algorithms
- `reduce_sequence` — Folding & aggregation algorithms
- `simd_string_info` — SIMD/SWAR text processing (throughput: 10-20GB/s)

### 5. Receipt Verification (1 tool)
- `receipt_inspect` — Inspect & verify POWL execution receipts

### 6. Cross-crate Integration (1 tool)
- `system_capabilities` — Unified capability report across all crates

## Binary Details
- **Binary Path**: `/Users/sac/bcinr/target/debug/bcinr-mcp`
- **Registration**: Integrated via `/Users/sac/.claude/settings.json` and `.mcp.json`
- **Underlying Framework**: Built using the `rmcp` (v2.0.0) runtime crate.
