# CLAUDE.md — bcinr Development Guide

**bcinr** (BranchlessCInRust v26.9.28) is a performance-first systems library with branchless algorithms, PDDL planning, POWL workflows, and cryptographic receipts. All primitives are O(1)/O(log n), deterministic, and side-channel resilient.

## Workspace Structure

```
bcinr/
├── crates/
│   ├── bcinr-logic/     # Core algorithms (300+ branchless implementations), no_std
│   ├── bcinr-cmca/      # CMCA numeric allocation engine (publish = false)
│   ├── bcinr-pddl/      # PDDL 3.1 planner + causal independence
│   ├── bcinr-powl/      # POWL runtime + receipt verification (BLAKE3)
│   │                    #   `receipt::` — folded in from bcinr-powl-receipt
│   ├── bcinr-mfw-ir/    # MFW intermediate representation
│   └── bcinr-guarded/   # Guarded execution (publish = false)
├── tools/               # Reporter, contract gate, bench auditor, cheat scanner, ggen
└── docs/                # Diátaxis documentation, release records (docs/releases/)
```

`bcinr-mcp`, `bcinr-api`, `bcinr-ffi`, `bcinr-bench` and `bcinr-pddl-lsp` were removed
from the workspace (see the comment in the root `Cargo.toml`); the MCP tool tables that
used to live here described `bcinr-mcp` and are gone with it.

## Core Principles

- **Deterministic:** All paths O(1/log n), branchless (no branch misprediction)
- **Memory-safe:** `#![forbid(unsafe_code)]` in algorithms; only 3 justified unsafe blocks
- **Zero-dependency:** `no_std` compatible
- **Cryptographic:** BLAKE3 receipts, Prolog8 admission gates

## Architecture

Derivation path: PDDL 3.1 domain → causal independence proof → POWL 2.0 decomposition
checked against the source net's language → branchless execution → BLAKE3 receipt.

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

**Before merge:** `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`

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

**Last Updated:** 2026-09-28 | **Version:** 26.9.28  
**Toolchain:** nightly (minimal profile) with MSRV 1.70  
**Test Status:** `cargo test --workspace` green as of v26.9.28  
**Unsafe Code:** 3 blocks (all proven safe, see SAFETY.md)
