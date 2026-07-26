# CLAUDE.md — bcinr Development Guide

**bcinr** (BranchlessCInRust v26.7.25) is a performance-first systems library with branchless algorithms, PDDL planning, POWL workflows, and cryptographic receipts. All primitives are O(1)/O(log n), deterministic, and side-channel resilient.

## Workspace Structure

```
bcinr/
├── bcinr-logic/         # Core algorithms (300+ branchless implementations)
├── bcinr-api/           # Additional API layer
├── bcinr-mcp/           # MCP server: 23 tools (PDDL, POWL, algorithms, receipts)
├── bcinr-pddl/          # PDDL 3.1 planner
├── bcinr-pddl-lsp/      # PDDL language server
├── bcinr-powl/          # POWL runtime (workflow compilation)
├── bcinr-powl-receipt/  # Receipt verification (BLAKE3)
├── tools/               # Utility tools
├── bcinr-bench/         # Benchmarks (Criterion)
└── docs/                # Diátaxis documentation
```

## Core Principles

- **Deterministic:** All paths O(1/log n), branchless (no branch misprediction)
- **Memory-safe:** `#![forbid(unsafe_code)]` in algorithms; only 3 justified unsafe blocks
- **Zero-dependency:** `no_std` compatible
- **Cryptographic:** BLAKE3 receipts, Prolog8 admission gates

## bcinr-mcp: Model Context Protocol Server

**23 tools** exposing entire bcinr ecosystem for Claude Code.

| Group | Count | Tools |
|-------|-------|-------|
| PDDL | 7 | `pddl_parse_domain`, `pddl_parse_problem`, `pddl_plan`, `pddl_admit_domain`, `manufacture_world` (+2) |
| POWL | 5 | `powl_compile_sequence`, `powl_compile_choice`, `powl_admit_context`, `powl_capability_check`, `powl_plan_to_tape` |
| Core | 3 | `bcinr_library_info`, `bcinr_mask_ops`, `bcinr_powl_info` |
| Algorithms | 6 | `utf8_validate`, `bitset_operations`, `dfa_info`, `scan_patterns`, `reduce_sequence`, `simd_string_info` |
| Receipts | 1 | `receipt_inspect` |
| Cross-crate | 1 | `system_capabilities` |

**Binary:** `/Users/sac/bcinr/target/debug/bcinr-mcp` (registered in `~/.claude/settings.json`)

**Tests:** `crates/bcinr-mcp/tests/integration_tests.rs` (18 dynamic tests, no hardcoded counts, 100% pass)

**Architecture:** Vision 2030 BRCE loop:
```
PDDL → Prolog8 gate → BFS plan → POWL tape → O(1) context → 
Branchless execute (UTF-8, bitset, DFA, scan, reduce, SIMD) → 
BLAKE3 receipt → receipt_inspect ✓
```

## Code Quality Standards

**Naming:** `PascalCase` types, `snake_case` functions, `UPPER_SNAKE_CASE` constants, `snake_case` modules.

**Documentation:** Public APIs require `/// examples`. Comments explain WHY (not WHAT). Inline code is self-documenting.

**Unsafe Code Policy:** `#![forbid(unsafe_code)]` enforced in algorithms. Only 3 justified unsafe blocks with Hoare-logic proofs:
- `mem.rs` — Memory arena bounds
- `autonomic/packed_key_table.rs` — Type-safe byte reinterpretation
- `patterns/deterministic_mpmc.rs` — Lock-free MPMC with CAS

See `crates/bcinr-logic/src/SAFETY.md` for full audit.

## Git Workflow

**Conventional commits:** `type(scope): description`
- `feat(mask)`, `fix(algorithms)`, `refactor(simd)`, `bench(bitset)`, `docs(PDDL)`, `test(...)`

**Before merge:** ✅ `make check` ✅ `make test` ✅ `make clippy` ✅ `make fmt`

## Common Tasks

**Add algorithm:** Create `crates/bcinr-logic/src/algorithms/new.rs`, write branchless implementation, add unit test in module, add benchmark in `bcinr-bench/`, document with examples, verify formally if safety-critical. Then: `make check && make test && make clippy && make fmt && git commit -m "feat(algorithms): ..."`

**Optimize algorithm:** Profile → identify bottleneck → implement → benchmark → commit with % improvement.

**Run specific test:** `cargo test -p bcinr-logic name -- --nocapture`

## Formal Verification & PhD Gates

This project includes Hoare-logic proofs (thesis & docs). **PhD Gates are NOT stubs** — they represent completed formal verification via Hoare-logic + proptest oracle matching. When modifying algorithms, preserve formal invariants and re-verify if claims change. See `phd_gates.md` for details.

## Dependencies & Supply Chain

**Zero runtime deps** in `bcinr-logic/` (security-critical). Dev deps: `criterion`, `proptest` (test only).

```bash
cargo make audit  # Check CVEs
cargo make deny   # License + supply chain
```

---

**Last Updated:** 2026-07-25 | **Version:** 26.7.25  
**Toolchain:** nightly (minimal profile) with MSRV 1.70  
**MCP Tools:** 23 (PDDL:7 + POWL:5 + core:3 + algo:6 + receipt:1 + xcrp:1)  
**Test Status:** 18/18 integration tests ✓  
**Unsafe Code:** 3 blocks (all proven safe, see SAFETY.md)
