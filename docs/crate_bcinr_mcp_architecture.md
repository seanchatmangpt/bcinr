# `bcinr-mcp` Architecture and Purpose

**Crate:** `bcinr-mcp`
**File Analyzed:** `crates/bcinr-mcp/src/main.rs`

## Purpose
The `bcinr-mcp` crate acts as a Model Context Protocol (MCP) server that exposes the entire functionality of the `bcinr` library as MCP tools over standard input/output (`stdio`). It effectively bridges the core deterministic routines of BCINR to external tools, LLMs, and agents that communicate via MCP, allowing them to access complex logic (PDDL, POWL) deterministically.

## Key Architectural Components

### 1. Tool Routing via `rmcp`
The crate leverages the `rmcp` library to define an MCP server. It uses a `BcinrServer` struct annotated with `#[tool_router(server_handler)]` to map async functions to tool calls. The server holds a `CapabilityCache` (from `cache.rs`) to memoize expensive computations like PDDL BFS planning results by caching their canonical serialized inputs.

### 2. Flexible Deserialization (`de_u64_flex`)
Because JSON inherently lacks a native 64-bit integer type (values > 2^53 often parse as `f64`), the crate includes a custom `de_u64_flex` deserializer. This prevents `invalid type: floating point, expected u64` errors by safely decoding strings or precision-safe floats back into strict `u64` values, which is necessary since LLMs often generate large numbers as strings.

### 3. Extensively Typed Tool Interfaces
The API is divided into distinct tool groups serving specific functional domains:

*   **Group 1 — PDDL:** Tools for reading domain info, parsing domains/problems, finding STRIPS plans, and running the `manufacture_world` loop. The latter validates models, plans, and yields a cryptographically verifiable (BLAKE3-chained) JSON receipt.
*   **Group 2 — POWL:** Provides tooling for compiling choices and sequences, checking capabilities, analyzing 64-bit schedules, and writing plans to tapes.
*   **Group 3 — Core bcinr:** Low-level core utilities (e.g., deterministic `bcinr_mask_ops`).
*   **Group 4 — bcinr-logic Algorithms:** Exposes low-level operations like `utf8_validate`, `bitset_operations`, DFA info, and SIMD string scanners.
*   **Group 5 & 6 — Receipts & Info:** Allows external inspection of POWL receipts and system capabilities.

### 4. Fail-Safe Boundary Logic
Adhering to BCINR's strict deterministic guidelines, the endpoints handle failure by returning JSON objects with `ok: false` and specific typed `refusal_code` / `error` fields, instead of panicking. For instance, `manufacture_world` does pre-validation of the `case_id` directly at the MCP boundary and cleanly blocks invalid characters before attempting to invoke the underlying library logic.
